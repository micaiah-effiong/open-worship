mod list_item;

use std::{cell::RefCell, rc::Rc};

use gtk::{
    gdk,
    gio::{ActionEntry, MenuItem, SimpleActionGroup},
    glib::clone,
    prelude::*,
    MultiSelection,
};
use list_item::ScriptureListItem;
use relm4::{prelude::*, typed_view::list::TypedListView};

use crate::{
    db::{
        connection::{BibleVerse, DatabaseConnection},
        query::Query,
    },
    dto,
    parser::parser,
    widgets::util,
};

#[derive(Debug)]
pub enum SearchScriptureInput {}

#[derive(Debug)]
pub enum SearchScriptureOutput {
    SendScriptures(dto::ListPayload),
    SendToSchedule(dto::ListPayload),
}

#[derive(Debug, Clone)]
pub struct SearchScriptureModel {
    list_view_wrapper: Rc<RefCell<TypedListView<ScriptureListItem, MultiSelection>>>,
    search_text: gtk::SearchEntry,
}

impl SearchScriptureModel {}

pub struct SearchScriptureInit {
    pub db_connection: Rc<RefCell<DatabaseConnection>>,
}

impl SearchScriptureModel {
    fn register_search_change(&mut self, db: Rc<RefCell<DatabaseConnection>>) {
        let list_model = self.list_view_wrapper.borrow().selection_model.clone();
        let list_view = self.list_view_wrapper.borrow().view.clone();
        let search_field = self.search_text.clone();
        let list_view_wrapper = self.list_view_wrapper.clone();

        search_field.connect_search_changed(clone!(
            #[strong]
            db,
            #[strong]
            list_view,
            #[strong]
            list_model,
            #[strong]
            list_view_wrapper,
            move |se| {
                let p = parser::Parser::parser(String::from(se.text()));
                let p = match p {
                    Some(p) => p,
                    None => return,
                };

                println!("CONNECT_SEARCH_CHANGED {:?}", p.eval());
                let evaluated = p.eval();

                let verses = match Query::get_chapter_query(
                    &db.borrow().connection,
                    String::from("KJV"),
                    evaluated.book,
                    evaluated.chapter,
                ) {
                    Ok(vs) => vs,
                    Err(x) => {
                        println!("SQL ERROR: \n{:?}", x);
                        return;
                    }
                };

                list_view_wrapper.borrow_mut().clear();
                for verse in verses {
                    list_view_wrapper.borrow_mut().append(ScriptureListItem {
                        data: dto::Scripture {
                            book: verse.book.clone(),
                            chapter: verse.chapter,
                            verse: verse.verse,
                            text: verse.text.clone(),
                        },
                    });
                }

                let pos = evaluated.verses.get(0).unwrap_or(&0).clone();
                list_model.select_item(pos.saturating_sub(1), true);

                for index in evaluated.verses.clone() {
                    list_model.select_item(index.saturating_sub(1), false);
                }

                let list = match list_view.first_child() {
                    Some(li) => util::widget_to_vec(&li),
                    None => return (),
                };

                for (i, li) in list.iter().enumerate() {
                    let vli = match evaluated.verses.first() {
                        Some(vli) => vli,
                        None => continue,
                    };

                    if vli.saturating_sub(1).eq(&(i as u32)) {
                        li.grab_focus();
                        break;
                    }
                }

                se.grab_focus();
            }
        ));

        search_field.connect_activate(clone!(
            #[strong]
            list_model,
            #[strong]
            list_view,
            move |se| {
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

        // action entries
        let add_to_schedule_action = ActionEntry::builder("add-to-schedule")
            .activate(clone!(
                #[strong]
                list_view_wrapper,
                #[strong]
                list_view,
                #[strong]
                sender,
                move |_, _, _| {
                    let payload = SearchScriptureModel::get_payload_for_selected_scriptures(
                        &list_view,
                        &list_view_wrapper.borrow(),
                    );

                    if let Some(payload) = payload {
                        let _ = sender.output(SearchScriptureOutput::SendToSchedule(payload));
                    }
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
        gesture.set_button(gdk::ffi::GDK_BUTTON_SECONDARY as u32);
        gesture.connect_pressed(clone!(move |_, _, x, y| {
            let rect = gtk::gdk::Rectangle::new(x as i32, y as i32, 0, 0);
            popover_menu.set_pointing_to(Some(&rect));
            popover_menu.popup();
        }));

        list_view.add_controller(gesture);
        list_view.insert_action_group("scripture", Some(&action_group));
    }

    fn register_selected(&mut self, sender: &ComponentSender<Self>) {
        let list_view = self.list_view_wrapper.borrow().selection_model.clone();
        // let model = self.clone();

        // list_view.connect_selection_changed(clone!(
        //     #[strong]
        //     sender,
        //     move |selection_model, _, _| {
        //         let pos = selection_model.selection().nth(0);
        //         let end_val = selection_model
        //             .selection()
        //             .nth((selection_model.selection().size() - 1) as u32);
        //
        //         let end = if end_val > pos { Some(end_val) } else { None };
        //
        //         sender.input(SearchScriptureInput::Selected(pos, end));
        //     }
        // ));
    }

    fn register_activate_selected(&mut self, sender: &ComponentSender<Self>) {
        let list_view = self.list_view_wrapper.borrow().view.clone();
        let typed_list = self.list_view_wrapper.clone();

        list_view.connect_activate(clone!(
            #[strong]
            typed_list,
            #[strong]
            sender,
            move |lv, _| {
                let payload = SearchScriptureModel::get_payload_for_selected_scriptures(
                    lv,
                    &typed_list.borrow(),
                );

                if let Some(payload) = payload {
                    let _ = sender.output(SearchScriptureOutput::SendScriptures(payload));
                }
            }
        ));
    }

    fn get_initial_scriptures(db: &DatabaseConnection) -> Result<Vec<BibleVerse>, rusqlite::Error> {
        return Query::get_chapter_query(
            &db.connection,
            String::from("KJV"),
            String::from("Genesis"),
            1,
        );
    }

    fn get_payload_for_selected_scriptures(
        lv: &gtk::ListView,
        typed_list: &TypedListView<ScriptureListItem, MultiSelection>,
    ) -> Option<dto::ListPayload> {
        let model = match lv.model() {
            Some(model) => model,
            None => return None,
        };

        let model = match model.downcast::<gtk::MultiSelection>() {
            Ok(model) => model,
            Err(err) => {
                println!("error getting model.\n{:?}", err);
                return None;
            }
        };

        let mut selected_items = Vec::new();
        let mut verse_vec = Vec::new();
        let mut book = String::new();
        for i in 0..model.n_items() {
            if model.is_selected(i) {
                if let Some(item) = typed_list.get(i) {
                    let a = item.borrow().clone();
                    selected_items.push(a.data.screen_display());
                    verse_vec.push(a.data.verse);
                    book = a.data.book;
                }
            }
        }

        let title: String;
        if selected_items.len() > 1 {
            let [first, last] = [verse_vec.first(), verse_vec.last()];

            if first.is_none() || last.is_none() {
                return None;
            }

            title = format!("{} {}:{}-{}", book, 1, first.unwrap(), last.unwrap());
        } else {
            if let Some(verse) = verse_vec.get(0) {
                title = format!("{} {}:{}", book, 1, verse);
            } else {
                return None;
            }
        }

        // list payload
        return Some(dto::ListPayload::new(
            title,
            0,
            selected_items.clone(),
            None,
        ));
    }

    fn load_initial_verses(&mut self, db_connection: &DatabaseConnection) {
        let list_view_wrapper = self.list_view_wrapper.clone();

        let verses = match SearchScriptureModel::get_initial_scriptures(db_connection) {
            Ok(r) => r,
            Err(_) => Vec::new(),
        };

        list_view_wrapper.borrow_mut().clear();
        for (i, verse) in verses.iter().enumerate() {
            list_view_wrapper.borrow_mut().append(ScriptureListItem {
                data: dto::Scripture {
                    book: verse.book.clone(),
                    chapter: verse.chapter,
                    verse: verse.verse,
                    text: verse.text.clone(),
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

        let mut model = SearchScriptureModel {
            list_view_wrapper: list_view_wrapper.clone(),
            search_text: gtk::SearchEntry::new(),
        };

        let list_view = model.list_view_wrapper.borrow().view.clone();
        let search_text = model.search_text.clone();
        let widgets = view_output!();

        let db = init.db_connection.clone();

        model.register_selected(&sender);
        model.register_activate_selected(&sender);
        model.register_context_menu(&sender);
        model.load_initial_verses(&init.db_connection.borrow());
        model.register_search_change(db);

        return relm4::ComponentParts { model, widgets };
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {};
    }
}
