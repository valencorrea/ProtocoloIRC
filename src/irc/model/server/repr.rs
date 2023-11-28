//! Modulo que se centra en las funcionalidades referentes a la representacion del server.
use std::sync::MutexGuard;

use crate::{
    irc::{
        constants::ERR_NOSUCHNICK,
        message::utils::no_such_nick,
        model::{channel::Channel, MTChannel, MTClient, ServerError},
    },
    try_lock,
};

use super::{Server, UserInfo};

impl Server {
    pub fn describe_connected_clients(&self, asker: MTClient) -> Vec<String> {
        let channels = try_lock!(self.channels);
        let mut results = Vec::new();
        for (_, channel) in channels.iter() {
            let lchannel = try_lock!(channel);
            results.extend(lchannel.describe_clients(asker.clone()))
        }
        results
    }

    fn get_channel_string(
        &self,
        client: MTClient,
        func: &dyn Fn(MutexGuard<Channel>) -> bool,
    ) -> Vec<Option<String>> {
        let channels = try_lock!(self.channels);
        let mut vec: Vec<Option<String>> = channels
            .values()
            .filter(|ch| {
                let lch = try_lock!(ch);
                func(lch)
            })
            .map(|ch| {
                let lch = try_lock!(ch);
                if try_lock!(client).is_in_channel(&lch.name) {
                    return Some(lch.channel_to_string());
                }
                lch.channel_to_string_outsider()
            })
            .collect();
        vec.sort();
        vec
    }

    pub fn get_channels_strings_by_names(
        &self,
        client: MTClient,
        client_names: Vec<String>,
    ) -> Vec<Option<String>> {
        self.get_channel_string(client, &|channel: MutexGuard<Channel>| {
            client_names.contains(&channel.name)
        })
    }

    pub fn get_all_channels_string(&self, client: MTClient) -> Vec<Option<String>> {
        self.get_channel_string(client, &|_: MutexGuard<Channel>| true)
    }

    pub fn describe_full_user_info(
        &self,
        nickname: &str,
        asker: MTClient,
    ) -> Result<UserInfo, ServerError> {
        let asker_sv_operator = { try_lock!(asker).server_operator };

        let mut user_info = UserInfo {
            user: String::new(),
            oper: None,
            end: String::new(),
            channels: Vec::new(),
        };

        let err = ServerError {
            code: ERR_NOSUCHNICK,
            msg: no_such_nick(nickname.as_bytes()),
        };

        match self.get_client_by_nickname(nickname) {
            Some(client) => {
                let client = try_lock!(client);
                if client.invisible && !asker_sv_operator {
                    return Err(err);
                }
                user_info.user = client.describe();
                user_info.channels = client.describe_channels();
                if client.server_operator {
                    user_info.oper = Some(format!("{} :is an IRC operator", nickname));
                }

                user_info.end = format!("{} :End of /WHOIS list", nickname)
            }
            None => return Err(err),
        }

        Ok(user_info)
    }

    fn describe_clients_no_channels(&self, for_oper: bool) -> Vec<String> {
        let mut no_channels = Vec::new();
        let lclients = try_lock!(self.clients);
        for (nick, cl) in lclients.iter() {
            let client = try_lock!(cl);
            let is_visible = !client.invisible || for_oper;
            if client.channel_amount() == 0 && is_visible {
                no_channels.push(nick.to_owned())
            }
        }
        no_channels
            .iter()
            .map(|user| format!("* {}", user))
            .collect()
    }

    fn describe_clients_in_channel(
        &self,
        channel_name: &str,
        channel: MTChannel,
        client: MTClient,
    ) -> Option<Vec<String>> {
        let is_oper = {
            let c = try_lock!(client);
            let belongs = c.is_in_channel(channel_name);
            let is_oper = c.is_channel_operator(channel_name) || c.server_operator;
            let visible = {
                let ch = try_lock!(channel);
                !ch.private && !ch.secret
            };

            if !belongs && !is_oper && !visible {
                return None;
            }

            is_oper
        };

        let clients: Vec<String> = {
            try_lock!(channel)
                .get_clients_names(is_oper)
                .iter()
                .map(|client| format!("{} {}", channel_name, client))
                .collect()
        };

        Some(clients)
    }

    pub fn describe_clients_for_channel(
        &self,
        client: MTClient,
        channel_name: &str,
    ) -> Option<Vec<String>> {
        match self.get_channel_by_name(channel_name) {
            Some(channel) => self.describe_clients_in_channel(channel_name, channel, client),
            None => None,
        }
    }

    pub fn describe_all_client_for_all_channels(&self, client: MTClient) -> Vec<String> {
        let mut response: Vec<String> = {
            try_lock!(self.channels)
                .iter()
                .flat_map(|(channel_name, channel)| {
                    self.describe_clients_in_channel(channel_name, channel.clone(), client.clone())
                        .unwrap_or(vec![])
                })
                .collect()
        };

        let is_sv_oper = { try_lock!(client).server_operator };
        response.append(&mut self.describe_clients_no_channels(is_sv_oper));

        response
    }

    fn describe_modes(&self, name: &str, conds: &[(bool, &str)]) -> String {
        let mut plus = Vec::new();
        let mut minus = Vec::new();

        for (cond, l) in conds {
            let v = if *cond { &mut plus } else { &mut minus };
            v.push(*l)
        }

        format!("{} : +{} -{}", name, plus.join(""), minus.join(""))
    }

    pub fn describe_client_modes(&self, client: MTClient) -> String {
        let c = try_lock!(client);

        let conds = [
            (c.invisible, "i"),
            (c.server_operator, "o"),
            (c.rec_sv_notices, "s"),
        ];

        self.describe_modes(&c.nickname, &conds)
    }

    pub fn describe_channel_modes(&self, channel: MTChannel) -> String {
        let c = try_lock!(channel);

        let conds = [
            (c.private, "p"),
            (c.secret, "s"),
            (c.invite_only, "i"),
            (c.topic_ops_only, "t"),
            (c.no_msg_outside, "n"),
            (c.moderated, "m"),
            (c.limit.is_some(), "l"),
            (c.password.is_some(), "k"),
        ];

        let mut desc = self.describe_modes(&c.name, &conds);
        if let Some(limit) = c.limit {
            desc = format!("{}: Limit: {}", desc, limit);
        }
        desc
    }
}
