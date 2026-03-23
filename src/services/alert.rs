use gobject_macro::gobject_props;
use gtk;
use gtk::glib;
use gtk::glib::subclass::types::ObjectSubclassIsExt;

#[gobject_props]
struct Alert {
    pub name: String,
    pub message: String,
    pub count: u32,
    pub active: bool,
    id: u32,
}

impl Default for Alert {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl Alert {
    pub fn id(&self) -> u32 {
        self.imp().id.borrow().clone()
    }
    pub fn set_id(&self, value: u32) {
        self.imp().id.replace(value);
    }
}
