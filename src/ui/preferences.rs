use super::app::AppInput;
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
}

#[derive(Debug)]
pub enum PreferencesMsg {
    Show,
    Close,
    UpdateSettings(Settings),
    Set(Pref),
}

#[derive(Debug)]
pub enum Pref {
    LocalNetworkSharing(bool),
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
                        add_prefix = &gtk::Image {
                            set_icon_name: Some("network-workgroup-symbolic"),
                        },
                        set_title: &tr!("Local network sharing"),
                        set_subtitle: &tr!("This feature allows access to other devices on the local network, such as for sharing, printing, streaming, etc."),

                        #[track = "model.changed(PreferencesModel::local_network_sharing())"]
                        set_active: model.local_network_sharing,

                        connect_active_notify[sender] => move |this| {
                            sender.input(PreferencesMsg::Set(Pref::LocalNetworkSharing(this.is_active())));
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

    async fn update(&mut self, message: Self::Input, sender: AsyncComponentSender<Self>) {
        self.reset();

        match message {
            PreferencesMsg::Show => self.window.present(),
            PreferencesMsg::Close => self.window.set_visible(false),
            PreferencesMsg::UpdateSettings(settings) => {
                self.set_local_network_sharing(settings.allow_lan);
            }
            PreferencesMsg::Set(pref) => {
                if let Some(err) = sender.output(AppInput::Set(pref)).err() {
                    log::error!("{err:#?}");
                };
            }
        }
    }
}
