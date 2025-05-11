use std::{cell::RefCell, rc::Rc, usize};

use crate::{
    dto::{self},
    structs::activity_list_item::ActivityListItem,
};
use gtk::{
    glib::{clone, property::PropertySet},
    prelude::*,
};
use relm4::{prelude::*, typed_view::list::TypedListView};

const MIN_GRID_HEIGHT: i32 = 300;
// const MIN_GRID_WIDTH: i32 = 300;

#[derive(Debug)]
pub enum PreviewViewerInput {
    // Selected(u32),
    // Activated(u32),
    NewList(dto::ListPayload),
    Background(String),
    GoLive,
}
#[derive(Debug)]
pub enum PreviewViewerOutput {
    Selected(dto::Payload),
    Activated(dto::ListPayload),
}
pub struct PreviewViewerInit {}

#[derive(Clone)]
pub struct PreviewViewerModel {
    list_view_wrapper: Rc<RefCell<TypedListView<ActivityListItem, gtk::SingleSelection>>>,
    slide: Rc<RefCell<Option<dto::ListPayload>>>,
}

impl PreviewViewerModel {
    fn new() -> Self {
        PreviewViewerModel {
            slide: Rc::new(RefCell::new(None)),
            list_view_wrapper: Rc::new(RefCell::new(TypedListView::new())),
        }
    }

    fn register_activate(&self, sender: &ComponentSender<Self>) {
        let list_view = self.list_view_wrapper.borrow().view.clone();
        let slide = self.slide.clone();

        list_view.connect_activate(clone!(
            #[strong]
            slide,
            #[strong]
            sender,
            move |_list_view, pos| {
                let slide = match slide.borrow().clone() {
                    Some(s) => s,
                    None => return,
                };

                let txt = match slide.list.get(pos as usize) {
                    Some(txt) => txt,
                    None => return, // &String::from(""),
                };

                let payload = dto::ListPayload {
                    text: txt.to_string(),
                    list: slide.list.clone(),
                    position: pos,
                    background_image: slide.background_image,
                };

                let _ = sender.output(PreviewViewerOutput::Activated(payload));
            }
        ));
    }

    fn register_selection_change(&self, sender: &ComponentSender<Self>) {
        let model = self.list_view_wrapper.borrow().selection_model.clone();
        // let list = self.list.borrow();
        // let wrapper = self.list_view_wrapper.clone();
        let slide = self.slide.clone();
        model.connect_selection_changed(clone!(
            #[strong]
            sender,
            #[strong]
            slide,
            move |selection_model, _, _| {
                let pos = selection_model.selected();

                let slide = match slide.borrow().clone() {
                    Some(s) => s,
                    None => return,
                };

                let txt = match slide.list.get(pos as usize) {
                    Some(txt) => txt,
                    None => return, //&String::from("Nothing"),
                };

                let payload = dto::Payload {
                    text: txt.to_string(),
                    position: pos,
                    background_image: slide.background_image,
                };

                let _ = sender.output(PreviewViewerOutput::Selected(payload));
            }
        ));
    }
}

#[relm4::component(pub)]
impl SimpleComponent for PreviewViewerModel {
    type Input = PreviewViewerInput;
    type Output = PreviewViewerOutput;
    type Init = PreviewViewerInit;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_height_request: MIN_GRID_HEIGHT,
            set_css_classes: &["pink_box", "ow-listview"],

            gtk::Label {
                set_label: &title
            },

            gtk::ScrolledWindow {
                set_vexpand: true,

                #[wrap(Some)]
                #[local_ref]
                set_child= &list_view -> gtk::ListView{},
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = PreviewViewerModel::new();
        let list_view = model.list_view_wrapper.borrow().view.clone();

        let title = match model.slide.borrow().clone() {
            Some(m) => m.text,
            None => String::new(),
        };

        let widgets = view_output!();
        model.register_selection_change(&sender);
        model.register_activate(&sender);

        relm4::ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            PreviewViewerInput::NewList(payload) => {
                self.slide.set(Some(payload.clone()));
                self.list_view_wrapper.borrow_mut().clear();

                for item in payload.list {
                    self.list_view_wrapper
                        .borrow_mut()
                        .append(ActivityListItem { text: item });
                }
                // self.list_view_wrapper.borrow().view.grab_focus();
            }
            PreviewViewerInput::Background(img) => {
                if let Some(slide) = self.slide.borrow_mut().as_mut() {
                    slide.background_image = Some(img.clone());
                }
                // self.list_view_wrapper.borrow().view.grab_focus();
            }
            PreviewViewerInput::GoLive => {
                let slide = match self.slide.borrow().clone() {
                    Some(s) => s,
                    None => return,
                };

                let model = self.list_view_wrapper.borrow().selection_model.clone();
                let index = model.selected();

                let text = match slide.list.get(index as usize) {
                    Some(item) => item.to_string(),
                    None => return,
                };

                let payload = dto::ListPayload {
                    text,
                    list: slide.list,
                    position: index,
                    background_image: slide.background_image,
                };

                let _ = sender.output(PreviewViewerOutput::Activated(payload));
            }
        };
    }
}
