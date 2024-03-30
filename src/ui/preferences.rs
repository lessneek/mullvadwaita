use super::app::AppMsg;
use adw::prelude::*;
use relm4::{
    component::{AsyncComponentParts, SimpleAsyncComponent},
    *,
};

#[tracker::track]
#[derive(Debug)]
pub struct PreferencesModel {
    window: adw::PreferencesWindow,
}

#[derive(Debug)]
pub enum PreferencesMsg {
    Show,
    Close,
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for PreferencesModel {
    type Init = ();
    type Input = PreferencesMsg;
    type Output = AppMsg;
    type Widgets = PreferencesWidgets;

    view! {
        adw::PreferencesWindow {
            connect_close_request[sender] => move |_| {
                sender.input(PreferencesMsg::Close);
                gtk::glib::Propagation::Stop
            },
        }
    }

    async fn init(
        _: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = PreferencesModel {
            window: root.clone(),
            tracker: 0,
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, _sender: AsyncComponentSender<Self>) {
        self.reset();

        match message {
            PreferencesMsg::Show => self.window.present(),
            PreferencesMsg::Close => self.window.set_visible(false),
        }
    }
}
