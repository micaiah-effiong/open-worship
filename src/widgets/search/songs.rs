pub mod edit_modal;
mod list_item;
mod toolbar;

use std::cell::RefCell;
use std::rc::Rc;

use gtk::SingleSelection;
use gtk::gio::{ActionEntry, MenuItem, SimpleActionGroup};
use gtk::glib::clone;
use gtk::prelude::*;
use list_item::SongListItemModel;
use relm4::prelude::*;
use relm4::typed_view::list::TypedListView;

use crate::db::query::Query;
use crate::dto::{SongData, SongObject};
use crate::widgets::canvas::serialise::SlideManagerData;
use crate::widgets::search::songs::edit_modal::SongEditWindow;

#[derive(Debug)]
pub enum SearchSongInput {
    OpenEditModel(Option<SongObject>), // NOTE: should use SongData
    NewSong(SongData),
    RemoveSong(u32),
}

#[derive(Debug)]
pub enum SearchSongOutput {
    SendToPreview(SlideManagerData),
    SendToSchedule(SlideManagerData),
}

#[derive(Debug)]
pub struct SearchSongModel {
    list_view_wrapper: Rc<RefCell<TypedListView<SongListItemModel, SingleSelection>>>,
    search_field: gtk::SearchEntry,
    edit_song_dialog: RefCell<SongEditWindow>,
}

impl SearchSongModel {
    fn register_search_field_events(&self) {
        let search = self.search_field.clone();
        let list = self.list_view_wrapper.clone();

        search.connect_search_changed(clone!(
            #[strong]
            list,
            move |se| {
                //
                let songs = match Query::get_songs(se.text().to_string()) {
                    Ok(q) => q,
                    Err(e) => {
                        eprintln!("SQL ERROR: {:?}", e);
                        return;
                    }
                };

                let mut lw = list.borrow_mut();
                lw.clear();
                songs.iter().for_each(|s| {
                    lw.append(SongListItemModel::new(s.clone().into()));
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

                    sender.input(SearchSongInput::OpenEditModel(Some(song_list_item.song)));
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
                    let song_list_item = match wrapper.borrow().get(model.selection().nth(0)) {
                        Some(item) => item.borrow().clone(),
                        None => return,
                    };

                    let _ = sender.output(SearchSongOutput::SendToSchedule(
                        song_list_item.clone().into(),
                    ));
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
                let song_list_item = match wrapper.borrow().get(pos) {
                    Some(item) => item.borrow().clone(),
                    None => return,
                };

                // let list_payload =
                //     dto::ListPayload::new(song_list_item.song.title(), 0, verse_list, None);
                let _ = sender.output(SearchSongOutput::SendToPreview(
                    song_list_item.clone().into(),
                ));
            }
        ));
    }

    // fn convert_edit_model_response(res: EditModelOutputMsg) -> SearchSongInput {
    //     match res {
    //         EditModelOutputMsg::Save(song) => SearchSongInput::NewSong(song),
    //     }
    // }
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

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 2,
                set_height_request: 48,

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

        let initial_songs = Query::get_songs("".into());
        match initial_songs {
            Ok(songs) => {
                for song in songs {
                    list_view_wrapper.append(SongListItemModel::new(song.into()));
                }
            }
            Err(e) => eprintln!("SQL ERROR: {:?}", e),
        }

        let list_view_wrapper = Rc::new(RefCell::new(list_view_wrapper));

        let edit_song_dialog = RefCell::new(SongEditWindow::new());

        let search_field = gtk::SearchEntry::new();

        let model = SearchSongModel {
            list_view_wrapper,
            search_field: search_field.clone(),
            edit_song_dialog,
        };

        let list_view = &model.list_view_wrapper.borrow().view.clone();
        model.register_listview_activate(&sender);
        model.register_context_menu(&sender);
        model.register_search_field_events();
        let widgets = view_output!();

        relm4::ComponentParts { model, widgets }
    }

    // TODO: Invalidate songs list
    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            SearchSongInput::OpenEditModel(song) => {
                let sew = SongEditWindow::new();
                self.edit_song_dialog.replace(sew.clone());

                self.edit_song_dialog.borrow().clone().connect_save({
                    let sender = sender.clone();
                    move |_, song_obj| {
                        println!("SONG saved");
                        sender.input(SearchSongInput::NewSong(song_obj.clone().into()));
                    }
                });
                sew.show(song);
            }
            SearchSongInput::NewSong(song) => {
                println!("SONG saved NEW SONG");
                let songs = Query::get_songs("".to_string());

                match songs {
                    Ok(songs) => {
                        self.list_view_wrapper.borrow_mut().clear();
                        let mut lw = self.list_view_wrapper.borrow_mut();
                        songs.iter().for_each(|s| {
                            lw.append(SongListItemModel::new(s.clone().into()));
                        });
                    }
                    Err(e) => eprintln!("SQL ERROR: {:?}", e),
                }
            }
            SearchSongInput::RemoveSong(pos) => {
                let song_item = match self.list_view_wrapper.borrow().get(pos) {
                    Some(si) => si.borrow().clone().song,
                    None => return,
                };

                match Query::delete_song(song_item.into()) {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("SQL ERROR: {:?}", e);
                        return;
                    }
                };

                match Query::get_songs("".to_string()) {
                    Ok(songs) => {
                        self.list_view_wrapper.borrow_mut().clear();
                        let mut lw = self.list_view_wrapper.borrow_mut();
                        songs.iter().for_each(|s| {
                            lw.append(SongListItemModel::new(s.clone().into()));
                        });
                    }
                    Err(e) => {
                        eprintln!("SQL ERROR: {:?}", e);
                        return;
                    }
                };

                self.list_view_wrapper.borrow().view.grab_focus();
            }
        };
    }
}
