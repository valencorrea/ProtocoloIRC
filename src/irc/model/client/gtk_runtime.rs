use std::{
    io::{self, BufRead, BufReader},
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
    thread::{self, JoinHandle},
};

use crate::{
    gui::{GuiMessage, IncomingMessage},
    irc::{
        constants::ERR_SERVERERR,
        model::{
            workers::dcc_handler::{ConnectionTypeWrapper, DccMessageHandler, ExecutedAction},
            ServerError,
        },
    },
};

use super::Client;

impl ConnectionTypeWrapper {
    fn should_be_handled(self) -> bool {
        return self == ConnectionTypeWrapper::Outgoing(ExecutedAction::Created)
            || self == ConnectionTypeWrapper::Outgoing(ExecutedAction::Destroyed);
    }
}

impl Client {
    pub fn run_gui_comms(
        mut self,
        rx_from_gui: Receiver<GuiMessage>,
        tx_to_ui: Sender<IncomingMessage>,
    ) -> Result<(), ServerError> {
        let thread = self.listen_from_sv(tx_to_ui.clone())?;

        self.receive_from_gui(rx_from_gui, tx_to_ui);

        self.tcp_destroy()?;

        self.thread_destroy(thread)?;

        Ok(())
    }

    fn receive_from_gui(
        &mut self,
        rx_from_gui: Receiver<GuiMessage>,
        tx_to_ui: Sender<IncomingMessage>,
    ) {
        let mut dcc_handler = DccMessageHandler::init();

        while let Ok(gui_message) = rx_from_gui.recv() {
            match gui_message {
                GuiMessage::Close => break,
                GuiMessage::MessageIRC(sv_message) => {
                    if let Err(e) = self.write_to_sv(&sv_message) {
                        println!("Can't send message to server: {:?}", e);
                        break;
                    }
                }
                _ => {
                    if let Err(e) = self.handle_dcc(&mut dcc_handler, gui_message, tx_to_ui.clone())
                    {
                        println!("Can't handle DCC message: {:?}", e);
                        break;
                    }
                }
            }
        }
    }

    fn handle_dcc(
        &mut self,
        dcc_handler: &mut DccMessageHandler,
        msg: GuiMessage,
        tx_to_ui: Sender<IncomingMessage>,
    ) -> std::io::Result<usize> {
        let (msg, res) = dcc_handler.handle(msg, tx_to_ui);

        if res.should_be_handled() {
            if let Some(n_msg) = DccMessageHandler::get_notice_for_cmd(&msg) {
                return self.write_to_sv(&n_msg);
            }
        }

        Ok(0)
    }

    fn clone_stream(&self) -> Result<TcpStream, ServerError> {
        match &self.stream {
            Some(s) => match s.try_clone() {
                Ok(s) => return Ok(s),
                Err(_) => {
                    self.tcp_destroy()?;
                    return Err(ServerError {
                        code: ERR_SERVERERR,
                        msg: "Can't create enough TCP streams to work properly".to_string(),
                    });
                }
            },
            None => {
                return Err(ServerError {
                    code: ERR_SERVERERR,
                    msg: "Can't create TCP Stream for hollow Client".to_string(),
                })
            }
        }
    }

    fn listen_from_sv(
        &self,
        tx_to_ui: Sender<IncomingMessage>,
    ) -> Result<JoinHandle<()>, ServerError> {
        let stream = self.clone_stream()?;

        Ok(thread::spawn(move || {
            let _ = stream.set_nonblocking(true);
            let mut lines = BufReader::new(stream).lines();
            loop {
                if let Some(line) = lines.next() {
                    match line {
                        Ok(p) => {
                            if let Err(e) = tx_to_ui.send(IncomingMessage::Server(p)) {
                                println!("[CLIENT-MESSAGE SENDER] {:?}", e);
                            }
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            continue;
                        }
                        Err(_) => {
                            println!("Disconnected from the server. Terminating [E1]");
                            break;
                        }
                    }
                } else {
                    println!("Disconnected from the server. Terminating [E2]");
                    break;
                }
            }
        }))
    }
}
