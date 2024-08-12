use std::u32;

use gtk::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
enum AppInput {}
struct AppModel {
    schedule_activity_viewer: relm4::Controller<ActivityViewerModel>,
    preview_activity_viewer: relm4::Controller<ActivityViewerModel>,
    live_activity_viewer: relm4::Controller<ActivityViewerModel>,

    preview_activity_screen: relm4::Controller<ActivityScreenModel>,
    live_activity_screen: relm4::Controller<ActivityScreenModel>,
    search_viewer: relm4::Controller<SearchModel>,
}
// struct AppWidgets {}

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

    // fn init_root() -> Self::Root {
    //     let window = gtk::Window::builder().title("Open Worship").build();
    //
    //     if let Some(display_geometry) = get_display_geometry() {
    //         window.set_default_width(display_geometry.width() / 2);
    //         window.set_default_height(display_geometry.height() / 2);
    //     }
    //
    //     return window;
    // }
    fn init(
        _init: Self::Init,
        window: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        // let close_action = gtk::gio::ActionEntry::builder("close")
        // .activate(gtk::glib::clone!(
        //     #[weak]
        //     window,
        //     move |_, _, _| window.close()
        // ))
        // .build();

        // let action_group = gtk::gio::SimpleActionGroup::new();
        // action_group.add_action_entries([close_action]);
        // let layout_box = build_layout();

        let schedule_activity_viewer = ActivityViewerModel::builder()
            .launch(ActivityViewerData {
                title: String::from("Schedule"),
            })
            .forward(_sender.input_sender(), |_| unreachable!());
        let preview_activity_viewer = ActivityViewerModel::builder()
            .launch(ActivityViewerData {
                title: String::from("Preview"),
            })
            .forward(_sender.input_sender(), |_| unreachable!());
        let live_activity_viewer = ActivityViewerModel::builder()
            .launch(ActivityViewerData {
                title: String::from("Live"),
            })
            .forward(_sender.input_sender(), |_| unreachable!());
        let search_viewer = SearchModel::builder()
            .launch(())
            .forward(_sender.input_sender(), |_| unreachable!());

        let preview_activity_screen = ActivityScreenModel::builder()
            .launch(())
            .forward(_sender.input_sender(), |_| unreachable!());
        let live_activity_screen = ActivityScreenModel::builder()
            .launch(())
            .forward(_sender.input_sender(), |_| unreachable!());

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
}

const APP_ID: &str = "com.open-worship";
const LIST_VEC: [&str; 5] = [
        "Lorem ipsum dolor sit amet, qui minim labore adipisicing minim sint cillum sint consectetur cupidatat.",
        "Lorem ipsum dolor sit amet, qui minim labore adipisicing minim sint cillum sint consectetur cupidatat.",
        "Lorem ipsum dolor sit amet, qui minim labore adipisicing minim sint cillum sint consectetur cupidatat.",
        "Lorem ipsum dolor sit amet, qui minim labore adipisicing minim sint cillum sint consectetur cupidatat.",
        "Lorem ipsum dolor sit amet, qui minim labore adipisicing minim sint cillum sint consectetur cupidatat. Lorem ipsum dolor sit amet, officia excepteur ex fugiat reprehenderit enim labore culpa sint ad nisi Lorem pariatur mollit ex esse exercitation amet. Nisi anim cupidatat excepteur officia. Reprehenderit nostrud nostrud ipsum Lorem est aliquip amet voluptate voluptate dolor minim nulla est proident. Nostrud officia pariatur ut officia. Sit irure elit esse ea nulla sunt ex occaecat reprehenderit commodo officia dolor Lorem duis laboris cupidatat officia voluptate. Culpa proident adipisicing id nulla nisi laboris ex in Lorem sunt duis officia eiusmod. Aliqua reprehenderit commodo ex non excepteur duis sunt velit enim. Voluptate laboris sint cupidatat ullamco ut ea consectetur et est culpa et culpa duis."
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

fn build_layout() -> gtk::Box {
    let layout_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();
    // header
    let header_box = gtk::Box::builder().height_request(48).build();
    build_header_content(&header_box);

    // body
    let body_box = gtk::Box::builder()
        .margin_end(12)
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .orientation(gtk::Orientation::Vertical)
        .build();
    build_body_content(&body_box);

    // footer
    let footer_label = gtk::Label::builder().label("footer").build();
    let footer_box = gtk::Box::builder().margin_end(12).build();
    footer_box.append(&footer_label);

    layout_box.append(&header_box);
    layout_box.append(&body_box);
    layout_box.append(&footer_box);

    return layout_box;
}

fn build_header_content(header_box: &gtk::Box) {
    let header_label = gtk::Label::builder().label("header").build();
    let header_space = gtk::Box::builder().hexpand(true).build();

    let button = gtk::Button::builder()
        .label("Go live")
        .margin_end(12)
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .build();

    let button_2 = gtk::Button::builder()
        .label("Blank")
        .margin_end(12)
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .build();

    button_2.set_css_classes(&["btn", "btn-blue"]);

    // let number = std::rc::Rc::new(std::cell::Cell::new(-1));

    // button_2.connect_clicked(gtk::glib::clone!(
    //     #[weak]
    //     number,
    //     #[weak]
    //     button,
    //     move |_| {
    //         number.set(number.get() + 1);
    //         let mut number_str = number.get().to_string();
    //         number_str.insert_str(0, "Live ");
    //         button.set_label(&number_str);
    //     }
    // ));
    //
    // button.connect_clicked(move |btn| {
    //     gtk::glib::spawn_future_local(gtk::glib::clone!(
    //         #[weak]
    //         btn,
    //         #[weak]
    //         number,
    //         async move {
    //             number.set(number.get() + 1);
    //             let mut num_str = number.get().to_string();
    //             num_str.insert_str(0, "Live ");
    //             btn.set_label(&num_str);
    //
    //             btn.set_sensitive(false);
    //             // glib::timeout_future_seconds(2).await;
    //             let wait_result = gtk::gio::spawn_blocking(move || {
    //                 let wait = std::time::Duration::from_secs(2);
    //                 std::thread::sleep(wait);
    //                 return true;
    //             })
    //             .await
    //             .expect("Blocking task must finish running");
    //             btn.set_sensitive(wait_result);
    //         }
    //     ));
    // });

    let button_close_from_action_entry = gtk::Button::builder()
        .label("close")
        .margin_end(12)
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .build();

    // button_close_from_action_entry.connect_clicked(gtk::glib::clone!(
    //     #[weak]
    //     button_close_from_action_entry,
    //     move |_| {
    //         button_close_from_action_entry
    //             .activate_action("custom-group.close", None)
    //             .expect("Should have close action")
    //     }
    // ));
    //
    header_box.append(&header_label);
    header_box.append(&header_space);
    header_box.append(&button);
    header_box.append(&button_2);
    header_box.append(&button_close_from_action_entry);
}

fn build_body_content(body_box: &gtk::Box) {
    let body_container = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .hexpand(true)
        .vexpand(true)
        .homogeneous(true)
        .build();

    body_container.add_css_class("blue_box");

    let pane1 = gtk::Paned::new(gtk::Orientation::Horizontal);
    let pane2 = gtk::Paned::new(gtk::Orientation::Horizontal);
    // schedule and search
    build_schedule_and_search(&pane1);

    // preview and screen
    build_preview_and_screen(&pane2);

    // live and screen
    build_live_and_screen(&pane2);

    pane1.set_end_child(Some(&pane2));
    pane1.set_shrink_end_child(false);
    pane1.set_shrink_start_child(false);

    pane2.set_shrink_end_child(false);
    pane2.set_shrink_start_child(false);

    body_container.append(&pane1);

    body_box.set_homogeneous(true);
    body_box.add_css_class("red_box");
    body_box.set_vexpand(true);
    body_box.append(&body_container);
}

fn build_schedule_activity_viewer(container: &gtk::Box) {
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

    let sub_list = (&LIST_VEC[..2]).to_owned();
    let single_selection_modal =
        gtk::SingleSelection::new(Some(sub_list.into_iter().collect::<gtk::StringList>()));
    let list_view =
        gtk::ListView::new(Some(single_selection_modal), Some(signal_selection_factory));

    let scroll_view = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .child(&list_view)
        .build();

    container.append(&scroll_view);
}

fn build_preview_activity_viewer(container: &gtk::Box) {
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

        // let gesture = gtk::GestureClick::new();
        //
        // gesture.set_button(gtk::gdk::ffi::GDK_BUTTON_PRIMARY as u32);
        // gesture.connect_pressed(|g, m, n, o| {
        //     g.set_state(gtk::EventSequenceState::Claimed);
        //     println!("clicked {} || {:?} || {:?}", m, n, o);
        //     list_item.
        //     g.set_state(gtk::EventSequenceState::Denied);
        // });
        //
        // label.add_controller(gesture);
    });

    // let cb_app_state = ;

    let single_selection_modal =
        gtk::SingleSelection::new(Some(LIST_VEC.into_iter().collect::<gtk::StringList>()));

    // single_selection_modal.connect_selection_changed(|m, p, n| {
    //     if let Some(item) = m.selected_item() {
    //         println!("act {:?} || {:?} || {:?}", item, p, n);
    //     }
    // });

    let list_view =
        gtk::ListView::new(Some(single_selection_modal), Some(signal_selection_factory));

    // list_view.connect_activate(move |m, n| {
    //     println!("lv-act {:?} || {:?}", m, n);
    //     println!("pre-list -> {:?}", app_state.borrow().live_state.list);
    //     let list = app_state.borrow().preview_state.list.clone();
    //     app_state.borrow_mut().live_state.set_list(list, None);
    //     println!("list -> {:?}", app_state.borrow().live_state.list);
    // });
    //
    let scroll_view = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .child(&list_view)
        .build();

    container.append(&scroll_view);
}

fn build_live_activity_viewer(container: &gtk::Box) {
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

        let gesture = gtk::GestureClick::new();

        gesture.set_button(gtk::gdk::ffi::GDK_BUTTON_PRIMARY as u32);
        gesture.connect_pressed(|g, m, _, _| {
            g.set_state(gtk::EventSequenceState::Claimed);
            println!("clicked {} ", m);
            g.set_state(gtk::EventSequenceState::Denied);
        });

        label.add_controller(gesture);
    });

    let single_selection_modal =
        gtk::SingleSelection::new(Some(LIST_VEC.into_iter().collect::<gtk::StringList>()));
    let list_view =
        gtk::ListView::new(Some(single_selection_modal), Some(signal_selection_factory));

    let scroll_view = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .child(&list_view)
        .build();

    container.append(&scroll_view);
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

fn build_schedule_and_search(container: &gtk::Paned) {
    let content_box = gtk::Box::builder()
        .homogeneous(true)
        .orientation(gtk::Orientation::Vertical)
        .vexpand(true)
        .width_request(MIN_GRID_WIDTH)
        .build();
    let content_pane = gtk::Paned::new(gtk::Orientation::Vertical);
    content_box.append(&content_pane);

    {
        let schedule_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .height_request(MIN_GRID_HEIGHT)
            .hexpand(true)
            .build();
        schedule_box.set_css_classes(&["pink_box", "ow-listview"]);
        let s_box_label = gtk::Label::builder().label("Schedule").build();
        schedule_box.append(&s_box_label);

        build_schedule_activity_viewer(&schedule_box);

        content_pane.set_start_child(Some(&schedule_box));
    }

    {
        let search_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .height_request(MIN_GRID_HEIGHT)
            .hexpand(true)
            .homogeneous(true)
            .build();
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
        search_box.append(&tab_box);

        content_pane.set_end_child(Some(&search_box));
    }

    content_pane.set_shrink_start_child(false);
    content_pane.set_shrink_end_child(false);

    container.set_start_child(Some(&content_box));
}

fn build_preview_and_screen(container: &gtk::Paned) {
    let content_box = gtk::Box::builder()
        .homogeneous(true)
        .orientation(gtk::Orientation::Vertical)
        .vexpand(true)
        .width_request(MIN_GRID_WIDTH)
        .build();
    let content_pane = gtk::Paned::new(gtk::Orientation::Vertical);
    content_box.append(&content_pane);

    {
        let preview_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .hexpand(true)
            .height_request(MIN_GRID_HEIGHT)
            .build();
        let preview_box_label = gtk::Label::builder().label("Preview").build();
        preview_box.append(&preview_box_label);
        preview_box.set_css_classes(&["pink_box", "ow-listview"]);

        build_preview_activity_viewer(&preview_box);
        content_pane.set_start_child(Some(&preview_box));
    }

    {
        let preview_screen_box = gtk::Box::builder()
            .homogeneous(true)
            .height_request(MIN_GRID_HEIGHT)
            .build();
        preview_screen_box.set_css_classes(&["brown_box", "black_bg_box"]);
        preview_screen_box.set_overflow(gtk::Overflow::Hidden);
        // preview_screen_box.set_vexpand(true);

        let preview_screen_label = gtk::Label::builder()
            .label(PREVIEW_SCREEN_LABEL_STR)
            .justify(gtk::Justification::Center)
            .wrap(true)
            .wrap_mode(gtk::pango::WrapMode::Word)
            .build();

        preview_screen_label.set_css_classes(&["red_box", "white", "yellow_box"]);
        preview_screen_label.set_markup(&format!(
            "<span foreground=\"white\" size=\"{}pt\"> testing </span>",
            30,
        ));
        preview_screen_box.append(&preview_screen_label);

        let preview_frame = gtk::Frame::new(None);
        preview_frame.set_child(Some(&preview_screen_box));

        content_pane.set_end_child(Some(&preview_frame));
    }

    content_pane.set_shrink_start_child(false);
    content_pane.set_shrink_end_child(false);

    container.set_start_child(Some(&content_box));
}

fn build_live_and_screen(container: &gtk::Paned) {
    let content_box = gtk::Box::builder()
        .homogeneous(true)
        .orientation(gtk::Orientation::Vertical)
        .vexpand(true)
        .width_request(MIN_GRID_WIDTH)
        .build();
    let content_pane = gtk::Paned::new(gtk::Orientation::Vertical);
    content_box.append(&content_pane);

    {
        let live_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .hexpand(true)
            .vexpand(true)
            .height_request(MIN_GRID_HEIGHT)
            .build();

        let live_box_label = gtk::Label::builder().label("Live").build();
        live_box.append(&live_box_label);
        live_box.set_css_classes(&["pink_box", "ow-listview"]);

        build_live_activity_viewer(&live_box);

        content_pane.set_start_child(Some(&live_box));
    }

    {
        let live_screen_box = gtk::Box::builder()
            .homogeneous(true)
            .height_request(MIN_GRID_HEIGHT)
            .build();
        live_screen_box.set_css_classes(&["brown_box", "black_bg_box"]);
        live_screen_box.set_overflow(gtk::Overflow::Hidden);
        // live_screen_box.set_vexpand(true);

        let live_screen_label = gtk::Label::builder()
            .label(PREVIEW_SCREEN_LABEL_STR)
            .justify(gtk::Justification::Center)
            .wrap(true)
            .wrap_mode(gtk::pango::WrapMode::Word)
            .build();

        live_screen_label.set_css_classes(&["red_box", "white", "yellow_box"]);
        live_screen_box.append(&live_screen_label);

        let live_frame = gtk::Frame::new(None);
        live_frame.set_child(Some(&live_screen_box));

        content_pane.set_end_child(Some(&live_frame));
    }

    content_pane.set_shrink_start_child(false);
    content_pane.set_shrink_end_child(false);

    container.set_end_child(Some(&content_box));
}

// model
// widget
// input
#[derive(Debug)]
enum ActivityViewerInput {}
struct ActivityViewerData {
    title: String,
}
struct ActivityViewerModel {
    title: String,
    list: gtk::ListView,
}

#[relm4::component]
impl SimpleComponent for ActivityViewerModel {
    type Input = ActivityViewerInput;
    type Output = ();
    type Init = ActivityViewerData;
    // type Root = gtk::Box;
    // type Widgets = ActivityViewerWidget;

    // fn init_root() -> Self::Root {
    //     let root_box = gtk::Box::builder()
    //         .orientation(gtk::Orientation::Vertical)
    //         .hexpand(true)
    //         .height_request(MIN_GRID_HEIGHT)
    //         .build();
    //     let s_box_label = gtk::Label::builder().label("Schedule").build();
    //     root_box.append(&s_box_label);
    //
    //     root_box.set_css_classes(&["pink_box", "ow-listview"]);
    //
    //     return root_box;
    // }

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

                #[local_ref]
                the_list -> gtk::ListView{
                    // #[wrap(Some)]
                    // set_model = gtk::SingleSelection{
                    //     set_model = Some(LIST_VEC.into_iter().collect::<gtk::StringList>())
                    // },
                    // #[wrap(Some)]
                    // set_factory = gtk::SignalListItemFactory{
                    //     // set_factory =
                    // },
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
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

            // let gesture = gtk::GestureClick::new();
            //
            // gesture.set_button(gtk::gdk::ffi::GDK_BUTTON_PRIMARY as u32);
            // gesture.connect_pressed(|g, m, n, o| {
            //     g.set_state(gtk::EventSequenceState::Claimed);
            //     println!("clicked {} || {:?} || {:?}", m, n, o);
            //     list_item.
            //     g.set_state(gtk::EventSequenceState::Denied);
            // });
            //
            // label.add_controller(gesture);
        });

        // let cb_app_state = ;

        let single_selection_modal =
            gtk::SingleSelection::new(Some(LIST_VEC.into_iter().collect::<gtk::StringList>()));

        // single_selection_modal.connect_selection_changed(|m, p, n| {
        //     if let Some(item) = m.selected_item() {
        //         println!("act {:?} || {:?} || {:?}", item, p, n);
        //     }
        // });

        let list_view =
            gtk::ListView::new(Some(single_selection_modal), Some(signal_selection_factory));

        list_view.connect_activate(move |m, n| {
            println!("lv-act {:?} || {:?}", m, n);
        });
        //
        // let scroll_view = gtk::ScrolledWindow::builder()
        //     .vexpand(true)
        //     .child(&list_view)
        //     .build();

        // root.append(&scroll_view);

        let model = ActivityViewerModel {
            title: _init.title,
            list: list_view.clone(),
        };

        let the_list = &model.list;

        let widgets = view_output!();

        return relm4::ComponentParts { model, widgets };
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

// fn build_ui(app: &gtk::Application) {
//     log_display_info();
//
//     let layout_box = build_layout();
//     let window = gtk::ApplicationWindow::builder()
//         .application(app)
//         .title("Open Worship")
//         .child(&layout_box)
//         .build();
//
//     let close_action = gtk::gio::ActionEntry::builder("close")
//         // .activate(gtk::glib::clone!(
//         //     #[weak]
//         //     window,
//         //     move |_, _, _| window.close()
//         // ))
//         .build();
//
//     let action_group = gtk::gio::SimpleActionGroup::new();
//     action_group.add_action_entries([close_action]);
//
//     window.insert_action_group("custom-group", Some(&action_group));
//     app.set_accels_for_action("custom-group.close", &["<Ctrl>W"]);
//
//     if let Some(display_geometry) = get_display_geometry() {
//         window.set_default_width(display_geometry.width() / 2);
//         window.set_default_height(display_geometry.height() / 2);
//     }
//
//     window.connect_destroy(|_| std::process::exit(0));
//     return window.present();
// }

// struct ActivityState {
//     list: Vec<String>,
//     selected: Option<u32>,
// }
//
// impl ActivityState {
//     fn new() -> Self {
//         return ActivityState {
//             list: Vec::new(),
//             selected: None,
//         };
//     }
//
//     fn set_selected(&mut self, index: u32) {
//         if (index as usize) < self.list.len() {
//             self.selected = Some(index);
//         }
//     }
//
//     fn set_list(&mut self, list: Vec<String>, index: Option<u32>) {
//         let len = list.len();
//         self.list = list;
//
//         let inx = match index {
//             Some(inx) => inx,
//             None => {
//                 if len > 0 {
//                     self.selected = Some(0);
//                 } else {
//                     self.selected = None;
//                 }
//                 return ();
//             }
//         };
//
//         if (inx as usize) < len {
//             self.selected = Some(inx);
//         } else {
//             self.selected = None;
//         }
//     }
// }
//
// struct OWAppState {
//     preview_state: ActivityState,
//     live_state: ActivityState,
//     schedule_state: ActivityState,
// }
//
// impl OWAppState {
//     fn new() -> Self {
//         return OWAppState {
//             preview_state: ActivityState::new(),
//             live_state: ActivityState::new(),
//             schedule_state: ActivityState::new(),
//         };
//     }
// }
