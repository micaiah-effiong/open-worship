use std::usize;

use gtk::{
    glib::{self, subclass::types::ObjectSubclassIsExt},
    prelude::WidgetExt,
};

use crate::{
    utils::WidgetChildrenExt,
    widgets::canvas::{serialise::SlideManagerData, text_item::TextItem},
};

mod signals {}
mod imp {
    use std::{
        cell::{Cell, RefCell},
        sync::OnceLock,
    };

    use gtk::{
        glib::{
            self, Properties,
            subclass::{
                Signal,
                object::{ObjectImpl, ObjectImplExt},
                types::{ObjectSubclass, ObjectSubclassExt},
            },
        },
        prelude::GtkWindowExt,
        subclass::{widget::WidgetImpl, window::WindowImpl},
    };

    use crate::{app_config::AppConfig, services::slide_manager::SlideManager};

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::ExtendedScreen)]
    pub struct ExtendedScreen {
        pub(super) slide_manager: RefCell<SlideManager>,
        pub clear: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ExtendedScreen {
        const NAME: &'static str = "ExtendedScreen";
        type Type = super::ExtendedScreen;
        type ParentType = gtk::Window;
    }

    impl ObjectImpl for ExtendedScreen {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            let size = 300;
            obj.set_default_size(size, (size as f32 / AppConfig::aspect_ratio()) as i32);
            // obj.set_decorated(false);
            obj.set_resizable(false);
            // let c = gtk::gdk::Cursor::from_name("none", None);
            // obj.set_cursor(c.as_ref());

            let sm = self.slide_manager.borrow();
            sm.set_animation(true);
            sm.show_end_presentation_slide();

            let frame = gtk::AspectFrame::new(0.5, 0.5, AppConfig::aspect_ratio(), false);
            frame.set_child(Some(&sm.slideshow()));

            obj.set_child(Some(&frame));
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNAL: OnceLock<Vec<Signal>> = OnceLock::new();

            SIGNAL.get_or_init(|| vec![])
        }
    }
    impl WidgetImpl for ExtendedScreen {}
    impl WindowImpl for ExtendedScreen {}

    impl ExtendedScreen {}
}

glib::wrapper! {
pub struct ExtendedScreen(ObjectSubclass<imp::ExtendedScreen>)
    @extends gtk::Window, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,gtk::Native,gtk::Root, gtk::ShortcutManager;
}

impl Default for ExtendedScreen {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl ExtendedScreen {
    pub fn new() -> Self {
        let obj: Self = glib::Object::new();

        obj
    }

    pub fn load_data(&self, data: &SlideManagerData) {
        let imp = self.imp();
        let sm = imp.slide_manager.borrow();

        {
            if let Some(mut end_slide) = data.slides.first().cloned() {
                end_slide.items.clear();
                end_slide.canvas_data.background_pattern = None;
                let e = sm.imp().end_presentation_slide.borrow();
                e.imp().save_data.replace(Some(end_slide.clone()));

                if let Some(canvas) = e.canvas() {
                    canvas.imp().sava_data.replace(Some(end_slide.canvas_data));
                    canvas.imp().load_data();
                    canvas.style();
                }
                e.load_slide();
            }
        }

        sm.reset();
        sm.load_data(data.clone());

        for slide in &sm.slides() {
            slide.load_slide();
            slide.set_presentation_mode(true);
        }
        sm.set_current_slide(sm.slides().get(data.current_slide as usize));

        self.clear_display(imp.clear.get());
    }

    pub fn set_pos(&self, position: u32) {
        let imp = self.imp();
        let sm = imp.slide_manager.borrow();

        if let Some(slide) = sm.slides().get(position as usize) {
            sm.set_current_slide(Some(slide));
        }
    }

    pub fn update_background(&self, img: String) {
        let imp = self.imp();
        let sm = imp.slide_manager.borrow();

        for slide in sm.slides() {
            let Some(canvas) = slide.canvas() else {
                continue;
            };
            canvas.set_background_pattern(img.clone());
            canvas.style();
        }
    }

    pub fn clear_display(&self, clear: bool) {
        let imp = self.imp();
        let sm = imp.slide_manager.borrow();

        imp.clear.set(clear);
        for slide in sm.slides() {
            let Some(c) = slide.canvas() else {
                return;
            };

            for text in c.widget().get_children::<TextItem>() {
                text.set_visible(!clear);
            }
        }
    }
}
