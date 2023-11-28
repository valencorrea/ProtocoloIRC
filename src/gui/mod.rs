use crate::irc::ctcp::message::FileTransferStatus;

pub mod actors;
pub mod components;
pub mod constants;
pub mod message_hub;
pub mod utils;

type Message = String;
type RawMessage = Vec<u8>;
type UcID = usize;
type TotalSize = u64;
type Transferred = u64;
pub enum GuiMessage {
    Close,
    MessageIRC(Message),
    OutgoingDCC(UcID, Message),
    IncomingDCC(UcID, Message),
}

#[derive(Debug)]
pub enum IncomingMessage {
    Server(Message),
    Client(UcID, Message),
    Resume(UcID, String, usize, String),
    ClientFile(UcID, TotalSize, Transferred),
}

pub enum DccCommands {
    Close,
    Message(RawMessage),
    FileTransfer(FileTransferStatus),
}

pub trait Reactor {
    fn react(&mut self, messages: &[IncomingMessage]) {
        for message in messages {
            self.react_single(message)
        }
    }

    fn react_single(&mut self, message: &IncomingMessage);
}
