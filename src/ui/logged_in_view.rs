use crate::tr;

use adw::prelude::*;
use relm4::prelude::*;

#[relm4::widget_template(pub)]
impl WidgetTemplate for LoggedInView {
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_valign: gtk::Align::Fill,
            set_margin_all: 20,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_halign: gtk::Align::Fill,

                #[name = "device_name_label"]
                gtk::Label {
                    set_css_classes: &["caption"],
                    set_use_markup: true,
                    set_hexpand: true,
                    set_margin_end: 10,
                    set_halign: gtk::Align::Start,
                },

                #[name = "time_left_label"]
                gtk::Label {
                    set_css_classes: &["caption"],
                    set_use_markup: true,
                    set_halign: gtk::Align::End,
                }
            },

            #[template]
            #[name = "tunnel_state_view"]
            TunnelStateView {},

            #[name = "tunnel_state_label"]
            gtk::Label {
                set_margin_bottom: 10,
                set_css_classes: &["title-4"],
                set_wrap: true,
                set_halign: gtk::Align::Start
            },

            #[name = "country_label"]
            gtk::Label {
                set_margin_bottom: 0,
                set_css_classes: &["title-1"],
                set_wrap: true,
                set_halign: gtk::Align::Start,
            },

            #[name = "city_label"]
            gtk::Label {
                set_margin_bottom: 20,
                set_css_classes: &["title-1"],
                set_wrap: true,
                set_halign: gtk::Align::Start
            },

            #[name = "hostname_listbox"]
            gtk::ListBox {
                add_css_class: "boxed-list",
                set_selection_mode: gtk::SelectionMode::None,
                set_margin_bottom: 20,

                #[name = "hostname_expander_row"]
                adw::ExpanderRow {
                    #[name = "tunnel_protocol_row"]
                    add_row = &adw::ActionRow {
                        set_title: &tr!("Tunnel protocol"),
                        set_css_classes: &["property", "monospace"],
                    },

                    #[name = "tunnel_in_row"]
                    add_row = &adw::ActionRow {
                        set_title: &tr!("In"),
                        set_css_classes: &["property", "monospace"],
                        set_subtitle_selectable: true,
                    },

                    #[name = "tunnel_out_row"]
                    add_row = &adw::ActionRow {
                        set_title: &tr!("Out"),
                        set_css_classes: &["property", "monospace"],
                        set_subtitle_selectable: true,
                    },
                },
            },

            // Connection buttons box.
            gtk::Box {
                add_css_class: "linked",
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::End,
                set_vexpand: true,
                set_width_request: 300,

                #[name = "secure_my_connection_button"]
                gtk::Button {
                    set_hexpand: true,
                    set_label: &tr!("Secure my connection"),
                    set_css_classes: &["opaque", "secure_my_connection_btn"],
                },

                #[name = "cancel_button"]
                gtk::Button {
                    set_hexpand: true,
                    set_label: &tr!("Cancel"),
                    set_css_classes: &["opaque", "disconnect_btn"],
                },

                #[name = "disconnect_button"]
                gtk::Button {
                    set_hexpand: true,
                    set_label: &tr!("Disconnect"),
                    set_css_classes: &["opaque", "disconnect_btn"],
                },

                #[name = "reconnect_button"]
                gtk::Button {
                    set_css_classes: &["opaque", "reconnect_btn"],
                    set_icon_name: "arrow-circular-top-right-symbolic",
                },
            }
        }
    }
}

#[relm4::widget_template(pub)]
impl WidgetTemplate for TunnelStateView {
    view! {
        adw::Bin {
            set_height_request: 128,
            set_width_request: 128,
            set_margin_all: 16,
            set_halign: gtk::Align::Center,

            #[name = "view_stack"]
            gtk::Stack {
                add_named[Some("connected")] = &gtk::Image {
                    set_icon_name: Some("network-vpn-symbolic"),
                    set_css_classes: &[
                        "connection_state_icon",
                        "connected",
                        "icon-dropshadow"
                    ]
                },

                add_named[Some("connecting")] = &gtk::Spinner {
                    set_spinning: true,
                    set_height_request: 64,
                    set_width_request: 64,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                },

                add_named[Some("disabled")] = &gtk::Image {
                    set_icon_name: Some("network-vpn-disabled-symbolic"),
                    set_css_classes: &[
                        "connection_state_icon",
                        "disabled",
                        "icon-dropshadow"
                    ]
                },

                add_named[Some("disconnected")] = &gtk::Image {
                    set_icon_name: Some("network-vpn-disconnected-symbolic"),
                    set_css_classes: &[
                        "connection_state_icon",
                        "disconnected",
                        "icon-dropshadow"
                    ]
                },

                add_named[Some("_")] = &gtk::Label {}
            }
        }
    }
}
