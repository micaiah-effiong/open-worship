use std::u32;

pub mod widgets;
use gtk::prelude::*;
use relm4::prelude::*;
use widgets::activity_screen::ActivityScreenModel;
use widgets::live_activity_viewer::{
    LiveViewerData, LiveViewerInput, LiveViewerModel, LiveViewerOutput,
};
use widgets::preview_activity_viewer::{
    PreviewViewerData, PreviewViewerModel, PreviewViewerOutput,
};
use widgets::schedule_activity_viewer::{
    ScheduleViewerData, ScheduleViewerModel, ScheduleViewerOutput,
};
use widgets::search::{SearchInit, SearchModel};

#[derive(Debug)]
enum AppInput {
    ScheduleActivitySelected(Vec<String>, u32),
    PreviewActivitySelected(Vec<String>, u32),
    LiveActivitySelected(Vec<String>, u32),
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
            ScheduleViewerOutput::Selected(list, num) => {
                AppInput::ScheduleActivitySelected(list, num)
            }
        };
    }
    fn convert_live_activity_response(res: LiveViewerOutput) -> AppInput {
        return match res {
            LiveViewerOutput::Selected(list, num) => {
                println!(
                    "app receive live selected pos={:?} len={:?}",
                    num,
                    list.len()
                );
                AppInput::LiveActivitySelected(list, num)
            }
        };
    }
    fn convert_preview_activity_response(res: PreviewViewerOutput) -> AppInput {
        return match res {
            PreviewViewerOutput::Selected(list, num) => {
                println!(
                    "app receive preview selected pos={:?} len={:?}",
                    num,
                    list.len()
                );
                AppInput::PreviewActivitySelected(list, num)
            }
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
                list: Vec::new(),
                selected_index: None,
            })
            .forward(
                sender.input_sender(),
                AppModel::convert_schedule_activity_response,
            );
        let preview_activity_viewer = PreviewViewerModel::builder()
            .launch(PreviewViewerData {
                title: String::from("Preview"),
                list: Vec::from(LIST_VEC.map(|s| s.to_string())),
                selected_index: None,
            })
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
            .forward(sender.input_sender(), |_| unreachable!());

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
            AppInput::PreviewActivitySelected(list, num) => {
                self.live_activity_viewer
                    .emit(LiveViewerInput::NewList(list, num)) //
            }
            AppInput::LiveActivitySelected(_, _) => return,
            AppInput::ScheduleActivitySelected(_, _) => return,
        };
    }
}

const APP_ID: &str = "com.open-worship";

const LIST_VEC: [&str; 16] = [
    "Golden sun, a radiant masterpiece, paints the canvas of the morning sky,",
    "With hues of pink and softest blue, a breathtaking, ethereal sight,",
    "A gentle breeze, a whispered lullaby, carries softly through and through,",
    "Enveloping the world in calm as morning dew begins to fall anew.",
    "Dew-kissed flowers, adorned with sparkling gems, open wide to greet the day,",
    "Unfurling petals, soft and sweet, in a vibrant, colorful display,",
    "Nature's beauty, a masterpiece, unfolds before our wondering eyes,",
    "Inviting us to pause and breathe, beneath the endless, open skies.",
    "Children laugh, their joy infectious, as they chase their dreams so high,",
    "Imaginations soar and fly, reaching for the boundless sky,",
    "Hopeful wishes, like tiny stars, twinkle brightly in their hearts,",
    "As golden moments slip away, leaving precious, lasting marks.",
    "Hand in hand, we'll journey on, through life's winding, twisting road,",
    "With courage, strength, and hearts aflame, carrying hope's precious load,",
    "Brighter days, a promised land, await us just beyond the bend,",
    "As love and friendship's bonds endure, forever and without an end.",
];

// const MIN_GRID_HEIGHT: i32 = 300;
const MIN_GRID_WIDTH: i32 = 300;

fn main() {
    let app = relm4::RelmApp::new(APP_ID);
    load_css();
    log_display_info();
    app.run::<AppModel>(None);
}

fn load_css() {
    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_path("src/style.css");

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("Could not connect to display"),
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_USER,
    );
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
