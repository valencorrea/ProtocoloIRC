//! Modulo que se centra en las funcionalidades referentes al mensaje de join.
use crate::irc::constants::{RPL_CHANNELMODEIS, RPL_ENDOFNAMES};
use crate::irc::message::utils::validate_realname_valid_none;
use crate::{
    irc::{
        constants::{ERR_NEEDMOREPARAMS, RPL_NAMREPLY},
        message::utils::{validate_command, validate_irc_params_len},
        model::{client::Client, server::Server, MTClient},
        responses::{builder::ResponseBuilder, ResponseType},
    },
    try_lock,
};
use std::sync::{Arc, Mutex};

use super::{
    generic_message::GenericMessage,
    serializer::MessageSerializer,
    topic::topic_response,
    utils::{generate_string, split_csl_none, validate_channels},
    Command, FromGeneric, MessageError, Replicable, Serializable, ServerExecutable,
};

#[derive(Debug)]
/// Struct del mensaje referido a join
/// Contiene un prefijo opcional representado por una referencia u8,
/// una lista de claves opcional representado por una referencia u8,
/// y una lista de canales representado por una referencia a vector u8,
pub struct Join<'a> {
    pub prefix: Option<&'a [u8]>,
    pub keys: Option<Vec<&'a [u8]>>,
    pub channels: Vec<&'a [u8]>,
}

impl<'a> FromGeneric<'a> for Join<'a> {
    /// constructor de mensaje join a partir de un mensaje generico
    /// puede llegar a enviar un error si el comando no es join,
    /// o si el largo de los parametros no es mayor o igual a 1
    fn from_generic(mut generic: GenericMessage<'a>) -> Result<Self, MessageError> {
        validate_command(generic.command, Command::Join)?;
        validate_irc_params_len(&generic.parameters, 2, 1, ERR_NEEDMOREPARAMS)?;

        let prefix = validate_realname_valid_none(generic.prefix)?;
        let channels = validate_channels(generic.parameters.pop_front())?;
        let keys = split_csl_none(generic.parameters.pop_front());

        match &keys {
            Some(k) => {
                if k.len() > channels.len() {
                    return Err(MessageError::InvalidFormat);
                }
            }
            None => (),
        };
        Ok(Self {
            prefix,
            channels,
            keys,
        })
    }
}

impl Serializable for Join<'_> {
    fn serialize(&self) -> String {
        let mut s =
            MessageSerializer::new(self.prefix, Command::Join).add_csl_params(&self.channels);

        if let Some(k) = &self.keys {
            s = s.add_csl_params(k);
        }

        s.serialize()
    }
}

impl Join<'_> {
    fn iterate_channels(
        &self,
        for_client: bool,
        server: &Server,
        client: Arc<Mutex<Client>>,
    ) -> Vec<(usize, String)> {
        let mut res = vec![];
        let keys: Vec<&[u8]> = match &self.keys {
            Some(k) => k.to_vec(),
            None => Vec::new(),
        };

        for (index, ch) in self.channels.iter().enumerate() {
            let pwd = keys.get(index).map(|key| generate_string(key));
            let channel_name = generate_string(ch);
            if for_client {
                res.append(&mut self.client_side_join(&channel_name, pwd, server, client.clone()));
            } else {
                res.append(&mut self.server_side_join(&channel_name, pwd, server, client.clone()));
            }
        }

        res
    }

    fn client_side_join(
        &self,
        channel_name: &str,
        pwd: Option<String>,
        server: &Server,
        client: Arc<Mutex<Client>>,
    ) -> Vec<(usize, String)> {
        let mut res = vec![];
        match server.join_client_to_channel(channel_name, pwd, client.clone()) {
            Ok(channel) => {
                let clients = server.get_clients_for_channel(channel.clone(), client.clone());
                for channel_client in clients {
                    res.push((RPL_NAMREPLY, format!("{} {}", channel_name, channel_client)));
                }
                res.push((RPL_ENDOFNAMES, "End of /NAMES list".to_owned()));

                res.push(topic_response(channel_name, server.get_topic(channel_name)));

                res.push((RPL_CHANNELMODEIS, server.describe_channel_modes(channel)));

                self.notify(server, channel_name, client);
            }
            Err(e) => res.push((e.code, e.msg)),
        };
        res
    }

    fn server_side_join(
        &self,
        channel_name: &str,
        pwd: Option<String>,
        server: &Server,
        client: Arc<Mutex<Client>>,
    ) -> Vec<(usize, String)> {
        if let Some(channel) = server.get_channel_by_name(channel_name) {
            server.add_client_to_channel(client.clone(), channel);
        } else {
            server.create_channel(channel_name.to_owned(), pwd, Some(client.clone()));
        }
        self.notify(server, channel_name, client);
        vec![(0, "a".to_owned())] //ignore
    }

    fn notify(&self, server: &Server, channel_name: &str, joiner: MTClient) {
        let joiner_nick = { try_lock!(joiner).nickname.to_owned() };
        server.server_action_notify(&format!(
            "{}: {} {}",
            RPL_NAMREPLY, channel_name, joiner_nick
        ))
    }
}

impl Replicable for Join<'_> {
    fn _execute(&self, server: &Server, client: MTClient) -> (Vec<ResponseType>, bool) {
        let mut responseb = ResponseBuilder::new();

        let responses = self.iterate_channels(true, server, client);

        for response in responses {
            responseb = responseb.add_content_for_response(response.0, response.1);
        }

        (responseb.build(), true)
    }

    fn forward(&mut self, client: MTClient) -> String {
        let nick = { try_lock!(client).nickname.to_owned() };
        self.prefix = None;
        format!(":{} {}", nick, self.serialize())
    }
}

impl ServerExecutable for Join<'_> {
    fn _execute_for_server(&self, server: &Server) -> Vec<ResponseType> {
        if let Some(prefix) = self.prefix {
            let nickname = generate_string(prefix);
            if let Some(client) = server.get_client_by_nickname(&nickname) {
                let _ = self.iterate_channels(false, server, client);
            }
        }
        ResponseBuilder::new().build()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::irc::message::MessageError::{
        IRCDefined, InvalidCommand, InvalidFormat, TooManyParams,
    };
    use std::collections::vec_deque::VecDeque;

    #[test]
    fn test_valid_join_message_without_prefix() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"pass"); //password
        parameters.push_front(b"#channel"); //channel

        let generic = GenericMessage {
            command: Command::Join,
            prefix: None,
            parameters,
        };

        let join = Join::from_generic(generic).unwrap();

        assert_eq!(join.channels[0], b"#channel");
        assert_eq!(join.keys.unwrap()[0], b"pass");
    }

    #[test]
    fn test_valid_join_message_with_two_channels() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"pass"); //password
        parameters.push_front(b"#channel,#channel1"); //channel

        let generic = GenericMessage {
            command: Command::Join,
            prefix: None,
            parameters,
        };

        let join = Join::from_generic(generic).unwrap();

        assert_eq!(join.channels[0], b"#channel");
        assert_eq!(join.channels[1], b"#channel1");
        assert_eq!(join.keys.unwrap()[0], b"pass");
    }

    #[test]
    fn test_valid_join_message_with_prefix() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"pass"); //password
        parameters.push_front(b"#channel"); //channel

        let generic = GenericMessage {
            command: Command::Join,
            prefix: Some(b":testnick"),
            parameters,
        };

        let join = Join::from_generic(generic).unwrap();

        assert_eq!(join.channels[0], b"#channel");
        assert_eq!(join.keys.unwrap()[0], b"pass");
        assert_eq!(join.prefix.unwrap(), b"testnick");
    }

    #[test]
    fn test_join_invalid_command() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"pass"); //password
        parameters.push_front(b"#channel"); //channel

        let generic = GenericMessage {
            command: Command::Password,
            prefix: None,
            parameters,
        };

        let err = Join::from_generic(generic).unwrap_err();

        assert_eq!(err, InvalidCommand);
    }

    #[test]
    fn test_join_invalid_keys() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"pass,pass1"); //password
        parameters.push_front(b"#channel"); //channel

        let generic = GenericMessage {
            command: Command::Join,
            prefix: None,
            parameters,
        };

        let err = Join::from_generic(generic).unwrap_err();

        assert_eq!(err, InvalidFormat);
    }

    #[test]
    fn test_join_too_much_parameters() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"pass"); //password
        parameters.push_front(b"#channel"); //channel
        parameters.push_front(b"pass"); //password
        parameters.push_front(b"#channel"); //channel

        let generic = GenericMessage {
            command: Command::Join,
            prefix: None,
            parameters,
        };

        let err = Join::from_generic(generic).unwrap_err();

        assert_eq!(err, TooManyParams);
    }

    #[test]
    fn test_join_too_few_parameters() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::Join,
            prefix: None,
            parameters,
        };

        let err = Join::from_generic(generic).unwrap_err();

        assert_eq!(err, IRCDefined(ERR_NEEDMOREPARAMS));
    }
}
