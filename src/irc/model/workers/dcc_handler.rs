use std::{collections::HashMap, sync::mpsc::Sender, thread::JoinHandle};

use crate::{
    gui::{DccCommands, GuiMessage, IncomingMessage},
    irc::ctcp::{
        utils::{to_dcc_command, to_dcc_command_from_notice},
        ConnectionType, DCCHandler, DccMessage,
    },
};

#[derive(PartialEq, Eq, Debug)]
pub enum ExecutedAction {
    NoAction,
    Created,
    Sent,
    Destroyed,
}

#[derive(PartialEq, Eq)]
pub enum ConnectionTypeWrapper {
    Invalid,
    Incoming(ExecutedAction),
    Outgoing(ExecutedAction),
}

pub struct DccMessageHandler {
    dcc_connections: HashMap<usize, DCCHandler>,
    finished_conns: Vec<JoinHandle<()>>,
}
///  Action Mapping
///             |                Incoming               |               Outgoing                |
///             |No Existant        |           Existant|No Existant        |           Existant|        
/// SEND        |NoAction           |Created            |Created            |NoAction           |     
/// CHAT        |Created            |NoAction           |Created            |NoAction           |
/// RESUME      |NoAction           |NoAction(Setup)    |NoAction           |NoAction           |
/// CLOSE       |NoAction           |Destroyed          |NoAction           |Destroyed          |
/// *: Depending on if it the file exists
/// NoAction does not neccesarily means no internal operation is done, though no operation is the norm it should not be expected.
impl DccMessageHandler {
    pub fn init() -> Self {
        Self {
            dcc_connections: HashMap::new(),
            finished_conns: Vec::new(),
        }
    }

    pub fn handle(
        &mut self,
        gui_message: GuiMessage,
        tx_svtoui: Sender<IncomingMessage>,
    ) -> (GuiMessage, ConnectionTypeWrapper) {
        match gui_message {
            GuiMessage::OutgoingDCC(to, dcc_msg) => {
                let command = Self::parse_dcc_command(&dcc_msg);
                if command.is_none() {
                    return (
                        GuiMessage::OutgoingDCC(to, dcc_msg.to_owned()),
                        ConnectionTypeWrapper::Outgoing(self.send_to_connection(to, &dcc_msg)),
                    );
                }
                let mut cmd = command.unwrap();
                let ret = self.handle_cmd_for_existent(to, &cmd).unwrap_or_else(|| {
                    self.handle_cmd_for_new(to, ConnectionType::Outgoing, &mut cmd, tx_svtoui)
                });
                (
                    GuiMessage::OutgoingDCC(to, cmd.complete_message(&dcc_msg)),
                    ConnectionTypeWrapper::Outgoing(ret),
                )
            }
            GuiMessage::IncomingDCC(to, dcc_msg) => {
                let command = Self::parse_dcc_command(&dcc_msg);
                if command.is_none() {
                    return (
                        GuiMessage::IncomingDCC(to, dcc_msg.to_owned()),
                        ConnectionTypeWrapper::Incoming(self.send_to_connection(to, &dcc_msg)),
                    );
                }
                let mut cmd = command.unwrap();
                let ret = self.handle_cmd_for_existent(to, &cmd).unwrap_or_else(|| {
                    self.handle_cmd_for_new(to, ConnectionType::Incoming, &mut cmd, tx_svtoui)
                });
                (
                    GuiMessage::IncomingDCC(to, "can't touch this".to_string()),
                    ConnectionTypeWrapper::Incoming(ret),
                )
            }
            _ => return (GuiMessage::Close, ConnectionTypeWrapper::Invalid),
        }
    }

    pub fn get_notice_for_cmd(gui_message: &GuiMessage) -> Option<String> {
        match gui_message {
            GuiMessage::OutgoingDCC(_, cmd) | GuiMessage::IncomingDCC(_, cmd) => {
                Some(cmd.to_string())
            }
            _ => None,
        }
    }

    fn finish_connection(&mut self, id: usize) {
        if let Some((jh, _)) = self.dcc_connections.remove(&id) {
            self.finished_conns.push(jh);
        }
    }

    fn parse_dcc_command(possible_cmd: &str) -> Option<Box<dyn DccMessage>> {
        to_dcc_command(possible_cmd).or_else(|| to_dcc_command_from_notice(possible_cmd))
    }

    fn handle_cmd_for_existent(
        &mut self,
        id: usize,
        cmd: &Box<dyn DccMessage>,
    ) -> Option<ExecutedAction> {
        println!(
            "Intentando mandar para conexion existente: {} {:?}",
            id, self.dcc_connections
        );
        let (_, tx) = self.dcc_connections.get(&id)?;
        let action = cmd.execute_existent_connection(tx);
        if action == ExecutedAction::Destroyed {
            self.finish_connection(id);
        }
        Some(action)
    }

    fn handle_cmd_for_new(
        &mut self,
        id: usize,
        toc: ConnectionType,
        cmd: &mut Box<dyn DccMessage>,
        tx_svtoui: Sender<IncomingMessage>,
    ) -> ExecutedAction {
        let dcc_handler = cmd.execute_new_connection(tx_svtoui, id, toc);

        if let Some(handler) = dcc_handler {
            self.dcc_connections.insert(id, handler);
            return ExecutedAction::Created;
        }

        ExecutedAction::NoAction
    }

    fn send_to_connection(&self, id: usize, msg: &str) -> ExecutedAction {
        if let Some((_, tx)) = self.dcc_connections.get(&id) {
            let _ = tx.send(DccCommands::Message(msg.as_bytes().to_vec()));
            return ExecutedAction::Sent;
        }
        ExecutedAction::NoAction
    }
}
