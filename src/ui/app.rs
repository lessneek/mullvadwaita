use std::convert::identity;

use super::account::{AccountModel, AccountMsg};
use super::preferences::{Pref, PreferencesModel, PreferencesMsg};

use crate::extensions::{ToStr, TunnelStateExt};
use crate::mullvad::{self, DaemonConnector, Event};

use crate::tr;
use chrono::prelude::*;
use futures::FutureExt;
use mullvad_management_interface::Error;
use smart_default::SmartDefault;

use relm4::actions::{AccelsPlus, RelmAction, RelmActionGroup};
use relm4::prelude::*;

use adw::prelude::*;

use mullvad_types::account::{AccountData, AccountToken};
use mullvad_types::device::{AccountAndDevice, DeviceState};
use mullvad_types::states::TunnelState;
use talpid_types::tunnel::ActionAfterDisconnect;

#[derive(Debug)]
pub enum AppInput {
    SecureMyConnection,
    CancelConnection,
    Disconnect,
    Reconnect,
    Account,
    Preferences,
    About,
    Set(Pref),
    Login(AccountToken),
    Logout,
    CreateAccount,
}

#[derive(Debug)]
pub enum AppMsg {
    DaemonEvent(Event),
    Ignore,
}

#[tracker::track]
#[derive(SmartDefault)]
pub struct AppModel {
    #[no_eq]
    state: AppState,

    #[no_eq]
    tunnel_state: Option<TunnelState>,
    account_data: Option<AccountData>,
    account_history: Option<AccountToken>,

    banner_label: Option<String>,
    device_name: Option<String>,
    time_left: Option<String>,
    tunnel_state_label: Option<String>,
    country: Option<String>,
    city: Option<String>,
    hostname: Option<String>,
    tunnel_protocol: Option<String>,
    tunnel_in: Option<String>,
    tunnel_out: Option<String>,

    #[no_eq]
    components: Option<AppComponents>,

    #[do_not_track]
    daemon_connector: DaemonConnector,

    #[no_eq]
    account_action: Option<RelmAction<AccountAction>>,
}

pub struct AppComponents {
    account: AsyncController<AccountModel>,
    preferences: AsyncController<PreferencesModel>,
}

#[derive(Debug, SmartDefault)]
#[allow(clippy::large_enum_variant)]
enum AppState {
    LoggedIn(AccountAndDevice),
    LoggedOut,
    Revoked,
    #[default]
    ConnectingToDaemon,
}

impl AppState {
    fn get_account_and_device(&self) -> Option<&AccountAndDevice> {
        match self {
            AppState::LoggedIn(ref account_and_device) => Some(account_and_device),
            _ => None,
        }
    }
}

impl AppModel {
    fn is_logged_in(&self) -> bool {
        matches!(self.get_state(), AppState::LoggedIn { .. })
    }

    fn can_secure_connection(&self) -> bool {
        matches!(
            self.get_tunnel_state(),
            Some(
                TunnelState::Disconnected { .. }
                    | TunnelState::Disconnecting(
                        ActionAfterDisconnect::Nothing | ActionAfterDisconnect::Block
                    )
            )
        )
    }

    fn is_connected(&self) -> bool {
        self.get_tunnel_state()
            .as_ref()
            .map_or(false, |ts| ts.is_connected())
    }

    fn is_connecting_or_reconnecting(&self) -> bool {
        self.get_tunnel_state()
            .as_ref()
            .map(|ts| ts.is_connecting_or_reconnecting() || ts.is_in_error_state())
            .unwrap_or(false)
    }

    fn can_disconnect(&self) -> bool {
        self.get_tunnel_state()
            .as_ref()
            .map(|ts| ts.is_connected())
            .unwrap_or(false)
    }

    fn can_reconnect(&self) -> bool {
        self.get_tunnel_state()
            .as_ref()
            .map(|ts| ts.is_connecting_or_connected())
            .unwrap_or(false)
    }

    fn tunnel_state_changed(&self) -> bool {
        self.changed(AppModel::tunnel_state())
    }

    fn get_account_token(&self) -> Option<String> {
        self.state
            .get_account_and_device()
            .map(|acc| acc.account_token.clone())
    }

    fn fetch_account_data(&self, sender: AsyncComponentSender<Self>) {
        if let Some(account_token) = self.get_account_token() {
            let mut daemon_connector = self.daemon_connector.clone();

            sender.oneshot_command(async move {
                if let Ok(account_data) = daemon_connector.get_account_data(account_token).await {
                    return AppMsg::DaemonEvent(Event::AccountData(account_data));
                }
                AppMsg::Ignore
            });
        }
    }

    fn update_properties(&mut self) {
        if let Some(ts) = self.get_tunnel_state() {
            let banner_label = match ts {
                TunnelState::Error(error_state) => Some(format!("{}", error_state.cause())),
                _ => None,
            };

            let tunnel_state_label = Some(ts.get_tunnel_state_label());
            let country = ts.get_country();
            let city = ts.get_city();
            let hostname = ts.get_hostname();
            let tunnel_protocol = ts.get_tunnel_protocol();
            let tunnel_in = ts.get_tunnel_in();
            let tunnel_out = ts.get_tunnel_out();

            self.set_banner_label(banner_label);
            self.set_tunnel_state_label(tunnel_state_label);
            self.set_country(country);
            self.set_city(city);
            self.set_hostname(hostname);
            self.set_tunnel_protocol(tunnel_protocol);
            self.set_tunnel_in(tunnel_in);
            self.set_tunnel_out(tunnel_out);
        }

        self.set_device_name(match self.get_state() {
            AppState::LoggedIn(devacc) => {
                Some(tr!("<b>Device name</b>: {}", devacc.device.pretty_name()))
            }
            _ => None,
        });

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

                    #[transition(SlideLeftRight)]
                    match (model.get_state(), model.get_tunnel_state()) {
                        (AppState::LoggedIn(_), Some(tunnel_state)) => {
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

                                    match tunnel_state {
                                        TunnelState::Connected { .. } => {
                                            gtk::Image {
                                                set_icon_name: Some("network-vpn-symbolic"),
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
                                                set_icon_name: Some("network-vpn-disabled-symbolic"),
                                                set_css_classes: &[
                                                    "connection_state_icon",
                                                    "disabled",
                                                    "icon-dropshadow"
                                                ]
                                            }
                                        },
                                        TunnelState::Disconnected { locked_down: false, .. } => {
                                            gtk::Image {
                                                set_icon_name: Some("network-vpn-disconnected-symbolic"),
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

                                    #[track = "model.tunnel_state_changed()"]
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

                                        #[track = "model.tunnel_state_changed()"]
                                        set_visible: model.can_secure_connection()
                                    },

                                    gtk::Button {
                                        connect_clicked => AppInput::CancelConnection,
                                        set_hexpand: true,
                                        set_label: &tr!("Cancel"),
                                        set_css_classes: &["opaque", "disconnect_btn"],

                                        #[track = "model.tunnel_state_changed()"]
                                        set_visible: model.is_connecting_or_reconnecting()
                                    },

                                    gtk::Button {
                                        connect_clicked => AppInput::Disconnect,
                                        set_hexpand: true,
                                        set_label: &tr!("Disconnect"),
                                        set_css_classes: &["opaque", "disconnect_btn"],

                                        #[track = "model.tunnel_state_changed()"]
                                        set_visible: model.can_disconnect()
                                    },

                                    gtk::Button {
                                        connect_clicked => AppInput::Reconnect,
                                        set_css_classes: &["opaque", "reconnect_btn"],
                                        set_icon_name: "arrow-circular-top-right-symbolic",

                                        #[track = "model.tunnel_state_changed()"]
                                        set_visible: model.can_reconnect(),
                                    },
                                }
                            }
                        }
                        // Login page.
                        (AppState::LoggedOut | AppState::Revoked, ..) => {
                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_margin_all: 20,
                                set_valign: gtk::Align::Center,

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

                                    append: account_number = &adw::EntryRow {
                                        set_title: &tr!("Enter your account number"),

                                        #[track = "model.changed(AppModel::state())"]
                                        set_text: "",

                                        connect_entry_activated[login_button] => move |_| {
                                            login_button.emit_clicked();
                                        },

                                        add_suffix: login_button = &gtk::Button {
                                            set_icon_name: "arrow2-right-symbolic",
                                            set_valign: gtk::Align::Center,
                                            set_css_classes: &["flat"],
                                            set_receives_default: true,
                                            connect_clicked[sender, account_number] => move |_| {
                                                sender.input(AppInput::Login(account_number.text().into()));
                                            }
                                        },
                                    },

                                    adw::ActionRow {
                                        #[track = "model.changed(AppModel::account_history())"]
                                        set_title: model.get_account_history().to_str(),

                                        #[track = "model.changed(AppModel::account_history())"]
                                        set_visible: model.get_account_history().is_some(),

                                        set_activatable: true,

                                        connect_activated[account_number, login_button] => move |this| {
                                            account_number.set_text(this.title().as_ref());
                                            login_button.emit_clicked();
                                        }
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

                                    gtk::Button {
                                        set_label: &tr!("Create account"),
                                        connect_clicked[sender] => move |_| {
                                            sender.input(AppInput::CreateAccount);
                                        }
                                    },
                                },
                            }
                        }
                        (AppState::ConnectingToDaemon, ..) | (_, None) => {
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
                &tr!("Account") => AccountAction,
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

        // Actions
        let mut group = RelmActionGroup::<WindowActionGroup>::new();
        let account_action: RelmAction<AccountAction>;
        {
            let app = relm4::main_adw_application();
            app.set_accelerators_for_action::<PreferencesAction>(&["<primary>comma"]);

            // Account
            {
                let sender = sender.clone();
                account_action = RelmAction::<AccountAction>::new_stateless(move |_| {
                    sender.input(AppInput::Account);
                });
                account_action.set_enabled(false);
                group.add_action(account_action.clone());
            }

            // Preferences
            {
                let sender = sender.clone();
                group.add_action(RelmAction::<PreferencesAction>::new_stateless(move |_| {
                    sender.input(AppInput::Preferences);
                }));
            }

            // About
            {
                let sender = sender.clone();
                group.add_action(RelmAction::<AboutAction>::new_stateless(move |_| {
                    sender.input(AppInput::About);
                }));
            }
        }

        let model = AppModel {
            components: Some(AppComponents {
                account: AccountModel::builder()
                    .transient_for(&root)
                    .launch(())
                    .forward(sender.input_sender(), identity),
                preferences: PreferencesModel::builder()
                    .transient_for(&root)
                    .launch(())
                    .forward(sender.input_sender(), identity),
            }),
            account_action: Some(account_action),
            ..Default::default()
        };

        let widgets = view_output!();

        group.register_for_widget(&widgets.main_window);

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
            AppInput::Login(account_token) => {
                let error_text = daemon_connector
                    .login_account(account_token)
                    .await
                    .map_err(|err| {
                        match err.downcast_ref() {
                            Some(Error::InvalidAccount) => {
                                tr!("Login failed. Invalid account number.")
                            }
                            // TODO: process other errors.
                            _ => tr!("Login failed"),
                        }
                    })
                    .err();
                self.set_banner_label(error_text);
            }
            AppInput::Logout => {
                let _ = daemon_connector.logout_account().await;
            }
            AppInput::CreateAccount => {
                let error_text = daemon_connector
                    .create_new_account()
                    .await
                    .map_err(|_| tr!("Creating account failed")) // TODO: process other errors.
                    .err();
                self.set_banner_label(error_text);
            }
            AppInput::SecureMyConnection => {
                let _ = daemon_connector.secure_my_connection().await;
            }
            AppInput::Reconnect => {
                let _ = daemon_connector.reconnect().await;
            }
            AppInput::CancelConnection | AppInput::Disconnect => {
                let _ = daemon_connector.disconnect().await;
            }
            AppInput::Account => {
                if let Some(components) = self.get_components() {
                    components.account.emit(AccountMsg::Show);
                }
            }
            AppInput::Preferences => {
                if let Some(components) = self.get_components() {
                    components.preferences.emit(PreferencesMsg::Show);
                }
            }
            AppInput::Set(pref) => match pref {
                Pref::AutoConnect(value) => {
                    self.daemon_connector.set_auto_connect(value).await.ok();
                }
                Pref::LocalNetworkSharing(value) => {
                    self.daemon_connector.set_allow_lan(value).await.ok();
                }
                Pref::LockdownMode(value) => {
                    self.daemon_connector
                        .set_block_when_disconnected(value)
                        .await
                        .ok();
                }
                Pref::EnableIPv6(value) => {
                    self.daemon_connector.set_enable_ipv6(value).await.ok();
                }
            },
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
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            AppMsg::Ignore => {}
            AppMsg::DaemonEvent(event) => {
                log::debug!("Daemon event: {:#?}", event);
                match event {
                    Event::TunnelState(new_tunnel_state) => {
                        self.set_tunnel_state(Some(new_tunnel_state));
                        self.fetch_account_data(sender.clone());
                    }
                    Event::ConnectingToDaemon => self.set_state(AppState::ConnectingToDaemon),
                    Event::Device(device_event) => match device_event.new_state {
                        DeviceState::LoggedIn(account_and_device) => {
                            self.set_state(AppState::LoggedIn(account_and_device.clone()));

                            if let Some(components) = self.get_components() {
                                components
                                    .account
                                    .emit(AccountMsg::UpdateAccountAndDevice(account_and_device));
                            }
                            self.fetch_account_data(sender.clone());
                        }
                        DeviceState::LoggedOut => {
                            self.set_state(AppState::LoggedOut);
                            if let Ok(token) = self.daemon_connector.get_account_history().await {
                                self.set_account_history(token);
                            }
                        }
                        DeviceState::Revoked => self.set_state(AppState::Revoked),
                    },
                    Event::RemoveDevice(_) => {}
                    Event::AccountData(account_data) => {
                        self.set_account_data(Some(account_data.clone()));

                        if let Some(components) = self.get_components() {
                            components
                                .account
                                .emit(AccountMsg::UpdateAccountData(account_data));
                        }
                    }
                    Event::Setting(settings) => {
                        if let Some(components) = self.get_components() {
                            components
                                .preferences
                                .emit(PreferencesMsg::UpdateSettings(settings));
                        }
                    }
                    Event::AppVersionInfo(_) => {}
                    Event::RelayList(_) => {}
                    Event::NewAccessMethod(_) => {}
                };
                self.update_properties();

                if self.changed(AppModel::state()) {
                    if let Some(account_action) = &self.account_action {
                        account_action.set_enabled(self.is_logged_in());
                    }
                }
            }
        }
    }
}

async fn listen_to_mullvad_events(out: relm4::Sender<AppMsg>) {
    let mut events_rx = mullvad::events_receiver();

    log::trace!("Listening for status updates...");

    while let Some(event) = events_rx.recv().await {
        if let Err(msg) = out.send(AppMsg::DaemonEvent(event)) {
            log::debug!("Can't send an app message {msg:?} because all receivers were dropped");
            break;
        }
    }

    log::trace!("Status updates stopped.");
}

relm4::new_action_group!(WindowActionGroup, "win");
relm4::new_stateless_action!(AccountAction, WindowActionGroup, "account");
relm4::new_stateless_action!(PreferencesAction, WindowActionGroup, "preferences");
relm4::new_stateless_action!(AboutAction, WindowActionGroup, "about");

impl Clone for AccountAction {
    fn clone(&self) -> Self {
        Self {}
    }
}
