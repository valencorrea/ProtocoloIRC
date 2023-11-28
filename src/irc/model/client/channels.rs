use crate::{irc::model::MTChannel, try_lock};

use super::Client;

impl Client {
    pub fn add_channel(&mut self, channel: MTChannel) {
        let channel_name = { try_lock!(channel).name.to_owned() };
        self.channels.insert(channel_name, channel);
    }

    pub fn is_in_channel(&self, channel_name: &str) -> bool {
        self.channels.contains_key(channel_name)
    }

    pub fn remove_channel(&mut self, channel_name: &str) {
        let _ = self.channels.remove(channel_name);
    }

    pub fn set_channel_operator(&mut self, channel_name: String, channel: MTChannel) {
        self.channel_operator.insert(channel_name, channel);
    }

    pub fn del_channel_operator(&mut self, channel_name: &str) {
        self.channel_operator.remove(channel_name);
    }

    pub fn is_channel_operator(&self, channel_name: &str) -> bool {
        self.channel_operator.contains_key(channel_name)
    }

    pub fn channel_amount(&self) -> usize {
        self.channels.keys().count()
    }

    pub fn is_invited(&self, channel_name: &str) -> bool {
        self.channel_invites.contains(&channel_name.to_owned())
    }

    pub fn add_invite(&mut self, channel_name: &str) {
        self.channel_invites.push(channel_name.to_owned());
    }

    pub fn remove_invite(&mut self, channel_name: &str) {
        self.channel_invites.retain(|ch_n| ch_n != channel_name);
    }
}
