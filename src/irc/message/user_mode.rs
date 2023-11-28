//! Modulo que se centra en las funcionalidades referentes al mensaje de user mode.
use crate::irc::{
    constants::{ERR_NEEDMOREPARAMS, ERR_NOSUCHNICK, ERR_UNKNOWNMODE, RPL_UMODEIS},
    model::{server::Server, MTClient},
    responses::{builder::ResponseBuilder, ResponseType},
};

use crate::try_lock;

use super::{
    generic_message::GenericMessage,
    serializer::MessageSerializer,
    utils::{
        generate_string, no_such_nick, validate_command, validate_irc_params_len,
        validate_name_invalid_none, validate_user_modes,
    },
    Command, FromGeneric, MessageError, ModesAction, Replicable, Serializable, UserModes,
};

#[derive(Debug)]
pub struct UserMode<'a> {
    pub nickname: &'a [u8],
    pub mode: Option<ModesAction<UserModes>>,
}

impl<'a> FromGeneric<'a> for UserMode<'a> {
    fn from_generic(mut generic: GenericMessage<'a>) -> Result<Self, MessageError> {
        validate_command(generic.command, Command::Mode)?;
        validate_irc_params_len(&generic.parameters, 2, 1, ERR_NEEDMOREPARAMS)?;

        let nickname = validate_name_invalid_none(generic.parameters.pop_front())?;

        let mut mode = None;
        if !generic.parameters.is_empty() {
            mode = Some(validate_user_modes(generic.parameters)?);
        }

        Ok(Self { nickname, mode })
    }
}

impl Serializable for UserMode<'_> {
    fn serialize(&self) -> String {
        let mut s = MessageSerializer::new(None, Command::Mode);

        s = s.add_parameter(self.nickname);

        if let Some(m) = &self.mode {
            s = s.add_parameter(&[m.as_byte(), (**m).as_byte()]);
        }

        s.serialize()
    }
}

impl UserMode<'_> {
    pub fn dispatch(&self, server: &Server, client: MTClient) {
        if let Some(mode) = &self.mode {
            let (to, action) = match mode {
                ModesAction::Add(action) => (true, action),
                ModesAction::Remove(action) => (false, action),
            };

            match action {
                UserModes::Invisible => server.set_client_invisible(client, to),
                UserModes::ReceiveServerNotices => server.set_client_receive_sv_notices(client, to),
                UserModes::IRCOperator => server.set_client_sv_operator(client, to),
            }
        }
    }
}

impl Replicable for UserMode<'_> {
    fn _execute(&self, server: &Server, client: MTClient) -> (Vec<ResponseType>, bool) {
        let mut res = ResponseBuilder::new();
        {
            if generate_string(self.nickname) != try_lock!(client).nickname {
                return (
                    res.add_content_for_response(ERR_NOSUCHNICK, no_such_nick(self.nickname))
                        .build(),
                    false,
                );
            }
        }

        self.dispatch(server, client.clone());

        res = res.add_content_for_response(RPL_UMODEIS, server.describe_client_modes(client));

        (res.build(), true)
    }

    fn execute(&mut self, server: &Server, client: MTClient) -> Vec<ResponseType> {
        self._execute(server, client).0
    }

    fn forward(&mut self, _client: MTClient) -> String {
        self.serialize()
    }
}

impl UserModes {
    pub fn parse(ident: u8) -> Result<UserModes, MessageError> {
        match ident {
            b'i' => Ok(UserModes::Invisible),
            b's' => Ok(UserModes::ReceiveServerNotices),
            b'o' => Ok(UserModes::IRCOperator),
            _ => Err(MessageError::IRCDefined(ERR_UNKNOWNMODE)),
        }
    }

    pub fn as_byte(&self) -> u8 {
        match self {
            UserModes::Invisible => b'i',
            UserModes::ReceiveServerNotices => b's',
            UserModes::IRCOperator => b'o',
        }
    }
}
