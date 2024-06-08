use adw::prelude::*;
use gtk::{
    glib::{object::IsA, SignalHandlerId},
    Editable, Entry,
};
use relm4::adw;

pub trait EntryExt: IsA<Entry> + 'static {
    fn connect_delegate_insert_text<F>(&self, f: F) -> SignalHandlerId
    where
        F: Fn(&Self, &Editable, &str, &mut i32) + 'static,
    {
        let entry = self.clone();
        let delegate = self.as_ref().delegate().expect("delegate");

        delegate.connect_insert_text(move |delegate, text, position| {
            f(&entry, delegate, text, position);
        })
    }
}

impl<O: IsA<Entry>> EntryExt for O {}
