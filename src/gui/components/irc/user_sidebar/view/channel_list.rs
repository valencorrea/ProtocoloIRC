use std::rc::Rc;

use glib::{clone, Cast};
use gtk::{
    traits::{ContainerExt, WidgetExt},
    Builder, ListBox, Widget,
};

use crate::gui::utils::build;

use super::channel_button_constructor::{ChannelButtonConstructor, ConstructorInformation};

pub trait ChannelChange {
    fn channel_change(&self, new_channel: &str);
}
pub struct ChannelList {
    btn_constructor_info: ConstructorInformation,
    button_list: ListBox,
}

impl ChannelList {
    pub fn init(builder: &Builder) -> Self {
        Self {
            btn_constructor_info: ChannelButtonConstructor::setup(builder),
            button_list: build(builder, "channels_list"),
        }
    }

    pub fn setup_add_btn(&self, interested: Rc<dyn ChannelChange>) {
        let (a, b) = &self.btn_constructor_info;

        self.button_list.connect_add(
            clone!(@weak a, @weak b => move |_: &ListBox, widget: &Widget| {
                let new_button = widget
                    .clone()
                    .downcast::<gtk::Button>()
                    .expect("Couldn't get button for channel");
                ChannelButtonConstructor::set_new_button(&(a,b), new_button, interested.clone());
            }),
        );
    }

    pub fn render(&self, channels: Vec<&String>) {
        let children = self.button_list.children();
        for child in &children {
            self.button_list.remove(child);
        }

        for channel in channels {
            let btn = gtk::Button::with_label(&channel);
            btn.set_visible(true);
            self.button_list.add(&btn);
        }
    }
}
