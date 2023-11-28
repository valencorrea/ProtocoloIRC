use std::{cell::RefCell, rc::Rc};

use gtk::Builder;

use crate::{
    gui::components::irc::user_sidebar::{
        controller::{NickUpdateEvent, Observer},
        model::nick_storage::NickStorage,
    },
    ignore,
};

use super::{
    channel_list::{ChannelChange, ChannelList},
    user_list::UserList,
};

struct CurrentChannel {
    pub value: Option<String>,
}

pub struct UserSidebar {
    storage: Rc<RefCell<NickStorage>>,
    current_channel: RefCell<CurrentChannel>,
    channel_buttons_container: ChannelList,
    user_buttons_container: UserList,
}

impl UserSidebar {
    pub fn init(builder: &Builder, storage: Rc<RefCell<NickStorage>>) -> Rc<Self> {
        let ret = Rc::new(Self {
            storage,
            current_channel: RefCell::new(CurrentChannel { value: None }),
            channel_buttons_container: ChannelList::init(builder),
            user_buttons_container: UserList::init(builder),
        });

        ret.channel_buttons_container.setup_add_btn(ret.clone());
        ret.user_buttons_container.setup_add_btn();
        ret
    }
}

impl UserSidebar {
    fn render_users(&self, channel: &str, storage: &NickStorage) {
        if let Some(content) = storage.get_channel(channel) {
            self.user_buttons_container.render(content);
        }
    }

    fn render_channels(&self) {
        self.channel_buttons_container
            .render(self.storage.as_ref().borrow().get_channels())
    }
}

impl ChannelChange for UserSidebar {
    fn channel_change(&self, new_channel: &str) {
        let mut channel_name = self.current_channel.borrow_mut();
        channel_name.value = Some(new_channel.to_owned());
        self.render_users(new_channel, &self.storage.as_ref().borrow());
    }
}

impl Observer for UserSidebar {
    fn update(&self, event: &NickUpdateEvent) {
        match event {
            NickUpdateEvent::ChannelInsert(channel, _)
            | NickUpdateEvent::ChannelDelete(channel, _) => {
                self.render_channels();

                if let Some(current) = &self.current_channel.borrow().value {
                    if current == channel {
                        self.render_users(&channel, &self.storage.borrow());
                    }
                }
            }
            NickUpdateEvent::NickChange(_, _) => {
                todo!()
            }
            _ => {
                ignore!();
            }
        };
    }
}
