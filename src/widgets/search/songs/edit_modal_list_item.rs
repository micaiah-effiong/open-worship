use relm4::gtk::{prelude::*, TextBuffer};
use relm4::RelmWidgetExt;
use relm4::{gtk, typed_view::list::RelmListItem, view};

#[derive(Debug, Clone)]
pub struct EditSongModalListItem {
    pub text_buffer: TextBuffer,
}

pub struct EditSongModalListItemWidget {
    text_view: gtk::TextView,
}

impl Drop for EditSongModalListItemWidget {
    fn drop(&mut self) {
        // self.label.buffer();
    }
}

impl RelmListItem for EditSongModalListItem {
    type Root = gtk::TextView;
    type Widgets = EditSongModalListItemWidget;

    fn setup(_list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        view! {
            list_view = gtk::TextView{
                set_margin_all: 8,
                set_hexpand: true,
                set_editable: true,
                set_height_request: 40,
            },
        }

        let widgets = EditSongModalListItemWidget {
            text_view: list_view.clone(),
        };

        (list_view, widgets)
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        widgets.text_view.set_buffer(Some(&self.text_buffer));
    }
}
