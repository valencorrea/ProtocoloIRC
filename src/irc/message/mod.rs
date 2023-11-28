//! Modulo que se centra en las funcionalidades referentes al mensaje de mod.
use self::generic_message::GenericMessage;
use crate::irc::model::server::Server;
use crate::irc::responses::ResponseType;
use crate::try_lock;
use std::ops::Deref;

use super::model::{MTClient, MTServerConnection};

pub mod away;
pub mod channel_mode;
pub mod generic_message;
pub mod generic_mode;
pub mod invite;
pub mod join;
pub mod kick;
pub mod list;
pub mod names;
pub mod nickname;
pub mod notice;
pub mod oper;
pub mod part;
pub mod password;
pub mod private;
pub mod quit;
pub mod serializer;
pub mod server;
pub mod server_quit;
pub mod topic;
pub mod user;
pub mod user_mode;
pub mod utils;
pub mod who;
pub mod whois;

#[derive(Debug, PartialEq, Eq)]
pub enum MessageError {
    EmptyMessage,
    MessageTooLong,
    InvalidCommand,
    InvalidFormat,
    InvalidHostmask,
    TooManyParams,
    IRCDefined(usize),
    DCCDefined(usize),
}

pub enum ReceiverType {
    HostMask(String),
    ServerMask(String),
    Nickname(String),
    ChannelName(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Password,
    Nick,
    User,
    Server,
    Oper,
    Quit,
    ServerQuit,
    Join,
    Part,
    Mode,
    Topic,
    Names,
    List,
    Invite,
    Kick,
    PrivateMessage,
    Notice,
    Who,
    WhoIs,
    Away,
}

impl Command {
    pub fn to_str(&self) -> &'static str {
        match self {
            Command::Password => "PASS",
            Command::Nick => "NICK",
            Command::User => "USER",
            Command::Server => "SERVER",
            Command::Oper => "OPER",
            Command::Quit => "QUIT",
            Command::ServerQuit => "SQUIT",
            Command::Join => "JOIN",
            Command::Part => "PART",
            Command::Mode => "MODE",
            Command::Topic => "TOPIC",
            Command::Names => "NAMES",
            Command::List => "LIST",
            Command::Invite => "INVITE",
            Command::Kick => "KICK",
            Command::PrivateMessage => "PRIVMSG",
            Command::Notice => "NOTICE",
            Command::Who => "WHO",
            Command::WhoIs => "WHOIS",
            Command::Away => "AWAY",
        }
    }

    fn from_str(command: &str) -> Option<Command> {
        match command {
            "NICK" => Some(Command::Nick),
            "PASS" => Some(Command::Password),
            "USER" => Some(Command::User),
            "SERVER" => Some(Command::Server),
            "SQUIT" => Some(Command::ServerQuit),
            "OPER" => Some(Command::Oper),
            "QUIT" => Some(Command::Quit),
            "PRIVMSG" => Some(Command::PrivateMessage),
            "NOTICE" => Some(Command::Notice),
            "JOIN" => Some(Command::Join),
            "PART" => Some(Command::Part),
            "NAMES" => Some(Command::Names),
            "LIST" => Some(Command::List),
            "INVITE" => Some(Command::Invite),
            "WHO" => Some(Command::Who),
            "WHOIS" => Some(Command::WhoIs),
            "MODE" => Some(Command::Mode),
            "TOPIC" => Some(Command::Topic),
            "KICK" => Some(Command::Kick),
            "AWAY" => Some(Command::Away),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum ChannelModes<'a> {
    Operator(&'a [u8]),
    Private,
    Secret,
    InviteOnly,
    TopicOnlyOperators,
    NoMessageFromOutside,
    Moderated,
    Limit(Option<u32>),
    SpeakInModeratedChannel(&'a [u8]),
    ChannelKey(Option<&'a [u8]>),
}

#[derive(Debug)]
pub enum UserModes {
    Invisible,
    ReceiveServerNotices,
    IRCOperator,
}

#[derive(Debug)]
pub enum ModesAction<T> {
    Add(T),
    Remove(T),
}

pub const COLON: u8 = 58;
pub const SPACE: u8 = 32;
pub const DOT: u8 = 46;
pub const CR: u8 = 13;
pub const LF: u8 = 10;
pub const NUL: u8 = 0;
pub const BELL: u8 = 7;
pub const COMMA: u8 = 44;
pub const HASH: u8 = 35;
pub const DOLLAR: u8 = 36;
pub const AMPERSAND: u8 = 38;
pub const QUESTION_MARK: u8 = 63;
pub const ASTERISK: u8 = 42;
pub const MAX_HOSTNAME_LENGTH: usize = 253;
pub const MAX_HOSTNAME_LABEL_LENGTH: usize = 63;
pub const BRACKET_OPEN: u8 = 91;
pub const BRACKET_CLOSE: u8 = 93;

//Message can't be more than 510 bytes long, therefore at most you can get ~250 params. The number is exagerated so it is easier
pub const UNLIMITED_MAX_LEN: usize = 600;

pub trait FromGeneric<'a> {
    fn from_generic(generic: GenericMessage<'a>) -> Result<Self, MessageError>
    where
        Self: Sized;
}

pub trait Serializable {
    fn serialize(&self) -> String;
}

pub trait Replicable: Serializable {
    fn _execute(&self, server: &Server, client: MTClient) -> (Vec<ResponseType>, bool);

    fn forward(&mut self, client: MTClient) -> String;

    fn execute(&mut self, server: &Server, client: MTClient) -> Vec<ResponseType> {
        let (response, should_replicate) = self._execute(server, client.clone());
        let forward = self.forward(client);
        if should_replicate {
            server.replicate_to_all_servers(&forward);
        }
        response
    }
}

pub trait Executable: Serializable {
    fn _execute(&self, server: &Server, client: MTClient) -> Vec<ResponseType>;

    fn execute(&mut self, server: &Server, client: MTClient) -> Vec<ResponseType> {
        self._execute(server, client)
    }
}

pub trait ServerExecutable: Serializable
where
    Self: std::fmt::Debug,
{
    fn _execute_for_server(&self, server: &Server) -> Vec<ResponseType>;

    fn forward(&self, _: &Server, _: &MTServerConnection) -> String {
        self.serialize()
    }

    fn replicate(&self, server: &Server, origin: MTServerConnection) {
        let forward = self.forward(server, &origin);
        let origin = { try_lock!(origin).servername.to_owned() };
        server.replicate_to_all_servers_sans_origin(&forward, &origin);
    }

    fn execute_for_server(&self, server: &Server, origin: MTServerConnection) -> Vec<ResponseType> {
        let res = self._execute_for_server(server);
        self.replicate(server, origin);
        res
    }
}

impl<T> ModesAction<T> {
    pub fn as_byte(&self) -> u8 {
        match self {
            ModesAction::Add(_) => b'+',
            ModesAction::Remove(_) => b'-',
        }
    }
}

impl<T> Deref for ModesAction<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            ModesAction::Add(v) => v,
            ModesAction::Remove(v) => v,
        }
    }
}
