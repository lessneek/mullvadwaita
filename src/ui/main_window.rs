use crate::{icon_names, tr, ui::logged_in_view::LoggedInView};

use adw::prelude::*;
use gtk::StackTransitionType;
use relm4::prelude::*;

use super::login_view::LoginView;

#[relm4::widget_template(pub)]
impl WidgetTemplate for MainWindow {
    view! {
        #[name = "main_window"]
        adw::Window {
            set_default_size: (300, 600),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar {
                    add_css_class: "flat",

                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: "Mullvadwaita",
                        set_subtitle: "for Mullvad VPN",
                    },

                    #[name = "primary_menu_button"]
                    pack_end = &gtk::MenuButton {
                        set_icon_name: icon_names::MENU_LARGE,
                    },
                },

                #[name = "banner"]
                adw::Banner {},

                adw::Clamp {
                    set_maximum_size: 600,

                    #[name = "view_stack"]
                    gtk::Stack {
                        set_transition_type: StackTransitionType::SlideLeftRight,

                        #[template]
                        #[name = "logged_in_view"]
                        add_named[Some("logged_in")] = &LoggedInView {},

                        #[template]
                        #[name = "login_view"]
                        add_named[Some("login")] = &LoginView {},

                        add_named[Some("connecting_to_daemon")] = &gtk::Label {
                            set_label: &tr!("Connecting to Mullvad system service..."),
                            set_margin_all: 5,
                            add_css_class: "title-4",
                            set_wrap: true
                        }
                    }
                }
            }
        }
    }
}

impl AsRef<gtk::Window> for MainWindow {
    fn as_ref(&self) -> &gtk::Window {
        self.main_window.as_ref()
    }
}
