use gtk::{
    glib::{self, subclass::types::ObjectSubclassIsExt},
    prelude::BoxExt,
};

use crate::{
    services::slide_manager::SlideManager,
    widgets::editor::{canvas_toolbar::CanvasToolbar, text_toolbar::TextToolbar},
};

mod imp {
    use std::cell::RefCell;

    use gtk::{
        glib::{
            self,
            subclass::{object::ObjectImpl, types::ObjectSubclass},
        },
        subclass::{box_::BoxImpl, widget::WidgetImpl},
    };

    use crate::services::slide_manager::SlideManager;

    #[derive(Debug, Default)]
    pub struct EditorToolbar {
        pub slide_manager: RefCell<SlideManager>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EditorToolbar {
        const NAME: &'static str = "EditorToolbar";
        type Type = super::EditorToolbar;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for EditorToolbar {}
    impl WidgetImpl for EditorToolbar {}
    impl BoxImpl for EditorToolbar {}
}

glib::wrapper! {
    pub struct EditorToolbar(ObjectSubclass<imp::EditorToolbar>)
        @extends  gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for EditorToolbar {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl EditorToolbar {
    pub fn new(slide_manager: &SlideManager) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().slide_manager.replace(slide_manager.clone());

        let text_toolbar = TextToolbar::new(slide_manager);
        let canvas_toolbar = CanvasToolbar::new(slide_manager);
        let spacer = gtk::Box::builder().hexpand(true).build();

        let base = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        base.append(&text_toolbar);
        base.append(&spacer);
        base.append(&canvas_toolbar);

        obj.append(&base);

        obj
    }
}
