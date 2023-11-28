use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, RecvTimeoutError, Sender};
use std::thread::{self};
use std::time::Duration;

use crate::gui::{DccCommands, IncomingMessage};
use crate::irc::constants::ERR_NEEDMOREPARAMS;
use crate::irc::ctcp::constants::{CHAT_PROTOCOL, DCC_CHAT};
use crate::irc::ctcp::utils::{
    to_notice_command, validate_dcc_params_len, validate_ip, validate_port, validate_protocol,
};
use crate::irc::message::generic_message::GenericMessage;
use crate::irc::message::notice::Notice;
use crate::irc::message::utils::{generate_string, split_message};
use crate::irc::message::{FromGeneric, MessageError};

use crate::irc::ctcp::{ConnectionType, DCCHandler, DccMessage};
use crate::irc::model::workers::dcc_handler::ExecutedAction;

#[derive(Debug)]
pub struct DccChat {
    pub protocol: String,
    pub ip: Option<String>,
    pub port: Option<String>,
}

enum TcpWrapper {
    L(TcpListener),
    S(TcpStream),
}

impl DccChat {
    pub fn parse(input: &str) -> Result<Self, MessageError> {
        let mut ip = None;
        let mut port = None;

        let mut tokens = split_message(input);
        validate_dcc_params_len(&tokens, 3, 1, ERR_NEEDMOREPARAMS)?;

        let protocol = generate_string(validate_protocol(tokens.pop_front())?);
        if !tokens.is_empty() {
            ip = Some(generate_string(validate_ip(tokens.pop_front())?));
            port = Some(validate_port(tokens.pop_front())?);
        }

        Ok(Self { protocol, ip, port })
    }
}

impl DccMessage for DccChat {
    fn complete_message(&self, original_msg: &str) -> String {
        println!(
            "Completing message for DCC CHAT with IP: {} and PORT: {}",
            self.ip.as_ref().unwrap(),
            self.port.as_ref().unwrap()
        );

        let notice = Notice::from_generic(GenericMessage::parse(original_msg).unwrap()).unwrap();
        let dcc_chat = format!(
            "{} {} {} {}",
            DCC_CHAT,
            CHAT_PROTOCOL,
            self.ip.as_ref().unwrap(),
            self.port.as_ref().unwrap()
        );

        to_notice_command(generate_string(notice.nickname), dcc_chat)
    }

    fn execute_new_connection(
        &mut self,
        tx_svtoui: Sender<IncomingMessage>,
        ucid: usize,
        type_of_connection: ConnectionType,
    ) -> Option<DCCHandler> {
        match type_of_connection {
            ConnectionType::Outgoing => {
                if let Ok(listener) = TcpListener::bind("127.0.0.1:0") {
                    self.ip = Some(listener.local_addr().unwrap().ip().to_string());
                    self.port = Some(listener.local_addr().unwrap().port().to_string());

                    let h = self.launch_thread(
                        tx_svtoui,
                        TcpWrapper::L(listener),
                        ucid,
                        Duration::from_millis(100),
                        Duration::from_millis(100),
                    );
                    return Some(h);
                }

                None
            }
            ConnectionType::Incoming => {
                let w = TcpWrapper::S(
                    TcpStream::connect(format!("{}:{}", self.ip.as_ref()?, &self.port.as_ref()?))
                        .ok()?,
                );
                Some(self.launch_thread(
                    tx_svtoui,
                    w,
                    ucid,
                    Duration::from_millis(100),
                    Duration::from_millis(100),
                ))
            }
        }
    }

    fn execute_existent_connection(&self, _tx_uitoc: &Sender<DccCommands>) -> ExecutedAction {
        ExecutedAction::NoAction
    }
}

impl DccChat {
    fn launch_thread(
        &self,
        tx_svtoui: Sender<IncomingMessage>,
        wrapper: TcpWrapper,
        ucid: usize,
        tcp_stream_timeout: Duration,
        ui_recv: Duration,
    ) -> DCCHandler {
        let (tx_uitoc, rx_uitoc) = channel::<DccCommands>();

        let thread = thread::spawn(move || {
            let mut buff: Vec<u8> = vec![0; 65535];

            if let Some(tcp_stream) = Self::get_stream(wrapper) {
                let mut write_stream = tcp_stream.try_clone().unwrap();

                let mut read_stream = tcp_stream;
                read_stream
                    .set_read_timeout(Some(tcp_stream_timeout))
                    .unwrap();

                'main_loop: loop {
                    match read_stream.read(&mut buff) {
                        Ok(x) => {
                            if x > 0 {
                                if let Err(_) = tx_svtoui.send(IncomingMessage::Client(
                                    ucid,
                                    generate_string(&buff[0..x]),
                                )) {
                                    break 'main_loop;
                                };
                            }
                        }
                        Err(e) => {
                            if e.kind() != ErrorKind::WouldBlock && e.kind() != ErrorKind::TimedOut
                            {
                                break 'main_loop;
                            }
                        }
                    }

                    match rx_uitoc.recv_timeout(ui_recv) {
                        Ok(cmd) => match cmd {
                            DccCommands::Close => break 'main_loop,
                            DccCommands::Message(msg) => {
                                if let Err(_) = write_stream.write(&msg) {
                                    break 'main_loop;
                                };
                            }
                            DccCommands::FileTransfer(_) => {}
                        },
                        Err(e) => {
                            if e != RecvTimeoutError::Timeout {
                                break 'main_loop;
                            }
                        }
                    }
                }

                let _ = write_stream.shutdown(std::net::Shutdown::Both);
            }
        });

        (thread, tx_uitoc)
    }

    fn get_stream(tcp: TcpWrapper) -> Option<TcpStream> {
        match tcp {
            TcpWrapper::L(listener) => {
                if let Ok((tcp_stream, _)) = listener.accept() {
                    return Some(tcp_stream);
                }
                None
            }
            TcpWrapper::S(s) => Some(s),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::irc::ctcp::constants::{ERR_INVALIDIP, ERR_INVALIDPORT, ERR_INVALIDPROTOCOL};
    use crate::irc::message::MessageError::{DCCDefined, TooManyParams};

    #[test]
    fn test_chat_no_params_error() {
        let input = String::new();

        let err = DccChat::parse(input.as_str()).unwrap_err();

        assert_eq!(err, DCCDefined(ERR_NEEDMOREPARAMS));
    }

    #[test]
    fn test_chat_less_needed_params_error() {
        let input = String::new();

        let err = DccChat::parse(input.as_str()).unwrap_err();

        assert_eq!(err, DCCDefined(ERR_NEEDMOREPARAMS));
    }

    #[test]
    fn test_chat_more_needed_params_error() {
        let input = String::from("first-param second-param third-param fourth-param");

        let err = DccChat::parse(input.as_str()).unwrap_err();

        assert_eq!(err, TooManyParams);
    }

    #[test]
    fn test_chat_invalid_protocol() {
        let input = String::from("SOMETHING");

        let err = DccChat::parse(input.as_str()).unwrap_err();

        assert_eq!(err, DCCDefined(ERR_INVALIDPROTOCOL));
    }

    #[test]
    fn test_chat_invalid_ip_ending_error() {
        let input = String::from("CHAT 1.2.3. 9290");

        let err = DccChat::parse(input.as_str()).unwrap_err();

        assert_eq!(err, DCCDefined(ERR_INVALIDIP));
    }

    #[test]
    fn test_chat_invalid_ip_start_error() {
        let input = String::from("CHAT .1.2.3 9290");

        let err = DccChat::parse(input.as_str()).unwrap_err();

        assert_eq!(err, DCCDefined(ERR_INVALIDIP));
    }

    #[test]
    fn test_chat_invalid_ip_len_error() {
        let input = String::from("CHAT 1.2.3.4.5.6.7.8.9.10.11 9290");

        let err = DccChat::parse(input.as_str()).unwrap_err();

        assert_eq!(err, DCCDefined(ERR_INVALIDIP));
    }

    #[test]
    fn test_chat_invalid_port_min_len_nedeed_error() {
        let input = String::from("CHAT 1.2.3 1");

        let err = DccChat::parse(input.as_str()).unwrap_err();

        assert_eq!(err, DCCDefined(ERR_INVALIDPORT));
    }

    #[test]
    fn test_chat_invalid_port_max_len_nedeed_error() {
        let input = String::from("CHAT 1.2.3 929090");

        let err = DccChat::parse(input.as_str()).unwrap_err();

        assert_eq!(err, DCCDefined(ERR_INVALIDPORT));
    }
}
