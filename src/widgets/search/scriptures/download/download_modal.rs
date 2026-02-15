use gtk::{
    self,
    glib::{self, subclass::types::ObjectSubclassIsExt},
    prelude::*,
};
use serde::{Deserialize, Serialize};

use crate::{db::query::Query, utils::ListViewExtra};

#[derive(Default, Debug, Deserialize, Serialize, Clone, glib::Boxed)]
#[boxed_type(name = "BibleDownload")]
pub struct BibleDownload {
    pub name: String,
    pub download_url: Option<String>,
    pub active: Option<bool>,
}

mod bible {
    use std::cell::{Cell, RefCell};

    use gtk::glib::{
        self, Properties,
        prelude::ObjectExt,
        subclass::{object::ObjectImpl, types::ObjectSubclass},
    };
    use gtk::subclass::prelude::DerivedObjectProperties;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::BibleDownloadObj)]
    pub struct BibleDownloadObj {
        #[property(get)]
        pub details: RefCell<super::BibleDownload>,
        #[property(get, set)]
        pub already_added: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BibleDownloadObj {
        const NAME: &'static str = "BibleDownloadObj";
        type Type = super::BibleDownloadObj;
    }

    #[glib::derived_properties]
    impl ObjectImpl for BibleDownloadObj {}
}

glib::wrapper! {
pub struct BibleDownloadObj(ObjectSubclass<bible::BibleDownloadObj>);
}

impl Default for BibleDownloadObj {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl BibleDownloadObj {
    fn from_data(data: BibleDownload) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().details.replace(data);

        obj
    }
}

mod signals {
    pub(super) const NEW_TRANSLATION: &str = "new-translation";
    pub(super) const RELOAD_TRANSLATION: &str = "reload-translation";
}

mod imp {
    use std::{cell::RefCell, sync::OnceLock};

    use gtk::{
        gio,
        glib::{
            self,
            object::{Cast, CastNone},
            subclass::{Signal, object::ObjectImpl, types::ObjectSubclass},
            types::StaticType,
        },
        prelude::ListItemExt,
        subclass::{prelude::*, window::WindowImpl},
    };

    use crate::{
        format_resource,
        utils::ListViewExtra,
        widgets::search::scriptures::download::{
            download_list_item::TranslationListItem,
            download_modal::{BibleDownload, BibleDownloadObj, signals},
        },
    };

    #[derive(Default, Debug, gtk::CompositeTemplate)]
    #[template(resource = "/com/openworship/app/ui/download_bible_window.ui")]
    pub struct DownloadBibleWindow {
        pub(super) installed_translations: RefCell<Vec<String>>,
        #[template_child]
        pub(super) listview: gtk::TemplateChild<gtk::ListView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DownloadBibleWindow {
        const NAME: &'static str = "DownloadBibleWindow";
        type Type = super::DownloadBibleWindow;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DownloadBibleWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let listview = self.listview.clone();
            let model = gtk::gio::ListStore::new::<BibleDownloadObj>();
            let model = gtk::SingleSelection::new(Some(model));
            listview.set_model(Some(&model));

            let factory = gtk::SignalListItemFactory::new();
            listview.set_factory(Some(&factory));
            factory.connect_setup(|_, listitem| {
                let li = listitem
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Expected ListItem");

                li.set_child(Some(&TranslationListItem::default()));
            });
            factory.connect_bind(|_, listitem| {
                let li = listitem
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Expected ListItem");

                let item = li
                    .item()
                    .and_downcast::<BibleDownloadObj>()
                    .expect("Expected SongObject");

                let child = li
                    .child()
                    .and_downcast::<TranslationListItem>()
                    .expect("Exected SongListItem");

                child.load_data(&item);
            });
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();

            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder(signals::NEW_TRANSLATION)
                        .param_types([String::static_type()])
                        .build(),
                    Signal::builder(signals::RELOAD_TRANSLATION).build(),
                ]
            })
        }
    }
    impl WidgetImpl for DownloadBibleWindow {}
    impl WindowImpl for DownloadBibleWindow {}

    impl DownloadBibleWindow {
        pub(super) fn register_import_bible(&self, translations: Vec<String>) {
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

            let bible_resource = match gio::resources_lookup_data(
                format_resource!("data/bible.json"),
                gio::ResourceLookupFlags::NONE,
            ) {
                Ok(r) => r,
                Err(e) => {
                    glib::g_warning!("Resource", "Failed to load resource: {:?}", e);
                    return;
                }
            };
            let download_list_result = match str::from_utf8(&bible_resource) {
                Ok(r) => serde_json::from_str::<Vec<BibleDownload>>(r),
                Err(e) => {
                    glib::g_warning!("Resource", "Failed str from bytes : {:?}", e);
                    return;
                }
            };

            if let Ok(download_list) = download_list_result {
                for item in download_list {
                    if !item.active.unwrap_or(false) || item.download_url.is_none() {
                        continue;
                    }

                    let mut item = item.clone();
                    if let Some((before, _)) = item.name.split_once('.') {
                        item.name = before.to_string();
                    }

                    let name = item.name.clone();
                    let list_data = BibleDownloadObj::from_data(item);
                    list_data.set_already_added(translation_map.contains_key(&name));
                    self.listview.append_item(&list_data);
                }
            }
        }
    }
}

glib::wrapper! {
pub struct DownloadBibleWindow(ObjectSubclass<imp::DownloadBibleWindow>)
    @extends gtk::Window, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,gtk::Native,gtk::Root, gtk::ShortcutManager;
}

impl Default for DownloadBibleWindow {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl DownloadBibleWindow {
    pub fn new(installed_translations: Vec<String>) -> Self {
        let obj: Self = glib::Object::new();

        obj.imp()
            .installed_translations
            .replace(installed_translations.clone());
        obj.imp().register_import_bible(installed_translations);
        obj
    }

    fn emit_new_translation(&self, data: String) {
        self.emit_by_name::<()>(signals::NEW_TRANSLATION, &[&data]);
    }
    pub fn connect_new_translation<F: Fn(&Self, String) + 'static>(&self, f: F) {
        self.connect_closure(
            signals::NEW_TRANSLATION,
            false,
            glib::closure_local!(|obj: &Self, data: String| f(obj, data)),
        );
    }

    fn emit_reload_translation(&self) {
        self.emit_by_name::<()>(signals::RELOAD_TRANSLATION, &[]);
    }
    pub fn connect_reload_translation<F: Fn(&Self) + 'static>(&self, f: F) {
        self.connect_closure(
            signals::RELOAD_TRANSLATION,
            false,
            glib::closure_local!(|obj: &Self| f(obj)),
        );
    }

    pub fn new_translation(&self, t: String) {
        self.emit_new_translation(t);
        // update list
        self.reload_translation();
    }
    pub fn reload_translation(&self) {
        if let Ok(list) = Query::get_translations() {
            let mut installed_translations = self.imp().installed_translations.borrow_mut();
            installed_translations.clear();
            installed_translations.extend(list);

            self.imp().listview.remove_all();
            self.imp()
                .register_import_bible(installed_translations.clone());
            self.emit_reload_translation();
        }
    }
}
