// Super Main

mod gui;
mod irc;

use irc::model::WHAT_TO_RUN_POS;
use irc::responses::{InternalType, ResponseType};

use crate::irc::model::client::Client;
use crate::irc::model::server::Server;
use crate::irc::model::ConnectionError;
use gui::components::irc::ircwindow::run_app;
use std::env::args;
use std::io::stdin;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;

fn main() -> Result<(), ConnectionError> {
    let argv = args().collect::<Vec<String>>();

    if argv[WHAT_TO_RUN_POS] == "server" {
        // run as server
        let mut s = match Server::create(&argv) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        if let Err(reason) = s.load_file() {
            println!("[SERVER - PERSISTENCY] Can't load datafile {}", reason)
        }

        match Server::server_run(Arc::new(s)) {
            Ok(_) => {}
            Err(_) => return Err(ConnectionError::InternalServerError),
        }
    } else if argv[WHAT_TO_RUN_POS] == "server-connect" {
        let mut s = match Server::create(&argv) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        if let Err(reason) = s.load_file() {
            println!("[SERVER - PERSISTENCY] Can't load datafile {}", reason)
        }

        let s = Arc::new(s);
        let (tx, th) = Server::server_connect(s.clone(), &argv)?;

        match Server::server_run(s) {
            Ok(_) => {}
            Err(_) => return Err(ConnectionError::InternalServerError),
        }
        let _ = tx.send(ResponseType::InternalResponse(InternalType::Quit));
        let _ = th.join();
    } else if argv[WHAT_TO_RUN_POS] == "client" {
        // run as client
        let c = match Client::create_fromargs(argv) {
            Ok(v) => v,
            Err(_) => {
                println!("[CLIENT] Can't properly create client");
                return Err(ConnectionError::InternalServerError);
            }
        };

        // GTK-CLIENT to SERVER
        let (tx_uitosv, rx_uitosv) = channel();

        // SERVER to CLIENT-GTK
        let (tx_svtoui, rx_svtoui) = channel();

        let client_thread = thread::spawn(move || {
            if let Err(e) = c.run_gui_comms(rx_uitosv, tx_svtoui) {
                println!("[CLIENT] Can't run from GUI {:?}", e);
            };
        });

        run_app(tx_uitosv, rx_svtoui);

        let tid = client_thread.thread().id();

        match client_thread.join() {
            Ok(_) => println!("[CLIENT - THREAD MANAGEMENT]: Cleaning thread {:?}", tid),
            Err(e) => println!(
                "[SERVER - THREAD MANAGEMENT]: Couldn't clean thread {:?}, {:?}",
                tid, e
            ),
        };
    } else if argv[WHAT_TO_RUN_POS] == "client-no-gui" {
        let c = match Client::create_fromargs(argv) {
            Ok(v) => v,
            Err(_) => {
                println!("[CLIENT] Can't properly create client");
                return Err(ConnectionError::InternalServerError);
            }
        };

        if let Err(e) = c.client_run(&mut stdin()) {
            println!("{:?}", e);
        };
    }

    Ok(())
}
