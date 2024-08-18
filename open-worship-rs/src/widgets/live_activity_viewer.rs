use std::{cell::RefCell, rc::Rc};

use gtk::{
    glib::{clone, property::PropertySet},
    prelude::*,
};
use relm4::prelude::*;

use crate::dto;

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
    Activated(String),
}
pub struct LiveViewerData {
    pub title: String,
    pub list: Vec<String>,
    pub selected_index: Option<u32>,
}

#[derive(Clone)]
pub struct LiveViewerModel {
    title: String,
    list: Rc<RefCell<Vec<String>>>,
    selected_index: Rc<RefCell<Option<u32>>>,
    list_view: gtk::ListView,
}

impl LiveViewerModel {
    fn listen_for_items_change(&self) {
        let selection_model = match self.list_view.model() {
            Some(sm) => sm,
            None => return,
        };

        let selected_index = self.clone().selected_index;
        let list_view = self.clone().list_view;

        selection_model.clone().connect_items_changed(clone!(
            @strong
            list_view,
            @strong
            selected_index,
            => move |_, _, _, _| {
                let index = match selected_index.borrow().clone() {
                    Some(inx) => inx,
                    None => return,
                };

                selection_model.select_item(index.clone(), true);
                list_view.grab_focus();

                let mut li = list_view.first_child();
                println!("first child 0 => {:?}", &li.clone().unwrap());

                let mut i = 0;
                loop {

                    if i == index || li.is_none(){
                        break;
                    }

                    if let Some(list_item) = li{
                        // println!("loop {i} => {:?}", &list_item);
                        li = list_item.next_sibling();
                    }

                    i += 1;
                }

                if let Some(list_item) = li{
                    println!("loop {i} => {:?}", &list_item);
                    list_item.grab_focus();
                    // list_view.set_focus_child(Some(&list_item));
                }

            }

        ));
    }
}

#[relm4::component(pub)]
impl SimpleComponent for LiveViewerModel {
    type Input = LiveViewerInput;
    type Output = LiveViewerOutput;
    type Init = LiveViewerData;

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

                // #[wrap(Some)]
                // set_child = &gtk::Viewport {
                // set_scroll_to_focus: true,

                #[wrap(Some)]
                #[local_ref]
                set_child = &list_view -> gtk::ListView {
                    // connect_activate[sender] => move |list_view,_|{
                        // let selection_model = match list_view.model() {
                        //     Some(m)=>m,
                        //     None=>return,
                        // };
                        //
                        // let single_selection_model =
                        //     match selection_model.downcast_ref::<gtk::SingleSelection>() {
                        //         Some(ss) => ss,
                        //         None => return,
                        //     };
                        //
                        // let pos = single_selection_model.selected();
                        // println!("live activate {:?}", &pos);
                        //
                        // sender.input(LiveViewerInput::Activated(pos));
                    // },

                    #[wrap(Some)]
                    #[name="single_selection_model"]
                    set_model = &gtk::SingleSelection {
                        #[watch]
                        set_model:Some( &model.list.borrow().clone().into_iter().collect::<gtk::StringList>()),

                        // #[watch]
                        // set_selected: model.selected_index,

                        connect_selection_changed[sender, model=model.clone()] => move |selection_model,_,_|{
                            let pos = selection_model.selected();
                            let list  = &model.list.borrow();
                            let txt = match list.get(pos as usize) {
                                Some(txt) => txt,
                                None => &String::from(""),
                            };
                            // println!("live selec no={:?} text={:?}", &pos, &list);
                            println!("Damn selected");

                            selection_model.selected_item();

                            let payload = dto::Payload{
                                text: txt.to_string(),
                                position: pos,
                            };

                            let _ = sender.output(LiveViewerOutput::Selected(payload));
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
                }
                // },
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let list_view = gtk::ListView::builder().build();

        let model = LiveViewerModel {
            title: init.title,
            list: Rc::new(RefCell::new(init.list)),
            list_view: list_view.clone(),
            selected_index: Rc::new(RefCell::new(None)),
        };

        let widgets = view_output!();

        model.listen_for_items_change();

        return relm4::ComponentParts { model, widgets };
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            LiveViewerInput::NewList(list_payload) => {
                self.selected_index.set(Some(list_payload.position));
                self.list.borrow_mut().clear();
                self.list
                    .borrow_mut()
                    .append(&mut list_payload.list.clone());
            }
        };
    }
}
