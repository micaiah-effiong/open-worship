use std::usize;

use gtk::{
    glib::{self, object::CastNone, subclass::types::ObjectSubclassIsExt},
    prelude::{GtkWindowExt, WidgetExt},
};

use crate::{
    services::message_alert_manager::MessageAlertManager,
    utils::WidgetChildrenExt,
    widgets::{
        canvas::{canvas_item::CanvasItem, serialise::SlideManagerData},
        message_alert_wrapper::MessageAlertWapper,
        stream_wrapper::WidgetMediaStream,
    },
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

    use crate::{
        app_config::AppConfig, services::slide_manager::SlideManager,
        widgets::extended_screen::build_source_box,
    };

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

            #[cfg(not(debug_assertions))]
            {
                use gtk::prelude::WidgetExt;

                obj.set_decorated(false);
                let c = gtk::gdk::Cursor::from_name("none", None);
                obj.set_cursor(c.as_ref());
                obj.set_resizable(false);
            }

            let sm = self.slide_manager.borrow();
            sm.set_animation(true);
            sm.show_end_presentation_slide();

            let frame = gtk::AspectFrame::new(0.5, 0.5, AppConfig::aspect_ratio(), false);
            frame.set_child(Some(&sm.slideshow()));

            let binding = frame.into();
            let root = build_source_box(&binding);
            obj.set_child(Some(root));
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
        let obj: Self = glib::Object::new();
        obj.imp().slide_manager.borrow().set_log(true);

        obj
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
            // let d = sm.serialise();
            // let mut dd = data.clone();
            // dd.current_slide = d.current_slide.clone();
            // if d == dd {
            //     self.set_pos(data.current_slide);
            //     println!("exisit");
            //     return;
            // }
        };

        sm.set_title(data.title.clone());
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

        // sm.reset();
        // sm.load_data(data.clone());
        sm.reload_data(data.clone());

        for slide in &sm.slides() {
            slide.set_presentation_mode(true);
        }

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

            for text in c.widget().get_children::<CanvasItem>() {
                text.set_visible(!clear);
            }
        }
    }

    pub fn set_alert_manager(&self, alert_manager: &MessageAlertManager) {
        let imp = self.imp();
        let sm = imp.slide_manager.borrow();

        let binding = self.child();
        let Some(a_frame) = binding.and_downcast_ref::<gtk::AspectFrame>() else {
            return;
        };
        a_frame.set_child(None::<&gtk::Widget>);
        let alert_wrapper = MessageAlertWapper::new(&sm, alert_manager);
        a_frame.set_child(Some(&alert_wrapper));
    }
}

pub fn build_source_box(source_box: &gtk::Widget) -> &gtk::Widget {
    let stream = WidgetMediaStream::new(source_box);

    let video = gtk::Video::builder().hexpand(true).vexpand(true).build();
    video.set_media_stream(Some(&stream));

    let w = gtk::Window::new();
    w.set_child(Some(&video));
    w.present();

    source_box.into()
}
