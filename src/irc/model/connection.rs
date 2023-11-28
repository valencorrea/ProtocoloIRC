//! Modulo que se centra en las funcionalidades referentes a la representacion de conecciones.
use std::{
    io::{self, BufRead, BufReader, Write},
    net::{Shutdown, TcpStream},
    sync::{mpsc::Receiver, Arc},
    time::Duration,
};

use crate::{
    irc::{
        constants::ERR_REGMISSING,
        message::generic_message::GenericMessage,
        responses::{builder::ResponseBuilder, InternalType, ResponseType},
    },
    try_lock,
};

use super::{
    client::Client, server::Server, server_connection::ServerConnection, utils::mt, MTClient,
    MTServerConnection,
};

#[derive(Debug, PartialOrd, PartialEq, Eq)]
enum ConnectionStep {
    PasswordNotSetted,
    PasswordSet,
    NickSet,
}

#[derive(Debug, PartialOrd, PartialEq, Eq)]
enum ConnectionType {
    Unknown,
    Client,
    Server,
}

pub struct Connection {
    pub password: Option<String>,
    pub conn_nick: Option<String>,
    pub write_stream: TcpStream,
    conn_step: ConnectionStep,
    conn_type: ConnectionType,
    pub username: Option<String>,
    pub hostname: Option<String>,
    pub servername: Option<String>,
    pub realname: Option<String>,
    pub hopcount: Option<u32>,
    pub uplink: Option<String>,
}

impl Connection {
    pub fn handle_connection(server: Arc<Server>, stream: TcpStream) -> Result<(), ()> {
        if stream.set_nonblocking(false).is_err() {
            let _ = stream.shutdown(Shutdown::Both);
            println!("[SERVER - CONNECTION] Can't properly setup the connection");
            return Err(());
        };
        let read_stream = stream;
        let write_stream = match read_stream.try_clone() {
            Ok(v) => v,
            Err(_) => {
                println!("[SERVER - CONNECTION] Can't properly setup the connection");
                let _ = read_stream.shutdown(Shutdown::Both);
                return Err(());
            }
        };

        let mut conn = Connection {
            password: None,
            conn_nick: None,
            write_stream,
            conn_step: ConnectionStep::PasswordNotSetted,
            conn_type: ConnectionType::Unknown,
            username: None,
            hostname: None,
            servername: None,
            realname: None,
            hopcount: None,
            uplink: None,
        };

        conn.handle_initial_connection(server.clone(), &read_stream);
        match conn.conn_type {
            ConnectionType::Unknown => {
                if let Err(e) = read_stream.shutdown(Shutdown::Both) {
                    println!(
                        "[SERVER - CONNECTION] Error while trying to shutdown stream.\n {}",
                        e
                    )
                };
                println!("[SERVER - CONNECTION] Connection closed before registration finished");
            }
            ConnectionType::Client => match Client::from_connection(conn) {
                Ok(client) => {
                    Connection::handle_client(server, mt(client), &read_stream)?;
                }
                Err(_) => {
                    println!("[SERVER - CONNECTION] Error while creating the client connection");
                }
            },
            ConnectionType::Server => match ServerConnection::from_connection(conn) {
                Ok(sv_connection) => {
                    Connection::handle_server_connection(
                        server,
                        mt(sv_connection),
                        &read_stream,
                        None,
                    )?;
                }
                Err(_) => {
                    println!("[SERVER - CONNECTION] Error while creating the client connection")
                }
            },
        }

        if let Err(e) = read_stream.shutdown(Shutdown::Both) {
            println!(
                "[SERVER - CONNECTION] Error while trying to shutdown stream.\n {}",
                e
            )
        };

        Ok(())
    }

    pub fn set_password(&mut self, password: String) {
        self.password = Some(password);
        self.conn_step = ConnectionStep::PasswordSet
    }

    pub fn get_password(&self) -> Option<&String> {
        self.password.as_ref()
    }

    pub fn quit(&mut self) {
        self.conn_type = ConnectionType::Unknown;
    }

    pub fn set_nickname(&mut self, nickname: String) -> Result<(), (usize, String)> {
        if self.conn_step < ConnectionStep::PasswordSet {
            return Err((
                ERR_REGMISSING,
                "You need to send PASS before trying to send NICK and USER combiantion".to_owned(),
            ));
        }
        self.conn_step = ConnectionStep::NickSet;
        self.conn_nick = Some(nickname);
        self.conn_type = ConnectionType::Client;
        Ok(())
    }

    pub fn get_nickname(&self) -> Option<&String> {
        self.conn_nick.as_ref()
    }

    pub fn set_client_connection(
        &mut self,
        username: String,
        hostname: String,
        servername: String,
        realname: String,
    ) {
        self.username = Some(username);
        self.hostname = Some(hostname);
        self.servername = Some(servername);
        self.realname = Some(realname);
    }

    pub fn set_server_connection(
        &mut self,
        servername: String,
        hopcount: u32,
        uplink: Option<String>,
    ) {
        self.conn_type = ConnectionType::Server;
        self.servername = Some(servername);
        self.hopcount = Some(hopcount);
        self.uplink = uplink;
    }

    pub fn can_be_server(&self) -> bool {
        self.conn_type != ConnectionType::Client
    }

    fn handle_initial_connection(&mut self, server: Arc<Server>, read_stream: &TcpStream) {
        let addr = match read_stream.peer_addr() {
            Ok(sa) => sa.to_string(),
            Err(_) => "Unknown".to_owned(),
        };

        let reader = BufReader::new(read_stream);
        let mut lines = reader.lines();

        let mut reg_done = false;
        loop {
            if let Some(line) = lines.next() {
                let l = match line {
                    Ok(p) => {
                        if p.is_empty() {
                            break;
                        }
                        p
                    }
                    Err(_) => {
                        break;
                    }
                };
                println!("[UNREGISTERED - {}]: {} ", addr, l);

                let responses = match GenericMessage::parse(&l) {
                    Ok(v) => v.execute_registration(&server, self),
                    Err(e) => ResponseBuilder::new().add_from_error(e).build(),
                };

                for response in responses {
                    if let ResponseType::InternalResponse(t) = &response {
                        match *t {
                            InternalType::Quit => {
                                return;
                            }
                            InternalType::Upgrade => reg_done = true,
                        }
                    }
                    if let Some(res) = response.serialize() {
                        if let Err(e) = self.write_stream.write(format!("{}\r\n", &res).as_bytes())
                        {
                            eprintln!("{}", e);
                        }
                    }
                }
            }

            if reg_done {
                break;
            }
        }
    }
}

impl Connection {
    pub fn handle_client(
        server: Arc<Server>,
        client: MTClient,
        stream: &TcpStream,
    ) -> Result<(), ()> {
        let addr = match stream.peer_addr() {
            Ok(sa) => sa.to_string(),
            Err(_) => "Unknown".to_owned(),
        };

        server.introduce_new_client(client.clone());

        let reader = BufReader::new(stream);

        let mut lines = reader.lines();
        let mut keep_listening = true;

        loop {
            if let Some(line) = lines.next() {
                let l = match line {
                    Ok(p) => {
                        if p.is_empty() {
                            break;
                        }
                        p
                    }
                    Err(_) => {
                        break;
                    }
                };
                println!("[CLIENT {}]: {} ", addr, l);

                let responses = match GenericMessage::parse(&l) {
                    Ok(v) => v.execute(server.as_ref(), client.clone()),
                    Err(e) => ResponseBuilder::new().add_from_error(e).build(),
                };

                for response in responses {
                    let mut locked_client = try_lock!(client);
                    if let ResponseType::InternalResponse(InternalType::Quit) = &response {
                        keep_listening = false;
                    }
                    if let Some(res) = response.serialize() {
                        if let Err(e) = locked_client.write_to_sv(&res) {
                            eprintln!("{}", e);
                        }
                    }
                }
            }

            if !keep_listening {
                break;
            }
        }

        Ok(())
    }

    pub fn handle_server_connection(
        server: Arc<Server>,
        server_connection: MTServerConnection,
        read_stream: &TcpStream,
        killer: Option<Receiver<ResponseType>>,
    ) -> Result<(), ()> {
        let addr = match read_stream.peer_addr() {
            Ok(sa) => sa.to_string(),
            Err(_) => "Unknown".to_owned(),
        };

        if let Err(e) = server.register_server_connection(server_connection.clone()) {
            server.write_to_server(server_connection, &format!("{} :{}", e.code, e.msg));
            return Err(());
        };

        server.introduce_server(server_connection.clone());

        let reader = BufReader::new(read_stream);

        let mut lines = reader.lines();
        let mut keep_listening = true;
        loop {
            if let Some(line) = lines.next() {
                let l = match line {
                    Ok(p) => {
                        if p.is_empty() {
                            break;
                        }
                        p
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        if let Some(tx) = &killer {
                            if let Ok(ResponseType::InternalResponse(InternalType::Quit)) =
                                tx.recv_timeout(Duration::from_millis(50))
                            {
                                //FOR SOME REASON TRY RECV DOESN'T WORK
                                break;
                            }
                        }
                        continue;
                    }
                    Err(_) => {
                        break;
                    }
                };
                println!("[FROM SERVER - {}]: {} ", addr, l);

                let responses = match GenericMessage::parse(&l) {
                    Ok(v) => v.execute_for_server(&server, server_connection.clone()),
                    Err(e) => ResponseBuilder::new().add_from_error(e).build(),
                };

                for response in responses {
                    if let ResponseType::InternalResponse(InternalType::Quit) = &response {
                        keep_listening = false;
                    }
                }
            }

            if !keep_listening {
                break;
            }
        }

        Ok(())
    }
}
