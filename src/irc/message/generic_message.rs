//! Modulo que se centra en las funcionalidades referentes al parseo
//! de los mensajes.
use crate::irc::message::away::Away;
use crate::irc::message::generic_mode::Mode;
use crate::irc::message::invite::Invite;
use crate::irc::message::join::Join;
use crate::irc::message::kick::Kick;
use crate::irc::message::list::List;
use crate::irc::message::names::Names;
use crate::irc::message::nickname::Nickname;
use crate::irc::message::notice::Notice;
use crate::irc::message::oper::Oper;
use crate::irc::message::part::Part;
use crate::irc::message::password::Password;
use crate::irc::message::private::Private;
use crate::irc::message::quit::Quit;
use crate::irc::message::server::Sv;
use crate::irc::message::server_quit::ServerQuit;
use crate::irc::message::topic::Topic;
use crate::irc::message::user::User;
use crate::irc::message::utils::*;
use crate::irc::message::who::Who;
use crate::irc::message::whois::Whois;
use crate::irc::message::Executable;
use crate::irc::message::MessageError;
use crate::irc::message::MessageError::*;
use crate::irc::message::Replicable;
use crate::irc::message::ServerExecutable;
use crate::irc::message::{Command, FromGeneric};
use crate::irc::model::connection::Connection;
use crate::irc::model::server::Server;
use crate::irc::model::{MTClient, MTServerConnection};
use crate::irc::responses::builder::ResponseBuilder;
use crate::irc::responses::ResponseType;
use std::collections::vec_deque::VecDeque;

pub const MESSAGE_LIMIT: usize = 510;

#[derive(Debug)]
/// Struct de mensaje generico
/// Contiene un commando perteneciente al enum de los commandos,
/// un prefijo opcional representado por una referencia a vector u8,
/// y los parametros del mensaje representado por un vector de referencias u8,
pub struct GenericMessage<'a> {
    pub command: Command,
    pub prefix: Option<&'a [u8]>,
    pub parameters: VecDeque<&'a [u8]>,
}

impl<'a> GenericMessage<'a> {
    /// constructor de mensaje generico
    /// puede llegar a enviar un error si el string enviado es vacio,
    /// se excede el limite de largo de mensaje
    /// o algun atributo del mensaje no respeta las precondiciones
    pub fn parse(message: &'a str) -> Result<GenericMessage, MessageError> {
        if message.is_empty() {
            return Err(EmptyMessage);
        }
        if message.len() > MESSAGE_LIMIT {
            return Err(MessageTooLong);
        }

        let mut tokens = split_message(message);

        if tokens.is_empty() {
            return Err(EmptyMessage);
        }

        let prefix = Self::retrieve_prefix(&mut tokens);

        let command = match Self::retrieve_command(&mut tokens) {
            Some(v) => v,
            None => return Err(InvalidCommand),
        };

        Ok(GenericMessage {
            command,
            prefix,
            parameters: tokens,
        })
    }

    pub fn execute(self, server: &Server, client: MTClient) -> Vec<ResponseType> {
        macro_rules! execute {
            ($message: expr) => {
                match $message {
                    Ok(mut msg) => msg.execute(server, client),
                    Err(e) => ResponseBuilder::new().add_from_error(e).build(),
                }
            };
        }

        match self.command {
            Command::Password => execute!(Password::from_generic(self)),
            Command::Nick => execute!(Nickname::from_generic(self)),
            Command::User => execute!(User::from_generic(self)),
            Command::Oper => execute!(Oper::from_generic(self)),
            Command::Quit => execute!(Quit::from_generic(self)),
            Command::Join => execute!(Join::from_generic(self)),
            Command::Part => execute!(Part::from_generic(self)),
            Command::Names => execute!(Names::from_generic(self)),
            Command::List => execute!(List::from_generic(self)),
            Command::Invite => execute!(Invite::from_generic(self)),
            Command::PrivateMessage => execute!(Private::from_generic(self)),
            Command::Notice => execute!(Notice::from_generic(self)),
            Command::Who => execute!(Who::from_generic(self)),
            Command::WhoIs => execute!(Whois::from_generic(self)),
            Command::Topic => execute!(Topic::from_generic(self)),
            Command::Mode => execute!(Mode::from_generic(self)),
            Command::Kick => execute!(Kick::from_generic(self)),
            Command::Away => execute!(Away::from_generic(self)),
            _ => ResponseBuilder::new()
                .add_from_error(MessageError::InvalidFormat)
                .build(),
        }
    }

    pub fn execute_registration(self, server: &Server, conn: &mut Connection) -> Vec<ResponseType> {
        macro_rules! execute {
            ($message: expr) => {
                match $message {
                    Ok(msg) => msg.execute_init(server, conn),
                    Err(e) => ResponseBuilder::new().add_from_error(e).build(),
                }
            };
        }

        match self.command {
            Command::Password => execute!(Password::from_generic(self)),
            Command::Nick => execute!(Nickname::from_generic(self)),
            Command::User => execute!(User::from_generic(self)),
            Command::Server => execute!(Sv::from_generic(self)),
            Command::Quit => execute!(Quit::from_generic(self)),
            _ => ResponseBuilder::new()
                .add_from_error(MessageError::InvalidFormat)
                .build(),
        }
    }

    pub fn execute_for_server(
        self,
        server: &Server,
        server_connection: MTServerConnection,
    ) -> Vec<ResponseType> {
        macro_rules! execute {
            ($message: expr) => {
                match $message {
                    Ok(msg) => msg.execute_for_server(server, server_connection),
                    Err(e) => ResponseBuilder::new().add_from_error(e).build(),
                }
            };
        }

        match self.command {
            Command::Nick => execute!(Nickname::from_generic(self)),
            Command::Quit => execute!(Quit::from_generic(self)),
            Command::Join => execute!(Join::from_generic(self)),
            Command::Part => execute!(Part::from_generic(self)),
            Command::Invite => execute!(Invite::from_generic(self)),
            Command::PrivateMessage => execute!(Private::from_generic(self)),
            Command::Notice => execute!(Notice::from_generic(self)),
            Command::Topic => execute!(Topic::from_generic(self)),
            Command::Mode => execute!(Mode::from_generic(self)),
            Command::Kick => execute!(Kick::from_generic(self)),
            Command::User => execute!(User::from_generic(self)),
            Command::Away => execute!(Away::from_generic(self)),
            Command::Server => execute!(Sv::from_generic(self)),
            Command::ServerQuit => execute!(ServerQuit::from_generic(self)),
            _ => ResponseBuilder::new()
                .add_from_error(MessageError::InvalidFormat)
                .build(),
        }
    }

    /// funcion encargada de desenvolver la logica de devolver el prefijo
    fn retrieve_prefix(tokens: &mut VecDeque<&'a [u8]>) -> Option<&'a [u8]> {
        let first_token = tokens.front();
        if first_token?.len() < 2 || !starts_with_colon(first_token?) {
            return None;
        }
        strip_colon(tokens.pop_front()?)
    }

    /// funcion encargada de desenvolver la logica de devolver el comando
    fn retrieve_command(tokens: &mut VecDeque<&'a [u8]>) -> Option<Command> {
        let command = tokens.pop_front()?;

        let command_str = match std::str::from_utf8(command) {
            Ok(v) => v.to_uppercase(),
            Err(_) => return None,
        };

        Command::from_str(&command_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_without_prefix_valid_command() {
        let test_msg = "NICK Kilroy";

        let generic = GenericMessage::parse(test_msg).unwrap();

        assert_eq!(generic.command, Command::Nick);
        assert!(generic.prefix.is_none());
        assert!(!generic.parameters.is_empty())
    }

    #[test]
    fn test_message_without_prefix_valid_command_lowercase() {
        let test_msg = "nick Kilroy";

        let generic = GenericMessage::parse(test_msg).unwrap();

        assert_eq!(generic.command, Command::Nick);
        assert!(generic.prefix.is_none());
        assert!(!generic.parameters.is_empty())
    }

    #[test]
    fn test_message_with_prefix_valid_command() {
        let test_msg = ":WiZ NICK Kilroy";

        let generic = GenericMessage::parse(test_msg).unwrap();

        assert_eq!(generic.command, Command::Nick);
        assert_eq!(generic.prefix.unwrap(), b"WiZ");
        assert!(!generic.parameters.is_empty());
    }

    #[test]
    fn test_message_without_prefix_invalid_command() {
        let test_msg = ":WiZ CHANGE_NICKNAME Kilroy";

        let generic = GenericMessage::parse(test_msg);

        assert!(generic.is_err());
        assert_eq!(generic.unwrap_err(), InvalidCommand);
    }

    #[test]
    fn test_message_empty() {
        let test_msg = "";

        let generic = GenericMessage::parse(test_msg);

        assert!(generic.is_err());
        assert_eq!(generic.unwrap_err(), EmptyMessage);
    }

    #[test]
    fn test_pure_space_message() {
        let test_msg = "                              ";

        let generic = GenericMessage::parse(test_msg);

        assert!(generic.is_err());
        assert_eq!(generic.unwrap_err(), EmptyMessage);
    }

    #[test]
    fn test_message_too_long() {
        let test_arr = [0x3A; MESSAGE_LIMIT * 2];
        let test_msg = std::str::from_utf8(&test_arr).unwrap();

        let generic = GenericMessage::parse(test_msg);

        assert!(generic.is_err());
        assert_eq!(generic.unwrap_err(), MessageTooLong);
    }

    #[test]
    fn test_correct_parameters() {
        let test_msg = ":WiZ NICK Kilroy Other Parameters To Test";

        let generic = GenericMessage::parse(test_msg).unwrap();

        assert!(!generic.parameters.is_empty());
        assert_eq!(generic.parameters[0], b"Kilroy");
        assert_eq!(generic.parameters[1], b"Other");
        assert_eq!(generic.parameters[2], b"Parameters");
        assert_eq!(generic.parameters[3], b"To");
        assert_eq!(generic.parameters[4], b"Test");
    }
}
