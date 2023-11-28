use std::sync::{Arc, Mutex};

use self::{
    channel::Channel, client::Client, client_account::ClientAccount,
    server_connection::ServerConnection,
};

pub mod channel;
pub mod client;
pub mod client_account;
pub mod connection;
pub mod server;
pub mod server_connection;
pub mod utils;
pub mod workers;

#[derive(Debug, PartialEq, Eq)]
pub enum ConnectionError {
    InvalidArguments,
    InternalServerError,
}

#[derive(Debug)]
pub struct ServerError {
    pub code: usize,
    pub msg: String,
}

pub type MTClient = Arc<Mutex<Client>>;
pub type MTChannel = Arc<Mutex<Channel>>;
pub type MTClientAccount = Arc<Mutex<ClientAccount>>;
pub type MTServerConnection = Arc<Mutex<ServerConnection>>;

pub const WHAT_TO_RUN_POS: usize = 1;

pub const SERVER_ARGS: usize = 3; //Junk + WHAT TO RUN + Port
pub const SERVER_CONNECT_ARGS: usize = SERVER_ARGS + 3;

pub const SERVER_PORT_POS: usize = 2;

pub const SERVER_CONNECT_IP_POS: usize = SERVER_PORT_POS + 1;
pub const SERVER_CONNECT_PORT_POS: usize = SERVER_CONNECT_IP_POS + 1;
pub const SERVER_CONNECT_PASSWORD_POS: usize = SERVER_CONNECT_PORT_POS + 1;

static CLIENT_ARGS: usize = 4;
pub const CLIENT_IP_POS: usize = 2;
pub const CLIENT_PORT_POS: usize = 3;
