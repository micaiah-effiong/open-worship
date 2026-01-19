use std::cell::Cell;
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
        //
        gtk::glib::set_application_name("Open worship");
        gtk::gio::resources_register_include!("resources.gresource")
            .expect("could not find app resources");

        let provider = gtk::CssProvider::new();
        provider.load_from_resource("/com/openworship/app/style.css");

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
    // window.fullscreen();

    window
}
