use gtk::prelude::*;
use relm4::prelude::*;

const MIN_GRID_HEIGHT: i32 = 300;
// const MIN_GRID_WIDTH: i32 = 300;

#[derive(Debug)]
pub enum LiveViewerInput {
    Selected(u32),
    NewList(Vec<String>, u32),
}
#[derive(Debug)]
pub enum LiveViewerOutput {
    Selected(Vec<String>, u32),
}
pub struct LiveViewerData {
    pub title: String,
    pub list: Vec<String>,
    pub selected_index: Option<u32>,
}
pub struct LiveViewerModel {
    title: String,
    list: Vec<String>,
    selected_index: u32,
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
                #[wrap(Some)]
                set_child= &gtk::ListView{
                    #[wrap(Some)]
                    #[name="single_selection_model"]
                    set_model = &gtk::SingleSelection {
                        #[watch]
                        set_model:Some( &model.list.clone().into_iter().collect::<gtk::StringList>()),

                        #[watch]
                        set_selected: model.selected_index,

                        connect_selection_changed[sender] => move |selection_model,_,_|{
                            let single_selection_model =
                                match selection_model.downcast_ref::<gtk::SingleSelection>() {
                                    Some(ss) => ss,
                                    None => return,
                                };

                            let pos = single_selection_model.selected();
                            println!("live selec {:?}", &pos);

                            sender.input(LiveViewerInput::Selected(pos));
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

        let model = LiveViewerModel {
            title: init.title,
            list: init.list,
            selected_index, // list_view: list_view.clone(),
        };

        let widgets = view_output!();

        return relm4::ComponentParts { model, widgets };
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        let _ = match message {
            LiveViewerInput::Selected(position) => {
                self.selected_index = position;
                sender.output(LiveViewerOutput::Selected(self.list.clone(), position))
            }
            LiveViewerInput::NewList(list, pos) => {
                self.selected_index = pos;
                self.list = list.clone();

                println!("live new sli pos={}, si={}", pos, self.selected_index);
                Ok(())
            }
        };

        return ();
    }
}
