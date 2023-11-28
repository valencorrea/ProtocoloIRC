//! Modulo que se centra en las funcionalidades referentes al mensaje de kick.
use crate::irc::constants::{
    ERR_CHANOPRIVSNEEDED, ERR_NEEDMOREPARAMS, ERR_SAMEUSER, ERR_USERNOTINCHANNEL, RPL_CHANNELOUT,
};
use crate::irc::message::utils::{
    generate_string, validate_channel, validate_command, validate_irc_params_len,
    validate_name_invalid_none, validate_name_valid_none, validate_text,
};
use crate::irc::message::{Command, FromGeneric, GenericMessage, MessageError, UNLIMITED_MAX_LEN};
use crate::irc::model::server::Server;
use crate::irc::model::MTClient;
use crate::irc::responses::builder::ResponseBuilder;
use crate::irc::responses::ResponseType;
use crate::try_lock;

use super::serializer::MessageSerializer;
use super::utils::generate_string_from_vec;
use super::{Replicable, Serializable, ServerExecutable};

#[derive(Debug)]
pub struct Kick<'a> {
    // todo se puede extender a vector de channels y users
    pub prefix: Option<&'a [u8]>,
    pub channel: &'a [u8],
    pub user: &'a [u8],
    pub comment: Option<Vec<&'a [u8]>>,
}

impl<'a> FromGeneric<'a> for Kick<'a> {
    fn from_generic(mut generic: GenericMessage<'a>) -> Result<Self, MessageError> {
        validate_command(generic.command, Command::Kick)?;
        validate_irc_params_len(
            &generic.parameters,
            UNLIMITED_MAX_LEN,
            2,
            ERR_NEEDMOREPARAMS,
        )?;

        let prefix = validate_name_valid_none(generic.prefix)?;
        let channel = validate_channel(generic.parameters.pop_front())?;
        let user = validate_name_invalid_none(generic.parameters.pop_front())?;

        let mut comment = None;
        if !generic.parameters.is_empty() {
            comment = Some(validate_text(generic.parameters)?);
        }

        Ok(Self {
            prefix,
            channel,
            user,
            comment,
        })
    }
}

impl Serializable for Kick<'_> {
    fn serialize(&self) -> String {
        let mut s = MessageSerializer::new(self.prefix, Command::Kick)
            .add_parameter(self.channel)
            .add_parameter(self.user);

        if let Some(c) = &self.comment {
            s = s.add_trailing_params(c);
        }

        s.serialize()
    }
}

impl Replicable for Kick<'_> {
    fn _execute(&self, server: &Server, client: MTClient) -> (Vec<ResponseType>, bool) {
        let mut response = ResponseBuilder::new();
        let mut should_replicate = true;

        let channel_name = generate_string(self.channel);

        let user_name = generate_string(self.user);

        let kicker_name = { try_lock!(client).nickname.to_owned() };

        {
            if kicker_name == user_name {
                response = response.add_content_for_response(
                    ERR_SAMEUSER,
                    "You can't kick yourself off a channel, use PART".to_owned(),
                );
                return (response.build(), false);
            }
        }

        let kicked_user = match server.get_client_by_nickname(user_name.as_str()) {
            Some(u) => u,
            None => {
                let err_msg = format!(
                    "{} {} :They aren't on that channel",
                    user_name, channel_name
                );
                response = response.add_content_for_response(ERR_USERNOTINCHANNEL, err_msg);
                return (response.build(), false);
            }
        };

        if !server.is_channel_operator(client, channel_name.as_str()) {
            let err_msg = format!("{} :You're not channel operator", &channel_name);
            response = response.add_content_for_response(ERR_CHANOPRIVSNEEDED, err_msg);
            return (response.build(), false);
        };

        if let Err(e) = server.remove_client_from_channel(&channel_name, kicked_user.clone()) {
            should_replicate = false;
            response = response.add_content_for_response(e.code, e.msg);
        };

        let reason = match &self.comment {
            Some(r) => generate_string_from_vec(r),
            None => "No reason given".to_owned(),
        };

        let mut kicked = try_lock!(kicked_user);

        server.send_message_to_local_client(
            &mut kicked,
            &format!(
                "{} :Kicked from channel\n{} {} :{}",
                channel_name, channel_name, kicker_name, reason
            ),
        );

        self.notify(server, &channel_name, &user_name);

        (response.build(), should_replicate)
    }

    fn forward(&mut self, client: MTClient) -> String {
        let nick = { try_lock!(client).nickname.to_owned() };
        self.prefix = None;
        format!(":{} {}", nick, self.serialize())
    }
}

impl Kick<'_> {
    fn notify(&self, server: &Server, channel: &str, user: &str) {
        server.server_action_notify(&format!("{}: {} {}", RPL_CHANNELOUT, channel, user))
    }
}

impl ServerExecutable for Kick<'_> {
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
mod topic_tests {
    use crate::irc::constants::ERR_NEEDMOREPARAMS;
    use crate::irc::message::kick::Kick;
    use crate::irc::message::GenericMessage;
    use crate::irc::message::MessageError::{IRCDefined, InvalidCommand, InvalidFormat};
    use crate::irc::message::{Command, FromGeneric};
    use std::collections::VecDeque;

    #[test]
    fn generic_message_with_different_command_error() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::User,
            prefix: None,
            parameters,
        };

        let err = Kick::from_generic(generic).unwrap_err();

        assert_eq!(err, InvalidCommand);
    }

    #[test]
    fn generic_message_with_no_params_error() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::Kick,
            prefix: None,
            parameters,
        };

        let err = Kick::from_generic(generic).unwrap_err();

        assert_eq!(err, IRCDefined(ERR_NEEDMOREPARAMS));
    }

    #[test]
    fn generic_message_with_channel_and_user_ok() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"#channel");
        parameters.push_back(b"pepe");

        let generic = GenericMessage {
            command: Command::Kick,
            prefix: None,
            parameters,
        };

        let kick = Kick::from_generic(generic).unwrap();

        assert_eq!(kick.channel, b"#channel");
        assert_eq!(kick.user, b"pepe");
    }

    #[test]
    fn generic_message_with_bad_formatted_channel_error() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"@channel");
        parameters.push_back(b"pepe");

        let generic = GenericMessage {
            command: Command::Kick,
            prefix: None,
            parameters,
        };

        let err = Kick::from_generic(generic).unwrap_err();

        assert_eq!(err, InvalidFormat);
    }

    #[test]
    fn generic_message_with_bad_formatted_user_error() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"#channel");
        parameters.push_back(b" ");

        let generic = GenericMessage {
            command: Command::Kick,
            prefix: None,
            parameters,
        };

        let err = Kick::from_generic(generic).unwrap_err();

        assert_eq!(err, InvalidFormat);
    }

    #[test]
    fn generic_message_with_prefix_ok() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"#channel");
        parameters.push_back(b"pepe");

        let generic = GenericMessage {
            command: Command::Kick,
            prefix: Some(b"Wiz"),
            parameters,
        };

        let kick = Kick::from_generic(generic).unwrap();

        assert_eq!(kick.prefix.unwrap(), b"Wiz");
        assert_eq!(kick.channel, b"#channel");
        assert_eq!(kick.user, b"pepe");
        assert_eq!(kick.comment, None);
    }

    #[test]
    fn generic_message_with_many_words_in_comment_ok() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"#channel");
        parameters.push_back(b"pepe");
        parameters.push_back(b":this");
        parameters.push_back(b"is");
        parameters.push_back(b"a");
        parameters.push_back(b"comment");

        let generic = GenericMessage {
            command: Command::Kick,
            prefix: Some(b"Wiz"),
            parameters,
        };

        let kick = Kick::from_generic(generic).unwrap();
        let kick_comment = kick.comment.unwrap();

        assert_eq!(kick.prefix.unwrap(), b"Wiz");
        assert_eq!(kick.channel, b"#channel");
        assert_eq!(kick.user, b"pepe");
        assert_eq!(kick_comment.len(), 4);
        assert_eq!(kick_comment[0], b"this");
        assert_eq!(kick_comment[1], b"is");
        assert_eq!(kick_comment[2], b"a");
        assert_eq!(kick_comment[3], b"comment");
    }
}
