//! Modulo que se centra en las funcionalidades referentes a la modificacion de canales por parte del server.
use crate::{
    irc::{
        constants::{ERR_CHANOPRIVSNEEDED, ERR_NOSUCHCHANNEL, ERR_SERVERERR},
        model::{channel::Channel, utils::mt, MTChannel, MTClient, ServerError},
    },
    try_lock,
};

use super::Server;

impl Server {
    pub fn get_channel_by_name(&self, channel_name: &str) -> Option<MTChannel> {
        let locked_channels = try_lock!(self.channels);
        Some(locked_channels.get(channel_name)?.clone())
    }

    fn add_creator(
        &self,
        ocreator: Option<MTClient>,
        channel_name: &str,
        channel: MTChannel,
    ) -> MTChannel {
        if let Some(creatorm) = ocreator {
            {
                self.set_client_channel_operator(creatorm.clone(), channel.clone());
            }
            {
                let mut creator = try_lock!(creatorm);
                creator.remove_invite(channel_name);
                creator.add_channel(channel.clone());
            }
            {
                try_lock!(channel).insert_client(creatorm);
            }
        }
        channel
    }

    pub fn create_channel(
        &self,
        channel_name: String,
        pwd: Option<String>,
        creator: Option<MTClient>,
    ) -> MTChannel {
        let channel = {
            let new_channel = mt(Channel::create_from(channel_name.to_owned(), pwd));
            try_lock!(self.channels).insert(channel_name.to_owned(), new_channel.clone());
            new_channel
        };

        self.add_creator(creator, &channel_name, channel)
    }

    pub fn remove_channel(&self, channel_name: &str) {
        try_lock!(self.channels).remove(channel_name);
    }

    pub fn get_clients_for_channel(&self, _channel: MTChannel, client: MTClient) -> Vec<String> {
        let channel = try_lock!(_channel);
        let is_oper = {
            let c = try_lock!(client);
            c.is_channel_operator(&channel.name) || c.server_operator
        };
        channel.get_clients_names(is_oper)
    }

    pub fn set_channel_private(&self, channel: MTChannel, to: bool) {
        try_lock!(channel).private = to;
    }

    pub fn set_channel_secret(&self, channel: MTChannel, to: bool) {
        try_lock!(channel).secret = to;
    }

    pub fn set_channel_invite_only(&self, channel: MTChannel, to: bool) {
        try_lock!(channel).invite_only = to;
    }

    pub fn set_channel_topic_ops_only(&self, channel: MTChannel, to: bool) {
        try_lock!(channel).topic_ops_only = to;
    }

    pub fn set_channel_no_msg_outside(&self, channel: MTChannel, to: bool) {
        try_lock!(channel).no_msg_outside = to;
    }

    pub fn set_channel_moderated(&self, channel: MTChannel, to: bool) {
        try_lock!(channel).moderated = to;
    }

    pub fn set_channel_limit(
        &self,
        channel: MTChannel,
        limit: Option<u32>,
    ) -> Result<(), ServerError> {
        match limit {
            Some(val) => {
                let mut lchannel = try_lock!(channel);
                let curamount = lchannel.client_amount();
                let ch_name = &lchannel.name;
                let uval = usize::try_from(val);

                match uval {
                    Ok(usizeval) => {
                        if curamount > usizeval {
                            return Err(ServerError {
                                code: 1001,
                                msg: format!(
                                    "{} :Current amount of clients is bigger than the limit",
                                    ch_name
                                ),
                            });
                        }
                    }
                    Err(_) => {
                        return Err(ServerError {
                            code: ERR_SERVERERR,
                            msg: "Couldn't properly cast a number".to_owned(),
                        })
                    }
                }
                lchannel.limit = Some(val);
                Ok(())
            }
            None => {
                try_lock!(channel).limit = None;
                Ok(())
            }
        }
    }

    pub fn set_channel_pwd(&self, channel: MTChannel, pwd: Option<String>) {
        try_lock!(channel).password = pwd;
    }

    pub fn get_topic(&self, channel_name: &str) -> Result<Option<String>, ServerError> {
        match self.get_channel_by_name(channel_name) {
            Some(c) => Ok(try_lock!(c).get_topic()),
            None => Err(ServerError {
                code: ERR_NOSUCHCHANNEL,
                msg: format!("{} :No such channel", channel_name),
            }),
        }
    }

    pub fn set_topic(
        &self,
        client: MTClient,
        channel_name: &str,
        topic: &str,
    ) -> Result<(), ServerError> {
        match self.get_channel_by_name(channel_name) {
            Some(c) => {
                let mut channel = try_lock!(c);
                let is_oper = { try_lock!(client).is_channel_operator(channel_name) };
                let need_oper = channel.topic_ops_only;
                if need_oper && !is_oper {
                    return Err(ServerError {
                        code: ERR_CHANOPRIVSNEEDED,
                        msg: format!("{} :You're not channel operator", &channel_name),
                    });
                }

                channel.set_topic(topic);

                Ok(())
            }
            None => Err(ServerError {
                code: ERR_NOSUCHCHANNEL,
                msg: format!("{} :No such channel", channel_name),
            }),
        }
    }
}
