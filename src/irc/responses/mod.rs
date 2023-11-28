//! Modulo que se centra en las funcionalidades referentes a la representacion de canales.
pub mod builder;
pub mod response;

use crate::irc::constants::*;

use self::response::Response;

#[derive(PartialEq, Debug, Eq)]
pub enum InternalType {
    Quit,
    Upgrade,
}

#[derive(PartialEq, Debug, Eq)]
pub enum ResponseType {
    NoResponse,
    Code(usize),
    Content(Response),
    InternalResponse(InternalType),
}

pub fn parsing_irc_defined_error_message(error_number: usize) -> String {
    match error_number {
        ERR_NEEDMOREPARAMS => "Not enough parameters",
        ERR_NOSUCHSERVER => "No such server",
        ERR_NONICKNAMEGIVEN => "No nickname given",
        ERR_NOTEXTTOSEND => "No text to send",
        ERR_NORECIPIENT => "No recipient given",
        _ => "Badly formatted message",
    }
    .to_owned()
}

impl ResponseType {
    pub fn serialize(self) -> Option<String> {
        match self {
            Self::Code(v) => Some(v.to_string()),
            Self::Content(rs) => Some(rs.serialize()),
            _ => None,
        }
    }
}
