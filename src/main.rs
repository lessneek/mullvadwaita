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
use anyhow::Result;

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
use talpid_types::tunnel::ActionAfterDisconnect;

#[derive(Debug)]
enum AppInput {
    SecureMyConnection,
    CancelConnection,
    Disconnect,
    Reconnect,
}

#[derive(Debug)]
enum AppMsg {
    DaemonEvent(Event),
}

#[tracker::track]
#[derive(SmartDefault)]
struct AppModel {
    #[no_eq]
    daemon_state: DaemonState,
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
    fn is_daemon_connected(&self) -> bool {
        matches!(self.daemon_state, DaemonState::Connected { .. })
    }

    fn can_secure_connection(&self) -> bool {
        match &self.daemon_state {
            DaemonState::Connected { tunnel_state } => matches!(
                **tunnel_state,
                TunnelState::Disconnected { .. }
                    | TunnelState::Disconnecting(
                        ActionAfterDisconnect::Nothing | ActionAfterDisconnect::Block
                    )
            ),
            _ => false,
        }
    }

    fn can_cancel_connection(&self) -> bool {
        match &self.daemon_state {
            DaemonState::Connected { tunnel_state } => tunnel_state.is_connecting_or_reconnecting(),
            _ => false,
        }
    }

    fn can_disconnect(&self) -> bool {
        match &self.daemon_state {
            DaemonState::Connected { tunnel_state } => tunnel_state.is_connected(),
            _ => false,
        }
    }

    fn can_reconnect(&self) -> bool {
        match &self.daemon_state {
            DaemonState::Connected { tunnel_state } => tunnel_state.is_connecting_or_connected(),
            _ => false,
        }
    }

    fn get_state_label(&self) -> String {
        match &self.daemon_state {
            DaemonState::Connected { tunnel_state } => {
                match &**tunnel_state {
                    TunnelState::Connected { endpoint, .. } => {
                        if endpoint.quantum_resistant {
                            tr!(
                                // Creating a secure connection that isn't breakable by quantum computers.
                                "QUANTUM SECURE CONNECTION"
                            )
                        } else {
                            tr!("SECURE CONNECTION")
                        }
                    }
                    TunnelState::Connecting { endpoint, .. } => {
                        if endpoint.quantum_resistant {
                            tr!("CREATING QUANTUM SECURE CONNECTION")
                        } else {
                            tr!("CREATING SECURE CONNECTION")
                        }
                    }
                    TunnelState::Disconnected { locked_down, .. } => {
                        if *locked_down {
                            tr!("BLOCKED CONNECTION")
                        } else {
                            tr!("UNSECURED CONNECTION")
                        }
                    }
                    TunnelState::Disconnecting(
                        ActionAfterDisconnect::Nothing | ActionAfterDisconnect::Block,
                    ) => tr!("DISCONNECTING"),
                    TunnelState::Disconnecting(ActionAfterDisconnect::Reconnect) => {
                        tr!("CREATING SECURE CONNECTION")
                    }
                    TunnelState::Error(error_state) => {
                        format!("{}: {:?}", tr!("ERROR"), error_state)
                    }
                }
            }
            DaemonState::Connecting => {
                tr!("Connecting to Mullvad system service...")
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
                        #[track = "model.changed(AppModel::daemon_state())"]
                        set_label: &model.get_state_label(),
                        set_margin_all: 5,
                        add_css_class: "title-4",
                        set_wrap: true,
                    },

                    gtk::Box {
                        #[track = "model.changed(AppModel::daemon_state())"]
                        set_visible: model.is_daemon_connected(),

                        add_css_class: "linked",
                        set_margin_all: 20,
                        set_halign: Center,
                        set_valign: End,
                        set_vexpand: true,
                        set_width_request: 300,

                        gtk::Button {
                            connect_clicked => AppInput::SecureMyConnection,
                            set_hexpand: true,
                            set_label: &tr!("Secure my connection"),
                            set_css_classes: &["opaque", "secure_my_connection_btn"],

                            #[track = "model.changed(AppModel::daemon_state())"]
                            set_visible: model.can_secure_connection()
                        },

                        gtk::Button {
                            connect_clicked => AppInput::CancelConnection,
                            set_hexpand: true,
                            set_label: &tr!("Cancel"),
                            set_css_classes: &["opaque", "disconnect_btn"],

                            #[track = "model.changed(AppModel::daemon_state())"]
                            set_visible: model.can_cancel_connection()
                        },

                        gtk::Button {
                            connect_clicked => AppInput::Disconnect,
                            set_hexpand: true,
                            set_label: &tr!("Disconnect"),
                            set_css_classes: &["opaque", "disconnect_btn"],

                            #[track = "model.changed(AppModel::daemon_state())"]
                            set_visible: model.can_disconnect()
                        },

                        gtk::Button {
                            connect_clicked => AppInput::Reconnect,
                            set_css_classes: &["opaque", "reconnect_btn"],
                            set_icon_name: icon_name::REFRESH_LARGE,

                            #[track = "model.changed(AppModel::daemon_state())"]
                            set_visible: model.can_reconnect(),
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

                    trace!("Listening for status updates...");

                    while let Some(event) = event_rx.recv().await {
                        debug!("Daemon event: {:#?}", event);
                        out.send(AppMsg::DaemonEvent(event)).unwrap();
                    }

                    trace!("Status updates stopped.");
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
            AppInput::SecureMyConnection => {}
            AppInput::Reconnect => {}
            AppInput::CancelConnection => {}
            AppInput::Disconnect => {}
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
                self.set_daemon_state(daemon_state);
            }
        }
    }
}

fn init_logger() -> Result<(), log::SetLoggerError> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Error)
        .with_module_level("mullvadwaita", log::LevelFilter::Debug)
        .env()
        .with_colors(true)
        .init()
}

fn init_gettext(
) -> Result<Vec<i18n_embed::unic_langid::LanguageIdentifier>, i18n_embed::I18nEmbedError> {
    use i18n_embed::{gettext::gettext_language_loader, DesktopLanguageRequester};

    use rust_embed::RustEmbed;

    #[derive(RustEmbed)]
    #[folder = "i18n/mo"] // path to the compiled localization resources
    struct Translations;

    i18n_embed::select(
        &gettext_language_loader!(),
        &Translations {},
        &DesktopLanguageRequester::requested_languages(),
    )
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger()?;

    debug!("mullvadwaita starting...");

    init_gettext()?;

    tokio::task::spawn_blocking(|| {
        let app = RelmApp::new("draft.mullvadwaita");
        relm4_icons::initialize_icons();
        app.set_global_css(include_str!("./res/global.css"));
        app.run::<AppModel>(());
    });

    Ok(())
}
