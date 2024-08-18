use std::{cell::RefCell, rc::Rc, usize};

use crate::dto;
use gtk::prelude::*;
use relm4::prelude::*;

const MIN_GRID_HEIGHT: i32 = 300;
// const MIN_GRID_WIDTH: i32 = 300;

#[derive(Debug)]
pub enum PreviewViewerInput {
    // Selected(u32),
    // Activated(u32),
    NewList(dto::ListPayload),
}
#[derive(Debug)]
pub enum PreviewViewerOutput {
    Selected(dto::Payload),
    Activated(dto::ListPayload),
}
pub struct PreviewViewerData {
    pub title: String,
    pub list: Vec<String>,
    pub selected_index: Option<u32>,
}

#[derive(Clone)]
pub struct PreviewViewerModel {
    title: String,
    list: Rc<RefCell<Vec<String>>>,

    /// Because selected_index is used to updated selected list-item
    /// it must be updated for every input that changes the selected item
    selected_index: u32,
    // list_view: gtk::ListView,
}

#[relm4::component(pub)]
impl SimpleComponent for PreviewViewerModel {
    type Input = PreviewViewerInput;
    type Output = PreviewViewerOutput;
    type Init = PreviewViewerData;

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
                // set_child: Some(&model.list_view)
                #[wrap(Some)]
                set_child= &gtk::ListView{
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
                        println!("activate-preview {:?}", &pos);

                        let list = model.list.borrow();

                        let txt = match list.get(pos as usize) {
                            Some(txt) => txt,
                            None => &String::from(""),
                        };

                        let payload = dto::ListPayload {
                            text: txt.to_string(),
                            list: list.clone(),
                            position: pos
                        };
                        let _ = sender.output(PreviewViewerOutput::Activated(payload));
                    },

                    #[wrap(Some)]
                    #[name="single_selection_model"]
                    set_model = &gtk::SingleSelection {

                        #[watch]
                        set_model:Some( &model.list.borrow().clone().into_iter().collect::<gtk::StringList>()),

                        // #[watch]
                        // set_selected: model.selected_index,

                        connect_selection_changed[sender, list=model.clone().list ] => move |selection_model,_,_|{
                            let single_selection_model =
                                match selection_model.downcast_ref::<gtk::SingleSelection>() {
                                    Some(ss) => ss,
                                    None => return,
                                };

                            let pos = single_selection_model.selected();
                            println!("selec {:?}", &pos);

                            let list = list.borrow().clone();
                            let txt = match list.get(pos as usize) {
                                Some(txt) => txt,
                                None => &String::from(""),
                            };

                            let payload = dto::Payload {
                                text: txt.to_string(),
                                position: pos
                            };

                            let _ = sender.output(PreviewViewerOutput::Selected(payload ));
                        }
                    },

                    #[wrap(Some)]
                    set_factory= &gtk::SignalListItemFactory{
                        connect_setup => move |_, list_item|{
                            let label = gtk::Label::builder()
                                .ellipsize(gtk::pango::EllipsizeMode::End)
                                .wrap_mode(gtk::pango::WrapMode::Word)
                                .lines(2)
                                .margin_top(12)
                                .margin_bottom(12)
                                .halign(gtk::Align::Start)
                                .justify(gtk::Justification::Fill)
                                .build();

                            list_item
                                .downcast_ref::<gtk::ListItem>()
                                .expect("Must be a list item")
                                .set_child(Some(&label));

                            list_item
                                .property_expression("item")
                                .chain_property::<gtk::StringObject>("string")
                                .bind(&label, "label", gtk::Widget::NONE);
                        }
                    }
                },
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let selected_index = match init.selected_index {
            Some(index) => index,
            None => 0,
        };

        let model = PreviewViewerModel {
            title: init.title,
            list: Rc::new(RefCell::new(init.list)),
            selected_index, // list_view: list_view.clone(),
        };

        let widgets = view_output!();

        return relm4::ComponentParts { model, widgets };
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        let _ = match message {
            PreviewViewerInput::NewList(payload) => {
                self.list.borrow_mut().clear();
                self.list.borrow_mut().append(&mut payload.list.clone());
                self.selected_index = payload.position;

                println!(
                    "preview new sli pos={}, si={}",
                    payload.position, self.selected_index
                );
            }
        };

        return ();
    }
}
