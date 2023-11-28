//! Modulo que se centra en las funcionalidades referentes al mensaje de notice.
use crate::irc::constants::{ERR_NOSUCHNICK, ERR_NOTEXTTOSEND, RPL_AWAY};
use crate::irc::message::utils::{
    validate_command, validate_irc_params_len, validate_name_invalid_none,
};
use crate::irc::message::GenericMessage;
use crate::irc::message::{Command, FromGeneric, MessageError};
use crate::irc::model::server::Server;
use crate::irc::model::MTClient;
use crate::irc::responses::builder::ResponseBuilder;
use crate::irc::responses::ResponseType;
use crate::try_lock;

use super::utils::{generate_string, generate_string_from_vec, validate_text};
use super::{private::Private, serializer::MessageSerializer};
use super::{ReceiverType, Replicable, Serializable, ServerExecutable, UNLIMITED_MAX_LEN};

#[derive(Debug)]
pub struct Notice<'a> {
    pub prefix: Option<&'a [u8]>,
    pub nickname: &'a [u8],
    pub text: Vec<&'a [u8]>,
}

impl<'a> FromGeneric<'a> for Notice<'a> {
    fn from_generic(mut generic: GenericMessage<'a>) -> Result<Self, MessageError> {
        validate_command(generic.command, Command::Notice)?;
        validate_irc_params_len(&generic.parameters, UNLIMITED_MAX_LEN, 2, ERR_NOTEXTTOSEND)?;
        let nickname = validate_name_invalid_none(generic.parameters.pop_front())?;
        let text = validate_text(generic.parameters)?;

        Ok(Self {
            prefix: generic.prefix,
            nickname,
            text,
        })
    }
}

impl Serializable for Notice<'_> {
    fn serialize(&self) -> String {
        let s = MessageSerializer::new(None, Command::Notice)
            .add_parameter(self.nickname)
            .add_trailing_params(&self.text);

        s.serialize()
    }
}

impl Replicable for Notice<'_> {
    //To mantain semantic coherency, both Notice and Private Message are going to be Replicable, even though the replication strategy will be according to the message

    fn _execute(&self, server: &Server, client: MTClient) -> (Vec<ResponseType>, bool) {
        let mut response = ResponseBuilder::new();
        let mut should_replicate = true;
        let sender_nick = { try_lock!(client).nickname.to_owned() };
        let nosuchnick = format!("{} :No such nick", generate_string(self.nickname));

        match Private::receiver_type(self.nickname) {
            ReceiverType::Nickname(nick) => {
                let client_message =
                    format!("{}: {}", sender_nick, generate_string_from_vec(&self.text));
                let server_message = format!(":{} {}", sender_nick, self.serialize());
                if let Err(e) =
                    server.try_send_message_to_client(&nick, &client_message, &server_message, true)
                {
                    if e.code != RPL_AWAY {
                        should_replicate = false;
                        response = response.add_content_for_response(e.code, e.msg)
                    }
                }
            }
            _ => {
                should_replicate = false;
                response = response.add_content_for_response(ERR_NOSUCHNICK, nosuchnick)
            }
        };
        (response.build(), should_replicate)
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

impl ServerExecutable for Notice<'_> {
    fn _execute_for_server(&self, server: &Server) -> Vec<ResponseType> {
        if let Some(v) = self.prefix {
            if let Some(client) = server.get_client_by_nickname(&generate_string(v)) {
                return self._execute(server, client).0;
            }
        }
        ResponseBuilder::new().build()
    }
}

#[cfg(test)]
mod notice_parse_tests {
    use super::*;
    use crate::irc::message::MessageError;
    use std::collections::vec_deque::VecDeque;

    #[test]
    fn test_notice_valid_parameters_ok() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"nickname");
        parameters.push_back(b":text");

        let generic = GenericMessage {
            command: Command::Notice,
            prefix: None,
            parameters,
        };

        let notice = Notice::from_generic(generic).unwrap();

        assert_eq!(notice.nickname, b"nickname");
        assert_eq!(notice.text[0], b"text");
    }

    #[test]
    fn test_notice_text_with_spaces_ok() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"nickname");
        parameters.push_back(b":this");
        parameters.push_back(b"is");
        parameters.push_back(b"a");
        parameters.push_back(b"text");

        let generic = GenericMessage {
            command: Command::Notice,
            prefix: None,
            parameters,
        };

        let notice = Notice::from_generic(generic).unwrap();

        assert_eq!(notice.nickname, b"nickname");
        assert_eq!(notice.text[0], b"this");
        assert_eq!(notice.text[1], b"is");
        assert_eq!(notice.text[2], b"a");
        assert_eq!(notice.text[3], b"text");
    }

    #[test]
    fn test_notice_no_params_error() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::Notice,
            prefix: None,
            parameters,
        };

        let err = Notice::from_generic(generic).unwrap_err();

        assert_eq!(err, MessageError::IRCDefined(ERR_NOTEXTTOSEND));
    }

    #[test]
    fn test_notice_less_needed_params_error() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"nickname");

        let generic = GenericMessage {
            command: Command::Notice,
            prefix: None,
            parameters,
        };

        let err = Notice::from_generic(generic).unwrap_err();

        assert_eq!(err, MessageError::IRCDefined(ERR_NOTEXTTOSEND));
    }

    #[test]
    fn test_no_text_to_send() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"one");
        parameters.push_back(b"two");
        parameters.push_back(b"three");
        parameters.push_back(b"four");

        let generic = GenericMessage {
            command: Command::Notice,
            prefix: None,
            parameters,
        };

        let err = Notice::from_generic(generic).unwrap_err();

        assert_eq!(err, MessageError::InvalidFormat);
    }
}
