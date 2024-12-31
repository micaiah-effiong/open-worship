use std::{cell::RefCell, rc::Rc, usize};

use gtk::{
    glib::{clone, property::PropertySet},
    prelude::*,
};
use relm4::{prelude::*, typed_view::list::TypedListView};

use crate::{dto, structs::activity_list_item::ActivityListItem};

const MIN_GRID_HEIGHT: i32 = 300;
// const MIN_GRID_WIDTH: i32 = 300;

#[derive(Debug)]
pub enum LiveViewerInput {
    // Selected(u32),
    // Activated(u32),
    NewList(dto::ListPayload),
}
#[derive(Debug)]
pub enum LiveViewerOutput {
    Selected(dto::Payload),
    // Activated(String),
}
pub struct LiveViewerInit {
    pub title: String,
    pub list: Vec<String>,
    pub selected_index: Option<u32>,
}

#[derive(Clone)]
pub struct LiveViewerModel {
    title: String,
    list: Rc<RefCell<Vec<String>>>,
    selected_index: Rc<RefCell<Option<u32>>>,
    background_image: Rc<RefCell<Option<String>>>,
    list_view_wrapper: Rc<RefCell<TypedListView<ActivityListItem, gtk::SingleSelection>>>,
}

impl LiveViewerModel {
    fn new() -> Self {
        let list_view_wrapper = Rc::new(RefCell::new(TypedListView::new()));

        return LiveViewerModel {
            title: String::from(""),
            list: Rc::new(RefCell::new(Vec::new())),
            selected_index: Rc::new(RefCell::new(None)),
            background_image: Rc::new(RefCell::new(None)),
            list_view_wrapper,
        };
    }
}

impl LiveViewerModel {
    fn listen_for_selection_changed(&self, sender: &ComponentSender<LiveViewerModel>) {
        let model = self.clone();
        let selection_model = model.list_view_wrapper.borrow().selection_model.clone();
        let list = model.list.clone();
        let background_image = model.background_image.borrow().clone();
        let selected_index = model.selected_index.borrow().clone();

        selection_model.connect_selection_changed(clone!(
            #[strong]
            list,
            #[strong]
            sender,
            #[strong]
            background_image,
            #[strong]
            selected_index,
            move |selection_model, _, _| {
                let pos = selection_model.selected();
                let txt = match list.borrow().get(pos as usize) {
                    Some(txt) => txt.clone(),
                    None => return,
                };

                println!("selection changed {:?}", selected_index);

                let payload = dto::Payload {
                    text: txt,
                    position: pos,
                    background_image: background_image.clone(),
                };

                let _ = sender.output(LiveViewerOutput::Selected(payload));
            }
        ));
    }

    fn listen_for_items_change(&self) {
        let model = self.clone();
        let selection_model = model.list_view_wrapper.borrow().selection_model.clone();
        let selected_index = model.selected_index;
        let list_view = model.list_view_wrapper.borrow().view.clone();

        selection_model.connect_items_changed(clone!(
            #[strong]
            list_view,
            #[strong]
            selected_index,
            move |selection_model, _, _, _| {
                println!("here {:?}", selected_index);

                let index = match selected_index.borrow().clone() {
                    Some(inx) => inx,
                    None => return,
                };

                selection_model.select_item(index.clone(), true);
                list_view.grab_focus();

                let mut li = list_view.first_child();
                println!("first child 0 => {:?}", &li);

                let mut i = 0;
                loop {
                    if i == index || li.is_none() {
                        break;
                    }

                    if let Some(list_item) = li {
                        // println!("loop {i} => {:?}", &list_item);
                        li = list_item.next_sibling();
                    }

                    i += 1;
                }

                if let Some(list_item) = li {
                    list_item.grab_focus();
                }
            }
        ));
    }
}

#[relm4::component(pub)]
impl SimpleComponent for LiveViewerModel {
    type Input = LiveViewerInput;
    type Output = LiveViewerOutput;
    type Init = LiveViewerInit;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_height_request: MIN_GRID_HEIGHT,
            set_css_classes: &["pink_box", "ow-listview"],

            gtk::Label {
                set_label: &model.title
            },

            gtk::ScrolledWindow {
                set_vexpand: true,

                #[wrap(Some)]
                #[local_ref]
                set_child = &list_view -> gtk::ListView {}
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = LiveViewerModel::new();
        let list_view = model.list_view_wrapper.borrow().view.clone();
        let widgets = view_output!();

        model.listen_for_items_change();
        model.listen_for_selection_changed(&sender);

        return relm4::ComponentParts { model, widgets };
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            LiveViewerInput::NewList(list_payload) => {
                println!("new here {:?}", &list_payload.position);
                self.selected_index.set(Some(list_payload.position));
                println!("new here update {:?}", &self.selected_index.borrow());

                self.list.borrow_mut().clear();
                self.list
                    .borrow_mut()
                    .append(&mut list_payload.list.clone());

                self.list_view_wrapper.borrow_mut().clear();

                let mut list = Vec::new();
                for item in list_payload.list {
                    list.push(ActivityListItem { text: item });
                }
                self.list_view_wrapper.borrow_mut().extend_from_iter(list);
            }
        };
    }
}
