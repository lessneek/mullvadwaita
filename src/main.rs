mod extensions;
mod mullvad;
#[macro_use]
mod prelude;

#[macro_use]
extern crate tr;

use crate::extensions::TunnelStateExt;
use crate::mullvad::Event;
use crate::prelude::*;

use futures::FutureExt;
use smart_default::SmartDefault;

use relm4::{
    adw,
    component::Component,
    gtk::{
        prelude::{ButtonExt, GtkWindowExt},
        traits::{OrientableExt, WidgetExt},
        Align::*,
    },
    ComponentParts, ComponentSender, RelmApp, RelmWidgetExt,
};
use relm4_icons::icon_name;

use mullvad_types::states::TunnelState;

#[derive(Debug)]
enum AppInput {
    SwitchConnection,
    Reconnect,
}

#[derive(Debug)]
enum AppMsg {
    DaemonEvent(Event),
}

#[tracker::track]
#[derive(SmartDefault)]
struct AppModel {
    state_label: String,
    connection_button_label: String,
    connection_button_css: &'static [&'static str],
    is_daemon_connected: bool,
    is_tunnel_connecting_or_connected: bool,
}

#[derive(Debug, SmartDefault)]
enum DaemonState {
    Connected {
        tunnel_state: Box<TunnelState>,
    },
    #[default]
    Connecting,
}

impl AppModel {
    fn on_daemon_state_change(&mut self, daemon_state: &DaemonState) {
        use TunnelState::*;

        match daemon_state {
            DaemonState::Connected { tunnel_state } => {
                let tunnel_state = &**tunnel_state;

                self.set_is_daemon_connected(true);

                self.set_is_tunnel_connecting_or_connected(
                    tunnel_state.is_connecting_or_connected(),
                );

                self.set_connection_button_css(match tunnel_state {
                    Disconnected { .. } => &["opaque", "secure_connection_btn"],
                    Connected { .. } | Connecting { .. } => &["opaque", "disconnect_btn"],
                    _ => &[],
                });

                self.set_state_label(match tunnel_state {
                    Connected { endpoint, .. } => {
                        if endpoint.quantum_resistant {
                            tr!(
                                // Creating a secure connection that isn't breakable by quantum computers.
                                "QUANTUM SECURE CONNECTION"
                            )
                        } else {
                            tr!("SECURE CONNECTION")
                        }
                    }
                    Connecting { endpoint, .. } => {
                        if endpoint.quantum_resistant {
                            tr!("CREATING QUANTUM SECURE CONNECTION")
                        } else {
                            tr!("CREATING SECURE CONNECTION")
                        }
                    }
                    Disconnected { locked_down, .. } => {
                        if *locked_down {
                            tr!("BLOCKED CONNECTION")
                        } else {
                            tr!("UNSECURED CONNECTION")
                        }
                    }
                    _ => "".to_owned(),
                });

                self.set_connection_button_label(match tunnel_state {
                    Connected { .. } => tr!("Disconnect"),
                    Connecting { .. } => tr!("Cancel"),
                    Disconnected { .. } => tr!("Secure my connection"),
                    _ => "".to_owned(),
                });
            }
            DaemonState::Connecting => {
                self.set_is_daemon_connected(false);
                self.set_state_label(tr!("Connecting to Mullvad system service..."));
            }
        }
    }
}

#[relm4::component]
impl Component for AppModel {
    type Init = ();
    type Input = AppInput;
    type Output = ();
    type CommandOutput = AppMsg;

    view! {
        adw::Window {
            set_title: Some("Mullvad VPN"),
            set_default_size: (300, 600),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    gtk::Label {
                        #[track = "model.changed(AppModel::state_label())"]
                        set_label: &model.state_label,
                        set_margin_all: 5,
                        add_css_class: "title-4",
                        set_wrap: true,
                    },

                    gtk::Box {
                        #[track = "model.changed(AppModel::is_daemon_connected())"]
                        set_visible: model.is_daemon_connected,

                        add_css_class: "linked",
                        set_margin_all: 20,
                        set_halign: Center,
                        set_valign: End,
                        set_vexpand: true,
                        set_width_request: 300,

                        gtk::Button {
                            connect_clicked => AppInput::SwitchConnection,
                            set_hexpand: true,

                            #[track = "model.changed(AppModel::connection_button_label())"]
                            set_label: model.connection_button_label.as_str(),

                            #[track = "model.changed(AppModel::connection_button_css())"]
                            set_css_classes: model.connection_button_css,
                        },

                        gtk::Button {
                            connect_clicked => AppInput::Reconnect,

                            #[track = "model.changed(AppModel::is_tunnel_connecting_or_connected())"]
                            set_visible: model.is_tunnel_connecting_or_connected,

                            set_css_classes: &["opaque", "reconnect_btn"],
                            set_icon_name: icon_name::REFRESH_LARGE
                        },
                    }
                }
            }
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        sender.command(|out, shutdown| {
            shutdown
                // Performs this operation until a shutdown is triggered
                .register(async move {
                    let mut event_rx = mullvad::watch();

                    debug!("Listening for status updates...");

                    while let Some(event) = event_rx.recv().await {
                        debug!("Daemon event: {:?}", event);
                        out.send(AppMsg::DaemonEvent(event)).unwrap();
                    }

                    debug!("Status updates stopped.");
                })
                // Perform task until a shutdown interrupts it
                .drop_on_shutdown()
                // Wrap into a `Pin<Box<Future>>` for return
                .boxed()
        });

        let model = AppModel::default();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        self.reset();

        match message {
            AppInput::SwitchConnection => {}
            AppInput::Reconnect => {}
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            AppMsg::DaemonEvent(event) => {
                let daemon_state = match event {
                    Event::TunnelState(state) => DaemonState::Connected {
                        tunnel_state: state,
                    },
                    Event::ConnectingToDaemon => DaemonState::Connecting,
                };
                self.on_daemon_state_change(&daemon_state);
            }
        }
    }
}

fn init_logger() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();
}

fn init_gettext() {
    use i18n_embed::{gettext::gettext_language_loader, DesktopLanguageRequester};

    use rust_embed::RustEmbed;

    #[derive(RustEmbed)]
    #[folder = "i18n/mo"] // path to the compiled localization resources
    struct Translations;

    let _ = i18n_embed::select(
        &gettext_language_loader!(),
        &Translations {},
        &DesktopLanguageRequester::requested_languages(),
    );
}

#[tokio::main]
async fn main() {
    init_logger();

    debug!("mullvadwaita starting...");

    init_gettext();

    tokio::task::spawn_blocking(|| {
        let app = RelmApp::new("draft.mullvadwaita");
        relm4_icons::initialize_icons();
        app.set_global_css(GLOBAL_CSS);
        app.run::<AppModel>(());
    });
}

const GLOBAL_CSS: &str = r#"
.secure_connection_btn {
    color: white;
    background-color: @green_3;
}

.disconnect_btn {
    color: white;
    background-color: @red_3;
}

.reconnect_btn {
    color: white;
    background-color: @red_3;
}
"#;
