use crate::gui::components::irc::user_sidebar::model::nick_storage::NickStorage;
use crate::gui::utils::build;
use crate::gui::GuiMessage;
use crate::ignore;
use crate::irc::ctcp::constants::DCC_CLOSE;
use crate::irc::ctcp::dcc_relay::{DccAction, DccActor};
use crate::irc::ctcp::utils::to_notice_command;
use glib::GString;
use gtk::prelude::WidgetExtManual;
use gtk::traits::{GtkWindowExt, ProgressBarExt, WidgetExt};
use gtk::{ApplicationWindow, Builder, ProgressBar};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub struct SendActor {
    window: ApplicationWindow,
    progress_bar: ProgressBar,
}

impl SendActor {
    fn setup() -> Builder {
        let dcc_src = include_str!("../templates/dcc-send-window.glade");
        Builder::from_string(dcc_src)
    }

    pub fn build(
        ucid: usize,
        tx: Sender<GuiMessage>,
        storage: Rc<RefCell<NickStorage>>,
        nick: &str,
    ) -> Rc<Self> {
        let builder = Self::setup();

        let progress_bar = build(&builder, "progress_bar");

        let current_nick = storage
            .as_ref()
            .borrow()
            .get_user_nick()
            .unwrap_or("No esta registrado".to_owned())
            .clone();

        let window: ApplicationWindow = build(&builder, "DCCSendWindow");
        let title = window.title().unwrap_or_else(|| GString::from(""));
        window.set_title(&format!("{} - From {} To {}", title, current_nick, nick));

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
            window,
            progress_bar,
        });
    }
}

impl DccActor for SendActor {
    fn act(&self, _: usize, action: DccAction) {
        match action {
            DccAction::New => {
                self.window.show_all();
            }
            DccAction::NewMessage(_) => {
                ignore!();
            }
            DccAction::FileUpdate(complete, partial) => {
                let fraction: f64 = partial as f64 / complete as f64;
                self.progress_bar.set_fraction(fraction);
            }
            DccAction::Destroy => {
                self.window.close();
            }
        }
    }
}
