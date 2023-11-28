//! Modulo que se centra en las funcionalidades referentes al mensaje de whois.
use crate::irc::constants::{
    ERR_NONICKNAMEGIVEN, RPL_ENDOFWHOIS, RPL_WHOISCHANNELS, RPL_WHOISOPERATOR, RPL_WHOISUSER,
};
use crate::irc::message::utils::{validate_command, validate_irc_params_len};
use crate::irc::message::GenericMessage;
use crate::irc::message::{Command, FromGeneric, MessageError};
use crate::irc::model::server::Server;
use crate::irc::model::MTClient;
use crate::irc::responses::builder::ResponseBuilder;
use crate::irc::responses::ResponseType;

use super::serializer::MessageSerializer;
use super::utils::{generate_string, validate_hostname, validate_nickmasks};
use super::{Executable, Serializable};

#[derive(Debug)]
pub struct Whois<'a> {
    pub server: Option<&'a [u8]>,
    pub nickmask: Vec<&'a [u8]>,
}

impl<'a> FromGeneric<'a> for Whois<'a> {
    fn from_generic(mut generic: GenericMessage<'a>) -> Result<Self, MessageError> {
        validate_command(generic.command, Command::WhoIs)?;
        validate_irc_params_len(&generic.parameters, 2, 1, ERR_NONICKNAMEGIVEN)?;

        let mut server = None;
        let nickmask;

        if generic.parameters.len() == 2 {
            server = Some(validate_hostname(generic.parameters.pop_front())?);
            nickmask = validate_nickmasks(generic.parameters.pop_front())?;
        } else {
            nickmask = validate_nickmasks(generic.parameters.pop_front())?;
        }

        Ok(Self { nickmask, server })
    }
}

impl Serializable for Whois<'_> {
    fn serialize(&self) -> String {
        let mut s = MessageSerializer::new(None, Command::WhoIs);
        if let Some(server) = self.server {
            s = s.add_parameter(server);
        }

        s = s.add_csl_params(&self.nickmask);

        s.serialize()
    }
}

impl Executable for Whois<'_> {
    fn _execute(&self, server: &Server, client: MTClient) -> Vec<ResponseType> {
        let mut response = ResponseBuilder::new();
        let nicks: Vec<String> = self
            .nickmask
            .iter()
            .map(|nick| generate_string(nick))
            .collect();

        for nick in nicks {
            match server.describe_full_user_info(&nick, client.clone()) {
                Ok(user_info) => {
                    response = response.add_content_for_response(RPL_WHOISUSER, user_info.user);
                    if let Some(oper) = user_info.oper {
                        response = response.add_content_for_response(RPL_WHOISOPERATOR, oper);
                    }

                    for channel in user_info.channels {
                        response = response.add_content_for_response(RPL_WHOISCHANNELS, channel);
                    }

                    response = response.add_content_for_response(RPL_ENDOFWHOIS, user_info.end);
                }
                Err(err) => {
                    response = response.add_content_for_response(err.code, err.msg);
                }
            }
        }

        response.build()
    }
}

#[cfg(test)]
mod whois_parse_tests {
    use crate::irc::constants::ERR_NONICKNAMEGIVEN;
    use crate::irc::message::whois::Whois;
    use crate::irc::message::GenericMessage;
    use crate::irc::message::MessageError::InvalidCommand;
    use crate::irc::message::{Command, FromGeneric, MessageError};
    use std::collections::VecDeque;

    #[test]
    fn whois_validate_command_error() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::User,
            prefix: None,
            parameters,
        };

        let err = Whois::from_generic(generic).unwrap_err();

        assert_eq!(err, InvalidCommand);
    }

    #[test]
    fn whois_with_many_parameters_error() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"param1");
        parameters.push_back(b"param2");
        parameters.push_back(b"param3");
        parameters.push_back(b"param4");
        parameters.push_back(b"param5");
        parameters.push_back(b"param6");
        parameters.push_back(b"param7");

        let generic = GenericMessage {
            command: Command::WhoIs,
            prefix: None,
            parameters,
        };

        let err = Whois::from_generic(generic).unwrap_err();

        assert_eq!(err, MessageError::TooManyParams);
    }

    #[test]
    fn whois_with_no_parameters_error() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::WhoIs,
            prefix: None,
            parameters,
        };

        let whois = Whois::from_generic(generic).unwrap_err();

        assert_eq!(whois, MessageError::IRCDefined(ERR_NONICKNAMEGIVEN));
    }

    #[test]
    fn whois_with_one_nick_and_servername_ok() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"servername");
        parameters.push_back(b"nickname");

        let generic = GenericMessage {
            command: Command::WhoIs,
            prefix: None,
            parameters,
        };

        let whois = Whois::from_generic(generic).unwrap();
        let nicks = whois.nickmask;

        assert_eq!(whois.server.unwrap(), b"servername");
        assert_eq!(nicks[0], b"nickname");
        assert_eq!(nicks.len(), 1);
    }

    #[test]
    fn whois_with_many_nicks_and_servername_ok() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"servername");
        parameters.push_back(b"nickname1,nickname2,nickname3,nickname4,nickname5");

        let generic = GenericMessage {
            command: Command::WhoIs,
            prefix: None,
            parameters,
        };

        let whois = Whois::from_generic(generic).unwrap();
        let nicks = whois.nickmask;

        assert_eq!(whois.server.unwrap(), b"servername");
        assert_eq!(nicks[0], b"nickname1");
        assert_eq!(nicks[1], b"nickname2");
        assert_eq!(nicks[2], b"nickname3");
        assert_eq!(nicks[3], b"nickname4");
        assert_eq!(nicks[4], b"nickname5");
        assert_eq!(nicks.len(), 5);
    }

    #[test]
    fn whois_with_no_ascii_alphabetic_servername_error() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"1@servername");
        parameters.push_back(b"nickname");

        let generic = GenericMessage {
            command: Command::WhoIs,
            prefix: None,
            parameters,
        };

        let err = Whois::from_generic(generic).unwrap_err();

        assert_eq!(err, MessageError::InvalidFormat);
    }
}
