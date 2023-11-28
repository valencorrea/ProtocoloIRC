use crate::gui::components::irc::user_sidebar::model::nick_storage::NickStorage;
use crate::gui::utils::{append_on_buffer, build};
use crate::gui::GuiMessage;
use crate::irc::ctcp::constants::DCC_CLOSE;
use crate::irc::ctcp::dcc_relay::{DccAction, DccActor};
use crate::irc::ctcp::utils::to_notice_command;
use glib::{clone, GString};
use gtk::prelude::WidgetExtManual;
use gtk::traits::{ButtonExt, EntryExt, GtkWindowExt, TextViewExt, WidgetExt};
use gtk::{ApplicationWindow, Builder, Button, TextView};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub struct ChatActor {
    text_area: TextView,
    window: ApplicationWindow,
    nick: String,
}

impl ChatActor {
    fn setup() -> Builder {
        let dcc_src = include_str!("../templates/dcc-chat-window.glade");
        Builder::from_string(dcc_src)
    }

    pub fn build(
        ucid: usize,
        tx: Sender<GuiMessage>,
        storage: Rc<RefCell<NickStorage>>,
        nick: &str,
    ) -> Rc<Self> {
        let builder: Builder = Self::setup();

        let btn: Button = build(&builder, "button_message");

        let text_area: TextView = build(&builder, "textview_message");

        let entry: gtk::Entry = build(&builder, "entry_message");

        btn.connect_clicked(clone!(@weak text_area, @weak entry => move |_|{
            entry.activate();
        }));

        let tx_cp = tx.clone();

        let current_nick = storage
            .as_ref()
            .borrow()
            .get_user_nick()
            .unwrap_or("No esta registrado".to_owned())
            .clone();

        entry.connect_activate(clone!(@weak entry, @weak text_area => move |_| {
            let msg = entry.text().to_string();
            if msg.is_empty(){
                return;
            }
            if let Some(buffer) = text_area.buffer(){
                append_on_buffer(&buffer, &format!("{}: {}", &current_nick, msg));
            }

            let _ = tx_cp.send(GuiMessage::OutgoingDCC(ucid, msg));
            entry.set_text("");
        }));

        let window: ApplicationWindow = build(&builder, "DCCChatWindow");
        let title = window.title().unwrap_or_else(|| GString::from(""));
        window.set_title(&format!(
            "{} - Current User {} Talking to {}",
            title,
            storage
                .as_ref()
                .borrow()
                .get_user_nick()
                .unwrap_or("No esta registrado".to_owned()),
            nick
        ));

        let tx_cp = tx.clone();
        let nickname = nick.to_string();
        window.connect_delete_event(move |w, _| {
            let msg = to_notice_command(nickname.clone(), DCC_CLOSE.to_string());

            let _ = tx_cp.send(GuiMessage::OutgoingDCC(ucid, msg));
            unsafe {
                w.destroy();
            }
            gtk::Inhibit(false)
        });

        return Rc::new(Self {
            text_area,
            window,
            nick: nick.to_owned(),
        });
    }
}

impl DccActor for ChatActor {
    fn act(&self, _: usize, action: DccAction) {
        match action {
            DccAction::New => {
                self.window.show_all();
            }
            DccAction::NewMessage(content) => {
                self.update_text_area(&content);
            }
            DccAction::FileUpdate(_, _) => {
                println!("Ignore");
            }
            DccAction::Destroy => {
                self.window.close();
            }
        }
    }
}

impl ChatActor {
    fn update_text_area(&self, content: &str) {
        if let Some(buffer) = self.text_area.buffer() {
            append_on_buffer(&buffer, &format!("{}: {}", self.nick, content));
        }
    }
}
