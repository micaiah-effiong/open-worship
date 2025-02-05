use std::{cell::RefCell, rc::Rc};

use gtk::{glib::clone, pango::ffi::PANGO_WEIGHT_BOLD, prelude::*, SingleSelection};
use relm4::{prelude::*, typed_view::list::TypedListView};

use crate::{
    db::{connection::DatabaseConnection, query::Query},
    dto::{self, DisplayPayload, Song},
    widgets::{
        activity_screen::{ActivityScreenInput, ActivityScreenModel},
        search::songs::{
            edit_modal_list_item::EditSongModalListItem, list_item::SongListItemModel,
        },
    },
};

#[derive(Debug)]
pub struct EditModel {
    /// indicates if the edit modal is visible/active
    pub is_active: bool,
    pub is_new_song: bool,
    pub song: Option<Song>,
    pub screen: Controller<ActivityScreenModel>,
    pub list_wrapper: Rc<RefCell<TypedListView<EditSongModalListItem, SingleSelection>>>,
    pub song_title_entry: Rc<RefCell<gtk::Entry>>,
    pub db_connection: Rc<RefCell<DatabaseConnection>>,
}

#[derive(Debug)]
pub enum EditModelInputMsg {
    Show(Option<Song>),
    Hide,
    AddVerse,
    RemoveVerse,
    UpdateActivityScreen(DisplayPayload),

    #[doc(hidden)]
    Response(gtk::ResponseType),
}

#[derive(Debug)]
pub enum EditModelOutputMsg {
    Save(Song),
}

/// Help determine how to store date in datebase
/// Insert or Update
#[derive(Debug)]
pub enum EditModelOpeningState {
    /// INSERT
    New,
    /// Update(id)
    Edit(u32),
}

pub struct EditModelInit {
    pub db_connection: Rc<RefCell<DatabaseConnection>>,
}

const WIDTH: i32 = 1200;

#[relm4::component(pub)]
impl SimpleComponent for EditModel {
    type Init = EditModelInit;
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
                                    set_orientation: gtk::Orientation::Horizontal,
                                    gtk::Button {
                                        set_tooltip: "Add slide",
                                        set_icon_name:"plus",
                                        connect_clicked => EditModelInputMsg::AddVerse
                                    },
                                    gtk::Button {
                                        set_tooltip: "Remove slide",
                                        set_icon_name:"minus",
                                        connect_clicked => EditModelInputMsg::RemoveVerse
                                    },
                                },
                                gtk::Box {
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
        init: Self::Init,
        root: Self::Root,
        sender: relm4::prelude::ComponentSender<Self>,
    ) -> relm4::prelude::ComponentParts<Self> {
        let screen = ActivityScreenModel::builder()
            .launch(())
            .forward(sender.input_sender(), |_| unimplemented!());

        let typed_list_view: TypedListView<EditSongModalListItem, SingleSelection> =
            TypedListView::new();

        let model = EditModel {
            is_active: false,
            screen,
            list_wrapper: Rc::new(RefCell::new(typed_list_view)),
            song_title_entry: Rc::new(RefCell::new(gtk::Entry::default())),
            db_connection: init.db_connection.clone(),
            is_new_song: true,
            song: None,
        };

        let text_entry = model.song_title_entry.borrow().clone();

        let list_view = model.list_wrapper.clone().borrow().view.clone();

        let widgets = view_output!();
        model.register_response_ok(&widgets.ok_btn, &text_entry, &sender);
        EditModel::register_list_view_selection_changed(&model, sender.clone());

        return relm4::ComponentParts { widgets, model };
    }
    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            EditModelInputMsg::Show(song) => {
                self.is_active = true;
                self.song = song.clone();
                if let Some(song) = song {
                    self.load_song(&song, &sender);
                    self.is_new_song = false
                } else {
                    self.is_new_song = true
                }
                // TODO: check for insert or update
            }
            EditModelInputMsg::Hide => {
                self.is_active = false;
            }
            EditModelInputMsg::AddVerse => self.add_new_verse(&sender),
            EditModelInputMsg::RemoveVerse => {
                let selection_model = match self.list_wrapper.borrow().view.model() {
                    Some(model) => model,
                    None => return,
                };

                let size = selection_model.n_items();
                let mut pos: Option<u32> = None;

                for index in 0..size {
                    if !selection_model.is_selected(index) {
                        continue;
                    }

                    self.list_wrapper.borrow_mut().remove(index);
                    pos = Some(index);
                    break;
                }

                let len = selection_model.n_items();
                let mut pos = match pos {
                    Some(pos) => pos,
                    None => return,
                };

                if len.eq(&0) {
                    pos = len; // pos should be None
                } else if pos.ge(&len) {
                    pos = len - 1;
                }

                selection_model.select_item(pos, true);

                let mut list_child = self.list_wrapper.borrow().view.first_child();

                for i in 0..=pos {
                    if i == pos || list_child.is_none() {
                        break;
                    }

                    if let Some(list_item) = list_child {
                        list_child = list_item.next_sibling();
                    }
                }

                if let Some(list_item) = list_child {
                    list_item.grab_focus();
                }
            }
            EditModelInputMsg::UpdateActivityScreen(payload) => {
                // payload;
                println!("buffer changed {:?}", payload);
                self.screen
                    .emit(ActivityScreenInput::DisplayUpdate(payload));
            }
            EditModelInputMsg::Response(res) => {
                let _ = match res {
                    gtk::ResponseType::Ok => {
                        let mut verses: Vec<String> = Vec::new();

                        let mut list_wrapper = self.list_wrapper.borrow_mut();
                        for song_index in 0..list_wrapper.len() {
                            let song = match list_wrapper.get(song_index) {
                                Some(song) => song.borrow().clone(),
                                None => continue,
                            };

                            let start = song.text_buffer.start_iter();
                            let end = song.text_buffer.end_iter();
                            let song_str = song.text_buffer.text(&start, &end, true).to_string();

                            verses.push(song_str);
                        }

                        let title = self.song_title_entry.borrow_mut().buffer();
                        let song = Song::new(
                            title.text().to_string(),
                            verses,
                            match self.song.clone() {
                                Some(s) => s.song_id,
                                None => 0,
                            },
                        );
                        let song_model = SongListItemModel::new(song.clone());

                        // TODO:
                        // save song to database

                        let mut conn = self.db_connection.borrow_mut();
                        if self.is_new_song {
                            match Query::insert_song(&mut conn.connection, song) {
                                Ok(()) => (),
                                Err(x) => println!("SQL ERROR: {:?}", x),
                            };
                        } else {
                            match Query::update_song(&mut conn.connection, song) {
                                Ok(()) => (),
                                Err(x) => println!("SQL ERROR: {:?}", x),
                            };
                        }

                        // TODO: invalidate song query
                        let _ = sender.output(EditModelOutputMsg::Save(song_model.song));

                        list_wrapper.clear();
                        // to prevent "already mutably borrowed" error
                        drop(list_wrapper);

                        title.set_text("");
                        self.is_active = false;
                    }
                    gtk::ResponseType::Cancel => {
                        self.is_active = false;
                        self.list_wrapper.borrow_mut().clear();
                    }
                    _ => return,
                };

                self.screen.emit(ActivityScreenInput::DisplayUpdate(
                    dto::DisplayPayload::new("".to_string()),
                ));
            }
        };
    }
}

impl EditModel {
    fn register_response_ok(
        &self,
        button: &gtk::Button,
        text_entry: &gtk::Entry,
        sender: &ComponentSender<EditModel>,
    ) {
        let list = self.list_wrapper.clone();
        let db = self.db_connection.clone();

        button.connect_clicked(clone!(
            #[strong]
            text_entry,
            #[strong]
            sender,
            #[strong]
            list,
            #[strong]
            db,
            move |_| {
                if text_entry.buffer().text().is_empty() {
                    let label = gtk::Label::builder().label("Title cannot be empty").build();
                    let win = gtk::Window::builder()
                        .default_width(300)
                        .default_height(100)
                        .modal(true)
                        .focus_visible(true)
                        .child(&label)
                        .build();
                    win.set_visible(true);
                    return;
                }

                sender.input(EditModelInputMsg::Response(gtk::ResponseType::Ok));
            }
        ));
    }

    fn register_list_view_selection_changed(&self, sender: ComponentSender<Self>) {
        let list_wrapper = self.list_wrapper.clone();
        let list_model = list_wrapper.borrow().selection_model.clone();

        list_model.connect_selection_changed(clone!(
            #[strong]
            list_wrapper,
            move |m, _, _| {
                let list = list_wrapper.borrow();

                let index = m.selection().nth(0);

                if let Some(item) = list.get(index) {
                    let item = item.borrow();
                    let start = &item.text_buffer.start_iter();
                    let end = &item.text_buffer.end_iter();
                    let text = &item.text_buffer.text(start, end, true);
                    let payload = DisplayPayload::new(text.to_string());
                    println!("move focus: 1. {:?}\n 2. {:?}\n", payload, index);
                    sender.input(EditModelInputMsg::UpdateActivityScreen(payload));
                }
            }
        ));
    }

    fn load_song(&self, song: &Song, sender: &ComponentSender<Self>) {
        for verse in &song.verses {
            let buffer = gtk::TextBuffer::new(None);
            buffer.set_text(&verse.text);

            buffer.connect_changed(clone!(
                #[strong]
                sender,
                move |m| {
                    let text = &m.text(&m.start_iter(), &m.end_iter(), true);

                    let payload = DisplayPayload::new(text.to_string());
                    sender.input(EditModelInputMsg::UpdateActivityScreen(payload));
                }
            ));

            self.list_wrapper
                .borrow_mut()
                .append(EditSongModalListItem {
                    text_buffer: buffer,
                });
        }

        self.song_title_entry.borrow().set_text(&song.title);
        if let Some(model) = self.list_wrapper.borrow().view.model() {
            if model.n_items() > 0 {
                model.select_item(0, true);
            }

            if let Some(child) = self.list_wrapper.borrow().view.first_child() {
                child.grab_focus();
            }
        }
    }

    fn add_new_verse(&self, sender: &ComponentSender<Self>) {
        let buffer = gtk::TextBuffer::new(None);
        buffer.set_text("New verse");

        // tag
        let bold_tag = buffer.create_tag(Some("bold"), &[("weight", &PANGO_WEIGHT_BOLD)]);

        let (start, end) = buffer.bounds();
        match bold_tag {
            Some(b) => buffer.apply_tag(&b, &start, &end),
            None => (),
        }

        // println!(
        //     "NEW BUFFER\n {:?}\n{:?}",
        //     buffer.text(&start, &end, true),
        //     buffer.tag_table()
        // );

        buffer.connect_changed(clone!(
            #[strong]
            sender,
            move |m| {
                let text = &m.text(&m.start_iter(), &m.end_iter(), true);

                let payload = DisplayPayload::new(text.to_string());
                sender.input(EditModelInputMsg::UpdateActivityScreen(payload));
            }
        ));

        self.list_wrapper
            .borrow_mut()
            .append(EditSongModalListItem {
                text_buffer: buffer,
            });

        if let Some(model) = self.list_wrapper.borrow().view.model() {
            model.select_item(model.n_items() - 1, true);
            if let Some(child) = self.list_wrapper.borrow().view.last_child() {
                child.grab_focus();
            }
        }
    }
}
