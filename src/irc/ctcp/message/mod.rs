pub mod chat;
pub mod close;
pub mod resume;
pub mod send;

#[derive(PartialEq, Eq, Debug)]
pub enum FileTransferStatus {
    WaitingForInformation,
    From(u64),
    Finished,
}
