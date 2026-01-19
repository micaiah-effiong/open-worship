use gtk::glib;
use gtk::glib::subclass::types::ObjectSubclassIsExt;

#[derive(Debug, Clone)]
pub struct Payload {
    pub text: String,
    pub position: u32,
    pub background_image: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ListPayload {
    pub text: String,
    pub position: u32,
    pub list: Vec<String>,
    pub background_image: Option<String>,
}

impl ListPayload {
    pub fn new(
        text: String,
        position: u32,
        list: Vec<String>,
        background_image: Option<String>,
    ) -> ListPayload {
        ListPayload {
            text,
            position,
            list,
            background_image,
        }
    }
}

#[derive(Debug, Clone, glib::Boxed)]
#[boxed_type(name = "DisplayPayload")]
pub struct DisplayPayload {
    pub text: String,
    /// image src/filepath
    pub background_image: Option<String>,
}

impl DisplayPayload {
    pub fn new(text: String) -> Self {
        DisplayPayload {
            text,
            background_image: None,
        }
    }
}

// SONG VERSE
#[derive(Debug, Clone, Default, PartialEq, Eq, glib::Boxed)]
#[boxed_type(name = "SongVerse")]
pub struct SongVerse {
    /// song tags are identifiers like
    /// - chorus
    /// - verse
    /// - etc...
    pub tag: Option<String>,
    pub text: String,
    pub slide: Option<String>,
}

impl SongVerse {
    pub fn new(text: String, tag: Option<String>, slide: Option<String>) -> Self {
        SongVerse { tag, text, slide }
    }
}

// SONG

#[derive(Debug, Clone, Default)]
pub struct SongData {
    pub song_id: u32,
    pub title: String,
    pub verses: Vec<SongVerse>,
}

impl SongData {
    pub fn new(id: u32, title: String, verses: Vec<SongVerse>) -> Self {
        Self {
            song_id: id,
            title,
            verses,
        }
    }
}

mod imp {
    use std::cell::RefCell;

    use super::*;
    use gtk::glib::{
        Properties,
        subclass::{object::ObjectImpl, prelude::*, types::ObjectSubclass},
    };
    use gtk::prelude::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type=super::SongObject)]
    pub struct SongObject {
        #[property(name="title", get, set, type=String, member=title)]
        #[property(name="song-id", get, set, type=u32, member=song_id)]
        pub data: RefCell<SongData>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SongObject {
        const NAME: &'static str = "SongObject";
        type Type = super::SongObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for SongObject {}
}

glib::wrapper! {
pub struct SongObject(ObjectSubclass<imp::SongObject>);
}

impl Default for SongObject {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl SongObject {
    pub fn new(title: String, verse_list: Vec<String>, song_id: u32) -> Self {
        let mut verses = Vec::new();

        for verse in verse_list {
            verses.push(SongVerse::new(verse, None, None));
        }

        Self::from_verses(title, verses, song_id)
    }

    pub fn from_verses(title: String, verses: Vec<SongVerse>, song_id: u32) -> Self {
        let obj: Self = glib::Object::builder()
            .property("song-id", song_id)
            .property("title", title)
            .build();
        obj.set_verses(verses);

        obj
    }

    pub fn verses(&self) -> Vec<SongVerse> {
        self.imp().data.borrow().verses.clone()
    }

    pub fn set_verses(&self, verses: Vec<SongVerse>) {
        self.imp().data.borrow_mut().verses = verses;
    }

    pub fn add_verse(&self, verse: SongVerse) {
        self.imp().data.borrow_mut().verses.push(verse);
    }
    pub fn song_data(&self) -> SongData {
        self.imp().data.borrow().clone()
    }
}
impl From<SongData> for SongObject {
    fn from(data: SongData) -> Self {
        let obj = SongObject::from_verses(data.title, data.verses, data.song_id);
        obj
    }
}

// Convert from SongObject to SongData
impl From<SongObject> for SongData {
    fn from(obj: SongObject) -> Self {
        obj.imp().data.borrow().clone()
    }
}
// SCRIPTURE

#[derive(Debug, Clone)]
pub struct Scripture {
    pub book: String,
    pub chapter: u32,
    pub verse: u32,
    pub text: String,
    pub translation: String,
}

impl Scripture {
    pub fn screen_display(&self) -> String {
        let text = format!(
            "{}\n{} {}:{} ({})",
            self.text, self.book, self.chapter, self.verse, self.translation
        );
        text
    }
}
