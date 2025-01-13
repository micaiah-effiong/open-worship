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

use crate::dto;

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
}

impl SearchScriptureModel {}

pub struct SearchScriptureInit {}

impl SearchScriptureModel {
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
        popover_menu.set_position(gtk::PositionType::Right);
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
                    println!("MS selections {:?}", &payload);
                    let _ = sender.output(SearchScriptureOutput::SendScriptures(payload));
                }
            }
        ));
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
                    selected_items.push(a.screen_display());
                    verse_vec.push(a.verse);
                    book = a.book;
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

            // #[name="search_field"]
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 2,
                set_height_request: 48,
                add_css_class: "green_double_box",

                gtk::SearchEntry {
                    set_placeholder_text: Some("Genesis 1:1"),
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
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let mut typed_list_view: TypedListView<ScriptureListItem, MultiSelection> =
            TypedListView::new();

        for i in 0..=150 {
            typed_list_view.append(ScriptureListItem {
                book: "Genesis".to_string(),
                chapter: 1,
                verse: i,
                text: LIST_VEC[0].to_string(),
            })
        }

        let list_view_wrapper = Rc::new(RefCell::new(typed_list_view));

        let mut model = SearchScriptureModel { list_view_wrapper };
        model.register_selected(&sender);
        model.register_context_menu(&sender);
        let list_view = model.list_view_wrapper.borrow().view.clone();

        let widgets = view_output!();

        return relm4::ComponentParts { model, widgets };
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {};
    }
}

const LIST_VEC: [&str; 1] = [
    "Golden sun, a radiant masterpiece, paints the canvas of the morning sky. With hues of pink and softest blue, a breathtaking, ethereal sight,",
];
