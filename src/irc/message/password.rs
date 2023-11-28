//! Modulo que se centra en las funcionalidades referentes al mensaje de password.
use crate::irc::constants::{ERR_ALREADYREGISTRED, ERR_NEEDMOREPARAMS, RPL_PWDSET};
use crate::irc::message::utils::{validate_command, validate_irc_params_len, validate_password};
use crate::irc::message::Executable;
use crate::irc::model::connection::Connection;
use crate::irc::model::server::Server;
use crate::irc::model::MTClient;
use crate::irc::responses::builder::ResponseBuilder;
use crate::irc::responses::ResponseType;

use super::serializer::MessageSerializer;
use super::utils::generate_string;
use super::Serializable;
use super::{generic_message::GenericMessage, Command, FromGeneric, MessageError};

#[derive(Debug)]
/// Struct del mensaje referido a password
/// Contiene un prefijo opcional representado por una referencia a vector u8,
/// y una contrasenia representada por una referencia a vector u8,
pub struct Password<'a> {
    pub prefix: Option<&'a [u8]>,
    pub password: &'a [u8],
}

impl<'a> FromGeneric<'a> for Password<'a> {
    /// constructor de mensaje oper a partir de un mensaje generico
    /// puede llegar a enviar un error si el comando no es password,
    /// o si el largo de los parametros no es 1
    fn from_generic(mut generic: GenericMessage<'a>) -> Result<Self, MessageError> {
        validate_command(generic.command, Command::Password)?;
        validate_irc_params_len(&generic.parameters, 1, 1, ERR_NEEDMOREPARAMS)?;
        let password = validate_password(generic.parameters.pop_front())?;

        Ok(Self {
            prefix: generic.prefix,
            password,
        })
    }
}

impl Serializable for Password<'_> {
    fn serialize(&self) -> String {
        let s = MessageSerializer::new(self.prefix, Command::Password).add_parameter(self.password);

        s.serialize()
    }
}

impl Executable for Password<'_> {
    fn _execute(&self, _server: &Server, _client: MTClient) -> Vec<ResponseType> {
        ResponseBuilder::new()
            .add_content_for_response(ERR_ALREADYREGISTRED, "You may not register".to_owned())
            .build()
    }
}

impl Password<'_> {
    pub fn execute_init(self, _server: &Server, connection: &mut Connection) -> Vec<ResponseType> {
        let response = ResponseBuilder::new();
        let pwd = generate_string(self.password);

        connection.set_password(pwd);

        response
            .add_content_for_response(RPL_PWDSET, "New password was set".to_owned())
            .build()
    }
}

#[cfg(test)]
mod password_parse_tests {
    use crate::irc::constants::ERR_NEEDMOREPARAMS;
    use crate::irc::message::MessageError::{IRCDefined, InvalidCommand, TooManyParams};
    use std::collections::vec_deque::VecDeque;
    //use std::collections::VecDeque;

    use super::*;

    #[test]
    fn test_password_from_valid_generic() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"password");
        let generic = GenericMessage {
            command: Command::Password,
            prefix: None,
            parameters,
        };

        let pwd = Password::from_generic(generic).unwrap();

        assert_eq!(pwd.password, b"password");
    }

    #[test]
    fn test_password_from_valid_generic_with_long_pwd() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"passwordpasswordpasswordpasswordpasswordpasswordpassword");
        let generic = GenericMessage {
            command: Command::Password,
            prefix: None,
            parameters,
        };

        let pwd = Password::from_generic(generic).unwrap();

        assert_eq!(
            pwd.password,
            b"passwordpasswordpasswordpasswordpasswordpasswordpassword"
        );
    }

    #[test]
    fn test_password_from_generic_too_much_arguments() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"password");
        parameters.push_back(b"extra");
        let generic = GenericMessage {
            command: Command::Password,
            prefix: None,
            parameters,
        };

        let err = Password::from_generic(generic).unwrap_err();

        assert_eq!(err, TooManyParams);
    }

    #[test]
    fn test_password_from_generic_too_few_arguments() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();
        let generic = GenericMessage {
            command: Command::Password,
            prefix: None,
            parameters,
        };

        let err = Password::from_generic(generic).unwrap_err();

        assert_eq!(err, IRCDefined(ERR_NEEDMOREPARAMS));
    }

    #[test]
    fn test_password_from_erronous_command() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();
        let generic = GenericMessage {
            command: Command::Nick,
            prefix: None,
            parameters,
        };

        let err = Password::from_generic(generic).unwrap_err();

        assert_eq!(err, InvalidCommand);
    }
}
