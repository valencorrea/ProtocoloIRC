use glib::clone;
use gtk::{
    traits::{ButtonExt, EntryExt},
    Builder, Button, Entry,
};

use crate::{gui::utils::build, irc::message::Command};

pub type UserButtonConstructorInformation = Entry;

pub struct UserButtonConstructor {}

impl UserButtonConstructor {
    pub fn setup(builder: &Builder) -> UserButtonConstructorInformation {
        build(builder, "entry_message")
    }
}

impl UserButtonConstructor {
    pub fn set_new_button(info: &UserButtonConstructorInformation, button: Button) {
        let entry = info;

        button.connect_clicked(clone!(@weak entry => move |btn|
            let user_name = btn.label().expect("Couldn't get name for user").to_string();
            entry.set_text(&format!("{} {} :", Command::PrivateMessage.to_str(), user_name));
        ));
    }
}
