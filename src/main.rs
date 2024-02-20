mod mullvad;

use log::debug;

use mullvad_types::states::TunnelState;
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

use futures::FutureExt;

use mullvad::Event;
use smart_default::SmartDefault;

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
    connection_button_label: &'static str,
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
    fn on_daemon_state_update(&mut self, daemon_state: &DaemonState) {
        match &daemon_state {
            DaemonState::Connected { tunnel_state } => {
                self.set_is_daemon_connected(true);
                match &**tunnel_state {
                    TunnelState::Connected { endpoint, .. } => {
                        self.set_state_label(format!("Connected: {}", endpoint));
                        self.set_is_tunnel_connecting_or_connected(true);
                        self.set_connection_button_label("Disconnect");
                    }
                    TunnelState::Connecting { endpoint, .. } => {
                        self.set_state_label(format!("Connecting: {}", endpoint));
                        self.set_is_tunnel_connecting_or_connected(true);
                        self.set_connection_button_label("Cancel");
                    }
                    TunnelState::Disconnected { .. } => {
                        self.set_state_label("Disconnected".to_string());
                        self.set_is_tunnel_connecting_or_connected(false);
                        self.set_connection_button_label("Secure my connection");
                    }
                    TunnelState::Disconnecting(_) => {
                        self.set_state_label("Disconnecting...".to_string());
                    }
                    TunnelState::Error(state) => {
                        self.set_state_label(format!("Error: {:?}", state));
                    }
                }
            }
            DaemonState::Connecting => {
                self.set_is_daemon_connected(false);
                self.set_state_label("Connecting to daemon...".to_string());
            }
        }

        self.set_connection_button_css(
            if let DaemonState::Connected { tunnel_state } = &daemon_state {
                match &**tunnel_state {
                    TunnelState::Disconnected { .. } => &["suggested-action"],
                    TunnelState::Connected { .. } | TunnelState::Connecting { .. } => {
                        &["destructive-action"]
                    }
                    _ => &[],
                }
            } else {
                &[]
            },
        );
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
            set_title: Some("Mullvadwaita"),
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
                        add_css_class: "title-1"
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
                            set_label: model.connection_button_label,

                            #[track = "model.changed(AppModel::connection_button_css())"]
                            set_css_classes: model.connection_button_css,
                        },

                        gtk::Button {
                            connect_clicked => AppInput::Reconnect,

                            #[track = "model.changed(AppModel::is_tunnel_connecting_or_connected())"]
                            set_visible: model.is_tunnel_connecting_or_connected,

                            set_css_classes: &["suggested-action"],
                            set_icon_name: icon_name::REFRESH_LARGE
                        },
                    }
                }
            }
        }
    }

    fn init(
        _: Self::Init,
        root: &Self::Root,
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
            AppInput::SwitchConnection => todo!(),
            AppInput::Reconnect => todo!(),
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
                self.on_daemon_state_update(&daemon_state);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new().init().unwrap();

    debug!("mullvadwaita starting...");

    tokio::task::spawn_blocking(|| {
        let app = RelmApp::new("draft.mullvadwaita");
        relm4_icons::initialize_icons();
        app.run::<AppModel>(());
    });
}
