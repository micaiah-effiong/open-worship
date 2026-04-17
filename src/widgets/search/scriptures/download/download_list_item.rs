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
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };

    use futures_util::stream::AbortHandle;
    use gtk::{
        glib::{
            object::{CastNone, ObjectExt},
            subclass::{
                object::{ObjectImpl, ObjectImplExt},
                types::{ObjectSubclass, ObjectSubclassExt},
            },
            types::StaticType,
        },
        prelude::BoxExt,
        subclass::{box_::BoxImpl, widget::WidgetImpl},
    };

    use crate::widgets::search::scriptures::download::{
        download_page::BibleDownload, utils::ImportBibleStatus,
    };

    use super::*;

    #[derive(Default, Debug)]
    pub struct TranslationListItem {
        text: RefCell<gtk::Label>,
        status_label: RefCell<gtk::Label>,
        cancel_btn: RefCell<gtk::Button>,
        btn: RefCell<gtk::Button>,

        data: RefCell<BibleDownload>,
        already_added: Cell<bool>,
        cancel_state: Rc<RefCell<Option<AbortHandle>>>,
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

            let btn = self.btn.borrow().clone();
            btn.set_css_classes(&["flat", "circular", "small-20-btn"]);
            btn.set_icon_name("download");

            let cancel_btn = self.cancel_btn.borrow().clone();
            cancel_btn.set_icon_name("circle-close");
            cancel_btn.set_css_classes(&["flat", "circular", "small-20-btn"]);
            cancel_btn.set_visible(false);
            cancel_btn.set_widget_name("Abort");
            let cancel = self.cancel_state.clone();
            cancel_btn.connect_clicked(move |btn| {
                btn.set_visible(false);
                let Some(handler) = cancel.take() else {
                    return;
                };
                handler.abort();
            });

            obj.append(&label);
            obj.append(&self.status_label.borrow().clone());
            obj.append(&btn.clone());
            obj.append(&cancel_btn);

            btn.bind_property("visible", &cancel_btn, "visible")
                .invert_boolean()
                .sync_create()
                .build();
            // btn.set_visible(false);
        }
    }
    impl WidgetImpl for TranslationListItem {}
    impl BoxImpl for TranslationListItem {}

    impl TranslationListItem {
        pub(super) fn load_data(&self, data: &BibleDownload) {
            // let data = main_data.details();
            self.already_added.set(data.already_added());
            self.data.replace(data.clone());

            let name = data.name();
            if self.already_added.get() {
                self.btn.borrow().set_icon_name("uninstall");
                self.text.borrow().set_label(&format!("{name} (Installed)"));
            } else {
                self.btn.borrow().set_icon_name("download");
                self.text.borrow().set_label(&name);
            };

            let already_added = self.already_added.get();
            let data = self.data.borrow().clone();
            let status_label = self.status_label.borrow().clone();
            let cancel = self.cancel_state.clone();

            self.btn.borrow().connect_clicked({
                move |btn| {
                    gtk::glib::spawn_future_local(glib::clone!(
                        #[weak]
                        btn,
                        #[strong]
                        data,
                        #[strong]
                        already_added,
                        #[strong]
                        status_label,
                        #[strong]
                        cancel,
                        async move {
                            let Some(sender) = btn
                                .ancestor(DownloadBiblePage::static_type())
                                .and_downcast::<DownloadBiblePage>()
                            else {
                                return;
                            };

                            if already_added {
                                let delete_result = Query::delete_bible_translation(data.name());
                                match delete_result {
                                    Ok(_) => {
                                        sender.reload_translation();
                                        btn.set_icon_name("download");
                                    }
                                    Err(e) => {
                                        btn.set_icon_name("uninstall");
                                        eprintln!("SQL ERROR: error removing translation\n{:?}", e);
                                    }
                                }
                            } else {
                                btn.set_visible(false);
                                status_label.set_label("0%");

                                let cancel_clone = cancel.clone();
                                let abort_handler =
                                    utils::import_bible2(data.clone(), move |msg| {
                                        match msg {
                                            Ok(ImportBibleStatus::Progress(pct)) => {
                                                status_label.set_label(&format!("{pct}%"))
                                            }
                                            Ok(ImportBibleStatus::Done(name)) => {
                                                status_label.set_label("");
                                                btn.set_visible(true);
                                                sender.new_translation(name);
                                            }
                                            Ok(_) => (),
                                            Err(_) => {
                                                status_label.set_label("");
                                                btn.set_visible(true);
                                                cancel_clone.replace(None);
                                                return;
                                            }
                                        };
                                    });

                                cancel.replace(Some(abort_handler));
                            }
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
