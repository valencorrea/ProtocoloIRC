//! Modulo que se centra en las funcionalidades referentes al mensaje de nick.
use super::{
    generic_message::GenericMessage,
    serializer::MessageSerializer,
    utils::{
        generate_string, try_parse_number, validate_name_invalid_none, validate_name_valid_none,
    },
    Command, FromGeneric, MessageError, Replicable, Serializable, ServerExecutable,
};
use crate::irc::model::server::Server;
use crate::irc::{
    constants::ERR_NONICKNAMEGIVEN,
    responses::{builder::ResponseBuilder, ResponseType},
};
use crate::irc::{
    constants::RPL_NICKSET,
    message::utils::{validate_command, validate_irc_params_len},
    model::MTClient,
};
use crate::{irc::model::connection::Connection, try_lock};

#[derive(Debug)]
/// Struct del mensaje referido a nickname
/// Contiene un nickname representado por una referencia a vector u8,
/// un prefijo opcional representado por una referencia a vector u8,
/// y un hopcount opcional representado por un numero positivo,
pub struct Nickname<'a> {
    pub prefix: Option<&'a [u8]>,
    pub nickname: &'a [u8],
    pub hopcount: Option<u32>,
}

impl<'a> FromGeneric<'a> for Nickname<'a> {
    /// constructor de mensaje nickname a partir de un mensaje generico
    /// puede llegar a enviar un error si el comando no es nick,
    /// o si el largo de los parametros no es ni 1 ni 2
    fn from_generic(mut generic: GenericMessage<'a>) -> Result<Self, MessageError> {
        validate_command(generic.command, Command::Nick)?;
        validate_irc_params_len(&generic.parameters, 2, 1, ERR_NONICKNAMEGIVEN)?;
        validate_name_valid_none(generic.prefix)?;
        let nickname = validate_name_invalid_none(generic.parameters.pop_front())?;

        let hop = generic.parameters.pop_front();
        let mut hopcount: Option<u32> = None;

        if let Some(value) = hop {
            hopcount = Some(try_parse_number(value)?);
        }

        Ok(Self {
            prefix: generic.prefix,
            nickname,
            hopcount,
        })
    }
}

impl Serializable for Nickname<'_> {
    fn serialize(&self) -> String {
        let mut s = MessageSerializer::new(self.prefix, Command::Nick).add_parameter(self.nickname);

        if let Some(v) = self.hopcount {
            s = s.add_number(v);
        }

        s.serialize()
    }
}

impl Replicable for Nickname<'_> {
    fn _execute(&self, server: &Server, client: MTClient) -> (Vec<ResponseType>, bool) {
        let mut response = ResponseBuilder::new();
        let mut should_replicate = true;
        let nick = generate_string(self.nickname);
        let rmsg = format!("{} :You have a new nick", &nick);
        match server.change_nickname(client, nick) {
            Ok(_) => response = response.add_content_for_response(RPL_NICKSET, rmsg),
            Err(e) => {
                should_replicate = false;
                response = response.add_content_for_response(e.code, e.msg)
            }
        };

        (response.build(), should_replicate)
    }

    fn execute(&mut self, server: &Server, client: MTClient) -> Vec<ResponseType> {
        let old_nick = { try_lock!(client).nickname.to_owned() };
        let forward = format!(":{} {}", old_nick, self.serialize());
        let (response, should_replicate) = self._execute(server, client);
        if should_replicate {
            server.replicate_to_all_servers(&forward);
        }
        response
    }

    fn forward(&mut self, client: MTClient) -> String {
        let nick = { try_lock!(client).nickname.to_owned() };
        self.prefix = None;
        format!(":{} {}", nick, self.serialize())
    }
}

impl Nickname<'_> {
    pub fn execute_init(self, _server: &Server, connection: &mut Connection) -> Vec<ResponseType> {
        let mut response = ResponseBuilder::new();
        let nick = generate_string(self.nickname);
        let rmsg = format!("{} :You have a new nick", &nick);
        match connection.set_nickname(nick) {
            Ok(_) => {
                response = response.add_content_for_response(RPL_NICKSET, rmsg);
            }
            Err((code, msg)) => response = response.add_content_for_response(code, msg),
        };
        response.build()
    }
}

impl ServerExecutable for Nickname<'_> {
    fn _execute_for_server(&self, server: &Server) -> Vec<ResponseType> {
        let new_nick = generate_string(self.nickname);
        let old_nick = match self.prefix {
            Some(v) => generate_string(v),
            None => {
                // If no prefix is present, it means that a server is pairing new presenting a new user.
                server.add_data_client_by_nick(new_nick);
                return ResponseBuilder::new().build();
            }
        };

        if let Some(client) = server.get_client_by_nickname(&old_nick) {
            let _ = server.change_nickname(client, new_nick);
        };

        ResponseBuilder::new().build()
    }
}

#[cfg(test)]
mod nickname_parse_tests {
    use super::*;
    use crate::irc::constants::ERR_NONICKNAMEGIVEN;
    use std::collections::vec_deque::VecDeque;

    #[test]
    fn test_nickname_from_valid_generic_no_hopcount_no_prefix() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"newNick");
        let generic = GenericMessage {
            command: Command::Nick,
            prefix: None,
            parameters,
        };

        let nick = Nickname::from_generic(generic).unwrap();

        assert_eq!(nick.nickname, b"newNick");
        assert!(nick.prefix.is_none());
        assert!(nick.hopcount.is_none());
    }

    #[test]
    fn test_nick_from_valid_generic_no_hopcount_valid_prefix() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"newNick");
        let generic = GenericMessage {
            command: Command::Nick,
            prefix: Some(b"oldNick"),
            parameters,
        };

        let nick = Nickname::from_generic(generic).unwrap();

        assert_eq!(nick.nickname, b"newNick");
        assert_eq!(nick.prefix.unwrap(), b"oldNick");
        assert!(nick.hopcount.is_none());
    }

    #[test]
    fn test_nick_from_valid_generic_valid_hopcount_valid_prefix() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        let hopcount = b"32";
        parameters.push_front(hopcount);
        parameters.push_front(b"newNick");
        let generic = GenericMessage {
            command: Command::Nick,
            prefix: Some(b"oldNick"),
            parameters,
        };

        let nick = Nickname::from_generic(generic).unwrap();

        assert_eq!(nick.nickname, b"newNick");
        assert_eq!(nick.prefix.unwrap(), b"oldNick");
        assert_eq!(nick.hopcount.unwrap(), 32u32);
    }

    #[test]
    fn test_nickname_invalid_command() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"newNick");
        let generic = GenericMessage {
            command: Command::Password,
            prefix: None,
            parameters,
        };

        let err = Nickname::from_generic(generic).unwrap_err();

        assert_eq!(err, MessageError::InvalidCommand);
    }

    #[test]
    fn test_nickname_invalid_nickname() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"123");
        let generic = GenericMessage {
            command: Command::Nick,
            prefix: None,
            parameters,
        };

        let err = Nickname::from_generic(generic).unwrap_err();

        assert_eq!(err, MessageError::InvalidFormat);
    }

    #[test]
    fn test_nickname_invalid_prefix() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"valid");
        let generic = GenericMessage {
            command: Command::Nick,
            prefix: Some(b"123"),
            parameters,
        };

        let err = Nickname::from_generic(generic).unwrap_err();

        assert_eq!(err, MessageError::InvalidFormat);
    }

    #[test]
    fn test_nickname_invalid_hopcount() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"abcde");
        parameters.push_front(b"valid");
        let generic = GenericMessage {
            command: Command::Nick,
            prefix: Some(b"valid"),
            parameters,
        };

        let err = Nickname::from_generic(generic).unwrap_err();

        assert_eq!(err, MessageError::InvalidFormat);
    }

    #[test]
    fn test_nickname_too_much_parameters() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"abcde");
        parameters.push_front(b"abcde");
        parameters.push_front(b"valid");
        let generic = GenericMessage {
            command: Command::Nick,
            prefix: Some(b"valid"),
            parameters,
        };

        let err = Nickname::from_generic(generic).unwrap_err();

        assert_eq!(err, MessageError::TooManyParams);
    }

    #[test]
    fn test_nickname_too_few_parameters() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();
        let generic = GenericMessage {
            command: Command::Nick,
            prefix: None,
            parameters,
        };

        let err = Nickname::from_generic(generic).unwrap_err();

        assert_eq!(err, MessageError::IRCDefined(ERR_NONICKNAMEGIVEN));
    }
}
