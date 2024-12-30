use crate::icon_names;
use crate::tr;

use adw::prelude::*;
use relm4::prelude::*;

#[relm4::widget_template(pub)]
impl WidgetTemplate for LoginView {
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: 20,
            set_valign: gtk::Align::Center,

            // Shown when `lockdown mode` is enabled.
            #[name = "disable_lockdown_mode_bin"]
            adw::Bin {
                set_css_classes: &["card"],
                set_margin_bottom: 20,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_all: 20,
                    set_halign: gtk::Align::Fill,
                    set_spacing: 12,

                    gtk::Label {
                        set_label: &tr!("Blocking internet"),
                        set_css_classes: &["title-4"],
                        set_halign: gtk::Align::Start,
                    },

                    gtk::Label {
                        set_label: &tr!("<b>Lockdown mode</b> is enabled. Disable it to unblock your connection."),
                        set_use_markup: true,
                        set_wrap: true,
                        set_halign: gtk::Align::Start,
                    },

                    #[name = "disable_lockdown_mode_button"]
                    gtk::Button {
                        set_label: &tr!("Disable"),
                        set_css_classes: &["opaque", "disable_lockdown_mode_btn"],
                    }
                }
            },

            gtk::Label {
                set_label: &tr!("Login"),
                set_margin_bottom: 20,
                add_css_class: "title-1",
                set_halign: gtk::Align::Start,
            },

            gtk::ListBox {
                add_css_class: "boxed-list",
                set_selection_mode: gtk::SelectionMode::None,
                set_margin_bottom: 20,

                #[name = "account_number"]
                append = &adw::EntryRow {
                    set_title: &tr!("Enter your account number"),

                    #[name = "login_button_stack"]
                    add_suffix = &gtk::Stack {
                        add_named[Some("logging_in")] = &gtk::Spinner {
                            set_spinning: true,
                        },

                        #[name = "login_button"]
                        add_named[Some("default")] = &gtk::Button {
                            set_icon_name: icon_names::ARROW2_RIGHT,
                            set_valign: gtk::Align::Center,
                            set_css_classes: &["opaque", "login_btn"],
                            set_receives_default: true,
                        },
                    },
                },

                #[name = "account_history_row"]
                adw::ActionRow {
                    set_activatable: true,

                    connect_activated[account_number, login_button] => move |this| {
                        account_number.set_text(this.title().as_ref());
                        login_button.emit_clicked();
                    },

                    #[name = "clear_account_history_button"]
                    add_suffix = &gtk::Button {
                        set_icon_name: icon_names::CROSS_LARGE_CIRCLE_FILLED,
                        set_valign: gtk::Align::Center,
                        set_css_classes: &["flat"],
                    },
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::Label {
                    set_text: &tr!("Don’t have an account number?"),
                    set_halign: gtk::Align::Start,
                    set_css_classes: &["caption-heading"],
                    set_margin_bottom: 10,
                },

                #[name = "create_account_button"]
                gtk::Button {
                    set_label: &tr!("Create account"),
                },
            },
        }
    }
}
