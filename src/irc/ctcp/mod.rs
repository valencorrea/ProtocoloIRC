use std::sync::mpsc::Sender;
use std::thread::JoinHandle;

use crate::gui::{DccCommands, IncomingMessage};

use super::model::workers::dcc_handler::ExecutedAction;

pub mod constants;
pub mod dcc_relay;
pub mod message;
pub mod utils;

pub type DCCHandler = (JoinHandle<()>, Sender<DccCommands>);

#[derive(PartialEq, Eq)]
pub enum ConnectionType {
    Outgoing,
    Incoming,
}

pub trait DccMessage {
    fn execute_new_connection(
        &mut self,
        tx_svtoui: Sender<IncomingMessage>,
        ucid: usize,
        type_of_connection: ConnectionType,
    ) -> Option<DCCHandler>;

    fn execute_existent_connection(&self, tx_uitoc: &Sender<DccCommands>) -> ExecutedAction;

    fn complete_message(&self, original_msg: &str) -> String;
}
