use gtk::glib;
use gtk::glib::subclass::types::ObjectSubclassIsExt;

use crate::{
    services::settings::ApplicationSettings,
    widgets::canvas::serialise::{CanvasItemType, SlideData, SlideManagerData},
};

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

impl Into<SlideManagerData> for SongObject {
    fn into(self) -> SlideManagerData {
        let settings = ApplicationSettings::get_instance();
        let slide_list = self
            .verses()
            .into_iter()
            .map(|s| {
                let mut s = s
                    .slide
                    .as_ref()
                    .and_then(|val| serde_json::from_str(val).ok())
                    .unwrap_or_else(SlideData::from_default);
                for v in &mut s.items {
                    match &mut v.item_type {
                        CanvasItemType::Text(text_item_data) => {
                            text_item_data.font = settings.song_font();
                        }
                        CanvasItemType::Unknown => (),
                    };
                }
                s
            })
            .collect::<Vec<_>>();

        let mut sm_data = SlideManagerData::new(0, 0, slide_list);
        sm_data.title = self.title();
        sm_data
    }
}

// SCRIPTURE

#[derive(Debug, Default, Clone, glib::Boxed)]
#[boxed_type(name = "Scripture")]
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

pub mod scripture {
    use super::*;
    mod imp {
        use std::cell::{Cell, RefCell};

        use gtk::glib::{
            Properties,
            subclass::{object::ObjectImpl, types::ObjectSubclass},
        };
        use gtk::prelude::ObjectExt;
        use gtk::subclass::prelude::DerivedObjectProperties;

        use super::*;

        #[derive(Debug, Default, Properties)]
        #[properties(wrapper_type = super::ScriptureObject)]
        pub struct ScriptureObject {
            #[property(get, construct_only)]
            pub item: RefCell<Scripture>,
            #[property(get, construct_only)]
            pub full_reference: Cell<bool>,
        }

        #[glib::object_subclass]
        impl ObjectSubclass for ScriptureObject {
            const NAME: &'static str = "ScriptureObject";
            type Type = super::ScriptureObject;
        }

        #[glib::derived_properties]
        impl ObjectImpl for ScriptureObject {}
    }

    glib::wrapper! {
        pub struct ScriptureObject(ObjectSubclass<imp::ScriptureObject>);
    }

    impl Default for ScriptureObject {
        fn default() -> Self {
            glib::Object::new()
        }
    }

    impl Into<SlideData> for ScriptureObject {
        fn into(self) -> SlideData {
            let settings = ApplicationSettings::get_instance();

            let text = self.item().screen_display();
            let mut slide_data = SlideData::from_default();

            for v in &mut slide_data.items {
                match &mut v.item_type {
                    CanvasItemType::Text(text_item_data) => {
                        text_item_data.font = settings.scripture_font();
                        text_item_data.text_data = glib::base64_encode(text.as_bytes()).into();
                    }
                    CanvasItemType::Unknown => (),
                };
            }

            slide_data
        }
    }

    impl ScriptureObject {
        pub fn new(scripture: Scripture, full_reference: bool) -> Self {
            let obj: Self = glib::Object::builder()
                .property("item", scripture)
                .property("full_reference", full_reference)
                .build();

            obj
        }
    }
}

//
// ScheduleActivityViewer
pub mod schedule_data {
    use super::*;

    pub mod imp {
        use std::cell::RefCell;

        use gtk::glib::{
            Properties,
            subclass::{object::ObjectImpl, types::ObjectSubclass},
        };

        use gtk::prelude::ObjectExt;
        use gtk::subclass::prelude::DerivedObjectProperties;

        use crate::widgets::canvas::serialise::SlideManagerData;

        use super::*;

        #[derive(Default, Properties)]
        #[properties(wrapper_type=super::ScheduleData)]
        pub struct ScheduleData {
            #[property(set, get, construct)]
            pub slide_data: RefCell<SlideManagerData>,
            #[property(set, get, construct)]
            pub title: RefCell<String>,
        }

        #[glib::object_subclass]
        impl ObjectSubclass for ScheduleData {
            const NAME: &'static str = "ScheduleData";
            type Type = super::ScheduleData;
        }

        #[glib::derived_properties]
        impl ObjectImpl for ScheduleData {}
    }

    glib::wrapper! {
        pub struct ScheduleData(ObjectSubclass<imp::ScheduleData>);
    }

    impl Default for ScheduleData {
        fn default() -> Self {
            glib::Object::new()
        }
    }

    impl ScheduleData {
        pub fn new(title: String, data: SlideManagerData) -> Self {
            let obj: Self = glib::Object::builder()
                .property("title", title)
                .property("slide_data", data)
                .build();

            obj
        }
    }
}

pub mod background_obj {
    use super::*;
    mod imp {
        use std::cell::RefCell;

        use gtk::glib::Properties;
        use gtk::glib::subclass::{object::ObjectImpl, types::ObjectSubclass};
        use gtk::prelude::ObjectExt;
        use gtk::subclass::prelude::DerivedObjectProperties;

        use super::*;

        #[derive(Default, Debug, Properties)]
        #[properties(wrapper_type = super::BackgroundObj)]
        pub struct BackgroundObj {
            #[property(get, construct_only)]
            pub title: RefCell<String>,
            #[property(get, construct_only)]
            pub src: RefCell<String>,
        }

        #[glib::object_subclass]
        impl ObjectSubclass for BackgroundObj {
            const NAME: &'static str = "BackgroundObj";
            type Type = super::BackgroundObj;
        }

        #[glib::derived_properties]
        impl ObjectImpl for BackgroundObj {}
    }

    glib::wrapper! {
        pub struct BackgroundObj(ObjectSubclass<imp::BackgroundObj>);
    }

    impl Default for BackgroundObj {
        fn default() -> Self {
            glib::Object::new()
        }
    }

    impl BackgroundObj {
        pub fn new(src: String, title: Option<String>) -> Self {
            let title = match title {
                Some(t) => t,
                None => {
                    let name = match std::path::Path::new(&src).file_name() {
                        Some(name) => name.to_str(),
                        None => panic!("Invalid file name"),
                    };
                    match name {
                        Some(name) => name.to_string(),
                        None => panic!("Error converting file name to string"),
                    }
                }
            };

            let obj: Self = glib::Object::builder()
                .property("title", title)
                .property("src", src)
                .build();
            obj
        }
    }
}
