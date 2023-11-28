//! Modulo que se centra en las funcionalidades referentes al mensaje de server quit.
use super::serializer::MessageSerializer;
use super::utils::generate_string;
use super::{generic_message::GenericMessage, Command, FromGeneric, MessageError};
use super::{Serializable, ServerExecutable};
use crate::irc::constants::ERR_NEEDMOREPARAMS;
use crate::irc::message::utils::{validate_command, validate_irc_params_len, validate_text};
use crate::irc::message::UNLIMITED_MAX_LEN;
use crate::irc::model::server::Server;
use crate::irc::model::MTServerConnection;
use crate::irc::responses::builder::ResponseBuilder;
use crate::irc::responses::{InternalType, ResponseType};
use crate::try_lock;

#[derive(Debug)]
pub struct ServerQuit<'a> {
    pub prefix: Option<&'a [u8]>,
    pub server: &'a [u8],
    pub message: Vec<&'a [u8]>,
}

impl<'a> FromGeneric<'a> for ServerQuit<'a> {
    fn from_generic(mut generic: GenericMessage<'a>) -> Result<Self, MessageError> {
        validate_command(generic.command, Command::ServerQuit)?;
        validate_irc_params_len(
            &generic.parameters,
            UNLIMITED_MAX_LEN,
            2,
            ERR_NEEDMOREPARAMS,
        )?;
        let server = match generic.parameters.pop_front() {
            Some(v) => v,
            None => return Err(MessageError::InvalidFormat),
        };
        let message = match validate_text(generic.parameters) {
            Ok(v) => v,
            Err(_) => return Err(MessageError::InvalidFormat),
        };

        Ok(Self {
            prefix: generic.prefix,
            server,
            message,
        })
    }
}

impl Serializable for ServerQuit<'_> {
    fn serialize(&self) -> String {
        MessageSerializer::new(self.prefix, Command::ServerQuit)
            .add_parameter(self.server)
            .add_trailing_params(&self.message)
            .serialize()
    }
}

impl ServerExecutable for ServerQuit<'_> {
    fn _execute_for_server(&self, _: &Server) -> Vec<ResponseType> {
        // Implements for semantic purposes
        ResponseBuilder::new().build()
    }

    fn execute_for_server(&self, server: &Server, origin: MTServerConnection) -> Vec<ResponseType> {
        let mut response = ResponseBuilder::new();
        let servername = generate_string(self.server);
        {
            if try_lock!(origin).servername == servername {
                response = response.add_internal_response(InternalType::Quit)
            }
        }

        self.replicate(server, origin);

        server.delete_server_by_name(&servername);

        response.build()
    }
}
#[cfg(test)]
mod server_quit_parse_tests {
    use super::*;
    use crate::irc::message::MessageError::IRCDefined;
    use crate::irc::message::MessageError::InvalidCommand;
    use std::collections::vec_deque::VecDeque;

    #[test]
    fn generic_message_with_different_command_error() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::User,
            prefix: None,
            parameters,
        };

        let err = ServerQuit::from_generic(generic).unwrap_err();

        assert_eq!(err, InvalidCommand);
    }

    #[test]
    fn generic_message_with_no_params_error() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::ServerQuit,
            prefix: None,
            parameters,
        };

        let err = ServerQuit::from_generic(generic).unwrap_err();

        assert_eq!(err, IRCDefined(ERR_NEEDMOREPARAMS));
    }
    #[test]
    fn test_valid_mssg_without_space() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b":mssg");
        parameters.push_front(b"a.b.c");
        let generic = GenericMessage {
            command: Command::ServerQuit,
            prefix: None,
            parameters,
        };

        let quit = ServerQuit::from_generic(generic).unwrap();
        let msg = quit.message;

        assert_eq!(msg[0], b"mssg");
        assert_eq!(quit.server, b"a.b.c");
    }

    #[test]
    fn test_valid_mssg_with_one_space() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b":ms sg");
        parameters.push_front(b"a.b.c");
        let generic = GenericMessage {
            command: Command::ServerQuit,
            prefix: None,
            parameters,
        };

        let quit = ServerQuit::from_generic(generic).unwrap();
        let msg = quit.message;

        assert_eq!(msg, vec![b"ms sg"]);

        assert_eq!(quit.server, b"a.b.c");
    }

    #[test]
    fn test_valid_mssg_with_two_spaces() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b":Gone to have lunch");
        parameters.push_front(b"a.b.c");
        let generic = GenericMessage {
            command: Command::ServerQuit,
            prefix: None,
            parameters,
        };

        let quit = ServerQuit::from_generic(generic).unwrap();
        let msg = quit.message;

        assert_eq!(msg, vec![b"Gone to have lunch"]);

        assert_eq!(quit.server, b"a.b.c");
    }

    #[test]
    fn test_valid_mssg_null() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::ServerQuit,
            prefix: None,
            parameters,
        };

        let err = ServerQuit::from_generic(generic).unwrap_err();

        assert_eq!(err, IRCDefined(ERR_NEEDMOREPARAMS));
    }
}
