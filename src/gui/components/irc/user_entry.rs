use std::sync::mpsc::Sender;

use glib::clone;
use gtk::{
    prelude::BuilderExtManual,
    traits::{AdjustmentExt, ButtonExt, EntryExt, GtkWindowExt, ScrolledWindowExt, WidgetExt},
    ApplicationWindow, Builder, Button, Entry, ScrolledWindow, TextBuffer,
};

use crate::{
    gui::utils::{append_on_buffer, is_command, send_message},
    irc::message::Command,
};

type F<T> = &'static dyn Fn(String) -> T;

pub struct UserEntry<T> {
    gtk_window: ApplicationWindow,
    gtk_textbuffer: TextBuffer,
    gtk_scrolled: ScrolledWindow,
    gtk_entrybtn: Button,
    tx: Sender<T>,
}

impl<T> UserEntry<T> {
    pub fn start(
        builder: &Builder,
        window_id: &str,
        textbuffer_id: &str,
        scrolled_id: &str,
        entrybtn_id: &str,
        tx: Sender<T>,
    ) -> Self {
        Self {
            gtk_window: builder.object(window_id).expect("Couldn't get IrcWindow"),
            gtk_textbuffer: builder
                .object(textbuffer_id)
                .expect("Couldn't get text buffer"),
            gtk_scrolled: builder
                .object(scrolled_id)
                .expect("Couldn't get scrolled window"),
            gtk_entrybtn: builder
                .object(entrybtn_id)
                .expect("Couldn't get scrolled window"),
            tx,
        }
    }

    fn adjust_scrolled(&self) {
        let adj = self.gtk_scrolled.vadjustment();
        adj.set_value(adj.lower());
    }

    fn send_message_from_entry(&self, entry: Entry, transformer: F<T>) {
        let msg = String::from(entry.text());
        entry.set_text("");

        append_on_buffer(&self.gtk_textbuffer, &msg);
        self.adjust_scrolled();
        entry.is_focus();

        if msg.is_empty() {
            return;
        }
        let r = transformer(msg);
        if send_message(&self.tx, r).is_err() {
            self.gtk_window.close();
        }
    }

    pub fn hook(self, entry: Entry, transformer: F<T>) {
        self.gtk_entrybtn
            .connect_clicked(clone!(@weak entry => move |_|{
                entry.activate();
            }));

        entry.connect_activate(clone!(@weak entry=> move|_|{
            if is_command(&entry.text(), Command::Quit.to_str()){
                self.gtk_window.close();
                return;
            }

            self.send_message_from_entry(entry, transformer);
        }));
    }
}
