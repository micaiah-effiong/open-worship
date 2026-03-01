pub mod download;

use std::cell::RefCell;

use gtk::gio::{ActionEntry, MenuItem, SimpleActionGroup};
use gtk::glib::{self, SignalHandlerId, clone};
use gtk::prelude::*;
use gtk::{MultiSelection, StringObject};

use crate::db::connection::BibleVerse;
use crate::db::query::Query;
use crate::dto;
use crate::parser::parser::{self, BibleReference};
use crate::utils::WidgetChildrenExt;
use crate::widgets::canvas::serialise::SlideManagerData;
use crate::widgets::search::scriptures::download::download_modal::DownloadBibleWindow;

mod signals {
    pub(super) const SEND_SCRIPTURES: &str = "send-scriptures";
    pub(super) const SEND_TO_SCHEDULE: &str = "send-to-schedule";
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
enum SearchMode {
    Evaluated(BibleReference),
    #[default]
    Fuzz,
}

mod imp {
    use std::sync::OnceLock;

    use crate::{
        dto::scripture::ScriptureObject, utils::ListViewExtra,
        widgets::canvas::serialise::SlideData,
    };

    use super::*;
    use gtk::{
        gio,
        glib::{
            SignalHandlerId,
            subclass::{
                Signal,
                object::{ObjectImpl, ObjectImplExt},
                types::{ObjectSubclass, ObjectSubclassExt},
            },
        },
        subclass::{
            box_::BoxImpl,
            widget::{
                CompositeTemplateCallbacksClass, CompositeTemplateClass,
                CompositeTemplateInitializingExt, WidgetClassExt, WidgetImpl,
            },
        },
    };

    #[derive(Default, Debug, gtk::CompositeTemplate)]
    #[template(resource = "/com/openworship/app/ui/search_scripture.ui")]
    pub struct SearchScripture {
        selection_signal_handler: RefCell<Option<SignalHandlerId>>,
        search_signal_handler: RefCell<Option<SignalHandlerId>>,
        translation: RefCell<String>,

        #[template_child]
        search_text: gtk::TemplateChild<gtk::SearchEntry>,
        #[template_child]
        listview: gtk::TemplateChild<gtk::ListView>,
        #[template_child]
        import_btn: gtk::TemplateChild<gtk::Button>,
        #[template_child]
        dropdown: gtk::TemplateChild<gtk::DropDown>,

        search_mode: RefCell<SearchMode>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SearchScripture {
        const NAME: &'static str = "SearchScripture";
        type Type = super::SearchScripture;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SearchScripture {
        fn constructed(&self) {
            self.parent_constructed();

            let listview = self.listview.clone();
            let model = gio::ListStore::new::<ScriptureObject>();
            let model = gtk::MultiSelection::new(Some(model));
            listview.set_model(Some(&model));

            let factory = gtk::SignalListItemFactory::new();
            listview.set_factory(Some(&factory));
            factory.connect_setup(move |_, list_item| {
                let li_widget = gtk::Label::builder()
                    .ellipsize(gtk::pango::EllipsizeMode::End)
                    .build();
                let base = gtk::Box::builder().build();
                base.append(&li_widget);

                let li = list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Needs to be ListItem");

                li.set_child(Some(&base));
            });

            factory.connect_bind(move |_, list_item| {
                let scripture_obj = list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Needs to be ListItem")
                    .item()
                    .and_downcast::<ScriptureObject>()
                    .expect("The item has to be an `Slide`.");
                let data = scripture_obj.item();

                let label = list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Needs to be ListItem")
                    .child()
                    .and_downcast::<gtk::Box>()
                    .expect("The child has to be a `Box`.")
                    .first_child()
                    .and_downcast::<gtk::Label>()
                    .expect("The first_child has to be a Label");

                let book_reference = format!("{}:{} \t{}", data.chapter, data.verse, data.text);
                let text = match scripture_obj.full_reference() {
                    true => format!("{} {book_reference}", data.book),
                    false => book_reference,
                };
                label.set_label(&text);
            });

            let translations = Self::get_bible_translations();
            if let Some(first) = translations.first() {
                self.load_initial_verses(first.to_string());
            }

            self.register_activate_selected();
            self.register_context_menu();
            self.register_drag();
            self.register_search_change();
            self.load_bible_translations(translations);
            self.register_translation_change();
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();

            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder(signals::SEND_SCRIPTURES)
                        .param_types([SlideManagerData::static_type()])
                        .build(),
                    Signal::builder(signals::SEND_TO_SCHEDULE)
                        .param_types([SlideManagerData::static_type()])
                        .build(),
                ]
            })
        }
    }
    impl WidgetImpl for SearchScripture {}
    impl BoxImpl for SearchScripture {}

    #[gtk::template_callbacks]
    impl SearchScripture {
        #[template_callback]
        fn handle_search_activate(&self, _: &gtk::SearchEntry) {
            self.send_to_preview();
            glib::g_message!("SearchScripture", "handle_search_activate");
        }

        #[template_callback]
        fn open_download_modal(&self, _: &gtk::Button) {
            glib::g_message!("SearchScripture", "open_download_modal");

            let tranlations = Self::get_bible_translations();

            let win = DownloadBibleWindow::new(tranlations);
            win.set_modal(false);
            win.connect_new_translation(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_, t| {
                    glib::g_message!("SearchScripture", "WIN = new translation: {t}");
                    imp.new_translation(t);
                }
            ));
            win.connect_reload_translation(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    glib::g_message!("SearchScripture", "WIN = reload translation");
                    imp.reload_translations();
                }
            ));

            win.present();
        }
    }

    /// events
    impl SearchScripture {
        fn change_translation(&self, t: String) {
            println!("SCRP UPDATE 2");
            self.translation.replace(t.clone());

            println!("SCRP UPDATE \n {:?}, {:?}", t, self.translation.clone());
            self.search_bible(
                self.search_text.text().to_string(),
                &self.translation.borrow(),
                &self.listview.clone(),
            );
        }

        fn reload_translations(&self) {
            let t = Self::get_bible_translations();
            self.load_bible_translations(t);
        }
        fn new_translation(&self, t: String) {
            if self.translation.borrow().is_empty() {
                self.translation.replace(t.clone());
                self.load_initial_verses(t);
            }
            self.reload_translations();
        }
        fn send_to_preview(&self) {
            let selected_verses = Self::get_selected_scripture(&self.listview);

            if selected_verses.is_empty() {
                return;
            }

            let selected_slide_data: Vec<SlideData> = selected_verses
                .clone()
                .iter()
                .map(|v| v.clone().into())
                .collect();
            let mut payload = SlideManagerData::new(0, 0, selected_slide_data);

            match self.search_mode.borrow().clone() {
                SearchMode::Evaluated(evaluated) => {
                    let verse = Self::compress_verses(
                        &selected_verses
                            .iter()
                            .map(|t| t.item().verse)
                            .collect::<Vec<_>>(),
                    );

                    let new_text = format!(
                        "{} {}:{}",
                        evaluated.book,
                        evaluated.chapter,
                        verse.join(",")
                    );

                    payload.title = new_text.clone();
                }
                SearchMode::Fuzz => (),
            };

            self.obj().emit_send_scriptures(payload);
        }
    }

    /// functions
    impl SearchScripture {
        fn get_bible_translations() -> Vec<std::string::String> {
            match Query::get_translations() {
                Ok(l) => l,
                Err(e) => {
                    eprintln!(
                        "SQL ERROR: An error occured while getting translations {:?}",
                        e
                    );
                    vec![]
                }
            }
        }
        fn get_selected_scripture(listview: &gtk::ListView) -> Vec<ScriptureObject> {
            let items = listview
                .get_selected_items()
                .iter()
                .filter_map(|v| v.downcast_ref::<ScriptureObject>().cloned())
                .collect::<Vec<_>>();

            items
        }
        fn get_initial_scriptures(translation: String) -> Result<Vec<BibleVerse>, rusqlite::Error> {
            Query::search_by_chapter_query(translation, String::from("Genesis"), 1)
        }

        fn load_bible_translations(&self, translations: Vec<String>) {
            if translations.is_empty() {
                return;
            }
            let dropdown = self.dropdown.clone();

            // TODO: ensure previously selected translation is not altered

            let translation_slice = translations
                .iter()
                .map(|i| i.as_str())
                .collect::<Vec<&str>>()
                .as_slice()
                .to_owned();
            let str_list = gtk::StringList::new(&translation_slice);
            let single_model = gtk::SingleSelection::new(Some(str_list.clone()));
            dropdown.set_model(Some(&single_model));
            dropdown.set_selected(0);

            // update model translation
            if self.translation.borrow().is_empty() {
                if let Some(t) = translations.first() {
                    *self.translation.borrow_mut() = t.to_string()
                }
            } else {
                for i in 0..str_list.n_items() {
                    let item = match str_list.string(i) {
                        Some(i) => i.to_string(),
                        None => continue,
                    };

                    if item == self.translation.borrow().clone() {
                        dropdown.set_selected(i);
                    }
                }
            }
        }

        fn register_translation_change(&self) {
            self.dropdown.connect_selected_item_notify(glib::clone!(
                #[weak(rename_to=imp)]
                self,
                move |dropdown| {
                    let item = match dropdown.selected_item() {
                        Some(i) => i,
                        None => return,
                    };

                    let item = match item.downcast::<StringObject>() {
                        Ok(i) => i,
                        Err(_) => return,
                    };

                    println!("CONNECT_DIRECTION_CHANGED {:?}", item);
                    imp.change_translation(item.into());
                }
            ));

            // dropdown.connect_selected_notify(|dropdown| {
            //     let a = dropdown.selected_item();
            //     println!("CONNECT_DIRECTION_CHANGED {:?}", a);
            // });
        }

        fn parser_bible_reference(search_text: &str) -> Option<BibleReference> {
            let p = parser::Parser::parser(search_text.to_string());
            if let Some(p) = p {
                let evaluated = p.eval();
                return Some(evaluated);
            }

            None
        }

        fn search_bible(
            &self,
            search_text: String,
            bible_translation: &str,
            listview: &gtk::ListView,
        ) -> std::vec::Vec<u32> {
            let mut verse_index = Vec::new();
            if bible_translation.is_empty() {
                eprintln!("NO TRANSLATION");
                return verse_index;
            }

            let t = bible_translation.to_owned();
            let (verses, mut evaluated) = match Self::parser_bible_reference(&search_text) {
                Some(evaluated) => {
                    self.search_mode
                        .replace(SearchMode::Evaluated(evaluated.clone()));
                    println!("CONNECT_SEARCH_CHANGED {:?}", evaluated);
                    let verses = Query::search_by_chapter_query(
                        t.clone(),
                        evaluated.book.clone(),
                        evaluated.chapter,
                    );

                    (verses, Some(evaluated))
                }
                None => {
                    self.search_mode.replace(SearchMode::Fuzz);
                    let verses =
                        Query::search_by_partial_text_query(t.clone(), search_text.clone());
                    (verses, None)
                }
            };

            let verses = match verses {
                Ok(vs) => vs,
                Err(x) => {
                    println!("SQL ERROR: \n{:?}", x);
                    return verse_index;
                }
            };

            listview.remove_all();

            if let Some(e) = &mut evaluated {
                if let Some(v) = verses.first() {
                    e.book = v.book.clone();
                }
                e.verses.iter().for_each(|v| {
                    verse_index.push(*v);
                });
            }

            verses.iter().for_each(|verse| {
                let scripture = dto::Scripture {
                    book: verse.book.clone(),
                    chapter: verse.chapter,
                    verse: verse.verse,
                    text: verse.text.clone(),
                    translation: t.clone(),
                };
                let item = ScriptureObject::new(scripture, evaluated.is_none());
                listview.append_item(&item);
            });

            verse_index
        }

        fn select_list_items(
            listview: &gtk::ListView,
            selection_model: &MultiSelection,
            items: Vec<u32>,
        ) {
            if items.is_empty() {
                return;
            }

            selection_model.unselect_all();
            for index in items.iter() {
                selection_model.select_item(*index, false);
            }

            let list = listview.children().collect::<Vec<_>>();

            if let Some(vli) = items.first() {
                // subtract here since list.get uses zero based index
                match list.get(*vli as usize) {
                    Some(li) => li.grab_focus(),
                    None => false,
                };
            }
        }

        fn register_search_change(&self) {
            let search_field = self.search_text.clone();

            let handle_id = search_field.connect_changed(glib::clone!(
                #[weak(rename_to=imp)]
                self,
                move |se| {
                    let verses = imp.search_bible(
                        se.text().to_string(),
                        &imp.translation.borrow(),
                        &imp.listview,
                    );

                    let lv = imp.listview.clone();
                    let Some(model) = lv.model().and_downcast::<gtk::MultiSelection>() else {
                        return;
                    };

                    if let Some(selection_handler_id) = imp.selection_signal_handler.take() {
                        model.block_signal(&selection_handler_id);
                        Self::select_list_items(
                            &lv,
                            &model,
                            verses.iter().map(|v| v.saturating_sub(1)).collect(),
                        );
                        se.grab_focus();
                        model.unblock_signal(&selection_handler_id);
                        // send to preview
                        imp.send_to_preview();
                        imp.selection_signal_handler
                            .replace(Some(selection_handler_id));
                    }
                }
            ));

            self.search_signal_handler.replace(Some(handle_id));
        }

        fn register_drag(&self) {
            let listview = self.listview.clone();
            let text_entry = self.search_text.clone();

            let drag_source = gtk::DragSource::new();
            drag_source.set_actions(gtk::gdk::DragAction::COPY);
            drag_source.connect_prepare({
                let listview = listview.clone();
                move |_, _, _| {
                    let payload = Self::get_selected_scripture(&listview);

                    let payload = SlideManagerData::from_list(
                        text_entry.text().to_string(),
                        0,
                        payload.iter().map(|t| t.item().screen_display()).collect(),
                        None,
                    );

                    let content = gtk::gdk::ContentProvider::for_value(&payload.to_value());
                    Some(content)
                }
            });

            // drag_source.connect_drag_begin({
            //     let lv = listview.clone();
            //     move |ds, drag| {
            //         // let Some(model) = lv.model().and_downcast::<gtk::MultiSelection>() else {
            //         //     return;
            //         // };
            //         //
            //         // let li = lv
            //         //     .children()
            //         //     .filter_map(|v| {
            //         //         (v.accessible_role() == gtk::AccessibleRole::ListItem).then_some(v)
            //         //     })
            //         //     .collect::<Vec<_>>();
            //         //
            //         // let selections = model.selection();
            //         // let mut selected_w = Vec::new();
            //         // for (i, w) in li.iter().enumerate() {
            //         //     if selections.contains(i as u32) {
            //         //         ds.set_icon(w.snap().as_ref(), 0, 0);
            //         //         selected_w.push(w);
            //         //     }
            //         // }
            //
            //         // let item_text = item_text.to_string();
            //         // drag.set_icon_name(Some("document-properties"), 0, 0);
            //     }
            // });

            listview.add_controller(drag_source);
        }

        fn register_context_menu(&self) {
            // action entries
            let add_to_schedule_action = ActionEntry::builder("add-to-schedule")
                .activate(clone!(
                    #[weak(rename_to=imp)]
                    self,
                    move |_, _, _| {
                        let listview = imp.listview.clone();
                        let text_entry = imp.search_text.clone();
                        let payload = Self::get_selected_scripture(&listview);

                        let payload = SlideManagerData::from_list(
                            text_entry.text().to_string(),
                            0,
                            payload.iter().map(|t| t.item().screen_display()).collect(),
                            None,
                        );

                        imp.obj().emit_send_to_schedule(payload);
                    }
                ))
                .build();

            // action group
            let action_group = SimpleActionGroup::new();
            action_group.add_action_entries([add_to_schedule_action]);

            // popover menu
            let menu = gtk::gio::Menu::new();
            {
                let add_to_schedule_menu_item =
                    MenuItem::new(Some("Add to schedule"), Some("scripture.add-to-schedule"));
                menu.insert_item(0, &add_to_schedule_menu_item);
            }

            let popover_menu = gtk::PopoverMenu::from_model(Some(&menu));
            popover_menu.set_parent(&self.listview.clone());
            popover_menu.set_has_arrow(false);
            popover_menu.set_halign(gtk::Align::Start);
            popover_menu.set_valign(gtk::Align::Start);

            let gesture = gtk::GestureClick::new();
            gesture.set_button(gtk::gdk::BUTTON_SECONDARY);
            gesture.connect_pressed(clone!(move |gc, _, x, y| {
                let rect = gtk::gdk::Rectangle::new(x as i32, y as i32, 0, 0);
                popover_menu.set_pointing_to(Some(&rect));
                popover_menu.popup();
                gc.set_state(gtk::EventSequenceState::Claimed);
            }));

            self.listview.add_controller(gesture);
            self.listview
                .insert_action_group("scripture", Some(&action_group));
        }

        fn register_activate_selected(&self) {
            let Some(model) = self.listview.model() else {
                return;
            };

            let connect_change_fn = |imp: glib::subclass::ObjectImplRef<SearchScripture>| {
                imp.send_to_preview();
                let listview = imp.listview.clone();
                let text_entry = imp.search_text.clone();

                let selected_verses = Self::get_selected_scripture(&listview);

                // update text entry
                match imp.search_mode.borrow().clone() {
                    SearchMode::Evaluated(evaluated) => {
                        let verse = Self::compress_verses(
                            &selected_verses
                                .iter()
                                .map(|t| t.item().verse)
                                .collect::<Vec<_>>(),
                        );

                        let new_text = format!(
                            "{} {}:{}",
                            evaluated.book,
                            evaluated.chapter,
                            verse.join(",")
                        );

                        let search_signal_handler = imp.search_signal_handler.take();
                        if let Some(handler_id) = search_signal_handler {
                            text_entry.block_signal(&handler_id);
                            text_entry.set_text(&new_text);
                            text_entry.unblock_signal(&handler_id);
                            imp.search_signal_handler.replace(Some(handler_id));
                        }
                    }
                    SearchMode::Fuzz => (),
                };
            };

            self.listview.connect_activate(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_, _| connect_change_fn(imp)
            ));
            let handle_id = model.connect_selection_changed(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_m, _pos, _n_items| connect_change_fn(imp)
            ));

            self.selection_signal_handler.replace(Some(handle_id));
        }

        fn load_initial_verses(&self, translation: String) {
            let listview = self.listview.clone();

            let verses = Self::get_initial_scriptures(translation.clone()).unwrap_or_default();

            listview.remove_all();
            for (i, verse) in verses.iter().enumerate() {
                let scripture = dto::Scripture {
                    book: verse.book.clone(),
                    chapter: verse.chapter,
                    verse: verse.verse,
                    text: verse.text.clone(),
                    translation: translation.clone(),
                };
                let item = ScriptureObject::new(scripture, false);
                listview.append_item(&item);

                if i == 0 {
                    let text = format!("{} {}:{}", verse.book, verse.chapter, verse.verse);
                    self.search_text.set_text(&text);

                    if let Some(eval) = Self::parser_bible_reference(&text) {
                        self.search_mode.replace(SearchMode::Evaluated(eval));
                    }
                }
            }
        }

        fn compress_verses(list: &[u32]) -> Vec<String> {
            let mut result = Vec::new();
            if list.is_empty() {
                return result;
            }

            let mut sorted_list = list.to_owned();
            sorted_list.sort();

            let mut start = sorted_list.first().unwrap();
            let mut prev = sorted_list.first().unwrap();

            for curr in sorted_list.iter().skip(1) {
                if *curr != prev + 1 {
                    if start != prev {
                        result.push(format!("{start}-{prev}"));
                    } else {
                        result.push(start.to_string());
                    }
                    start = curr;
                }
                prev = curr;
            }

            if start != prev {
                result.push(format!("{start}-{prev}"));
            } else {
                result.push(start.to_string());
            }

            result
        }
    }
}

glib::wrapper! {
    pub struct SearchScripture(ObjectSubclass<imp::SearchScripture>)
    @extends  gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for SearchScripture {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl SearchScripture {
    pub fn new() -> Self {
        glib::Object::new()
    }

    //
    fn emit_send_to_schedule(&self, data: SlideManagerData) {
        self.emit_by_name::<()>(signals::SEND_TO_SCHEDULE, &[&data]);
    }
    pub fn connect_send_to_schedule<F: Fn(&Self, &SlideManagerData) + 'static>(
        &self,
        f: F,
    ) -> SignalHandlerId {
        self.connect_closure(
            signals::SEND_TO_SCHEDULE,
            false,
            glib::closure_local!(|obj: &Self, data: &SlideManagerData| f(obj, data)),
        )
    }

    //
    fn emit_send_scriptures(&self, data: SlideManagerData) {
        self.emit_by_name::<()>(signals::SEND_SCRIPTURES, &[&data]);
    }
    pub fn connect_send_scriptures<F: Fn(&Self, &SlideManagerData) + 'static>(
        &self,
        f: F,
    ) -> SignalHandlerId {
        self.connect_closure(
            signals::SEND_SCRIPTURES,
            false,
            glib::closure_local!(|obj: &Self, data: &SlideManagerData| f(obj, data)),
        )
    }
}
