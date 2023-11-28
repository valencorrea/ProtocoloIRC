use glib::clone;
use gtk::{
    prelude::BuilderExtManual,
    traits::{ButtonExt, EntryExt},
    Builder, Button, Entry,
};

pub struct IRCCommandBtn {
    gtk_cmdbtn: Button,
}

impl IRCCommandBtn {
    pub fn start(builder: &Builder, btn_id: &str) -> Self {
        Self {
            gtk_cmdbtn: builder.object(btn_id).expect("Couldn't get button"),
        }
    }
}

impl IRCCommandBtn {
    pub fn hook(self, entry: Entry, write_to_entry: &'static str) {
        let button = self.gtk_cmdbtn;

        button.connect_clicked(clone!(@weak button => move|_|{
            entry.set_text(write_to_entry)
        }));
    }
}
