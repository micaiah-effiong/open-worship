use crate::application::MainApplication;
use crate::utils::setup_theme_listener;

use gtk::prelude::*;

pub fn run() {
    gtk::init().expect("Could not initialize gtk");

    app_init();
    log_display_info();
    let app = MainApplication::default();
    app.run();
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
}

fn app_init() {
    gtk::gio::resources_register_include!("resources.gresource")
        .expect("could not find app resources");

    setup_theme_listener();
    match gtk::glib::setenv("GTK_CSD", "0", false) {
        Ok(_) => (),
        Err(e) => {
            println!("An error occured while setting GTK_CSD:\n{:?}", e);
        }
    };
}
