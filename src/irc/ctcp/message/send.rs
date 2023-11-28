use crate::gui::{DccCommands, IncomingMessage};
use crate::ignore;
use crate::irc::constants::ERR_NEEDMOREPARAMS;
use crate::irc::ctcp::constants::DCC_SEND;
use crate::irc::ctcp::utils::{
    to_notice_command, validate_dcc_params_len, validate_ip, validate_port, validate_pos_number,
    PATH_FILES_DOWNLOAD, PATH_FILES_UPLOAD,
};
use crate::irc::message::generic_message::GenericMessage;
use crate::irc::message::notice::Notice;
use crate::irc::message::utils::{generate_string, split_message};
use crate::irc::message::{FromGeneric, MessageError};
use crate::irc::model::workers::dcc_handler::ExecutedAction;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;

use crate::irc::ctcp::utils::validate_filename;
use crate::irc::ctcp::{ConnectionType, DCCHandler, DccMessage};
use std::path::{Path, PathBuf};

use super::FileTransferStatus;

#[derive(Debug)]
pub struct DccSend {
    pub file_name: String, // todo una sola palabra
    pub file_path: String,
    pub ip: Option<String>,
    pub port: Option<String>,
    pub file_size: String,
}

impl DccSend {
    pub fn parse(input: &str) -> Result<Self, MessageError> {
        let mut ip = None;
        let mut port = None;

        let mut tokens = split_message(input);
        validate_dcc_params_len(&tokens, 4, 2, ERR_NEEDMOREPARAMS)?;

        let file_name = generate_string(validate_filename(tokens.pop_front())?);

        let file_path = PathBuf::from(format!("{}/{}", PATH_FILES_UPLOAD, file_name))
            .into_os_string()
            .to_str()
            .expect("Couldn't get entire path name")
            .to_string();

        println!("file_name is: {}", file_name);

        if tokens.len() > 1 {
            ip = Some(generate_string(validate_ip(tokens.pop_front())?));
            port = Some(validate_port(tokens.pop_front())?);
        }
        let file_size = generate_string(validate_pos_number(tokens.pop_front())?); // todo ver el type
        Ok(Self {
            file_name,
            file_path,
            ip,
            port,
            file_size,
        })
    }
}

impl DccMessage for DccSend {
    fn complete_message(&self, original_msg: &str) -> String {
        let notice = Notice::from_generic(GenericMessage::parse(original_msg).unwrap()).unwrap();
        let dcc_send = format!(
            "{} {} {} {} {}",
            DCC_SEND,
            self.file_name,
            self.ip.as_ref().unwrap(), // todo refactor de los unwrap
            self.port.as_ref().unwrap(),
            self.file_size
        );

        to_notice_command(generate_string(notice.nickname), dcc_send)
    }

    fn execute_new_connection(
        &mut self,
        tx_svtoui: Sender<IncomingMessage>,
        ucid: usize,
        type_of_connection: ConnectionType,
    ) -> Option<DCCHandler> {
        // println!("SEND: New Connection");
        if !Path::new(&self.file_path).exists() {
            println!(
                "Problemas con el file name. Lo que llego: {}",
                self.file_name
            );
            return None;
        }

        match type_of_connection {
            ConnectionType::Outgoing => {
                if let Ok(listener) = TcpListener::bind("127.0.0.1:0") {
                    self.ip = Some(listener.local_addr().unwrap().ip().to_string());
                    self.port = Some(listener.local_addr().unwrap().port().to_string());

                    Some(self.launch_thread_for_sender(
                        tx_svtoui,
                        listener,
                        ucid,
                        Duration::from_millis(100),
                    ))
                } else {
                    None
                }
            }
            ConnectionType::Incoming => {
                let tcp_stream =
                    TcpStream::connect(format!("{}:{}", self.ip.as_ref()?, &self.port.as_ref()?))
                        .ok()?;

                Some(self.launch_thread_for_receiver(tx_svtoui, tcp_stream, ucid))
            }
        }
    }

    fn execute_existent_connection(&self, _tx_uitoc: &Sender<DccCommands>) -> ExecutedAction {
        println!("SEND: Existent Connection");
        ExecutedAction::NoAction
    }
}

impl DccSend {
    fn launch_thread_for_sender(
        &self,
        tx_svtoui: Sender<IncomingMessage>,
        listener: TcpListener,
        ucid: usize,
        tcp_stream_timeout: Duration,
    ) -> DCCHandler {
        let (tx_uitoc, rx_uitoc) = channel::<DccCommands>();
        let filename = self.file_path.to_owned();
        let thread = thread::spawn(move || {
            let (mut bufreader, file_size) = match File::open(filename.clone()) {
                Ok(f) => {
                    let b = BufReader::new(f);
                    let s = match fs::metadata(filename) {
                        Ok(metada) => metada.len(),
                        Err(e) => {
                            println!(
                                "[CTC - ERROR]: Can't open file to send \n [CTC - ERROR] {:?}",
                                e
                            );
                            return;
                        }
                    };

                    (b, s)
                }
                Err(e) => {
                    println!(
                        "[CTC - ERROR]: Can't open file to send \n [CTC - ERROR] {:?}",
                        e
                    );
                    return;
                }
            };

            let mut transfer_status = FileTransferStatus::WaitingForInformation;

            if let Ok((mut tcp_stream, _)) = listener.accept() {
                let mut buf = vec![0; 1024];

                if let Err(_) = tcp_stream.set_read_timeout(Some(tcp_stream_timeout)) {
                    return;
                };

                if let Err(_) = tcp_stream.set_write_timeout(Some(tcp_stream_timeout)) {
                    return;
                }

                'main_loop: loop {
                    match transfer_status {
                        FileTransferStatus::WaitingForInformation => {
                            match rx_uitoc.recv() {
                                Ok(s) => {
                                    match s {
                                        DccCommands::Close => {
                                            break 'main_loop;
                                        }
                                        DccCommands::Message(_) => {
                                            //Ignore
                                        }
                                        DccCommands::FileTransfer(ft) => {
                                            match ft {
                                                FileTransferStatus::From(f) => {
                                                    let _ = bufreader
                                                        .seek_relative(f.try_into().unwrap());
                                                }
                                                _ => {
                                                    ignore!();
                                                }
                                            }
                                            transfer_status = ft;
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("[CTC - ERROR]: Can't receive information \n [CTC - ERROR] {:?}", e);
                                    break 'main_loop;
                                }
                            }
                        }
                        FileTransferStatus::From(from) => {
                            if from >= file_size {
                                let _ = tx_svtoui
                                    .send(IncomingMessage::ClientFile(ucid, file_size, file_size));
                                break 'main_loop;
                            }

                            let read_size = if file_size - from > 1024 {
                                transfer_status = FileTransferStatus::From(from + 1024);
                                1024
                            } else {
                                buf = vec![0; (file_size - from).try_into().unwrap()];
                                transfer_status = FileTransferStatus::Finished;
                                file_size - from
                            };
                            if let Err(e) = bufreader.read_exact(&mut buf) {
                                println!("[CTC - ERROR]: Unexpected error while reading file\n [CTC - ERROR] {:?}", e);
                                break 'main_loop;
                            };

                            if let Err(e) = tcp_stream.write(&buf) {
                                println!("[CTC - ERROR]: Unexpected error while writing to client\n [CTC - ERROR] {:?}", e);
                                break 'main_loop;
                            }

                            if let Err(e) = tx_svtoui.send(IncomingMessage::ClientFile(
                                ucid,
                                file_size,
                                from + read_size,
                            )) {
                                println!("[CTC - ERROR]: Unexpected error while writing to UI. Transfer can continue\n [CTC - ERROR] {:?}", e);
                            };
                        }
                        FileTransferStatus::Finished => {
                            break 'main_loop;
                        }
                    }
                    match rx_uitoc.recv_timeout(Duration::from_nanos(10)) {
                        Ok(c) => match c {
                            DccCommands::Close => break 'main_loop,
                            _ => {
                                //ignore
                            }
                        },
                        Err(e) => {
                            if e != RecvTimeoutError::Timeout {
                                break 'main_loop;
                            }
                        }
                    }
                }
                let _ = tcp_stream.shutdown(std::net::Shutdown::Both);
            }
        });

        (thread, tx_uitoc)
    }

    fn launch_thread_for_receiver(
        &self,
        tx_svtoui: Sender<IncomingMessage>,
        mut tcp_stream: TcpStream,
        ucid: usize,
    ) -> DCCHandler {
        let (tx_uitoc, rx_uitoc) = channel::<DccCommands>();
        let filename = self.file_name.to_owned();
        let port = self.port.as_ref().unwrap().to_owned();
        let expected_file_size = self.file_size.parse::<u64>().unwrap();
        let thread = thread::spawn(move || {
            let (mut file, mut file_size) = match Self::open_file(&filename) {
                Ok((f, fs)) => (f, fs),
                Err(_) => {
                    return;
                }
            };

            let _ = tx_svtoui.send(IncomingMessage::Resume(
                ucid,
                filename,
                file_size.try_into().unwrap(),
                port,
            ));

            let mut buf = vec![0; 1024]; // TODO: vec![0;65000]

            'main_loop: loop {
                match tcp_stream.read(&mut buf) {
                    Ok(nb) => {
                        if nb == 0 {
                            break 'main_loop;
                        }
                        let _ = file.write(&buf[..nb]);
                        file_size = file_size + TryInto::<u64>::try_into(nb).unwrap();
                        let _ = tx_svtoui.send(IncomingMessage::ClientFile(
                            ucid,
                            expected_file_size,
                            file_size,
                        ));
                    }
                    Err(e) => {
                        if e.kind() != ErrorKind::WouldBlock && e.kind() != ErrorKind::TimedOut {
                            println!("Error leyendo desde el tcp_stream {}", e);
                            break 'main_loop;
                        }
                    }
                }

                match rx_uitoc.recv_timeout(Duration::from_nanos(10)) {
                    Ok(cmd) => match cmd {
                        DccCommands::Close => break 'main_loop,
                        _ => {}
                    },
                    Err(_) => {}
                }
            }

            let _ = tcp_stream.shutdown(std::net::Shutdown::Both);
        });

        (thread, tx_uitoc)
    }

    fn open_file(filename: &str) -> io::Result<(File, u64)> {
        fs::create_dir_all(PATH_FILES_DOWNLOAD)?;

        let f = OpenOptions::new()
            .append(true)
            .create(true)
            .open(format!("{}/{}", PATH_FILES_DOWNLOAD, filename))?;
        let len = f.metadata()?.len();
        Ok((f, len))
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::irc::ctcp::constants::{ERR_INVALIDIP, ERR_INVALIDPORT};
    use crate::irc::message::MessageError::{DCCDefined, InvalidFormat, TooManyParams};

    #[test]
    fn test_send_no_params_error() {
        let input = String::new();

        let err = DccSend::parse(input.as_str()).unwrap_err();

        assert_eq!(err, DCCDefined(ERR_NEEDMOREPARAMS));
    }

    #[test]
    fn test_send_less_needed_params_error() {
        let input = String::from("file_name");

        let err = DccSend::parse(input.as_str()).unwrap_err();

        assert_eq!(err, DCCDefined(ERR_NEEDMOREPARAMS));
    }

    #[test]
    fn test_send_more_needed_params_error() {
        let input = String::from("first-param second-param third-param fourth-param fifth-param");

        let err = DccSend::parse(input.as_str()).unwrap_err();

        assert_eq!(err, TooManyParams);
    }

    #[test]
    fn test_send_invalid_file_name_error() {
        let input = String::from("file/name 1.2.3.5 9290 64");

        let err = DccSend::parse(input.as_str()).unwrap_err();

        assert_eq!(err, InvalidFormat);
    }

    #[test]
    fn test_send_invalid_ip_ending_error() {
        let input = String::from("fileName 1.2.3.5. 9290 64");

        let err = DccSend::parse(input.as_str()).unwrap_err();

        assert_eq!(err, DCCDefined(ERR_INVALIDIP));
    }

    #[test]
    fn test_send_invalid_ip_start_error() {
        let input = String::from("fileName .1.2.3.5 9290 64");

        let err = DccSend::parse(input.as_str()).unwrap_err();

        assert_eq!(err, DCCDefined(ERR_INVALIDIP));
    }

    #[test]
    fn test_send_invalid_ip_len_error() {
        let input = String::from("fileName 1.2.3.4.5.6.7.8.9.10.11 9290 64");

        let err = DccSend::parse(input.as_str()).unwrap_err();

        assert_eq!(err, DCCDefined(ERR_INVALIDIP));
    }

    #[test]
    fn test_send_invalid_port_min_len_nedeed_error() {
        let input = String::from("fileName 1.2.3.5 1 64");

        let err = DccSend::parse(input.as_str()).unwrap_err();

        assert_eq!(err, DCCDefined(ERR_INVALIDPORT));
    }

    #[test]
    fn test_send_invalid_port_max_len_nedeed_error() {
        let input = String::from("fileName 1.2.3.5 929090 64");

        let err = DccSend::parse(input.as_str()).unwrap_err();

        assert_eq!(err, DCCDefined(ERR_INVALIDPORT));
    }

    #[test]
    fn test_send_all_params_ok() {
        let input = String::from("fileName 1.2.3.4 9290 32");

        let send = DccSend::parse(input.as_str()).unwrap();

        assert_eq!(send.file_name, "fileName");
        assert_eq!(send.ip.unwrap(), "1.2.3.4");
        assert_eq!(send.port.unwrap(), "9290");
        assert_eq!(send.file_size, "32");
    }
}
