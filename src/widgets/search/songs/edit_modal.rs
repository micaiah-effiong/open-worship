use gtk::prelude::*;
use relm4::{prelude::*, SimpleComponent};

use crate::widgets::activity_screen::ActivityScreenModel;

#[derive(Debug)]
pub struct EditModel {
    pub is_active: bool,
    pub screen: Controller<ActivityScreenModel>,
}

#[derive(Debug)]
pub enum EditModelInputMsg {
    Show,
    Hide,

    #[doc(hidden)]
    Response(gtk::ResponseType),
}

#[derive(Debug)]
pub enum EditModelOutputMsg {
    Ok,
    Cancel,
}

const WIDTH: i32 = 1200;

#[relm4::component(pub)]
impl SimpleComponent for EditModel {
    type Init = ();
    type Output = EditModelOutputMsg;
    type Input = EditModelInputMsg;

    view! {
        #[name="window"]
        gtk::Window {
            set_title: Some("Add Song"),
            set_default_width:WIDTH,
            set_default_height:700,
            set_modal: true,
            set_focus_visible: true,
            set_resizable: false,

            #[watch]
            set_visible: model.is_active,

            // title
            // style section
            // editor | viewer
            gtk::Box{
                set_hexpand: true,
                set_vexpand: true,
                set_orientation: gtk::Orientation::Vertical,
                set_homogeneous: false,

                gtk::Box{
                    set_height_request: 80,
                    set_css_classes: &["brown_box"],
                },
                gtk::Box{
                    set_height_request: 60,
                    set_css_classes: &["brown_box"],
                },
                gtk::Box{
                    set_hexpand: true,
                    set_vexpand: true,

                    gtk::Paned {
                        set_position: WIDTH/2,
                        set_shrink_start_child:false,
                        set_shrink_end_child:false,

                        #[wrap(Some)]
                        set_start_child = &gtk::Frame{
                            set_hexpand:true,

                            gtk::Box{
                                set_orientation: gtk::Orientation::Vertical
                            }
                        },

                        set_end_child = Some(model.screen.widget()),

                    }
                },
                gtk::Box{
                    set_height_request: 50,
                    set_css_classes: &["brown_box"],

                    gtk::Box{
                        set_hexpand: true
                    },
                    gtk::Box{
                        set_spacing: 5,
                        gtk::Button{
                            set_label: "Ok",
                            connect_clicked => EditModelInputMsg::Response(gtk::ResponseType::Ok)
                        },

                        gtk::Button{
                            set_label: "Cancel",
                            connect_clicked => EditModelInputMsg::Response(gtk::ResponseType::Cancel)
                        }
                    }
                },
            },

            connect_close_request[sender] => move |m| {
                println!("destroy {:?}", m);
                sender.input(EditModelInputMsg::Hide);
                return gtk::glib::Propagation::Stop;
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: relm4::prelude::ComponentSender<Self>,
    ) -> relm4::prelude::ComponentParts<Self> {
        let screen = ActivityScreenModel::builder()
            .launch(())
            .forward(sender.input_sender(), |_| unimplemented!());

        let model = EditModel {
            is_active: true,
            screen,
        };

        let widgets = view_output!();

        return relm4::ComponentParts { widgets, model };
    }
    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            EditModelInputMsg::Show => {
                self.is_active = true;
            }
            EditModelInputMsg::Hide => {
                self.is_active = false;
            }
            EditModelInputMsg::Response(res) => {
                let _ = match res {
                    gtk::ResponseType::Ok => {
                        self.is_active = false;
                        sender.output(EditModelOutputMsg::Ok)
                    }
                    gtk::ResponseType::Cancel => {
                        self.is_active = false;
                        sender.output(EditModelOutputMsg::Cancel)
                    }
                    _ => return,
                };
            }
        };
    }
}
