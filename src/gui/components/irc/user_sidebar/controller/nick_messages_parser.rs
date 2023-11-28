use std::{cell::RefCell, rc::Rc};

use crate::{
    gui::{
        components::irc::user_sidebar::model::nick_storage::NickStorage, IncomingMessage, Reactor,
    },
    irc::constants::{RPL_CHANNELOUT, RPL_NAMREPLY, RPL_NICKCHANGE, RPL_NICKIN, RPL_NICKOUT},
};

use super::{NickUpdateEvent, Observer};

pub struct NickParser {
    change_observer: Vec<Rc<dyn Observer>>,
    storage: Rc<RefCell<NickStorage>>,
}

impl NickParser {
    pub fn new() -> Self {
        Self {
            change_observer: vec![],
            storage: Rc::new(RefCell::new(NickStorage::new())),
        }
    }
}

impl NickParser {
    pub fn add_change_observer(&mut self, observer: Rc<dyn Observer>) {
        self.change_observer.push(observer);
    }

    #[allow(dead_code)]
    pub fn remove_change_observer(&mut self, observer: Rc<dyn Observer>) {
        let res = self
            .change_observer
            .iter()
            .position(|x| Rc::ptr_eq(x, &observer));
        if let Some(idx) = res {
            self.change_observer.remove(idx);
        }
    }
}

impl Reactor for NickParser {
    fn react_single(&mut self, message: &IncomingMessage) {
        if let IncomingMessage::Server(msg) = message {
            let event_result = self.server_message(msg);
            if let Some(event) = event_result {
                for observer in &self.change_observer {
                    observer.update(&event)
                }
            }
        }
    }
}

impl NickParser {
    pub fn storage(&self) -> Rc<RefCell<NickStorage>> {
        self.storage.clone()
    }
}

impl NickParser {
    fn extract_code(msg: &str) -> Option<(usize, &str)> {
        let split: Vec<&str> = msg.split(":").collect();
        if split.len() != 2 {
            return None;
        }

        let code = usize::from_str_radix(*split.first().unwrap(), 10).ok()?;

        Some((code, *split.get(1).unwrap()))
    }

    fn server_message(&mut self, msg: &str) -> Option<NickUpdateEvent> {
        let (code, rest) = Self::extract_code(msg)?;
        let ret = match code {
            RPL_NAMREPLY => self.add_to_channel(rest),
            RPL_CHANNELOUT => self.remove_from_channel(rest),
            RPL_NICKIN => self.add_user(rest),
            RPL_NICKCHANGE => self.change_user(rest),
            RPL_NICKOUT => self.remove_user(rest),
            _ => None,
        };
        ret
    }
}

impl NickParser {
    fn extract(how_many: usize, msg: &str) -> Option<Vec<&str>> {
        let splitted: Vec<&str> = msg
            .split(" ")
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
            .collect();

        if splitted.len() != how_many {
            return None;
        }

        Some(splitted)
    }

    fn channel_user(msg: &str) -> Option<(&str, &str)> {
        let extracted = Self::extract(2, msg)?;
        Some((extracted.get(0)?, extracted.get(1)?))
    }

    fn user(msg: &str) -> Option<&str> {
        let extracted = Self::extract(1, msg)?;
        Some(extracted.get(0)?)
    }

    fn user_user(msg: &str) -> Option<(&str, &str)> {
        let extracted = Self::extract(2, msg)?;
        Some((extracted.get(0)?, extracted.get(1)?))
    }

    fn add_to_channel(&mut self, rest: &str) -> Option<NickUpdateEvent> {
        let (channel, nick) = Self::channel_user(rest)?;

        if channel == "*" {
            return self.add_user(nick);
        }

        self.storage
            .as_ref()
            .borrow_mut()
            .add_to_channel(channel, nick);

        Some(NickUpdateEvent::ChannelInsert(
            channel.to_owned(),
            nick.to_owned(),
        ))
    }

    fn remove_from_channel(&mut self, rest: &str) -> Option<NickUpdateEvent> {
        let (channel, nick) = Self::channel_user(rest)?;

        self.storage
            .as_ref()
            .borrow_mut()
            .remove_from_channel(channel, nick);

        Some(NickUpdateEvent::ChannelDelete(
            channel.to_owned(),
            nick.to_owned(),
        ))
    }

    fn add_user(&mut self, rest: &str) -> Option<NickUpdateEvent> {
        let nick = Self::user(rest)?;

        self.storage.as_ref().borrow_mut().add_nick(nick);

        return Some(NickUpdateEvent::NoChanges);
    }

    fn change_user(&mut self, rest: &str) -> Option<NickUpdateEvent> {
        let (old, new) = Self::user_user(rest)?;

        self.storage.as_ref().borrow_mut().change_nick(old, new);

        Some(NickUpdateEvent::NickChange(old.to_owned(), new.to_owned()))
    }

    fn remove_user(&mut self, rest: &str) -> Option<NickUpdateEvent> {
        let nick = Self::user(rest)?;

        self.storage.as_ref().borrow_mut().remove_nick(nick);

        Some(NickUpdateEvent::NoChanges)
    }
}
