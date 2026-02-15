mod background;
mod scriptures;
pub mod songs;

use gtk::{glib, prelude::*};

use crate::widgets::canvas::serialise::SlideManagerData;

mod signals {
    pub(super) const PREVIEW_BACKGROUND: &str = "preview-background";
    pub(super) const PREVIEW_SLIDES: &str = "preview-slides";
    pub(super) const ADD_TO_SCHEDULE: &str = "add-to-schedule";
}

mod imp {
    use std::{cell::RefCell, sync::OnceLock};

    use gtk::{
        glib::{
            self,
            subclass::{
                Signal,
                object::ObjectImpl,
                types::{ObjectSubclass, ObjectSubclassExt},
            },
            types::{StaticType, StaticTypeExt},
        },
        subclass::{
            box_::BoxImpl,
            widget::{
                CompositeTemplateCallbacksClass, CompositeTemplateClass,
                CompositeTemplateInitializingExt, WidgetImpl,
            },
        },
    };

    use crate::widgets::{
        canvas::serialise::SlideManagerData,
        search::{
            background::SearchBackground, scriptures::SearchScripture, signals, songs::SearchSong,
        },
    };

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/openworship/app/ui/search.ui")]
    pub struct SearchActivityViewer {
        background_image: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SearchActivityViewer {
        const NAME: &'static str = "SearchActivityViewer";
        type Type = super::SearchActivityViewer;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            SearchSong::ensure_type();
            SearchScripture::ensure_type();
            SearchBackground::ensure_type();

            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SearchActivityViewer {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();

            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder(signals::ADD_TO_SCHEDULE)
                        .param_types([SlideManagerData::static_type()])
                        .build(),
                    Signal::builder(signals::PREVIEW_SLIDES)
                        .param_types([SlideManagerData::static_type()])
                        .build(),
                    Signal::builder(signals::PREVIEW_BACKGROUND)
                        .param_types([String::static_type()])
                        .build(),
                ]
            })
        }
    }
    impl WidgetImpl for SearchActivityViewer {}
    impl BoxImpl for SearchActivityViewer {}

    #[gtk::template_callbacks]
    impl SearchActivityViewer {
        #[template_callback]
        fn handle_preview_song(&self, data: &SlideManagerData, _: SearchSong) {
            self.preview_slides(data);
        }
        #[template_callback]
        fn handle_schedule_song(&self, data: &SlideManagerData, _: SearchSong) {
            self.obj().emit_add_to_schedule(data);
        }
        #[template_callback]
        fn handle_schedule_scripture(&self, data: &SlideManagerData, _: SearchScripture) {
            self.obj().emit_add_to_schedule(data);
        }
        #[template_callback]
        fn handle_preview_scripture(&self, data: &SlideManagerData, _: SearchScripture) {
            self.preview_slides(data);
        }
        #[template_callback]
        fn handle_preview_background(&self, data: String, _: SearchBackground) {
            self.obj().emit_preview_background(data);
        }
    }

    impl SearchActivityViewer {
        fn preview_slides(&self, data: &SlideManagerData) {
            let mut data = data.clone();
            data.slides.iter_mut().for_each(|v| {
                if v.canvas_data.background_pattern.is_none() {
                    v.canvas_data.background_pattern = self.background_image.borrow().clone();
                }
            });
            self.obj().emit_preview_slides(&data);
        }
    }
}

glib::wrapper! {
    pub struct SearchActivityViewer(ObjectSubclass<imp::SearchActivityViewer>)
    @extends  gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for SearchActivityViewer {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl SearchActivityViewer {
    pub fn new() -> Self {
        glib::Object::new()
    }

    fn emit_add_to_schedule(&self, slides: &SlideManagerData) {
        self.emit_by_name(signals::ADD_TO_SCHEDULE, &[slides])
    }
    pub fn connect_add_to_schedule<F: Fn(&Self, &SlideManagerData) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            signals::ADD_TO_SCHEDULE,
            false,
            glib::closure_local!(|obj: &Self, data: &SlideManagerData| f(obj, data)),
        )
    }

    fn emit_preview_slides(&self, slides: &SlideManagerData) {
        self.emit_by_name(signals::PREVIEW_SLIDES, &[slides])
    }
    pub fn connect_preview_slides<F: Fn(&Self, &SlideManagerData) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            signals::PREVIEW_SLIDES,
            false,
            glib::closure_local!(|obj: &Self, data: &SlideManagerData| f(obj, data)),
        )
    }

    fn emit_preview_background(&self, bg: String) {
        self.emit_by_name(signals::PREVIEW_BACKGROUND, &[&bg])
    }
    pub fn connect_preview_background<F: Fn(&Self, String) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            signals::PREVIEW_BACKGROUND,
            false,
            glib::closure_local!(|obj: &Self, data: String| f(obj, data)),
        )
    }
}
