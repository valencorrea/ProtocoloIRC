//! Modulo que se centra en las funcionalidades referentes al mensaje de topic.
use crate::irc::constants::{ERR_NEEDMOREPARAMS, RPL_NOTOPIC, RPL_TOPIC};
use crate::irc::message::utils::{
    generate_string, generate_string_from_vec, validate_channel, validate_command,
    validate_irc_params_len, validate_text,
};
use crate::irc::message::{Command, FromGeneric, GenericMessage, MessageError, UNLIMITED_MAX_LEN};
use crate::irc::model::server::Server;
use crate::irc::model::{MTClient, ServerError};
use crate::irc::responses::builder::ResponseBuilder;
use crate::irc::responses::ResponseType;
use crate::try_lock;

use super::serializer::MessageSerializer;
use super::{Replicable, Serializable, ServerExecutable};

#[derive(Debug)]
pub struct Topic<'a> {
    pub prefix: Option<&'a [u8]>,
    pub channel: &'a [u8],
    pub topic: Option<Vec<&'a [u8]>>,
}

impl<'a> FromGeneric<'a> for Topic<'a> {
    fn from_generic(mut generic: GenericMessage<'a>) -> Result<Self, MessageError> {
        validate_command(generic.command, Command::Topic)?;
        validate_irc_params_len(
            &generic.parameters,
            UNLIMITED_MAX_LEN,
            1,
            ERR_NEEDMOREPARAMS,
        )?;

        let channel = validate_channel(generic.parameters.pop_front())?;

        let mut topic = None;
        if !generic.parameters.is_empty() {
            topic = Some(validate_text(generic.parameters)?);
        }

        Ok(Self {
            prefix: generic.prefix,
            channel,
            topic,
        })
    }
}

impl Serializable for Topic<'_> {
    fn serialize(&self) -> String {
        let mut s = MessageSerializer::new(self.prefix, Command::Topic).add_parameter(self.channel);

        if let Some(t) = &self.topic {
            s = s.add_trailing_params(t);
        }

        s.serialize()
    }
}

impl Replicable for Topic<'_> {
    fn _execute(&self, server: &Server, client: MTClient) -> (Vec<ResponseType>, bool) {
        let mut response = ResponseBuilder::new();
        let mut should_replicate = true;
        let channel_name = generate_string(self.channel);

        let topic_res = match &self.topic {
            Some(t) => {
                let topic_as_string = generate_string_from_vec(t);
                if let Err(e) = server.set_topic(client, &channel_name, &topic_as_string) {
                    should_replicate = false;
                    (e.code, e.msg)
                } else {
                    (RPL_TOPIC, format!("{} :{}", channel_name, topic_as_string))
                }
            }
            None => topic_response(&channel_name, server.get_topic(&channel_name)),
        };

        response = response.add_content_for_response(topic_res.0, topic_res.1);

        (response.build(), should_replicate)
    }

    fn forward(&mut self, client: MTClient) -> String {
        let nick = { try_lock!(client).nickname.to_owned() };
        self.prefix = None;
        format!(":{} {}", nick, self.serialize())
    }
}

impl ServerExecutable for Topic<'_> {
    fn _execute_for_server(&self, server: &Server) -> Vec<ResponseType> {
        if let Some(v) = self.prefix {
            if let Some(client) = server.get_client_by_nickname(&generate_string(v)) {
                return self._execute(server, client).0;
            }
        } else {
            //Either presentation or replication of topic querying
            let channel_name = generate_string(self.channel);
            if let Some(topic) = &self.topic {
                //Channel presentation, change it no matter what.
                let topic_as_string = generate_string_from_vec(topic);
                if let Some(channel) = server.get_channel_by_name(&channel_name) {
                    try_lock!(channel).set_topic(&topic_as_string);
                }
            }
        }
        ResponseBuilder::new().build()
    }
}

pub fn topic_response(
    channel_name: &str,
    topic_res: Result<Option<String>, ServerError>,
) -> (usize, String) {
    match topic_res {
        Ok(t) => match t {
            Some(t) => (RPL_TOPIC, format!("{} :{}", channel_name, t)),
            None => (RPL_NOTOPIC, format!("{} :No topic is set", channel_name)),
        },
        Err(e) => (e.code, e.msg),
    }
}

#[cfg(test)]
mod topic_tests {
    use std::collections::vec_deque::VecDeque;
    //    use std::collections::VecDeque;
    use crate::irc::constants::ERR_NEEDMOREPARAMS;
    use crate::irc::message::topic::Topic;
    use crate::irc::message::GenericMessage;
    use crate::irc::message::MessageError::{IRCDefined, InvalidCommand, InvalidFormat};
    use crate::irc::message::{Command, FromGeneric};

    #[test]
    fn generic_message_with_different_command_error() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::User,
            prefix: None,
            parameters,
        };

        let err = Topic::from_generic(generic).unwrap_err();

        assert_eq!(err, InvalidCommand);
    }

    #[test]
    fn generic_message_with_no_params_error() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::Topic,
            prefix: None,
            parameters,
        };

        let err = Topic::from_generic(generic).unwrap_err();

        assert_eq!(err, IRCDefined(ERR_NEEDMOREPARAMS));
    }

    #[test]
    fn generic_message_just_with_channel_ok() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"#channel");

        let generic = GenericMessage {
            command: Command::Topic,
            prefix: None,
            parameters,
        };

        let topic_mgs = Topic::from_generic(generic).unwrap();

        assert_eq!(topic_mgs.channel, b"#channel");
    }

    #[test]
    fn generic_message_just_with_channel_with_invalid_format_error() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"channel");

        let generic = GenericMessage {
            command: Command::Topic,
            prefix: None,
            parameters,
        };

        let err = Topic::from_generic(generic).unwrap_err();

        assert_eq!(err, InvalidFormat);
    }

    #[test]
    fn generic_message_with_channel_and_topic_ok() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"#channel");
        parameters.push_back(b":topic");

        let generic = GenericMessage {
            command: Command::Topic,
            prefix: None,
            parameters,
        };

        let topic_mgs = Topic::from_generic(generic).unwrap();
        let topic_name = topic_mgs.topic.unwrap();

        assert_eq!(topic_mgs.channel, b"#channel");
        assert_eq!(topic_name.len(), 1);
        assert_eq!(topic_name[0], b"topic");
    }

    #[test]
    fn generic_message_with_channel_and_prefix_ok() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"#channel");

        let generic = GenericMessage {
            command: Command::Topic,
            prefix: Some(b"Wiz"),
            parameters,
        };

        let topic_mgs = Topic::from_generic(generic).unwrap();

        assert_eq!(topic_mgs.channel, b"#channel");
    }

    #[test]
    fn generic_message_with_channel_and_invalid_topic_error() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"#channel");
        parameters.push_back(b"topic");

        let generic = GenericMessage {
            command: Command::Topic,
            prefix: None,
            parameters,
        };

        let err = Topic::from_generic(generic).unwrap_err();

        assert_eq!(err, InvalidFormat);
    }

    #[test]
    fn generic_message_with_all_params_ok() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"#channel");
        parameters.push_back(b":topic");

        let generic = GenericMessage {
            command: Command::Topic,
            prefix: Some(b"Wiz"),
            parameters,
        };

        let topic_mgs = Topic::from_generic(generic).unwrap();
        let topic_name = topic_mgs.topic.unwrap();

        assert_eq!(topic_mgs.prefix.unwrap(), b"Wiz");
        assert_eq!(topic_mgs.channel, b"#channel");
        assert_eq!(topic_name.len(), 1);
        assert_eq!(topic_name[0], b"topic");
    }

    #[test]
    fn generic_message_with_many_words_in_topic_ok() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_back(b"#channel");
        parameters.push_back(b":this");
        parameters.push_back(b"is");
        parameters.push_back(b"a");
        parameters.push_back(b"topic");

        let generic = GenericMessage {
            command: Command::Topic,
            prefix: Some(b"Wiz"),
            parameters,
        };

        let topic_mgs = Topic::from_generic(generic).unwrap();
        let topic_name = topic_mgs.topic.unwrap();

        assert_eq!(topic_mgs.prefix.unwrap(), b"Wiz");
        assert_eq!(topic_mgs.channel, b"#channel");
        assert_eq!(topic_name.len(), 4);
        assert_eq!(topic_name[0], b"this");
        assert_eq!(topic_name[1], b"is");
        assert_eq!(topic_name[2], b"a");
        assert_eq!(topic_name[3], b"topic");
    }
}
