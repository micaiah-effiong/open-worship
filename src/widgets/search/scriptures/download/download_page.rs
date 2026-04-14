use gobject_macro::gobject_props;
use gtk::{
    self,
    glib::{self, subclass::types::ObjectSubclassIsExt},
    prelude::*,
};
use serde::{
    Deserialize, Deserializer,
    de::{self, MapAccess, Visitor},
};
use std::fmt;

use crate::{db::query::Query, utils::ListViewExtra};

#[gobject_props]
pub struct BibleDownload {
    pub name: String,
    pub download_url: String,
    pub active: bool,
    pub already_added: bool,
}

impl Default for BibleDownload {
    fn default() -> Self {
        glib::Object::new()
    }
}

struct BibleDownloadVisitor;

impl<'de> Visitor<'de> for BibleDownloadVisitor {
    type Value = BibleDownload;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("BibleDownload struct")
    }

    fn visit_map<A>(self, mut map: A) -> Result<BibleDownload, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut name = None;
        let mut active = None;
        let mut download_url = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "name" => name = Some(map.next_value()?),
                "download_url" => download_url = Some(map.next_value()?),
                "active" => active = Some(map.next_value().unwrap_or_default()),
                _ => {
                    map.next_value::<serde::de::IgnoredAny>()?;
                }
            }
        }

        let mut name: String = name.ok_or_else(|| de::Error::missing_field("name"))?;
        if let Some(i) = name.rfind('.') {
            name.truncate(i);
        }
        let download_url = download_url.ok_or_else(|| de::Error::missing_field("download_url"))?;
        let active = active.unwrap_or_default();

        Ok(BibleDownload::new(name, download_url, active, false))
    }
}

impl<'de> Deserialize<'de> for BibleDownload {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(BibleDownloadVisitor)
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
        subclass::prelude::*,
    };

    use crate::{
        format_resource, utils::ListViewExtra,
        widgets::search::scriptures::download::download_list_item::TranslationListItem,
    };

    use super::*;

    #[derive(Default, Debug, gtk::CompositeTemplate)]
    #[template(resource = "/com/openworship/app/ui/download_bible_page.ui")]
    pub struct DownloadBiblePage {
        pub(super) installed_translations: RefCell<Vec<String>>,
        #[template_child]
        pub(super) listview: gtk::TemplateChild<gtk::ListView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DownloadBiblePage {
        const NAME: &'static str = "DownloadBiblePage";
        type Type = super::DownloadBiblePage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DownloadBiblePage {
        fn constructed(&self) {
            self.parent_constructed();

            let listview = self.listview.clone();
            let model = gtk::gio::ListStore::new::<BibleDownload>();
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
                    .and_downcast::<BibleDownload>()
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
    impl WidgetImpl for DownloadBiblePage {}
    impl BoxImpl for DownloadBiblePage {}

    impl DownloadBiblePage {
        pub(super) fn register_import_bible(&self, translations: Vec<String>) {
            self.installed_translations.replace(translations.clone());
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

            self.listview.remove_all();
            match download_list_result {
                Ok(download_list) => {
                    for item in download_list {
                        if !item.active() {
                            continue;
                        }

                        let name = item.name();
                        item.set_already_added(translation_map.contains_key(&name));
                        self.listview.append_item(&item);
                    }
                }
                Err(e) => println!("HIT2 err {:?}", e),
            }
        }
    }
}

glib::wrapper! {
pub struct DownloadBiblePage(ObjectSubclass<imp::DownloadBiblePage>)
    @extends  gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for DownloadBiblePage {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl DownloadBiblePage {
    pub fn new(installed_translations: Vec<String>) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().register_import_bible(installed_translations);
        obj
    }

    pub fn load_translations(&self, installed_translations: Vec<String>) {
        self.imp().register_import_bible(installed_translations);
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
            let installed_translations = {
                // NOTE:
                // this ensures the borrow_mut lifetime does not go out
                // since other function calls may borrow same value
                let mut installed_translations = self.imp().installed_translations.borrow_mut();
                installed_translations.clear();
                installed_translations.extend(list);
                installed_translations.clone()
            };

            self.imp().listview.remove_all();
            self.imp()
                .register_import_bible(installed_translations.clone());
            self.emit_reload_translation();
        }
    }
}
