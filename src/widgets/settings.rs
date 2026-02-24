use gtk::glib;

mod imp {
    use std::{cell::RefCell, collections::HashMap};

    use gtk::{
        gdk::{
            self,
            prelude::{DisplayExt, MonitorExt},
        },
        gio::prelude::{ListModelExtManual, SettingsExtManual},
        glib::{
            self,
            object::CastNone,
            subclass::{
                object::{ObjectImpl, ObjectImplExt},
                types::{ObjectSubclass, ObjectSubclassExt, ObjectSubclassIsExt},
            },
            types::StaticTypeExt,
        },
        prelude::GtkWindowExt,
        subclass::{
            widget::{
                CompositeTemplateClass, CompositeTemplateInitializingExt, WidgetClassExt,
                WidgetImpl,
            },
            window::WindowImpl,
        },
    };

    use crate::{
        application::OwApplication,
        services::settings::ApplicationSettings,
        widgets::canvas::{canvas::Canvas, serialise::CanvasData},
    };

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/openworship/app/ui/settings_window.ui")]
    pub struct SettingsWindow {
        #[template_child]
        sidebar: gtk::TemplateChild<gtk::StackSidebar>,
        #[template_child]
        stack: gtk::TemplateChild<gtk::Stack>,
        #[template_child]
        output_view: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        monitor_dropdowon: gtk::TemplateChild<gtk::DropDown>,

        #[template_child]
        monitor_width: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        monitor_height: gtk::TemplateChild<gtk::Label>,

        #[template_child]
        demo_screen: gtk::TemplateChild<Canvas>,
        #[template_child]
        screen_aspect_frame: gtk::TemplateChild<gtk::AspectFrame>,

        // scripture
        #[template_child]
        show_reference: gtk::TemplateChild<gtk::CheckButton>,
        #[template_child]
        show_verse_number: gtk::TemplateChild<gtk::CheckButton>,
        #[template_child]
        show_passage: gtk::TemplateChild<gtk::CheckButton>,
        #[template_child]
        show_only_reference: gtk::TemplateChild<gtk::CheckButton>,
        #[template_child]
        break_new_verse: gtk::TemplateChild<gtk::CheckButton>,

        monitor_map: RefCell<HashMap<String, gdk::Monitor>>,
    }
    impl SettingsWindow {
        fn bind_settings_values(&self) {
            let settings = ApplicationSettings::get_instance();
            let show_reference = self.show_reference.clone();
            settings
                .bind("show-reference", &show_reference, "active")
                .build();

            let show_verse_number = self.show_verse_number.clone();
            settings
                .bind("show-verse-number", &show_verse_number, "active")
                .build();

            let show_passage = self.show_passage.clone();
            settings
                .bind("show-passage", &show_passage, "active")
                .build();

            let show_only_reference = self.show_only_reference.clone();
            settings
                .bind("show-only-reference", &show_only_reference, "active")
                .build();

            let break_new_verse = self.break_new_verse.clone();
            settings
                .bind("break-new-verse", &break_new_verse, "active")
                .build();
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SettingsWindow {
        const NAME: &'static str = "SettingsWindow";
        type Type = super::SettingsWindow;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            Canvas::ensure_type();
            klass.bind_template();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SettingsWindow {
        fn constructed(&self) {
            self.parent_constructed();
            self.bind_settings_values();

            self.sidebar.set_stack(&self.stack);

            self.demo_screen
                .imp()
                .sava_data
                .replace(Some(CanvasData::default()));
            self.demo_screen.imp().load_data();
            self.demo_screen.style();

            if let Some(display) = gtk::gdk::Display::default()
                && let Some(dropdown_model) = self
                    .monitor_dropdowon
                    .model()
                    .and_downcast::<gtk::StringList>()
            {
                let mut mon_hash = self.monitor_map.borrow_mut();
                let mon_list = display.monitors();

                for monitor in mon_list.iter::<gdk::Monitor>() {
                    let Ok(monitor) = monitor else {
                        return;
                    };

                    if let Some(name) = monitor.model() {
                        let name = name.to_string();
                        dropdown_model.append(&name);
                        mon_hash.insert(name.clone(), monitor);
                    }
                }
            }

            let fn_connect = glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |dropdown: &gtk::DropDown| {
                    let Some(item) = dropdown.selected_item().and_downcast::<gtk::StringObject>()
                    else {
                        return;
                    };
                    let item: String = item.into();

                    let monitor_map = imp.monitor_map.borrow();
                    let Some(monitor) = monitor_map.get(&item) else {
                        imp.monitor_width.set_label("0");
                        imp.monitor_height.set_label("0");
                        imp.screen_aspect_frame.set_ratio(1.0);
                        return;
                    };
                    let geo = monitor.geometry();
                    imp.monitor_width.set_label(&geo.width().to_string());
                    imp.monitor_height.set_label(&geo.height().to_string());

                    let ratio = geo.width() as f32 / geo.height() as f32;
                    imp.screen_aspect_frame.set_ratio(ratio);

                    let Some(app) = imp.obj().application().and_downcast::<OwApplication>() else {
                        return;
                    };

                    let ext_screen = app.main_window().extended_screen();
                    ext_screen.fullscreen_on_monitor(monitor);
                }
            );

            fn_connect(&self.monitor_dropdowon.clone());
            self.monitor_dropdowon
                .connect_selected_item_notify(fn_connect);
        }
    }

    impl WidgetImpl for SettingsWindow {}
    impl WindowImpl for SettingsWindow {}
}

glib::wrapper! {
pub struct SettingsWindow(ObjectSubclass<imp::SettingsWindow>)
    @extends gtk::Window, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,gtk::Native,gtk::Root, gtk::ShortcutManager;
}

impl Default for SettingsWindow {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl SettingsWindow {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
