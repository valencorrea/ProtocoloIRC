use std::collections::HashMap;
use std::net::TcpStream;

use super::Client;
use crate::irc::model::connection::Connection;
use crate::irc::model::{ConnectionError, CLIENT_ARGS, CLIENT_IP_POS, CLIENT_PORT_POS};
use crate::unwrap;
use crate::ConnectionError::InvalidArguments;

impl Client {
    pub fn create_fromargs(argv: Vec<String>) -> Result<Client, ConnectionError> {
        if argv.len() != CLIENT_ARGS {
            return Err(InvalidArguments);
        }

        let stream = match TcpStream::connect(format!(
            "{}:{}",
            argv[CLIENT_IP_POS], argv[CLIENT_PORT_POS]
        )) {
            Ok(v) => v,
            Err(e) => {
                println!("{}", e);
                return Err(InvalidArguments);
            }
        };

        Ok(Client::create_fromtcp(stream))
    }

    pub fn create_fromtcp(tcp: TcpStream) -> Client {
        let a = match tcp.peer_addr() {
            Ok(v) => v.to_string(),
            Err(_) => Client::random_nick(),
        };
        Client {
            stream: Some(tcp),
            nickname: (a),
            hostname: String::new(),
            username: String::new(),
            servername: String::new(),
            pass: None,
            realname: String::new(),
            away_message: None,
            channels: HashMap::new(),
            server_operator: false,
            invisible: false,
            rec_sv_notices: true,
            channel_operator: HashMap::new(),
            channel_invites: Vec::new(),
        }
    }

    pub fn from_connection(conn: Connection) -> Result<Client, ()> {
        let nickname = unwrap!(conn.conn_nick);
        let hostname = unwrap!(conn.hostname);
        let username = unwrap!(conn.username);
        let servername = unwrap!(conn.servername);
        let password = unwrap!(conn.password);
        let realname = unwrap!(conn.realname);
        let stream = conn.write_stream;

        Ok(Client {
            stream: Some(stream),
            nickname,
            hostname,
            username,
            servername,
            pass: Some(password),
            realname,
            away_message: None,
            channels: HashMap::new(),
            server_operator: false,
            invisible: false,
            rec_sv_notices: true,
            channel_operator: HashMap::new(),
            channel_invites: Vec::new(),
        })
    }

    pub fn for_data(nickname: String) -> Client {
        Client {
            stream: None,
            nickname,
            hostname: String::new(),
            username: String::new(),
            servername: String::new(),
            pass: None,
            realname: String::new(),
            away_message: None,
            channels: HashMap::new(),
            server_operator: false,
            invisible: false,
            rec_sv_notices: true,
            channel_operator: HashMap::new(),
            channel_invites: Vec::new(),
        }
    }

    pub fn random_nick() -> String {
        rand::random::<u64>().to_string()
    }
}
