extern crate gio;
extern crate glib;
extern crate gtk;

use gio::prelude::*;
use gtk::{prelude::*, ApplicationWindow, Builder, Entry, Label};
use std::cell::RefCell;
use std::rc::Rc;
use std::{
    sync::mpsc::{Receiver, RecvTimeoutError, Sender},
    time::Duration,
};

use crate::irc::{ctcp::dcc_relay::DccRelay, message::Command};

use crate::gui::actors::chat_actor::ChatActor;
use crate::gui::actors::send_actor::SendActor;
use crate::gui::{
    components::irc::{
        command_buttons::CommandButton, dcc_button::DCCButton, user_entry::UserEntry,
        window::Window,
    },
    constants::{COMMANDS, IRC_WELCOME},
    message_hub::MessageHub,
    utils::{send_message, to_server_message},
    GuiMessage, IncomingMessage,
};

use super::user_sidebar::controller::nick_messages_parser::NickParser;
use super::user_sidebar::login_watcher::LoginWatcher;
use super::user_sidebar::model::nick_storage::NickStorage;
use super::user_sidebar::view::display::UserSidebar;

fn show_ui(tx_uitosv: Sender<GuiMessage>, rx_svtoui: Receiver<IncomingMessage>) {
    gtk::init().expect("Couldn't open IrcWindow");

    let glade_src = include_str!("../../templates/ircwindow.glade");

    let builder = Builder::from_string(glade_src);

    let irc_window: ApplicationWindow =
        builder.object("IrcWindow").expect("Couldn't get IrcWindow");

    UserEntry::start(
        &builder,
        "IrcWindow",
        "textbuffer_message",
        "window_scrolled",
        "button_message",
        tx_uitosv.clone(),
    )
    .hook(
        builder.object("entry_message").expect("Couldn't get entry"),
        &to_server_message,
    );

    let mut message_hub = MessageHub::build(rx_svtoui);

    build_cmds(&builder);

    let parser = NickParser::new();
    let storage = parser.storage().clone();
    let user_sidebar = UserSidebar::init(&builder, storage.clone());

    let window = Window::start(
        &builder,
        "textbuffer_message",
        "IrcWindow",
        parser.storage(),
    );
    message_hub.add_reactor(window);

    message_hub
        .add_reactor(parser)
        .as_ref()
        .borrow_mut()
        .add_change_observer(user_sidebar);

    let dcc_relay_b: DccRelay = DccRelay::start(
        tx_uitosv.clone(),
        storage.clone(),
        |ucid, tx, storage, nick| SendActor::build(ucid, tx, storage, nick),
        |ucid, tx, storage, nick| ChatActor::build(ucid, tx, storage, nick),
    );

    let dcc_relay = message_hub.add_reactor(dcc_relay_b);
    build_dcc(&builder, dcc_relay.clone(), storage.clone());

    let login_watcher = LoginWatcher::new(tx_uitosv.clone());
    message_hub.add_reactor(login_watcher);

    let w = irc_window.clone();
    let tx = tx_uitosv.clone();

    let channelname: Label = builder
        .object("channel_name")
        .expect("Couldn't get text channel name container");
    channelname.set_label(IRC_WELCOME);

    // ON WINDOW CLOSED
    let tx2 = tx_uitosv.clone();
    irc_window.connect_delete_event(move |window: &ApplicationWindow, _| {
        let quit = Command::Quit.to_str().to_owned();
        let _ = send_message(&tx2, to_server_message(quit));
        let _ = tx2.send(GuiMessage::Close);
        unsafe {
            window.destroy();
        }
        gtk::main_quit();
        Inhibit(false)
    });

    glib::timeout_add_local(Duration::from_millis(200), move || {
        if let Err(e) = message_hub.listen_for_messages(Duration::from_millis(100)) {
            if e != RecvTimeoutError::Timeout {
                let _ = tx.send(GuiMessage::Close);
                w.close();
                return Continue(false);
            }
        }

        Continue(true)
    });

    // show window
    irc_window.show_all();

    gtk::main();
    // app.run();
}

fn build_cmds(builder: &Builder) {
    let entry: Entry = builder.object("entry_message").expect("Couldn't get entry");

    for command in COMMANDS {
        CommandButton::start(builder, command.0).hook(entry.clone(), command.1);
    }
}

fn build_dcc(
    builder: &Builder,
    dcc_relay: Rc<RefCell<DccRelay>>,
    user_storage: Rc<RefCell<NickStorage>>,
) {
    DCCButton::start(builder).hook(dcc_relay, user_storage);
}

pub fn run_app(tx_uitosv: Sender<GuiMessage>, rx_svtoui: Receiver<IncomingMessage>) {
    show_ui(tx_uitosv, rx_svtoui);
}
