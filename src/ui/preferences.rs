use adw::prelude::*;
use relm4::prelude::*;

use mullvad_types::{
    relay_constraints::{RelayConstraints, RelaySettings},
    settings::Settings,
};

use crate::{
    icon_names, tr,
    ui::{
        app::AppInput,
        types::*,
        variant_selector::{Unique, VariantMode, VariantSelectorMsg},
        widgets::InfoButton,
    },
};

use super::variant_selector::VariantSelector;

// TODO: get the nets from mullvad sources.
static ALLOWED_LAN_NETS: [&str; 6] = [
    "10.0.0.0/8",
    "172.16.0.0/12",
    "192.168.0.0/16",
    "169.254.0.0/16",
    "fe80::/10",
    "fc00::/7",
];

#[tracker::track]
#[derive(Debug)]
pub struct PreferencesModel {
    window: adw::PreferencesDialog,
    parent: Option<gtk::Widget>,

    #[no_eq]
    tunnel_protocol_selector: Controller<VariantSelector<TunnelProtocol>>,

    #[no_eq]
    wireguard_port_selector: Controller<VariantSelector<WireGuardPort>>,

    local_network_sharing: bool,
    lockdown_mode: bool,
    enable_ipv6: bool,
    auto_connect: bool,

    settings: Option<Settings>,
}

#[derive(Debug)]
pub struct PreferencesInit {
    pub parent: Option<gtk::Widget>,
}

#[derive(Debug)]
pub enum PreferencesMsg {
    Show,
    Close,
    UpdateSettings(Box<Settings>),
    TunnelProtocolChanged(TunnelProtocol),
    WireGuardPortChanged(WireGuardPort),
    SetMultihop(bool),
    TryEnableLockdownModeWithAlert,
    SetLockdownMode(bool),
}

#[derive(Debug)]
pub enum Pref {
    AutoConnect(bool),
    LocalNetworkSharing(bool),
    LockdownMode(bool),
    EnableIPv6(bool),
    RelaySettings(Box<RelaySettings>),
}

impl PreferencesModel {
    fn get_normal_relay_constraints(&self) -> Option<&RelayConstraints> {
        if let Some(RelaySettings::Normal(relay_constraints)) = self
            .get_settings()
            .as_ref()
            .map(|settings| &settings.relay_settings)
        {
            return Some(relay_constraints);
        }
        None
    }

    fn update_normal_relay_constraints<F>(&mut self, sender: AsyncComponentSender<Self>, func: F)
    where
        F: FnOnce(&mut RelayConstraints),
    {
        if let Some(RelaySettings::Normal(relay_constraints)) = self
            .get_mut_settings()
            .as_mut()
            .map(|settings| &mut settings.relay_settings)
        {
            func(relay_constraints);

            sender
                .output(AppInput::Set(Pref::RelaySettings(Box::new(
                    RelaySettings::Normal(relay_constraints.clone()),
                ))))
                .ok();
        }
    }

    fn get_tunnel_protocol(&self) -> Option<TunnelProtocol> {
        self.get_normal_relay_constraints()
            .map(|relay_constraints| relay_constraints.tunnel_protocol.into())
    }

    fn get_wireguard_port(&self) -> Option<WireGuardPort> {
        self.get_normal_relay_constraints()
            .map(|relay_constraints| relay_constraints.wireguard_constraints.port.into())
    }

    fn is_multihop_enabled(&self) -> bool {
        self.get_normal_relay_constraints()
            .map(|relay_constraints| relay_constraints.wireguard_constraints.multihop())
            .unwrap_or_default()
    }

    fn is_quantum_resistant_enabled(&self) -> bool {
        self.get_settings()
            .as_ref()
            .map(|settings| {
                matches!(
                    settings.tunnel_options.wireguard.quantum_resistant,
                    mullvad_types::wireguard::QuantumResistantState::On
                )
            })
            .unwrap_or_default()
    }

    fn is_daita_enabled(&self) -> bool {
        self.get_settings()
            .as_ref()
            .map(|settings| settings.tunnel_options.wireguard.daita.enabled)
            .unwrap_or_default()
    }

    fn is_multihop_allowed(&self) -> bool {
        self.get_tunnel_protocol()
            .map(|value| match value {
                TunnelProtocol::WireGuard => true,
                TunnelProtocol::OpenVPN => false,
            })
            .unwrap_or_default()
    }
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for PreferencesModel {
    type Init = PreferencesInit;
    type Input = PreferencesMsg;
    type Output = AppInput;
    type Widgets = PreferencesWidgets;

    view! {
        adw::PreferencesDialog {
            set_can_close: false,
            connect_close_attempt[sender] => move |_| {
                sender.input(PreferencesMsg::Close);
            },
            add = &adw::PreferencesPage {
                add = &adw::PreferencesGroup {
                    set_title: &tr!("VPN"),

                    // Auto-connect.
                    add = &adw::SwitchRow {
                        set_title: &tr!("Auto-connect"),
                        add_prefix = &gtk::Image {
                            set_icon_name: Some(icon_names::WIRED_LOCK_CLOSED),
                        },
                        set_subtitle: &tr!("Automatically connect to a server when the app launches."),

                        #[track = "model.changed(PreferencesModel::auto_connect())"]
                        #[block_signal(auto_connect_active_notify_handler)]
                        set_active: model.auto_connect,

                        connect_active_notify[sender] => move |this| {
                            let _ = sender.output(AppInput::Set(Pref::AutoConnect(this.is_active())));
                        } @auto_connect_active_notify_handler
                    },

                    // Local network sharing.
                    add = &adw::ActionRow {
                        set_title: &tr!("Local network sharing"),
                        add_prefix = &gtk::Image {
                            set_icon_name: Some(icon_names::NETWORK_WORKGROUP),
                        },
                        set_activatable: true,

                        connect_activated[local_network_sharing_switch] => move |_| {
                            if local_network_sharing_switch.get_sensitive() {
                                local_network_sharing_switch.activate();
                            }
                        },

                        #[template]
                        add_suffix = &InfoButton {
                            #[template_child]
                            info_label {
                                set_label: {
                                    &format!("{}\n\n{}{}",
                                        &tr!("This feature allows access to other devices on the local network, such as for sharing, printing, streaming, etc."),
                                        &tr!("It does this by allowing network communication outside the tunnel to local multicast and broadcast ranges as well as to and from these private IP ranges:"),
                                        ALLOWED_LAN_NETS.iter().fold(String::new(), |acc, &s| format!("{acc}\n • {s}"))
                                    )
                                },
                            }
                        },

                        #[name = "local_network_sharing_switch"]
                        add_suffix = &gtk::Switch {
                            set_valign: gtk::Align::Center,

                            #[track = "model.changed(PreferencesModel::local_network_sharing())"]
                            #[block_signal(local_network_sharing_active_notify_handler)]
                            set_active: model.local_network_sharing,

                            connect_active_notify[sender] => move |this| {
                                let _ = sender.output(AppInput::Set(Pref::LocalNetworkSharing(this.is_active())));
                            } @local_network_sharing_active_notify_handler,
                        },
                    },

                    // Enable IPv6.
                    add = &adw::ActionRow {
                        set_title: &tr!("Enable IPv6"),
                        add_prefix = &gtk::Image {
                            set_icon_name: Some(icon_names::GLOBE_ALT2),
                        },
                        set_activatable: true,

                        connect_activated[enable_ipv6_switch] => move |_| {
                            if enable_ipv6_switch.get_sensitive() {
                                enable_ipv6_switch.activate();
                            }
                        },

                        #[template]
                        add_suffix = &InfoButton {
                            #[template_child]
                            info_label {
                                set_label: {
                                    &format!("{}\n\n{}",
                                        &tr!("IPv4 is always enabled and the majority of websites and applications use this protocol. We do not recommend enabling IPv6 unless you know you need it."),
                                        &tr!("When this feature is enabled, IPv6 can be used alongside IPv4 in the VPN tunnel to communicate with internet services.")
                                    )
                                },
                            }
                        },

                        #[name = "enable_ipv6_switch"]
                        add_suffix = &gtk::Switch {
                            set_valign: gtk::Align::Center,

                            #[track = "model.changed(PreferencesModel::enable_ipv6())"]
                            #[block_signal(enable_ipv6_active_notify_handler)]
                            set_active: model.enable_ipv6,

                            connect_active_notify[sender] => move |this| {
                                let _ = sender.output(AppInput::Set(Pref::EnableIPv6(this.is_active())));
                            } @enable_ipv6_active_notify_handler,
                        },
                    },

                    // Kill switch.
                    add = &adw::ActionRow {
                        set_title: &tr!("Kill switch"),
                        add_prefix = &gtk::Image {
                            set_icon_name: Some(icon_names::STOP_SIGN_LARGE),
                        },
                        set_activatable: true,

                        connect_activated[kill_switch_info_button] => move |_| {
                            kill_switch_info_button.info_menu_button.set_active(true);
                        },

                        #[template]
                        #[name = "kill_switch_info_button"]
                        add_suffix =
                            &InfoButton {
                                #[template_child]
                                info_label {
                                    set_label: {
                                        &format!("{}\n\n{}",
                                        &tr!("This built-in feature prevents your traffic from leaking outside of the VPN tunnel if your network suddenly stops working or if the tunnel fails, it does this by blocking your traffic until your connection is reestablished."),
                                        &tr!("The difference between the Kill Switch and Lockdown Mode is that the Kill Switch will prevent any leaks from happening during automatic tunnel reconnects, software crashes and similar accidents. With Lockdown Mode enabled, you must be connected to a Mullvad VPN server to be able to reach the internet. Manually disconnecting or quitting the app will block your connection.")
                                    )
                                },
                            }
                        },

                        add_suffix = &gtk::Switch {
                            set_valign: gtk::Align::Center,
                            set_active: true,
                            set_sensitive: false,
                        },
                    },

                    // Lockdown mode.
                    add = &adw::ActionRow {
                        set_title: &tr!("Lockdown mode"),
                        set_activatable: true,

                        add_prefix = &gtk::Image {
                            set_icon_name: Some(icon_names::SHIELD_FULL),
                        },

                        connect_activated[lockdown_mode_switch] => move |_| {
                            if lockdown_mode_switch.get_sensitive() {
                                lockdown_mode_switch.activate();
                            }
                        },

                        #[template]
                        add_suffix = &InfoButton {
                            #[template_child]
                            info_label {
                                set_label: {
                                    &format!("{}\n\n{}",
                                        &tr!("The difference between the Kill Switch and Lockdown Mode is that the Kill Switch will prevent any leaks from happening during automatic tunnel reconnects, software crashes and similar accidents."),
                                        &tr!("With Lockdown Mode enabled, you must be connected to a Mullvad VPN server to be able to reach the internet. Manually disconnecting or quitting the app will block your connection.")
                                    )
                                },
                            }
                        },

                        #[name = "lockdown_mode_switch"]
                        add_suffix = &gtk::Switch {
                            set_valign: gtk::Align::Center,

                            #[track = "model.changed(PreferencesModel::lockdown_mode())"]
                            #[block_signal(lockdown_mode_active_notify_handler)]
                            set_active: model.lockdown_mode,

                            connect_active_notify[sender] => move |this| {
                                if this.is_active() {
                                    sender.input(PreferencesMsg::TryEnableLockdownModeWithAlert);
                                } else {
                                    sender.input(PreferencesMsg::SetLockdownMode(false));
                                }
                            } @lockdown_mode_active_notify_handler,
                        },
                    },

                    // Multihop.
                    add = &adw::ActionRow {
                        set_title: &tr!("Multihop"),
                        set_activatable: true,

                        connect_activated[multihop_not_available_info_button, multihop_switch] => move |_| {
                            if multihop_switch.get_sensitive() {
                                multihop_switch.activate();
                            } else {
                                multihop_not_available_info_button.info_menu_button.set_active(true);
                            }
                        },

                        add_prefix = &gtk::Image {
                            set_icon_name: Some(icon_names::FUNCTION_THIRD_ORDER_HORIZONTAL),
                        },

                        #[template]
                        #[name = "multihop_not_available_info_button"]
                        add_suffix = &InfoButton {
                            #[template_child]
                            info_menu_button {
                                set_icon_name: icon_names::WARNING_OUTLINE,
                            },

                            #[template_child]
                            info_label {
                                set_label: {
                                    &tr!("Switch Tunnel protocol to “Wireguard” or “Automatic” to make Multihop available.")
                                },
                            },

                            #[track = "model.changed(PreferencesModel::settings())"]
                            set_visible: !model.is_multihop_allowed(),
                        },

                        #[template]
                        add_suffix = &InfoButton {
                            #[template_child]
                            info_label {
                                set_label: {
                                    &tr!("Multihop routes your traffic into one WireGuard server and out another, making it harder to trace. This results in increased latency but increases anonymity online.")
                                },
                            },
                        },

                        #[name = "multihop_switch"]
                        add_suffix = &gtk::Switch {
                            set_valign: gtk::Align::Center,

                            #[track = "model.changed(PreferencesModel::settings())"]
                            #[block_signal(multihop_active_notify_handler)]
                            set_active: model.is_multihop_enabled(),

                            connect_active_notify[sender] => move |this| {
                                sender.input(PreferencesMsg::SetMultihop(this.is_active()));
                            } @multihop_active_notify_handler,

                            #[track = "model.changed(PreferencesModel::settings())"]
                            set_sensitive: model.is_multihop_allowed(),
                        },
                    },
                },

                // Tunnel protocol.
                add = &adw::PreferencesGroup {
                    set_title: &tr!("Tunnel protocol"),

                    #[local_ref]
                    add = tunnel_protocol_selector -> gtk::ListBox {
                        add_css_class: "boxed-list"
                    }
                },

                // WireGuard port.
                add = &adw::PreferencesGroup {
                    set_title: &tr!("WireGuard port"),

                    #[local_ref]
                    add = wireguard_port_selector -> gtk::ListBox {
                        add_css_class: "boxed-list"
                    },

                    #[template]
                    #[wrap(Some)]
                    set_header_suffix = &InfoButton {
                        #[template_child]
                        info_label {
                            set_label: {
                                &format!("{}\n\n{}",
                                    &tr!("The automatic setting will randomly choose from the valid port ranges shown below."),
                                    &tr!("The custom port can be any value inside the valid ranges: {}.", ALLOWED_WIRE_GUARD_PORTS)
                                )
                            },
                        }
                    },
                }
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let tunnel_protocol_selector = VariantSelector::<TunnelProtocol>::builder()
            .launch(TunnelProtocol::get_all_variants())
            .forward(sender.input_sender(), PreferencesMsg::TunnelProtocolChanged);

        let wireguard_port_selector = VariantSelector::<WireGuardPort>::builder()
            .launch(WireGuardPort::get_all_variants())
            .forward(sender.input_sender(), PreferencesMsg::WireGuardPortChanged);

        let model = PreferencesModel {
            window: root.clone(),
            parent: init.parent,
            tunnel_protocol_selector,
            wireguard_port_selector,
            auto_connect: false,
            enable_ipv6: false,
            local_network_sharing: false,
            lockdown_mode: false,
            settings: None,

            tracker: Default::default(),
        };

        let tunnel_protocol_selector = model.tunnel_protocol_selector.widget();
        let wireguard_port_selector = model.wireguard_port_selector.widget();

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, sender: AsyncComponentSender<Self>) {
        self.reset();

        log::debug!("PreferencesMsg: {message:#?}");

        match message {
            PreferencesMsg::Show => self.window.present(self.parent.as_ref()),
            PreferencesMsg::Close => self.window.force_close(),
            PreferencesMsg::UpdateSettings(settings) => {
                self.set_auto_connect(settings.auto_connect);
                self.set_local_network_sharing(settings.allow_lan);
                self.set_lockdown_mode(settings.block_when_disconnected);
                self.set_enable_ipv6(settings.tunnel_options.generic.enable_ipv6);
                self.set_settings(Some(*settings));

                self.tunnel_protocol_selector
                    .emit(VariantSelectorMsg::SelectVariant(
                        self.get_tunnel_protocol(),
                    ));

                self.tunnel_protocol_selector
                    .emit(VariantSelectorMsg::SetVariantMode(
                        TunnelProtocol::OpenVPN.get_id(),
                        {
                            let mut need_to_disable_these_settings = vec![];

                            if self.is_daita_enabled() {
                                need_to_disable_these_settings.push(tr!("DAITA"));
                            }
                            if self.is_multihop_enabled() {
                                need_to_disable_these_settings.push(tr!("Multihop"));
                            }
                            if self.is_quantum_resistant_enabled() {
                                need_to_disable_these_settings
                                    .push(tr!("Quantum-resistant tunnel"));
                            }

                            if need_to_disable_these_settings.is_empty() {
                                VariantMode::Enabled
                            } else {
                                VariantMode::DisabledWithInfo(tr!(
                                    "To select OpenVPN, please disable these settings: {}.",
                                    need_to_disable_these_settings.join(", ")
                                ))
                            }
                        },
                    ));

                self.wireguard_port_selector
                    .emit(VariantSelectorMsg::SelectVariant(self.get_wireguard_port()));
            }
            PreferencesMsg::TunnelProtocolChanged(tunnel_protocol) => {
                self.update_normal_relay_constraints(sender, |relay_constraints| {
                    relay_constraints.tunnel_protocol = tunnel_protocol.into()
                });
            }
            PreferencesMsg::WireGuardPortChanged(port) => {
                self.update_normal_relay_constraints(sender, |relay_constraints| {
                    relay_constraints.wireguard_constraints.port = port.into()
                });
            }
            PreferencesMsg::SetMultihop(value) => {
                self.update_normal_relay_constraints(sender, |relay_constraints| {
                    relay_constraints.wireguard_constraints.use_multihop(value)
                });
            }
            PreferencesMsg::TryEnableLockdownModeWithAlert => {
                let lockdown_alertdialog =
                    adw::AlertDialog::builder()
                    .body(tr!("Attention: enabling this will always require a Mullvad VPN connection in order to reach the internet.\n\nThe app’s built-in kill switch is always on. This setting will additionally block the internet if clicking Disconnect or Quit."))
                    .build();
                lockdown_alertdialog
                    .add_responses(&[("back", &tr!("Back")), ("enable", &tr!("Enable anyway"))]);
                lockdown_alertdialog
                    .set_response_appearance("enable", adw::ResponseAppearance::Destructive);
                lockdown_alertdialog.set_default_response(Some("back"));
                lockdown_alertdialog.connect_response(None, move |_, response| {
                    sender.input(PreferencesMsg::SetLockdownMode(response == "enable"));
                });
                lockdown_alertdialog.present(Some(&self.window));
            }
            PreferencesMsg::SetLockdownMode(value) => {
                self.set_lockdown_mode(value);
                self.tracker |= Self::lockdown_mode(); // Force update the tracker to correctly update the view.
                let _ = sender.output(AppInput::Set(Pref::LockdownMode(value)));
            }
        }
    }
}
