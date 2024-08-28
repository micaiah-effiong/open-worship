use gtk::prelude::*;
use relm4::prelude::*;
use widgets::activity_screen::{ActivityScreenInput, ActivityScreenModel};
use widgets::live_activity_viewer::{
    LiveViewerData, LiveViewerInput, LiveViewerModel, LiveViewerOutput,
};
use widgets::preview_activity_viewer::{
    PreviewViewerInit, PreviewViewerInput, PreviewViewerModel, PreviewViewerOutput,
};
use widgets::schedule_activity_viewer::{
    ScheduleData, ScheduleViewerData, ScheduleViewerModel, ScheduleViewerOutput,
};
use widgets::search::{SearchInit, SearchModel, SearchOutput};
mod dto;
mod structs;
mod widgets;

#[derive(Debug)]
enum AppInput {
    ScheduleActivityActivated(dto::ListPayload),
    PreviewActivitySelected(dto::Payload),
    PreviewActivityActivated(dto::ListPayload),
    LiveActivitySelected(dto::Payload),
    LiveActivityActivated(String),

    //
    SearchPreviewBackground(String),
    SearchPreviewActivity(dto::ListPayload),
}
struct AppModel {
    schedule_activity_viewer: relm4::Controller<ScheduleViewerModel>,
    preview_activity_viewer: relm4::Controller<PreviewViewerModel>,
    live_activity_viewer: relm4::Controller<LiveViewerModel>,

    preview_activity_screen: relm4::Controller<ActivityScreenModel>,
    live_activity_screen: relm4::Controller<ActivityScreenModel>,
    search_viewer: relm4::Controller<SearchModel>,
}

impl AppModel {
    fn convert_schedule_activity_response(res: ScheduleViewerOutput) -> AppInput {
        return match res {
            ScheduleViewerOutput::Activated(payload) => {
                AppInput::ScheduleActivityActivated(payload)
            }
        };
    }
    fn convert_live_activity_response(res: LiveViewerOutput) -> AppInput {
        return match res {
            LiveViewerOutput::Selected(payload) => AppInput::LiveActivitySelected(payload),
            LiveViewerOutput::Activated(txt) => AppInput::LiveActivityActivated(txt),
        };
    }
    fn convert_preview_activity_response(res: PreviewViewerOutput) -> AppInput {
        return match res {
            PreviewViewerOutput::Selected(payload) => {
                println!("app preview {:?}", payload);
                AppInput::PreviewActivitySelected(payload)
            }
            PreviewViewerOutput::Activated(text) => AppInput::PreviewActivityActivated(text),
        };
    }

    fn convert_search_response(res: SearchOutput) -> AppInput {
        return match res {
            SearchOutput::PreviewBackground(image_src) => {
                AppInput::SearchPreviewBackground(image_src)
            }
            SearchOutput::PreviewScriptures(list) => AppInput::SearchPreviewActivity(list),
            SearchOutput::PreviewSongs(list) => AppInput::SearchPreviewActivity(list),
        };
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
        main_window = gtk::Window{
            // layout box
            #[wrap(Some)]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                // header box
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_height_request: 48,
                },

                // body box
                gtk::Box {
                    set_margin_all: 12,
                    set_orientation: gtk::Orientation::Vertical,

                    gtk::Box {
                        set_hexpand: true,
                        set_vexpand: true,
                        set_homogeneous: true,
                        set_orientation: gtk::Orientation::Horizontal,
                        add_css_class: "blue_box",

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
                                    set_start_child = Some(model.schedule_activity_viewer.widget()),
                                    // set_start_child = &gtk::Box {
                                    //     set_orientation: gtk::Orientation::Vertical,
                                    //     set_height_request: MIN_GRID_HEIGHT,
                                    //     set_hexpand: true,
                                    //     set_css_classes: &["pink_box", "ow-listview"],
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
                                        set_start_child = Some(model.preview_activity_viewer.widget()),
                                        set_end_child = Some(model.preview_activity_screen.widget()),
                                    }
                                },

                                #[wrap(Some)]
                                set_end_child = &gtk::Box {
                                    set_homogeneous: true,
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_vexpand: true,
                                    set_width_request: MIN_GRID_WIDTH,

                                    gtk::Paned {
                                        set_orientation: gtk::Orientation::Vertical,
                                        set_shrink_start_child: false,
                                        set_shrink_end_child: false,
                                        set_start_child = Some(model.live_activity_viewer.widget()),
                                        set_end_child = Some(model.live_activity_screen.widget()),
                                    }
                                }
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
        let schedule_activity_viewer = ScheduleViewerModel::builder()
            .launch(ScheduleViewerData {
                title: String::from("Schedule"),
                list: Vec::new() as Vec<ScheduleData>,
            })
            .forward(
                sender.input_sender(),
                AppModel::convert_schedule_activity_response,
            );
        let preview_activity_viewer = PreviewViewerModel::builder()
            .launch(PreviewViewerInit {})
            .forward(
                sender.input_sender(),
                AppModel::convert_preview_activity_response,
            );
        let live_activity_viewer = LiveViewerModel::builder()
            .launch(LiveViewerData {
                title: String::from("Live"),
                list: Vec::new(),
                selected_index: None,
            })
            .forward(
                sender.input_sender(),
                AppModel::convert_live_activity_response,
            );
        let search_viewer = SearchModel::builder()
            .launch(SearchInit {})
            .forward(sender.input_sender(), AppModel::convert_search_response);

        let preview_activity_screen = ActivityScreenModel::builder()
            .launch(())
            .forward(sender.input_sender(), |_| unreachable!());
        let live_activity_screen = ActivityScreenModel::builder()
            .launch(())
            .forward(sender.input_sender(), |_| unreachable!());

        let model = AppModel {
            schedule_activity_viewer,
            preview_activity_viewer,
            live_activity_viewer,
            search_viewer,
            preview_activity_screen,
            live_activity_screen,
        };
        let widgets = view_output!();

        if let Some(display_geometry) = get_display_geometry() {
            window.set_default_width(display_geometry.width() / 2);
            window.set_default_height(display_geometry.height() / 2);
        }

        return relm4::ComponentParts { model, widgets };
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            // schedule
            AppInput::ScheduleActivityActivated(payload) => {
                self.preview_activity_viewer
                    .emit(PreviewViewerInput::NewList(payload.clone()));
                if let Some(txt) = payload.list.get(0) {
                    self.preview_activity_screen
                        .emit(ActivityScreenInput::DisplayUpdate(
                            dto::DisplayPayload::new(txt.to_string()),
                        ));
                }
            }

            // live
            AppInput::LiveActivityActivated(_) => return,
            AppInput::LiveActivitySelected(payload) => {
                self.live_activity_screen
                    .emit(ActivityScreenInput::DisplayUpdate(
                        dto::DisplayPayload::new(payload.text),
                    ))
            }

            // preview
            AppInput::PreviewActivitySelected(payload) => {
                self.preview_activity_screen
                    .emit(ActivityScreenInput::DisplayUpdate(
                        dto::DisplayPayload::new(payload.text),
                    ));
            }
            AppInput::PreviewActivityActivated(list_payload) => {
                self.live_activity_viewer
                    .emit(LiveViewerInput::NewList(list_payload.clone())); //
                self.preview_activity_screen
                    .emit(ActivityScreenInput::DisplayUpdate(
                        dto::DisplayPayload::new(list_payload.text.clone()),
                    ));
                self.live_activity_screen
                    .emit(ActivityScreenInput::DisplayUpdate(
                        dto::DisplayPayload::new(list_payload.text.clone()),
                    ));

                if let Some(image_src) = list_payload.background_image {
                    self.live_activity_screen
                        .emit(ActivityScreenInput::DisplayBackground(image_src));
                }
            }

            AppInput::SearchPreviewBackground(image_src) => {
                self.preview_activity_screen
                    .emit(ActivityScreenInput::DisplayBackground(image_src.clone()));
                self.preview_activity_viewer
                    .emit(PreviewViewerInput::Background(image_src));
            }
            AppInput::SearchPreviewActivity(list_payload) => {
                if let Some(item) = list_payload.list.get(0) {
                    self.preview_activity_screen
                        .emit(ActivityScreenInput::DisplayUpdate(
                            dto::DisplayPayload::new(item.clone()),
                        ));
                }
                self.preview_activity_viewer
                    .emit(PreviewViewerInput::NewList(list_payload));
            }
        };
    }
}

const APP_ID: &str = "com.open-worship.app";
const RESOURECE_PATH: &str = "/com/open-worship/app";

// const MIN_GRID_HEIGHT: i32 = 300;
const MIN_GRID_WIDTH: i32 = 300;

fn main() {
    let app = relm4::main_application();
    app.set_application_id(Some(APP_ID));
    app.set_resource_base_path(Some(RESOURECE_PATH));
    relm4_icons::initialize_icons();

    let app = relm4::RelmApp::from_app(app);
    let _ = relm4::gtk::init();

    log_display_info();
    let _ = app.set_global_css_from_file(std::path::Path::new("./src/style.css"));

    app.run::<AppModel>(None);
}

fn log_display_info() {
    let display_backend = gtk::gdk::Display::default().expect("no display");

    let binding = display_backend.monitors();
    let d = binding
        .into_iter()
        .map(|m| m.unwrap().downcast::<gtk::gdk::Monitor>());

    d.for_each(|m| {
        let x_mon = m.unwrap();
        println!("|	monitor {:?}", &x_mon);
        println!("|	model {:?}", x_mon.model());
        println!("|	manufacturer {:?}", x_mon.manufacturer());
        println!("|	geometry {:?}", x_mon.geometry());
        println!(
            "|	ratio {:?}",
            (x_mon.geometry().width() as f32 / x_mon.geometry().height() as f32)
        );
        println!("|	refresh rate {:?}hz", x_mon.refresh_rate() / 1000);
    });

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

    return Some(geometry);
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
