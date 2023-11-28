//! Modulo que se centra en las funcionalidades referentes al mensaje de private.
use std::collections::vec_deque::VecDeque;

use crate::irc::model::MTClient;
use crate::irc::{
    constants::ERR_NORECIPIENT,
    model::server::Server,
    responses::{builder::ResponseBuilder, ResponseType},
};
use crate::try_lock;

use super::utils::{generate_string_from_vec, validate_channel};
use super::{
    generic_message::GenericMessage,
    serializer::MessageSerializer,
    utils::{
        generate_string, split_csl, starts_with_colon, validate_command, validate_hostmask,
        validate_irc_params_len, validate_name_valid_none, validate_receivers, validate_text,
    },
    Command, FromGeneric, MessageError, ReceiverType, Serializable, HASH, UNLIMITED_MAX_LEN,
};
use super::{Replicable, ServerExecutable};

#[derive(Debug)]
pub struct Private<'a> {
    pub prefix: Option<&'a [u8]>,
    pub receivers: Vec<&'a [u8]>,
    pub text: Vec<&'a [u8]>,
}

impl<'a> FromGeneric<'a> for Private<'a> {
    fn from_generic(mut generic: GenericMessage<'a>) -> Result<Self, MessageError> {
        validate_command(generic.command, Command::PrivateMessage)?;
        validate_irc_params_len(&generic.parameters, UNLIMITED_MAX_LEN, 2, ERR_NORECIPIENT)?;

        let prefix = validate_name_valid_none(generic.prefix)?;
        let receivers = Self::extract_receivers(&mut generic.parameters)?;
        let text = validate_text(generic.parameters)?;

        Ok(Self {
            prefix,
            receivers,
            text,
        })
    }
}

impl Private<'_> {
    fn extract_receivers<'a>(
        params: &mut VecDeque<&'a [u8]>,
    ) -> Result<Vec<&'a [u8]>, MessageError> {
        let receivers = match params.pop_front() {
            Some(v) => {
                if v.is_empty() || starts_with_colon(v) {
                    return Err(MessageError::InvalidFormat);
                }
                v
            }
            None => return Err(MessageError::InvalidFormat),
        };

        let receivers_list = split_csl(Some(receivers))?;

        validate_receivers(&receivers_list)?;

        Ok(receivers_list)
    }

    pub fn receiver_type(receiver: &[u8]) -> ReceiverType {
        if validate_channel(Some(receiver)).is_ok() {
            return ReceiverType::ChannelName(generate_string(receiver));
        }

        if validate_hostmask(Some(receiver)).is_ok() {
            return {
                if receiver[0] == HASH {
                    ReceiverType::HostMask(generate_string(receiver))
                } else {
                    ReceiverType::ServerMask(generate_string(receiver))
                }
            };
        }
        ReceiverType::Nickname(generate_string(receiver))
    }
}

impl Serializable for Private<'_> {
    fn serialize(&self) -> String {
        let s = MessageSerializer::new(None, Command::PrivateMessage)
            .add_csl_params(&self.receivers)
            .add_trailing_params(&self.text);

        s.serialize()
    }
}

impl Private<'_> {
    fn __execute(
        &self,
        server: &Server,
        client: MTClient,
        mut auto_replicate: bool,
    ) -> (Vec<ResponseType>, bool) {
        let mut response = ResponseBuilder::new();
        let sender_nick = { try_lock!(client).nickname.to_owned() };
        let receivers = self.receivers.to_owned();
        let text = generate_string_from_vec(&self.text);

        let server_message = format!(":{} {}", sender_nick, self.serialize());

        for receiver in receivers {
            match Self::receiver_type(receiver) {
                ReceiverType::Nickname(nick) => {
                    let client_message = format!("{}: {}", sender_nick, text);

                    if let Err(e) = server.try_send_message_to_client(
                        &nick,
                        &client_message,
                        &server_message,
                        auto_replicate,
                    ) {
                        response = response.add_content_for_response(e.code, e.msg)
                    }
                }
                ReceiverType::ChannelName(channel_name) => {
                    let client_message = format!(
                        "{} {}: {}",
                        channel_name,
                        sender_nick,
                        generate_string_from_vec(&self.text)
                    );
                    if let Err(e) = server.try_send_message_to_channel(
                        client.clone(),
                        &channel_name,
                        &client_message,
                        &server_message,
                        auto_replicate,
                    ) {
                        response = response.add_content_for_response(e.code, e.msg)
                    }
                }
                _ => println!("TODO: handle masks"),
            }
            auto_replicate = false;
        }

        (response.build(), false)
    }
}

impl Replicable for Private<'_> {
    //To mantain semantic coherency, both Notice and Private Message are going to be Replicable, even though the replication strategy will be according to the message
    fn _execute(&self, server: &Server, client: MTClient) -> (Vec<ResponseType>, bool) {
        self.__execute(server, client, true)
    }

    fn execute(&mut self, server: &Server, client: MTClient) -> Vec<ResponseType> {
        self._execute(server, client).0
    }

    fn forward(&mut self, client: MTClient) -> String {
        let nick = { try_lock!(client).nickname.to_owned() };
        self.prefix = None;
        format!(":{} {}", nick, self.serialize())
    }
}

impl ServerExecutable for Private<'_> {
    fn _execute_for_server(&self, server: &Server) -> Vec<ResponseType> {
        if let Some(v) = self.prefix {
            if let Some(client) = server.get_client_by_nickname(&generate_string(v)) {
                return self.__execute(server, client, false).0;
            }
        }
        ResponseBuilder::new().build()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_valid_private() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"Wiz,jtotolsunoulufi,$*.fi");
        parameters.push_back(b":Start");
        parameters.push_back(b"of");
        parameters.push_back(b"the");
        parameters.push_back(b"message");

        let generic = GenericMessage {
            command: Command::PrivateMessage,
            prefix: None,
            parameters,
        };

        let pmsg = Private::from_generic(generic).unwrap();

        assert_eq!(pmsg.receivers.len(), 3);
        assert_eq!(pmsg.receivers[0], b"Wiz");
        assert_eq!(pmsg.receivers[1], b"jtotolsunoulufi");
        assert_eq!(pmsg.receivers[2], b"$*.fi");

        assert_eq!(pmsg.text.len(), 4);
        assert_eq!(pmsg.text[0], b"Start");
        assert_eq!(pmsg.text[1], b"of");
        assert_eq!(pmsg.text[2], b"the");
        assert_eq!(pmsg.text[3], b"message");
    }

    #[test]
    fn test_few_args() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"list,of,receivers");

        let generic = GenericMessage {
            command: Command::PrivateMessage,
            prefix: None,
            parameters,
        };

        let pmsg = Private::from_generic(generic).unwrap_err();

        assert_eq!(pmsg, MessageError::IRCDefined(ERR_NORECIPIENT))
    }

    #[test]
    fn test_invalid_hostmask() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"Wiz,jtotolsunoulufi,$*");
        parameters.push_back(b":Start");
        parameters.push_back(b"of");
        parameters.push_back(b"the");
        parameters.push_back(b"message");

        let generic = GenericMessage {
            command: Command::PrivateMessage,
            prefix: None,
            parameters,
        };

        let pmsg = Private::from_generic(generic).unwrap_err();

        assert_eq!(pmsg, MessageError::InvalidFormat)
    }

    #[test]
    fn test_invalid_command() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"Wiz,jto@tolsun.oulu.fi,$*.fi");
        parameters.push_back(b":Start");
        parameters.push_back(b"of");
        parameters.push_back(b"the");
        parameters.push_back(b"message");

        let generic = GenericMessage {
            command: Command::Nick,
            prefix: None,
            parameters,
        };

        let pmsg = Private::from_generic(generic).unwrap_err();

        assert_eq!(pmsg, MessageError::InvalidCommand)
    }

    #[test]
    fn test_valid_prefix() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"Wiz,jtotolsunoulufi,$*.fi");
        parameters.push_back(b":Start");
        parameters.push_back(b"of");
        parameters.push_back(b"the");
        parameters.push_back(b"message");

        let generic = GenericMessage {
            command: Command::PrivateMessage,
            prefix: Some(b"Angel"),
            parameters,
        };

        let pmsg = Private::from_generic(generic).unwrap();

        assert_eq!(pmsg.prefix.unwrap(), b"Angel");
    }

    #[test]
    fn test_invalid_prefix() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"Wiz,jtotolsunoulufi,$*.fi");
        parameters.push_back(b":Start");
        parameters.push_back(b"of");
        parameters.push_back(b"the");
        parameters.push_back(b"message");

        let generic = GenericMessage {
            command: Command::PrivateMessage,
            prefix: Some(b"123Angel"),
            parameters,
        };

        let pmsg = Private::from_generic(generic).unwrap_err();

        assert_eq!(pmsg, MessageError::InvalidFormat);
    }
}
