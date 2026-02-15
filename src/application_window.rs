use gtk::gio;
use gtk::glib;
use gtk::prelude::GtkWindowExt;

mod imp {
    use std::cell::RefCell;

    use gtk::{
        gdk::prelude::{DisplayExt, MonitorExt},
        gio::prelude::{ApplicationExt, ListModelExt},
        glib::{
            self, Properties,
            object::Cast,
            subclass::{
                object::{ObjectImpl, ObjectImplExt},
                types::ObjectSubclass,
            },
            types::StaticTypeExt,
        },
        prelude::{GtkWindowExt, ObjectExt, ToggleButtonExt, WidgetExt},
        subclass::{
            prelude::{ApplicationWindowImpl, DerivedObjectProperties},
            widget::{
                CompositeTemplateCallbacksClass, CompositeTemplateClass,
                CompositeTemplateInitializingExt, WidgetClassExt, WidgetImpl,
            },
            window::WindowImpl,
        },
    };

    use crate::widgets::{
        activity_viewer::ActivityViewer, canvas::serialise::SlideManagerData,
        extended_screen::ExtendedScreen, schedule_activity_viewer::ScheduleActivityViewer,
        search::SearchActivityViewer,
    };

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/com/openworship/app/ui/app_window.ui")]
    #[properties(wrapper_type=super::MainApplicationWindow)]
    pub struct MainApplicationWindow {
        #[template_child]
        #[property(get)]
        window_box: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        schedule_viewer: gtk::TemplateChild<ScheduleActivityViewer>,
        #[template_child]
        search_viewer: gtk::TemplateChild<SearchActivityViewer>,
        #[template_child]
        preview_viewer: gtk::TemplateChild<ActivityViewer>,
        #[template_child]
        live_viewer: gtk::TemplateChild<ActivityViewer>,

        #[property(get)]
        extended_screen: RefCell<ExtendedScreen>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MainApplicationWindow {
        const NAME: &'static str = "MainApplicationWindow";
        type Type = super::MainApplicationWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            SearchActivityViewer::ensure_type();
            ScheduleActivityViewer::ensure_type();
            ActivityViewer::ensure_type();

            klass.bind_template();
            klass.bind_template_callbacks();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MainApplicationWindow {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }
    impl WidgetImpl for MainApplicationWindow {}
    impl WindowImpl for MainApplicationWindow {}
    impl ApplicationWindowImpl for MainApplicationWindow {}

    #[gtk::template_callbacks]
    impl MainApplicationWindow {
        #[template_callback]
        fn handle_close_request(&self, w: &gtk::ApplicationWindow) -> glib::Propagation {
            if let Some(app) = w.application() {
                app.quit();
            };

            glib::Propagation::Stop
        }

        #[template_callback]
        fn handle_go_live(&self, _: &gtk::Button) {
            self.preview_viewer.emit_activate_slide();
        }

        #[template_callback]
        fn handle_clear_live(&self, btn: &gtk::ToggleButton) {
            self.live_viewer.clear_display(btn.is_active());
            self.extended_screen.borrow().clear_display(btn.is_active());
        }

        #[template_callback]
        fn handle_toggle_live(&self, btn: &gtk::ToggleButton) {
            self.extended_screen.borrow().set_visible(btn.is_active());
        }

        #[template_callback]
        fn handle_activate_schedule(&self, data: &SlideManagerData, _: &ScheduleActivityViewer) {
            self.preview_viewer.load_data(data);
        }

        #[template_callback]
        fn handle_search_preview_slides(&self, data: &SlideManagerData, _: &SearchActivityViewer) {
            self.preview_viewer.load_data(data);
        }

        #[template_callback]
        fn handle_search_preview_background(&self, img: String, _: &SearchActivityViewer) {
            self.preview_viewer.update_background(img);
        }

        #[template_callback]
        fn handle_search_add_to_schedule(&self, data: &SlideManagerData, _: &SearchActivityViewer) {
            self.schedule_viewer.add_new_item(data);
        }

        #[template_callback]
        fn handle_preview_activate_slide(&self, data: &SlideManagerData, _: &ActivityViewer) {
            self.live_viewer.load_data(data);
            self.extended_screen.borrow().load_data(data);
        }

        #[template_callback]
        fn handle_live_slide_change(&self, position: u32, _: &ActivityViewer) {
            self.extended_screen.borrow().set_pos(position);
        }
    }

    impl MainApplicationWindow {
        fn get_display_geometry() -> Option<gtk::gdk::Rectangle> {
            let display_backend = gtk::gdk::Display::default().expect("no display");

            let x_mon = match display_backend.monitors().item(0) {
                Some(val) => val.downcast::<gtk::gdk::Monitor>(),
                None => return None,
            };

            let geometry = match x_mon {
                Ok(val) => val.geometry(),
                Err(err) => {
                    println!("Error {:?}", err);
                    return None;
                }
            };

            Some(geometry)
        }
    }
}

glib::wrapper! {
    pub struct MainApplicationWindow(ObjectSubclass<imp::MainApplicationWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Default for MainApplicationWindow {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl MainApplicationWindow {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn show_all(&self) {
        self.present();
        self.extended_screen().present();
    }
}
