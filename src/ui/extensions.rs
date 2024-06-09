use gtk::{
    glib::{object::IsA, SignalHandlerId},
    prelude::{EntryExt as _, *},
    Editable, Entry,
};
use relm4::prelude::*;

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

    fn enable_input_purpose_behavior(&self) -> SignalHandlerId {
        self.connect_delegate_insert_text(|this, delegate, text, _position| {
            if this.input_purpose() == gtk::InputPurpose::Digits
                && text.chars().any(|c| !c.is_ascii_digit())
            {
                delegate.stop_signal_emission_by_name("insert-text");
            }
        })
    }
}

impl<O: IsA<Entry>> EntryExt for O {}
