mod app;
mod config;
mod db;
mod dto;
mod notepad;
mod parser;
mod services;
mod structs;
mod utils;
mod widgets;

fn main() {
    // app::run();
    notepad::init_app();
}
