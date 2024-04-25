use std::convert::identity;

use super::about;
use super::account::{AccountModel, AccountMsg};
use super::main_window::MainWindow;
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
    ClearAccountHistory,
}

#[derive(Debug)]
pub enum AppMsg {
    DaemonEvent(Event),
    LoginError(String),
    CreateAccountError(String),
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

    lockdown_mode: bool,

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
    Login(LoginState),
    #[default]
    ConnectingToDaemon,
}

#[derive(Debug, SmartDefault)]
enum LoginState {
    #[default]
    Normal,
    LoggingIn,
    CreatingAccount,
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

    fn is_logging_in(&self) -> bool {
        matches!(self.get_state(), AppState::Login(LoginState::LoggingIn))
    }

    fn is_creating_account(&self) -> bool {
        matches!(
            self.get_state(),
            AppState::Login(LoginState::CreatingAccount)
        )
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

    fn state_changed(&self) -> bool {
        self.changed(AppModel::state())
    }

    fn tunnel_state_changed(&self) -> bool {
        self.changed(AppModel::tunnel_state())
    }

    fn get_tunnel_state_if_changed(&self) -> Option<&TunnelState> {
        self.tunnel_state_changed()
            .then(|| self.get_tunnel_state().as_ref())
            .flatten()
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
        if let Some(ts) = self.get_tunnel_state_if_changed() {
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

        if self.state_changed() {
            if let Some(account_action) = &self.account_action {
                account_action.set_enabled(self.is_logged_in());
            }

            self.set_device_name(match self.get_state() {
                AppState::LoggedIn(acc_and_dev) => Some(tr!(
                    "<b>Device name</b>: {}",
                    acc_and_dev.device.pretty_name()
                )),
                _ => None,
            });
        }
    }

    fn get_current_view_name(&self) -> &'static str {
        match (self.get_state(), self.get_tunnel_state().is_some()) {
            // Main page.
            (AppState::LoggedIn(_), true) => "logged_in",
            (AppState::Login(_), ..) => "login",
            (AppState::ConnectingToDaemon, ..) | (_, false) => "connecting_to_daemon",
        }
    }

    pub fn get_tunnel_state_view_name(&self) -> &'static str {
        self.get_tunnel_state()
            .as_ref()
            .map(|tunnel_state| match tunnel_state {
                TunnelState::Connected { .. } => "connected",
                TunnelState::Connecting { .. } => "connecting",
                TunnelState::Disconnected {
                    locked_down: true, ..
                } => "disabled",
                TunnelState::Disconnected {
                    locked_down: false, ..
                } => "disconnected",
                _ => "_",
            })
            .unwrap_or("_")
    }
}

#[relm4::component(async, pub)]
impl AsyncComponent for AppModel {
    type Init = ();
    type Input = AppInput;
    type Output = ();
    type CommandOutput = AppMsg;

    view! {
        #[root]
        #[template]
        #[name = "main_window"]
        MainWindow {
            #[template_child]
            primary_menu_button {
                set_menu_model: Some(&primary_menu),
            },

            #[template_child]
            banner {
                #[track = "model.changed(AppModel::banner_label())"]
                set_title: model.get_banner_label().to_str(),

                #[track = "model.changed(AppModel::banner_label())"]
                set_revealed: model.get_banner_label().is_some(),
            },

            #[template_child]
            view_stack {
                #[watch]
                set_visible_child_name: model.get_current_view_name(),
            },

            #[template_child]
            logged_in_view.device_name_label {
                #[track = "model.changed(AppModel::device_name())"]
                set_label: model.get_device_name().to_str(),
            },

            #[template_child]
            logged_in_view.time_left_label {
                #[track = "model.changed(AppModel::time_left())"]
                set_label: model.get_time_left().to_str(),
            },

            #[template_child]
            logged_in_view.tunnel_state_view.view_stack {
                #[track = "model.tunnel_state_changed()"]
                set_visible_child_name: model.get_tunnel_state_view_name(),
            },

            #[template_child]
            logged_in_view.tunnel_state_label {
                #[track = "model.changed(AppModel::tunnel_state_label())"]
                set_label: model.get_tunnel_state_label().to_str(),

                #[track = "model.tunnel_state_changed()"]
                set_class_active[model.is_connected()]: "connected_state_label"
            },

            #[template_child]
            logged_in_view.country_label {
                #[track = "model.changed(AppModel::country())"]
                set_label: model.get_country().to_str(),
            },

            #[template_child]
            logged_in_view.city_label {
                #[track = "model.changed(AppModel::city())"]
                set_label: model.get_city().to_str(),
            },

            #[template_child]
            logged_in_view.hostname_listbox {
                #[track = "model.changed(AppModel::hostname())"]
                set_visible: model.get_hostname().is_some(),
            },

            #[template_child]
            logged_in_view.hostname_expander_row {
                #[track = "model.changed(AppModel::hostname())"]
                set_title: model.get_hostname().to_str(),
            },

            #[template_child]
            logged_in_view.tunnel_protocol_row {
                #[track = "model.changed(AppModel::tunnel_protocol())"]
                set_subtitle: model.get_tunnel_protocol().to_str(),
            },

            #[template_child]
            logged_in_view.tunnel_in_row {
                #[track = "model.changed(AppModel::tunnel_in())"]
                set_subtitle: model.get_tunnel_in().to_str(),
            },

            #[template_child]
            logged_in_view.tunnel_out_row {
                #[track = "model.changed(AppModel::tunnel_out())"]
                set_subtitle: model.get_tunnel_out().to_str(),
            },

            #[template_child]
            logged_in_view.secure_my_connection_button {
                connect_clicked => AppInput::SecureMyConnection,

                #[track = "model.tunnel_state_changed()"]
                set_visible: model.can_secure_connection(),
            },

            #[template_child]
            logged_in_view.cancel_button {
                connect_clicked => AppInput::CancelConnection,

                #[track = "model.tunnel_state_changed()"]
                set_visible: model.is_connecting_or_reconnecting(),
            },

            #[template_child]
            logged_in_view.disconnect_button {
                connect_clicked => AppInput::Disconnect,

                #[track = "model.tunnel_state_changed()"]
                set_visible: model.can_disconnect(),
            },

            #[template_child]
            logged_in_view.reconnect_button {
                connect_clicked => AppInput::Reconnect,

                #[track = "model.tunnel_state_changed()"]
                set_visible: model.can_reconnect(),
            },

            #[template_child]
            login_view {
                #[watch]
                set_sensitive: !(model.is_logging_in() | model.is_creating_account()),
            },

            #[template_child]
            login_view.disable_lockdown_mode_bin {
                #[track = "model.changed(AppModel::lockdown_mode())"]
                set_visible: model.lockdown_mode,
            },

            #[template_child]
            login_view.disable_lockdown_mode_button {
                connect_clicked[sender] => move |_| {
                    sender.input(AppInput::Set(Pref::LockdownMode(false)));
                }
            },

            #[template_child]
            login_view.account_number {
                #[track = "model.is_logged_in()"]
                set_text: "",

                connect_entry_activated[main_window] => move |_| {
                    main_window.login_view.login_button.emit_clicked();
                },
            },

            #[template_child]
            login_view.login_button_stack {
                set_visible_child_name: if model.is_logging_in() { "logging_in" } else { "default" }
            },

            #[template_child]
            login_view.login_button {
                connect_clicked[sender, main_window] => move |_| {
                    sender.input(AppInput::Login(main_window.login_view.account_number.text().into()));
                }
            },

            #[template_child]
            login_view.account_history_row {
                #[track = "model.changed(AppModel::account_history())"]
                set_title: model.get_account_history().to_str(),

                #[track = "model.changed(AppModel::account_history())"]
                set_visible: model.get_account_history().is_some(),
            },

            #[template_child]
            login_view.clear_account_history_button {
                connect_clicked[sender] => move |_| {
                    sender.input(AppInput::ClearAccountHistory);
                }
            },

            #[template_child]
            login_view.create_account_button {
                connect_clicked[sender] => move |_| {
                    sender.input(AppInput::CreateAccount);
                },
            },
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
                    .transient_for(&*root)
                    .launch(())
                    .forward(sender.input_sender(), identity),
                preferences: PreferencesModel::builder()
                    .transient_for(&*root)
                    .launch(())
                    .forward(sender.input_sender(), identity),
            }),
            account_action: Some(account_action),
            ..Default::default()
        };

        let widgets = view_output!();

        group.register_for_widget(&*widgets.main_window);

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) {
        self.reset();

        log::debug!("AppInput: {message:#?}");

        match message {
            AppInput::Login(account_token) => {
                self.set_banner_label(None);
                self.set_state(AppState::Login(LoginState::LoggingIn));

                let mut daemon_connector = self.daemon_connector.clone();
                sender.oneshot_command(async move {
                    let login_result =
                        daemon_connector
                            .login_account(account_token)
                            .await
                            .map_err(|err| {
                                log::debug!("Login error: {:#?}", err);
                                match err.downcast_ref() {
                                    Some(Error::InvalidAccount) => {
                                        tr!("Login failed. Invalid account number.")
                                    }
                                    Some(Error::TooManyDevices) => {
                                        tr!("Login failed. Too many devices.")
                                    }
                                    // TODO: process other errors.
                                    _ => tr!("Login failed"),
                                }
                            });
                    match login_result {
                        Ok(_) => AppMsg::Ignore,
                        Err(error) => AppMsg::LoginError(error),
                    }
                });
            }
            AppInput::Logout => {
                let _ = self.daemon_connector.logout_account().await;
            }
            AppInput::CreateAccount => {
                self.set_banner_label(None);
                self.set_state(AppState::Login(LoginState::CreatingAccount));

                let mut daemon_connector = self.daemon_connector.clone();
                sender.oneshot_command(async move {
                    let result = daemon_connector.create_new_account().await.map_err(|err| {
                        log::debug!("{:#?}", err);
                        // TODO: process other errors.
                        tr!("Creating account failed")
                    });
                    match result {
                        Ok(_) => AppMsg::Ignore,
                        Err(error) => AppMsg::CreateAccountError(error),
                    }
                });
            }
            AppInput::ClearAccountHistory => {
                let _ = self.daemon_connector.clear_account_history().await;
                if let Ok(token) = self.daemon_connector.get_account_history().await {
                    self.set_account_history(token);
                }
            }
            AppInput::SecureMyConnection => {
                let _ = self.daemon_connector.secure_my_connection().await;
            }
            AppInput::Reconnect => {
                let _ = self.daemon_connector.reconnect().await;
            }
            AppInput::CancelConnection | AppInput::Disconnect => {
                let _ = self.daemon_connector.disconnect().await;
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
                Pref::RelaySettings(relay_settings) => {
                    self.daemon_connector
                        .set_relay_settings(*relay_settings)
                        .await
                        .ok();
                }
            },
            AppInput::About => about::show_about_dialog(&**root),
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
                        // TODO: process `revoked` state.
                        DeviceState::LoggedOut | DeviceState::Revoked => {
                            self.set_state(AppState::Login(LoginState::Normal));
                            if let Ok(token) = self.daemon_connector.get_account_history().await {
                                self.set_account_history(token);
                            }
                        }
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
                        self.set_lockdown_mode(settings.block_when_disconnected);

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
            }
            AppMsg::LoginError(error) | AppMsg::CreateAccountError(error) => {
                self.set_banner_label(Some(error));
                self.set_state(AppState::Login(LoginState::Normal));
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
