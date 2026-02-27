use gtk::{self, glib};

//
mod imp {
    use std::cell::Cell;

    use gtk::glib::{
        self, Properties,
        subclass::{object::ObjectImpl, types::ObjectSubclass},
    };

    use gtk::prelude::ObjectExt;
    use gtk::subclass::prelude::DerivedObjectProperties;

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::IntegerObject)]
    pub struct IntegerObject {
        #[property(get, set, construct)]
        number: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IntegerObject {
        const NAME: &'static str = "IntegerObject";
        type Type = super::IntegerObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for IntegerObject {}
}

glib::wrapper! {
    pub struct IntegerObject(ObjectSubclass<imp::IntegerObject>);
}

impl IntegerObject {
    pub fn new(number: u32) -> Self {
        glib::Object::builder().property("number", number).build()
    }
}
