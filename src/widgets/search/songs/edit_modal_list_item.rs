use relm4::gtk::prelude::*;
use relm4::RelmWidgetExt;
use relm4::{gtk, typed_view::list::RelmListItem, view};

#[derive(Debug, Clone)]
pub struct EditSongModalListItem {
    pub text: String,
}

pub struct EditSongModalListItemWidget {
    text_buffer: gtk::TextBuffer,
}

impl Drop for EditSongModalListItemWidget {
    fn drop(&mut self) {
        // self.label.buffer();
    }
}

impl RelmListItem for EditSongModalListItem {
    type Root = gtk::Box;
    type Widgets = EditSongModalListItemWidget;

    fn setup(_list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        let text_buffer = gtk::TextBuffer::new(Some(&gtk::TextTagTable::new()));

        view! {
            list_view = gtk::Box{
                set_margin_all: 8,
                gtk::TextView {
                    set_hexpand: true,
                    set_editable: true,
                    set_height_request: 40,

                    set_buffer = Some(&text_buffer),
                },
            }
        }

        let widgets = EditSongModalListItemWidget { text_buffer };

        return (list_view, widgets);
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        widgets.text_buffer.set_text(&self.text);
    }
}
