use crate::gui::{DccCommands, IncomingMessage};
use crate::irc::constants::ERR_NEEDMOREPARAMS;
use crate::irc::ctcp::constants::DCC_RESUME;
use crate::irc::ctcp::utils::{
    to_notice_command, validate_dcc_params_len, validate_filename, validate_port,
    validate_pos_number,
};
use crate::irc::ctcp::{ConnectionType, DCCHandler, DccMessage};
use crate::irc::message::generic_message::GenericMessage;
use crate::irc::message::notice::Notice;
use crate::irc::message::{FromGeneric, MessageError};
use crate::irc::model::workers::dcc_handler::ExecutedAction;
use std::sync::mpsc::Sender;

use crate::irc::message::utils::{generate_string, split_message};

#[derive(Debug)]
pub struct DccResume {
    pub filename: String,
    pub port: String,
    pub position: String,
}

impl DccResume {
    pub fn parse(input: &str) -> Result<Self, MessageError> {
        let mut tokens = split_message(input);
        validate_dcc_params_len(&tokens, 3, 3, ERR_NEEDMOREPARAMS)?;

        let filename = generate_string(validate_filename(tokens.pop_front())?);
        let port = validate_port(tokens.pop_front())?;
        let position = generate_string(validate_pos_number(tokens.pop_front())?);

        Ok(Self {
            filename,
            port,
            position,
        })
    }
}

impl DccMessage for DccResume {
    fn execute_new_connection(
        &mut self,
        _tx_svtoui: Sender<IncomingMessage>,
        _ucid: usize,
        _type_of_connection: ConnectionType,
    ) -> Option<DCCHandler> {
        println!("RESUME: New Connection");
        None
    }

    fn execute_existent_connection(&self, tx_uitoc: &Sender<DccCommands>) -> ExecutedAction {
        println!("RESUME: Existant Connection");
        match self.position.to_owned().parse() {
            Ok(p) => {
                let _ = tx_uitoc.send(DccCommands::FileTransfer(super::FileTransferStatus::From(
                    p,
                )));
                ()
            }
            Err(e) => {
                println!("Error parseando un int????: {:?}", e);
                ()
            }
        };
        ExecutedAction::Created
    }

    fn complete_message(&self, original_msg: &str) -> String {
        let notice = Notice::from_generic(GenericMessage::parse(original_msg).unwrap()).unwrap();
        let dcc_resume = format!(
            "{} {} {} {}",
            DCC_RESUME, self.filename, self.port, self.position
        );

        to_notice_command(generate_string(notice.nickname), dcc_resume)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::irc::ctcp::constants::ERR_INVALIDPORT;
    use crate::irc::message::MessageError::{DCCDefined, InvalidFormat, TooManyParams};

    #[test]
    fn test_resume_no_params_error() {
        let input = String::new();

        let err = DccResume::parse(input.as_str()).unwrap_err();

        assert_eq!(err, DCCDefined(ERR_NEEDMOREPARAMS));
    }

    #[test]
    fn test_resume_less_needed_params_error() {
        let input = String::from("file_name");

        let err = DccResume::parse(input.as_str()).unwrap_err();

        assert_eq!(err, DCCDefined(ERR_NEEDMOREPARAMS));
    }

    #[test]
    fn test_resume_more_needed_params_error() {
        let input = String::from("first-param second-param third-param fourth-param");

        let err = DccResume::parse(input.as_str()).unwrap_err();

        assert_eq!(err, TooManyParams);
    }

    #[test]
    fn test_resume_invalid_file_name_error() {
        let input = String::from("file/name 9290 128");

        let err = DccResume::parse(input.as_str()).unwrap_err();

        assert_eq!(err, InvalidFormat);
    }

    #[test]
    fn test_resume_invalid_port_min_len_nedeed_error() {
        let input = String::from("fileName 1 64");

        let err = DccResume::parse(input.as_str()).unwrap_err();

        assert_eq!(err, DCCDefined(ERR_INVALIDPORT));
    }

    #[test]
    fn test_resume_invalid_port_max_len_nedeed_error() {
        let input = String::from("fileName 929090 64");

        let err = DccResume::parse(input.as_str()).unwrap_err();

        assert_eq!(err, DCCDefined(ERR_INVALIDPORT));
    }
}
