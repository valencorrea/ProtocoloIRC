use std::{collections::HashMap, sync::Mutex};

use super::{MTChannel, MTClient, MTClientAccount, MTServerConnection};

pub mod channel_modif;
pub mod client_modif;
pub mod persist;
pub mod repr;
pub mod runtime;
pub mod send_to_client;
pub mod server_connection;

#[derive(Debug)]
pub struct Server {
    pub host: String,
    pub clients: Mutex<HashMap<String, MTClient>>,
    pub channels: Mutex<HashMap<String, MTChannel>>,
    pub accounts: Mutex<HashMap<String, MTClientAccount>>,
    pub sv_connections: Mutex<HashMap<String, MTServerConnection>>,
}

pub struct UserInfo {
    pub user: String,
    pub oper: Option<String>,
    pub end: String,
    pub channels: Vec<String>,
}
