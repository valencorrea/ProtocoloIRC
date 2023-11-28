//! Modulo que se centra en las funcionalidades referentes a la modificacion de canales por parte del server.
use std::{
    io::{stdin, BufRead, BufReader},
    sync::mpsc::Sender,
};

use super::ServerCommand;

pub fn listen_commands(tx: Sender<ServerCommand>) {
    let reader = BufReader::new(stdin());
    for line in reader.lines().flatten() {
        match ServerCommand::parse(&line) {
            Ok(c) => {
                if let ServerCommand::Shutdown = c {
                    if tx.send(c).is_err() {
                        println!("[SERVER] Error while sending the command");
                        continue;
                    }
                    break;
                }
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}
