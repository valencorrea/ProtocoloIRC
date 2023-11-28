use std::{net::Shutdown, thread::JoinHandle};

use crate::irc::{constants::ERR_SERVERERR, model::ServerError};

use super::Client;

impl Client {
    pub fn tcp_destroy(&self) -> Result<(), ServerError> {
        if let Some(stream) = &self.stream {
            match stream.shutdown(Shutdown::Both) {
                Ok(_) => {
                    println!("[CLIENT] Cleaned up TCP connection");
                }
                Err(_) => {
                    return Err(ServerError {
                        code: ERR_SERVERERR,
                        msg: "Internal server error".to_string(),
                    })
                }
            }
        }
        Ok(())
    }

    pub fn thread_destroy(&self, thread: JoinHandle<()>) -> Result<(), ServerError> {
        match thread.join() {
            Ok(_) => {
                println!("[CLIENT] Cleaned up used threads");
                Ok(())
            }
            Err(_) => Err(ServerError {
                code: ERR_SERVERERR,
                msg: "Can't properly clean up thread".to_string(),
            }),
        }
    }
}
