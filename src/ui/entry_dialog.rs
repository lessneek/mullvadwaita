use adw::prelude::*;
use relm4::prelude::*;
use relm4::SimpleComponent;
use std::fmt;

use super::variant_selector::EntryConverter;

#[tracker::track]
pub struct EntryDialog<T: fmt::Debug> {
    dialog: adw::Dialog,
    title: String,

    #[no_eq]
    value: Option<T>,

    #[no_eq]
    entry_text: String,
    error: Option<String>,

    #[do_not_track]
    converter: Option<EntryConverter<T, String>>,
}

impl<T: fmt::Debug> EntryDialog<T> {
    fn get_value_as_text(&self) -> String {
        if let (Some(value), Some(converter)) = (self.get_value(), &self.converter) {
            converter.to_string(value).unwrap_or_default()
        } else {
            "".to_string()
        }
    }

    fn error_changed(&self) -> bool {
        self.changed(Self::error())
    }
}

impl<T: fmt::Debug> fmt::Debug for EntryDialog<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EntryDialog")
            .field("dialog", &self.dialog)
            .field("value", &self.value)
            .field("error", &self.error)
            .finish()
    }
}

pub struct EntryDialogInit {
    pub ok_button_label: String,
}

#[derive(Debug)]
pub enum EntryDialogMsg<T: fmt::Debug + 'static> {
    Open {
        value: T,
        title: String,
        converter: EntryConverter<T, String>,
        parent: gtk::Widget,
    },
    TextChanged(String),
    Apply,
}

#[derive(Debug)]
pub struct EntryDialogOutput<T: fmt::Debug> {
    pub value: T,
}

#[relm4::component(pub)]
impl<T> SimpleComponent for EntryDialog<T>
where
    T: fmt::Debug + PartialEq + 'static,
{
    type Input = EntryDialogMsg<T>;

    type Output = EntryDialogOutput<T>;

    type Init = EntryDialogInit;

    type Widgets = EntryDialogWidgets;

    view! {
        adw::Dialog {
            set_width_request: 300,

            #[wrap(Some)]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar {
                    add_css_class: "flat",

                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        #[track = "model.changed(EntryDialog::<T>::title())"]
                        set_title: model.get_title(),
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_all: 10,

                    #[name = "entry"]
                    gtk::Entry {
                        set_halign: gtk::Align::Fill,
                        set_margin_bottom: 10,

                        #[track = "model.changed(EntryDialog::<T>::entry_text())"]
                        #[block_signal(connect_text_notify_handler)]
                        set_text: model.get_entry_text(),

                        connect_text_notify[sender] => move |this| {
                            sender.input(EntryDialogMsg::TextChanged(this.text().into()));
                        } @connect_text_notify_handler,

                        connect_activate[sender] => move |_| {
                            sender.input(EntryDialogMsg::Apply);
                        },

                        #[track = "model.error_changed()"]
                        set_class_active[model.error.is_some()]: "error",

                        #[track = "model.error_changed()"]
                        set_secondary_icon_name: model.error.as_ref().map(|_| "issue-symbolic"),

                        #[track = "model.error_changed()"]
                        set_secondary_icon_tooltip_text: model.error.as_deref(),
                    },

                    #[name = "ok_button"]
                    gtk::Button {
                        set_label: &init.ok_button_label,
                        set_halign: gtk::Align::Fill,
                        set_css_classes: &["suggested-action"],

                        #[track = "model.changed(EntryDialog::<T>::value())"]
                        set_sensitive: model.value.is_some(),

                        connect_clicked[sender] => move |_| {
                            sender.input(EntryDialogMsg::Apply);
                        }
                    }
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = EntryDialog::<T> {
            dialog: root.clone(),
            title: "".to_string(),
            value: None,
            entry_text: "".to_string(),
            error: None,
            converter: None,
            tracker: 0,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();

        match message {
            EntryDialogMsg::Open {
                value,
                title,
                converter,
                parent,
            } => {
                self.set_title(title);
                self.set_value(Some(value));
                self.set_error(None);
                self.converter = Some(converter);
                self.set_entry_text(self.get_value_as_text());

                self.dialog.present(&parent);
            }
            EntryDialogMsg::TextChanged(text) => {
                if let Some(converter) = &self.converter {
                    match converter.parse(&text) {
                        Ok(value) => {
                            self.set_value(Some(value));
                            self.set_error(None);
                        }
                        Err(err) => {
                            self.set_value(None);
                            self.set_error(Some(err));
                        }
                    }
                }
            }
            EntryDialogMsg::Apply => {
                if let Some(value) = self.value.take() {
                    self.dialog.close();
                    let _ = sender.output(EntryDialogOutput { value });
                }
            }
        }
    }
}
