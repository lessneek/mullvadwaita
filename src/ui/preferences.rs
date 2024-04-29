use adw::prelude::*;
use relm4::prelude::*;

use mullvad_types::{
    relay_constraints::{Constraint, RelaySettings},
    settings::Settings,
};

use talpid_types::net::TunnelType;

use crate::{
    tr,
    ui::{
        app::AppInput,
        radio_buttons_list::{
            RadioButtonsList, RadioButtonsListMsg, RadioButtonsListVariant, VariantType,
        },
        widgets::InfoButton,
    },
};

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
    window: adw::PreferencesWindow,

    #[no_eq]
    tunnel_protocol_group: Controller<RadioButtonsList<TunnelProtocol>>,

    local_network_sharing: bool,
    lockdown_mode: bool,
    enable_ipv6: bool,
    auto_connect: bool,
    relay_settings: Option<RelaySettings>,
}

#[derive(Debug)]
pub enum PreferencesMsg {
    Show,
    Close,
    UpdateSettings(Settings),
    TunnelProtocolChanged(TunnelProtocol),
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
    fn get_tunnel_protocol(&self) -> Option<TunnelProtocol> {
        if let Some(RelaySettings::Normal(relay_constraints)) = self.get_relay_settings() {
            return Some(relay_constraints.tunnel_protocol.into());
        }
        None
    }
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for PreferencesModel {
    type Init = ();
    type Input = PreferencesMsg;
    type Output = AppInput;
    type Widgets = PreferencesWidgets;

    view! {
        adw::PreferencesWindow {
            connect_close_request[sender] => move |_| {
                sender.input(PreferencesMsg::Close);
                gtk::glib::Propagation::Stop
            },
            add = &adw::PreferencesPage {
                add = &adw::PreferencesGroup {
                    set_title: &tr!("VPN"),

                    // Auto-connect.
                    add = &adw::SwitchRow {
                        set_title: &tr!("Auto-connect"),
                        add_prefix = &gtk::Image {
                            set_icon_name: Some("network-vpn-symbolic"),
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
                    add = &adw::SwitchRow {
                        set_title: &tr!("Local network sharing"),
                        add_prefix = &gtk::Image {
                            set_icon_name: Some("network-workgroup-symbolic"),
                        },

                        #[track = "model.changed(PreferencesModel::local_network_sharing())"]
                        #[block_signal(local_network_sharing_active_notify_handler)]
                        set_active: model.local_network_sharing,

                        connect_active_notify[sender] => move |this| {
                            let _ = sender.output(AppInput::Set(Pref::LocalNetworkSharing(this.is_active())));
                        } @local_network_sharing_active_notify_handler,

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
                    },

                    // Enable IPv6.
                    add = &adw::SwitchRow {
                        set_title: &tr!("Enable IPv6"),
                        add_prefix = &gtk::Image {
                            set_icon_name: Some("globe-alt2-symbolic"),
                        },

                        #[track = "model.changed(PreferencesModel::enable_ipv6())"]
                        #[block_signal(enable_ipv6_active_notify_handler)]
                        set_active: model.enable_ipv6,

                        connect_active_notify[sender] => move |this| {
                            let _ = sender.output(AppInput::Set(Pref::EnableIPv6(this.is_active())));
                        } @enable_ipv6_active_notify_handler,

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
                    },

                    // Kill switch.
                    add = &adw::ActionRow {
                        set_title: &tr!("Kill switch"),
                        add_prefix = &gtk::Image {
                            set_icon_name: Some("stop-sign-large-symbolic"),
                        },
                        set_activatable: true,

                        connect_activated[kill_switch_info_button] => move |_| {
                            kill_switch_info_button.info_menu_button.set_active(true);
                        },

                        add_suffix = &gtk::Box {
                            gtk::Switch {
                                set_valign: gtk::Align::Center,
                                set_margin_end: 7,
                                set_active: true,
                                set_sensitive: false,
                            },

                            #[template]
                            #[name = "kill_switch_info_button"]
                            InfoButton {
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
                        }
                    },

                    // Lockdown mode.
                    add = &adw::SwitchRow {
                        set_title: &tr!("Lockdown mode"),
                        add_prefix = &gtk::Image {
                            set_icon_name: Some("security-high-symbolic"),
                        },

                        #[track = "model.changed(PreferencesModel::lockdown_mode())"]
                        #[block_signal(lockdown_mode_active_notify_handler)]
                        set_active: model.lockdown_mode,

                        connect_active_notify[sender] => move |this| {
                            let _ = sender.output(AppInput::Set(Pref::LockdownMode(this.is_active())));
                        } @lockdown_mode_active_notify_handler,

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
                    },
                },

                // Tunnel protocol.
                add = &adw::PreferencesGroup {
                    set_title: &tr!("Tunnel protocol"),

                    #[local_ref]
                    add = tunnel_protocol_list -> gtk::ListBox {
                        add_css_class: "boxed-list"
                    }
                }
            }
        }
    }

    async fn init(
        _: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let tunnel_protocol_group = RadioButtonsList::<TunnelProtocol>::builder()
            .launch(TunnelProtocol::get_all_variants())
            .forward(sender.input_sender(), PreferencesMsg::TunnelProtocolChanged);

        let model = PreferencesModel {
            window: root.clone(),
            tunnel_protocol_group,

            auto_connect: false,
            enable_ipv6: false,
            local_network_sharing: false,
            lockdown_mode: false,
            relay_settings: None,

            tracker: Default::default(),
        };

        let tunnel_protocol_list = model.tunnel_protocol_group.widget();
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, sender: AsyncComponentSender<Self>) {
        self.reset();

        log::debug!("PreferencesMsg: {message:#?}");

        match message {
            PreferencesMsg::Show => self.window.present(),
            PreferencesMsg::Close => self.window.set_visible(false),
            PreferencesMsg::UpdateSettings(settings) => {
                self.set_auto_connect(settings.auto_connect);
                self.set_local_network_sharing(settings.allow_lan);
                self.set_lockdown_mode(settings.block_when_disconnected);
                self.set_enable_ipv6(settings.tunnel_options.generic.enable_ipv6);
                self.set_relay_settings(Some(settings.relay_settings));

                self.tunnel_protocol_group
                    .emit(RadioButtonsListMsg::SelectVariant(
                        self.get_tunnel_protocol(),
                    ));
            }
            PreferencesMsg::TunnelProtocolChanged(pref) => {
                if let Some(RelaySettings::Normal(relay_constraints)) =
                    self.get_mut_relay_settings()
                {
                    relay_constraints.tunnel_protocol = pref.into();

                    sender
                        .output(AppInput::Set(Pref::RelaySettings(Box::new(
                            RelaySettings::Normal(relay_constraints.clone()),
                        ))))
                        .ok();
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TunnelProtocol {
    Automatic,
    WireGuard,
    OpenVPN,
}

impl From<Constraint<TunnelType>> for TunnelProtocol {
    fn from(value: Constraint<TunnelType>) -> Self {
        match value {
            Constraint::Any => TunnelProtocol::Automatic,
            Constraint::Only(TunnelType::Wireguard) => TunnelProtocol::WireGuard,
            Constraint::Only(TunnelType::OpenVpn) => TunnelProtocol::OpenVPN,
        }
    }
}

impl From<TunnelProtocol> for Constraint<TunnelType> {
    fn from(val: TunnelProtocol) -> Self {
        match val {
            TunnelProtocol::Automatic => Constraint::Any,
            TunnelProtocol::WireGuard => Constraint::Only(TunnelType::Wireguard),
            TunnelProtocol::OpenVPN => Constraint::Only(TunnelType::OpenVpn),
        }
    }
}

impl TunnelProtocol {
    fn get_all_variants() -> Vec<TunnelProtocol> {
        use TunnelProtocol::*;
        vec![Automatic, WireGuard, OpenVPN]
    }
}

impl RadioButtonsListVariant for TunnelProtocol {
    fn get_variant_type(&self) -> VariantType<Self> {
        use TunnelProtocol::*;
        match self {
            Automatic => VariantType::Label(tr!("Automatic")),
            WireGuard => VariantType::Label(tr!("WireGuard")),
            OpenVPN => VariantType::Label(tr!("OpenVPN")),
        }
    }
}

#[derive(Debug, Clone, Copy)]
