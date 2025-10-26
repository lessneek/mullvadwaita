use crate::icon_names;
use adw::prelude::*;
use relm4::prelude::*;

#[relm4::widget_template(pub)]
impl WidgetTemplate for InfoButton {
    view! {
        #[name = "info_menu_button"]
        gtk::MenuButton {
            set_icon_name: icon_names::shipped::INFO_OUTLINE,
            set_valign: gtk::Align::Center,
            set_css_classes: &["flat"],

            #[wrap(Some)]
            set_popover = &gtk::Popover {
                set_position: gtk::PositionType::Bottom,

                    #[name = "info_label"]
                    gtk::Label {
                        set_wrap: true,
                        set_margin_all: 6,
                        set_max_width_chars: 50,
                        set_width_request: 300,
                    }
            }
        }
    }
}
