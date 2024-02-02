mod mullvad;

use log::debug;

use relm4::{
    adw,
    component::Component,
    gtk::{
        prelude::{ButtonExt, GtkWindowExt},
        traits::{OrientableExt, WidgetExt},
        Align::*,
    },
    ComponentParts, ComponentSender, RelmApp, RelmWidgetExt,
};

use relm4_icons::icon_name;

use futures::FutureExt;

use mullvad::Status::{self, *};

#[derive(Debug)]
enum AppInput {
    SwitchConnection,
    Reconnect,
}

#[derive(Debug)]
enum AppMsg {
    StatusChanged(Status),
}

#[tracker::track]
struct AppModel {
    status: Status,
}

#[relm4::component]
impl Component for AppModel {
    type Init = ();
    type Input = AppInput;
    type Output = ();
    type CommandOutput = AppMsg;

    view! {
        adw::Window {
            set_title: Some("Mullvadwaita"),
            set_default_size: (300, 600),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    gtk::Label {
                        #[track = "model.changed(AppModel::status())"]
                        set_label: &format!("{:?}", model.status),
                        set_margin_all: 5,
                        add_css_class: "title-1"
                    },

                    gtk::Box {
                        add_css_class: "linked",
                        set_margin_all: 20,
                        set_halign: Center,
                        set_valign: End,
                        set_vexpand: true,
                        set_width_request: 300,

                        gtk::Button {
                            connect_clicked => AppInput::SwitchConnection,
                            set_hexpand: true,

                            #[track = "model.changed(AppModel::status())"]
                            set_label: {
                                match model.status {
                                    Disconnected => "Secure my connection",
                                    Connected => "Disconnect",
                                    Connecting => "Cancel",
                                    _ => "_"
                                }
                            },

                            #[track = "model.changed(AppModel::status())"]
                            set_css_classes: {
                                match model.status {
                                    Disconnected => &["suggested-action"],
                                    Connected | Connecting => &["destructive-action"],
                                    _ => &[]
                                }
                            },

                            #[track = "model.changed(AppModel::status())"]
                            set_visible: model.status != WaitingForService,
                        },

                        gtk::Button {
                            connect_clicked => AppInput::Reconnect,

                            #[track = "model.changed(AppModel::status())"]
                            set_visible: matches!(model.status, Connected | Connecting),

                            set_css_classes: &["suggested-action"],
                            set_icon_name: icon_name::REFRESH_LARGE
                        },
                    }
                }
            }
        }
    }

    fn init(
        _: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        sender.command(|out, shutdown| {
            shutdown
                // Performs this operation until a shutdown is triggered
                .register(async move {
                    let mut status_rx = mullvad::watch();

                    debug!("Listening for status updates...");

                    while (status_rx.changed().await).is_ok() {
                        let status = *status_rx.borrow_and_update();
                        debug!("Status changed: {:?}", status);
                        out.send(AppMsg::StatusChanged(status)).unwrap();
                    }

                    debug!("Status updates stopped.");
                })
                // Perform task until a shutdown interrupts it
                .drop_on_shutdown()
                // Wrap into a `Pin<Box<Future>>` for return
                .boxed()
        });

        let model = AppModel {
            status: WaitingForService,
            tracker: 0,
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        self.reset();

        self.set_status(match message {
            AppInput::SwitchConnection => match self.get_status() {
                Connected | Connecting => Disconnected,
                Disconnected => Connected,
                WaitingForService => WaitingForService,
            },
            AppInput::Reconnect => Status::Connecting,
        })
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            AppMsg::StatusChanged(status) => self.set_status(status),
        }
    }
}

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new().init().unwrap();

    debug!("mullvadwaita starting...");

    tokio::task::spawn_blocking(|| {
        let app = RelmApp::new("draft.mullvadwaita");
        relm4_icons::initialize_icons();
        app.run::<AppModel>(());
    });
}
