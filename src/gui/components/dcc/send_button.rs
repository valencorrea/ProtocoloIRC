use glib::{clone, Cast};
use gtk::traits::{ButtonExt, ContainerExt, FileChooserExt, GtkWindowExt, ToggleButtonExt};
use gtk::{ApplicationWindow, Builder, ButtonBox, FileChooserButton, Widget};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use crate::gui::actors::send_actor::SendActor;
use crate::gui::utils::build;
use crate::irc::ctcp::constants::DCC_SEND;
use crate::irc::ctcp::dcc_relay::DccRelay;
use crate::irc::ctcp::utils::to_notice_command;

use crate::irc::ctcp::utils::PATH_FILES_UPLOAD;

pub struct SendButton {}

impl SendButton {
    pub fn start(
        builder: &Builder,
        button_id: &str,
        dcc_window: ApplicationWindow,
        dcc_relay: Rc<RefCell<DccRelay>>,
        user_container: ButtonBox,
    ) -> Self {
        let file_chooser: FileChooserButton = build(&builder, button_id);

        file_chooser.set_current_folder(Path::new(&PATH_FILES_UPLOAD));

        file_chooser.connect_selection_changed(clone!(@weak file_chooser => move |_| {
            let path = file_chooser.filename();

            if path.is_none() {
                return;
            }

            let path_buf = path.expect(&"Couldn't get file path".to_string());
            let filen = path_buf.file_name().and_then(|v| {v.to_str()});
            let files = path_buf.metadata().and_then(|v| {Ok(v.len())});
            if filen.is_none() || files.is_err(){
                return;
            }

            let file_name = filen.expect("Se pudrio todo");
            let file_size = files.expect("Se pudrio todo 2: La venganza del archivo");

            let dcc_command = format!("{} {} {} {} {}", DCC_SEND, file_name, "0.0.0.0", "9290", file_size);

            let selected = get_selected_nick(&user_container);

            if let Some(nick) = selected {
                let notice_command = to_notice_command(nick.clone(), dcc_command);
                println!("Notice command: {}", notice_command);
                dcc_relay.as_ref().borrow().start_new_client(
                    nick.clone(),
                    notice_command,
                    |ucid, tx, storage, nick| {SendActor::build(ucid, tx, storage, nick)},
                );
                dcc_window.close();
            }
        }));

        return Self {};
    }
}

fn get_selected_nick(user_container: &ButtonBox) -> Option<String> {
    user_container
        .children()
        .iter()
        .find_map(|w: &Widget| {
            let b = w.clone().downcast::<gtk::RadioButton>().unwrap();
            match b.is_active() {
                true => b.label(),
                false => None,
            }
        })
        .map(|v| v.to_string())
}
