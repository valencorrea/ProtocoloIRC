//! Modulo que se centra en las funcionalidades referentes a la representacion de cuentas de clientes.
use crate::irc::message::utils::{validate_name_invalid_none, validate_password};

use super::{
    client::Client,
    utils::{deserialize_err, serialize_option},
};

#[derive(Debug)]
pub struct ClientAccount {
    pub nickname: String,
    pub username: String,
    pub pwd: Option<String>,
}

enum Serialize {
    Nickname = 0,
    Username,
    Password,
}

impl ClientAccount {
    pub fn deserialize(data: &[&str]) -> Result<ClientAccount, String> {
        if data.len() != 3 {
            return Err(deserialize_err("Invalid file format"));
        }
        if validate_name_invalid_none(Some(data[Serialize::Nickname as usize].as_bytes())).is_err()
        {
            return Err(deserialize_err("Invalid nickname"));
        }
        if validate_name_invalid_none(Some(data[Serialize::Username as usize].as_bytes())).is_err()
        {
            return Err(deserialize_err("Invalid username"));
        }
        if validate_password(Some(data[Serialize::Password as usize].as_bytes())).is_err() {
            return Err(deserialize_err("Invalid password"));
        }
        let serialized_pwd = data[Serialize::Password as usize];
        let mut pwd = None;
        if !serialized_pwd.is_empty() {
            pwd = Some(serialized_pwd.to_owned());
        }

        Ok(ClientAccount {
            nickname: data[Serialize::Nickname as usize].to_owned(),
            username: data[Serialize::Username as usize].to_owned(),
            pwd,
        })
    }

    pub fn serialize(&self) -> Vec<String> {
        let mut r = vec![String::new(); 3];

        r[Serialize::Nickname as usize] = (self.nickname).to_owned();
        r[Serialize::Username as usize] = (self.username).to_owned();
        r[Serialize::Password as usize] = serialize_option(&self.pwd);

        r
    }

    pub fn for_client(client: &Client) -> ClientAccount {
        ClientAccount {
            nickname: client.nickname.to_owned(),
            username: client.username.to_owned(),
            pwd: client.pass.to_owned(),
        }
    }
}
