use gtk::{
    self,
    glib::{self, subclass::types::ObjectSubclassIsExt},
    prelude::{ButtonExt, WidgetExt},
};

use crate::{
    db::query::Query,
    widgets::search::scriptures::download::download_page::{BibleDownload, DownloadBiblePage},
};

use super::utils;

mod imp {
    use std::cell::{Cell, RefCell};

    use gtk::{
        glib::{
            object::CastNone,
            subclass::{
                object::{ObjectImpl, ObjectImplExt},
                types::{ObjectSubclass, ObjectSubclassExt},
            },
            types::StaticType,
        },
        prelude::BoxExt,
        subclass::{box_::BoxImpl, widget::WidgetImpl},
    };

    use crate::widgets::search::scriptures::download::download_page::BibleDownload;

    use super::*;

    #[derive(Default, Debug)]
    pub struct TranslationListItem {
        text: RefCell<gtk::Label>,
        btn: RefCell<gtk::Button>,

        data: RefCell<BibleDownload>,
        already_added: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TranslationListItem {
        const NAME: &'static str = "TranslationListItem";
        type Type = super::TranslationListItem;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for TranslationListItem {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            let label = gtk::Label::builder()
                .hexpand(true)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .valign(gtk::Align::Start)
                .halign(gtk::Align::Start)
                .margin_start(8)
                .margin_end(8)
                .build();
            self.text.replace(label.clone());
            self.btn.borrow().set_css_classes(&["small-btn"]);

            obj.append(&label);
            obj.append(&self.btn.borrow().clone());
        }
    }
    impl WidgetImpl for TranslationListItem {}
    impl BoxImpl for TranslationListItem {}

    impl TranslationListItem {
        pub(super) fn load_data(&self, data: &BibleDownload) {
            // let data = main_data.details();
            self.already_added.set(data.already_added());
            self.data.replace(data.clone());
            println!("name2 {}", data.name());
            if let Some(name) = data.name().split(".").collect::<Vec<_>>().first().cloned() {
                println!("alread_added2 {}", self.already_added.get());
                if self.already_added.get() {
                    self.btn.borrow().set_label("Uninstall");
                    self.text.borrow().set_label(&format!("{name} (Installed)"));
                } else {
                    self.btn.borrow().set_label("Install");
                    self.text.borrow().set_label(name);
                };
            };

            let imp = self.downgrade();
            let already_added = self.already_added.get();
            let data = self.data.borrow().clone();
            self.btn.borrow().connect_clicked({
                let Some(imp) = imp.upgrade() else {
                    return;
                };
                move |btn| {
                    gtk::glib::spawn_future_local(glib::clone!(
                        #[weak]
                        imp,
                        #[weak]
                        btn,
                        #[strong]
                        data,
                        #[strong]
                        already_added,
                        async move {
                            let Some(sender) = btn
                                .ancestor(DownloadBiblePage::static_type())
                                .and_downcast::<DownloadBiblePage>()
                            else {
                                return;
                            };

                            //
                            btn.set_sensitive(false);
                            println!("alread_added {}", imp.already_added.get());
                            println!("name {}", data.name());
                            if already_added {
                                let delete_result = Query::delete_bible_translation(data.name());
                                match delete_result {
                                    Ok(_) => {
                                        sender.reload_translation();
                                        btn.set_label("Install");
                                    }
                                    Err(e) => {
                                        btn.set_label("Uninstall");
                                        eprintln!("SQL ERROR: error removing translation\n{:?}", e);
                                    }
                                }
                            } else {
                                btn.set_label("Installing");

                                let installed_translation = utils::import_bible(&data, |msg| {
                                    match msg {
                                        Ok(msg) => btn.set_label(&msg),
                                        Err(_) => {
                                            btn.set_label("Install");
                                            return;
                                        }
                                    };
                                })
                                .await;

                                match installed_translation {
                                    Some(t) => {
                                        sender.new_translation(t);
                                        btn.set_label("Installed");
                                    }
                                    None => btn.set_label("Install"),
                                };
                            }

                            btn.set_sensitive(true);
                        }
                    ));
                }
            });
        }
    }
}

glib::wrapper! {
    pub struct TranslationListItem(ObjectSubclass<imp::TranslationListItem>)
    @extends  gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for TranslationListItem {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl TranslationListItem {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn load_data(&self, data: &BibleDownload) {
        self.imp().load_data(data)
    }
}
