//! Modulo que se centra en las funcionalidades referentes al mensaje de names.
use super::{
    generic_message::GenericMessage,
    serializer::MessageSerializer,
    utils::{generate_string, validate_channels, validate_command, validate_irc_params_len},
    Command, Executable, FromGeneric, MessageError, Serializable,
};
use crate::irc::{
    constants::{RPL_ENDOFNAMES, RPL_NAMREPLY},
    model::{server::Server, MTClient},
    responses::{builder::ResponseBuilder, ResponseType},
};

#[derive(Debug)]

pub struct Names<'a> {
    pub channels: Option<Vec<&'a [u8]>>,
}

impl<'a> FromGeneric<'a> for Names<'a> {
    fn from_generic(mut generic: GenericMessage<'a>) -> Result<Self, MessageError> {
        validate_command(generic.command, Command::Names)?;
        validate_irc_params_len(&generic.parameters, 1, 0, RPL_ENDOFNAMES)?;

        let mut channels = None;

        if !generic.parameters.is_empty() {
            channels = Option::from(validate_channels(generic.parameters.pop_back())?);
        }

        Ok(Self { channels })
    }
}

impl Serializable for Names<'_> {
    fn serialize(&self) -> String {
        let mut s = MessageSerializer::new(None, Command::Names);

        if let Some(v) = &self.channels {
            s = s.add_csl_params(v);
        }

        s.serialize()
    }
}

impl Executable for Names<'_> {
    fn _execute(&self, server: &Server, client: MTClient) -> Vec<ResponseType> {
        let mut response = ResponseBuilder::new();

        let channel_list: Option<Vec<String>> = self.channels.as_ref().map(|channels| {
            channels
                .iter()
                .map(|channel| generate_string(channel))
                .collect()
        });

        let string_list = match channel_list {
            Some(channel_names) => channel_names
                .iter()
                .flat_map(|channel_name| {
                    server
                        .describe_clients_for_channel(client.clone(), channel_name)
                        .unwrap_or(vec![])
                })
                .collect(),
            None => server.describe_all_client_for_all_channels(client),
        };

        for s in string_list {
            response = response.add_content_for_response(RPL_NAMREPLY, s)
        }

        response =
            response.add_content_for_response(RPL_ENDOFNAMES, "End of /NAMES list".to_owned());
        response.build()
    }
}

#[cfg(test)]
mod test {
    use std::collections::VecDeque;

    use super::*;

    #[test]
    fn test_valid_channel() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();

        parameters.push_back(b"#twilight_zone");

        let generic = GenericMessage {
            command: Command::Names,
            prefix: None,
            parameters,
        };

        let names = Names::from_generic(generic).unwrap();
        let channels = names.channels.unwrap();

        assert_eq!(channels.len(), 1);
        assert_eq!(channels[0], b"#twilight_zone");
    }

    #[test]
    fn test_valid_channel_multiple() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();

        parameters.push_back(b"#oz-ops,&group5");

        let generic = GenericMessage {
            command: Command::Names,
            prefix: None,
            parameters,
        };

        let names = Names::from_generic(generic).unwrap();
        let channels = names.channels.unwrap();

        assert_eq!(channels.len(), 2);
        assert_eq!(channels[0], b"#oz-ops");
        assert_eq!(channels[1], b"&group5");
    }

    #[test]
    fn test_too_many_params() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();

        parameters.push_back(b"#oz-ops,&group5");
        parameters.push_back(b"a");

        let generic = GenericMessage {
            command: Command::Names,
            prefix: None,
            parameters,
        };

        let names = Names::from_generic(generic).unwrap_err();

        assert_eq!(names, MessageError::TooManyParams);
    }

    #[test]
    fn test_no_too_few_params() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::Names,
            prefix: None,
            parameters,
        };

        let names = Names::from_generic(generic).unwrap();

        assert!(names.channels.is_none());
    }

    #[test]
    fn test_err_cmd() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::PrivateMessage,
            prefix: None,
            parameters,
        };

        let names = Names::from_generic(generic).unwrap_err();

        assert_eq!(names, MessageError::InvalidCommand);
    }

    #[test]
    fn test_invalid_channel() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();

        parameters.push_front(b"@channel");

        let generic = GenericMessage {
            command: Command::Names,
            prefix: None,
            parameters,
        };

        let err = Names::from_generic(generic).unwrap_err();

        assert_eq!(err, MessageError::InvalidFormat);
    }
}
