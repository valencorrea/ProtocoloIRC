//! Modulo que se centra en las funcionalidades referentes al mensaje de channel mode.
use std::collections::VecDeque;

use crate::irc::{
    constants::{
        ERR_CHANOPRIVSNEEDED, ERR_NEEDMOREPARAMS, ERR_NOSUCHCHANNEL, ERR_NOSUCHNICK,
        ERR_UNKNOWNMODE, RPL_CHANNELMODEIS,
    },
    model::{server::Server, MTChannel, MTClient, ServerError},
    responses::{builder::ResponseBuilder, ResponseType},
};

use super::{
    generic_message::GenericMessage,
    serializer::MessageSerializer,
    utils::{
        generate_string, no_such_nick, try_parse_number, validate_channel, validate_channel_modes,
        validate_command, validate_irc_params_len, validate_name_invalid_none, validate_user,
    },
    ChannelModes, Command, FromGeneric, MessageError, ModesAction, Replicable, Serializable,
};

#[derive(Debug)]
pub struct ChannelMode<'a> {
    pub channel: &'a [u8],
    pub mode: Option<ModesAction<ChannelModes<'a>>>,
}

impl<'a> FromGeneric<'a> for ChannelMode<'a> {
    fn from_generic(mut generic: GenericMessage<'a>) -> Result<Self, MessageError> {
        validate_command(generic.command, Command::Mode)?;
        validate_irc_params_len(&generic.parameters, 3, 1, ERR_NEEDMOREPARAMS)?;

        let channel = validate_channel(generic.parameters.pop_front())?;

        let mut mode = None;

        if !generic.parameters.is_empty() {
            mode = Some(validate_channel_modes(generic.parameters)?);
        }

        Ok(Self { channel, mode })
    }
}

impl Serializable for ChannelMode<'_> {
    fn serialize(&self) -> String {
        let mut s = MessageSerializer::new(None, Command::Mode);

        s = s.add_parameter(self.channel);

        if let Some(mode) = &self.mode {
            s = s.add_parameter(&[mode.as_byte(), (**mode).as_byte()]);
            match **mode {
                ChannelModes::Operator(v) => s = s.add_parameter(v),
                ChannelModes::Limit(Some(n)) => s = s.add_number(n),
                ChannelModes::SpeakInModeratedChannel(v) => s = s.add_parameter(v),
                ChannelModes::ChannelKey(Some(k)) => s = s.add_parameter(k),
                _ => {}
            };
        }

        s.serialize()
    }
}

impl ChannelMode<'_> {
    pub fn dispatch(&self, server: &Server, channel: MTChannel) -> Result<(), ServerError> {
        if let Some(mode) = &self.mode {
            let (to, action) = match mode {
                ModesAction::Add(action) => (true, action),
                ModesAction::Remove(action) => (false, action),
            };

            let res = match action {
                ChannelModes::Operator(nick) => {
                    let nickname = generate_string(nick);
                    let client = server.get_client_by_nickname(&nickname);
                    match client {
                        Some(cl) => {
                            if to {
                                server.set_client_channel_operator(cl, channel);
                            } else {
                                server.del_client_channel_operator(cl, channel);
                            }

                            Ok(())
                        }
                        None => Err(ServerError {
                            code: ERR_NOSUCHNICK,
                            msg: no_such_nick(nickname.as_bytes()),
                        }),
                    }
                }
                ChannelModes::Private => {
                    server.set_channel_private(channel, to);
                    Ok(())
                }
                ChannelModes::Secret => {
                    server.set_channel_secret(channel, to);
                    Ok(())
                }
                ChannelModes::InviteOnly => {
                    server.set_channel_invite_only(channel, to);
                    Ok(())
                }
                ChannelModes::TopicOnlyOperators => {
                    server.set_channel_topic_ops_only(channel, to);
                    Ok(())
                }
                ChannelModes::NoMessageFromOutside => {
                    server.set_channel_no_msg_outside(channel, to);
                    Ok(())
                }
                ChannelModes::Moderated => {
                    server.set_channel_moderated(channel, to);
                    Ok(())
                }
                ChannelModes::Limit(limit) => match limit {
                    Some(l) => {
                        if to {
                            server.set_channel_limit(channel, Some(*l))
                        } else {
                            server.set_channel_limit(channel, None)
                        }
                    }
                    None => server.set_channel_limit(channel, None),
                },
                ChannelModes::SpeakInModeratedChannel(nick) => {
                    let nickname = generate_string(nick);
                    let client = server.get_client_by_nickname(&nickname);
                    match client {
                        Some(cl) => {
                            if to {
                                server.client_speak_in_moderated_channel(cl, channel)
                            } else {
                                server.client_no_speak_in_moderated_channel(cl, channel)
                            }
                        }
                        None => Err(ServerError {
                            code: ERR_NOSUCHNICK,
                            msg: no_such_nick(nickname.as_bytes()),
                        }),
                    }
                }
                ChannelModes::ChannelKey(key) => {
                    match key {
                        Some(k) => {
                            let pwd = generate_string(k);
                            if to {
                                server.set_channel_pwd(channel, Some(pwd));
                            } else {
                                server.set_channel_pwd(channel, None);
                            }
                        }
                        None => server.set_channel_pwd(channel, None),
                    };
                    Ok(())
                }
            };

            res?;
        }
        Ok(())
    }
}

impl Replicable for ChannelMode<'_> {
    fn _execute(&self, server: &Server, client: MTClient) -> (Vec<ResponseType>, bool) {
        let mut response = ResponseBuilder::new();

        let channel_name = generate_string(self.channel);

        let channel = match server.get_channel_by_name(&channel_name) {
            Some(ch) => {
                if self.mode.is_none() {
                    return (
                        response
                            .add_content_for_response(
                                RPL_CHANNELMODEIS,
                                server.describe_channel_modes(ch),
                            )
                            .build(),
                        false,
                    );
                }

                let channel_op = server.is_channel_operator(client, channel_name.as_str());
                if !channel_op {
                    return (
                        response
                            .add_content_for_response(
                                ERR_CHANOPRIVSNEEDED,
                                format!("{} :You're not channel operator", channel_name),
                            )
                            .build(),
                        false,
                    );
                }
                ch
            }
            None => {
                return (
                    response
                        .add_content_for_response(
                            ERR_NOSUCHCHANNEL,
                            format!("{} :No such channel", channel_name),
                        )
                        .build(),
                    false,
                )
            }
        };

        let cchannel = channel.clone();

        if let Err(e) = self.dispatch(server, channel) {
            response = response.add_content_for_response(e.code, e.msg)
        };

        response = response
            .add_content_for_response(RPL_CHANNELMODEIS, server.describe_channel_modes(cchannel));

        (response.build(), true)
    }

    fn forward(&mut self, _client: MTClient) -> String {
        self.serialize()
    }

    fn execute(&mut self, server: &Server, client: MTClient) -> Vec<ResponseType> {
        self._execute(server, client).0
    }
}

impl<'a> ChannelModes<'a> {
    pub fn parse(
        ident: u8,
        mut params: VecDeque<&'a [u8]>,
    ) -> Result<ChannelModes<'a>, MessageError> {
        match ident {
            b'o' => {
                validate_irc_params_len(&params, 1, 1, ERR_NEEDMOREPARAMS)?;
                Ok(ChannelModes::Operator(validate_user(params.pop_front())?))
            }
            b'p' => Ok(ChannelModes::Private),
            b's' => Ok(ChannelModes::Secret),
            b'i' => Ok(ChannelModes::InviteOnly),
            b't' => Ok(ChannelModes::TopicOnlyOperators),
            b'n' => Ok(ChannelModes::NoMessageFromOutside),
            b'm' => Ok(ChannelModes::Moderated),
            b'l' => match params.pop_front() {
                Some(v) => Ok(ChannelModes::Limit(Some(try_parse_number(v)?))),
                None => Ok(ChannelModes::Limit(None)),
            },
            b'v' => {
                validate_irc_params_len(&params, 1, 1, ERR_NEEDMOREPARAMS)?;
                Ok(ChannelModes::SpeakInModeratedChannel(
                    validate_name_invalid_none(params.pop_front())?,
                ))
            }
            b'k' => Ok(ChannelModes::ChannelKey(params.pop_front())),
            _ => Err(MessageError::IRCDefined(ERR_UNKNOWNMODE)),
        }
    }

    pub fn as_byte(&self) -> u8 {
        match self {
            ChannelModes::Operator(_) => b'o',
            ChannelModes::Private => b'p',
            ChannelModes::Secret => b's',
            ChannelModes::InviteOnly => b'i',
            ChannelModes::TopicOnlyOperators => b't',
            ChannelModes::NoMessageFromOutside => b'n',
            ChannelModes::Moderated => b'm',
            ChannelModes::Limit(_) => b'l',
            ChannelModes::SpeakInModeratedChannel(_) => b'v',
            ChannelModes::ChannelKey(_) => b'k',
        }
    }
}
