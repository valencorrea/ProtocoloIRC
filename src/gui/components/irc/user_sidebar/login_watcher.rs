use std::sync::mpsc::Sender;

use crate::{
    gui::{GuiMessage, IncomingMessage, Reactor},
    ignore,
    irc::constants::RPL_SUCLOGIN,
};

pub struct LoginWatcher {
    tx: Sender<GuiMessage>,
}

impl LoginWatcher {
    pub fn new(tx: Sender<GuiMessage>) -> Self {
        Self { tx }
    }
}

impl Reactor for LoginWatcher {
    fn react_single(&mut self, message: &IncomingMessage) {
        match message {
            IncomingMessage::Server(msg) => {
                if msg.starts_with(&RPL_SUCLOGIN.to_string()) {
                    let _ = self.tx.send(GuiMessage::MessageIRC("NAMES".to_owned()));
                }
            }
            _ => {
                ignore!();
            }
        }
    }
}
