pub mod nick_messages_parser;

type Channel = String;
type User = String;

#[derive(Debug)]
pub enum NickUpdateEvent {
    NoChanges,
    ChannelInsert(Channel, User),
    ChannelDelete(Channel, User),
    NickChange(User, User),
}

pub trait Observer {
    fn update(&self, event: &NickUpdateEvent);
}
