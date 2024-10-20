use std::{cell::RefCell, rc::Rc};

use gtk::{glib::clone, prelude::*, SingleSelection};
use relm4::{prelude::*, typed_view::list::TypedListView, SimpleComponent};

use crate::widgets::{
    activity_screen::ActivityScreenModel,
    search::songs::{edit_modal_list_item::EditSongModalListItem, list_item::SongListItem},
};

#[derive(Debug)]
pub struct EditModel {
    /// indicates if the edit modal is visible/active
    pub is_active: bool,
    pub screen: Controller<ActivityScreenModel>,
    pub list_wrapper: Rc<RefCell<TypedListView<EditSongModalListItem, SingleSelection>>>,
    pub song_title_entry: Rc<RefCell<gtk::Entry>>,
}

#[derive(Debug)]
pub enum EditModelInputMsg {
    Show,
    Hide,
    AddVerse,

    #[doc(hidden)]
    Response(gtk::ResponseType),
}

#[derive(Debug)]
pub enum EditModelOutputMsg {
    Save(SongListItem),
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

                    gtk::Box{
                        set_margin_vertical: 13,
                        set_margin_horizontal: 6,
                        gtk::Label{
                            set_label: "Title",
                            set_margin_end: 6,
                        },
                        #[local_ref]
                        text_entry -> gtk::Entry{
                            set_placeholder_text:Some("Enter song title"),
                            connect_insert_text=>move|l,m,n|{
                                println!("song_title: \n{:?} \n{:?} \n{:?}", l,m,n);
                            }
                        },
                    },
                    gtk::Box{
                        set_hexpand:true
                    }
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
                                set_orientation: gtk::Orientation::Vertical,
                                set_hexpand:true,

                                gtk::Box{
                                    set_vexpand: true,
                                    set_orientation: gtk::Orientation::Vertical,


                                    gtk::ScrolledWindow {
                                        set_vexpand: true,

                                        #[local_ref]
                                        list_view -> gtk::ListView {
                                            set_vexpand: true,
                                            set_css_classes: &["blue_box"],
                                        },
                                    }
                                },
                                gtk::Box {
                                    gtk::Button {
                                        set_tooltip: "Add side",
                                        set_icon_name:"plus",
                                        connect_clicked => EditModelInputMsg::AddVerse
                                    },
                                },

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
                        #[name="ok_btn"]
                        gtk::Button{
                            set_label: "Ok",
                            // connect_clicked[sender, text_entry] => move|_|{
                            //     if text_entry.buffer().text().is_empty() {
                            //         let win = gtk::Window::builder()
                            //             .default_width(300)
                            //             .default_height(100)
                            //             .modal(true)
                            //             .focus_visible(true)
                            //             .build();
                            //
                            //         win.show();
                            //         return;
                            //     }
                            //
                            //     sender.input(EditModelInputMsg::Response(gtk::ResponseType::Ok));
                            // }
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

        let typed_list_view: TypedListView<EditSongModalListItem, SingleSelection> =
            TypedListView::new();

        // let verse_list: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));

        // for i in 0..=7 {
        //     typed_list_view.append(EditSongModalListItem {
        //         text: format!("Verse {i}"),
        //     })
        // }

        let model = EditModel {
            is_active: false,
            screen,
            list_wrapper: Rc::new(RefCell::new(typed_list_view)),
            song_title_entry: Rc::new(RefCell::new(gtk::Entry::default())),
        };

        let text_entry = model.song_title_entry.borrow().clone();

        let list_view = model.list_wrapper.clone().borrow().view.clone();

        let widgets = view_output!();
        EditModel::register_ok(&widgets.ok_btn, &text_entry, &sender);

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
            EditModelInputMsg::AddVerse => {
                let buffer = gtk::TextBuffer::new(None);
                buffer.set_text("New verse");

                self.list_wrapper
                    .borrow_mut()
                    .append(EditSongModalListItem {
                        text_buffer: buffer,
                    });
            }
            EditModelInputMsg::Response(res) => {
                let _ = match res {
                    gtk::ResponseType::Ok => {
                        let mut verses: Vec<String> = Vec::new();

                        let mut rm_list = self.list_wrapper.borrow_mut();
                        for song_index in 0..rm_list.len() {
                            let song = match rm_list.get(song_index) {
                                Some(song) => song.borrow().clone(),
                                None => continue,
                            };

                            let start = song.text_buffer.start_iter();
                            let end = song.text_buffer.end_iter();
                            let song_str = song.text_buffer.text(&start, &end, true).to_string();

                            verses.push(song_str);
                        }

                        let title = self.song_title_entry.borrow_mut().buffer();
                        let song = SongListItem::new(title.text().to_string(), verses);

                        // TODO:
                        // save song to database
                        // invalidate song query
                        let _ = sender.output(EditModelOutputMsg::Save(song));

                        rm_list.clear();
                        title.set_text("");
                        self.is_active = false;
                    }
                    gtk::ResponseType::Cancel => {
                        self.is_active = false;
                        self.list_wrapper.borrow_mut().clear();
                    }
                    _ => return,
                };
            }
        };
    }
}

impl EditModel {
    fn register_ok(
        button: &gtk::Button,
        text_entry: &gtk::Entry,
        sender: &ComponentSender<EditModel>,
    ) {
        button.connect_clicked(clone!(
            @strong button,
            @strong text_entry,
            @strong sender,
            => move |_| {
                if !text_entry.buffer().text().is_empty() {
                    sender.input(EditModelInputMsg::Response(gtk::ResponseType::Ok));
                    return;
                }

                let label = gtk::Label::builder()
                    .label("Title cannot be empty")
                    .build();

                let win = gtk::Window::builder()
                    .default_width(300)
                    .default_height(100)
                    .modal(true)
                    .focus_visible(true)
                    .child(&label)
                    .build();

                win.show();
            }
        ));
    }
}
