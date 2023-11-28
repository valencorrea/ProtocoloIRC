use std::sync::mpsc::{SendError, Sender};

use glib::{IsA, Object};
use gtk::{prelude::BuilderExtManual, traits::TextBufferExt, Builder, TextBuffer};

use crate::irc::constants::RPL_NICKSET;

use super::GuiMessage;

pub fn check_nickname(s: &str) -> String {
    if !s.contains(&RPL_NICKSET.to_string()) {
        return "".to_owned();
    }

    let resp = s
        .to_owned()
        .split('\n')
        .map(|u| u.to_string())
        .collect::<Vec<String>>();

    let mut nick = "".to_owned();
    for r in resp {
        if !r.starts_with(&RPL_NICKSET.to_string()) {
            continue;
        }
        // 1202 :n1 :You have a new nick
        nick = r.split(':').map(|u| u.to_string()).collect::<Vec<String>>()[1]
            .trim_end()
            .to_owned();
    }

    nick
}

pub fn append_on_buffer(b: &TextBuffer, s: &str) {
    let (start, end) = b.bounds();
    let mut text: String = match b.text(&start, &end, true) {
        Some(v) => v.to_string(),
        None => "".to_string(),
    };

    // concat text to message
    text.push_str(s);
    text.push('\n');

    b.set_text(&text);
}

pub fn is_command(text: &str, command: &str) -> bool {
    text.to_uppercase().starts_with(command)
}

pub fn send_message<T>(tx: &Sender<T>, msg: T) -> Result<(), SendError<T>> {
    tx.send(msg)
}

pub fn to_server_message(s: String) -> GuiMessage {
    GuiMessage::MessageIRC(s)
}

pub fn build<T>(builder: &Builder, id: &str) -> T
where
    T: IsA<Object>,
{
    builder.object(id).expect(&format!("Couldn't get {}", id))
}
