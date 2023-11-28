//! Modulo que se centra en las funcionalidades referentes a los mensajes genericos
use crate::irc::{
    constants::ERR_NEEDMOREPARAMS,
    model::{server::Server, MTClient},
    responses::{builder::ResponseBuilder, ResponseType},
};

use super::{
    channel_mode::ChannelMode,
    generic_message::GenericMessage,
    user_mode::UserMode,
    utils::{generate_string, validate_channel},
    FromGeneric, MessageError, ModesAction, Replicable, Serializable, ServerExecutable, UserModes,
};

#[derive(Debug)]
enum ModeType {
    User,
    Channel,
}
#[derive(Debug)]
pub struct Mode<'a> {
    _type: ModeType,
    pub channel_mode: Option<ChannelMode<'a>>,
    pub user_mode: Option<UserMode<'a>>,
}

impl<'a> FromGeneric<'a> for Mode<'a> {
    fn from_generic(mut generic: GenericMessage<'a>) -> Result<Mode, MessageError> {
        let first_param = generic.parameters.pop_front();

        if first_param.is_none() {
            return Err(MessageError::IRCDefined(ERR_NEEDMOREPARAMS));
        }

        match validate_channel(first_param) {
            Ok(channel) => {
                generic.parameters.push_front(channel);
                Ok(Self {
                    _type: ModeType::Channel,
                    channel_mode: Some(ChannelMode::from_generic(generic)?),
                    user_mode: None,
                })
            }
            Err(_) => {
                match first_param {
                    Some(other) => {
                        generic.parameters.push_front(other);
                        Ok(Self {
                            _type: ModeType::User,
                            channel_mode: None,
                            user_mode: Some(UserMode::from_generic(generic)?),
                        })
                    }
                    None => Err(MessageError::IRCDefined(ERR_NEEDMOREPARAMS)), //This can't happen, but YEY for security
                }
            }
        }
    }
}

impl Serializable for Mode<'_> {
    fn serialize(&self) -> String {
        match self._type {
            ModeType::User => match &self.user_mode {
                Some(comm) => comm.serialize(),
                None => "HOW IN HELL DID WE GET HERE".to_owned(),
            },
            ModeType::Channel => match &self.channel_mode {
                Some(comm) => comm.serialize(),
                None => "HOW IN HELL DID WE GET HERE".to_owned(),
            },
        }
    }
}

impl Replicable for Mode<'_> {
    fn _execute(&self, server: &Server, client: MTClient) -> (Vec<ResponseType>, bool) {
        match self._type {
            ModeType::User => match &self.user_mode {
                Some(comm) => comm._execute(server, client),
                None => (
                    ResponseBuilder::new()
                        .add_from_error(MessageError::InvalidFormat)
                        .build(),
                    false,
                ),
            },
            ModeType::Channel => match &self.channel_mode {
                Some(comm) => comm._execute(server, client),
                None => (
                    ResponseBuilder::new()
                        .add_from_error(MessageError::InvalidFormat)
                        .build(),
                    false,
                ),
            },
        }
    }

    fn forward(&mut self, client: MTClient) -> String {
        match self._type {
            ModeType::User => match &mut self.user_mode {
                Some(comm) => comm.forward(client),
                None => String::new(),
            },
            ModeType::Channel => match &mut self.channel_mode {
                Some(comm) => comm.forward(client),
                None => String::new(),
            },
        }
    }
}

impl ServerExecutable for Mode<'_> {
    fn _execute_for_server(&self, server: &Server) -> Vec<ResponseType> {
        if let Some(cm) = &self.channel_mode {
            match &cm.mode {
                Some(_) => {
                    if let Some(channel) = server.get_channel_by_name(&generate_string(cm.channel))
                    {
                        let _ = cm.dispatch(server, channel);
                    };
                }
                None => {
                    server.create_channel(generate_string(cm.channel), None, None);
                }
            }
        }

        if let Some(um) = &self.user_mode {
            if let Some(client) = server.get_client_by_nickname(&generate_string(um.nickname)) {
                if let Some(ModesAction::Add(UserModes::IRCOperator)) = &um.mode {
                    server.force_set_client_sv_operator(client, true);
                } else {
                    um.dispatch(server, client);
                }
            }
        }
        ResponseBuilder::new().build()
    }
}
