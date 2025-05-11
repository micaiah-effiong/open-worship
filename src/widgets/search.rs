mod background;
mod scriptures;
mod songs;

use std::{cell::RefCell, rc::Rc};

use gtk::{glib::property::PropertySet, prelude::*};
use relm4::prelude::*;

use background::{SearchBacgroundOutput, SearchBackgroundInit, SearchBackgroundModel};
use scriptures::{SearchScriptureInit, SearchScriptureModel, SearchScriptureOutput};
use songs::{SearchSongInit, SearchSongModel, SearchSongOutput};

use crate::{db::connection::DatabaseConnection, dto};

const MIN_GRID_HEIGHT: i32 = 300;
// const MIN_GRID_WIDTH: i32 = 300;

// search area (notebook)
#[derive(Debug)]
pub enum SearchModelInput {
    PreviewBackground(String),
    PreviewScriptures(dto::ListPayload),
    PreviewSongs(dto::ListPayload),
    AddToSchedule(dto::ListPayload),
}

#[derive(Debug)]
pub enum SearchOutput {
    PreviewBackground(String),
    PreviewScriptures(dto::ListPayload),
    PreviewSongs(dto::ListPayload),
    AddToSchedule(dto::ListPayload),
}

#[derive(Debug)]
pub struct SearchModel {
    background_page: relm4::Controller<SearchBackgroundModel>,
    scripture_page: relm4::Controller<SearchScriptureModel>,
    song_page: relm4::Controller<SearchSongModel>,
    background_image: Rc<RefCell<Option<String>>>,
    db_connection: Rc<RefCell<Option<DatabaseConnection>>>,
}

impl SearchModel {
    fn convert_background_msg(msg: SearchBacgroundOutput) -> SearchModelInput {
        match msg {
            SearchBacgroundOutput::SendPreviewBackground(bg_src) => {
                SearchModelInput::PreviewBackground(bg_src)
            }
        }
    }

    fn convert_scripture_msg(msg: SearchScriptureOutput) -> SearchModelInput {
        match msg {
            SearchScriptureOutput::SendScriptures(list_payload) => {
                SearchModelInput::PreviewScriptures(list_payload)
            }
            SearchScriptureOutput::SendToSchedule(list_payload) => {
                SearchModelInput::AddToSchedule(list_payload)
            }
        }
    }

    fn convert_song_msg(msg: SearchSongOutput) -> SearchModelInput {
        match msg {
            SearchSongOutput::SendToPreview(list_payload) => {
                SearchModelInput::PreviewSongs(list_payload)
            }
            SearchSongOutput::SendToSchedule(list_payload) => {
                SearchModelInput::AddToSchedule(list_payload)
            }
        }
    }
}

pub struct SearchInit {
    pub db_connection: Rc<RefCell<Option<DatabaseConnection>>>,
}

impl SearchModel {}

#[relm4::component(pub)]
impl SimpleComponent for SearchModel {
    type Init = SearchInit;
    type Output = SearchOutput;
    type Input = SearchModelInput;

    view! {
        #[root]
        gtk::Box{
            set_orientation: gtk::Orientation::Vertical,
            set_height_request: MIN_GRID_HEIGHT,
            set_hexpand: true,
            set_homogeneous: true,

            #[name="tab_box"]
            gtk::Box {
                set_orientation:gtk::Orientation::Horizontal,
                set_spacing: 3,
                set_css_classes: &["purple_box", "ow-listview"],
                set_height_request: 48,

                gtk::Notebook {
                    set_hexpand: true,

                    #[local_ref]
                    append_page[Some(&gtk::Label::new(Some("Songs")))] = song_page_widget -> gtk::Box {},

                    #[local_ref]
                    append_page[Some(&gtk::Label::new(Some("Scriptures")))] = scripture_page_widget -> gtk::Box{},

                    #[local_ref]
                    append_page[Some(&gtk::Label::new(Some("Backgrounds")))] = background_page_widget -> gtk::Box{}
                }
            }

        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let song_page = SearchSongModel::builder()
            .launch(SearchSongInit {
                db_connection: init.db_connection.clone(),
            })
            .forward(sender.input_sender(), SearchModel::convert_song_msg);
        let scripture_page = SearchScriptureModel::builder()
            .launch(SearchScriptureInit {
                db_connection: init.db_connection.clone(),
            })
            .forward(sender.input_sender(), SearchModel::convert_scripture_msg);
        let background_page = SearchBackgroundModel::builder()
            .launch(SearchBackgroundInit {})
            .forward(sender.input_sender(), SearchModel::convert_background_msg);

        let model = SearchModel {
            song_page,
            background_page,
            scripture_page,
            background_image: Rc::new(RefCell::new(None)),
            db_connection: init.db_connection,
        };

        let background_page_widget = model.background_page.widget();
        let scripture_page_widget = model.scripture_page.widget();
        let song_page_widget = model.song_page.widget();

        let widgets = view_output!();

        relm4::ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            SearchModelInput::PreviewBackground(bg) => {
                self.background_image.set(Some(bg.clone()));
                let _ = sender.output(SearchOutput::PreviewBackground(bg));
            }
            SearchModelInput::PreviewScriptures(list) => {
                let item = dto::ListPayload::new(
                    list.text,
                    list.position,
                    list.list,
                    match list.background_image {
                        Some(bg) => Some(bg),
                        None => self.background_image.borrow().clone(),
                    },
                );
                let _ = sender.output(SearchOutput::PreviewScriptures(item));
            }
            SearchModelInput::PreviewSongs(list) => {
                let item = dto::ListPayload::new(
                    list.text,
                    list.position,
                    list.list,
                    match list.background_image {
                        Some(bg) => Some(bg),
                        None => self.background_image.borrow().clone(),
                    },
                );
                let _ = sender.output(SearchOutput::PreviewSongs(item));
            }
            SearchModelInput::AddToSchedule(list) => {
                let item = dto::ListPayload::new(
                    list.text,
                    list.position,
                    list.list,
                    match list.background_image {
                        Some(bg) => Some(bg),
                        None => self.background_image.borrow().clone(),
                    },
                );
                let _ = sender.output(SearchOutput::AddToSchedule(item));
            }
        };
    }
}
