//! Modulo que se centra en las funcionalidades referentes a la representacion de canales.
use std::collections::HashMap;

use crate::irc::constants::{ERR_CHANLIMIT, ERR_SERVERERR};
use crate::irc::{
    constants::{ERR_BADCHANNELKEY, ERR_NOTONCHANNEL},
    message::utils::validate_channel,
};

use crate::try_lock;

use super::{
    utils::{
        deserialize_bool, deserialize_err, deserialize_num, deseriaze_usernames, serialize_bool,
        serialize_list, serialize_option,
    },
    MTClient, ServerError,
};

#[derive(Debug)]
pub struct Channel {
    pub name: String,
    pub topic: Option<String>,
    pub password: Option<String>,
    pub clients: HashMap<String, MTClient>,
    pub private: bool,
    pub secret: bool,
    pub invite_only: bool,
    pub topic_ops_only: bool,
    pub no_msg_outside: bool,
    pub moderated: bool,
    pub limit: Option<u32>,
    pub allow_moderated: HashMap<String, MTClient>,
    pub registered_operators: HashMap<String, bool>,
}

enum Serialize {
    Name = 0,
    Key,
    Topic,
    Private,
    Secret,
    InviteOnly,
    TopicOnlyOperators,
    NoMessageFromOutside,
    Moderated,
    Limit,
    RegisteredOperators,
}

// Creation
impl Channel {
    pub fn create_from(name: String, key: Option<String>) -> Channel {
        Channel {
            name,
            topic: None,
            password: key,
            clients: HashMap::new(),
            private: false,
            secret: false,
            invite_only: false,
            topic_ops_only: true,
            no_msg_outside: true,
            moderated: false,
            limit: None,
            allow_moderated: HashMap::new(),
            registered_operators: HashMap::new(),
        }
    }

    pub fn deserialize(data: &[&str]) -> Result<Channel, String> {
        let mut c = Channel::create_from("d".to_owned(), None);
        if data.len() != 11 {
            return Err(deserialize_err("Invalid format"));
        }

        if validate_channel(Some(data[Serialize::Name as usize].as_bytes())).is_err() {
            return Err(deserialize_err("Invalid channel name"));
        }

        c.name = data[Serialize::Name as usize].to_owned();

        if !data[Serialize::Key as usize].is_empty() {
            c.password = Some(data[Serialize::Key as usize].to_owned())
        }

        if !(data[Serialize::Topic as usize].is_empty()) {
            c.topic = Some(data[Serialize::Topic as usize].to_owned())
        }

        c.private = deserialize_bool(data[Serialize::Private as usize])?;
        c.secret = deserialize_bool(data[Serialize::Secret as usize])?;
        c.invite_only = deserialize_bool(data[Serialize::InviteOnly as usize])?;
        c.topic_ops_only = deserialize_bool(data[Serialize::TopicOnlyOperators as usize])?;
        c.no_msg_outside = deserialize_bool(data[Serialize::NoMessageFromOutside as usize])?;
        c.moderated = deserialize_bool(data[Serialize::Moderated as usize])?;
        if !data[Serialize::Limit as usize].is_empty() {
            c.limit = Some(deserialize_num(data[Serialize::Limit as usize])?);
        }

        c.registered_operators =
            deseriaze_usernames(data[Serialize::RegisteredOperators as usize])?;

        Ok(c)
    }

    pub fn serialize(&self) -> Vec<String> {
        let mut r: Vec<String> = vec![String::new(); 11];
        r[Serialize::Name as usize] = self.name.to_owned();
        r[Serialize::Key as usize] = serialize_option(&self.password);
        r[Serialize::Topic as usize] = serialize_option(&self.topic);
        r[Serialize::Private as usize] = serialize_bool(self.private);
        r[Serialize::Secret as usize] = serialize_bool(self.secret);
        r[Serialize::InviteOnly as usize] = serialize_bool(self.invite_only);
        r[Serialize::TopicOnlyOperators as usize] = serialize_bool(self.topic_ops_only);
        r[Serialize::NoMessageFromOutside as usize] = serialize_bool(self.no_msg_outside);
        r[Serialize::Moderated as usize] = serialize_bool(self.moderated);
        r[Serialize::Limit as usize] = serialize_option(&self.limit);
        r[Serialize::RegisteredOperators as usize] =
            serialize_list(&self.registered_operators.keys().collect::<Vec<&String>>());
        r
    }
}

// Client manipulation
impl Channel {
    pub fn join_client(
        &mut self,
        client: MTClient,
        password: Option<String>,
    ) -> Result<(), ServerError> {
        if password.as_deref() == self.password.as_deref() {
            return self.add_client(client);
        }
        Err(ServerError {
            code: ERR_BADCHANNELKEY,
            msg: format!("{} :Cannot join channel (+k)", self.name),
        })
    }

    pub fn add_client(&mut self, client: MTClient) -> Result<(), ServerError> {
        if let Some(limit) = self.limit {
            match usize::try_from(limit) {
                Ok(v) => {
                    if v == self.client_amount() {
                        return Err(ServerError {
                            code: ERR_CHANLIMIT,
                            msg: format!("{} :Channel reach member limit", self.name),
                        });
                    }
                }
                Err(_) => {
                    return Err(ServerError {
                        code: ERR_SERVERERR,
                        msg: "Internal numeric conversion problem".to_owned(),
                    })
                }
            }
        }
        self.insert_client(client);
        Ok(())
    }

    pub fn insert_client(&mut self, client: MTClient) {
        let nickname = { try_lock!(client).nickname.to_owned() };
        self.clients.insert(nickname, client);
    }

    pub fn remove_client_by_nickname(&mut self, nickname: &str) -> Result<(), ServerError> {
        match self.clients.remove(nickname) {
            Some(_) => Ok(()),
            None => Err(ServerError {
                code: ERR_NOTONCHANNEL,
                msg: format!("{} :You're not on that channel", self.name),
            }),
        }
    }

    pub fn client_amount(&self) -> usize {
        self.clients.keys().len()
    }

    pub fn change_nickname(&mut self, old_nick: &str, new_nick: &str) {
        if let Some(client) = self.clients.remove(old_nick) {
            self.clients.insert(new_nick.to_owned(), client);
        }

        if self.registered_operators.remove(old_nick).is_some() {
            self.registered_operators.insert(new_nick.to_owned(), false);
        }
    }

    pub fn allow_client_for_moderated(&mut self, client: MTClient) -> Result<(), ServerError> {
        let nickname = { try_lock!(client).nickname.to_owned() };
        {
            if !try_lock!(client).is_in_channel(&self.name) {
                return Err(ServerError {
                    code: ERR_NOTONCHANNEL,
                    msg: format!("{} :{} not on channel", &self.name, nickname),
                });
            }
        }

        self.allow_moderated.insert(nickname, client);
        Ok(())
    }

    pub fn disallow_client_for_moderated(&mut self, client: MTClient) -> Result<(), ServerError> {
        let c = try_lock!(client);
        if !c.is_in_channel(&self.name) {
            return Err(ServerError {
                code: ERR_NOTONCHANNEL,
                msg: format!("{} :{} not on channel", &self.name, &c.nickname),
            });
        }
        self.allow_moderated.remove(&c.nickname);
        Ok(())
    }

    pub fn is_allowed_for_moderated(&self, client: &MTClient) -> bool {
        self.allow_moderated
            .contains_key(&try_lock!(client).nickname)
    }
}

// String representation
impl Channel {
    pub fn get_clients_names(&self, is_oper: bool) -> Vec<String> {
        let mut clients_nicknames = Vec::new();
        for (nicks, client) in self.clients.iter() {
            if try_lock!(client).invisible && !is_oper {
                continue;
            }
            clients_nicknames.push(nicks.to_owned());
        }
        clients_nicknames.sort();
        clients_nicknames
    }

    pub fn get_name(&self) -> String {
        self.name.to_owned()
    }

    pub fn channel_to_string(&self) -> String {
        let name = self.get_name();
        match self.topic.to_owned() {
            None => format!("{}: No topic set ", name),
            Some(topic) => format!("{}: {}", name, topic),
        }
    }

    pub fn channel_to_string_outsider(&self) -> Option<String> {
        if self.secret {
            return None;
        }
        if self.private {
            return Some(format!("{} :Private", &self.name));
        }

        Some(self.channel_to_string())
    }

    pub fn describe_clients(&self, asker: MTClient) -> Vec<String> {
        let asker_operator = { try_lock!(asker).is_channel_operator(&self.name) };

        let mut res = Vec::new();
        for (_, client) in self.clients.iter() {
            let lclient = try_lock!(client);
            if lclient.invisible && !asker_operator {
                continue;
            }
            res.push(format!(
                "{} :{} {} {} :{}",
                self.name, lclient.username, lclient.hostname, lclient.nickname, lclient.realname
            ));
        }

        res
    }

    pub fn get_topic(&self) -> Option<String> {
        self.topic.as_ref().map(|t| t.to_owned())
    }

    pub fn set_topic(&mut self, topic: &str) {
        self.topic = Some(topic.to_owned());
    }

    pub fn private_message(&self) -> String {
        if self.private {
            format!("MODE {} +p", self.name)
        } else {
            format!("MODE {} -p", self.name)
        }
    }

    pub fn secret_message(&self) -> String {
        if self.secret {
            format!("MODE {} +s", self.name)
        } else {
            format!("MODE {} -s", self.name)
        }
    }

    pub fn invite_only_message(&self) -> String {
        if self.invite_only {
            format!("MODE {} +i", self.name)
        } else {
            format!("MODE {} -i", self.name)
        }
    }

    pub fn topic_ops_only_message(&self) -> String {
        if self.topic_ops_only {
            format!("MODE {} +t", self.name)
        } else {
            format!("MODE {} -t", self.name)
        }
    }

    pub fn no_msg_outside_message(&self) -> String {
        if self.no_msg_outside {
            format!("MODE {} +n", self.name)
        } else {
            format!("MODE {} -n", self.name)
        }
    }

    pub fn moderated_message(&self) -> String {
        if self.moderated {
            format!("MODE {} +m", self.name)
        } else {
            format!("MODE {} -m", self.name)
        }
    }

    pub fn limit_message(&self) -> String {
        if let Some(l) = self.limit.to_owned() {
            format!("MODE {} +l {}", self.name, l)
        } else {
            format!("MODE {} -l", self.name)
        }
    }

    pub fn key_message(&self) -> String {
        if let Some(k) = self.password.to_owned() {
            format!("MODE {} +k {}", self.name, k)
        } else {
            format!("MODE {} -k", self.name)
        }
    }
    pub fn mode_message(&self) -> String {
        format!("MODE {}", self.name.to_owned())
    }
    pub fn topic_message(&self) -> Option<String> {
        self.topic
            .as_ref()
            .map(|v| format!("TOPIC {} :{}", self.name.to_owned(), v))
    }
}
