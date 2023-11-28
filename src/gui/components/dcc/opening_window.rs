extern crate gio;
extern crate glib;
extern crate gtk;

use glib::Cast;
use gtk::traits::*;
use gtk::*;
use std::{cell::RefCell, rc::Rc};

use crate::gui::components::irc::user_sidebar::model::nick_storage::NickStorage;
use crate::gui::{components::dcc::chat_button::ChatButton, utils::build};
use crate::irc::ctcp::dcc_relay::DccRelay;

use super::send_button::SendButton;

pub fn show_dcc(dcc_relay: Rc<RefCell<DccRelay>>, user_storage: Rc<RefCell<NickStorage>>) {
    gtk::init().expect("Couldn't open DCCWindow");

    let dcc_src = include_str!("../../templates/dcc-opening.glade");
    let builder = Builder::from_string(dcc_src);
    let dcc_window: ApplicationWindow = build(&builder, "DCCOpeningWindow");

    let user_container = build(&builder, "users_list");

    let current_user = {
        let user_storage_ref = user_storage.as_ref().borrow();
        user_storage_ref.get_user_nick()
    };

    let client_list = {
        let user_storage_ref = user_storage.as_ref().borrow();

        user_storage_ref
            .get_all()
            .into_iter()
            .filter_map(|user| {
                if let Some(current) = &current_user {
                    if current == &user {
                        return None;
                    }
                    return Some(user);
                }
                return None;
            })
            .collect()
    };

    setup_user_container(&user_container);
    create_user_buttons(&user_container, client_list);

    ChatButton::start(
        &builder,
        "chat_button",
        dcc_window.clone(),
        dcc_relay.clone(),
        user_container.clone(),
    );

    SendButton::start(
        &builder,
        "file_chooser",
        dcc_window.clone(),
        dcc_relay,
        user_container,
    );

    dcc_window.show_all();

    gtk::main();
}

fn setup_user_container(user_container: &ButtonBox) {
    user_container.connect_add(move |_: &ButtonBox, widget: &Widget| {
        let new_button = widget
            .clone()
            .downcast::<gtk::RadioButton>()
            .expect("Couldn't get button for user");
        action_for_btn(new_button);
    });
}

fn create_user_buttons(user_container: &ButtonBox, users: Vec<String>) {
    let mut group: Option<RadioButton> = None;

    for user in users {
        let button = RadioButton::with_label(&user);
        button.join_group(group.as_ref());
        button.set_visible(true);
        user_container.add(&button);
        if group.is_none() {
            group = Some(button);
        }
    }
}

fn action_for_btn(btn: RadioButton) {
    btn.connect_clicked(|btn: &RadioButton| {
        println!("{:?}", btn.label().expect("esploto"));
    });
}
