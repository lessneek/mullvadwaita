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

#[derive(Debug)]
pub struct RadioButton<V> {
    variant: V,
    check_button: gtk::CheckButton,
    active_notify_handler: SignalHandlerId,
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
    type Widgets = Vec<RadioButton<V>>;

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

        let mut widgets = vec![];

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
                    if let Some(RadioButton {
                        check_button,
                        active_notify_handler,
                        ..
                    }) = widgets
                        .iter()
                        .find(|RadioButton { variant: v, .. }| *v == variant)
                    {
                        check_button.block_signal(active_notify_handler);
                        check_button.set_active(true);
                        check_button.unblock_signal(active_notify_handler);
                    }
                } else {
                    for RadioButton { check_button, .. } in widgets.iter() {
                        check_button.set_active(false);
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
        widgets: &mut Vec<RadioButton<V>>,
        sender: ComponentSender<Self>,
    ) {
        root.remove_all();
        widgets.clear();

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
                                set_group: widgets.first().map(|r_btn| &r_btn.check_button),

                                connect_active_notify[sender, variant] => move |this| {
                                    if this.is_active() {
                                        sender.input(RadioButtonsListMsg::VariantSelected(variant));
                                    }
                                } @active_notify_handler,
                            },

                            connect_activated[check_button] => move |_| {
                                check_button.emit_activate();
                            },
                        }
                    }
                    widgets.push(RadioButton {
                        variant: *variant,
                        check_button,
                        active_notify_handler,
                    });

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
