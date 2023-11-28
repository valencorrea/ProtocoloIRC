//! Modulo que se centra en las funcionalidades referentes a la representacion de canales.

use crate::irc::constants::{
    ERR_SERVERERR, RPL_ENDOFNAMES, RPL_LIST, RPL_LISTEND, RPL_LISTSTART, RPL_NAMREPLY,
};
#[derive(PartialEq, Debug, Eq)]
pub struct Response {
    numeric: usize,
    content: String,
}
impl Response {
    pub fn new(number: usize, content: String) -> Self {
        Self {
            numeric: number,
            content,
        }
    }

    pub fn serialize(self) -> String {
        format!("{}: {}", self.numeric, self.content)
    }

    pub fn deserialize(response: &str) -> Self {
        let r: Vec<&str> = response.split(':').collect();

        Self {
            numeric: match r[0].trim_end().parse::<usize>() {
                Ok(v) => v,
                Err(_) => 0,
            },
            content: if r.len() > 1 {
                r[1].to_owned()
            } else {
                String::new()
            },
        }
    }

    pub fn is_printable(&self) -> bool {
        let mut non_printable_nums: Vec<usize> = Vec::new();

        non_printable_nums.push(ERR_SERVERERR);
        non_printable_nums.push(RPL_LISTSTART);
        non_printable_nums.push(RPL_LIST);
        non_printable_nums.push(RPL_LISTEND);
        non_printable_nums.push(RPL_NAMREPLY);
        non_printable_nums.push(RPL_ENDOFNAMES);

        !(non_printable_nums.contains(&self.numeric))
    }
}
