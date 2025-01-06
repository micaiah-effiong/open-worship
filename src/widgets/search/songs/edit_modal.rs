use std::{cell::RefCell, rc::Rc};

use gtk::{glib::clone, pango::ffi::PANGO_WEIGHT_BOLD, prelude::*, SingleSelection};
use relm4::{prelude::*, typed_view::list::TypedListView};

use crate::{
    dto::{DisplayPayload, Song},
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
    pub screen: Controller<ActivityScreenModel>,
    pub list_wrapper: Rc<RefCell<TypedListView<EditSongModalListItem, SingleSelection>>>,
    pub song_title_entry: Rc<RefCell<gtk::Entry>>,
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
                        },

                        #[name="cancel_btn"]
                        gtk::Button{
                            set_label: "Cancel",
                        }
                    }
                },
            },

            connect_close_request[sender] => move |_m| {
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

        let model = EditModel {
            is_active: false,
            screen,
            list_wrapper: Rc::new(RefCell::new(typed_list_view)),
            song_title_entry: Rc::new(RefCell::new(gtk::Entry::default())),
        };

        let text_entry = model.song_title_entry.borrow().clone();

        let list_view = model.list_wrapper.clone().borrow().view.clone();

        let widgets = view_output!();
        EditModel::register_response_ok(&widgets.ok_btn, &text_entry, &sender);
        EditModel::register_response_cancel(&widgets.cancel_btn, &sender);
        EditModel::register_list_view_selection_changed(&model, &sender);

        return relm4::ComponentParts { widgets, model };
    }
    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            EditModelInputMsg::Show(song) => {
                self.is_active = true;
                if let Some(song) = song {
                    self.load_song(&song, &sender)
                }
            }
            EditModelInputMsg::Hide => {
                self.is_active = false;
            }
            EditModelInputMsg::AddVerse => self.add_new_verse(&sender),
            EditModelInputMsg::RemoveVerse => {
                let mut list = self.list_wrapper.borrow_mut();
                let list_view = list.view.clone();
                let model = match list_view.model() {
                    Some(model) => model,
                    None => return,
                };

                let s = model.selection().nth(0);
                list.remove(s);

                if model.n_items().eq(&0) {
                    sender.input(EditModelInputMsg::UpdateActivityScreen(
                        DisplayPayload::new(String::new()),
                    ));
                    return;
                }

                list_view.grab_focus();
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
                        let song_model =
                            SongListItemModel::new(Song::new(title.text().to_string(), verses));

                        // TODO:
                        // save song to database
                        // invalidate song query
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
            }
        };
    }
}

impl EditModel {
    fn register_response_cancel(button: &gtk::Button, sender: &ComponentSender<EditModel>) {
        button.connect_clicked(clone!(
            #[strong]
            sender,
            move |_| {
                sender.input(EditModelInputMsg::Response(gtk::ResponseType::Cancel));

                sender.input(EditModelInputMsg::UpdateActivityScreen(
                    DisplayPayload::new(String::new()),
                ));
            }
        ));
    }

    fn register_response_ok(
        button: &gtk::Button,
        text_entry: &gtk::Entry,
        sender: &ComponentSender<EditModel>,
    ) {
        button.connect_clicked(clone!(
            #[strong]
            text_entry,
            #[strong]
            sender,
            move |_| {
                if !text_entry.buffer().text().is_empty() {
                    sender.input(EditModelInputMsg::UpdateActivityScreen(
                        DisplayPayload::new(String::new()),
                    ));

                    sender.input(EditModelInputMsg::Response(gtk::ResponseType::Ok));
                    return;
                }

                let label = gtk::Label::builder().label("Title cannot be empty").build();

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

    fn register_list_view_selection_changed(&self, sender: &ComponentSender<Self>) {
        let list_wrapper = self.list_wrapper.clone();

        if let Some(view) = self.list_wrapper.borrow().view.model() {
            view.connect_selection_changed(clone!(
                #[strong]
                list_wrapper,
                #[strong]
                sender,
                move |m, _, _| {
                    let list = list_wrapper.borrow();

                    let s = m.selection().nth(0);
                    if let Some(item) = list.get(s) {
                        let text_buffer = &item.borrow().text_buffer;
                        let (start, end) = text_buffer.bounds();
                        sender.input(EditModelInputMsg::UpdateActivityScreen(
                            DisplayPayload::new(String::from(text_buffer.text(&start, &end, true))),
                        ));
                    }
                }
            ));
        }
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
        buffer.create_tag(Some("bold"), &[("weight", &PANGO_WEIGHT_BOLD)]);

        let (start, end) = buffer.bounds();
        buffer.apply_tag_by_name("bold", &start, &end);

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
                text_buffer: buffer.clone(),
            });
        let wrapper = self.list_wrapper.borrow();

        let list_view = &wrapper.view;
        if let Some(model) = list_view.model() {
            if wrapper.len().eq(&1) {
                model.select_item(0, true);
                let text = &buffer.text(&buffer.start_iter(), &buffer.end_iter(), true);
                let payload = DisplayPayload::new(text.to_string());
                sender.input(EditModelInputMsg::UpdateActivityScreen(payload));
            } else {
                model.select_item(model.n_items() - 1, true);
            }
            if let Some(child) = list_view.last_child() {
                child.grab_focus();
            }
        }
    }
}
