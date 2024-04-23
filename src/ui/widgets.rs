use adw::prelude::*;
use relm4::prelude::*;

#[relm4::widget_template(pub)]
impl WidgetTemplate for InfoButton {
    view! {
        #[name = "info_menu_button"]
        gtk::MenuButton {
            set_icon_name: "info-outline-symbolic",
            set_valign: gtk::Align::Center,
            set_css_classes: &["flat"],

            #[wrap(Some)]
            set_popover = &gtk::Popover {
                set_position: gtk::PositionType::Bottom,

                gtk::ScrolledWindow {
                    set_propagate_natural_height: true,
                    set_propagate_natural_width: true,

                    #[name = "info_label"]
                    gtk::Label {
                        set_wrap: true,
                        set_max_width_chars: 42,
                        set_width_request: 300,
                    }
                }
            }
        }
    }
}
