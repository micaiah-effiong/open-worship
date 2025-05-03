use std::{cell::RefCell, rc::Rc};

use gtk::{
    glib::clone,
    prelude::{ButtonExt, WidgetExt},
};
use relm4::{gtk, typed_view::list::RelmListItem, view, ComponentSender, RelmWidgetExt};

use crate::db::{connection::DatabaseConnection, query::Query};

use super::{
    download_modal::{BibleDownload, DownloadBibleInput, DownloadBibleModel},
    utils,
};

#[derive(Debug, Clone)]
pub struct BibleDownloadListItem {
    pub data: BibleDownload,
    pub conn: Rc<RefCell<Option<DatabaseConnection>>>,
    pub already_added: bool,
    pub parent_sender: ComponentSender<DownloadBibleModel>,
}

pub struct BibleListItemWidget {
    text: gtk::Label,
    btn: gtk::Button,
}

impl Drop for BibleListItemWidget {
    fn drop(&mut self) {}
}

impl RelmListItem for BibleDownloadListItem {
    type Root = gtk::Box;
    type Widgets = BibleListItemWidget;

    fn setup(_list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        view! {
            list_box = gtk::Box {
                #[name="text"]
                gtk::Label {
                    set_hexpand: true,
                    set_ellipsize: gtk::pango::EllipsizeMode::End,
                    set_align: gtk::Align::Start,
                    set_margin_horizontal: 8,
                },
                #[name="btn"]
                gtk::Button{
                    // set_label:"Install",
                },
            }
        }

        let widgets = BibleListItemWidget { text, btn };

        return (list_box, widgets);
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        let a = self.data.name.split(".").collect::<Vec<&str>>();
        if let Some(name) = a.get(0) {
            let name = name.to_string();
            match self.already_added {
                true => {
                    widgets.btn.set_label("Uninstall");
                    widgets.text.set_label(&format!("{} (Installed)", name));
                }
                false => {
                    widgets.btn.set_label("Install");
                    widgets.text.set_label(&format!("{}", name));
                }
            };
        }

        let conn = self.conn.clone();
        let data = self.data.clone();
        let already_added = self.already_added.clone();
        let sender = self.parent_sender.clone();

        widgets.btn.connect_clicked(move |btn| {
            gtk::glib::spawn_future_local(clone!(
                #[strong]
                btn,
                #[strong]
                data,
                #[strong]
                conn,
                #[strong]
                already_added,
                #[strong]
                sender,
                async move {
                    btn.set_sensitive(false);

                    match already_added {
                        true => {
                            let name = data.name.split(".").collect::<Vec<&str>>();
                            if let Some(name) = name.get(0) {
                                let name = format!("{}", name.to_string());
                                let delete_result = Query::delete_bible_translation(conn, name);

                                match delete_result {
                                    Ok(_) => {
                                        sender.input(DownloadBibleInput::ReloadTranslation);
                                        btn.set_label("Install");
                                    }
                                    Err(e) => {
                                        btn.set_label("Uninstall");
                                        eprintln!("SQL ERROR: error removing translation\n{:?}", e);
                                    }
                                }
                            }
                        }
                        false => {
                            btn.set_label("Installing");

                            let installed_translation = utils::import_bible(conn, &data, |msg| {
                                btn.set_label(&msg.to_string())
                            })
                            .await;
                            if let Some(installed_t) = installed_translation {
                                sender.input(DownloadBibleInput::NewTranslation(installed_t));
                            }

                            btn.set_label("Installed");
                        }
                    }

                    btn.set_sensitive(true);
                }
            ));
        });
    }
}
