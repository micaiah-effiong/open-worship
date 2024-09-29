use relm4::gtk::prelude::*;
use relm4::{gtk, typed_view::list::RelmListItem, view};

#[derive(Debug, Clone)]
pub struct EditSongModalListItem {
    pub text: String,
}

pub struct EditSongModalListItemWidget {
    label: gtk::TextView,
}

impl Drop for EditSongModalListItemWidget {
    fn drop(&mut self) {
        self.label.buffer();
    }
}

impl RelmListItem for EditSongModalListItem {
    type Root = gtk::Box;
    type Widgets = EditSongModalListItemWidget;

    fn setup(_list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        view! {
            list_view = gtk::Box{
                #[name="list_item_label"]
                gtk::TextView {
                    set_editable: true,
                    set_hexpand:true,
                    set_vexpand: true,
                }
            }
        }

        // list_item_label.set_wra(mode)

        let widgets = EditSongModalListItemWidget {
            label: list_item_label,
        };

        return (list_view, widgets);
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        widgets.label.buffer().set_text(&self.text);
    }
}
