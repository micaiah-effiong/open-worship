use gtk::{
    glib::{self, subclass::types::ObjectSubclassIsExt},
    prelude::BoxExt,
};

use crate::{
    services::slide_manager::SlideManager,
    widgets::search::songs::toolbar::{canvas_toolbar::CanvasToolbar, text_toolbar::TextToolbar},
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
    pub struct SongEditorToolbar {
        pub slide_manager: RefCell<SlideManager>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SongEditorToolbar {
        const NAME: &'static str = "SongEditorToolbar";
        type Type = super::SongEditorToolbar;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for SongEditorToolbar {}
    impl WidgetImpl for SongEditorToolbar {}
    impl BoxImpl for SongEditorToolbar {}
}

glib::wrapper! {
    pub struct SongEditorToolbar(ObjectSubclass<imp::SongEditorToolbar>)
        @extends  gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for SongEditorToolbar {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl SongEditorToolbar {
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
