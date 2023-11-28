//! Modulo que se centra en las funcionalidades referentes al mensaje de server.
use super::serializer::MessageSerializer;
use super::utils::generate_string_from_vec;
use super::{generic_message::GenericMessage, Command, FromGeneric, MessageError};
use super::{Serializable, ServerExecutable};
use crate::irc::constants::ERR_NEEDMOREPARAMS;
use crate::irc::constants::{ERR_REGMISSING, RPL_REGISTERED};
use crate::irc::message::utils::{
    generate_string, retrieve_hostname, validate_command, validate_irc_params_len, validate_text,
};
use crate::irc::message::UNLIMITED_MAX_LEN;
use crate::irc::model::connection::Connection;
use crate::irc::model::server::Server;
use crate::irc::model::server_connection::ServerConnection;
use crate::irc::model::utils::mt;
use crate::irc::model::MTServerConnection;
use crate::irc::responses::builder::ResponseBuilder;
use crate::irc::responses::{InternalType, ResponseType};

#[derive(Debug)]
pub struct Sv<'a> {
    pub hop: u32,
    pub hostname: Option<&'a [u8]>,
    pub prefix: Option<&'a [u8]>,
    pub server_name: &'a [u8],
    pub address: Vec<&'a [u8]>,
}

impl<'a> FromGeneric<'a> for Sv<'a> {
    fn from_generic(mut generic: GenericMessage<'a>) -> Result<Self, MessageError> {
        validate_command(generic.command, Command::Server)?;
        validate_irc_params_len(
            &generic.parameters,
            UNLIMITED_MAX_LEN,
            4,
            ERR_NEEDMOREPARAMS,
        )?;
        let server_name = match generic.parameters.pop_front() {
            Some(v) => v,
            None => return Err(MessageError::IRCDefined(ERR_NEEDMOREPARAMS)),
        };
        let hop = match generic.parameters.pop_front() {
            Some(v) => match generate_string(v).parse::<u32>() {
                Ok(c) => c,
                Err(_) => return Err(MessageError::InvalidFormat),
            },
            None => return Err(MessageError::IRCDefined(ERR_NEEDMOREPARAMS)),
        };
        let address = validate_text(generic.parameters)?;
        let hostname = retrieve_hostname(address[0])?;
        Ok(Self {
            hop,
            hostname,
            prefix: generic.prefix,
            server_name,
            address,
        })
    }
}

impl Serializable for Sv<'_> {
    fn serialize(&self) -> String {
        let s = MessageSerializer::new(self.prefix, Command::Server)
            .add_parameter(self.server_name)
            .add_number(self.hop)
            .add_trailing_params(&self.address);

        s.serialize()
    }
}

impl Sv<'_> {
    pub fn execute_init(self, _server: &Server, connection: &mut Connection) -> Vec<ResponseType> {
        let mut response = ResponseBuilder::new();

        if !connection.can_be_server() {
            return response
                .add_content_for_response(
                    ERR_REGMISSING,
                    "You can't be a SERVER. NICK message already sent.".to_owned(),
                )
                .build();
        }
        let server_name = generate_string(self.server_name);
        let msg = format!("{} :SERVER succesfully registered", &server_name);

        let uplink = None;

        connection.set_server_connection(server_name, 1, uplink);
        response = response
            .add_internal_response(InternalType::Upgrade)
            .add_content_for_response(RPL_REGISTERED, msg);

        response.build()
    }
}

impl ServerExecutable for Sv<'_> {
    fn _execute_for_server(&self, _: &Server) -> Vec<ResponseType> {
        // Implements for semantic purposes
        ResponseBuilder::new().build()
    }

    fn forward(&self, server: &Server, _: &MTServerConnection) -> String {
        let servername = generate_string(self.server_name);
        format!(
            ":{} SERVER {} {} :{}",
            server.host,  // I'm the uplink for this server for all next servers
            servername,   // The server that is new to the red
            self.hop + 1, // For the next ones, the new server is one more because they need me
            generate_string_from_vec(&self.address)
        )
    }

    fn execute_for_server(&self, server: &Server, origin: MTServerConnection) -> Vec<ResponseType> {
        if self.hop == 1 {
            //Server that I just connected to, is giving me his name.
            let real_servername = generate_string(self.server_name);
            server.update_default_servername(real_servername);
        } else {
            let uplink = match self.prefix {
                Some(u) => Some(generate_string(u)),
                None => return ResponseBuilder::new().build(),
                //Any SERVER message executed here, has to have an uplink
            };
            let servername = generate_string(self.server_name);
            server.add_data_server_connection(mt(ServerConnection::for_data(
                servername, self.hop, uplink,
            )));

            self.replicate(server, origin);
        }

        ResponseBuilder::new().build()
    }
}

#[cfg(test)]
mod topic_tests {
    use crate::irc::constants::ERR_NEEDMOREPARAMS;
    use crate::irc::message::server::Sv;
    use crate::irc::message::GenericMessage;
    use crate::irc::message::MessageError::{IRCDefined, InvalidCommand};
    use crate::irc::message::{Command, FromGeneric};
    use std::collections::vec_deque::VecDeque;

    #[test]
    fn generic_message_with_different_command_error() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::User,
            prefix: None,
            parameters,
        };

        let err = Sv::from_generic(generic).unwrap_err();

        assert_eq!(err, InvalidCommand);
    }

    #[test]
    fn generic_message_with_no_params_error() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::Server,
            prefix: None,
            parameters,
        };

        let err = Sv::from_generic(generic).unwrap_err();

        assert_eq!(err, IRCDefined(ERR_NEEDMOREPARAMS));
    }

    #[test]
    fn generic_message_just_without_optionals_ok() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();

        parameters.push_front(b"server");
        parameters.push_front(b"Experimental");
        parameters.push_front(b":[tolsun.oulu.fi]");
        parameters.push_front(b"1");
        parameters.push_front(b"test.oulu.fi");

        let generic = GenericMessage {
            command: Command::Server,
            prefix: None,
            parameters,
        };

        let sv_mgs = Sv::from_generic(generic).unwrap();

        assert_eq!(sv_mgs.hop, 1);
        assert_eq!(sv_mgs.hostname.unwrap(), b"tolsun.oulu.fi");
        assert_eq!(sv_mgs.server_name, b"test.oulu.fi");
    }
}
