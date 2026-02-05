use std::cell::RefCell;

use crate::config::{self, AppConfig};
use crate::utils::setup_theme_listener;
use crate::widgets::activity_viewer::ActivityViewer;
use crate::widgets::canvas::serialise::SlideManagerData;
use crate::widgets::extended_screen::{self, ExtendedScreen};
use crate::widgets::schedule_activity_viewer::ScheduleActivityViewer;
use crate::{dto, format_resource};

#[cfg(not(target_os = "macos"))]
use gtk::PopoverMenuBar;
use gtk::gdk;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::*;
use relm4::prelude::*;

mod icon_names {
    include!(concat!(env!("OUT_DIR"), "/icon_names.rs"));
}
use crate::widgets::search::{SearchInit, SearchModel, SearchOutput};

#[derive(Debug)]
enum AppInput {
    ScheduleActivityAddNew(SlideManagerData),
    ClearLiveDisplay(bool),
    PreviewGoLive,

    //
    SearchPreviewBackground(String),
    SearchPreviewActivity(SlideManagerData),
}

struct AppModel {
    schedule_viewer: RefCell<ScheduleActivityViewer>,
    search_viewer: relm4::Controller<SearchModel>,
    preview_viewer: RefCell<ActivityViewer>,
    live_viewer: RefCell<ActivityViewer>,
    extended_screen: RefCell<ExtendedScreen>,
}

impl AppModel {
    fn convert_search_response(res: SearchOutput) -> AppInput {
        match res {
            SearchOutput::PreviewBackground(image_src) => {
                AppInput::SearchPreviewBackground(image_src)
            }
            SearchOutput::PreviewSongs(list) => AppInput::SearchPreviewActivity(list),
            SearchOutput::AddToSchedule(list) => AppInput::ScheduleActivityAddNew(list),
            // SearchOutput::PreviewScriptures(list_payload) => todo!(),
            // TODO:
            SearchOutput::PreviewScriptures(list) => AppInput::SearchPreviewActivity(list.into()),
        }
    }
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = Option<()>;
    type Input = AppInput;
    type Output = ();
    // type Root = gtk::Window;
    // type Widgets = AppWidgets;

    view! {
        #[root]
        main_window = gtk::ApplicationWindow{
            // layout box
            #[wrap(Some)]
            #[name="window_box"]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                // header box
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    // set_height_request: 48,
                    // set_margin_all:3 ,
                    set_margin_horizontal: 12,
                    set_margin_vertical: 3,

                    gtk::Box {
                        set_hexpand: true,
                    },

                    gtk::Box {
                        set_spacing: 5,
                        gtk::Button {
                            connect_clicked => AppInput::PreviewGoLive,
                            set_css_classes: &["flat"],
                            gtk::Label {
                                set_label: "Go live"
                            }
                        },
                        gtk::ToggleButton {
                            set_label: "Clear",
                            set_css_classes: &["flat"],
                            connect_toggled [sender]=> move |t| {
                                sender.input(AppInput::ClearLiveDisplay(t.is_active()))
                            },
                        },
                        gtk::ToggleButton {
                            set_label: "Live",
                            set_css_classes: &["flat"],
                            set_active: true,
                            connect_toggled [screen = model.extended_screen.borrow().clone()]=> move |t| {
                                screen.set_visible(t.is_active());
                            },
                        }
                    }
                },
                gtk::Separator{},

                // body box
                gtk::Box {
                    set_margin_horizontal: 12,
                    set_margin_vertical: 3,
                    set_orientation: gtk::Orientation::Vertical,

                    gtk::Box {
                        set_hexpand: true,
                        set_vexpand: true,
                        set_homogeneous: true,
                        set_orientation: gtk::Orientation::Horizontal,

                        // pane1
                        gtk::Paned {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_shrink_start_child: false,
                            set_shrink_end_child: false,

                            #[wrap(Some)]
                            set_start_child = &gtk::Box {
                                set_homogeneous: true,
                                set_orientation: gtk::Orientation::Vertical,
                                set_vexpand: true,
                                set_width_request: MIN_GRID_WIDTH,

                                gtk::Paned {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_shrink_start_child: false,
                                    set_shrink_end_child: false,

                                    // schedule box
                                    set_start_child = Some(&model.schedule_viewer.borrow().clone()),
                                    // set_start_child = Some(model.schedule_activity_viewer.widget()),
                                    // set_start_child = &gtk::Box {
                                    //     set_orientation: gtk::Orientation::Vertical,
                                    //     set_height_request: MIN_GRID_HEIGHT,
                                    //     set_hexpand: true,
                                    //     set_css_classes: &[ "ow-listview"],
                                    //
                                    //     gtk::Label {
                                    //         set_label: "Schedule"
                                    //     }
                                    // }

                                    set_end_child = Some(model.search_viewer.widget()),

                                }

                            },

                            #[wrap(Some)]
                            set_end_child = &gtk::Paned {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_shrink_start_child: false,
                                set_shrink_end_child: false,

                                set_start_child = Some(&model.preview_viewer.borrow().clone()),
                                // #[wrap(Some)]
                                // set_start_child = &gtk::Box {
                                //     set_homogeneous: true,
                                //     set_orientation: gtk::Orientation::Vertical,
                                //     set_vexpand: true,
                                //     set_width_request: MIN_GRID_WIDTH,
                                //
                                //     gtk::Paned {
                                //         set_orientation: gtk::Orientation::Vertical,
                                //         set_shrink_start_child: false,
                                //         set_shrink_end_child: false,
                                //         set_start_child = Some(model.preview_activity_viewer.widget()),
                                //         set_end_child = Some(&model.preview_activity_screen.borrow().clone()),
                                //     }
                                // },

                                set_end_child =Some(&model.live_viewer.borrow().clone()),
                                // #[wrap(Some)]
                                // set_end_child = &gtk::Box {
                                //     set_homogeneous: true,
                                //     set_orientation: gtk::Orientation::Vertical,
                                //     set_vexpand: true,
                                //     set_width_request: MIN_GRID_WIDTH,
                                //
                                //     gtk::Paned {
                                //         set_orientation: gtk::Orientation::Vertical,
                                //         set_shrink_start_child: false,
                                //         set_shrink_end_child: false,
                                //         set_start_child = Some(model.live_activity_viewer.widget()),
                                //         set_end_child = Some(&model.live_activity_screen.borrow().clone()),
                                //     }
                                // }
                            }

                        }

                            // pane2
                    }
                },

                // footer box
                append = &gtk::Box {
                    set_margin_end: 12,
                    set_orientation: gtk::Orientation::Vertical,

                     gtk::Label {
                       set_label: "footer",
                    }
                },

            }
        }
    }

    fn init(
        _init: Self::Init,
        window: Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let search_viewer = SearchModel::builder()
            .launch(SearchInit {})
            .forward(sender.input_sender(), AppModel::convert_search_response);

        let schedule_viewer = ScheduleActivityViewer::new();

        let preview_viewer = ActivityViewer::new("Preview");
        preview_viewer.set_widget_name("pv");

        let live_viewer = ActivityViewer::new("Live");
        live_viewer.set_widget_name("lv");
        let extended_screen = ExtendedScreen::new();

        schedule_viewer.connect_activate({
            let preview_viewer = preview_viewer.clone();
            move |_, data| {
                preview_viewer.load_data(data);
            }
        });
        preview_viewer.connect_activate_slide({
            let live_viewer = live_viewer.clone();
            let extended_screen = extended_screen.clone();
            move |_, data| {
                live_viewer.load_data(data);
                extended_screen.load_data(data);
            }
        });
        live_viewer.connect_slide_change({
            let extended_screen = extended_screen.clone();
            move |_, position| {
                extended_screen.set_pos(*position);
            }
        });

        let model = AppModel {
            schedule_viewer: RefCell::new(schedule_viewer),
            search_viewer,
            preview_viewer: RefCell::new(preview_viewer),
            live_viewer: RefCell::new(live_viewer),
            extended_screen: RefCell::new(extended_screen),
        };
        let widgets = view_output!();

        if let Some(display_geometry) = get_display_geometry() {
            window.set_default_width(display_geometry.width() / 2);
            window.set_default_height(display_geometry.height() / 2);
        }

        let app = relm4::main_application();
        add_app_menu(Some(&app), Some(&widgets.window_box));
        add_app_actions(&app);

        window.connect_destroy(move |_| {
            println!("Close");
        });

        widgets.main_window.present();

        let nd_screen = model.extended_screen.borrow().clone();
        app.add_window(&nd_screen);
        nd_screen.present();
        println!(
            "2nd-screen display = {:?}",
            WidgetExt::display(&nd_screen).monitors().n_items()
        );

        let monitors = WidgetExt::display(&nd_screen).monitors();
        if monitors.n_items() > 1
            && let Some(last_monitor) = monitors.iter::<gdk::Monitor>().last().and_then(|v| v.ok())
        {
            nd_screen.fullscreen_on_monitor(&last_monitor);
        }

        relm4::ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            AppInput::ScheduleActivityAddNew(payload) => {
                self.schedule_viewer.borrow().add_new_item(&payload);
            }

            // search model
            AppInput::SearchPreviewBackground(image_src) => {
                self.preview_viewer
                    .borrow()
                    .update_background(image_src.clone());
            }
            AppInput::SearchPreviewActivity(list_payload) => {
                self.preview_viewer.borrow().load_data(&list_payload);
            }
            AppInput::ClearLiveDisplay(cleared) => {
                self.live_viewer.borrow().clear_display(cleared);
                self.extended_screen.borrow().clear_display(cleared);
            }
            AppInput::PreviewGoLive => self.preview_viewer.borrow().emit_activate_slide(),
        };
    }
}

// const MIN_GRID_HEIGHT: i32 = 300;
const MIN_GRID_WIDTH: i32 = 300;

pub fn run() {
    gtk::glib::set_application_name("Open worship");
    let app = relm4::main_application();
    app.set_application_id(Some(config::APP_ID));
    app.set_resource_base_path(Some(config::RESOURCE_PATH));
    // relm4_icons::initialize_icons(icon_names::GRESOURCE_BYTES, icon_names::RESOURCE_PREFIX);

    let app = relm4::RelmApp::from_app(app);
    relm4::gtk::init().expect("Could not init gtk");
    app_init();

    log_display_info();

    // setup app
    AppConfig::init();

    app.run::<AppModel>(None);
}

fn log_display_info() {
    let display_backend = gtk::gdk::Display::default().expect("no display");

    let binding = display_backend.monitors();
    let d = binding
        .into_iter()
        .map(|m| m.unwrap().downcast::<gtk::gdk::Monitor>())
        .collect::<Vec<_>>();

    for m in &d {
        let x_mon = m.clone().unwrap();
        println!("|	monitor {:?}", &x_mon);
        println!("|	model {:?}", x_mon.model());
        println!("|	manufacturer {:?}", x_mon.manufacturer());
        println!("|	geometry {:?}", x_mon.geometry());
        println!("|	scale factor {:?}", x_mon.scale_factor());
        println!(
            "|	ratio {:?}",
            (x_mon.geometry().width() as f32 / x_mon.geometry().height() as f32)
        );
        println!("|	refresh rate {:?}hz", x_mon.refresh_rate());
    }

    if let Some(display) = d.last().and_then(|v| v.clone().ok()) {
        let aspect_ratio = display.geometry().width() as f32 / display.geometry().height() as f32;
        // let _ = AppConfig::set_aspect_ratio(aspect_ratio);
    }

    get_display_geometry();
}

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

fn get_last_display() {}

#[cfg(not(target_os = "macos"))]
fn add_app_menu(_app: Option<&gtk::Application>, window_box: Option<&gtk::Box>) {
    let window_box = match window_box {
        Some(w) => w,
        None => return,
    };

    let menu = build_app_menu();

    let menu_model: gtk::gio::MenuModel = menu.into();
    let menubar = PopoverMenuBar::from_model(Some(&menu_model));

    window_box.prepend(&menubar);
}

#[cfg(target_os = "macos")]
fn add_app_menu(app: Option<&gtk::Application>, _window_box: Option<&gtk::Box>) {
    let app = match app {
        Some(a) => a,
        None => return,
    };

    let menu = build_app_menu();
    app.set_menubar(Some(&menu));
}

fn build_app_menu() -> gtk::gio::Menu {
    let menu = gtk::gdk::gio::Menu::new();

    let file_menu = gtk::gio::Menu::new();
    menu.append_submenu(Some("File"), &file_menu);

    let edit_menu = gtk::gio::Menu::new();
    menu.append_submenu(Some("Edit"), &edit_menu);

    menu
}

fn add_app_actions(app: &gtk::Application) {
    let quit_action = gtk::gio::ActionEntry::builder("quit")
        .activate(|app: &gtk::Application, _, _| app.quit())
        .build();
    let about_action = gtk::gio::ActionEntry::builder("about")
        .activate(|_: &gtk::Application, _, _| add_app_about())
        .build();

    app.add_action_entries([quit_action, about_action]);
}

fn add_app_about(/* window: &impl IsA<gtk::Window> */) {
    let dialog = gtk::AboutDialog::builder()
        // .transient_for(window)
        // .modal(true)
        .program_name("About Openworship")
        .version(env!("CARGO_PKG_VERSION"))
        .website("https://github.com/micaiah-effiong/open-worship")
        .license_type(gtk::License::MitX11)
        .authors(["Micah Effiong"])
        .logo_icon_name("openworship-symbolic")
        .build();

    dialog.present();
}

// fn load_css() {
//     let css_provider = gtk::CssProvider::new();
//     css_provider.load_from_path("src/style.css");
//
//     gtk::style_context_add_provider_for_display(
//         &gtk::gdk::Display::default().expect("Could not connect to display"),
//         &css_provider,
//         gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
//     );
// }

fn app_init() {
    gtk::gio::resources_register_include!("resources.gresource")
        .expect("could not find app resources");

    setup_theme_listener();
    // if let Some(g_settings) = gtk::Settings::default() {
    //     g_settings.set_gtk_application_prefer_dark_theme(true);
    // }
    match gtk::glib::setenv("GTK_CSD", "0", false) {
        Ok(_) => (),
        Err(e) => {
            println!("An error occured while setting GTK_CSD:\n{:?}", e);
        }
    };
    // gtk::Window::set_default_icon_name("");
}
