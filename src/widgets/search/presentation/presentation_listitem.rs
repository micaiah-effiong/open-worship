use gtk::glib;
use gtk::glib::object::{Cast, ObjectExt};
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::subclass::prelude::ObjectSubclass;

use crate::widgets::canvas::serialise::SlideManagerData;
use crate::widgets::search::presentation::PresentationObj;

mod imp {
    use std::cell::RefCell;

    use gtk::{
        glib::subclass::{
            object::{ObjectImpl, ObjectImplExt},
            types::ObjectSubclassExt,
        },
        prelude::{BoxExt, OrientableExt, WidgetExt},
        subclass::{box_::BoxImpl, widget::WidgetImpl},
    };

    use super::*;

    #[derive(Default)]
    pub struct PresentationListItem {
        pub(super) pic: RefCell<gtk::Picture>,
        pub(super) label: RefCell<gtk::Label>,
        pub(super) bindings: RefCell<Vec<glib::Binding>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PresentationListItem {
        const NAME: &'static str = "PresentationListItem";
        type Type = super::PresentationListItem;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for PresentationListItem {
        fn constructed(&self) {
            self.parent_constructed();

            let b = self.obj().clone();
            b.set_orientation(gtk::Orientation::Horizontal);
            b.set_spacing(4);

            let label = self.label.borrow().clone();
            label.set_xalign(0.0);

            let pic = self.pic.borrow().clone();
            pic.set_height_request(40);
            pic.set_css_classes(&["bg-preview-box"]);

            b.append(&pic);
            b.append(&label);
        }
    }
    impl WidgetImpl for PresentationListItem {}
    impl BoxImpl for PresentationListItem {}
}

glib::wrapper! {
    pub struct PresentationListItem(ObjectSubclass<imp::PresentationListItem>)
        @extends  gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for PresentationListItem {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl PresentationListItem {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn bind(&self, data: &PresentationObj) {
        let mut bindings = self.imp().bindings.borrow_mut();

        let binding = data
            .bind_property("data", &self.imp().label.borrow().clone(), "label")
            .transform_to(|_, data: &SlideManagerData| Some(data.title.clone()))
            .sync_create()
            .build();
        bindings.push(binding);

        let binding = data
            .bind_property("data", &self.imp().pic.borrow().clone(), "paintable")
            .transform_to(|_, data: &SlideManagerData| {
                let slide = data.slides.first()?;

                gtk::gdk::Texture::from_bytes(&glib::Bytes::from(&slide.preview.clone()))
                    .ok()
                    .map(|t| t.upcast::<gtk::gdk::Paintable>())
            })
            .sync_create()
            .build();
        bindings.push(binding);
    }

    pub fn unbind(&self) {
        for item in self.imp().bindings.borrow_mut().drain(..) {
            item.unbind();
        }
    }
}
