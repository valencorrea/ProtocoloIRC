//! Modulo que se centra en las funcionalidades referentes a la representacion de conecciones de servers.
use std::{
    io::Write,
    net::{Shutdown, TcpStream},
};

use crate::unwrap;

use super::{connection::Connection, ConnectionError};

#[derive(Debug)]
pub struct ServerConnection {
    pub servername: String,
    pub password: Option<String>,
    pub hopcount: u32,
    pub write_stream: Option<TcpStream>,
    pub uplink: Option<String>,
}

impl ServerConnection {
    pub fn from_connection(connection: Connection) -> Result<ServerConnection, ()> {
        let servername = unwrap!(connection.servername);
        let password = unwrap!(connection.password); // This has to exist
        let hopcount = unwrap!(connection.hopcount);
        Ok(ServerConnection {
            servername,
            password: Some(password),
            hopcount,
            write_stream: Some(connection.write_stream),
            uplink: connection.uplink,
        })
    }

    pub fn create(
        servername: String,
        password: String,
        stream: TcpStream,
    ) -> Result<(ServerConnection, TcpStream), ConnectionError> {
        let ss = match stream.try_clone() {
            Ok(a) => a,
            Err(_) => return Err(ConnectionError::InternalServerError),
        };

        Ok((
            ServerConnection {
                servername,
                password: Some(password),
                hopcount: 1,
                write_stream: Some(stream),
                uplink: None,
            },
            ss,
        ))
    }

    pub fn for_data(servername: String, hopcount: u32, uplink: Option<String>) -> Self {
        Self {
            servername,
            password: None,
            hopcount,
            write_stream: None,
            uplink,
        }
    }
}

impl ServerConnection {
    pub fn write_line(&mut self, msg: &str) {
        if let Some(s) = &mut self.write_stream {
            if let Err(e) = s.write(format!("{}\r\n", msg).as_bytes()) {
                println!("[TO SERVER] Write failed.\n{}", e);
            };
        }
    }

    pub fn shutdown(&self) {
        if let Some(stream) = &self.write_stream {
            let _ = stream.shutdown(Shutdown::Both);
        }
    }
}
