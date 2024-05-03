use adw::prelude::*;
use gtk::glib::SignalHandlerId;
use relm4::prelude::*;
use std::fmt::Debug;

use super::types::{ViewElement, ViewType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadioButtonsList<V> {
    variants: Vec<V>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RadioButtonsListMsg<V> {
    VariantSelected(V),
    SelectVariant(Option<V>),
}

#[derive(Default, Debug)]
pub struct RadioButtonsListWidgets<V> {
    check_buttons: Vec<(V, gtk::CheckButton, SignalHandlerId)>,
}

impl<V> Component for RadioButtonsList<V>
where
    V: ViewElement + Debug + Clone + Copy + PartialEq + 'static,
{
    type CommandOutput = ();
    type Input = RadioButtonsListMsg<V>;
    type Output = V;
    type Init = Vec<V>;
    type Root = gtk::ListBox;
    type Widgets = RadioButtonsListWidgets<V>;

    fn init_root() -> Self::Root {
        gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .build()
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = RadioButtonsList { variants: init };

        let mut widgets = RadioButtonsListWidgets::<V> {
            check_buttons: vec![],
        };

        model.render(&root, &mut widgets, sender);

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        log::debug!("{message:#?}");

        match message {
            RadioButtonsListMsg::VariantSelected(variant) => {
                sender.output(variant).ok();
            }
            RadioButtonsListMsg::SelectVariant(variant) => {
                if let Some(variant) = variant {
                    if let Some((_, button, handler_id)) = widgets
                        .check_buttons
                        .iter()
                        .find(|(btn_variant, _, _)| *btn_variant == variant)
                    {
                        button.block_signal(handler_id);
                        button.set_active(true);
                        button.unblock_signal(handler_id);
                    }
                } else {
                    for (_, button, _) in widgets.check_buttons.iter() {
                        button.set_active(false);
                    }
                }
            }
        }
    }
}

impl<V> RadioButtonsList<V>
where
    V: ViewElement + Debug + Clone + Copy + PartialEq + 'static,
{
    fn render(
        &self,
        root: &gtk::ListBox,
        widgets: &mut RadioButtonsListWidgets<V>,
        sender: ComponentSender<Self>,
    ) {
        root.remove_all();
        widgets.check_buttons.clear();

        for variant in self.variants.iter() {
            match variant.get_view_type() {
                ViewType::Label(label) => {
                    relm4::view! {
                        #[name = "row"]
                        adw::ActionRow {
                            set_title: &label,
                            set_activatable: true,

                            #[name = "check_button"]
                            add_prefix = &gtk::CheckButton {
                                set_group: widgets.check_buttons.first().map(|(_, button, _)| button),

                                connect_active_notify[sender, variant] => move |this| {
                                    if this.is_active() {
                                        sender.input(RadioButtonsListMsg::VariantSelected(variant));
                                    }
                                } @handler_id,
                            },

                            connect_activated[check_button] => move |_| {
                                check_button.emit_activate();
                            },
                        }
                    }
                    widgets
                        .check_buttons
                        .push((*variant, check_button, handler_id));

                    root.append(&row);
                }
                ViewType::Entry(label, _) => {
                    let row = adw::EntryRow::builder().title(&label).build();
                    row.add_suffix(&gtk::Entry::default());
                    root.append(&row);
                    // TODO: implement.
                }
            };
        }
    }
}
