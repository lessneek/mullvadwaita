use gtk::prelude::{ButtonExt, GtkWindowExt};
use relm4::gtk::traits::{OrientableExt, WidgetExt};
use relm4::{adw, gtk, ComponentParts, ComponentSender, RelmApp, RelmWidgetExt, SimpleComponent};

use relm4_icons::icon_name;

use gtk::Align::*;
use ConnectionState::*;

#[derive(Debug)]
enum AppInput {
    SwitchConnection,
    Reconnect,
}

#[derive(PartialEq, Debug)]
enum ConnectionState {
    Connected,
    Disconnected,
    Connecting,
}

#[tracker::track]
struct AppModel {
    state: ConnectionState,
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Input = AppInput;
    type Output = ();
    type Init = ConnectionState;

    view! {
        adw::Window {
            set_title: Some("Mullvadwaita"),
            set_default_width: 300,
            set_default_height: 600,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    gtk::Label {
                        #[track = "model.changed(AppModel::state())"]
                        set_label: &format!("State: {:?}", model.state),
                        set_margin_all: 5,
                    },

                    gtk::Box {
                        add_css_class: "linked",
                        set_margin_all: 20,
                        set_halign: Center,
                        set_valign: End,
                        set_vexpand: true,
                        set_width_request: 300,

                        gtk::Button {
                            set_hexpand: true,

                            #[track = "model.changed(AppModel::state())"]
                            set_label: {
                                match model.state {
                                    Disconnected => "Secure my connection",
                                    Connected => "Disconnect",
                                    Connecting => "Cancel"
                                }
                            },

                            #[track = "model.changed(AppModel::state())"]
                            set_css_classes: {
                                match model.state {
                                    Disconnected => &["suggested-action"],
                                    Connected | Connecting => &["destructive-action"],
                                }
                            },

                            connect_clicked => AppInput::SwitchConnection
                        },

                        gtk::Button {
                            #[track = "model.changed(AppModel::state())"]
                            set_visible: model.state == Connected,
                            connect_clicked => AppInput::Reconnect,
                            set_css_classes: &["suggested-action"],
                            set_icon_name: icon_name::REFRESH_LARGE
                        },
                    }
                }
            }
        }
    }

    fn init(
        state: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AppModel { state, tracker: 0 };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        self.reset();

        self.set_state(match message {
            AppInput::SwitchConnection => match self.get_state() {
                Connected | Connecting => Disconnected,
                Disconnected => Connected,
            },
            AppInput::Reconnect => ConnectionState::Connecting,
        })
    }
}

fn main() {
    let app = RelmApp::new("relm4.test.simple_manual");
    relm4_icons::initialize_icons();
    app.run::<AppModel>(ConnectionState::Disconnected);
}
