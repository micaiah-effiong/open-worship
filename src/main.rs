mod app;
mod app_config;
mod application;
mod application_window;
mod db;
mod dto;
mod notepad;
mod parser;
mod services;
mod structs;
mod utils;
mod widgets;

fn main() {
    app::run();
    // notepad::init_app();
}
