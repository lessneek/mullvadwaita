use adw::prelude::*;
use core::fmt;
use gtk::glib::SignalHandlerId;
use relm4::prelude::*;
use std::fmt::Debug;
use std::str::FromStr;
use std::{collections::BTreeMap, sync::Arc};

use crate::{if_let_map, tr};

use super::entry_dialog::{EntryDialog, EntryDialogInit, EntryDialogMsg, EntryDialogOutput};

pub trait Unique {
    type Id: Debug + Copy + Ord;

    fn get_id(&self) -> Self::Id;
}

pub trait VariantValue: Unique + Sized + Send + Clone + Debug + PartialEq + 'static {}

pub fn label_variant<T: VariantValue>(label: String, value: T) -> Variant<T> {
    Variant::Label(LabelVariant::<T> { label, value })
}

pub fn entry_variant<T: VariantValue + FromStr>(
    title: String,
    value: T,
    converter: EntryConverter<T, String>,
) -> Variant<T> {
    Variant::Entry(EntryVariant::<T>::new(title, value, converter))
}

type ParseFn<T, Err> = Box<dyn Fn(&str) -> Result<T, Err> + Send + Sync + 'static>;
type ToStringFn<T> = Box<dyn Fn(&T) -> Option<String> + Send + Sync + 'static>;

#[derive(Clone)]
pub struct EntryConverter<T, Err> {
    parse: Arc<ParseFn<T, Err>>,
    to_string: Arc<ToStringFn<T>>,
}

impl<T, Err> EntryConverter<T, Err> {
    pub fn new(parse: ParseFn<T, Err>, to_string: ToStringFn<T>) -> Self {
        EntryConverter::<T, Err> {
            parse: Arc::new(parse),
            to_string: Arc::new(to_string),
        }
    }

    pub fn parse(&self, s: &str) -> Result<T, Err> {
        (self.parse)(s)
    }

    pub fn to_string(&self, value: &T) -> Option<String> {
        (self.to_string)(value)
    }
}

impl<T, Err> fmt::Debug for EntryConverter<T, Err> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EntryTexter").finish()
    }
}

#[derive(Debug)]
pub enum Variant<T: VariantValue> {
    Label(LabelVariant<T>),
    Entry(EntryVariant<T>),
}

impl<T: VariantValue> Variant<T> {
    pub fn get_id(&self) -> T::Id {
        match self {
            Variant::Label(variant) => variant.get_id(),
            Variant::Entry(variant) => variant.get_id(),
        }
    }

    pub fn get_value(&self) -> &T {
        match self {
            Variant::Label(variant) => variant.get_value(),
            Variant::Entry(variant) => variant.get_value(),
        }
    }

    pub fn as_entry_variant(&self) -> Option<&EntryVariant<T>> {
        if_let_map!(self to Variant::<T>::Entry(variant) => variant)
    }

    pub fn as_entry_variant_mut(&mut self) -> Option<&mut EntryVariant<T>> {
        if_let_map!(self to Variant::<T>::Entry(variant) => variant)
    }
}

#[derive(Debug)]
pub struct LabelVariant<T: VariantValue> {
    label: String,
    value: T,
}

impl<T: VariantValue> LabelVariant<T> {
    pub fn get_id(&self) -> T::Id {
        self.value.get_id()
    }

    pub fn get_label(&self) -> &str {
        &self.label
    }

    pub fn get_value(&self) -> &T {
        &self.value
    }
}

#[derive(Debug)]
pub struct EntryVariant<T: VariantValue> {
    title: String,
    value: T,
    value_as_string: String,
    converter: EntryConverter<T, String>,
}

impl<T: VariantValue> EntryVariant<T> {
    pub fn new(title: String, value: T, converter: EntryConverter<T, String>) -> Self {
        let value_as_string = converter.to_string(&value).unwrap_or_default();
        Self {
            title,
            value,
            value_as_string,
            converter,
        }
    }

    pub fn get_id(&self) -> T::Id {
        self.value.get_id()
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn try_set_value_from_text(&mut self, text: &str) -> Result<(), String> {
        self.set_value(self.converter.parse(text)?);
        Ok(())
    }

    pub fn get_value(&self) -> &T {
        &self.value
    }

    pub fn set_value(&mut self, new_value: T) {
        if new_value.get_id() == self.get_id() {
            self.value = new_value;
            self.value_as_string = self.converter.to_string(&self.value).unwrap_or_default();
        }
    }

    pub fn get_value_as_str(&self) -> &str {
        &self.value_as_string
    }
}

#[derive(Debug)]
pub struct VariantSelector<T: VariantValue> {
    variant_rows: BTreeMap<T::Id, VariantRow<T>>,
    entry_dialog: Controller<EntryDialog<T>>,
}

#[derive(Debug)]
pub enum VariantSelectorMsg<T: VariantValue> {
    VariantSelected(T::Id),
    SelectVariant(Option<T>),
    OpenEntryDialog(T::Id),
    EntryDialogOutput(EntryDialogOutput<T>),
}

#[derive(Debug)]
pub enum VariantRow<T: VariantValue> {
    Label(LabelVariantRow<T>),
    Entry(EntryVariantRow<T>),
}

#[derive(Debug)]
pub struct LabelVariantRow<T: VariantValue> {
    variant: LabelVariant<T>,
    check_button: gtk::CheckButton,
    check_button_active_notify_handler: SignalHandlerId,
}

impl<T: VariantValue> LabelVariantRow<T> {
    fn set_active(&mut self, value: bool) {
        self.check_button
            .block_signal(&self.check_button_active_notify_handler);
        self.check_button.set_active(value);
        self.check_button
            .unblock_signal(&self.check_button_active_notify_handler);
    }
}

#[derive(Debug)]
pub struct EntryVariantRow<T: VariantValue> {
    variant: EntryVariant<T>,
    check_button: gtk::CheckButton,
    check_button_active_notify_handler: SignalHandlerId,
    action_row: adw::ActionRow,
}

impl<T: VariantValue> EntryVariantRow<T> {
    fn set_active(&mut self, value: bool) {
        self.check_button
            .block_signal(&self.check_button_active_notify_handler);
        self.check_button.set_active(value);
        self.check_button
            .unblock_signal(&self.check_button_active_notify_handler);
    }

    fn set_value(&mut self, new_value: T) {
        self.variant.set_value(new_value);
        let value_as_str = self.variant.get_value_as_str();
        self.action_row.set_subtitle(value_as_str);
    }
}

impl<T: VariantValue> VariantRow<T> {
    fn set_active(&mut self, value: bool) {
        match self {
            VariantRow::Label(row) => row.set_active(value),
            VariantRow::Entry(row) => row.set_active(value),
        }
    }

    fn get_value(&self) -> &T {
        match self {
            VariantRow::Label(row) => row.variant.get_value(),
            VariantRow::Entry(row) => row.variant.get_value(),
        }
    }
}

impl<T> Component for VariantSelector<T>
where
    T: VariantValue,
{
    type CommandOutput = ();
    type Input = VariantSelectorMsg<T>;
    type Output = T;
    type Init = Vec<Variant<T>>;
    type Root = gtk::ListBox;
    type Widgets = ();

    fn init_root() -> Self::Root {
        gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .build()
    }

    fn init(
        variants: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = VariantSelector::<T> {
            variant_rows: Default::default(),
            entry_dialog: EntryDialog::builder()
                .launch(EntryDialogInit {
                    ok_button_label: tr!("OK"),
                })
                .forward(sender.input_sender(), VariantSelectorMsg::EntryDialogOutput),
        };

        model.render(&root, variants, sender);

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        log::debug!("{message:#?}");

        match message {
            VariantSelectorMsg::VariantSelected(id) => {
                if let Some(row) = self.variant_rows.get_mut(&id) {
                    sender.output(row.get_value().clone()).ok();
                }
            }
            VariantSelectorMsg::SelectVariant(new_value) => {
                if let Some(new_value) = new_value {
                    let id = new_value.get_id();
                    if let Some(row) = self.variant_rows.get_mut(&id) {
                        if let VariantRow::Entry(row) = row {
                            row.variant.set_value(new_value.clone());
                            row.action_row.set_subtitle(row.variant.get_value_as_str());
                        }

                        row.set_active(true);
                    }
                } else {
                    for row in self.variant_rows.values_mut() {
                        row.set_active(false);
                    }
                }
            }
            VariantSelectorMsg::OpenEntryDialog(id) => {
                if let Some(row) = self.get_entry_row(&id) {
                    self.entry_dialog.emit(EntryDialogMsg::Open {
                        title: tr!("Edit value"),
                        value: row.variant.get_value().clone(),
                        converter: row.variant.converter.clone(),
                        parent: root.widget_ref().clone(),
                    })
                }
            }
            VariantSelectorMsg::EntryDialogOutput(output) => {
                let id = output.value.get_id();
                if let Some(row) = self.get_entry_row_mut(&output.value.get_id()) {
                    row.set_value(output.value);
                    if row.check_button.is_active() {
                        sender.input(VariantSelectorMsg::VariantSelected(id));
                    }
                }
            }
        }
    }
}

impl<T: VariantValue> VariantSelector<T> {
    fn get_entry_row(&self, id: &T::Id) -> Option<&EntryVariantRow<T>> {
        self.variant_rows
            .get(id)
            .and_then(|row| if_let_map!(row to VariantRow::Entry(row) => row))
    }

    fn get_entry_row_mut(&mut self, id: &T::Id) -> Option<&mut EntryVariantRow<T>> {
        self.variant_rows
            .get_mut(id)
            .and_then(|row| if_let_map!(row to VariantRow::Entry(row) => row))
    }

    fn render(
        &mut self,
        root: &gtk::ListBox,
        variants: Vec<Variant<T>>,
        sender: ComponentSender<Self>,
    ) {
        root.remove_all();
        self.variant_rows.clear();

        let mut group_check_button: Option<gtk::CheckButton> = None;

        for variant in variants.into_iter() {
            let id = variant.get_id();
            match variant {
                Variant::Label(variant) => {
                    relm4::view! {
                        #[name = "action_row"]
                        adw::ActionRow {
                            set_title: variant.get_label(),
                            set_activatable: true,

                            #[name = "check_button"]
                            add_prefix = &gtk::CheckButton {
                                set_group: group_check_button.as_ref(),

                                connect_active_notify[sender, id] => move |this| {
                                    if this.is_active() {
                                        sender.input(VariantSelectorMsg::VariantSelected(id));
                                    }
                                } @check_button_active_notify_handler,
                            },

                            connect_activated[check_button] => move |_| {
                                check_button.emit_activate();
                            },
                        }
                    }
                    group_check_button.get_or_insert(check_button.clone());

                    self.variant_rows.insert(
                        id,
                        VariantRow::Label(LabelVariantRow {
                            variant,
                            check_button,
                            check_button_active_notify_handler,
                        }),
                    );

                    root.append(&action_row);
                }
                Variant::Entry(variant) => {
                    relm4::view! {
                        #[name = "action_row"]
                        adw::ActionRow {
                            set_title: variant.get_title(),
                            set_subtitle: variant.get_value_as_str(),
                            set_activatable: true,
                            add_css_class: "property",

                            #[name = "check_button"]
                            add_prefix = &gtk::CheckButton {
                                set_group: group_check_button.as_ref(),

                                connect_active_notify[sender, id] => move |this| {
                                    if this.is_active() {
                                        sender.input(VariantSelectorMsg::VariantSelected(id));
                                    }
                                } @check_button_active_notify_handler,
                            },

                            connect_activated[check_button] => move |_| {
                                check_button.emit_activate();
                            },

                            add_suffix = &gtk::Button {
                                set_icon_name: "edit-symbolic",
                                set_valign: gtk::Align::Center,
                                set_css_classes: &["flat"],

                                connect_clicked[sender, id] => move |_| {
                                    sender.input(VariantSelectorMsg::OpenEntryDialog(id));
                                }
                            }
                        }
                    }
                    group_check_button.get_or_insert(check_button.clone());

                    self.variant_rows.insert(
                        id,
                        VariantRow::Entry(EntryVariantRow {
                            variant,
                            check_button,
                            check_button_active_notify_handler,
                            action_row: action_row.clone(),
                        }),
                    );

                    root.append(&action_row);
                }
            };
        }
    }
}
