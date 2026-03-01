pub mod edit_modal;
mod list_item;
mod toolbar;

use gtk::glib;
use gtk::prelude::*;

use crate::widgets::canvas::serialise::SlideManagerData;

mod signals {
    pub(super) const SEND_TO_PREVIEW: &str = "send-to-preview";
    pub(super) const SEND_TO_SCHEDULE: &str = "send-to-schedule";
}

mod imp {
    use std::sync::OnceLock;

    use gtk::{
        gio::{
            self,
            prelude::{ActionMapExtManual, ListModelExt},
        },
        glib::{
            self,
            object::{Cast, CastNone, ObjectExt},
            subclass::{
                Signal,
                object::ObjectImplExt,
                types::{ObjectSubclass, ObjectSubclassExt},
            },
            types::StaticType,
            value::ToValue,
        },
        prelude::{
            EditableExt, GestureExt, GestureSingleExt, ListItemExt, PopoverExt, SelectionModelExt,
            WidgetExt,
        },
        subclass::{
            box_::BoxImpl,
            prelude::ObjectImpl,
            widget::{
                CompositeTemplateClass, CompositeTemplateInitializingExt, WidgetClassExt,
                WidgetImpl,
            },
        },
    };

    use crate::{
        db::query::Query,
        dto::SongObject,
        utils::ListViewExtra,
        widgets::{
            canvas::serialise::SlideManagerData,
            search::songs::{edit_modal::SongEditWindow, list_item::SongListItem, signals},
        },
    };

    #[derive(Default, Debug, gtk::CompositeTemplate)]
    #[template(resource = "/com/openworship/app/ui/search_song.ui")]
    pub struct SearchSong {
        #[template_child]
        listview: gtk::TemplateChild<gtk::ListView>,
        #[template_child]
        search_field: gtk::TemplateChild<gtk::SearchEntry>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SearchSong {
        const NAME: &'static str = "SearchSong";
        type Type = super::SearchSong;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for SearchSong {
        fn constructed(&self) {
            self.parent_constructed();

            let listview = self.listview.clone();
            let model = gtk::gio::ListStore::new::<SongObject>();
            let model = gtk::SingleSelection::new(Some(model));
            listview.set_model(Some(&model));

            let factory = gtk::SignalListItemFactory::new();
            listview.set_factory(Some(&factory));
            factory.connect_setup(|_, listitem| {
                let li = listitem
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Expected ListItem");

                li.set_child(Some(&SongListItem::default()));
            });
            factory.connect_bind(|_, listitem| {
                let li = listitem
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Expected ListItem");

                let item = li
                    .item()
                    .and_downcast::<SongObject>()
                    .expect("Expected SongObject");

                let child = li
                    .child()
                    .and_downcast::<SongListItem>()
                    .expect("Exected SongListItem");

                child.load_data(item);
            });

            let initial_songs = Query::get_all_songs();
            match initial_songs {
                Ok(songs) => {
                    for song in songs {
                        let ss: SongObject = song.clone().into();
                        listview.append_item(&ss);
                    }
                }
                Err(e) => eprintln!("SQL ERROR: {:?}", e),
            }

            self.register_listview_activate();
            self.register_context_menu();
            self.register_search_field_events();
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();

            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder(signals::SEND_TO_SCHEDULE)
                        .param_types([SlideManagerData::static_type()])
                        .build(),
                    Signal::builder(signals::SEND_TO_PREVIEW)
                        .param_types([SlideManagerData::static_type()])
                        .build(),
                ]
            })
        }
    }
    impl WidgetImpl for SearchSong {}
    impl BoxImpl for SearchSong {}

    impl SearchSong {
        fn register_listview_activate(&self) {
            let listview = self.listview.clone();

            listview.connect_activate(glib::clone!(
                #[strong]
                listview,
                #[weak(rename_to=imp)]
                self,
                move |_lv, _pos| {
                    let Some(song_list_item) = listview
                        .get_selected_items()
                        .first()
                        .cloned()
                        .and_downcast::<SongObject>()
                    else {
                        return;
                    };

                    let list: SlideManagerData = song_list_item.into();

                    imp.obj().emit_send_to_preview(&list);
                }
            ));

            let Some(model) = listview.model().and_downcast::<gtk::SingleSelection>() else {
                return;
            };

            let change_fn =
                |model: &gtk::SingleSelection, imp: glib::subclass::ObjectImplRef<SearchSong>| {
                    let Some(song_list_item) = model.selected_item().and_downcast::<SongObject>()
                    else {
                        return;
                    };

                    let list: SlideManagerData = song_list_item.into();

                    imp.obj().emit_send_to_preview(&list);
                };

            model.connect_selection_changed(glib::clone!(
                #[weak(rename_to=imp)]
                self,
                move |model, _pos, _| change_fn(model, imp)
            ));

            model.connect_items_changed(glib::clone!(
                #[weak(rename_to=imp)]
                self,
                move |model, _pos, _, _| change_fn(model, imp)
            ));
        }

        fn register_context_menu(&self) {
            let listview = self.listview.clone();
            let model = match listview.model() {
                Some(m) => m,
                None => return,
            };

            let add_song_action = gio::ActionEntry::builder("add-song")
                .activate(glib::clone!(
                    #[weak(rename_to=imp)]
                    self,
                    move |_g: &gio::SimpleActionGroup, _sa, _v| {
                        imp.open_edit_modal(None);
                    }
                ))
                .build();

            let edit_action = gio::ActionEntry::builder("edit")
                .activate(glib::clone!(
                    #[strong]
                    model,
                    #[strong]
                    listview,
                    #[weak(rename_to=imp)]
                    self,
                    move |_g: &gio::SimpleActionGroup, _sa, _v| {
                        if model.n_items() == 0 {
                            return;
                        }
                        let Some(song_list_item) = listview
                            .get_selected_items()
                            .first()
                            .cloned()
                            .and_downcast::<SongObject>()
                        else {
                            return;
                        };

                        imp.open_edit_modal(Some(song_list_item));
                    }
                ))
                .build();

            let add_to_schedule_action = gio::ActionEntry::builder("add-to-schedule")
                .activate(glib::clone!(
                    #[strong]
                    listview,
                    #[weak(rename_to=imp)]
                    self,
                    move |_g: &gio::SimpleActionGroup, _sa, _v| {
                        let Some(song_list_item) = listview
                            .get_selected_items()
                            .first()
                            .cloned()
                            .and_downcast::<SongObject>()
                        else {
                            return;
                        };

                        imp.obj().emit_send_to_schedule(&song_list_item.into())
                    }
                ))
                .build();

            let delete_action = gio::ActionEntry::builder("delete")
                .activate(glib::clone!(
                    #[strong]
                    model,
                    #[weak(rename_to=imp)]
                    self,
                    move |_g: &gio::SimpleActionGroup, _sa, _v| {
                        imp.remove_song(model.selection().nth(0));
                    }
                ))
                .build();

            let menu_action_group = gio::SimpleActionGroup::new();
            menu_action_group.add_action_entries([
                add_song_action,
                edit_action,
                add_to_schedule_action,
                delete_action,
            ]);

            let menu = gtk::gio::Menu::new();
            let add_to_schedule =
                gio::MenuItem::new(Some("Add to schedule"), Some("song.add-to-schedule"));

            menu.insert_item(1, &add_to_schedule);
            menu.insert_item(
                2,
                &gio::MenuItem::new(Some("Add song"), Some("song.add-song")),
            );
            menu.insert_item(3, &gio::MenuItem::new(Some("Edit song"), Some("song.edit")));
            menu.insert_item(
                4,
                &gio::MenuItem::new(Some("Delete song"), Some("song.delete")),
            );

            let popover_menu = gtk::PopoverMenu::from_model(Some(&menu));
            popover_menu.set_has_arrow(false);
            popover_menu.set_halign(gtk::Align::Start);
            popover_menu.set_valign(gtk::Align::Start);
            popover_menu.set_parent(&listview);

            let gesture_click = gtk::GestureClick::new();
            gesture_click.set_button(gtk::gdk::BUTTON_SECONDARY);
            gesture_click.connect_pressed(glib::clone!(
                #[strong]
                popover_menu,
                move |gc, _, x, y| {
                    let rect = gtk::gdk::Rectangle::new(x as i32, y as i32, 10, 10);
                    popover_menu.set_pointing_to(Some(&rect));
                    popover_menu.popup();
                    gc.set_state(gtk::EventSequenceState::Claimed);
                }
            ));

            listview.insert_action_group("song", Some(&menu_action_group));
            listview.add_controller(gesture_click);
        }

        fn register_search_field_events(&self) {
            let search = self.search_field.clone();
            let list = self.listview.clone();

            search.connect_search_changed(glib::clone!(
                #[strong]
                list,
                move |se| {
                    let songs = match Query::get_songs(se.text().to_string()) {
                        Ok(q) => q,
                        Err(e) => {
                            eprintln!("SQL ERROR: {:?}", e);
                            return;
                        }
                    };

                    list.remove_all();
                    songs.iter().for_each(|s| {
                        let ss: SongObject = s.clone().into();
                        list.append_item(&ss);
                    });
                }
            ));

            search.connect_activate(glib::clone!(
                #[strong]
                list,
                move |_se| {
                    println!("S ACTIVATE");
                    let Some(model) = list.model().and_downcast::<gtk::SingleSelection>() else {
                        return;
                    };

                    let selected = model.selected().to_value();
                    list.emit_by_name_with_values("activate", &[selected]);
                }
            ));

            search.connect_next_match(|m| {
                println!("N_MATCH <C-g> {:?}", m);
            });
        }

        fn open_edit_modal(&self, song: Option<SongObject>) {
            let edit_window = SongEditWindow::new();

            edit_window.connect_save(glib::clone!(
                #[weak(rename_to=imp)]
                self,
                move |_, song_obj| {
                    println!("SONG saved");

                    imp.new_song(song_obj);
                }
            ));
            edit_window.show(song);
        }
        fn new_song(&self, song: &SongObject) {
            println!("SONG saved NEW SONG");
            let songs = Query::get_all_songs();

            match songs {
                Ok(songs) => {
                    self.listview.remove_all();
                    songs.iter().for_each(|s| {
                        let ss: SongObject = s.clone().into();
                        self.listview.append_item(&ss);
                    });
                }
                Err(e) => eprintln!("SQL ERROR: {:?}", e),
            }
        }

        fn remove_song(&self, pos: u32) {
            let Some(song_item) = self
                .listview
                .get_selected_items()
                .first()
                .cloned()
                .and_downcast::<SongObject>()
            else {
                return;
            };

            match Query::delete_song(song_item.into()) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("SQL ERROR: {:?}", e);
                    return;
                }
            };

            match Query::get_all_songs() {
                Ok(songs) => {
                    self.listview.remove_all();
                    songs.iter().for_each(|s| {
                        let ss: SongObject = s.clone().into();
                        self.listview.append_item(&ss);
                    });
                }
                Err(e) => {
                    eprintln!("SQL ERROR: {:?}", e);
                    return;
                }
            };

            self.listview.grab_focus();
        }
    }
}
glib::wrapper! {
    pub struct SearchSong(ObjectSubclass<imp::SearchSong>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for SearchSong {
    fn default() -> Self {
        glib::Object::new::<Self>()
    }
}

impl SearchSong {
    pub fn new() -> Self {
        glib::Object::new::<Self>()
    }

    fn emit_send_to_schedule(&self, data: &SlideManagerData) {
        self.emit_by_name::<()>(signals::SEND_TO_SCHEDULE, &[data]);
    }
    pub fn connect_send_to_schedule<F: Fn(&Self, &SlideManagerData) + 'static>(&self, f: F) {
        self.connect_closure(
            signals::SEND_TO_SCHEDULE,
            false,
            glib::closure_local!(|obj: &Self, data: &SlideManagerData| {
                f(obj, data);
            }),
        );
    }
    fn emit_send_to_preview(&self, data: &SlideManagerData) {
        self.emit_by_name::<()>(signals::SEND_TO_PREVIEW, &[data]);
    }

    pub fn connect_send_to_preview<F: Fn(&Self, &SlideManagerData) + 'static>(&self, f: F) {
        self.connect_closure(
            signals::SEND_TO_PREVIEW,
            false,
            glib::closure_local!(|obj: &Self, data: &SlideManagerData| {
                f(obj, data);
            }),
        );
    }
}
