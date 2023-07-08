use std::time::Duration;

use relm4::{
    adw,
    component::{AsyncComponent, AsyncComponentParts},
    gtk::{
        prelude::{ButtonExt, GtkWindowExt},
        traits::{OrientableExt, WidgetExt},
        Align::*,
    },
    loading_widgets::LoadingWidgets,
    view, AsyncComponentSender, RelmApp, RelmWidgetExt,
};

use relm4_icons::icon_name;

use Status::*;

#[derive(Debug)]
enum AppInput {
    SwitchConnection,
    Reconnect,
}

#[derive(PartialEq, Debug)]
enum Status {
    Connected,
    Disconnected,
    Connecting,
}

#[tracker::track]
struct AppModel {
    status: Status,
}

#[relm4::component(async)]
impl AsyncComponent for AppModel {
    type Init = Status;
    type Input = AppInput;
    type Output = ();
    type CommandOutput = ();

    view! {
        adw::Window {
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    // set_width_request: 300,

                    gtk::Label {
                        #[track = "model.changed(AppModel::status())"]
                        set_label: &format!("State: {:?}", model.status),
                        set_margin_all: 5,
                    },

                    gtk::Box {
                        add_css_class: "linked",
                        set_margin_all: 20,
                        set_halign: Center,
                        set_valign: End,
                        set_vexpand: true,
                        set_width_request: 300,

                        gtk::Button {
                            set_hexpand: true,

                            #[track = "model.changed(AppModel::status())"]
                            set_label: {
                                match model.status {
                                    Disconnected => "Secure my connection",
                                    Connected => "Disconnect",
                                    Connecting => "Cancel"
                                }
                            },

                            #[track = "model.changed(AppModel::status())"]
                            set_css_classes: {
                                match model.status {
                                    Disconnected => &["suggested-action"],
                                    Connected | Connecting => &["destructive-action"],
                                }
                            },

                            connect_clicked => AppInput::SwitchConnection
                        },

                        gtk::Button {
                            #[track = "model.changed(AppModel::status())"]
                            set_visible: model.status == Connected,
                            connect_clicked => AppInput::Reconnect,
                            set_css_classes: &["suggested-action"],
                            set_icon_name: icon_name::REFRESH_LARGE
                        },
                    }
                }
            }
        }
    }

    fn init_loading_widgets(root: &mut Self::Root) -> Option<LoadingWidgets> {
        view! {
            #[local_ref]
            root {
                set_title: Some("Mullvadwaita"),
                set_default_size: (400, 600),

                // This will be removed automatically by
                // LoadingWidgets when the full view has loaded
                #[name(spinner)]
                gtk::Spinner {
                    start: (),
                    set_halign: gtk::Align::Center,
                }
            }
        }
        Some(LoadingWidgets::new(root, spinner))
    }

    async fn init(
        status: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        tokio::time::sleep(Duration::from_secs(1)).await;

        let model = AppModel { status, tracker: 0 };
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        self.reset();

        self.set_status(match message {
            AppInput::SwitchConnection => match self.get_status() {
                Connected | Connecting => Disconnected,
                Disconnected => Connected,
            },
            AppInput::Reconnect => Status::Connecting,
        })
    }
}

fn main() {
    let app = RelmApp::new("draft.mullvadwaita");
    relm4_icons::initialize_icons();
    app.run_async::<AppModel>(Status::Disconnected);
}
