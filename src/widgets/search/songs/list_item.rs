use relm4::{
    gtk::{self},
    typed_view::list::RelmListItem,
    view,
};

#[derive(Debug, Clone)]
pub struct SongData {
    /// song tags are identifiers like
    /// - chorus
    /// - verse
    /// - etc...
    pub tag: Option<String>,
    pub text: String,
}

impl SongData {
    fn new(text: String, tag: Option<String>) -> Self {
        return SongData { tag, text };
    }
}

#[derive(Debug, Clone)]
pub struct SongListItem {
    pub title: String,
    pub verses: Vec<SongData>,
}

impl SongListItem {
    pub fn new(title: String, verse_list: Vec<String>) -> Self {
        let mut verses = Vec::new();

        for verse in verse_list {
            verses.push(SongData::new(verse, None));
        }

        return SongListItem { title, verses };
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

impl RelmListItem for SongListItem {
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

        return (list_box, widgets);
    }

    fn bind(&mut self, _widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        let text = format!("{}", self.title);
        _widgets.text.set_label(&text);
    }
}
