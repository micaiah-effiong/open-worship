use std::{cell::RefCell, rc::Rc};

use gtk::{prelude::*, SingleSelection};
use relm4::{
    gtk, typed_view::list::TypedListView, ComponentParts, ComponentSender, SimpleComponent,
};
use serde::{Deserialize, Serialize};

use crate::db::{connection::DatabaseConnection, query::Query};

use super::download_list_item::BibleDownloadListItem;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BibleDownload {
    pub name: String,
    pub download_url: Option<String>,
}

#[derive(Debug)]
pub struct DownloadBibleModel {
    list: Rc<RefCell<TypedListView<BibleDownloadListItem, SingleSelection>>>,
    db_connection: Rc<RefCell<Option<DatabaseConnection>>>,
    installed_translations: Vec<String>,
    is_active: bool,
}

#[derive(Debug)]
pub enum DownloadBibleInput {
    NewTranslation(String),
    ReloadTranslation,
    Done(String),
    Open,
    Close,
}

#[derive(Debug)]
pub enum DownloadBibleOutput {
    NewTranslation(String),
    ReloadTranslation,
}

pub struct DownloadBibleInit {
    pub db_connection: Rc<RefCell<Option<DatabaseConnection>>>,
    pub installed_translations: Vec<String>,
}

impl DownloadBibleModel {
    fn register_import_bible(
        &mut self,
        translations: Vec<String>,
        db: Rc<RefCell<Option<DatabaseConnection>>>,
        sender: ComponentSender<Self>,
        // btn: &gtk::Button,
    ) {
        let conn = db.clone();
        let mut translation_map: std::collections::HashMap<String, bool> =
            std::collections::HashMap::new();

        translations.iter().for_each(|i| {
            translation_map.insert(i.to_string(), true);
        });

        println!(
            "TRANSLATIONS \ntotal={}\nlist={:?}",
            translations.len(),
            translations
        );

        let bible_src = include_str!("../../../../../bible_download_path.json");
        let download_list_result = serde_json::from_str::<Vec<BibleDownload>>(bible_src);
        let mut list = self.list.borrow_mut();

        if let Ok(download_list) = download_list_result {
            for item in download_list {
                if item.download_url.is_some() {
                    let item_name = item.name.clone();
                    let item_name = item_name.split(".").collect::<Vec<&str>>();
                    if let Some(name) = item_name.first() {
                        let name = name.to_string();
                        list.append(BibleDownloadListItem {
                            data: item.clone(),
                            conn: conn.clone(),
                            already_added: translation_map.contains_key(&name),
                            parent_sender: sender.clone(),
                        });
                    }
                }
            }
        }
    }
}

#[relm4::component(pub)]
impl SimpleComponent for DownloadBibleModel {
    type Input = DownloadBibleInput;
    type Output = DownloadBibleOutput;
    type Init = DownloadBibleInit;

    view! {
        #[root]
        gtk::Window{
            set_title: Some("Download bible"),
            set_default_width:700,
            set_default_height:700,
            set_modal: true,
            set_focus_visible: true,
            set_resizable: false,

            #[watch]
            set_visible: model.is_active,

            connect_close_request[sender] => move |m| {
                println!("destroy {:?}", m);
                sender.input(DownloadBibleInput::Close);
                gtk::glib::Propagation::Stop
            },

            gtk::Box {
                gtk::ScrolledWindow{
                    set_vexpand: true,
                    set_hexpand: true,
                    #[wrap(Some)]
                    #[local_ref]
                    set_child = &list_view -> gtk::ListView {}
                },

                gtk::Box{

                }
            }

        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let list = Rc::new(RefCell::new(TypedListView::new()));
        let list_view = list.borrow().view.clone();
        let mut model = DownloadBibleModel {
            list,
            db_connection: init.db_connection.clone(),
            is_active: false,
            installed_translations: init.installed_translations.clone(),
        };

        //
        let widgets = view_output!();

        model.register_import_bible(
            model.installed_translations.clone(),
            model.db_connection.clone(),
            sender.clone(),
        );

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            DownloadBibleInput::Done(_) => todo!(),
            DownloadBibleInput::Open => self.is_active = true,
            DownloadBibleInput::Close => self.is_active = false,

            DownloadBibleInput::NewTranslation(t) => {
                // run installation
                println!("INSTALL >> {}", t);
                let _ = sender.output(DownloadBibleOutput::NewTranslation(t));
                // update list
                sender.input(DownloadBibleInput::ReloadTranslation);
            }
            DownloadBibleInput::ReloadTranslation => {
                if let Ok(list) = Query::get_translations(self.db_connection.clone()) {
                    self.installed_translations.clear();
                    self.installed_translations.extend(list);

                    self.list.borrow_mut().clear();
                    self.register_import_bible(
                        self.installed_translations.clone(),
                        self.db_connection.clone(),
                        sender.clone(),
                    );

                    let _ = sender.output(DownloadBibleOutput::ReloadTranslation);
                }
            }
        }
    }
}
