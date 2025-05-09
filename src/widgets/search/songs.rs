mod edit_modal;
mod edit_modal_list_item;
mod list_item;

use std::{cell::RefCell, rc::Rc};

use edit_modal::{EditModel, EditModelInit, EditModelInputMsg, EditModelOutputMsg};
use gtk::{
    gio::{ActionEntry, MenuItem, SimpleActionGroup},
    glib::clone,
    prelude::*,
    SingleSelection,
};
use list_item::SongListItemModel;
use relm4::{prelude::*, typed_view::list::TypedListView};

use crate::{
    db::{connection::DatabaseConnection, query::Query},
    dto::{self, Song},
};

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
    search_field: gtk::SearchEntry,
    edit_song_dialog: relm4::Controller<EditModel>,
    db_connection: Rc<RefCell<Option<DatabaseConnection>>>,
}

impl SearchSongModel {
    fn register_search_field_events(&self) {
        let search = self.search_field.clone();
        let list = self.list_view_wrapper.clone();
        let db = self.db_connection.clone();

        search.connect_search_changed(clone!(
            #[strong]
            db,
            #[strong]
            list,
            move |se| {
                //
                let songs = match Query::get_songs(db.clone(), se.text().to_string()) {
                    Ok(q) => q,
                    Err(e) => {
                        eprintln!("SQL ERROR: {:?}", e);
                        return ();
                    }
                };

                let mut lw = list.borrow_mut();
                lw.clear();
                songs.iter().for_each(|s| {
                    lw.append(SongListItemModel::new(s.clone()));
                });
            }
        ));

        search.connect_activate(clone!(
            #[strong]
            list,
            move |_se| {
                println!("S ACTIVATE");
                let t_list = list.borrow();
                let model = &t_list.selection_model;
                let view = &t_list.view;

                let selected = model.selected().to_value();
                view.emit_by_name_with_values("activate", &[selected]);
            }
        ));

        search.connect_next_match(|m| {
            println!("N_MATCH <C-g> {:?}", m);
        });
    }

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
        popover_menu.set_align(gtk::Align::Start);
        popover_menu.set_parent(&list_view);

        let gesture_click = gtk::GestureClick::new();
        gesture_click.set_button(gtk::gdk::BUTTON_SECONDARY);
        gesture_click.connect_pressed(clone!(
            #[strong]
            popover_menu,
            move |gc, _, x, y| {
                let rect = gtk::gdk::Rectangle::new(x as i32, y as i32, 10, 10);
                popover_menu.set_pointing_to(Some(&rect));
                popover_menu.popup();
                gc.set_state(gtk::EventSequenceState::Claimed);
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
                println!("P ACTIVATE");
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

pub struct SearchSongInit {
    pub db_connection: Rc<RefCell<Option<DatabaseConnection>>>,
}

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

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 2,
                set_height_request: 48,
                add_css_class: "green_double_box",

                // gtk::Label {
                //     set_label: "Title",
                //     set_margin_horizontal: 5,
                // },

                #[local_ref]
                append = &search_field -> gtk::SearchEntry {

                    set_placeholder_text: Some("Search title..."),
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
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let mut list_view_wrapper = TypedListView::new();

        let initial_songs = Query::get_songs(init.db_connection.clone(), "".to_string());
        match initial_songs {
            Ok(songs) => {
                for song in songs {
                    list_view_wrapper.append(SongListItemModel::new(song));
                }
            }
            Err(e) => eprintln!("SQL ERROR: {:?}", e),
        }

        let list_view_wrapper = Rc::new(RefCell::new(list_view_wrapper));

        let edit_song_dialog = EditModel::builder()
            .transient_for(&root)
            .launch(EditModelInit {
                db_connection: init.db_connection.clone(),
            })
            .forward(
                sender.input_sender(),
                SearchSongModel::convert_edit_model_response,
            );

        let search_field = gtk::SearchEntry::new();

        let model = SearchSongModel {
            list_view_wrapper,
            search_field: search_field.clone(),
            edit_song_dialog,
            db_connection: init.db_connection.clone(),
        };
        let list_view = &model.list_view_wrapper.borrow().view.clone();
        model.register_listview_activate(&sender);
        model.register_context_menu(&sender);
        model.register_search_field_events();
        let widgets = view_output!();

        return relm4::ComponentParts { model, widgets };
    }

    // TODO: Invalidate songs list
    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            SearchSongInput::OpenEditModel(song) => {
                self.edit_song_dialog.emit(EditModelInputMsg::Show(song));
            }
            SearchSongInput::NewSong(song) => {
                let songs = Query::get_songs(self.db_connection.clone(), "".to_string());

                match songs {
                    Ok(songs) => {
                        self.list_view_wrapper.borrow_mut().clear();
                        let mut lw = self.list_view_wrapper.borrow_mut();
                        songs.iter().for_each(|s| {
                            lw.append(SongListItemModel::new(s.clone()));
                        });
                    }
                    Err(e) => eprintln!("SQL ERROR: {:?}", e),
                }
            }
            SearchSongInput::RemoveSong(pos) => {
                let song_item = match self.list_view_wrapper.borrow().get(pos) {
                    Some(si) => si.borrow().clone().song,
                    None => return (),
                };

                match Query::delete_song(self.db_connection.clone(), song_item) {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("SQL ERROR: {:?}", e);
                        return ();
                    }
                };

                match Query::get_songs(self.db_connection.clone(), "".to_string()) {
                    Ok(songs) => {
                        self.list_view_wrapper.borrow_mut().clear();
                        let mut lw = self.list_view_wrapper.borrow_mut();
                        songs.iter().for_each(|s| {
                            lw.append(SongListItemModel::new(s.clone()));
                        });
                    }
                    Err(e) => {
                        eprintln!("SQL ERROR: {:?}", e);
                        return ();
                    }
                };

                self.list_view_wrapper.borrow().view.grab_focus();
            }
        };
    }
}
