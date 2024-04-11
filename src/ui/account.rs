use super::app::AppInput;
use crate::tr;
use adw::prelude::*;
use chrono::Local;
use mullvad_types::{account::AccountData, device::AccountAndDevice};
use relm4::{
    component::{AsyncComponentParts, SimpleAsyncComponent},
    *,
};

use smart_default::SmartDefault;

#[tracker::track]
#[derive(Debug, SmartDefault)]
pub struct AccountModel {
    window: adw::PreferencesWindow,

    device_name: String,
    account_number: String,
    paid_until: String,
}

#[derive(Debug)]
pub enum AccountMsg {
    Show,
    Close,
    UpdateAccountAndDevice(AccountAndDevice),
    UpdateAccountData(AccountData),
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for AccountModel {
    type Init = ();
    type Input = AccountMsg;
    type Output = AppInput;
    type Widgets = PreferencesWidgets;

    view! {
        adw::PreferencesWindow {
            set_title: Some(&tr!("Account")),
            set_search_enabled: false,
            connect_close_request[sender] => move |_| {
                sender.input(AccountMsg::Close);
                gtk::glib::Propagation::Stop
            },
            add = &adw::PreferencesPage {
                add = &adw::PreferencesGroup {
                    add = &adw::ActionRow {
                        set_title: &tr!("Device name"),

                        #[track = "model.changed(AccountModel::device_name())"]
                        set_subtitle: model.get_device_name(),

                        set_css_classes: &["property"],
                        set_subtitle_selectable: true,

                        add_suffix = &gtk::Button {
                            set_icon_name: "info-outline-symbolic",
                            set_valign: gtk::Align::Center,
                            set_css_classes: &["flat"],
                            connect_clicked[root] => move |_| {
                                gtk::AlertDialog::builder()
                                    .message(tr!("This is the name assigned to the device. Each device logged in on a Mullvad account gets a unique name that helps you identify it when you manage your devices in the app or on the website.\n\nYou can have up to 5 devices logged in on one Mullvad account.\n\nIf you log out, the device and the device name is removed. When you log back in again, the device will get a new name."))
                                    .build()
                                    .show(Some(&root));
                            }
                        },
                    },

                    add: account_number = &adw::PasswordEntryRow {
                        set_title: &tr!("Account number"),

                        #[track = "model.changed(AccountModel::account_number())"]
                        set_text: model.get_account_number(),

                        set_editable: false,

                        add_suffix = &gtk::Button {
                            set_icon_name: "copy-symbolic",
                            set_valign: gtk::Align::Center,
                            set_css_classes: &["flat", "image-button"],
                            connect_clicked[root, account_number] => move |_| {
                                let text = account_number.text();
                                root.primary_clipboard().set_text(text.as_ref());
                                root.clipboard().set_text(text.as_ref());
                            }
                        },
                    },

                    add = &adw::ActionRow {
                        set_title: &tr!("Paid until"),

                        #[track = "model.changed(AccountModel::paid_until())"]
                        set_subtitle: model.get_paid_until(),

                        set_css_classes: &["property"],
                        set_subtitle_selectable: true,
                    },

                },

                add = &adw::PreferencesGroup {
                    add = &gtk::Button {
                        connect_clicked[sender] => move |_| {
                            sender.input(AccountMsg::Close);
                            let _ = sender.output(AppInput::Logout);
                        },
                        set_hexpand: true,
                        set_label: &tr!("Log out"),
                        set_css_classes: &["opaque", "logout_btn"],
                    },
                }
            }
        }
    }

    async fn init(
        _: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = AccountModel {
            window: root.clone(),
            ..Default::default()
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, _sender: AsyncComponentSender<Self>) {
        self.reset();

        match message {
            AccountMsg::Show => self.window.present(),
            AccountMsg::Close => self.window.set_visible(false),
            AccountMsg::UpdateAccountAndDevice(account_and_device) => {
                self.set_device_name(account_and_device.device.pretty_name());
                self.set_account_number(account_and_device.account_token);
            }
            AccountMsg::UpdateAccountData(account_data) => {
                let paid_until = account_data.expiry.with_timezone(Local::now().offset());
                self.set_paid_until(paid_until.naive_local().to_string());
            }
        }
    }
}
