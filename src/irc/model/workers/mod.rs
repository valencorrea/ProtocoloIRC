use std::{
    collections::HashMap,
    thread::{JoinHandle, ThreadId},
};

pub mod client_management;
pub mod dcc_handler;
pub mod persistency;
pub mod server_command;

pub type ThreadMap = HashMap<ThreadId, JoinHandle<()>>;
pub enum ThreadManagement {
    KeepTrack(JoinHandle<()>),
    Clean(ThreadId),
    KillAll,
}

pub enum ServerCommand {
    Shutdown,
    Persisting,
    NormalOperation,
}

impl ServerCommand {
    pub fn parse(line: &str) -> Result<ServerCommand, String> {
        let u = line.to_ascii_uppercase();
        match u.as_str() {
            "SHUTDOWN" => Ok(ServerCommand::Shutdown),
            _ => {
                Err("[SERVER - COMMAND] Invalid Command\nValid Commands:\n\t- SHUTDOWN".to_owned())
            }
        }
    }
}
