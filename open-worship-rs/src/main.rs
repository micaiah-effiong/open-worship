use gtk::glib;
use gtk::prelude::*;

const APP_ID: &str = "com.open-worship";
const LIST_VEC: [&str; 5] = [
        "Lorem ipsum dolor sit amet, qui minim labore adipisicing minim sint cillum sint consectetur cupidatat.",
        "Lorem ipsum dolor sit amet, qui minim labore adipisicing minim sint cillum sint consectetur cupidatat.",
        "Lorem ipsum dolor sit amet, qui minim labore adipisicing minim sint cillum sint consectetur cupidatat.",
        "Lorem ipsum dolor sit amet, qui minim labore adipisicing minim sint cillum sint consectetur cupidatat.",
        "Lorem ipsum dolor sit amet, qui minim labore adipisicing minim sint cillum sint consectetur cupidatat. Lorem ipsum dolor sit amet, officia excepteur ex fugiat reprehenderit enim labore culpa sint ad nisi Lorem pariatur mollit ex esse exercitation amet. Nisi anim cupidatat excepteur officia. Reprehenderit nostrud nostrud ipsum Lorem est aliquip amet voluptate voluptate dolor minim nulla est proident. Nostrud officia pariatur ut officia. Sit irure elit esse ea nulla sunt ex occaecat reprehenderit commodo officia dolor Lorem duis laboris cupidatat officia voluptate. Culpa proident adipisicing id nulla nisi laboris ex in Lorem sunt duis officia eiusmod. Aliqua reprehenderit commodo ex non excepteur duis sunt velit enim. Voluptate laboris sint cupidatat ullamco ut ea consectetur et est culpa et culpa duis."
    ];

fn main() -> gtk::glib::ExitCode {
    let app = gtk::Application::builder().application_id(APP_ID).build();
    app.connect_startup(|_| load_css());
    app.connect_activate(build_ui);

    return app.run();
}

fn build_ui(app: &gtk::Application) {
    log_display_info();

    let layout_box = build_layout();
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Open Worship")
        .child(&layout_box)
        .build();

    let close_action = gtk::gio::ActionEntry::builder("close")
        .activate(gtk::glib::clone!(
            #[weak]
            window,
            move |_, _, _| window.close()
        ))
        .build();

    let action_group = gtk::gio::SimpleActionGroup::new();
    action_group.add_action_entries([close_action]);

    window.insert_action_group("custom-group", Some(&action_group));
    app.set_accels_for_action("custom-group.close", &["<Ctrl>W"]);

    if let Some(display_geometry) = get_display_geometry() {
        window.set_default_width(display_geometry.width() / 2);
        window.set_default_height(display_geometry.height() / 2);
    }

    window.connect_destroy(|_| std::process::exit(0));
    return window.present();
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
        .label("Preview")
        .margin_end(12)
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .build();

    button_2.set_css_classes(&["btn", "btn-blue"]);

    let number = std::rc::Rc::new(std::cell::Cell::new(-1));

    button_2.connect_clicked(gtk::glib::clone!(
        #[weak]
        number,
        #[weak]
        button,
        move |_| {
            number.set(number.get() + 1);
            let mut number_str = number.get().to_string();
            number_str.insert_str(0, "Live ");
            button.set_label(&number_str);
        }
    ));

    button.connect_clicked(move |btn| {
        gtk::glib::spawn_future_local(gtk::glib::clone!(
            #[weak]
            btn,
            #[weak]
            number,
            async move {
                number.set(number.get() + 1);
                let mut num_str = number.get().to_string();
                num_str.insert_str(0, "Live ");
                btn.set_label(&num_str);

                btn.set_sensitive(false);
                // glib::timeout_future_seconds(2).await;
                let wait_result = gtk::gio::spawn_blocking(move || {
                    let wait = std::time::Duration::from_secs(2);
                    std::thread::sleep(wait);
                    return true;
                })
                .await
                .expect("Blocking task must finish running");
                btn.set_sensitive(wait_result);
            }
        ));
    });

    let button_close_from_action_entry = gtk::Button::builder()
        .label("close")
        .margin_end(12)
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .build();

    button_close_from_action_entry.connect_clicked(gtk::glib::clone!(
        #[weak]
        button_close_from_action_entry,
        move |_| {
            button_close_from_action_entry
                .activate_action("custom-group.close", None)
                .expect("Should have close action")
        }
    ));

    header_box.append(&header_label);
    header_box.append(&header_space);
    header_box.append(&button);
    header_box.append(&button_2);
    header_box.append(&button_close_from_action_entry);
}

fn build_body_content(body_box: &gtk::Box) {
    let body_container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .hexpand_set(true)
        .vexpand_set(true)
        .homogeneous(true)
        .build();

    body_container.add_css_class("blue_box");
    build_activity_viewer(&body_container);
    build_search_and_preview(&body_container);

    body_box.set_homogeneous(true);
    body_box.add_css_class("red_box");
    body_box.set_vexpand(true);
    body_box.append(&body_container);
}

fn build_activity_viewer(content_box: &gtk::Box) {
    let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    container.set_homogeneous(true);
    container.add_css_class("green_box");

    let schedule_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();
    schedule_box.add_css_class("pink_box");
    let s_box_label = gtk::Label::builder().label("Schedule").build();
    schedule_box.append(&s_box_label);

    let preview_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();
    let preview_box_label = gtk::Label::builder().label("Preview").build();
    preview_box.append(&preview_box_label);
    preview_box.add_css_class("pink_box");

    let live_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .hexpand_set(true)
        .vexpand_set(true)
        .build();

    let live_box_label = gtk::Label::builder().label("Live").build();
    live_box.append(&live_box_label);
    live_box.add_css_class("pink_box");
    // live_box.append(&button_close_from_action_entry);
    build_live_activity_viewer(&live_box);
    build_live_activity_viewer(&preview_box);

    container.append(&schedule_box);
    container.append(&preview_box);
    container.append(&live_box);

    content_box.append(&container);
}

fn build_live_activity_viewer(container: &gtk::Box) {
    // let result_list_modal = gtk::gio::ListStore::new::<gtk::StringObject>();
    // result_list_modal.extend_from_slice(&list_vec);

    let signal_selection_factory = gtk::SignalListItemFactory::new();
    signal_selection_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::builder()
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .wrap_mode(gtk::pango::WrapMode::Word)
            .lines(2)
            .margin_top(12)
            .margin_bottom(12)
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

fn build_search_and_preview(container: &gtk::Box) {
    let content_box = gtk::Grid::new();
    content_box.set_column_homogeneous(true);
    content_box.set_row_homogeneous(true);
    container.append(&content_box);

    let search_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    {
        let tab_box = gtk::Box::new(gtk::Orientation::Horizontal, 3);
        tab_box.add_css_class("red_box");
        tab_box.set_height_request(48);

        let search_field_box = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        search_field_box.set_height_request(48);
        search_field_box.add_css_class("green_double_box");

        let search_input = gtk::SearchEntry::builder()
            .placeholder_text("Search...")
            .hexpand(true)
            .build();
        search_field_box.append(&search_input);

        let search_result_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        search_result_box.add_css_class("blue_box");
        search_result_box.set_vexpand(true);

        {
            let list_vec: gtk::StringList = (0..=3000).map(|num| num.to_string()).collect();
            // let result_list_modal = gtk::gio::ListStore::new::<gtk::StringObject>();
            // result_list_modal.extend_from_slice(&list_vec);

            let signal_selection_factory = gtk::SignalListItemFactory::new();
            signal_selection_factory.connect_setup(move |_, list_item| {
                let label = gtk::Label::new(None);
                label.set_ellipsize(gtk::pango::EllipsizeMode::End);

                list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Must be a list item")
                    .set_child(Some(&label));

                list_item
                    .property_expression("item")
                    .chain_property::<gtk::StringObject>("string")
                    .bind(&label, "label", gtk::Widget::NONE);
            });

            let single_selection_modal = gtk::SingleSelection::new(Some(list_vec));
            let list_view =
                gtk::ListView::new(Some(single_selection_modal), Some(signal_selection_factory));

            let scroll_view = gtk::ScrolledWindow::builder()
                .vexpand(true)
                .child(&list_view)
                .build();

            search_result_box.append(&scroll_view);
        }

        search_box.append(&tab_box);
        search_box.append(&search_field_box);
        search_box.append(&search_result_box);
    }

    let screen_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    {
        let preview_screen_box = gtk::Box::builder().build();
        preview_screen_box.add_css_class("brown_box");
        let live_screen_box = gtk::Box::builder().build();
        live_screen_box.add_css_class("brown_box");

        screen_box.set_homogeneous(true);
        screen_box.append(&preview_screen_box);
        screen_box.append(&live_screen_box);
    }

    search_box.add_css_class("yellow_box");
    screen_box.add_css_class("yellow_box");

    content_box.attach(&search_box, 0, 0, 1, 1);
    content_box.attach(&screen_box, 1, 0, 2, 1);
    content_box.add_css_class("green_double_box");
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
