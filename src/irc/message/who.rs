//! Modulo que se centra en las funcionalidades referentes al mensaje de who.
use crate::irc::constants::{ERR_NEEDMOREPARAMS, RPL_ENDOFWHO, RPL_WHOREPLY};
use crate::irc::message::utils::{validate_command, validate_irc_params_len};
use crate::irc::message::GenericMessage;
use crate::irc::message::MessageError::InvalidFormat;
use crate::irc::message::{Command, FromGeneric, MessageError};
use crate::irc::model::server::Server;
use crate::irc::model::MTClient;
use crate::irc::responses::builder::ResponseBuilder;
use crate::irc::responses::ResponseType;

use super::serializer::MessageSerializer;
use super::utils::validate_o_param;
use super::{Executable, Serializable};

#[derive(Debug)]
pub struct Who<'a> {
    pub nick: Option<&'a [u8]>,
    pub o: bool,
}

impl<'a> FromGeneric<'a> for Who<'a> {
    fn from_generic(mut generic: GenericMessage<'a>) -> Result<Self, MessageError> {
        validate_command(generic.command, Command::Who)?;
        validate_irc_params_len(&generic.parameters, 2, 0, ERR_NEEDMOREPARAMS)?;

        if generic.parameters.is_empty() {
            return Ok(Self {
                nick: None,
                o: false,
            });
        }

        let nick = match generic.parameters.pop_front() {
            Some(v) => v,
            None => return Err(InvalidFormat),
        };
        let o = validate_o_param(generic.parameters.pop_front())?;

        Ok(Self {
            nick: Some(nick),
            o,
        })
    }
}

impl Serializable for Who<'_> {
    fn serialize(&self) -> String {
        let mut s = MessageSerializer::new(None, Command::Who);

        if let Some(v) = self.nick {
            s = s.add_parameter(v);
        }

        if self.o {
            s = s.add_parameter(&[111u8]);
        }

        s.serialize()
    }
}

impl Executable for Who<'_> {
    fn _execute(&self, server: &Server, client: MTClient) -> Vec<ResponseType> {
        // TODO: pattern matching for the parameters
        let mut response = ResponseBuilder::new();

        let users_str = server.describe_connected_clients(client);

        for us in users_str {
            response = response.add_content_for_response(RPL_WHOREPLY, us)
        }

        response = response.add_content_for_response(RPL_ENDOFWHO, "End of /WHO list".to_owned());

        response.build()
    }
}

#[cfg(test)]
mod who_parse_tests {
    use crate::irc::message::who::Who;
    use crate::irc::message::GenericMessage;
    use crate::irc::message::MessageError::InvalidCommand;
    use crate::irc::message::{Command, FromGeneric, MessageError};
    use std::collections::VecDeque;

    #[test]
    fn who_validate_command_error() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::User,
            prefix: None,
            parameters,
        };

        let err = Who::from_generic(generic).unwrap_err();

        assert_eq!(err, InvalidCommand);
    }

    #[test]
    fn who_with_many_parameters_error() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"param1");
        parameters.push_back(b"param2");
        parameters.push_back(b"param3");

        let generic = GenericMessage {
            command: Command::Who,
            prefix: None,
            parameters,
        };

        let err = Who::from_generic(generic).unwrap_err();

        assert_eq!(err, MessageError::TooManyParams);
    }

    #[test]
    fn who_with_no_parameters_ok() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::Who,
            prefix: None,
            parameters,
        };

        let who = Who::from_generic(generic).unwrap();

        assert!(who.nick.is_none());
    }

    #[test]
    fn test_valid_who() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"*name");
        parameters.push_back(b"o");

        let generic = GenericMessage {
            command: Command::Who,
            prefix: None,
            parameters,
        };

        let who = Who::from_generic(generic).unwrap();
        let nick = who.nick.unwrap();

        assert!(who.o);
        assert_eq!(nick, b"*name");
    }
    #[test]
    fn test_valid_who_no_o() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"*name");

        let generic = GenericMessage {
            command: Command::Who,
            prefix: None,
            parameters,
        };

        let who = Who::from_generic(generic).unwrap();
        let nick = who.nick.unwrap();

        assert!(!who.o);
        assert_eq!(nick, b"*name");
    }
    #[test]
    fn who_with_bad_parameter_o_error() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"asd");
        parameters.push_front(b"*a");

        let generic = GenericMessage {
            command: Command::Who,
            prefix: None,
            parameters,
        };

        let err = Who::from_generic(generic).unwrap_err();

        assert_eq!(err, MessageError::InvalidFormat);
    }
}
