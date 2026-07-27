use gtk::gio;
use gtk::glib;
use gtk::glib::object::CastNone;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::BoxExt;
use gtk::prelude::GtkWindowExt;
use gtk::prelude::WidgetExt;

use crate::widgets::canvas::serialise::SlideManagerData;
use crate::widgets::editor::{Editor, EditorType};

mod imp {
    use std::{cell::RefCell, sync::OnceLock};

    use adw::subclass::prelude::AdwApplicationWindowImpl;
    use gtk::{
        gdk::prelude::{DisplayExt, MonitorExt},
        gio::prelude::{ApplicationExt, ListModelExt},
        glib::{
            self, Properties,
            object::{Cast, CastNone},
            subclass::{
                Signal,
                object::{ObjectImpl, ObjectImplExt},
                types::ObjectSubclass,
            },
            types::StaticTypeExt,
        },
        prelude::{GtkWindowExt, ObjectExt, PopoverExt, ToggleButtonExt, WidgetExt},
        subclass::{
            prelude::{ApplicationWindowImpl, DerivedObjectProperties},
            widget::{
                CompositeTemplateCallbacksClass, CompositeTemplateClass,
                CompositeTemplateInitializingExt, WidgetClassExt, WidgetImpl,
            },
            window::{WindowImpl, WindowImplExt},
        },
    };

    use crate::{
        services::message_alert_manager::MessageAlertManager,
        widgets::{
            activity_viewer::ActivityViewer, canvas::serialise::SlideManagerData, editor::Editor,
            extended_screen::ExtendedScreen, message_alert_viewer::MessageAlertViewer,
            schedule_activity_viewer::ScheduleActivityViewer, search::SearchActivityViewer,
        },
    };

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[cfg_attr(
        target_os = "macos",
        template(resource = "/com/openworship/app/ui/app_window_macos.ui")
    )]
    #[cfg_attr(
        not(target_os = "macos"),
        template(resource = "/com/openworship/app/ui/app_window.ui")
    )]
    #[properties(wrapper_type=super::MainApplicationWindow)]
    pub struct MainApplicationWindow {
        #[template_child]
        #[property(get)]
        window_box: gtk::TemplateChild<gtk::Box>,
        #[property(get)]
        #[template_child]
        schedule_viewer: gtk::TemplateChild<ScheduleActivityViewer>,
        #[template_child]
        search_viewer: gtk::TemplateChild<SearchActivityViewer>,
        #[property(get)]
        #[template_child]
        preview_viewer: gtk::TemplateChild<ActivityViewer>,
        #[property(get)]
        #[template_child]
        live_viewer: gtk::TemplateChild<ActivityViewer>,
        #[template_child]
        alert_popover: gtk::TemplateChild<gtk::Popover>,
        #[template_child]
        alert_btn: gtk::TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub(super) stack: gtk::TemplateChild<gtk::Stack>,
        #[template_child]
        pub(super) editor_view: gtk::TemplateChild<gtk::Box>,

        //
        #[property(get)]
        extended_screen: RefCell<ExtendedScreen>,

        //
        slide_change_handler_id: RefCell<Option<glib::SignalHandlerId>>,

        alert_manager: RefCell<MessageAlertManager>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MainApplicationWindow {
        const NAME: &'static str = "MainApplicationWindow";
        type Type = super::MainApplicationWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            SearchActivityViewer::ensure_type();
            ScheduleActivityViewer::ensure_type();
            ActivityViewer::ensure_type();
            MessageAlertViewer::ensure_type();
            Editor::ensure_type();

            klass.bind_template();
            klass.bind_template_callbacks();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MainApplicationWindow {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();

            SIGNALS.get_or_init(|| Vec::from([]))
        }

        fn constructed(&self) {
            self.parent_constructed();

            self.extended_screen
                .borrow()
                .set_alert_manager(&self.alert_manager.borrow().clone());
            let id = self.live_viewer.connect_slide_change(glib::clone!(
                #[weak(rename_to=imp)]
                self,
                move |_, position| {
                    imp.extended_screen.borrow().set_pos(position);
                }
            ));
            self.slide_change_handler_id.replace(Some(id));

            // set popover viewer
            self.alert_popover
                .set_child(Some(&self.alert_manager.borrow().viewer()));
            if let Some(btn) = self
                .alert_btn
                .first_child()
                .and_downcast_ref::<gtk::ToggleButton>()
            {
                btn.add_css_class("flat");
            }
        }
    }
    impl WidgetImpl for MainApplicationWindow {}
    impl WindowImpl for MainApplicationWindow {
        fn close_request(&self) -> glib::Propagation {
            println!("application_window.rs close");
            self.parent_close_request();
            glib::Propagation::Proceed
        }
    }
    impl ApplicationWindowImpl for MainApplicationWindow {}
    impl AdwApplicationWindowImpl for MainApplicationWindow {}

    #[gtk::template_callbacks]
    impl MainApplicationWindow {
        #[template_callback]
        fn handle_close_request(&self, w: &gtk::ApplicationWindow) -> glib::Propagation {
            if let Some(name) = self.stack.visible_child_name()
                && name == "editor"
            {
                if let Some(editor) = self.editor_view.first_child().and_downcast::<Editor>() {
                    editor.cancel_reponse();
                    return glib::Propagation::Stop;
                };
            };

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
        fn handle_show_black(&self, _: &gtk::Button) {
            glib::g_warning!("application_window", "TODO: Show black");
        }

        #[template_callback]
        fn handle_show_logo(&self, _: &gtk::Button) {
            glib::g_warning!("application_window", "TODO: Show logo");
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
            // block change
            let id = self.slide_change_handler_id.take();
            if let Some(id) = id {
                self.live_viewer.block_signal(&id);
                self.slide_change_handler_id.replace(Some(id));
            }

            self.live_viewer.load_data(data);
            self.extended_screen.borrow().load_data(data);

            // unblock change
            let id = self.slide_change_handler_id.take();
            if let Some(id) = id {
                self.live_viewer.unblock_signal(&id);
                self.slide_change_handler_id.replace(Some(id));
            }
        }

        // #[template_callback]
        // fn handle_live_slide_change(&self, position: u32, _: &ActivityViewer) {
        //     self.extended_screen.borrow().set_pos(position);
        // }
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
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
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

impl MainApplicationWindow {
    pub fn open_editor(
        &self,
        editor_type: Option<EditorType>,
        data: Option<SlideManagerData>,
    ) -> Editor {
        let imp = self.imp();
        let stack = imp.stack.clone();
        if let Some(editor) = stack.child_by_name("editor").and_downcast::<Editor>() {
            stack.remove(&editor);
            editor.unparent();
        };

        let editor = Editor::new(self.clone(), editor_type, data);
        imp.editor_view.append(&editor);
        stack.set_visible_child_name("editor");

        editor
    }

    pub fn close_editor(&self, editor: Editor) {
        let stack = self.imp().stack.clone();
        stack.set_visible_child_name("main");
        self.imp().editor_view.remove(&editor);
        editor.unparent();
    }
}
