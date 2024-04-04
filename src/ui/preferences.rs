use super::app::AppInput;
use crate::tr;
use adw::prelude::*;
use mullvad_types::settings::Settings;
use relm4::{
    component::{AsyncComponentParts, SimpleAsyncComponent},
    *,
};

use smart_default::SmartDefault;

#[tracker::track]
#[derive(Debug, SmartDefault)]
pub struct PreferencesModel {
    window: adw::PreferencesWindow,

    local_network_sharing: bool,
    lockdown_mode: bool,
    enable_ipv6: bool,
    auto_connect: bool,
}

#[derive(Debug)]
pub enum PreferencesMsg {
    Show,
    Close,
    UpdateSettings(Settings),
}

#[derive(Debug)]
pub enum Pref {
    AutoConnect(bool),
    LocalNetworkSharing(bool),
    LockdownMode(bool),
    EnableIPv6(bool),
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

                    add = &adw::SwitchRow {
                        set_title: &tr!("Auto-connect"),
                        add_prefix = &gtk::Image {
                            set_icon_name: Some("network-vpn-symbolic"),
                        },
                        set_subtitle: &tr!("Automatically connect to a server when the app launches."),

                        #[track = "model.changed(PreferencesModel::auto_connect())"]
                        set_active: model.auto_connect,

                        connect_active_notify[sender] => move |this| {
                            let _ = sender.output(AppInput::Set(Pref::AutoConnect(this.is_active())));
                        }
                    },

                    add = &adw::SwitchRow {
                        set_title: &tr!("Local network sharing"),
                        add_prefix = &gtk::Image {
                            set_icon_name: Some("network-workgroup-symbolic"),
                        },
                        set_subtitle: &tr!("This feature allows access to other devices on the local network, such as for sharing, printing, streaming, etc."),

                        #[track = "model.changed(PreferencesModel::local_network_sharing())"]
                        set_active: model.local_network_sharing,

                        connect_active_notify[sender] => move |this| {
                            let _ = sender.output(AppInput::Set(Pref::LocalNetworkSharing(this.is_active())));
                        }
                    },

                    add = &adw::SwitchRow {
                        set_title: &tr!("Lockdown mode"),
                        add_prefix = &gtk::Image {
                            set_icon_name: Some("security-high-symbolic"),
                        },
                        set_subtitle: &tr!("The difference between the Kill Switch and Lockdown Mode is that the Kill Switch will prevent any leaks from happening during automatic tunnel reconnects, software crashes and similar accidents."),

                        #[track = "model.changed(PreferencesModel::lockdown_mode())"]
                        set_active: model.lockdown_mode,

                        connect_active_notify[sender] => move |this| {
                            let _ = sender.output(AppInput::Set(Pref::LockdownMode(this.is_active())));
                        }
                    },

                    add = &adw::SwitchRow {
                        set_title: &tr!("Enable IPv6"),
                        add_prefix = &gtk::Image {
                            set_icon_name: Some("globe-alt2-symbolic"),
                        },
                        add_suffix = &gtk::Button {
                            set_icon_name: "info-outline-symbolic",
                            set_valign: gtk::Align::Center,
                            set_css_classes: &["flat"],
                            connect_clicked[root] => move |_| {
                                gtk::AlertDialog::builder()
                                    .message(tr!("IPv4 is always enabled and the majority of websites and applications use this protocol. We do not recommend enabling IPv6 unless you know you need it."))
                                    .build()
                                    .show(Some(&root));
                            }
                        },
                        set_subtitle: &tr!("When this feature is enabled, IPv6 can be used alongside IPv4 in the VPN tunnel to communicate with internet services."),

                        #[track = "model.changed(PreferencesModel::enable_ipv6())"]
                        set_active: model.enable_ipv6,

                        connect_active_notify[sender] => move |this| {
                            let _ = sender.output(AppInput::Set(Pref::EnableIPv6(this.is_active())));
                        }
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
        let model = PreferencesModel {
            window: root.clone(),
            ..Default::default()
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, _sender: AsyncComponentSender<Self>) {
        self.reset();

        match message {
            PreferencesMsg::Show => self.window.present(),
            PreferencesMsg::Close => self.window.set_visible(false),
            PreferencesMsg::UpdateSettings(settings) => {
                self.set_auto_connect(settings.auto_connect);
                self.set_local_network_sharing(settings.allow_lan);
                self.set_lockdown_mode(settings.block_when_disconnected);
                self.set_enable_ipv6(settings.tunnel_options.generic.enable_ipv6);
            }
        }
    }
}
