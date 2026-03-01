use gtk::{
    gio::{
        self,
        prelude::{ActionMapExtManual, ApplicationExt},
    },
    glib::{self, object::Cast},
    prelude::{GtkApplicationExt, GtkWindowExt},
};

use crate::{
    accels, app_config,
    widgets::{search::songs::edit_modal::SongEditWindow, settings_window::SettingsWindow},
};

mod imp {
    use std::cell::RefCell;

    use gtk::{
        gdk::{self, prelude::DisplayExt},
        gio::{
            prelude::{ListModelExt, ListModelExtManual},
            subclass::prelude::{ApplicationImpl, ApplicationImplExt},
        },
        glib::{
            Properties,
            subclass::{
                object::ObjectImpl,
                types::{ObjectSubclass, ObjectSubclassExt},
            },
        },
        prelude::{GtkWindowExt, ObjectExt, WidgetExt},
        subclass::prelude::{DerivedObjectProperties, GtkApplicationImpl},
    };

    use crate::{application_window::MainApplicationWindow, format_resource};

    use super::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::OwApplication)]
    pub struct OwApplication {
        #[property(get)]
        main_window: RefCell<MainApplicationWindow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for OwApplication {
        const NAME: &'static str = "OwApplication";
        type Type = super::OwApplication;
        type ParentType = gtk::Application;
    }

    #[glib::derived_properties]
    impl ObjectImpl for OwApplication {}

    impl ApplicationImpl for OwApplication {
        fn activate(&self) {
            self.parent_activate();

            let obj = self.obj();

            let main_window = obj.main_window();
            let extended_screen = main_window.extended_screen();
            obj.add_window(&main_window);
            obj.add_window(&extended_screen);

            self.add_app_menu(Some(&main_window.window_box()));

            main_window.show_all();

            let monitors = WidgetExt::display(&extended_screen).monitors();
            if monitors.n_items() > 1
                && let Some(last_monitor) =
                    monitors.iter::<gdk::Monitor>().last().and_then(|v| v.ok())
            {
                extended_screen.fullscreen_on_monitor(&last_monitor);
            }
        }

        fn startup(&self) {
            self.parent_startup();

            gtk::Window::set_default_icon_name(app_config::APP_ID);
            gtk::glib::set_application_name("Openworship");

            let obj = self.obj();

            obj.setup_gactions();
            obj.setup_accels();
        }
    }

    impl GtkApplicationImpl for OwApplication {}

    impl OwApplication {
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
            self.obj().set_menubar(Some(&Self::build_app_menu()));
        }

        fn build_app_menu() -> gtk::gio::MenuModel {
            let menu_builder =
                gtk::Builder::from_resource(format_resource!("ui", "app_menubar.ui"));

            let obj: gio::Menu = menu_builder
                .object("default-menu")
                .expect("App-menu not found");

            obj.into()
        }
    }
}

glib::wrapper! {
    pub struct OwApplication(ObjectSubclass<imp::OwApplication>)
        @extends gio::Application, gtk::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl Default for OwApplication {
    fn default() -> Self {
        glib::Object::builder()
            .property("application-id", app_config::APP_ID)
            .property("resource-base-path", app_config::RESOURCE_PATH)
            .build()
    }
}

impl OwApplication {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the global instance of `Application`.
    ///
    /// # Panics
    ///
    /// Panics if the app is not running or if this is called on a non-main thread.
    pub fn get() -> Self {
        debug_assert!(
            gtk::is_initialized_main_thread(),
            "application must only be accessed in the main thread"
        );

        gio::Application::default().unwrap().downcast().unwrap()
    }

    // pub fn run(&self) -> glib::ExitCode {
    //     // tracing::info!("Openworship ({})", APP_ID);
    //     // tracing::info!("Version: {} ({})", VERSION, PROFILE);
    //     // tracing::info!("Datadir: {}", PKGDATADIR);
    //
    //     let code = ApplicationExtManual::run(self);
    //     println!("CODE {:?}", code);
    //
    //     code
    // }

    fn setup_gactions(&self) {
        // let launch_uri_action = gio::ActionEntry::builder("launch-uri")
        //     .parameter_type(Some(&String::static_variant_type()))
        //     .activate(|obj: &Self, _, param| {
        //         let uri = param.unwrap().get::<String>().unwrap();
        //         glib::spawn_future_local(clone!(
        //             #[strong]
        //             obj,
        //             async move {
        //                 if let Err(err) = gtk::FileLauncher::new(Some(&gio::File::for_uri(&uri)))
        //                     .launch_future(obj.active_window().as_ref())
        //                     .await
        //                 {
        //                     // tracing::error!("Failed to launch uri `{}`: {:?}", uri, err);
        //                 }
        //             }
        //         ));
        //     })
        //     .build();
        // let show_in_files_action = gio::ActionEntry::builder("show-in-files")
        //     .parameter_type(Some(&String::static_variant_type()))
        //     .activate(|obj: &Self, _, param| {
        //         let uri = param.unwrap().get::<String>().unwrap();
        //         glib::spawn_future_local(clone!(
        //             #[strong]
        //             obj,
        //             async move {
        //                 if let Err(err) = gtk::FileLauncher::new(Some(&gio::File::for_uri(&uri)))
        //                     .open_containing_folder_future(obj.active_window().as_ref())
        //                     .await
        //                 {
        //                     // tracing::warn!("Failed to show `{}` in files: {:?}", uri, err);
        //                 }
        //             }
        //         ));
        //     })
        //     .build();

        // let show_about_action = gio::ActionEntry::builder("show-about")
        //     .activate(|obj: &Self, _, _| {
        //         about::present_dialog(&obj.window());
        //     })
        //     .build();

        let quit_action = gio::ActionEntry::builder("quit")
            .activate(|obj: &Self, _, _| obj.quit())
            .build();
        let about_action = gtk::gio::ActionEntry::builder("about")
            .activate(|_, _, _| Self::add_app_about())
            .build();
        let settings_action = gtk::gio::ActionEntry::builder("preferences")
            .activate(|_, _, _| SettingsWindow::new().present())
            .build();

        // FILE
        let add_song_action = gtk::gio::ActionEntry::builder("add-song")
            .activate(|_, _, _| {
                let win = SongEditWindow::new();
                win.show(None);
            })
            .build();
        let open = gio::ActionEntry::builder("open")
            .activate(|_, _, _| println!("Open activated"))
            .build();

        // HELP
        let report_bug = gio::ActionEntry::builder("report-bug")
            .activate(|_, _, _| {
                let create_issue_url =
                    format!("{}/issues/new/choose", env!("CARGO_PKG_REPOSITORY"));
                gtk::UriLauncher::new(&create_issue_url).launch(
                    None::<&gtk::Window>,
                    None::<&gio::Cancellable>,
                    |_| {},
                );
                //
            })
            .build();

        self.add_action_entries([
            // launch_uri_action,
            // show_in_files_action,
            // show_about_action,
            quit_action,
            about_action,
            settings_action,
            // FILE
            open,
            add_song_action,
            // HELP
            report_bug,
        ]);
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

    fn setup_accels(&self) {
        self.set_accels_for_action("app.preferences", &[accels!("comma")]);
        // self.set_accels_for_action("app.quit", &[accels!("q")]);
        // self.set_accels_for_action("window.close", &[accels!("w")]);
        self.set_accels_for_action("window.close", &[accels!("w")]);
        // self.set_accels_for_action("app.preferences", &[accels!(",")]);

        //FILE
        self.set_accels_for_action("app.open", &[accels!("o")]);
        self.set_accels_for_action("win.add-song", &[accels!("i")]);
    }
}
