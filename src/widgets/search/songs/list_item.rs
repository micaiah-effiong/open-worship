use relm4::{
    gtk::{self},
    typed_view::list::RelmListItem,
    view,
};

use crate::dto::Song;

/// song search list item
#[derive(Debug, Clone)]
pub struct SongListItemModel {
    pub song: Song,
}

impl SongListItemModel {
    pub fn new(song: Song) -> Self {
        SongListItemModel { song }
    }
    // pub fn screen_display(&self) -> String {
    //     let text = format!("{}\n{}", self.tag, self.text);
    //     return text;
    // }
}

pub struct SongListItemWidget {
    text: gtk::Label,
}

impl Drop for SongListItemWidget {
    fn drop(&mut self) {
        self.text.label();
    }
}

impl RelmListItem for SongListItemModel {
    type Root = gtk::Box;
    type Widgets = SongListItemWidget;

    fn setup(_list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        view! {
            list_box = gtk::Box {
                #[name="text"]
                gtk::Label {
                    set_ellipsize: gtk::pango::EllipsizeMode::End,
                }
            }
        }

        let widgets = SongListItemWidget { text };

        (list_box, widgets)
    }

    fn bind(&mut self, _widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        let text = self.song.title.to_string();
        _widgets.text.set_label(&text);
    }
}
