use relm4::gtk::prelude::*;
use relm4::{gtk, typed_view::list::RelmListItem, view};

#[derive(Debug, Clone)]
pub struct ActivityListItem {
    pub text: String,
}

pub struct ActivityListItemWidget {
    label: gtk::Label,
}

impl RelmListItem for ActivityListItem {
    type Root = gtk::Box;
    type Widgets = ActivityListItemWidget;

    fn setup(_list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        view! {
            list_view = gtk::Box{
                #[name="list_item_label"]
                gtk::Label {
                    set_ellipsize:gtk::pango::EllipsizeMode::End,
                    set_wrap_mode:gtk::pango::WrapMode::Word,
                    set_lines:2,
                    set_margin_top:12,
                    set_margin_bottom:12,
                    set_halign:gtk::Align::Start,
                    set_justify:gtk::Justification::Fill,
                }
            }
        }

        // list_item_label.set_wra(mode)

        let widgets = ActivityListItemWidget {
            label: list_item_label,
        };

        return (list_view, widgets);
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        widgets.label.set_label(&self.text);
    }
}
