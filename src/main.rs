#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;
mod app_config;
mod application;
mod application_window;
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
    // let p = parser::parser::Parser::parse(
    //     "Let it shine John 3:2,5; 1 Peter 2:3-10; John 1:1-3,5,7-10".into(),
    // );
    // println!("{:?}", p);

    app::run();
    // notepad::init_app();
}
