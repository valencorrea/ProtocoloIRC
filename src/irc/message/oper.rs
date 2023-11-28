//! Modulo que se centra en las funcionalidades referentes al mensaje de oper.
use super::{
    generic_message::GenericMessage,
    serializer::MessageSerializer,
    utils::{generate_string, validate_password, validate_user},
    Command, FromGeneric, MessageError, Replicable, Serializable,
};
use crate::irc::{constants::ERR_NEEDMOREPARAMS, responses::ResponseType};
use crate::{
    irc::{constants::RPL_YOUREOPER, model::MTClient},
    try_lock,
};

use crate::irc::message::utils::{validate_command, validate_irc_params_len};
use crate::irc::model::server::Server;
use crate::irc::responses::builder::ResponseBuilder;

#[derive(Debug)]
/// Struct del mensaje referido a oper
/// Contiene un nombre de usuario representada por una referencia a vector u8,
/// y una contraseña representada por una referencia a vector u8,
pub struct Oper<'a> {
    pub prefix: Option<&'a [u8]>,
    pub user: &'a [u8],
    pub password: &'a [u8],
}

impl<'a> FromGeneric<'a> for Oper<'a> {
    /// constructor de mensaje oper a partir de un mensaje generico
    /// puede llegar a enviar un error si el comando no es oper,
    /// o si el largo de los parametros no es 2
    fn from_generic(mut generic: GenericMessage<'a>) -> Result<Self, MessageError> {
        validate_command(generic.command, Command::Oper)?;
        validate_irc_params_len(&generic.parameters, 2, 2, ERR_NEEDMOREPARAMS)?;
        let user = validate_user(generic.parameters.pop_front())?;
        let password = validate_password(generic.parameters.pop_front())?;

        Ok(Self {
            prefix: generic.prefix,
            user,
            password,
        })
    }
}

impl Serializable for Oper<'_> {
    fn serialize(&self) -> String {
        let s = MessageSerializer::new(None, Command::Oper)
            .add_parameter(self.user)
            .add_parameter(self.password);

        s.serialize()
    }
}

impl Replicable for Oper<'_> {
    fn _execute(&self, server: &Server, client: MTClient) -> (Vec<ResponseType>, bool) {
        let res = ResponseBuilder::new();

        let user = generate_string(self.user);
        let pass = generate_string(self.password);

        if let Err(e) = server.set_irc_operator_privileges(client, user, pass) {
            return (res.add_content_for_response(e.code, e.msg).build(), false);
        }
        (
            res.add_content_for_response(RPL_YOUREOPER, "You are now an IRC operator".to_owned())
                .build(),
            true,
        )
    }

    fn forward(&mut self, client: MTClient) -> String {
        let nick = { try_lock!(client).nickname.to_owned() };
        format!("MODE {} +o", nick)
    }
}

#[cfg(test)]
mod oper_parse_tests {
    use crate::irc::message::MessageError::InvalidFormat;
    use std::collections::vec_deque::VecDeque;

    use super::*;

    #[test]
    fn test_valid_oper() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"pass");
        parameters.push_front(b"usr");
        let generic = GenericMessage {
            command: Command::Oper,
            prefix: None,
            parameters,
        };

        let oper = Oper::from_generic(generic).unwrap();

        assert_eq!(oper.user, b"usr");
        assert_eq!(oper.password, b"pass");
    }

    #[test]
    fn test_invalid_oper_invalid_usr() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"pass");
        parameters.push_front(b" ");
        let generic = GenericMessage {
            command: Command::Oper,
            prefix: None,
            parameters,
        };

        let err = Oper::from_generic(generic).unwrap_err();

        assert_eq!(err, InvalidFormat);
    }
}
