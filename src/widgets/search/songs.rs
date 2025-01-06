mod edit_modal;
mod edit_modal_list_item;
mod list_item;

use std::{cell::RefCell, rc::Rc};

use edit_modal::{EditModel, EditModelInputMsg, EditModelOutputMsg};
use gtk::{
    gio::{ActionEntry, MenuItem, SimpleActionGroup},
    glib::clone,
    prelude::*,
    SingleSelection,
};
use list_item::SongListItemModel;
use relm4::{prelude::*, typed_view::list::TypedListView};

use crate::dto::{self, Song};

#[derive(Debug)]
pub enum SearchSongInput {
    OpenEditModel(Option<Song>),
    NewSong(Song),
    RemoveSong(u32),
}

#[derive(Debug)]
pub enum SearchSongOutput {
    SendToPreview(dto::ListPayload),
    SendToSchedule(dto::ListPayload),
}

#[derive(Debug)]
pub struct SearchSongModel {
    list_view_wrapper: Rc<RefCell<TypedListView<SongListItemModel, SingleSelection>>>,
    edit_song_dialog: relm4::Controller<EditModel>,
}

impl SearchSongModel {
    /// handles list_view right click gesture
    fn register_context_menu(&self, sender: &ComponentSender<SearchSongModel>) {
        let wrapper = self.list_view_wrapper.clone();
        let list_view = self.list_view_wrapper.borrow().view.clone();
        let model = match list_view.model() {
            Some(m) => m,
            None => return,
        };

        let add_song_action = ActionEntry::builder("add-song")
            .activate(clone!(
                #[strong]
                sender,
                move |_g: &SimpleActionGroup, _sa, _v| {
                    sender.input(SearchSongInput::OpenEditModel(None));
                }
            ))
            .build();

        let edit_action = ActionEntry::builder("edit")
            .activate(clone!(
                #[strong]
                wrapper,
                #[strong]
                model,
                #[strong]
                sender,
                move |_g: &SimpleActionGroup, _sa, _v| {
                    if model.n_items() == 0 {
                        return;
                    }

                    let song_list_item = match wrapper.borrow().get(model.selection().nth(0)) {
                        Some(item) => item.borrow().clone(),
                        None => return,
                    };

                    let _ = sender.input(SearchSongInput::OpenEditModel(Some(song_list_item.song)));
                }
            ))
            .build();

        let add_to_schedule_action = ActionEntry::builder("add-to-schedule")
            .activate(clone!(
                #[strong]
                sender,
                #[strong]
                model,
                #[strong]
                wrapper,
                move |_g: &SimpleActionGroup, _sa, _v| {
                    if let Some(li) = wrapper.borrow().get(model.selection().nth(0)) {
                        let song = li.borrow().song.clone();
                        let _ = sender.output(SearchSongOutput::SendToSchedule(dto::ListPayload {
                            text: song.title,
                            position: 0,
                            list: song.verses.iter().map(|elt| elt.text.clone()).collect(),
                            background_image: None,
                        }));
                    }
                }
            ))
            .build();

        let delete_action = ActionEntry::builder("delete")
            .activate(clone!(
                #[strong]
                model,
                #[strong]
                sender,
                move |_g: &SimpleActionGroup, _sa, _v| {
                    sender.input(SearchSongInput::RemoveSong(model.selection().nth(0)));
                }
            ))
            .build();

        let menu_action_group = SimpleActionGroup::new();
        menu_action_group.add_action_entries([
            add_song_action,
            edit_action,
            add_to_schedule_action,
            delete_action,
        ]);

        let menu = gtk::gio::Menu::new();
        let add_to_schedule = MenuItem::new(Some("Add to schedule"), Some("song.add-to-schedule"));
        menu.insert_item(1, &add_to_schedule);
        menu.insert_item(2, &MenuItem::new(Some("Add song"), Some("song.add-song")));
        menu.insert_item(3, &MenuItem::new(Some("Edit song"), Some("song.edit")));
        menu.insert_item(4, &MenuItem::new(Some("Delete song"), Some("song.delete")));

        let popover_menu = gtk::PopoverMenu::from_model(Some(&menu));
        popover_menu.set_has_arrow(false);
        popover_menu.set_position(gtk::PositionType::Top);
        popover_menu.set_parent(&list_view);

        let gesture_click = gtk::GestureClick::new();
        gesture_click.set_button(gtk::gdk::ffi::GDK_BUTTON_SECONDARY as u32);
        gesture_click.connect_pressed(clone!(
            #[strong]
            popover_menu,
            move |_, _, x, y| {
                let rect = gtk::gdk::Rectangle::new(x as i32, y as i32, 10, 10);
                popover_menu.set_pointing_to(Some(&rect));
                popover_menu.popup();
            }
        ));

        list_view.insert_action_group("song", Some(&menu_action_group));
        list_view.add_controller(gesture_click);
    }

    /// handles list_view activate signal
    fn register_listview_activate(&self, sender: &ComponentSender<SearchSongModel>) {
        let wrapper = self.list_view_wrapper.clone();
        let list_view = self.list_view_wrapper.borrow().view.clone();

        list_view.connect_activate(clone!(
            #[strong]
            wrapper,
            #[strong]
            sender,
            move |_lv, pos| {
                let song_list_item = match wrapper.borrow().get(pos) {
                    Some(item) => item.borrow().clone(),
                    None => return,
                };

                let verse_list = song_list_item
                    .song
                    .verses
                    .into_iter()
                    .map(|s| s.text)
                    .collect::<Vec<String>>();
                let list_payload =
                    dto::ListPayload::new(song_list_item.song.title, 0, verse_list, None);
                let _ = sender.output(SearchSongOutput::SendToPreview(list_payload));
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
                list_view -> gtk::ListView {
                    set_show_separators: true
                }
            },

            gtk::Box {
                //
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
            let song_item = SongListItemModel::new(Song::new(song.0, Vec::from(song.1)));
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
        model.register_listview_activate(&sender);
        model.register_context_menu(&sender);

        let widgets = view_output!();

        return relm4::ComponentParts { model, widgets };
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            SearchSongInput::OpenEditModel(song) => {
                self.edit_song_dialog.emit(EditModelInputMsg::Show(song));
            }
            SearchSongInput::NewSong(song) => {
                self.list_view_wrapper
                    .borrow_mut()
                    .append(SongListItemModel::new(song));
            }
            SearchSongInput::RemoveSong(pos) => {
                let list_view = self.list_view_wrapper.borrow().view.clone();

                self.list_view_wrapper.borrow_mut().remove(pos);
                list_view.grab_focus();
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
