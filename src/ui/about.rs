use crate::icon_names;

use adw::prelude::*;
use relm4::prelude::*;

pub fn show_about_dialog(root: &impl IsA<gtk::Window>) {
    let dialog = adw::AboutWindow::builder()
        .icon_name(icon_names::BACKGROUND_APP_GHOST)
        .application_icon(icon_names::BACKGROUND_APP_GHOST)
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
