//! Modulo que se centra en las funcionalidades referentes a la representacion de clientes.

use std::{collections::HashMap, net::TcpStream};

use super::MTChannel;

pub mod channels;
pub mod create;
pub mod gtk_runtime;
pub mod nick;
pub mod no_gui_runtime;
pub mod repr;
pub mod runtime;
pub mod sv_comm;

#[derive(Debug)]
pub struct Client {
    pub stream: Option<TcpStream>,
    pub nickname: String,
    pub hostname: String,
    pub username: String,
    pub servername: String,
    pub pass: Option<String>,
    pub realname: String,
    pub away_message: Option<String>,
    pub channels: HashMap<String, MTChannel>,
    pub server_operator: bool,
    pub invisible: bool,
    pub rec_sv_notices: bool,
    pub channel_operator: HashMap<String, MTChannel>,
    pub channel_invites: Vec<String>,
}
