use std::{cell::RefCell, rc::Rc};

use glib::GString;
use gtk::{
    prelude::BuilderExtManual, traits::GtkWindowExt, ApplicationWindow, Builder, TextBuffer,
};

use crate::{
    gui::{
        utils::{append_on_buffer, check_nickname},
        IncomingMessage, Reactor,
    },
    irc::responses::response::Response,
};

use super::user_sidebar::model::nick_storage::NickStorage;

pub struct Window {
    gtk_textbuffer: TextBuffer,
    pub gtk_window: ApplicationWindow,
    nick_storage: Rc<RefCell<NickStorage>>,
}

impl Window {
    pub fn start(
        builder: &Builder,
        text_buffer: &str,
        window: &str,
        nick_storage: Rc<RefCell<NickStorage>>,
    ) -> Self {
        Self {
            gtk_textbuffer: builder
                .object(text_buffer)
                .expect("Couldn't get text buffer"),
            gtk_window: builder.object(window).expect("Couldn't get IrcWindow"),
            nick_storage,
        }
    }
}

impl Reactor for Window {
    fn react_single(&mut self, message: &IncomingMessage) {
        if let IncomingMessage::Server(msg) = message {
            let title = self.gtk_window.title().unwrap_or_else(|| GString::from(""));
            if !title.contains(" - ") {
                let nick = check_nickname(msg);
                if !nick.is_empty() {
                    self.gtk_window.set_title(&format!("{} - {}", title, nick));
                    self.nick_storage.as_ref().borrow_mut().set_user_nick(&nick)
                }
            }
            if !msg.is_empty() {
                let response = Response::deserialize(&msg);
                if response.is_printable() {
                    append_on_buffer(&self.gtk_textbuffer, msg);
                }
            }
        }
    }
}
