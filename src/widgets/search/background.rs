use gtk::glib;
use gtk::prelude::ObjectExt;

mod signals {
    pub(super) const SEND_PREVIEW_BACKGROUND: &str = "send-preview-background";
}
mod imp {
    use std::{cell::RefCell, sync::OnceLock};

    use gtk::{
        gio::{
            self,
            prelude::{FileExt, ListModelExt, ListModelExtManual},
        },
        glib::{
            self,
            object::{Cast, CastNone},
            subclass::{
                Signal,
                object::{ObjectImpl, ObjectImplExt},
                types::{ObjectSubclass, ObjectSubclassExt},
            },
            types::StaticType,
        },
        prelude::ListItemExt,
        subclass::{
            box_::BoxImpl,
            widget::{
                CompositeTemplateCallbacksClass, CompositeTemplateClass,
                CompositeTemplateInitializingExt, WidgetClassExt, WidgetImpl,
            },
        },
    };

    use crate::{
        app_config::AppConfigDir, dto::background_obj::BackgroundObj,
        services::file_manager::FileManager,
        structs::background_grid_list_item::BackgroundListItem,
        widgets::search::background::signals,
    };

    #[derive(Default, Debug, gtk::CompositeTemplate)]
    #[template(resource = "/com/openworship/app/ui/search_background.ui")]
    pub struct SearchBackground {
        #[template_child]
        search_text: gtk::TemplateChild<gtk::SearchEntry>,
        #[template_child]
        gridview: gtk::TemplateChild<gtk::GridView>,
        #[template_child]
        add_bg_btn: gtk::TemplateChild<gtk::Button>,
        #[template_child]
        remove_bg_btn: gtk::TemplateChild<gtk::Button>,

        //
        image_src_list: RefCell<Vec<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SearchBackground {
        const NAME: &'static str = "SearchBackground";
        type Type = super::SearchBackground;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SearchBackground {
        fn constructed(&self) {
            self.parent_constructed();

            let model = gio::ListStore::new::<BackgroundObj>();
            let model = gtk::SingleSelection::new(Some(model));
            self.gridview.set_model(Some(&model));

            let factory = gtk::SignalListItemFactory::new();
            self.gridview.set_factory(Some(&factory));

            factory.connect_setup(|_, list_item| {
                let li = list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Expected ListItem");
                li.set_child(Some(&BackgroundListItem::default()));
            });
            factory.connect_bind(|_, list_item| {
                let li = list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Expected ListItem");
                let item = li
                    .item()
                    .and_downcast::<BackgroundObj>()
                    .expect("Expected BackgroundObj");
                let child = li
                    .child()
                    .and_downcast::<BackgroundListItem>()
                    .expect("Expected BackgroundListItem");

                child.set_label(&item.title());
                child.set_picture_src(&item.src());
            });

            let list = Self::load_backgrounds();
            self.append_background(list, false);
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();

            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder(signals::SEND_PREVIEW_BACKGROUND)
                        .param_types([String::static_type()])
                        .build(),
                ]
            })
        }
    }

    impl WidgetImpl for SearchBackground {}
    impl BoxImpl for SearchBackground {}

    #[gtk::template_callbacks]
    impl SearchBackground {
        #[template_callback]
        fn handle_search_activate(&self) {}

        #[template_callback]
        fn handle_preview_background(&self, position: u32, _: &gtk::GridView) {
            let list = self.image_src_list.borrow();

            let Some(path) = list.get(position as usize) else {
                return;
            };

            self.obj().emit_send_preview_background(path.clone());
        }

        #[template_callback]
        fn handle_add_background(&self, _: &gtk::Button) {
            let file_filter = gtk::FileFilter::new();
            file_filter.add_mime_type("image/*");
            file_filter.set_name(Some("Images"));

            let mut list = glib::List::<gtk::FileFilter>::new();
            list.push_back(file_filter);
            let files = FileManager::open_files("Import background", "Import", &mut list);

            // ::<gtk::gio::File>
            let bg = files
                .iter::<gtk::gio::File>()
                .flatten()
                .filter_map(|file| {
                    let Some(path) = file.path() else {
                        return None;
                    };
                    let filename = path.display().to_string();
                    Some(filename)
                })
                .collect::<Vec<_>>();

            self.append_background(bg, true);
        }

        #[template_callback]
        fn handle_remove_background(&self, _: &gtk::Button) {
            let gridview = self.gridview.clone();

            let s_model = match gridview.model() {
                Some(model) => model,
                None => return,
            };

            let ss_model = match s_model.downcast_ref::<gtk::SingleSelection>() {
                Some(model) => model,
                None => return,
            };

            let selected_pos = ss_model.selected();
            self.remove_background(vec![selected_pos]);
        }
    }
    impl SearchBackground {
        fn load_backgrounds() -> Vec<String> {
            let mut path_list = Vec::new();
            let dir = match AppConfigDir::dir_path(AppConfigDir::Backgrounds).read_dir() {
                Ok(d) => d,
                Err(e) => {
                    println!(
                        "ERROR: could not read {:?} = {:?}",
                        AppConfigDir::Backgrounds,
                        e
                    );
                    return path_list;
                }
            };

            for entry in dir {
                let entry = match entry {
                    Ok(f) => f,
                    Err(_) => continue,
                };

                if let Ok(entry) = entry.metadata() {
                    if !entry.is_file() {
                        continue;
                    }
                }

                path_list.push(entry.path().display().to_string());
            }

            path_list
        }
        fn append_background(&self, bg: Vec<String>, should_link: bool) {
            let view = self.gridview.clone();

            for path in bg {
                let mut path = path.clone();
                if should_link {
                    let file = gio::File::for_path(&path);
                    let Some(link_path) =
                        FileManager::file_to_link(&file, AppConfigDir::Backgrounds)
                    else {
                        continue;
                    };

                    path = link_path
                }

                let Some(store) = view
                    .model()
                    .and_then(|v| v.downcast::<gtk::SingleSelection>().ok())
                    .and_then(|v| v.model().and_downcast::<gio::ListStore>())
                else {
                    return;
                };

                store.append(&BackgroundObj::new(path.clone(), None));
                self.image_src_list.borrow_mut().push(path);
            }
        }
        fn remove_background(&self, position_list: Vec<u32>) {
            for index in position_list.clone() {
                let gridview = self.gridview.clone();
                let Some(store) = gridview
                    .model()
                    .and_then(|v| v.downcast::<gtk::SingleSelection>().ok())
                    .and_then(|v| v.model().and_downcast::<gio::ListStore>())
                else {
                    return;
                };

                let item_to_remove = store.item(index);
                if let Some(item) = item_to_remove.and_downcast::<BackgroundObj>() {
                    store.remove(index);
                    let _ = std::fs::remove_file(&item.src());
                }
            }
        }
    }
}

glib::wrapper! {
    pub struct SearchBackground(ObjectSubclass<imp::SearchBackground>)
    @extends  gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for SearchBackground {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl SearchBackground {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn connect_send_preview_background<F: Fn(&Self, String) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            signals::SEND_PREVIEW_BACKGROUND,
            false,
            glib::closure_local!(|obj: &Self, bg: String| { f(obj, bg) }),
        )
    }
    fn emit_send_preview_background(&self, bg: String) {
        self.emit_by_name(signals::SEND_PREVIEW_BACKGROUND, &[&bg])
    }
}
