//! Modulo que se centra en las funcionalidades referentes al mensaje de quit.
use super::serializer::MessageSerializer;
use super::utils::generate_string_from_vec;
use super::{generic_message::GenericMessage, Command, FromGeneric, MessageError};
use super::{Replicable, Serializable, ServerExecutable};
use crate::irc::constants::RPL_NICKOUT;
use crate::irc::message::utils::{
    generate_string, validate_command, validate_irc_params_len, validate_text,
};
use crate::irc::message::UNLIMITED_MAX_LEN;
use crate::irc::model::connection::Connection;
use crate::irc::model::server::Server;
use crate::irc::model::MTClient;
use crate::irc::responses::builder::ResponseBuilder;
use crate::irc::responses::InternalType;
use crate::irc::{constants::ERR_NEEDMOREPARAMS, responses::ResponseType};
use crate::try_lock;

#[derive(Debug)]
pub struct Quit<'a> {
    pub prefix: Option<&'a [u8]>,
    pub message: Option<Vec<&'a [u8]>>,
}

impl<'a> FromGeneric<'a> for Quit<'a> {
    fn from_generic(generic: GenericMessage<'a>) -> Result<Self, MessageError> {
        validate_command(generic.command, Command::Quit)?;
        validate_irc_params_len(
            &generic.parameters,
            UNLIMITED_MAX_LEN,
            0,
            ERR_NEEDMOREPARAMS,
        )?;

        let message = match validate_text(generic.parameters) {
            Ok(v) => Some(v),
            Err(_) => None,
        };

        Ok(Self {
            prefix: generic.prefix,
            message,
        })
    }
}

impl Serializable for Quit<'_> {
    fn serialize(&self) -> String {
        let mut s = MessageSerializer::new(self.prefix, Command::Quit);

        if let Some(msg) = &self.message {
            s = s.add_trailing_params(msg);
        }

        s.serialize()
    }
}

impl Quit<'_> {
    pub fn execute_init(self, _server: &Server, connection: &mut Connection) -> Vec<ResponseType> {
        connection.quit();
        ResponseBuilder::new()
            .add_internal_response(InternalType::Quit)
            .build()
    }
}

impl Replicable for Quit<'_> {
    fn _execute(&self, server: &Server, client: MTClient) -> (Vec<ResponseType>, bool) {
        let msg = match &self.message {
            Some(m) => generate_string_from_vec(m),
            None => "Se retira del server".to_owned(),
        };

        server.quit_client(msg, client.clone());
        self.notify(server, client);
        (
            ResponseBuilder::new()
                .add_internal_response(InternalType::Quit)
                .build(),
            true,
        )
    }

    fn forward(&mut self, client: MTClient) -> String {
        let nick = { try_lock!(client).nickname.to_owned() };
        self.prefix = None;
        format!(":{} {}", nick, self.serialize())
    }
}

impl Quit<'_> {
    fn notify(&self, server: &Server, client: MTClient) {
        let nickname = { try_lock!(client).nickname.to_owned() };
        server.server_action_notify(&format!("{}: {}", RPL_NICKOUT, nickname));
    }
}

impl ServerExecutable for Quit<'_> {
    fn _execute_for_server(&self, server: &Server) -> Vec<ResponseType> {
        if let Some(v) = self.prefix {
            if let Some(client) = server.get_client_by_nickname(&generate_string(v)) {
                self._execute(server, client);
            }
        }

        ResponseBuilder::new().build()
    }
}
#[cfg(test)]
mod quit_parse_tests {
    use std::collections::vec_deque::VecDeque;

    use super::*;

    #[test]
    fn test_valid_mssg_without_space() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b":mssg");
        let generic = GenericMessage {
            command: Command::Quit,
            prefix: None,
            parameters,
        };

        let quit = Quit::from_generic(generic).unwrap();
        let msg = quit.message.unwrap();

        assert_eq!(msg[0], b"mssg");
    }

    #[test]
    fn test_valid_mssg_with_one_space() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b":ms sg");
        let generic = GenericMessage {
            command: Command::Quit,
            prefix: None,
            parameters,
        };

        let quit = Quit::from_generic(generic).unwrap();
        let msg = quit.message.unwrap();

        assert_eq!(msg, vec![b"ms sg"]);
    }

    #[test]
    fn test_valid_mssg_with_two_spaces() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b":Gone to have lunch");
        let generic = GenericMessage {
            command: Command::Quit,
            prefix: None,
            parameters,
        };

        let quit = Quit::from_generic(generic).unwrap();
        let msg = quit.message.unwrap();

        assert_eq!(msg, vec![b"Gone to have lunch"]);
    }

    #[test]
    fn test_valid_mssg_null() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::Quit,
            prefix: None,
            parameters,
        };

        let quit = Quit::from_generic(generic).unwrap();

        assert!(quit.message.is_none());
    }
}
