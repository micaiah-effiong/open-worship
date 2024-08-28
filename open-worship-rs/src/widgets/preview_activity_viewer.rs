mod list_item;
use std::{cell::RefCell, rc::Rc};

use crate::dto;
use gtk::{
    glib::{clone, property::PropertySet},
    prelude::*,
};
use list_item::ActivityListItem;
use relm4::{prelude::*, typed_view::list::TypedListView};

const MIN_GRID_HEIGHT: i32 = 300;
// const MIN_GRID_WIDTH: i32 = 300;

#[derive(Debug)]
pub enum PreviewViewerInput {
    // Selected(u32),
    // Activated(u32),
    NewList(dto::ListPayload),
    Background(String),
}
#[derive(Debug)]
pub enum PreviewViewerOutput {
    Selected(dto::Payload),
    Activated(dto::ListPayload),
}
pub struct PreviewViewerInit {}

#[derive(Clone)]
pub struct PreviewViewerModel {
    title: String,
    list: Rc<RefCell<Vec<String>>>,
    background_image: Rc<RefCell<Option<String>>>,
    list_view_wrapper: Rc<RefCell<TypedListView<ActivityListItem, gtk::SingleSelection>>>,
}

impl PreviewViewerModel {
    fn new() -> Self {
        return PreviewViewerModel {
            title: String::new(),
            list: Rc::new(RefCell::new(Vec::new())),
            background_image: Rc::new(RefCell::new(None)),
            list_view_wrapper: Rc::new(RefCell::new(TypedListView::new())),
        };
    }

    fn register_selection_change(&self, sender: &ComponentSender<Self>) {
        let model = self.list_view_wrapper.borrow().selection_model.clone();
        let list = self.list.borrow();
        let wrapper = self.list_view_wrapper.clone();
        let bg_image = self.background_image.clone();
        model.connect_selection_changed(clone!(
            @strong sender,
            @strong wrapper,
            @strong list,
            @strong bg_image,
            => move |selection_model,_,_|{

                let pos = selection_model.selected();
                println!("selec {:?}", &pos,);



                let txt = match wrapper.borrow().get(pos) {
                    Some(txt) => txt.borrow().text.clone(),
                    None => return//&String::from("Nothing"),
                };

                let payload = dto::Payload {
                    text: txt.to_string(),
                    position: pos,
                    background_image: bg_image.borrow().clone(),
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
                set_label: &model.title
            },

            gtk::ScrolledWindow {
                set_vexpand: true,

                #[wrap(Some)]
                #[local_ref]
                set_child= &list_view -> gtk::ListView{
                    connect_activate[sender, model] => move |list_view,_|{
                        let selection_model = match list_view.model() {
                            Some(m)=>m,
                            None=>return,
                        };

                        let ss_model = match selection_model.downcast_ref::<gtk::SingleSelection>(){
                            Some(ss)=>ss,
                            None => return,
                        };

                        let pos = ss_model.selected();
                        let wrapper = model.list_view_wrapper.borrow();
                        let txt = match wrapper.get(pos) {
                            Some(txt) => txt,
                            None => return// &String::from(""),
                        };

                        let item = txt.borrow().clone();


                        let payload = dto::ListPayload {
                            text: item.text.to_string(),
                            list: model.list.borrow().clone(),
                            position: pos,
                            background_image: model.background_image.borrow().clone(),
                        };

                        let _ = sender.output(PreviewViewerOutput::Activated(payload));
                    },

                },
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

        let widgets = view_output!();
        model.register_selection_change(&sender);

        return relm4::ComponentParts { model, widgets };
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            PreviewViewerInput::NewList(payload) => {
                self.list.borrow_mut().clear();
                self.list.borrow_mut().append(&mut payload.list.clone());
                self.list_view_wrapper.borrow_mut().clear();

                for item in payload.list {
                    self.list_view_wrapper
                        .borrow_mut()
                        .append(ActivityListItem { text: item });
                }
                self.list_view_wrapper.borrow().view.grab_focus();
            }
            PreviewViewerInput::Background(img) => {
                self.background_image.set(Some(img));
                self.list_view_wrapper.borrow().view.grab_focus();
            }
        };
    }
}
