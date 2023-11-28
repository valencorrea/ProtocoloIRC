use crate::gui::actors::chat_actor::ChatActor;
use crate::gui::utils::build;
use crate::irc::ctcp::constants::{CHAT_PROTOCOL, DCC_CHAT};
use crate::irc::ctcp::dcc_relay::DccRelay;
use crate::irc::ctcp::utils::{get_selected_nick, to_notice_command};

use gtk::prelude::GtkWindowExt;
use gtk::traits::ButtonExt;
use gtk::{ApplicationWindow, Builder, Button, ButtonBox};
use std::cell::RefCell;
use std::rc::Rc;

pub struct ChatButton {}

impl ChatButton {
    pub fn start(
        builder: &Builder,
        button_id: &str,
        dcc_window: ApplicationWindow,
        dcc_relay: Rc<RefCell<DccRelay>>,
        user_container: ButtonBox,
    ) -> Self {
        let chat_button: Button = build(&builder, button_id);
        chat_button.connect_clicked(move |_| {
            let dcc_command = format!("{} {} {} {}", DCC_CHAT, CHAT_PROTOCOL, "0.0.0.0", "9290");
            let selected = get_selected_nick(&user_container);
            if let Some(nick) = selected {
                let notice_command = to_notice_command(nick.clone(), dcc_command);

                dcc_relay.as_ref().borrow().start_new_client(
                    nick.clone(),
                    notice_command,
                    |ucid, tx, storage, nick| ChatActor::build(ucid, tx, storage, nick),
                );
                dcc_window.close();
            }
        });

        return Self {};
    }
}
