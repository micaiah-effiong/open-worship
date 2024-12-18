mod edit_modal;
mod edit_modal_list_item;
mod list_item;

use std::{cell::RefCell, rc::Rc};

use edit_modal::{EditModel, EditModelInputMsg, EditModelOutputMsg};
use gtk::{glib::clone, prelude::*, SingleSelection};
use list_item::SongListItem;
use relm4::{prelude::*, typed_view::list::TypedListView};

use crate::dto;

#[derive(Debug)]
pub enum SearchSongInput {
    OpenEditModel,
    NewSong(SongListItem),
}

#[derive(Debug)]
pub enum SearchSongOutput {
    SendSongs(dto::ListPayload),
}

#[derive(Debug)]
pub struct SearchSongModel {
    list_view_wrapper: Rc<RefCell<TypedListView<SongListItem, SingleSelection>>>,
    edit_song_dialog: relm4::Controller<EditModel>,
}

impl SearchSongModel {
    /// handles list_view activate signal
    fn register_activate(&self, sender: &ComponentSender<SearchSongModel>) {
        let wrapper = self.list_view_wrapper.clone();
        let list_view = self.list_view_wrapper.borrow().view.clone();

        list_view.connect_activate(clone!(
            @strong wrapper,
            @strong sender,
            => move |_,pos|{
                println!("song clicked");
                let song_list_item = match wrapper.borrow().get(pos){
                    Some(item)=>item.borrow().clone(),
                    None=>return
                };
                println!("song clicked {:?}", song_list_item);

                let verse_list = song_list_item.verses.into_iter().map(|s|s.text).collect::<Vec<String>>();
                let list_payload = dto::ListPayload::new(song_list_item.title, 0, verse_list, None);
                let _ = sender.output(SearchSongOutput::SendSongs(list_payload));
            }
        ));
    }

    fn convert_edit_model_response(res: EditModelOutputMsg) -> SearchSongInput {
        return match res {
            EditModelOutputMsg::Save(song) => SearchSongInput::NewSong(song),
        };
    }
}

impl SearchSongModel {}

pub struct SearchSongInit {}

impl SearchSongModel {}

#[relm4::component(pub)]
impl SimpleComponent for SearchSongModel {
    type Init = SearchSongInit;
    type Output = SearchSongOutput;
    type Input = SearchSongInput;

    view! {
        #[root]
        gtk::Box{
            set_orientation:gtk::Orientation::Vertical,
            set_vexpand: true,
            add_css_class: "blue_box",

            #[name="search_field"]
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 2,
                set_height_request: 48,
                add_css_class: "green_double_box",

                gtk::Label {
                    set_label: "Title",
                    set_margin_horizontal: 5,
                },

                gtk::SearchEntry {
                    set_placeholder_text: Some("Search..."),
                    set_hexpand: true
                }
            },

            gtk::ScrolledWindow {
                set_vexpand: true,

                #[local_ref]
                list_view -> gtk::ListView {}
            },

            gtk::Box {
                gtk::Button {
                    set_icon_name: "plus",
                    set_tooltip: "Add song",
                    connect_clicked => SearchSongInput::OpenEditModel,
                    // connect_clicked[sender] => move|_|{
                    //    sender.input(SearchSongInput::OpenEditModel) ;
                    // }
                }
            }

        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let mut list_view_wrapper = TypedListView::new();

        for song in get_default_data() {
            let song_item = SongListItem::new(song.0, Vec::from(song.1));
            list_view_wrapper.append(song_item);
        }
        let list_view_wrapper = Rc::new(RefCell::new(list_view_wrapper));

        let edit_song_dialog = EditModel::builder()
            .transient_for(&root)
            .launch(())
            .forward(
                sender.input_sender(),
                SearchSongModel::convert_edit_model_response,
            );

        let model = SearchSongModel {
            list_view_wrapper,
            edit_song_dialog,
        };
        let list_view = &model.list_view_wrapper.borrow().view.clone();
        model.register_activate(&sender);

        let widgets = view_output!();

        return relm4::ComponentParts { model, widgets };
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            SearchSongInput::OpenEditModel => {
                self.edit_song_dialog.emit(EditModelInputMsg::Show);
                println!("start opening");
            }
            SearchSongInput::NewSong(song) => {
                self.list_view_wrapper.borrow_mut().append(song);
            }
        };
    }
}

fn get_default_data() -> Vec<(std::string::String, [std::string::String; 4])> {
    return Vec::from([
        (
            "Echoes of the Soul".to_string(),
            [
                "In depths of silence, where thoughts reside,\nA melody whispers, a soulful tide.\nEmotions painted in hues of night,\nSearching for solace, a guiding light.".to_string(),
                "Lost in the echoes of yesterday's dream,\nA fragile heart, a delicate scheme.\nYearning for moments, pure and true,\nFinding strength within, a different view.".to_string(),
                "With every heartbeat, a rhythm's grace,\nA journey inward, a soulful space.\nThrough shadows and light, the spirit soars,\nDiscovering treasures, unlocking doors.".to_string(),
                "In harmony's embrace, the soul finds peace,\nA gentle whisper, a sweet release.\nWith hope as a compass, a steady hand,\nWalking the path, through this mortal land.".to_string()
            ]
        ),

        (
            "City Lights and Lonely Nights".to_string(),
            [
                "Neon signs blink, a dazzling display,\nBut shadows deepen as the night creeps in.\nA bustling city, a vibrant scene,\nYet solitude lingers, a haunting refrain.".to_string(),
                "Lost in the crowd, a faceless name,\nSearching for connection, a fading flame.\nThe rhythm of life, a constant chase,\nA longing for love, a warm embrace.".to_string(),
                "Dreams and aspirations, a fragile art,\nA fragile heart, torn apart.\nThe city's allure, a deceptive guise,\nHiding the yearning, beneath the disguise.".to_string(),
                "In the quiet moments, when silence descends,\nA soul yearns for peace, where solace transcends. \nA flicker of hope, a distant star,\nGuiding the way, from near or far.".to_string()  
            ]
        ),
        (
            "Whispers of the Wind".to_string(),
            [
                "The wind whispers secrets, through rustling leaves,\nCarrying stories, of life's reprieves.\nA gentle caress, a soothing sound,\nNature's symphony, all around.".to_string(),
                "From distant lands, it carries dreams,\nOf endless horizons, and sparkling streams.\nA touch of magic, a playful breeze,\nDancing with freedom, through ancient trees.".to_string(),
                "It sings of love, of loss, and pain,\nOf hope and resilience, again and again.\nA constant companion, a faithful friend,\nA gentle reminder, that life will transcend.".to_string(),
                "With every gust, a promise of new,\nA chance to start, with a different view.\nIn its embrace, find solace and grace,\nLet the wind's wisdom, fill your space.".to_string()
            ]
        ),
        ("Fragments of Time".to_string(),
            [
                "Time slips away, like grains of sand,\nLeaving footprints, on life's vast land.\nMemories linger, a bittersweet art,\nShaping the soul, from the very start.".to_string(),
                "In the tapestry of years gone by,\nMoments of joy, and reasons to cry.\nLessons learned, through trials and strife,\nBuilding resilience, for a stronger life.".to_string(),
                "The clock ticks on, an endless race,\nChasing dreams, at a frantic pace.\nYet, in stillness, find inner peace,\nA sanctuary of calm, a sweet release.".to_string(),
                "Embrace the present, with open heart, Let go of the past, make a brand new start. Time is a gift, a precious art,\nUse it wisely, from the very start.".to_string()
            ]
        ),
        (
            "Ocean's Lullaby".to_string(),
            [
                "Waves crash gently, on sandy shores,\nCarrying secrets, from ocean's cores.\nThe moon's soft glow, a silvered sea,\nA tranquil beauty, wild and free.".to_string(),
                "Beneath the surface, a world unknown,\nCreatures of wonder, a mystic zone.\nThe rhythm of tides, a constant flow,\nA dance of nature, a wondrous show.".to_string(),
                "In solitude's embrace, the soul finds peace,\nAs ocean's melody, offers release.\nThe vast expanse, a mirror of mind,\nReflecting depths, where answers reside.".to_string(),
                "With every tide, a chance to renew,\nTo wash away worries, old and new.\nIn ocean's wisdom, find strength to be,\nA harmonious part, of eternity.".to_string()
            ]
        )
    ]);
}
