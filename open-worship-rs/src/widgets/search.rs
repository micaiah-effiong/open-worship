mod background;
mod scriptures;
mod songs;

use gtk::prelude::*;
use relm4::prelude::*;

use background::{SearchBacgroundOutput, SearchBackgroundInit, SearchBackgroundModel};
use scriptures::{SearchScriptureInit, SearchScriptureModel};
use songs::{SearchSongInit, SearchSongModel};

const MIN_GRID_HEIGHT: i32 = 300;
// const MIN_GRID_WIDTH: i32 = 300;

// search area (notebook)
#[derive(Debug)]
pub enum SearchInput {
    PreviewBackground(String),
}

#[derive(Debug)]
pub enum SearchOutput {
    PreviewBackground(String),
}

#[derive(Debug)]
pub struct SearchModel {
    background_page: relm4::Controller<SearchBackgroundModel>,
    scripture_page: relm4::Controller<SearchScriptureModel>,
    song_page: relm4::Controller<SearchSongModel>,
}

impl SearchModel {
    fn convert_background_msg(msg: SearchBacgroundOutput) -> SearchInput {
        return match msg {
            SearchBacgroundOutput::SendPreviewBackground(bg_src) => {
                SearchInput::PreviewBackground(bg_src)
            }
        };
    }
}

pub struct SearchInit {
    pub image_src_list: Vec<String>,
}

impl SearchModel {}

#[relm4::component(pub)]
impl SimpleComponent for SearchModel {
    type Init = SearchInit;
    type Output = SearchOutput;
    type Input = SearchInput;

    view! {
        #[root]
        gtk::Box{
            set_orientation: gtk::Orientation::Vertical,
            set_height_request: MIN_GRID_HEIGHT,
            set_hexpand: true,
            set_homogeneous: true,

            #[name="tab_box"]
            gtk::Box {
                set_orientation:gtk::Orientation::Horizontal,
                set_spacing: 3,
                set_css_classes: &["purple_box", "ow-listview"],
                set_height_request: 48,

                gtk::Notebook {
                    set_hexpand: true,

                    #[local_ref]
                    append_page[Some(&gtk::Label::new(Some("Songs")))] = song_page_widget -> gtk::Box {},

                    #[local_ref]
                    append_page[Some(&gtk::Label::new(Some("Scriptures")))] = scripture_page_widget -> gtk::Box{},

                    #[local_ref]
                    append_page[Some(&gtk::Label::new(Some("Backgrounds")))] = background_page_widget -> gtk::Box{}
                }
            }

        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let song_page = SearchSongModel::builder()
            .launch(SearchSongInit {})
            .forward(sender.input_sender(), |_| unimplemented!());
        let scripture_page = SearchScriptureModel::builder()
            .launch(SearchScriptureInit {})
            .forward(sender.input_sender(), |_| unimplemented!());
        let background_page = SearchBackgroundModel::builder()
            .launch(SearchBackgroundInit {})
            .forward(sender.input_sender(), SearchModel::convert_background_msg);

        let model = SearchModel {
            song_page,
            background_page,
            scripture_page,
        };

        let background_page_widget = model.background_page.widget();
        let scripture_page_widget = model.scripture_page.widget();
        let song_page_widget = model.song_page.widget();

        let widgets = view_output!();

        return relm4::ComponentParts { model, widgets };
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            SearchInput::PreviewBackground(bg) => {
                let _ = sender.output(SearchOutput::PreviewBackground(bg));
            }
        };
    }
}

const LIST_VEC: [&str; 13] = [
    "Golden sun, a radiant masterpiece, paints the canvas of the morning sky. With hues of pink and softest blue, a breathtaking, ethereal sight,",
    "A gentle breeze, a whispered lullaby, carries softly through and through, Enveloping the world in calm as morning dew begins to fall anew.",
    "Dew-kissed flowers, adorned with sparkling gems, open wide to greet the day, Unfurling petals, soft and sweet, in a vibrant, colorful display,",
    "Nature's beauty, a masterpiece, unfolds before our wondering eyes,",
    "Inviting us to pause and breathe, beneath the endless, open skies.",
    "Children laugh, their joy infectious, as they chase their dreams so high,",
    "Imaginations soar and fly, reaching for the boundless sky,",
    "Hopeful wishes, like tiny stars, twinkle brightly in their hearts,",
    "As golden moments slip away, leaving precious, lasting marks.",
    "Hand in hand, we'll journey on, through life's winding, twisting road,",
    "With courage, strength, and hearts aflame, carrying hope's precious load,",
    "Brighter days, a promised land, await us just beyond the bend,",
    "As love and friendship's bonds endure, forever and without an end.",
];
