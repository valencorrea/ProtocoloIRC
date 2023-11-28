//! Modulo que se centra en las funcionalidades referentes al mensaje de user.

use crate::irc::constants::{ERR_ALREADYREGISTRED, ERR_NEEDMOREPARAMS, RPL_NICKIN, RPL_SUCLOGIN};
use crate::irc::message::utils::{validate_command, validate_irc_params_len};
use crate::irc::model::connection::Connection;
use crate::irc::model::server::Server;
use crate::irc::model::MTClient;
use crate::irc::responses::builder::ResponseBuilder;
use crate::irc::responses::{InternalType, ResponseType};

use super::serializer::MessageSerializer;
use super::utils::{generate_string, generate_string_from_vec};
use super::{
    generic_message::GenericMessage, utils::validate_hostname, utils::validate_name_invalid_none,
    utils::validate_realname, Command, FromGeneric, MessageError,
};
use super::{Executable, Serializable, ServerExecutable, UNLIMITED_MAX_LEN};

#[derive(Debug)]
/// Struct del mensaje referido a user
/// Contiene un prefijo opcional representado por una referencia u8,
/// un nombre de usuario representado por una referencia a vector u8,
/// un host representado por una referencia a vector u8,
/// y un nombre de server representado por una referencia a vector u8,
pub struct User<'a> {
    pub prefix: Option<&'a [u8]>,
    pub username: &'a [u8],
    pub hostname: &'a [u8],
    pub servername: &'a [u8],
    pub realname: Vec<&'a [u8]>,
}

impl<'a> FromGeneric<'a> for User<'a> {
    /// constructor de mensaje oper a partir de un mensaje generico
    /// puede llegar a enviar un error si el comando no es user,
    /// o si el largo de los parametros no es 4
    fn from_generic(mut generic: GenericMessage<'a>) -> Result<Self, MessageError> {
        validate_command(generic.command, Command::User)?;
        validate_irc_params_len(
            &generic.parameters,
            UNLIMITED_MAX_LEN,
            4,
            ERR_NEEDMOREPARAMS,
        )?;

        let username = validate_name_invalid_none(generic.parameters.pop_front())?;
        let hostname = validate_hostname(generic.parameters.pop_front())?;
        let servername = validate_hostname(generic.parameters.pop_front())?;
        let realname = validate_realname(generic.parameters)?;

        Ok(Self {
            prefix: generic.prefix,
            username,
            hostname,
            servername,
            realname,
        })
    }
}

impl Serializable for User<'_> {
    fn serialize(&self) -> String {
        let serialize = MessageSerializer::new(self.prefix, Command::User)
            .add_parameter(self.username)
            .add_parameter(self.hostname)
            .add_parameter(self.servername)
            .add_trailing_params(&self.realname);

        serialize.serialize()
    }
}

impl Executable for User<'_> {
    fn _execute(&self, _server: &Server, _client: MTClient) -> Vec<ResponseType> {
        ResponseBuilder::new()
            .add_content_for_response(ERR_ALREADYREGISTRED, "You may not register".to_owned())
            .build()
    }
}

impl User<'_> {
    pub fn execute_init(self, server: &Server, connection: &mut Connection) -> Vec<ResponseType> {
        let mut response = ResponseBuilder::new();
        let username = generate_string(self.username);
        let hostname = generate_string(server.host.as_bytes());
        let servername = generate_string(server.host.as_bytes());
        let realname = generate_string_from_vec(&self.realname);

        if server.conn_can_log_in(connection, &username) {
            response = response
                .add_internal_response(InternalType::Upgrade)
                .add_content_for_response(RPL_SUCLOGIN, "Succesfull login".to_owned());

            self.notify(server, connection.get_nickname().unwrap()); //Will always be correct because conn can log in is cheking it
            connection.set_client_connection(username, hostname, servername, realname);
        } else {
            response = response
                .add_content_for_response(ERR_ALREADYREGISTRED, "You may not register".to_owned())
        }

        response.build()
    }
}

impl User<'_> {
    fn notify(&self, server: &Server, nickname: &str) {
        server.server_action_notify(&format!("{}: {}", RPL_NICKIN, nickname))
    }
}

impl ServerExecutable for User<'_> {
    fn _execute_for_server(&self, server: &Server) -> Vec<ResponseType> {
        if let Some(v) = self.prefix {
            let client_nickname = generate_string(v);
            if let Some(client) = server.get_client_by_nickname(&client_nickname) {
                let username = generate_string(self.username);
                let hostname = generate_string(self.hostname);
                let servername = generate_string(self.servername);
                let realname = generate_string_from_vec(&self.realname);
                server.add_data_client_user_info(client, username, hostname, servername, realname);
                self.notify(server, &client_nickname);
            }
        }
        ResponseBuilder::new().build()
    }
}

#[cfg(test)]
mod user_parse_tests {
    use super::*;
    use crate::irc::constants::ERR_NEEDMOREPARAMS;
    use crate::irc::message::MessageError::{IRCDefined, InvalidCommand};
    use std::collections::vec_deque::VecDeque;

    #[test]
    fn test_username_valid_user_message_without_prefix() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"guest");
        parameters.push_back(b"tolmoon");
        parameters.push_back(b"tolsun");
        parameters.push_back(b":Ronnie");
        parameters.push_back(b"Reagan"); //realname

        let generic = GenericMessage {
            command: Command::User,
            prefix: None,
            parameters,
        };

        let user = User::from_generic(generic).unwrap();

        assert_eq!(user.username, b"guest");
        assert_eq!(user.hostname, b"tolmoon");
        assert_eq!(user.servername, b"tolsun");
        assert_eq!(user.realname[0], b"Ronnie");
        assert_eq!(user.realname[1], b"Reagan");
    }

    #[test]
    fn test_username_valid_user_message_with_prefix() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"guest");
        parameters.push_back(b"tolmoon");
        parameters.push_back(b"tolsun");
        parameters.push_back(b":Ronnie");
        parameters.push_back(b"Reagan"); //realname

        let generic = GenericMessage {
            command: Command::User,
            prefix: Some(b"testnick"),
            parameters,
        };

        let user = User::from_generic(generic).unwrap();

        assert_eq!(user.username, b"guest");
        assert_eq!(user.hostname, b"tolmoon");
        assert_eq!(user.servername, b"tolsun");
        assert_eq!(user.realname[0], b"Ronnie");
        assert_eq!(user.realname[1], b"Reagan");
        assert_eq!(user.prefix.unwrap(), b"testnick");
    }

    #[test]
    fn test_username_invalid_command() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"guest");
        parameters.push_back(b"tolmoon");
        parameters.push_back(b"tolsun");
        parameters.push_back(b":Ronnie");
        parameters.push_back(b"Reagan"); //realname

        let generic = GenericMessage {
            command: Command::Password,
            prefix: None,
            parameters,
        };

        let err = User::from_generic(generic).unwrap_err();

        assert_eq!(err, InvalidCommand);
    }

    #[test]
    fn test_username_too_much_parameters() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"guest");
        parameters.push_back(b"second_guest");
        parameters.push_back(b"tolmoon");
        parameters.push_back(b"tolsun");
        parameters.push_back(b":Ronnie");
        parameters.push_back(b"Reagan"); //realname

        let generic = GenericMessage {
            command: Command::User,
            prefix: None,
            parameters,
        };

        let err = User::from_generic(generic).unwrap_err();

        assert_eq!(err, MessageError::InvalidFormat);
    }

    #[test]
    fn test_username_too_few_parameters() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b":Ronnie");
        parameters.push_back(b"Reagan"); //realname

        let generic = GenericMessage {
            command: Command::User,
            prefix: None,
            parameters,
        };

        let err = User::from_generic(generic).unwrap_err();

        assert_eq!(err, IRCDefined(ERR_NEEDMOREPARAMS));
    }
}
