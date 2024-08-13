use crate::widgets::live_activity_viewer::{LiveViewerData, LiveViewerModel};
use crate::widgets::preview_activity_viewer::{
    PreviewViewerData, PreviewViewerModel, PreviewViewerOutput,
};

use std::u32;

pub mod widgets;
use gtk::{glib::clone, prelude::*};
use relm4::prelude::*;
use widgets::live_activity_viewer::{LiveViewerInput, LiveViewerOutput};

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
        main_window = gtk::Window{
            // if let Some(display_geometry) = get_display_geometry() {
            //     set_default_width = display_geometry.width() / 2;
            //     set_default_height = display_geometry.height() / 2;
            // },

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
            })
            .forward(sender.input_sender(), |_| unreachable!());
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
            .launch(())
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

        // window.set_child(Some(&layout_box));

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

const  PREVIEW_SCREEN_LABEL_STR: &str = "
Lorem ipsum dolor sit amet, officia excepteur ex fugiat reprehenderit enim labore culpa sint ad nisi Lorem pariatur mollit ex esse exercitation amet.
Nisi anim cupidatat excepteur officia.
Reprehenderit nostrud nostrud ipsum Lorem est aliquip amet voluptate voluptate dolor minim nulla est proident.
Nostrud officia pariatur ut officia.
Sit irure elit esse ea nulla sunt ex occaecat reprehenderit commodo officia dolor Lorem duis laboris cupidatat officia voluptate.
Culpa proident adipisicing id nulla nisi laboris ex in Lorem sunt duis officia eiusmod.
Aliqua reprehenderit commodo ex non excepteur duis sunt velit enim.
Voluptate laboris sint cupidatat ullamco ut ea consectetur et est culpa et culpa duis.
";

const MIN_GRID_HEIGHT: i32 = 300;
const MIN_GRID_WIDTH: i32 = 300;

fn main() {
    let app = relm4::RelmApp::new(APP_ID);
    load_css();
    log_display_info();
    app.run::<AppModel>(None);
}

fn build_bible_search_tab(container: &gtk::Notebook, label: &str) -> gtk::Box {
    let search_result_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    search_result_box.add_css_class("blue_box");
    search_result_box.set_vexpand(true);

    let search_field_box = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    search_field_box.set_height_request(48);
    search_field_box.add_css_class("green_double_box");
    {
        let search_input = gtk::SearchEntry::builder()
            .placeholder_text("Search...")
            .hexpand(true)
            .build();
        search_field_box.append(&search_input);
        search_result_box.append(&search_field_box);
    }

    // result lists
    {
        let list_model: gtk::StringList = (0..=3000).map(|num| num.to_string()).collect();
        // let result_list_modal = gtk::gio::ListStore::new::<gtk::StringObject>();
        // result_list_modal.extend_from_slice(&list_vec);

        let signal_selection_factory = gtk::SignalListItemFactory::new();
        signal_selection_factory.connect_setup(move |_, list_item| {
            let label = gtk::Label::new(None);
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            label.set_single_line_mode(true);
            label.set_halign(gtk::Align::Start);
            label.set_justify(gtk::Justification::Fill);

            list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("Must be a list item")
                .set_child(Some(&label));

            list_item
                .property_expression("item")
                .chain_property::<gtk::StringObject>("string")
                .bind(&label, "label", gtk::Widget::NONE);
        });

        let single_selection_modal = gtk::SingleSelection::new(Some(list_model));
        let list_view =
            gtk::ListView::new(Some(single_selection_modal), Some(signal_selection_factory));

        let scroll_view = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .child(&list_view)
            .build();

        search_result_box.append(&scroll_view);
    }

    let bible_label = gtk::Label::new(Some(label));
    container.append_page(&search_result_box, Some(&bible_label));

    return search_result_box;
}

fn build_background_search_tab(container: &gtk::Notebook, label: &str) -> gtk::Box {
    let search_result_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    search_result_box.add_css_class("blue_box");
    search_result_box.set_vexpand(true);

    let search_field_box = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    search_field_box.set_height_request(48);
    search_field_box.add_css_class("green_double_box");
    {
        let search_input = gtk::SearchEntry::builder()
            .placeholder_text("Search...")
            .hexpand(true)
            .build();
        search_field_box.append(&search_input);
        search_result_box.append(&search_field_box);
    }

    // result lists
    {
        let list_model: gtk::StringList = (0..=3000).map(|num| num.to_string()).collect();
        // let result_list_modal = gtk::gio::ListStore::new::<gtk::StringObject>();
        // result_list_modal.extend_from_slice(&list_model);

        let signal_selection_factory = gtk::SignalListItemFactory::new();
        signal_selection_factory.connect_setup(move |_, list_item| {
            let label = gtk::Label::new(None);
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            label.set_halign(gtk::Align::Start);
            label.set_justify(gtk::Justification::Fill);

            list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("Must be a list item")
                .set_child(Some(&label));

            list_item
                .property_expression("item")
                .chain_property::<gtk::StringObject>("string")
                .bind(&label, "label", gtk::Widget::NONE);
        });

        let single_selection_modal = gtk::SingleSelection::new(Some(list_model));
        let list_view =
            gtk::ListView::new(Some(single_selection_modal), Some(signal_selection_factory));

        let scroll_view = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .child(&list_view)
            .build();

        search_result_box.append(&scroll_view);
    }

    let bible_label = gtk::Label::new(Some(label));
    container.append_page(&search_result_box, Some(&bible_label));

    return search_result_box;
}

fn build_songs_search_tab(container: &gtk::Notebook, label: &str) -> gtk::Box {
    let search_result_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    search_result_box.add_css_class("blue_box");
    search_result_box.set_vexpand(true);

    let search_field_box = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    search_field_box.set_height_request(48);
    search_field_box.add_css_class("green_double_box");
    {
        let search_input = gtk::SearchEntry::builder()
            .placeholder_text("Search...")
            .hexpand(true)
            .build();
        search_field_box.append(&search_input);
        search_result_box.append(&search_field_box);
    }

    // result lists
    {
        let list_model: gtk::StringList = (0..=3000).map(|_| LIST_VEC[0]).collect();
        // let result_list_modal = gtk::gio::ListStore::new::<gtk::StringObject>();
        // result_list_modal.extend_from_slice(&list_model);

        let signal_selection_factory = gtk::SignalListItemFactory::new();
        signal_selection_factory.connect_setup(move |_, list_item| {
            let label = gtk::Label::new(None);
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            label.set_halign(gtk::Align::Start);
            label.set_justify(gtk::Justification::Fill);

            list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("Must be a list item")
                .set_child(Some(&label));

            list_item
                .property_expression("item")
                .chain_property::<gtk::StringObject>("string")
                .bind(&label, "label", gtk::Widget::NONE);
        });

        let single_selection_modal = gtk::SingleSelection::new(Some(list_model));
        let list_view =
            gtk::ListView::new(Some(single_selection_modal), Some(signal_selection_factory));

        let scroll_view = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .child(&list_view)
            .build();

        search_result_box.append(&scroll_view);
    }

    let bible_label = gtk::Label::new(Some(label));
    container.append_page(&search_result_box, Some(&bible_label));

    return search_result_box;
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

#[derive(Debug)]
enum ScheduleViewerInput {
    Selected(u32),
}
#[derive(Debug)]
enum ScheduleViewerOutput {
    Selected(Vec<String>, u32),
}
struct ScheduleViewerData {
    title: String,
    list: Vec<String>,
}
struct ScheduleViewerModel {
    title: String,
    list_view: gtk::ListView,
    list: Vec<String>,
}

#[relm4::component]
impl SimpleComponent for ScheduleViewerModel {
    type Input = ScheduleViewerInput;
    type Output = ScheduleViewerOutput;
    type Init = ScheduleViewerData;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_height_request: MIN_GRID_HEIGHT,
            set_css_classes: &["pink_box", "ow-listview"],

            gtk::Label {
                set_label: &model.title
            },

            gtk::ScrolledWindow {
                set_vexpand: true,
                set_child: Some(&model.list_view)
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let signal_selection_factory = gtk::SignalListItemFactory::new();
        signal_selection_factory.connect_setup(move |_, list_item| {
            let label = gtk::Label::builder()
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .wrap_mode(gtk::pango::WrapMode::Word)
                .lines(2)
                .margin_top(12)
                .margin_bottom(12)
                .halign(gtk::Align::Start)
                .justify(gtk::Justification::Fill)
                .build();

            list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("Must be a list item")
                .set_child(Some(&label));

            list_item
                .property_expression("item")
                .chain_property::<gtk::StringObject>("string")
                .bind(&label, "label", gtk::Widget::NONE);
        });

        let single_selection_modal = gtk::SingleSelection::new(Some(
            init.list.clone().into_iter().collect::<gtk::StringList>(),
        ));

        let list_view =
            gtk::ListView::new(Some(single_selection_modal), Some(signal_selection_factory));

        list_view.connect_activate(clone!(@strong sender =>  move |m, pos| {
                println!("lv-act {:?} || {:?}", m, pos);
                sender.input(ScheduleViewerInput::Selected(pos));
            }
        ));

        let model = ScheduleViewerModel {
            title: init.title,
            list: init.list,
            list_view: list_view.clone(),
        };

        let widgets = view_output!();

        return relm4::ComponentParts { model, widgets };
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        let _ = match message {
            ScheduleViewerInput::Selected(position) => {
                sender.output(ScheduleViewerOutput::Selected(self.list.clone(), position))
            }
        };

        return ();
    }
}

// search area (notebook)
#[derive(Debug)]
enum SearchInput {}
struct SearchModel {}
struct SearchWidget {}

impl SimpleComponent for SearchModel {
    type Init = ();
    type Output = ();
    type Root = gtk::Box;
    type Input = SearchInput;
    type Widgets = SearchWidget;

    fn init_root() -> Self::Root {
        return gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .height_request(MIN_GRID_HEIGHT)
            .hexpand(true)
            .homogeneous(true)
            .build();
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let tab_box = gtk::Box::new(gtk::Orientation::Horizontal, 3);
        tab_box.set_css_classes(&["purple_box", "ow-listview"]);
        tab_box.set_height_request(48);

        let notebook = gtk::Notebook::new();
        notebook.set_hexpand(true);
        {
            build_songs_search_tab(&notebook, "Songs");
            build_bible_search_tab(&notebook, "Scriptures");
            build_background_search_tab(&notebook, "Backgrounds");
        }
        tab_box.append(&notebook);
        root.append(&tab_box);

        return relm4::ComponentParts {
            model: SearchModel {},
            widgets: SearchWidget {},
        };
    }
}

// actrivity screen
#[derive(Debug)]
enum ActivityScreenInput {}
struct ActivityScreenModel {}
struct ActivityScreenWidget {}

impl SimpleComponent for ActivityScreenModel {
    type Init = ();
    type Input = ActivityScreenInput;
    type Output = ();
    type Root = gtk::Frame;
    type Widgets = ActivityScreenWidget;

    fn init_root() -> Self::Root {
        return gtk::Frame::new(None);
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let screen_box = gtk::Box::builder()
            .homogeneous(true)
            .height_request(MIN_GRID_HEIGHT)
            .build();
        screen_box.set_css_classes(&["brown_box", "black_bg_box"]);
        screen_box.set_overflow(gtk::Overflow::Hidden);

        let live_screen_label = gtk::Label::builder()
            .label(PREVIEW_SCREEN_LABEL_STR)
            .justify(gtk::Justification::Center)
            .wrap(true)
            .wrap_mode(gtk::pango::WrapMode::Word)
            .build();

        live_screen_label.set_css_classes(&["red_box", "white", "yellow_box"]);
        screen_box.append(&live_screen_label);

        root.set_child(Some(&screen_box));

        return relm4::ComponentParts {
            model: ActivityScreenModel {},
            widgets: ActivityScreenWidget {},
        };
    }
}
