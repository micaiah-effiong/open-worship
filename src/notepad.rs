use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk::gio::prelude::{ApplicationExt, ApplicationExtManual};
use gtk::glib::collections::slist;
use gtk::glib::object::IsA;
use gtk::glib::prelude::*;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::{
    AccessibleExt, BoxExt, ButtonExt, GtkApplicationExt, GtkWindowExt, TextBufferExt, TextViewExt,
    WidgetExt,
};
use gtk::subclass::window;
use gtk::{CssProvider, pango};
use relm4::{RelmRemoveAllExt, RelmWidgetExt};
use serde::Serialize;

use crate::services::slide::Slide;
use crate::services::slide_manager::SlideManager;
use crate::utils::WidgetChildrenExt;
use crate::widgets::canvas::canvas_item::{CanvasItem, CanvasItemExt};
use crate::widgets::canvas::serialise::{SlideData, SlideManagerData};
use crate::widgets::canvas::text_item::{self, TextItem};
use crate::widgets::{self, canvas, search};

pub fn init_app() {
    let _ = gtk::init();

    const APP_ID: &str = "com.openworship.app";
    const RESOURECE_PATH: &str = "/com/openworship/app";
    {
        // if let Some(g_settings) = gtk::Settings::default() {
        //     g_settings.set_gtk_application_prefer_dark_theme(true);
        // }

        //
        gtk::glib::set_application_name("Open worship");
        gtk::gio::resources_register_include!("resources.gresource")
            .expect("could not find app resources");

        let provider = gtk::CssProvider::new();
        provider.load_from_resource("/com/openworship/app/styles/style.css");

        if let Some(display) = gtk::gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
        match gtk::glib::setenv("GTK_CSD", "0", false) {
            Ok(_) => (),
            Err(e) => {
                println!("An error occured while setting GTK_CSD:\n{:?}", e);
            }
        };
    }

    let app = gtk::Application::new(Some(APP_ID), gtk::gio::ApplicationFlags::FLAGS_NONE);
    app.set_resource_base_path(Some(RESOURECE_PATH));

    // app.connect_activate(build_ui);
    app.connect_activate(|app| {
        {
            let win = search::songs::edit_modal::SongEditWindow::new();
            app.add_window(&win);
            win.show(None);
            println!("PRESENT");
            // SongEditWindow::new().present();
        }

        // build_ui(&app);
        // build_dnd_ui(&app);
    });

    app.run();
}

fn build_ui(app: &gtk::Application) {
    let app_window = gtk::ApplicationWindow::new(app);
    app_window.set_size_request(500, 500);
    app_window.set_default_width(500);
    app_window.set_default_height(500);

    let v_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let aspect_frame = gtk::AspectFrame::new(0.5, 0.5, 16.0 / 9.0, false);
    v_box.append(&aspect_frame);

    let load_data = r##"{
        "transition": 3,
        "items": [
            {
                "x": -602,
                "y": -16,
                "w": 2710,
                "h": 1529,
                "type": "text",
                "text-data": "UGFnZSAx",
                "font": "Open Sans",
                "font-size": 16,
                "font-style": "normal",
                "justification": 1,
                "align": 1,
                "color": "#fff",
                "text-underline": false,
                "text-outline": false,
                "text-shadow": true
            }
        ],
        "preview": "",
        "background-color": "#383E41",
        "background-pattern": ""
    }"##;
    let _slide_data: SlideData =
        serde_json::from_str(load_data).expect("Could not parse load_data");

    let sm = {
        // let slide = Slide::new(serde_json::from_str(load_data).ok());
        // slide.set_presentation_mode(false);
        // let mut slides = Vec::new();
        // slides.push(slide.clone());
        let sm = SlideManager::new();
        sm.load_data(SlideManagerData::new(0, 0, [_slide_data]));
        for s in sm.slides() {
            s.set_presentation_mode(true);
            println!("mode {}", s.presentation_mode());
        }
        sm
    };

    // let slide = Slide::new(None);
    // slide.add_item(
    //     TextItem::new(slide.canvas().as_ref(), None).upcast::<CanvasItem>(),
    //     false,
    //     false,
    // );
    // let canvas = slide.canvas().expect("No canvas found").clone();
    // slide.load_slide();

    // for i in _slide_data.items {
    //     let textitem = widgets::canvas::text_item::TextItem::new(Some(&canvas), Some(i));
    //
    //     slide.add_item(
    //         textitem.upcast::<canvas::canvas_item::CanvasItem>(),
    //         false,
    //         false,
    //     );
    // }

    let serialize_btn = gtk::Button::with_label("Serialize");
    let label = gtk::Label::new(Some("No data"));
    v_box.prepend(&serialize_btn);
    v_box.prepend(&label);
    let tv = gtk::TextView::new();
    v_box.prepend(&tv);

    serialize_btn.connect_clicked({
        // let canvas = canvas.clone();
        // let slide = slide.clone();
        // let label = label.clone();
        let tv = tv.clone();
        let sm = sm.clone();
        move |_| {
            let slide_data = sm.serialise();

            let mut buf = Vec::new();
            let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
            let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);

            let Ok(_) = slide_data.serialize(&mut ser) else {
                println!("failed to serialize");
                return;
            };

            let text = String::from_utf8(buf).unwrap_or("failed".to_string());
            // label.set_text(&text);
            tv.buffer().set_text(&text);

            //
        }
    });

    let buff_tv = gtk::TextView::new();
    sm.connect_current_slide_changed({
        let buff_tv = buff_tv.clone();
        move |slide| {
            if let Some(canvas) = slide.canvas() {
                for t in canvas.widget().get_children::<TextItem>() {
                    let buff = t.buffer();
                    buff_tv.set_buffer(Some(&buff));

                    // assert_eq!(buff, buff_tv.buffer());
                    println!(
                        ">>> change <<< \nbuff {:?}\nbuff_tv {:?}",
                        buff,
                        buff_tv.buffer()
                    );
                    break;
                }
            }
        }
    });

    let t_box = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    {
        let new_slide = gtk::Button::with_label("New slide");
        new_slide.connect_clicked({
            let sm = sm.clone();
            move |_| {
                sm.make_new_slide();
                sm.request_new_item(canvas::CanvasItemType::TEXT);
            }
        });

        let next_slide = gtk::Button::with_label("Next slide");
        next_slide.connect_clicked({
            let sm = sm.clone();
            move |_| sm.next_slide()
        });

        let previous_slide = gtk::Button::with_label("Previous slide");
        previous_slide.connect_clicked({
            let sm = sm.clone();
            move |_| sm.previous_slide()
        });

        t_box.append(&new_slide);
        t_box.append(&next_slide);
        t_box.append(&previous_slide);

        let add_window = gtk::Button::with_label("Live");
        add_window.connect_clicked({
            let app = app.downgrade().clone();
            let sm = sm.clone();

            move |_| {
                let Some(app) = app.upgrade() else {
                    println!("App Upgrade failed");
                    return;
                };

                let window = add_app_window();
                let aspect_frame = gtk::AspectFrame::new(0.5, 0.5, 16.0 / 9.0, false);

                app.add_window(&window);

                let load_window_slide = {
                    let window = aspect_frame.clone();
                    move |slide: &Slide| {
                        //
                        let data = slide.serialise();

                        let slide = Slide::new(Some(data));
                        slide.set_presentation_mode(true);
                        slide.load_slide();
                        window.remove_all();
                        window.set_child(slide.canvas().as_ref());
                    }
                };

                if let Some(s) = sm.current_slide() {
                    load_window_slide(&s);
                }
                sm.connect_current_slide_changed(load_window_slide);

                window.set_child(Some(&aspect_frame));
                window.present();
            }
        });
        t_box.append(&add_window);
    }

    v_box.append(&t_box);
    v_box.append(&buff_tv);

    // sm.new_slide(None, false);
    // sm.next_slide();
    aspect_frame.set_child(Some(&sm.slideshow()));

    app_window.set_child(Some(&v_box));
    app_window.present();
}

fn add_app_window() -> gtk::Window {
    let window = gtk::Window::new();
    let size = 500.0;

    window.set_width_request(size as i32);
    window.set_height_request((size / (16.0 / 9.0)) as i32);

    window.set_decorated(false);
    // window.fullscreen_on_monitor(monitor);
    // window.fullscreen();

    window
}

fn build_dnd_ui(app: &gtk::Application) {
    // Shared state for dragged item
    let dragged_item: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    // === SOURCE: Draggable buttons ===
    let source_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    source_box.set_margin_start(12);
    source_box.set_margin_end(12);
    source_box.set_margin_top(12);
    source_box.set_margin_bottom(12);

    let source_label = gtk::Label::new(Some("Drag these items:"));
    source_box.append(&source_label);

    let items = vec!["Item A", "Item B", "Item C"];
    for item_text in items {
        let button = gtk::Button::with_label(item_text);
        button.set_size_request(150, 40);

        // Setup drag source
        let drag_source = gtk::DragSource::new();
        drag_source.set_actions(gtk::gdk::DragAction::COPY);

        let item_text_clone = item_text.to_string();
        drag_source.connect_prepare(move |_source, _x, _y| {
            let content = gtk::gdk::ContentProvider::for_value(&item_text_clone.to_value());
            Some(content)
        });

        drag_source.connect_drag_begin(move |_source, drag| {
            let item_text = item_text.to_string();
            // drag.set_icon_name(Some("document-properties"), 0, 0);
        });

        button.add_controller(drag_source);
        source_box.append(&button);
    }

    let source_scroll = gtk::ScrolledWindow::new();
    source_scroll.set_child(Some(&source_box));
    source_scroll.set_vexpand(true);
    source_scroll.set_hexpand(true);

    // === TARGET: Drop zone ===
    let target_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    target_box.set_margin_start(12);
    target_box.set_margin_end(12);
    target_box.set_margin_top(12);
    target_box.set_margin_bottom(12);

    let target_label = gtk::Label::new(Some("Drop items here:"));
    target_box.append(&target_label);

    let drop_zone = gtk::Box::new(gtk::Orientation::Vertical, 8);
    drop_zone.set_css_classes(&["drop-zone"]);
    drop_zone.set_size_request(200, 300);

    let zone_label = gtk::Label::new(Some("Drop zone (empty)"));
    zone_label.set_css_classes(&["drop-hint"]);
    drop_zone.append(&zone_label);

    target_box.append(&drop_zone);

    // Setup drop target
    let drop_target = gtk::DropTarget::new(gtk::glib::Type::STRING, gtk::gdk::DragAction::COPY);

    drop_target.connect_drop({
        let drop_zone = drop_zone.clone();
        move |_target, value, _x, _y| {
            if let Ok(text) = value.get::<String>() {
                let item_button = gtk::Button::with_label(&text);
                item_button.set_size_request(150, 40);

                // Remove label if this is first item
                if drop_zone.first_child().is_some() {
                    if let Some(first) = drop_zone.first_child() {
                        if first.downcast_ref::<gtk::Label>().is_some() {
                            drop_zone.remove(&first);
                        }
                    }
                }

                drop_zone.append(&item_button);
                return true;
            }
            false
        }
    });

    drop_target.connect_motion(move |_target, _x, _y| gtk::gdk::DragAction::COPY);
    drop_target.connect_leave(move |_target| {});

    drop_zone.add_controller(drop_target);

    let target_scroll = gtk::ScrolledWindow::new();
    target_scroll.set_child(Some(&target_box));
    target_scroll.set_vexpand(true);
    target_scroll.set_hexpand(true);

    // === Main layout ===
    let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
    paned.set_start_child(Some(&source_scroll));
    paned.set_end_child(Some(&target_scroll));
    paned.set_shrink_start_child(false);
    paned.set_shrink_end_child(false);

    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    main_box.append(&paned);

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Drag and Drop Example")
        .default_width(600)
        .default_height(400)
        .child(&main_box)
        .build();

    // Add CSS for styling
    let provider = gtk::CssProvider::new();
    provider.load_from_data(
        r#"
        .drop-zone {
            border: 2px dashed #ccc;
            border-radius: 8px;
            background-color: #f5f5f5;
            padding: 8px;
        }
        
        .drop-zone:drop(active) {
            border-color: #4CAF50;
            background-color: #e8f5e9;
        }
        
        .drop-hint {
            color: #999;
            font-style: italic;
        }
        "#,
    );

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().unwrap(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    window.present();
}
