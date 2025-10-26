use crate::icon_names;
use relm4::prelude::*;

pub fn build_about_dialog() -> adw::AboutDialog {
    adw::AboutDialog::builder()
        .application_icon(icon_names::shipped::BACKGROUND_APP_GHOST)
        .application_name("Mullvadwaita")
        .developer_name("Lessneek")
        .website("Website")
        .copyright("© 2024 Lessneek")
        .license_type(gtk::License::Gpl30)
        .website("https://github.com/lessneek/mullvadwaita")
        .issue_url("https://github.com/lessneek/mullvadwaita/issues")
        .version(env!("CARGO_PKG_VERSION"))
        .developers(vec!["Lessneek"])
        .comments("Mullvad VPN daemon controller.")
        .presentation_mode(adw::DialogPresentationMode::BottomSheet)
        .build()
}
