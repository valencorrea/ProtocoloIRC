use crate::gui::{DccCommands, IncomingMessage};
use crate::irc::ctcp::constants::DCC_CLOSE;
use crate::irc::ctcp::utils::{to_notice_command, validate_dcc_params_len};
use crate::irc::ctcp::{ConnectionType, DCCHandler, DccMessage};
use crate::irc::message::generic_message::GenericMessage;
use crate::irc::message::notice::Notice;
use crate::irc::message::utils::generate_string;
use crate::irc::message::FromGeneric;
use crate::irc::model::workers::dcc_handler::ExecutedAction;
use crate::irc::{
    constants::ERR_NEEDMOREPARAMS,
    message::{utils::split_message, MessageError},
};
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub struct DccClose {}

impl DccClose {
    pub fn parse(input: &str) -> Result<Self, MessageError> {
        let tokens = split_message(input);
        validate_dcc_params_len(&tokens, 0, 0, ERR_NEEDMOREPARAMS)?;
        Ok(Self {})
    }
}

impl DccMessage for DccClose {
    fn execute_new_connection(
        &mut self,
        _tx_svtoui: Sender<IncomingMessage>,
        _ucid: usize,
        _type_of_connection: ConnectionType,
    ) -> Option<DCCHandler> {
        None
    }

    fn execute_existent_connection(&self, tx_uitoc: &Sender<DccCommands>) -> ExecutedAction {
        println!(
            "Executing CLOSE for existent conn, with sender {:?}",
            tx_uitoc
        );
        let _ = tx_uitoc.send(DccCommands::Close);
        println!(
            "Executed CLOSE for existent conn, with sender {:?}",
            tx_uitoc
        );
        ExecutedAction::Destroyed
    }

    fn complete_message(&self, original_msg: &str) -> String {
        let notice = Notice::from_generic(GenericMessage::parse(original_msg).unwrap()).unwrap();
        let dcc_close = String::from(DCC_CLOSE);

        to_notice_command(generate_string(notice.nickname), dcc_close)
    }
}
