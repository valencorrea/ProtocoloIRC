//! Modulo que se centra en las funcionalidades referentes al mensaje de away
use crate::irc::constants::{ERR_NEEDMOREPARAMS, RPL_NOAWAY, RPL_UNAWAY};
use crate::irc::message::utils::{
    validate_command, validate_irc_params_len, validate_name_valid_none, validate_text,
};
use crate::irc::message::GenericMessage;
use crate::irc::message::{Command, FromGeneric, MessageError, UNLIMITED_MAX_LEN};
use crate::irc::model::server::Server;
use crate::irc::model::MTClient;
use crate::irc::responses::builder::ResponseBuilder;
use crate::irc::responses::ResponseType;
use crate::try_lock;

use super::serializer::MessageSerializer;
use super::utils::{generate_string, generate_string_from_vec};
use super::{Replicable, Serializable, ServerExecutable};

#[derive(Debug)]
pub struct Away<'a> {
    pub prefix: Option<&'a [u8]>,
    pub message: Option<Vec<&'a [u8]>>,
}

impl<'a> FromGeneric<'a> for Away<'a> {
    fn from_generic(generic: GenericMessage<'a>) -> Result<Self, MessageError> {
        validate_command(generic.command, Command::Away)?;
        validate_irc_params_len(
            &generic.parameters,
            UNLIMITED_MAX_LEN,
            0,
            ERR_NEEDMOREPARAMS,
        )?;

        let prefix = validate_name_valid_none(generic.prefix)?;

        let mut message = None;

        if !generic.parameters.is_empty() {
            message = Some(validate_text(generic.parameters)?);
        };

        Ok(Self { prefix, message })
    }
}

impl Serializable for Away<'_> {
    fn serialize(&self) -> String {
        let mut s = MessageSerializer::new(self.prefix, Command::Away);

        if let Some(msg) = &self.message {
            s = s.add_trailing_params(msg);
        }

        s.serialize()
    }
}

impl Replicable for Away<'_> {
    fn _execute(&self, server: &Server, client: MTClient) -> (Vec<ResponseType>, bool) {
        let mut response = ResponseBuilder::new();

        match &self.message {
            Some(msg) => {
                let away_message = generate_string_from_vec(msg);
                server.set_client_away(client, away_message);
                response = response.add_content_for_response(
                    RPL_NOAWAY,
                    ":You have been marked as being away".to_owned(),
                );
            }
            None => {
                server.unset_client_away(client);
                response = response.add_content_for_response(
                    RPL_UNAWAY,
                    "You are no longed marked as being away".to_owned(),
                );
            }
        };

        (response.build(), true)
    }

    fn forward(&mut self, client: MTClient) -> String {
        let nick = { try_lock!(client).nickname.to_owned() };
        self.prefix = None;
        format!(":{} {}", nick, self.serialize())
    }
}

impl ServerExecutable for Away<'_> {
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
mod away_tests {
    use crate::irc::message::away::Away;
    use crate::irc::message::GenericMessage;
    use crate::irc::message::MessageError::InvalidCommand;
    use crate::irc::message::{Command, FromGeneric};
    use std::collections::VecDeque;

    #[test]
    fn generic_message_with_different_command_error() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::User,
            prefix: None,
            parameters,
        };

        let err = Away::from_generic(generic).unwrap_err();

        assert_eq!(err, InvalidCommand);
    }

    #[test]
    fn generic_message_with_no_params_ok() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::Away,
            prefix: None,
            parameters,
        };

        let away = Away::from_generic(generic).unwrap();

        assert_eq!(away.prefix, None);
        assert_eq!(away.message, None);
    }

    #[test]
    fn generic_message_with_prefix_ok() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::Away,
            prefix: Some(b"Wiz"),
            parameters,
        };

        let away = Away::from_generic(generic).unwrap();

        assert_eq!(away.prefix.unwrap(), b"Wiz");
        assert_eq!(away.message, None);
    }

    #[test]
    fn generic_message_with_comment_ok() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b":this");
        parameters.push_back(b"is");
        parameters.push_back(b"an");
        parameters.push_back(b"away");
        parameters.push_back(b"message");

        let generic = GenericMessage {
            command: Command::Away,
            prefix: None,
            parameters,
        };

        let away = Away::from_generic(generic).unwrap();
        let away_message = away.message.unwrap();

        assert_eq!(away.prefix, None);
        assert_eq!(away_message.len(), 5);
        assert_eq!(away_message[0], b"this");
        assert_eq!(away_message[1], b"is");
        assert_eq!(away_message[2], b"an");
        assert_eq!(away_message[3], b"away");
        assert_eq!(away_message[4], b"message");
    }

    #[test]
    fn generic_message_with_prefix_and_comment_ok() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b":this");
        parameters.push_back(b"is");
        parameters.push_back(b"an");
        parameters.push_back(b"away");
        parameters.push_back(b"message");

        let generic = GenericMessage {
            command: Command::Away,
            prefix: Some(b"Wiz"),
            parameters,
        };

        let away = Away::from_generic(generic).unwrap();
        let away_message = away.message.unwrap();

        assert_eq!(away.prefix.unwrap(), b"Wiz");
        assert_eq!(away_message.len(), 5);
        assert_eq!(away_message[0], b"this");
        assert_eq!(away_message[1], b"is");
        assert_eq!(away_message[2], b"an");
        assert_eq!(away_message[3], b"away");
        assert_eq!(away_message[4], b"message");
    }
}
