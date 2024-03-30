use std::convert::identity;

use crate::extensions::TunnelStateExt;
use crate::mullvad::{self, DaemonConnector, Event};
use crate::prelude::*;
use crate::ui::preferences::{PreferencesModel, PreferencesMsg};

use chrono::prelude::*;
use futures::FutureExt;
use smart_default::SmartDefault;

use relm4::actions::{RelmAction, RelmActionGroup};
use relm4::prelude::*;
use relm4_icons::icon_names;

use adw::prelude::*;

use mullvad_types::account::AccountData;
use mullvad_types::device::AccountAndDevice;
use mullvad_types::states::TunnelState;
use talpid_types::tunnel::ActionAfterDisconnect;

#[derive(Debug)]
pub enum AppInput {
    SecureMyConnection,
    CancelConnection,
    Disconnect,
    Reconnect,
    Preferences,
    About,
}

#[derive(Debug)]
pub enum AppMsg {
    DaemonEvent(Event),
}

#[tracker::track]
#[derive(SmartDefault)]
pub struct AppModel {
    #[no_eq]
    daemon_state: DaemonState,
    #[no_eq]
    account_and_device: Option<AccountAndDevice>,
    #[no_eq]
    account_data: Option<AccountData>,
    #[do_not_track]
    daemon_connector: DaemonConnector,
    device_name: Option<String>,
    time_left: Option<String>,
    banner_label: Option<String>,
    tunnel_state_label: Option<String>,
    country: Option<String>,
    city: Option<String>,
    hostname: Option<String>,
    tunnel_protocol: Option<String>,
    tunnel_in: Option<String>,
    tunnel_out: Option<String>,

    #[no_eq]
    components: Option<AppComponents>,
}

pub struct AppComponents {
    preferences: AsyncController<PreferencesModel>,
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
    fn get_tunnel_state(&self) -> Option<&TunnelState> {
        match &self.daemon_state {
            DaemonState::Connected { tunnel_state } => Some(tunnel_state),
            _ => None,
        }
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

    fn is_connected(&self) -> bool {
        self.get_tunnel_state()
            .map_or(false, |ts| ts.is_connected())
    }

    fn is_connecting_or_reconnecting(&self) -> bool {
        self.get_tunnel_state()
            .map(|ts| ts.is_connecting_or_reconnecting() || ts.is_in_error_state())
            .unwrap_or(false)
    }

    fn can_disconnect(&self) -> bool {
        self.get_tunnel_state()
            .map(|ts| ts.is_connected())
            .unwrap_or(false)
    }

    fn can_reconnect(&self) -> bool {
        self.get_tunnel_state()
            .map(|ts| ts.is_connecting_or_connected())
            .unwrap_or(false)
    }

    fn state_changed(&self) -> bool {
        self.changed(AppModel::daemon_state())
    }

    fn update_properties(&mut self) {
        self.set_banner_label(
            self.get_tunnel_state()
                .and_then(|tunnel_state| -> Option<String> {
                    match tunnel_state {
                        TunnelState::Error(error_state) => Some(format!("{}", error_state.cause())),
                        _ => None,
                    }
                }),
        );

        self.set_tunnel_state_label(
            self.get_tunnel_state()
                .map(|ts| ts.get_tunnel_state_label()),
        );

        self.set_country(self.get_tunnel_state().and_then(|ts| ts.get_country()));
        self.set_city(self.get_tunnel_state().and_then(|ts| ts.get_city()));
        self.set_hostname(self.get_tunnel_state().and_then(|ts| ts.get_hostname()));

        self.set_tunnel_protocol(
            self.get_tunnel_state()
                .and_then(|ts| ts.get_tunnel_protocol()),
        );

        self.set_tunnel_in(self.get_tunnel_state().and_then(|ts| ts.get_tunnel_in()));
        self.set_tunnel_out(self.get_tunnel_state().and_then(|ts| ts.get_tunnel_out()));

        self.set_device_name(
            self.get_account_and_device()
                .as_ref()
                .map(|acc| tr!("<b>Device name</b>: {}", acc.device.pretty_name())),
        );

        self.set_time_left(self.get_account_data().as_ref().map(|data| {
            let now = Utc::now();
            if now >= data.expiry {
                tr!("<b>Expired</b>")
            } else {
                let left = data.expiry - now;
                tr!("<b>Time left</b>: 1 day" | "<b>Time left</b>: {n} days" % left.num_days())
                    .to_string()
            }
        }));
    }
}

#[relm4::component(async, pub)]
impl AsyncComponent for AppModel {
    type Init = ();
    type Input = AppInput;
    type Output = ();
    type CommandOutput = AppMsg;

    view! {
        #[name = "main_window"]
        adw::Window {
            set_title: Some("Mullvad VPN"),
            set_default_size: (300, 600),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar {
                    add_css_class: "flat",

                    pack_end = &gtk::MenuButton {
                        set_icon_name: "open-menu-symbolic",
                        set_menu_model: Some(&primary_menu),
                    },
                },

                adw::Banner {
                    #[track = "model.changed(AppModel::banner_label())"]
                    set_title: model.get_banner_label().to_str(),

                    #[track = "model.changed(AppModel::banner_label())"]
                    set_revealed: model.get_banner_label().is_some()
                },

                adw::Clamp {
                    set_maximum_size: 600,

                    #[transition(SlideUpDown)]
                    match &model.daemon_state {
                        DaemonState::Connected { tunnel_state } => {
                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_valign: gtk::Align::Fill,
                                set_margin_all: 20,

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Horizontal,
                                    set_halign: gtk::Align::Fill,

                                    gtk::Label {
                                        #[track = "model.changed(AppModel::device_name())"]
                                        set_label: model.get_device_name().to_str(),
                                        set_css_classes: &["caption"],
                                        set_selectable: true,
                                        set_use_markup: true,
                                        set_hexpand: true,
                                        set_margin_end: 10,
                                        set_halign: gtk::Align::Start,
                                    },
                                    gtk::Label {
                                        #[track = "model.changed(AppModel::time_left())"]
                                        set_label: model.get_time_left().to_str(),
                                        set_css_classes: &["caption"],
                                        set_selectable: true,
                                        set_use_markup: true,
                                        set_halign: gtk::Align::End,
                                    }
                                },

                                adw::Bin {
                                    set_height_request: 128,
                                    set_width_request: 128,
                                    set_margin_all: 16,
                                    set_halign: gtk::Align::Center,

                                    match &**tunnel_state {
                                        TunnelState::Connected { .. } => {
                                            gtk::Image {
                                                set_icon_name: Some("network-vpn"),
                                                set_css_classes: &[
                                                    "connection_state_icon",
                                                    "connected",
                                                    "icon-dropshadow"
                                                ]
                                            }
                                        },
                                        TunnelState::Connecting { .. } => {
                                            gtk::Spinner {
                                                set_spinning: true,
                                                set_height_request: 64,
                                                set_width_request: 64,
                                                set_halign: gtk::Align::Center,
                                                set_valign: gtk::Align::Center,
                                            }
                                        },
                                        TunnelState::Disconnected { locked_down: true, .. } => {
                                            gtk::Image {
                                                set_icon_name: Some("network-vpn-disabled"),
                                                set_css_classes: &[
                                                    "connection_state_icon",
                                                    "disabled",
                                                    "icon-dropshadow"
                                                ]
                                            }
                                        },
                                        TunnelState::Disconnected { locked_down: false, .. } => {
                                            gtk::Image {
                                                set_icon_name: Some("network-vpn-disconnected"),
                                                set_css_classes: &[
                                                    "connection_state_icon",
                                                    "disconnected",
                                                    "icon-dropshadow"
                                                ]
                                            }
                                        },
                                        _ => {
                                            gtk::Label {}
                                        }
                                    }
                                },

                                gtk::Label {
                                    #[track = "model.changed(AppModel::tunnel_state_label())"]
                                    set_label: model.get_tunnel_state_label().to_str(),
                                    set_margin_bottom: 10,

                                    #[track = "model.state_changed()"]
                                    set_css_classes: if model.is_connected() {
                                        &["title-4", "connected_state_label"]
                                    } else {
                                        &["title-4"]
                                    },

                                    set_wrap: true,
                                    set_halign: gtk::Align::Start
                                },

                                gtk::Label {
                                    #[track = "model.changed(AppModel::country())"]
                                    set_label: model.get_country().to_str(),
                                    set_margin_bottom: 0,
                                    add_css_class: "title-1",
                                    set_wrap: true,
                                    set_halign: gtk::Align::Start,
                                },

                                gtk::Label {
                                    #[track = "model.changed(AppModel::city())"]
                                    set_label: model.get_city().to_str(),
                                    set_margin_bottom: 20,
                                    add_css_class: "title-1",
                                    set_wrap: true,
                                    set_halign: gtk::Align::Start
                                },

                                gtk::ListBox {
                                    add_css_class: "boxed-list",
                                    set_selection_mode: gtk::SelectionMode::None,
                                    set_margin_bottom: 20,

                                    #[track = "model.changed(AppModel::hostname())"]
                                    set_visible: model.get_hostname().is_some(),

                                    adw::ExpanderRow {
                                        #[track = "model.changed(AppModel::hostname())"]
                                        set_title: model.get_hostname().to_str(),

                                        add_row = &adw::ActionRow {
                                            set_title: "Tunnel protocol",
                                            set_css_classes: &["property", "monospace"],

                                            #[track = "model.changed(AppModel::tunnel_protocol())"]
                                            set_subtitle: model.get_tunnel_protocol().to_str(),
                                        },

                                        add_row = &adw::ActionRow {
                                            set_title: "In",
                                            set_css_classes: &["property", "monospace"],
                                            set_subtitle_selectable: true,

                                            #[track = "model.changed(AppModel::tunnel_in())"]
                                            set_subtitle: model.get_tunnel_in().to_str(),
                                        },

                                        add_row = &adw::ActionRow {
                                            set_title: "Out",
                                            set_css_classes: &["property", "monospace"],
                                            set_subtitle_selectable: true,

                                            #[track = "model.changed(AppModel::tunnel_out())"]
                                            set_subtitle: model.get_tunnel_out().to_str(),
                                        },
                                    },
                                },

                                // Connection buttons box.
                                gtk::Box {
                                    add_css_class: "linked",
                                    set_halign: gtk::Align::Center,
                                    set_valign: gtk::Align::End,
                                    set_vexpand: true,
                                    set_width_request: 300,

                                    gtk::Button {
                                        connect_clicked => AppInput::SecureMyConnection,
                                        set_hexpand: true,
                                        set_label: &tr!("Secure my connection"),
                                        set_css_classes: &["opaque", "secure_my_connection_btn"],

                                        #[track = "model.state_changed()"]
                                        set_visible: model.can_secure_connection()
                                    },

                                    gtk::Button {
                                        connect_clicked => AppInput::CancelConnection,
                                        set_hexpand: true,
                                        set_label: &tr!("Cancel"),
                                        set_css_classes: &["opaque", "disconnect_btn"],

                                        #[track = "model.state_changed()"]
                                        set_visible: model.is_connecting_or_reconnecting()
                                    },

                                    gtk::Button {
                                        connect_clicked => AppInput::Disconnect,
                                        set_hexpand: true,
                                        set_label: &tr!("Disconnect"),
                                        set_css_classes: &["opaque", "disconnect_btn"],

                                        #[track = "model.state_changed()"]
                                        set_visible: model.can_disconnect()
                                    },

                                    gtk::Button {
                                        connect_clicked => AppInput::Reconnect,
                                        set_css_classes: &["opaque", "reconnect_btn"],
                                        set_icon_name: icon_names::REFRESH_LARGE,

                                        #[track = "model.state_changed()"]
                                        set_visible: model.can_reconnect(),
                                    },
                                }
                            }
                        },
                        DaemonState::Connecting => {
                            gtk::Label {
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

    menu! {
        primary_menu: {
            section! {
                &tr!("Preferences") => PreferencesAction,
                &tr!("About") => AboutAction,
            },
        }
    }

    async fn init(
        _: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        sender.command(|out, shutdown| {
            shutdown
                .register(listen_to_mullvad_events(out))
                .drop_on_shutdown()
                .boxed()
        });

        let model = AppModel {
            components: Some(AppComponents {
                preferences: PreferencesModel::builder()
                    .transient_for(&root)
                    .launch(())
                    .forward(sender.command_sender(), identity),
            }),
            ..Default::default()
        };

        let widgets = view_output!();

        let mut group = RelmActionGroup::<WindowActionGroup>::new();
        // PreferencesAction
        {
            let sender = sender.clone();
            group.add_action(RelmAction::<PreferencesAction>::new_stateless(move |_| {
                sender.input(AppInput::Preferences);
            }));
        }
        // AboutAction
        {
            let sender = sender.clone();
            group.add_action(RelmAction::<AboutAction>::new_stateless(move |_| {
                sender.input(AppInput::About);
            }));
        }

        widgets
            .main_window
            .insert_action_group("win", Some(&group.into_action_group()));

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        _sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) {
        self.reset();

        let daemon_connector = &mut self.daemon_connector;

        match message {
            AppInput::SecureMyConnection => {
                let _ = daemon_connector.secure_my_connection().await;
            }
            AppInput::Reconnect => {
                let _ = daemon_connector.reconnect().await;
            }
            AppInput::CancelConnection | AppInput::Disconnect => {
                let _ = daemon_connector.disconnect().await;
            }
            AppInput::Preferences => {
                if let Some(components) = self.get_components() {
                    components.preferences.emit(PreferencesMsg::Show);
                }
            }
            AppInput::About => {
                let dialog = adw::AboutWindow::builder()
                    .icon_name("background-app-ghost-symbolic")
                    .application_icon("background-app-ghost-symbolic")
                    .application_name("Mullvadwaita")
                    .developer_name("Lessneek")
                    .website("Website")
                    .copyright("© 2024 Lessneek")
                    .license_type(gtk::License::Gpl30)
                    .website("https://github.com/lessneek/mullvadwaita")
                    .issue_url("https://github.com/lessneek/mullvadwaita/issues")
                    .version(env!("CARGO_PKG_VERSION"))
                    .modal(true)
                    .transient_for(root)
                    .developers(vec!["Lessneek", "aiska"])
                    .comments("Mullvad VPN daemon controller.")
                    .build();
                dialog.present();
            }
        }
    }

    async fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            AppMsg::DaemonEvent(event) => {
                match event {
                    Event::TunnelState(state) => {
                        self.set_daemon_state(DaemonState::Connected {
                            tunnel_state: state,
                        });
                    }
                    Event::ConnectingToDaemon => self.set_daemon_state(DaemonState::Connecting),
                    Event::DeviceState(device_state) => {
                        self.set_account_and_device(device_state.into_device());
                    }
                    Event::AccountData(account_data) => self.set_account_data(Some(account_data)),
                };
                self.update_properties();
            }
        }
    }
}

async fn listen_to_mullvad_events(out: relm4::Sender<AppMsg>) {
    let mut events_rx = mullvad::events_receiver();

    trace!("Listening for status updates...");

    while let Some(event) = events_rx.recv().await {
        debug!("Daemon event: {:#?}", event);
        if let Err(msg) = out.send(AppMsg::DaemonEvent(event)) {
            debug!("Can't send an app message {msg:?} because all receivers were dropped");
            break;
        }
    }

    trace!("Status updates stopped.");
}

relm4::new_action_group!(WindowActionGroup, "win");
relm4::new_stateless_action!(PreferencesAction, WindowActionGroup, "preferences");
relm4::new_stateless_action!(AboutAction, WindowActionGroup, "about");
