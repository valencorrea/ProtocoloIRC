//! Modulo que se centra en las funcionalidades referentes al mensaje de list.
use std::sync::{Arc, Mutex};

use crate::irc::constants::{ERR_NOSUCHSERVER, RPL_LIST, RPL_LISTEND, RPL_LISTSTART};
use crate::irc::message::utils::{
    validate_channels, validate_command, validate_irc_params_len, validate_name_valid_none,
};
use crate::irc::message::GenericMessage;
use crate::irc::message::{Command, Executable, FromGeneric, MessageError};
use crate::irc::model::client::Client;
use crate::irc::model::server::Server;
use crate::irc::responses::builder::ResponseBuilder;
use crate::irc::responses::ResponseType;

use super::serializer::MessageSerializer;
use super::utils::generate_string;
use super::Serializable;

#[derive(Debug)]
pub struct List<'a> {
    pub channels: Option<Vec<&'a [u8]>>,
    pub server: Option<&'a [u8]>,
}

impl<'a> FromGeneric<'a> for List<'a> {
    /// Constructor de mensaje List a partir de un mensaje generico.
    /// Tanto la lista de canales como el nombre del servidor pueden no ser enviados.
    /// Si se envia un nombre de servidor, entonces se debe pasar una lista de canales; no es
    /// necesario a la inversa.
    /// Devuelve error si el comando no es List, o si el largo de los parametros supera el maximo.
    fn from_generic(mut generic: GenericMessage<'a>) -> Result<Self, MessageError> {
        validate_command(generic.command, Command::List)?;
        validate_irc_params_len(&generic.parameters, 2, 0, ERR_NOSUCHSERVER)?;

        let mut server = None;
        let mut channels = None;

        if generic.parameters.len() > 1 {
            server = validate_name_valid_none(generic.parameters.pop_back())?;
        }

        if !generic.parameters.is_empty() {
            channels = Option::from(validate_channels(generic.parameters.pop_back())?);
        }

        Ok(Self { channels, server })
    }
}

impl Serializable for List<'_> {
    fn serialize(&self) -> String {
        let mut s = MessageSerializer::new(None, Command::List);

        if let Some(channels) = &self.channels {
            s = s.add_csl_params(channels)
        };

        if let Some(server) = self.server {
            s = s.add_parameter(server);
        }

        s.serialize()
    }
}

impl Executable for List<'_> {
    fn _execute(&self, server: &Server, client: Arc<Mutex<Client>>) -> Vec<ResponseType> {
        let mut response = ResponseBuilder::new();
        response =
            response.add_content_for_response(RPL_LISTSTART, "Channel :Users  Name".to_owned());

        let desc_list = match &self.channels {
            Some(channels) => {
                let channel_list = channels.iter().map(|ch| generate_string(ch)).collect();

                server.get_channels_strings_by_names(client, channel_list)
            }
            None => server.get_all_channels_string(client),
        }
        .into_iter()
        .flatten()
        .collect::<Vec<String>>();

        for desc in desc_list {
            response = response.add_content_for_response(RPL_LIST, desc);
        }

        response = response.add_content_for_response(RPL_LISTEND, "End of /LIST".to_owned());

        response.build()
    }
}

#[cfg(test)]
mod list_parse_tests {
    use crate::irc::message::list::List;
    use crate::irc::message::GenericMessage;
    use crate::irc::message::MessageError::InvalidCommand;
    use crate::irc::message::{Command, FromGeneric, MessageError};
    use std::collections::VecDeque;

    #[test]
    fn list_with_one_multiple_server_channel_ok() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"servername");
        parameters.push_front(b"#channel");
        let generic = GenericMessage {
            command: Command::List,
            prefix: None,
            parameters,
        };

        let list = List::from_generic(generic).unwrap();
        let channels = list.channels.unwrap();

        assert_eq!(channels[0], b"#channel");
        assert_eq!(channels.len(), 1);
        assert_eq!(list.server.unwrap(), b"servername");
    }

    #[test]
    fn list_with_one_single_server_channel_ok() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"servername");
        parameters.push_front(b"&channel");
        let generic = GenericMessage {
            command: Command::List,
            prefix: None,
            parameters,
        };

        let list = List::from_generic(generic).unwrap();
        let channels = list.channels.unwrap();

        assert_eq!(channels[0], b"&channel");
        assert_eq!(channels.len(), 1);
        assert_eq!(list.server.unwrap(), b"servername");
    }

    #[test]
    fn list_with_many_multiple_server_channel_ok() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"servername");
        parameters.push_front(b"#channel1,#channel2,#channel3,#channel4,#channel5");
        let generic = GenericMessage {
            command: Command::List,
            prefix: None,
            parameters,
        };

        let list = List::from_generic(generic).unwrap();
        let channels = list.channels.unwrap();

        assert_eq!(channels[0], b"#channel1");
        assert_eq!(channels[1], b"#channel2");
        assert_eq!(channels[2], b"#channel3");
        assert_eq!(channels[3], b"#channel4");
        assert_eq!(channels[4], b"#channel5");
        assert_eq!(channels.len(), 5);
        assert_eq!(list.server.unwrap(), b"servername");
    }

    #[test]
    fn list_with_many_single_server_channel_ok() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"servername");
        parameters.push_front(b"&channel1,&channel2,&channel3,&channel4,&channel5");
        let generic = GenericMessage {
            command: Command::List,
            prefix: None,
            parameters,
        };

        let list = List::from_generic(generic).unwrap();
        let channels = list.channels.unwrap();

        assert_eq!(channels[0], b"&channel1");
        assert_eq!(channels[1], b"&channel2");
        assert_eq!(channels[2], b"&channel3");
        assert_eq!(channels[3], b"&channel4");
        assert_eq!(channels[4], b"&channel5");
        assert_eq!(channels.len(), 5);
        assert_eq!(list.server.unwrap(), b"servername");
    }

    #[test]
    fn list_validate_command_error() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::User,
            prefix: None,
            parameters,
        };

        let err = List::from_generic(generic).unwrap_err();

        assert_eq!(err, InvalidCommand);
    }

    #[test]
    fn list_with_many_parameters_error() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"servername");
        parameters.push_front(b"servername");
        parameters.push_front(b"other parameter");
        parameters.push_front(b"another parameter");

        let generic = GenericMessage {
            command: Command::List,
            prefix: None,
            parameters,
        };

        let err = List::from_generic(generic).unwrap_err();

        assert_eq!(err, MessageError::TooManyParams);
    }

    #[test]
    fn list_with_no_parameters_ok() {
        let parameters: VecDeque<&[u8]> = VecDeque::new();

        let generic = GenericMessage {
            command: Command::List,
            prefix: None,
            parameters,
        };

        let list = List::from_generic(generic).unwrap();

        assert!(list.server.is_none());
        assert!(list.channels.is_none());
    }

    #[test]
    fn list_with_invalid_channel_error() {
        let mut parameters: VecDeque<&[u8]> = VecDeque::new();
        parameters.push_front(b"servername");
        parameters.push_front(b"@channel");
        let generic = GenericMessage {
            command: Command::List,
            prefix: None,
            parameters,
        };

        let err = List::from_generic(generic).unwrap_err();

        assert_eq!(err, MessageError::InvalidFormat);
    }
}
