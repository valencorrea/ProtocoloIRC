use std::rc::Rc;

use glib::clone;
use gtk::{
    traits::{ButtonExt, LabelExt, TextBufferExt},
    Builder, Button, Label, TextBuffer,
};

use crate::gui::utils::build;

use super::channel_list::ChannelChange;

pub type ConstructorInformation = (TextBuffer, Label);
pub struct ChannelButtonConstructor {}

impl ChannelButtonConstructor {
    pub fn setup(builder: &Builder) -> ConstructorInformation {
        (
            build(builder, "textbuffer_message"),
            build(builder, "channel_name"),
        )
    }
}

impl ChannelButtonConstructor {
    pub fn set_new_button(
        info: &ConstructorInformation,
        btn: Button,
        interested: Rc<dyn ChannelChange>,
    ) {
        let chat_area = &info.0;
        let channel_label = &info.1;
        btn.connect_clicked(
            clone!(@weak btn, @weak chat_area, @weak channel_label => move |_|
                let cname = btn.label().expect("Couldn't get name for channel").to_string();
                chat_area.set_text(&format!("Moved to channel {}", cname));
                channel_label.set_text(&cname);
                interested.channel_change(&cname);
            ),
        );
    }
}
