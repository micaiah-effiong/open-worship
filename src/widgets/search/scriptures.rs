mod list_item;

use std::{cell::RefCell, io::Write, rc::Rc};

use download::BibleDownloadListItem;
use gtk::{
    gio::{ActionEntry, MenuItem, SimpleActionGroup},
    glib::clone,
    prelude::*,
    MultiSelection, StringObject,
};
use list_item::ScriptureListItem;
use relm4::{prelude::*, typed_view::list::TypedListView};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::{
    config::AppConfigDir,
    db::{
        connection::{BibleTranslation, BibleVerse, DatabaseConnection},
        query::Query,
    },
    dto,
    parser::parser,
    widgets::util,
};

#[derive(Debug)]
pub enum SearchScriptureInput {
    ChangeTranslation(String),
    ReloadTranlations,
    NewTranslation(String),
}

#[derive(Debug)]
pub enum SearchScriptureOutput {
    SendScriptures(dto::ListPayload),
    SendToSchedule(dto::ListPayload),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BibleDownload {
    pub name: String,
    pub download_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SearchScriptureModel {
    list_view_wrapper: Rc<RefCell<TypedListView<ScriptureListItem, MultiSelection>>>,
    search_text: gtk::SearchEntry,
    dropdown: gtk::DropDown,
    translation: Rc<RefCell<String>>,
    db_connection: Rc<RefCell<DatabaseConnection>>,
}

impl SearchScriptureModel {}

pub struct SearchScriptureInit {
    pub db_connection: Rc<RefCell<DatabaseConnection>>,
}

impl SearchScriptureModel {
    fn get_bible_translations(db: Rc<RefCell<DatabaseConnection>>) -> Vec<std::string::String> {
        let conn = &db.borrow().connection;
        let list = match Query::get_translations(conn) {
            Ok(l) => l,
            Err(e) => {
                eprintln!(
                    "SQL ERROR: An error occured while getting translations {:?}",
                    e
                );
                return vec![];
            }
        };

        return list;
    }

    fn load_bible_translations(&mut self, dropdown: &gtk::DropDown, translations: Vec<String>) {
        if translations.len() < 1 {
            println!("NO TRANSLATION TO LOAD  ",);
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
            match translations.first() {
                Some(t) => *self.translation.borrow_mut() = t.to_string(),
                None => return,
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

    fn register_import_bible(
        &mut self,
        btn: &gtk::Button,
        translations: Vec<String>,
        db: Rc<RefCell<DatabaseConnection>>,
        sender: ComponentSender<Self>,
    ) {
        let conn = db.clone();
        let mut translation_map: std::collections::HashMap<String, bool> =
            std::collections::HashMap::new();

        translations.iter().for_each(|i| {
            translation_map.insert(i.to_string(), true);
        });

        btn.connect_clicked(clone!(
            #[strong]
            conn,
            #[strong]
            sender,
            move |_btn| {
                // TODO: prevent mutiple import windows
                let win = gtk::Window::new();
                win.set_default_height(256);
                win.set_default_width(256);

                let bible_src = include_str!("../../../bible_download_path.json");
                let download_list_result = serde_json::from_str::<Vec<BibleDownload>>(bible_src);

                let mut list = TypedListView::<BibleDownloadListItem, MultiSelection>::new();

                if let Ok(download_list) = download_list_result {
                    for item in download_list {
                        if item.download_url != None {
                            let item_name = item.name.clone();
                            let item_name = item_name.split(".").collect::<Vec<&str>>();
                            if let Some(name) = item_name.get(0) {
                                let name = name.to_string();
                                list.append(BibleDownloadListItem {
                                    data: item.clone(),
                                    conn: conn.clone(),
                                    already_added: translation_map.contains_key(&name),
                                    parent_sender: sender.clone(),
                                });
                            }
                        }
                    }
                }

                let boxx = gtk::Box::new(gtk::Orientation::Vertical, 2);
                let h_boxx = gtk::Box::new(gtk::Orientation::Horizontal, 2);

                let scroll = gtk::ScrolledWindow::new();
                scroll.set_child(Some(&list.view));
                scroll.set_vexpand(true);
                boxx.append(&scroll);
                boxx.append(&h_boxx);

                let btn = gtk::Button::new();
                btn.set_label("Done");
                h_boxx.append(&btn);

                let list = Rc::new(RefCell::new(list));

                btn.connect_clicked(clone!(
                    #[strong]
                    conn,
                    #[strong]
                    sender,
                    move |btn| {
                        gtk::glib::spawn_future_local(clone!(
                            #[strong]
                            list,
                            #[strong]
                            btn,
                            #[strong]
                            conn,
                            #[strong]
                            sender,
                            async move {
                                let list = list.borrow();
                                let selection = list.selection_model.selection();

                                let mut bible_list = Vec::new();
                                for i in 0..selection.size() {
                                    let i = selection.nth(i as u32);

                                    if let Some(item) = list.get(i as u32) {
                                        let item = item.borrow().to_owned().data;
                                        bible_list.push(item.clone());
                                        let installed_translation = import_bible(&item, conn).await;
                                        if let Some(installed_t) = installed_translation {
                                            sender.input(SearchScriptureInput::NewTranslation(
                                                installed_t,
                                            ));
                                        }
                                        break;
                                    }
                                }

                                match btn.toplevel_window() {
                                    Some(w) => w.close(),
                                    None => (),
                                }
                            }
                        ));
                    }
                ));

                win.connect_close_request(clone!(
                    #[strong]
                    sender,
                    move |_| {
                        sender.input(SearchScriptureInput::ReloadTranlations);
                        return gtk::glib::Propagation::Proceed;
                    }
                ));

                win.set_child(Some(&boxx));
                win.present();
            }
        ));
    }

    fn search_bible(
        search_text: String,
        bible_translation: &String,
        db: Rc<RefCell<DatabaseConnection>>,
        list_view_wrapper: Rc<RefCell<TypedListView<ScriptureListItem, MultiSelection>>>,
    ) {
        if bible_translation.is_empty() {
            eprintln!("NO TRANSLATION");
            return;
        }

        let p = parser::Parser::parser(search_text.clone());
        let t = bible_translation.clone().to_string();
        let (verses, evaluated) = match p {
            Some(p) => {
                let evaluated = p.eval();
                println!("CONNECT_SEARCH_CHANGED {:?}", evaluated);
                let verses = Query::search_by_chapter_query(
                    &db.borrow().connection,
                    t.clone(),
                    evaluated.book.clone(),
                    evaluated.chapter.clone(),
                );
                (verses, Some(evaluated))
            }
            None => {
                let verses = Query::search_by_partial_text_query(
                    &db.borrow().connection,
                    t.clone(),
                    search_text,
                );
                (verses, None)
            }
        };

        let verses = match verses {
            Ok(vs) => vs,
            Err(x) => {
                println!("SQL ERROR: \n{:?}", x);
                return;
            }
        };

        list_view_wrapper.borrow_mut().clear();
        for verse in verses {
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
        }

        /* select verse in listview */

        let list_model = list_view_wrapper.borrow().selection_model.clone();
        list_model.unselect_all();

        /*
         * exit of the search was a partial text search
         * and not a book reference
         */
        if evaluated.is_none() {
            list_model.select_item(0, true);
            return;
        }

        let evaluated = evaluated.unwrap();
        for index in evaluated.verses.clone() {
            list_model.select_item(index.saturating_sub(1), false);
        }

        let list_view = list_view_wrapper.borrow().view.clone();
        let list = match list_view.first_child() {
            Some(li) => util::widget_to_vec(&li),
            None => return (),
        };

        if let Some(vli) = evaluated.verses.first() {
            // subtract here since list.get uses zero based index
            match list.get(vli.saturating_sub(1) as usize) {
                Some(li) => li.grab_focus(),
                None => return (),
            };
        }
    }

    fn register_search_change(&mut self, db: Rc<RefCell<DatabaseConnection>>) {
        let list_model = self.list_view_wrapper.borrow().selection_model.clone();
        let list_view = self.list_view_wrapper.borrow().view.clone();
        let search_field = self.search_text.clone();
        let list_view_wrapper = self.list_view_wrapper.clone();
        let bible_translation = self.translation.clone();

        search_field.connect_search_changed(clone!(
            #[strong]
            bible_translation,
            #[strong]
            db,
            #[strong]
            list_view_wrapper,
            move |se| {
                println!("TYPING TRANSLATION {:?}", bible_translation);
                SearchScriptureModel::search_bible(
                    se.text().to_string(),
                    &bible_translation.borrow(),
                    db.clone(),
                    list_view_wrapper.clone(),
                );
                se.grab_focus();
            }
        ));

        search_field.connect_activate(clone!(
            #[strong]
            list_model,
            #[strong]
            list_view,
            move |_| {
                list_view.emit_by_name_with_values(
                    "activate",
                    &[list_model.selection().nth(0).to_value()],
                );
            }
        ));
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
                    let payload = SearchScriptureModel::get_payload_for_selected_scriptures(
                        &list_view_wrapper.borrow(),
                    );

                    let payload =
                        dto::ListPayload::new(text_entry.text().to_string(), 0, payload, None);

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

        typed_list
            .borrow()
            .selection_model
            .connect_selection_changed(clone!(
                #[strong]
                typed_list,
                #[strong]
                sender,
                #[strong]
                text_entry,
                move |_m, _pos, _n_items| {
                    let payload = SearchScriptureModel::get_payload_for_selected_scriptures(
                        &typed_list.borrow(),
                    );

                    let payload =
                        dto::ListPayload::new(text_entry.text().to_string(), 0, payload, None);
                    let _ = sender.output(SearchScriptureOutput::SendScriptures(payload));
                }
            ));
    }

    fn get_initial_scriptures(
        translation: String,
        db: &DatabaseConnection,
    ) -> Result<Vec<BibleVerse>, rusqlite::Error> {
        return Query::search_by_chapter_query(
            &db.connection,
            translation,
            String::from("Genesis"),
            1,
        );
    }

    fn get_payload_for_selected_scriptures(
        typed_list: &TypedListView<ScriptureListItem, MultiSelection>,
    ) -> Vec<String> {
        let selected_items = Vec::new();
        let model = match typed_list.view.model() {
            Some(model) => model,
            None => return selected_items,
        };

        let model = match model.downcast::<gtk::MultiSelection>() {
            Ok(model) => model,
            Err(err) => {
                println!("error getting model.\n{:?}", err);
                return selected_items;
            }
        };

        let mut selected_items = selected_items;
        let selections = model.selection();
        for i in 0..selections.size() {
            let item_index = selections.nth(i as u32);

            let verse_text = match typed_list.get(item_index) {
                Some(item) => item.borrow().clone().data.screen_display(),
                None => continue,
            };

            selected_items.push(verse_text);
        }

        return selected_items;
    }

    fn load_initial_verses(&mut self, translation: String, db_connection: &DatabaseConnection) {
        let list_view_wrapper = self.list_view_wrapper.clone();

        let verses = match SearchScriptureModel::get_initial_scriptures(
            translation.clone(),
            db_connection,
        ) {
            Ok(r) => r,
            Err(_) => Vec::new(),
        };

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
                    set_hexpand: true
                }
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

        let mut model = SearchScriptureModel {
            list_view_wrapper: list_view_wrapper.clone(),
            search_text: gtk::SearchEntry::new(),
            translation: Rc::new(RefCell::new(String::new())),
            db_connection: init.db_connection.clone(),
            dropdown: dropdown.clone(),
        };

        let list_view = model.list_view_wrapper.borrow().view.clone();
        let search_text = model.search_text.clone();
        let widgets = view_output!();

        let db = init.db_connection.clone();

        model.register_activate_selected(&sender);
        model.register_context_menu(&sender);

        let translations = SearchScriptureModel::get_bible_translations(init.db_connection.clone());
        if let Some(first) = translations.first() {
            model.load_initial_verses(first.to_string(), &init.db_connection.borrow());
        }

        model.register_search_change(db.clone());
        model.register_import_bible(
            &widgets.import_btn.clone(),
            translations.clone(),
            db,
            sender.clone(),
        );
        model.load_bible_translations(&widgets.dropdown, translations);
        model.register_translation_change(&widgets.dropdown, &sender);

        return relm4::ComponentParts { model, widgets };
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        println!("SCRP UPDATE");
        match message {
            SearchScriptureInput::ChangeTranslation(t) => {
                println!("SCRP UPDATE 2");
                *self.translation.borrow_mut() = t.clone();

                println!("SCRP UPDATE \n {:?}, {:?}", t, self.translation.clone());
                SearchScriptureModel::search_bible(
                    self.search_text.text().to_string(),
                    &self.translation.borrow(),
                    self.db_connection.clone(),
                    self.list_view_wrapper.clone(),
                );
            }
            SearchScriptureInput::ReloadTranlations => {
                let t = SearchScriptureModel::get_bible_translations(self.db_connection.clone());
                self.load_bible_translations(&self.dropdown.clone(), t);
            }

            SearchScriptureInput::NewTranslation(t) => {
                if self.translation.borrow().is_empty() {
                    *self.translation.borrow_mut() = t.clone();
                    self.load_initial_verses(t, &self.db_connection.clone().borrow());
                }
                sender.input(SearchScriptureInput::ReloadTranlations);
            }
        }
    }
}

mod download {

    use std::{cell::RefCell, rc::Rc};

    use gtk::{
        glib::clone,
        prelude::{ButtonExt, WidgetExt},
    };
    use relm4::{gtk, typed_view::list::RelmListItem, view, ComponentSender, RelmWidgetExt};

    use crate::{
        db::{connection::DatabaseConnection, query::Query},
        widgets::search::scriptures::import_bible,
    };

    use super::{BibleDownload, SearchScriptureInput, SearchScriptureModel};

    #[derive(Debug, Clone)]
    pub struct BibleDownloadListItem {
        pub data: BibleDownload,
        pub conn: Rc<RefCell<DatabaseConnection>>,
        pub already_added: bool,
        pub parent_sender: ComponentSender<SearchScriptureModel>,
    }

    pub struct BibleListItemWidget {
        text: gtk::Label,
        btn: gtk::Button,
    }

    impl Drop for BibleListItemWidget {
        fn drop(&mut self) {}
    }

    impl RelmListItem for BibleDownloadListItem {
        type Root = gtk::Box;
        type Widgets = BibleListItemWidget;

        fn setup(_list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
            view! {
                list_box = gtk::Box {
                    #[name="text"]
                    gtk::Label {
                        set_hexpand: true,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_align: gtk::Align::Start,
                        set_margin_horizontal: 8,
                    },
                    #[name="btn"]
                    gtk::Button{
                        // set_label:"Install",
                    },
                }
            }

            let widgets = BibleListItemWidget { text, btn };

            return (list_box, widgets);
        }

        fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
            let a = self.data.name.split(".").collect::<Vec<&str>>();
            if let Some(name) = a.get(0) {
                let name = name.to_string();
                match self.already_added {
                    true => {
                        widgets.btn.set_label("Uninstall");
                        widgets.text.set_label(&format!("{} (Installed)", name));
                    }
                    false => {
                        widgets.btn.set_label("Install");
                        widgets.text.set_label(&format!("{}", name));
                    }
                };
            }

            let conn = self.conn.clone();
            let data = self.data.clone();
            let already_added = self.already_added.clone();
            let sender = self.parent_sender.clone();

            widgets.btn.connect_clicked(move |btn| {
                gtk::glib::spawn_future_local(clone!(
                    #[strong]
                    btn,
                    #[strong]
                    data,
                    #[strong]
                    conn,
                    #[strong]
                    already_added,
                    #[strong]
                    sender,
                    async move {
                        btn.set_sensitive(false);

                        match already_added {
                            true => {
                                let a = data.name.split(".").collect::<Vec<&str>>();
                                if let Some(name) = a.get(0) {
                                    let name = name.to_string();
                                    let name = format!("{}", name);
                                    let mut conn = conn.borrow_mut();
                                    let delete_result =
                                        Query::delete_bible_translation(&mut conn.connection, name);

                                    match delete_result {
                                        Ok(_) => btn.set_label("Install"),
                                        Err(e) => {
                                            btn.set_label("Uninstall");
                                            eprintln!(
                                                "SQL ERROR: error removing translation\n{:?}",
                                                e
                                            );
                                        }
                                    }
                                }
                            }
                            false => {
                                btn.set_label("Installing");

                                let installed_translation = import_bible(&data, conn.clone()).await;
                                if let Some(installed_t) = installed_translation {
                                    sender.input(SearchScriptureInput::NewTranslation(installed_t));
                                }

                                btn.set_label("Installed");
                            }
                        }

                        btn.set_sensitive(true);
                    }
                ));
            });
        }
    }
}

async fn import_bible(
    bible: &BibleDownload,
    conn: Rc<RefCell<DatabaseConnection>>,
) -> Option<String> {
    println!("SELCETIONS {:?}", bible);

    let download_url = match &bible.download_url {
        Some(d) => d,
        None => return None,
    };

    let resp = reqwest::get(download_url)
        .await
        .expect("Request error")
        .bytes()
        .await
        .expect("Request error: Error getting response as bytes");

    let path_str = AppConfigDir::dir_path(AppConfigDir::DOWNLOADS);
    let file_path = path_str.join(bible.name.clone());
    let file = std::fs::File::create_new(&file_path);

    // if let Ok(mut file) = file {}

    let mut file = match file {
        Ok(f) => f,
        Err(e) => {
            eprintln!("failed to create file {:?}", e);
            return None;
        }
    };

    match file.write_all(&resp) {
        Ok(a) => a,
        Err(e) => {
            println!("Error saving file: {:?}", e);
            return None;
        }
    };
    println!("CREATED FILE {:?}", file_path);

    let db_conn = match Connection::open(&file_path) {
        Ok(conn) => conn,
        Err(e) => {
            println!("Error opening file: {:?}", e);
            return None;
        }
    };

    let translation: BibleTranslation = match db_conn.query_row(
        "SELECT translation, title, license FROM translations",
        [],
        |r| {
            let bt = BibleTranslation {
                translation: r.get::<_, String>(0)?,
                title: r.get::<_, String>(1)?,
                license: r.get::<_, String>(2)?,
            };

            Ok(bt)
        },
    ) {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "SQL ERROR: error getting downloaded translation info \n{:?}",
                e
            );
            return None;
        }
    };

    let translation_name = match bible.name.split(".").collect::<Vec<&str>>().get(0) {
        Some(name) => name.to_string(),
        None => return None,
    };

    let mut verses_sql = match db_conn.prepare(&format!(
        "SELECT id, book_id, chapter, verse, text FROM {}_verses",
        translation_name
    )) {
        Ok(s) => s,
        Err(e) => {
            println!("SQL ERROR: error getting downloaded verses \n{:?}", e);
            return None;
        }
    };

    let bible_verse = match verses_sql.query_map([], |r| {
        let bv = (
            r.get::<_, u32>(0)?, // id
            BibleVerse {
                book: "".to_string(),
                book_id: r.get::<_, u32>(1)?, // book_id
                chapter: r.get::<_, u32>(2)?, // chapter
                verse: r.get::<_, u32>(3)?,   // verse
                text: r.get::<_, String>(4)?, // text
            },
        );

        return Ok(bv);
    }) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("SQL ERROR: error getting downloaded verses \n{:?}", e);
            return None;
        }
    };

    let mut verses_vec = Vec::new();
    for row in bible_verse {
        match row {
            Ok(r) => {
                if r.1.book_id > 66 {
                    eprintln!("SQL ERROR: Book too large \n{:?}", verses_vec.get(0));
                    return None;
                }

                verses_vec.push(r);
            }
            Err(e) => {
                eprintln!("SQL ERROR: error extracting downloaded verses \n{:?}", e);
                return None;
            }
        };
    }

    let translation_name = translation.translation.clone();
    let res = Query::insert_verse(&mut conn.borrow_mut().connection, translation, verses_vec);
    println!("INSERTING VERESES DONE: {:?}", res);

    match std::fs::remove_file(&file_path) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("FILE ERROR: error removing downloaded verses \n{:?}", e);
            return None;
        }
    };

    return Some(translation_name);
    // list.selection_model
    //     .select_item(list.selection_model.selection().nth(0), true);
    //
    // btn.set_sensitive(false);
    // // win_clone.close();
}
