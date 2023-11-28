use glib::{clone, Cast};
use gtk::{
    traits::{ContainerExt, WidgetExt},
    Builder, Button, ListBox, Widget,
};

use crate::gui::utils::build;

use super::user_button_constructor::{UserButtonConstructor, UserButtonConstructorInformation};

pub struct UserList {
    btn_constructor_info: UserButtonConstructorInformation,
    user_list: ListBox,
}

impl UserList {
    pub fn init(builder: &Builder) -> Self {
        Self {
            btn_constructor_info: UserButtonConstructor::setup(builder),
            user_list: build(builder, "users_list"),
        }
    }
}

impl UserList {
    pub fn setup_add_btn(&self) {
        let info = &self.btn_constructor_info;

        self.user_list
            .connect_add(clone!(@weak info => move |_: &ListBox, widget: &Widget| {
                let new_button = widget
                    .clone()
                    .downcast::<gtk::Button>()
                    .expect("Couldn't get button for channel");
                UserButtonConstructor::set_new_button(&info, new_button);
            }));
    }

    pub fn render(&self, users: Vec<&str>) {
        for child in self.user_list.children() {
            self.user_list.remove(&child);
        }

        for user in users {
            let button = Button::with_label(user);
            button.set_visible(true);
            self.user_list.add(&button);
        }
    }
}
