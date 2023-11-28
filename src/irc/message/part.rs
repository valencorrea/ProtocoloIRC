//! Modulo que se centra en las funcionalidades referentes al mensaje de part.
use crate::irc::constants::{ERR_NEEDMOREPARAMS, RPL_CHANNELOUT, RPL_PART};
use crate::irc::message::utils::generate_string;
use crate::irc::model::server::Server;
use crate::irc::model::MTClient;
use crate::irc::responses::builder::ResponseBuilder;
use crate::irc::responses::ResponseType;
use crate::try_lock;

use super::{
    generic_message::GenericMessage,
    serializer::MessageSerializer,
    utils::{validate_channels, validate_command, validate_irc_params_len},
    Command, FromGeneric, MessageError, Serializable,
};
use super::{Replicable, ServerExecutable};

#[derive(Debug)]

pub struct Part<'a> {
    pub prefix: Option<&'a [u8]>,
    pub channels: Vec<&'a [u8]>,
}

impl<'a> FromGeneric<'a> for Part<'a> {
    fn from_generic(mut generic: GenericMessage<'a>) -> Result<Self, MessageError> {
        validate_command(generic.command, Command::Part)?;
        validate_irc_params_len(&generic.parameters, 1, 1, ERR_NEEDMOREPARAMS)?;

        let channels = validate_channels(generic.parameters.pop_front())?;

        Ok(Self {
            prefix: generic.prefix,
            channels,
        })
    }
}

impl Serializable for Part<'_> {
    fn serialize(&self) -> String {
        let s = MessageSerializer::new(self.prefix, Command::Part).add_csl_params(&self.channels);

        s.serialize()
    }
}

impl Replicable for Part<'_> {
    fn _execute(&self, server: &Server, client: MTClient) -> (Vec<ResponseType>, bool) {
        let mut res = ResponseBuilder::new();
        for channel in self.channels.iter() {
            let channel_name = generate_string(channel);
            match server.remove_client_from_channel(&channel_name, client.clone()) {
                Ok(_) => {
                    res = res.add_content_for_response(
                        RPL_PART,
                        format!("{} :Removed from channel", channel_name),
                    );

                    self.notify(server, &channel_name, client.clone());
                }
                Err(err) => res = res.add_content_for_response(err.code, err.msg),
            }
        }
        (res.build(), true)
    }

    fn forward(&mut self, client: MTClient) -> String {
        let nick = { try_lock!(client).nickname.to_owned() };
        self.prefix = None;
        format!(":{} {}", nick, self.serialize())
    }
}

impl Part<'_> {
    fn notify(&self, server: &Server, channel_name: &str, client: MTClient) {
        let client = { try_lock!(client).nickname.to_owned() };
        server.server_action_notify(&format!("{}: {} {}", RPL_CHANNELOUT, channel_name, client))
    }
}

impl ServerExecutable for Part<'_> {
    fn _execute_for_server(&self, server: &Server) -> Vec<ResponseType> {
        if let Some(prefix) = self.prefix {
            let nickname = generate_string(prefix);
            if let Some(client) = server.get_client_by_nickname(&nickname) {
                self._execute(server, client);
            }
        }

        ResponseBuilder::new().build()
    }
}

#[cfg(test)]
mod test {
    use std::collections::vec_deque::VecDeque;

    use super::*;

    #[test]
    fn test_valid_part() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();

        parameters.push_back(b"#twilight_zone");

        let generic = GenericMessage {
            command: Command::Part,
            prefix: None,
            parameters,
        };

        let part = Part::from_generic(generic).unwrap();

        assert_eq!(part.channels.len(), 1);
        assert_eq!(part.channels[0], b"#twilight_zone");
    }

    #[test]
    fn test_valid_part_multiple() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();

        parameters.push_back(b"#oz-ops,&group5");

        let generic = GenericMessage {
            command: Command::Part,
            prefix: None,
            parameters,
        };

        let part = Part::from_generic(generic).unwrap();

        assert_eq!(part.channels.len(), 2);
        assert_eq!(part.channels[0], b"#oz-ops");
        assert_eq!(part.channels[1], b"&group5");
    }

    #[test]
    fn test_too_many_params() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();

        parameters.push_back(b"#oz-ops,&group5");
        parameters.push_back(b"a");

        let generic = GenericMessage {
            command: Command::Part,
            prefix: None,
            parameters,
        };

        let part = Part::from_generic(generic).unwrap_err();

        assert_eq!(part, MessageError::TooManyParams);
    }

    #[test]
    fn test_too_few_paramas() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::Part,
            prefix: None,
            parameters,
        };

        let part = Part::from_generic(generic).unwrap_err();

        assert_eq!(part, MessageError::IRCDefined(ERR_NEEDMOREPARAMS));
    }

    #[test]
    fn test_err_cmd() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::PrivateMessage,
            prefix: None,
            parameters,
        };

        let part = Part::from_generic(generic).unwrap_err();

        assert_eq!(part, MessageError::InvalidCommand);
    }
}
