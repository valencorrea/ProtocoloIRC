use glib::clone;
use gtk::{prelude::BuilderExtManual, traits::ButtonExt, Builder, Button};
use std::cell::RefCell;
use std::rc::Rc;

use crate::gui::components::dcc::opening_window::show_dcc;
use crate::irc::ctcp::dcc_relay::DccRelay;

use super::user_sidebar::model::nick_storage::NickStorage;

pub struct DCCButton {
    gtk_cmdbtn: Button,
}

impl DCCButton {
    pub fn start(builder: &Builder) -> Self {
        Self {
            gtk_cmdbtn: builder
                .object("dcc_button")
                .expect("Couldn't get dcc button"),
        }
    }
}

impl DCCButton {
    pub fn hook(self, dcc_relay: Rc<RefCell<DccRelay>>, user_storage: Rc<RefCell<NickStorage>>) {
        let button = self.gtk_cmdbtn;

        button.connect_clicked(clone!(@weak button => move|_|{
            show_dcc(dcc_relay.clone(), user_storage.clone());
        }));
    }
}
