use gtk::glib::{self, subclass::types::ObjectSubclassIsExt};

use crate::services::slide_manager::SlideManager;

mod imp {
    use std::cell::RefCell;

    use adw::subclass::prelude::ObjectImplExt;
    use gtk::{
        gdk,
        glib::{
            self,
            object::CastNone,
            subclass::{
                object::ObjectImpl,
                types::{ObjectSubclass, ObjectSubclassExt},
            },
        },
        prelude::{BoxExt, ButtonExt, WidgetExt},
        subclass::{box_::BoxImpl, widget::WidgetImpl},
    };

    use crate::{
        app_config::AppConfigDir,
        services::{file_manager::FileManager, slide_manager::SlideManager},
        utils::{self, RGBExtra, WidgetExtrasExt},
    };

    #[derive(Debug, Default)]
    pub struct CanvasToolbar {
        pub slide_manager: glib::WeakRef<SlideManager>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CanvasToolbar {
        const NAME: &'static str = "CanvasToolbar";
        type Type = super::CanvasToolbar;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for CanvasToolbar {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_css_classes(&["toolbar"]);
        }
    }
    impl WidgetImpl for CanvasToolbar {}
    impl BoxImpl for CanvasToolbar {}

    impl CanvasToolbar {
        pub(super) fn build_canvas_toolbar(&self) {
            let toolbar = self.obj();
            toolbar.set_height_request(35);
            toolbar.set_spacing(8);
            toolbar.set_widget_name("canvas-toolbar-box");
            toolbar.set_margin_all(6);

            let Some(sm) = self.slide_manager.upgrade() else {
                return;
            };

            let undo_btn = gtk::Button::builder().icon_name("edit-undo").build();
            undo_btn.set_tooltip("Undo");
            {
                toolbar.append(&undo_btn);
                undo_btn.add_css_class("flat");
                undo_btn.connect_clicked({
                    let sm = sm.clone();
                    move |_| sm.history_manager().undo()
                });
            }

            let redo_btn = gtk::Button::builder().icon_name("edit-redo").build();
            redo_btn.set_tooltip("Redo");
            {
                toolbar.append(&redo_btn);
                redo_btn.add_css_class("flat");
                redo_btn.connect_clicked({
                    let sm = sm.clone();
                    move |_| sm.history_manager().redo()
                });
            }
        }

        pub(super) fn update_props(&self) {
            let Some(sm) = self.slide_manager.upgrade() else {
                return;
            };

            let Some(slide) = sm.current_slide() else {
                return;
            };
            let Some(c) = slide.canvas() else {
                return;
            };
        }
    }
}

glib::wrapper! {
    pub struct CanvasToolbar(ObjectSubclass<imp::CanvasToolbar>)
        @extends  gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for CanvasToolbar {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl CanvasToolbar {
    pub fn new(slide_manager: &SlideManager) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().slide_manager.set(Some(slide_manager));

        slide_manager.connect_current_slide_changed(glib::clone!(
            #[weak]
            obj,
            move |_, _| obj.imp().update_props()
        ));

        obj.imp().build_canvas_toolbar();

        obj
    }
}
