mod extensions;
mod macros;
mod mullvad;
mod ui;

use ui::app::AppModel;

use anyhow::Result;
use relm4::RelmApp;
use tr::tr;

mod icon_names {
    include!(concat!(env!("OUT_DIR"), "/icon_names.rs"));
}

fn init_logger() -> Result<(), log::SetLoggerError> {
    env_logger::try_init()
}

fn init_gettext(
) -> Result<Vec<i18n_embed::unic_langid::LanguageIdentifier>, i18n_embed::I18nEmbedError> {
    use i18n_embed::{gettext::gettext_language_loader, DesktopLanguageRequester};

    use rust_embed::RustEmbed;

    #[derive(RustEmbed)]
    #[folder = "i18n/mo"] // path to the compiled localization resources
    struct Translations;

    i18n_embed::select(
        &gettext_language_loader!(),
        &Translations {},
        &DesktopLanguageRequester::requested_languages(),
    )
}

fn main() -> Result<()> {
    init_logger()?;
    log::debug!("mullvadwaita starting...");
    init_gettext()?;

    let app = RelmApp::new("draft.mullvadwaita");
    relm4::set_global_css(include_str!("./res/global.css"));

    relm4_icons::initialize_icons(icon_names::GRESOURCE_BYTES, icon_names::RESOURCE_PREFIX);

    app.run_async::<AppModel>(());

    Ok(())
}
