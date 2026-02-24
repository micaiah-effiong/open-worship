use std::cell::OnceCell;
use std::cell::RefCell;

use gtk::gio;
use gtk::glib;

use gsettings_macro::gen_settings;

#[gen_settings(
    file = "./data/resources/com.openworship.app.gschema.xml",
    id = "com.openworship.app"
)]
pub struct ApplicationSettings;

impl ApplicationSettings {
    pub fn get_instance() -> Self {
        SINGLETON.with(|c| {
            let settings = c.get_or_init(|| Self::default());
            settings.clone()
        })
    }
}

thread_local! {
static SINGLETON: OnceCell<ApplicationSettings> = const { OnceCell::new() };
}
