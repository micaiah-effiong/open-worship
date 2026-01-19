mod download;
mod list_item;

use std::cell::RefCell;
use std::rc::Rc;

use download::download_modal::{
    DownloadBibleInit, DownloadBibleInput, DownloadBibleModel, DownloadBibleOutput,
};
use gtk::gio::{ActionEntry, MenuItem, SimpleActionGroup};
use gtk::glib::{SignalHandlerId, clone};
use gtk::prelude::*;
use gtk::{BitsetIter, ListView, MultiSelection, StringObject};
use list_item::ScriptureListItem;
use relm4::prelude::*;
use relm4::typed_view::list::TypedListView;

use crate::db::connection::{BibleVerse, DatabaseConnection};
use crate::db::query::Query;
use crate::parser::parser::{self, BibleReference};
use crate::{dto, utils};

#[derive(Debug)]
pub enum SearchScriptureInput {
    ChangeTranslation(String),
    ReloadTranlations,
    NewTranslation(String),
    SendToPreview,

    //
    OpenDownload,
}

#[derive(Debug)]
pub enum SearchScriptureOutput {
    SendScriptures(dto::ListPayload),
    SendToSchedule(dto::ListPayload),
}

#[derive(Debug)]
pub struct SearchScriptureModel {
    list_view_wrapper: Rc<RefCell<TypedListView<ScriptureListItem, MultiSelection>>>,
    search_text: gtk::SearchEntry,
    dropdown: gtk::DropDown,
    translation: Rc<RefCell<String>>,
    download_bible_modal: relm4::Controller<DownloadBibleModel>,
    selection_signal_handler: Rc<RefCell<Option<SignalHandlerId>>>,
    search_signal_handler: Rc<RefCell<Option<SignalHandlerId>>>,
}

impl SearchScriptureModel {}

pub struct SearchScriptureInit {}

impl SearchScriptureModel {
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

    fn load_bible_translations(&mut self, dropdown: &gtk::DropDown, translations: Vec<String>) {
        if translations.is_empty() {
            return;
        }

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

    fn register_translation_change(
        &mut self,
        dropdown: &gtk::DropDown,
        sender: &ComponentSender<SearchScriptureModel>,
    ) {
        let sender_clone = sender.clone();
        dropdown.connect_selected_item_notify(move |dropdown| {
            let item = match dropdown.selected_item() {
                Some(i) => i,
                None => return,
            };

            let item = match item.downcast::<StringObject>() {
                Ok(i) => i,
                Err(_) => return,
            };

            println!("CONNECT_DIRECTION_CHANGED {:?}", item);
            sender_clone.input(SearchScriptureInput::ChangeTranslation(String::from(item)));
        });

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
        search_text: String,
        bible_translation: &str,
        list_view_wrapper: Rc<RefCell<TypedListView<ScriptureListItem, MultiSelection>>>,
    ) -> std::vec::Vec<u32> {
        let mut verse_index = Vec::new();
        if bible_translation.is_empty() {
            eprintln!("NO TRANSLATION");
            return verse_index;
        }

        let t = bible_translation.to_owned();
        let (verses, evaluated) = match SearchScriptureModel::parser_bible_reference(&search_text) {
            Some(evaluated) => {
                println!("CONNECT_SEARCH_CHANGED {:?}", evaluated);
                let verses = Query::search_by_chapter_query(
                    t.clone(),
                    evaluated.book.clone(),
                    evaluated.chapter,
                );
                (verses, Some(evaluated))
            }
            None => {
                let verses = Query::search_by_partial_text_query(t.clone(), search_text);
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

        list_view_wrapper.borrow_mut().clear();
        if let Some(e) = &evaluated {
            e.verses.iter().for_each(|v| {
                verse_index.push(*v);
            });
        }

        verses.iter().for_each(|verse| {
            list_view_wrapper.borrow_mut().append(ScriptureListItem {
                full_reference: evaluated.is_none(),
                data: dto::Scripture {
                    book: verse.book.clone(),
                    chapter: verse.chapter,
                    verse: verse.verse,
                    text: verse.text.clone(),
                    translation: t.clone(),
                },
            });
        });

        verse_index
    }

    fn select_list_items(list_view: ListView, selection_model: MultiSelection, items: Vec<u32>) {
        selection_model.unselect_all();
        for index in items.iter() {
            selection_model.select_item(*index, false);
        }

        let list_view = list_view.clone();
        let list = match list_view.first_child() {
            Some(li) => utils::widget_to_vec(&li),
            None => return,
        };

        if let Some(vli) = items.first() {
            // subtract here since list.get uses zero based index
            match list.get(*vli as usize) {
                Some(li) => li.grab_focus(),
                None => false,
            };
        }
    }

    fn register_search_change(&mut self, sender: ComponentSender<Self>) {
        let search_field = self.search_text.clone();
        let list_view_wrapper = self.list_view_wrapper.clone();
        let bible_translation = self.translation.clone();
        let selection_signal_handler = self.selection_signal_handler.clone();

        let handle_id = search_field.connect_changed(clone!(
            #[strong]
            bible_translation,
            #[strong]
            list_view_wrapper,
            #[strong]
            selection_signal_handler,
            #[strong]
            sender,
            move |se| {
                let verses = SearchScriptureModel::search_bible(
                    se.text().to_string(),
                    &bible_translation.borrow(),
                    list_view_wrapper.clone(),
                );

                let lv = list_view_wrapper.borrow().view.clone();
                let sm = list_view_wrapper.borrow().selection_model.clone();

                if let Some(handler_id) = selection_signal_handler.borrow().as_ref() {
                    list_view_wrapper
                        .borrow()
                        .selection_model
                        .block_signal(handler_id);
                    SearchScriptureModel::select_list_items(
                        lv,
                        sm,
                        verses.iter().map(|v| v.saturating_sub(1)).collect(),
                    );
                    se.grab_focus();
                    list_view_wrapper
                        .borrow()
                        .selection_model
                        .unblock_signal(handler_id);
                    // send to preview
                    sender.input(SearchScriptureInput::SendToPreview);
                }
            }
        ));

        *self.search_signal_handler.borrow_mut() = Some(handle_id);
    }

    fn register_context_menu(&mut self, sender: &ComponentSender<Self>) {
        let list_view_wrapper = self.list_view_wrapper.clone();
        let list_view = self.list_view_wrapper.borrow().view.clone();
        let text_entry = self.search_text.clone();

        // action entries
        let add_to_schedule_action = ActionEntry::builder("add-to-schedule")
            .activate(clone!(
                #[strong]
                list_view_wrapper,
                #[strong]
                text_entry,
                #[strong]
                sender,
                move |_, _, _| {
                    let payload =
                        SearchScriptureModel::get_selected_scripture(&list_view_wrapper.borrow());

                    let payload = dto::ListPayload::new(
                        text_entry.text().to_string(),
                        0,
                        payload.iter().map(|t| t.data.screen_display()).collect(),
                        None,
                    );

                    let _ = sender.output(SearchScriptureOutput::SendToSchedule(payload));
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
        popover_menu.set_parent(&list_view);
        popover_menu.set_has_arrow(false);
        popover_menu.set_align(gtk::Align::Start);

        let gesture = gtk::GestureClick::new();
        gesture.set_button(gtk::gdk::BUTTON_SECONDARY);
        gesture.connect_pressed(clone!(move |gc, _, x, y| {
            let rect = gtk::gdk::Rectangle::new(x as i32, y as i32, 0, 0);
            popover_menu.set_pointing_to(Some(&rect));
            popover_menu.popup();
            gc.set_state(gtk::EventSequenceState::Claimed);
        }));

        list_view.add_controller(gesture);
        list_view.insert_action_group("scripture", Some(&action_group));
    }

    fn register_activate_selected(&mut self, sender: &ComponentSender<Self>) {
        let typed_list = self.list_view_wrapper.clone();
        let text_entry = self.search_text.clone();
        let search_signal_handler = self.search_signal_handler.clone();

        let handle_id = typed_list
            .borrow()
            .selection_model
            .connect_selection_changed(clone!(
                #[strong]
                typed_list,
                #[strong]
                sender,
                #[strong]
                text_entry,
                #[strong]
                search_signal_handler,
                move |_m, _pos, _n_items| {
                    // send selected to preview
                    let selected_verses =
                        SearchScriptureModel::get_selected_scripture(&typed_list.borrow());

                    let payload = dto::ListPayload::new(
                        text_entry.text().to_string(),
                        0,
                        selected_verses
                            .iter()
                            .map(|t| t.data.screen_display())
                            .collect(),
                        None,
                    );
                    let _ = sender.output(SearchScriptureOutput::SendScriptures(payload));

                    // update text entry
                    let evaluated = SearchScriptureModel::parser_bible_reference(
                        text_entry.text().to_string().as_ref(),
                    );
                    if let Some(evaluated) = evaluated {
                        let verse = SearchScriptureModel::compress_verses(
                            &selected_verses
                                .iter()
                                .map(|t| t.data.verse)
                                .collect::<Vec<_>>(),
                        );

                        let new_text = format!(
                            "{} {}:{}",
                            evaluated.book,
                            evaluated.chapter,
                            verse.join(",")
                        );

                        if let Some(handler_id) = search_signal_handler.borrow().as_ref() {
                            text_entry.block_signal(handler_id);
                            text_entry.set_text(&new_text);
                            text_entry.unblock_signal(handler_id);
                        }
                    }
                }
            ));

        *self.selection_signal_handler.borrow_mut() = Some(handle_id);
    }

    fn get_initial_scriptures(translation: String) -> Result<Vec<BibleVerse>, rusqlite::Error> {
        Query::search_by_chapter_query(translation, String::from("Genesis"), 1)
    }

    fn get_selected_scripture(
        typed_list: &TypedListView<ScriptureListItem, MultiSelection>,
    ) -> Vec<ScriptureListItem> {
        let selections = typed_list.selection_model.selection();
        let mut selected_verses = Vec::with_capacity(selections.size() as usize);

        if let Some((iter, initial_item)) = BitsetIter::init_first(&selections) {
            let mut iter_list = iter.collect::<Vec<u32>>();
            iter_list.insert(0, initial_item);

            iter_list.into_iter().for_each(|x| {
                if let Some(m) = typed_list.get(x) {
                    selected_verses.push(m.borrow().clone());
                }
            });
        };

        selected_verses
    }

    fn load_initial_verses(&mut self, translation: String) {
        let list_view_wrapper = self.list_view_wrapper.clone();

        let verses =
            SearchScriptureModel::get_initial_scriptures(translation.clone()).unwrap_or_default();

        list_view_wrapper.borrow_mut().clear();
        for (i, verse) in verses.iter().enumerate() {
            list_view_wrapper.borrow_mut().append(ScriptureListItem {
                full_reference: false,
                data: dto::Scripture {
                    book: verse.book.clone(),
                    chapter: verse.chapter,
                    verse: verse.verse,
                    text: verse.text.clone(),
                    translation: translation.clone(),
                },
            });

            if i == 0 {
                self.search_text.set_text(&format!(
                    "{} {}:{}",
                    verse.book.clone(),
                    verse.chapter,
                    verse.verse
                ));
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

#[relm4::component(pub)]
impl SimpleComponent for SearchScriptureModel {
    type Init = SearchScriptureInit;
    type Output = SearchScriptureOutput;
    type Input = SearchScriptureInput;

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

                #[local_ref]
                append = &search_text -> gtk::SearchEntry {
                    set_hexpand: true,
                    connect_activate[sender] => move |_| {
                        sender.input(SearchScriptureInput::SendToPreview);
                    }
                },
            },

            gtk::ScrolledWindow {
                set_vexpand: true,

                #[wrap(Some)]
                #[local_ref]
                set_child = &list_view -> gtk::ListView { }
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                #[name="import_btn"]
                gtk::Button{
                    set_icon_name:"plus",
                    connect_clicked[sender] => move |_|{
                        sender.input(SearchScriptureInput::OpenDownload);
                    }
                },

                #[local_ref]
                append = &dropdown -> gtk::DropDown { }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let typed_list_view: TypedListView<ScriptureListItem, MultiSelection> =
            TypedListView::new();

        let list_view_wrapper = Rc::new(RefCell::new(typed_list_view));
        let dropdown = gtk::DropDown::from_strings(&[]);

        let download_modal = DownloadBibleModel::builder()
            .launch(DownloadBibleInit {
                installed_translations: SearchScriptureModel::get_bible_translations(),
            })
            .forward(
                sender.input_sender(),
                SearchScriptureModel::convert_download_bible_response,
            );

        let mut model = SearchScriptureModel {
            list_view_wrapper: list_view_wrapper.clone(),
            search_text: gtk::SearchEntry::new(),
            translation: Rc::new(RefCell::new(String::new())),
            dropdown: dropdown.clone(),
            download_bible_modal: download_modal,
            selection_signal_handler: Rc::new(RefCell::new(None)),
            search_signal_handler: Rc::new(RefCell::new(None)),
        };

        let list_view = model.list_view_wrapper.borrow().view.clone();
        let search_text = model.search_text.clone();
        let widgets = view_output!();

        model.register_activate_selected(&sender);
        model.register_context_menu(&sender);

        let translations = SearchScriptureModel::get_bible_translations();
        if let Some(first) = translations.first() {
            model.load_initial_verses(first.to_string());
        }

        model.register_search_change(sender.clone());
        model.load_bible_translations(&widgets.dropdown, translations);
        model.register_translation_change(&widgets.dropdown, &sender);

        relm4::ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            SearchScriptureInput::ChangeTranslation(t) => {
                println!("SCRP UPDATE 2");
                *self.translation.borrow_mut() = t.clone();

                println!("SCRP UPDATE \n {:?}, {:?}", t, self.translation.clone());
                SearchScriptureModel::search_bible(
                    self.search_text.text().to_string(),
                    &self.translation.borrow(),
                    self.list_view_wrapper.clone(),
                );
            }
            SearchScriptureInput::ReloadTranlations => {
                let t = SearchScriptureModel::get_bible_translations();
                self.load_bible_translations(&self.dropdown.clone(), t);
            }

            SearchScriptureInput::NewTranslation(t) => {
                if self.translation.borrow().is_empty() {
                    self.translation.replace(t.clone());
                    self.load_initial_verses(t);
                }
                sender.input(SearchScriptureInput::ReloadTranlations);
            }
            SearchScriptureInput::OpenDownload => {
                self.download_bible_modal.emit(DownloadBibleInput::Open);
            }
            SearchScriptureInput::SendToPreview => {
                let selected_verses =
                    SearchScriptureModel::get_selected_scripture(&self.list_view_wrapper.borrow());

                let list_verse = selected_verses
                    .iter()
                    .map(|t| t.data.screen_display())
                    .collect();
                let payload =
                    dto::ListPayload::new(self.search_text.text().to_string(), 0, list_verse, None);

                let _ = sender.output(SearchScriptureOutput::SendScriptures(payload));
            }
        }
    }
}

impl SearchScriptureModel {
    fn convert_download_bible_response(res: DownloadBibleOutput) -> SearchScriptureInput {
        match res {
            DownloadBibleOutput::NewTranslation(t) => SearchScriptureInput::NewTranslation(t),
            DownloadBibleOutput::ReloadTranslation => SearchScriptureInput::ReloadTranlations,
        }
    }
}
