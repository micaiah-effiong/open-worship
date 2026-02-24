use gtk::gdk;
use gtk::gio::prelude::ApplicationExtManual;
use gtk::gio::prelude::ListModelExt;
use gtk::gio::prelude::ListModelExtManual;
use gtk::glib;

mod imp {
    use super::*;
    use std::cell::RefCell;

    use gtk::gdk::prelude::DisplayExt;
    use gtk::glib::Properties;
    use gtk::prelude::{GtkApplicationExt, WidgetExt};
    use gtk::{
        gio::{
            self,
            prelude::{ActionMapExtManual, ApplicationExt},
        },
        glib::{
            self,
            subclass::{
                object::{ObjectImpl, ObjectImplExt},
                types::{ObjectSubclass, ObjectSubclassExt},
            },
        },
        prelude::{GtkWindowExt, ObjectExt},
        subclass::prelude::DerivedObjectProperties,
    };

    use crate::app_config;
    use crate::application_window::MainApplicationWindow;

    #[derive(Default, Properties)]
    #[properties(wrapper_type=super::MainApplication)]
    pub struct MainApplication {
        #[property(get)]
        main_window: RefCell<MainApplicationWindow>,

        #[property(get)]
        app: RefCell<gtk::Application>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MainApplication {
        const NAME: &'static str = "MainApplication";
        type Type = super::MainApplication;
    }

    #[glib::derived_properties]
    impl ObjectImpl for MainApplication {
        fn constructed(&self) {
            self.parent_constructed();

            let app =
                gtk::Application::new(Some(app_config::APP_ID), gio::ApplicationFlags::empty());
            gtk::glib::set_application_name("Open worship");
            app.set_application_id(Some(app_config::APP_ID));
            app.set_resource_base_path(Some(app_config::RESOURCE_PATH));

            self.app.replace(app.clone());

            app.connect_activate(glib::clone!(
                #[weak(rename_to=imp)]
                self,
                move |app| {
                    let obj = imp.obj();
                    imp.add_app_menu(Some(&obj.main_window().window_box()));
                    imp.add_app_actions();

                    let main_window = obj.main_window();
                    let extended_screen = main_window.extended_screen();
                    app.add_window(&main_window);
                    app.add_window(&extended_screen);
                    main_window.show_all();

                    let monitors = WidgetExt::display(&extended_screen).monitors();
                    if monitors.n_items() > 1
                        && let Some(last_monitor) =
                            monitors.iter::<gdk::Monitor>().last().and_then(|v| v.ok())
                    {
                        extended_screen.fullscreen_on_monitor(&last_monitor);
                    }
                }
            ));
        }
    }
    impl MainApplication {}

    impl MainApplication {
        #[cfg(not(target_os = "macos"))]
        fn add_app_menu(&self, window_box: Option<&gtk::Box>) {
            use gtk::prelude::BoxExt;

            let window_box = match window_box {
                Some(w) => w,
                None => return,
            };

            let menu = Self::build_app_menu();

            let menu_model: gtk::gio::MenuModel = menu.into();
            let menubar = gtk::PopoverMenuBar::from_model(Some(&menu_model));

            window_box.prepend(&menubar);
        }

        #[cfg(target_os = "macos")]
        fn add_app_menu(&self, _window_box: Option<&gtk::Box>) {
            let menu = Self::build_app_menu();
            self.obj().app().set_menubar(Some(&menu));
        }

        fn build_app_menu() -> gtk::gio::Menu {
            let menu = gtk::gdk::gio::Menu::new();

            let file_menu = gtk::gio::Menu::new();
            menu.append_submenu(Some("File"), &file_menu);

            let edit_menu = gtk::gio::Menu::new();
            menu.append_submenu(Some("Edit"), &edit_menu);

            menu
        }

        fn add_app_actions(&self) {
            let quit_action = gtk::gio::ActionEntry::builder("quit")
                .activate(|app: &gtk::Application, _, _| app.quit())
                .build();
            let about_action = gtk::gio::ActionEntry::builder("about")
                .activate(|_, _, _| Self::add_app_about())
                .build();

            self.obj()
                .app()
                .add_action_entries([quit_action, about_action]);
        }

        fn add_app_about() {
            let dialog = gtk::AboutDialog::builder()
                .program_name("About Openworship")
                .version(env!("CARGO_PKG_VERSION"))
                .website("https://github.com/micaiah-effiong/open-worship")
                .license_type(gtk::License::MitX11)
                .authors(["Micah Effiong"])
                .logo_icon_name("openworship-symbolic")
                .build();

            dialog.present();
        }
    }
}

glib::wrapper! {
    pub struct MainApplication(ObjectSubclass<imp::MainApplication>);
}

impl Default for MainApplication {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl MainApplication {
    pub fn run(&self) -> glib::ExitCode {
        self.app().run()
    }
}
