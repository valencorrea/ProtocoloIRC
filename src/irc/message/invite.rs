//! Modulo que se centra en las funcionalidades referentes al mensaje de invite.
use crate::irc::constants::{ERR_NEEDMOREPARAMS, RPL_INVITING};
use crate::irc::message::utils::{
    generate_string, validate_channel, validate_command, validate_irc_params_len,
    validate_name_invalid_none, validate_realname_valid_none,
};
use crate::irc::message::{Command, Executable, FromGeneric, GenericMessage, MessageError};
use crate::irc::model::server::Server;
use crate::irc::model::MTClient;
use crate::irc::responses::builder::ResponseBuilder;
use crate::irc::responses::ResponseType;
use crate::try_lock;

use super::serializer::MessageSerializer;
use super::{Serializable, ServerExecutable};

#[derive(Debug)]
pub struct Invite<'a> {
    pub prefix: Option<&'a [u8]>,
    pub nickname: &'a [u8],
    pub channel: &'a [u8],
}

impl<'a> FromGeneric<'a> for Invite<'a> {
    fn from_generic(mut generic: GenericMessage<'a>) -> Result<Self, MessageError> {
        validate_command(generic.command, Command::Invite)?;
        validate_irc_params_len(&generic.parameters, 2, 2, ERR_NEEDMOREPARAMS)?;

        let nickname = validate_name_invalid_none(generic.parameters.pop_front())?;
        let channel = validate_channel(generic.parameters.pop_front())?;
        let prefix = validate_realname_valid_none(generic.prefix)?;

        Ok(Self {
            prefix,
            nickname,
            channel,
        })
    }
}

impl Serializable for Invite<'_> {
    fn serialize(&self) -> String {
        let s = MessageSerializer::new(None, Command::Invite)
            .add_parameter(self.nickname)
            .add_parameter(self.channel);

        s.serialize()
    }
}

impl Executable for Invite<'_> {
    fn _execute(&self, server: &Server, client: MTClient) -> Vec<ResponseType> {
        let mut response = ResponseBuilder::new();
        let nickname = generate_string(self.nickname);
        let channel_name = generate_string(self.channel);
        let rpl_msg = format!("{} {}", &channel_name, &nickname);

        match server.invite_to_channel(channel_name, nickname, client) {
            None => response = response.add_content_for_response(RPL_INVITING, rpl_msg),
            Some(err) => response = response.add_content_for_response(err.code, err.msg),
        };

        response.build()
    }
}

impl ServerExecutable for Invite<'_> {
    fn _execute_for_server(&self, server: &Server) -> Vec<ResponseType> {
        if let Some(prefix) = self.prefix {
            let nickname = generate_string(prefix);
            if let Some(client) = server.get_client_by_nickname(&nickname) {
                self._execute(server, client);
            }
        } else if let Some(client) = server.get_client_by_nickname(&generate_string(self.nickname))
        {
            try_lock!(client).add_invite(&generate_string(self.channel));
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
    fn test_invite_different_command_error() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::User,
            prefix: None,
            parameters,
        };

        let err = Invite::from_generic(generic).unwrap_err();

        assert_eq!(err, InvalidCommand);
    }

    #[test]
    fn test_invite_no_params_error() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::Invite,
            prefix: None,
            parameters,
        };

        let err = Invite::from_generic(generic).unwrap_err();

        assert_eq!(err, IRCDefined(ERR_NEEDMOREPARAMS));
    }

    #[test]
    fn test_invite_less_needed_params_error() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"nickname");

        let generic = GenericMessage {
            command: Command::Invite,
            prefix: None,
            parameters,
        };

        let err = Invite::from_generic(generic).unwrap_err();

        assert_eq!(err, IRCDefined(ERR_NEEDMOREPARAMS));
    }

    #[test]
    fn test_invite_more_needed_params_error() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"one param");
        parameters.push_front(b"two params");
        parameters.push_front(b"another param");
        parameters.push_front(b"one more param");

        let generic = GenericMessage {
            command: Command::Invite,
            prefix: None,
            parameters,
        };

        let err = Invite::from_generic(generic).unwrap_err();

        assert_eq!(err, TooManyParams);
    }

    #[test]
    fn test_invite_empty_channel_error() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"#Dust");
        parameters.push_front(b"Wiz");

        let generic = GenericMessage {
            command: Command::Invite,
            prefix: Some(b":Angel"),
            parameters,
        };

        let invite = Invite::from_generic(generic).unwrap();

        assert_eq!(invite.prefix.unwrap(), b"Angel");
        assert_eq!(invite.nickname, b"Wiz");
        assert_eq!(invite.channel, b"#Dust");
    }

    #[test]
    fn test_invite_with_prefix_ok() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"");
        parameters.push_front(b"nickname");

        let generic = GenericMessage {
            command: Command::Invite,
            prefix: None,
            parameters,
        };

        let err = Invite::from_generic(generic).unwrap_err();

        assert_eq!(err, InvalidFormat);
    }
}
