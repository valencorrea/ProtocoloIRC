use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct NickStorage {
    all_nicks: HashSet<String>,
    channels: HashMap<String, HashSet<String>>,
    current_nick: Option<String>,
}

impl NickStorage {
    pub fn new() -> Self {
        Self {
            all_nicks: HashSet::new(),
            channels: HashMap::new(),
            current_nick: None,
        }
    }
}

impl NickStorage {
    fn remove_from_channels(&mut self, nick: &str) {
        for nicks in self.channels.values_mut() {
            nicks.remove(nick);
        }
    }

    pub fn add_nick(&mut self, nick: &str) {
        self.all_nicks.insert(nick.to_owned());
    }

    pub fn add_channel(&mut self, channel: &str) {
        if !self.channels.contains_key(channel) {
            self.channels.insert(channel.to_owned(), HashSet::new());
        }
    }

    pub fn add_to_channel(&mut self, channel: &str, nick: &str) {
        self.add_channel(channel);
        self.add_nick(nick);
        self.channels
            .get_mut(channel)
            .and_then(|nicks| Some(nicks.insert(nick.to_owned())));
    }

    pub fn change_nick(&mut self, old: &str, new: &str) {
        self.all_nicks.remove(old);
        self.all_nicks.insert(new.to_owned());

        for nicks in self.channels.values_mut() {
            if nicks.contains(old) {
                nicks.remove(old);
                nicks.insert(new.to_owned());
            }
        }
    }

    pub fn remove_nick(&mut self, nick: &str) {
        self.all_nicks.remove(nick);
        self.remove_from_channels(nick);
    }

    pub fn remove_from_channel(&mut self, channel: &str, nick: &str) {
        if let Some(nicks) = self.channels.get_mut(channel) {
            nicks.remove(nick);
        }
    }

    pub fn set_user_nick(&mut self, nick: &str) {
        self.current_nick = Some(nick.trim().to_owned())
    }
}

impl NickStorage {
    fn to_vec(hs: &HashSet<String>) -> Vec<&str> {
        Vec::from_iter(hs.iter().map(|v| v.as_str()))
    }

    pub fn get_all(&self) -> Vec<String> {
        self.all_nicks.iter().map(|v| v.to_owned()).collect()
    }

    pub fn get_channel(&self, channel: &str) -> Option<Vec<&str>> {
        Some(Self::to_vec(self.channels.get(channel)?))
    }

    pub fn get_channels(&self) -> Vec<&String> {
        self.channels.keys().collect()
    }

    pub fn get_user_nick(&self) -> Option<String> {
        self.current_nick.to_owned()
    }
}
