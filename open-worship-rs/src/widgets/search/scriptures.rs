mod list_item;

use std::{cell::RefCell, rc::Rc};

use gtk::{glib::clone, prelude::*, MultiSelection};
use list_item::ScriptureListItem;
use relm4::{prelude::*, typed_view::list::TypedListView};

use crate::dto;

#[derive(Debug)]
pub enum SearchScriptureInput {}

#[derive(Debug)]
pub enum SearchScriptureOutput {
    SendScriptures(dto::ListPayload),
}

#[derive(Debug, Clone)]
pub struct SearchScriptureModel {
    list_view_wrapper: Rc<RefCell<TypedListView<ScriptureListItem, MultiSelection>>>,
}

impl SearchScriptureModel {}

pub struct SearchScriptureInit {}

impl SearchScriptureModel {
    fn register_selected(&mut self, sender: ComponentSender<Self>) {
        let list_view = self.list_view_wrapper.borrow().view.clone();
        let typed_list = self.list_view_wrapper.clone();

        list_view.connect_activate(clone!(
            @strong typed_list,
            =>move |lv, _| {
                //
                let model = lv
                    .model()
                    .unwrap() //
                    .downcast::<gtk::MultiSelection>()
                    .unwrap(); //
                let typed_list = typed_list.borrow();

                let mut selected_items = Vec::new();
                for i in 0..model.n_items() {
                    if model.is_selected(i) {

                        if let Some(item) = typed_list.get(i) {
                            let a = item.borrow().clone();
                            selected_items.push(a.screen_display());
                        }

                    }
                }

                // list payload
                let list_payload  = dto::ListPayload::new("title".to_string(), 0, selected_items.clone(), None);


                println!("MS selections {:?}", &list_payload);
                let _ = sender.output(SearchScriptureOutput::SendScriptures(list_payload));

            }
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
        model.register_selected(sender.clone());
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
