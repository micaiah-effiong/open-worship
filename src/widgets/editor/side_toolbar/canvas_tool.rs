use gtk::gdk;
use gtk::glib::{self, subclass::types::ObjectSubclassIsExt};

use crate::utils;
use crate::{services::slide_manager::SlideManager, widgets::canvas::canvas_item::CanvasItem};

mod imp {
    use std::cell::RefCell;

    use gtk::{
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
        utils::{RGBExtra, WidgetExtrasExt},
    };

    #[derive(Default)]
    pub struct CanvasTool {
        // pub(super) slide_manager: glib::WeakRef<SlideManager>,
        pub(super) color_btn: RefCell<gtk::ColorDialogButton>,
        pub(super) transition_btn: RefCell<gtk::DropDown>,
        pub(super) add_image_btn: RefCell<gtk::Button>,
        pub(super) rm_image_btn: RefCell<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CanvasTool {
        const NAME: &'static str = "CanvasTool";
        type Type = super::CanvasTool;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for CanvasTool {}
    impl WidgetImpl for CanvasTool {}
    impl BoxImpl for CanvasTool {}

    impl CanvasTool {
        pub(super) fn build_canvas_section(&self, slide_manager: &SlideManager) {
            let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
            content.set_css_classes(&["small-pad"]);
            self.obj().append(&content);

            let transition_btn = Self::build_transition_dropdown(self, slide_manager);
            content.append(&transition_btn);
            self.transition_btn.replace(transition_btn);

            let color_btn = Self::build_color(self, slide_manager);
            content.append(&color_btn);
            self.color_btn.replace(color_btn);

            let bg_box = gtk::Box::builder().spacing(3).build();
            content.append(&bg_box);

            let bg_btn = Self::build_bg_image(self, slide_manager);
            bg_box.append(&bg_btn);
            self.add_image_btn.replace(bg_btn);

            let rm_bg_btn = Self::build_rmbg_image(self, slide_manager);
            bg_box.append(&rm_bg_btn);
            self.rm_image_btn.replace(rm_bg_btn);
        }
    }

    impl CanvasTool {
        fn build_transition_dropdown(_: &Self, sm: &SlideManager) -> gtk::DropDown {
            let transition_btn = gtk::DropDown::from_strings(&[
                "No Transition",
                "Crossfade",
                "SlideRight",
                "SlideLeft",
                "SlideUp",
                "SlideDown",
                "SlideLeftRight",
                "SlideUpDown",
                "OverUp",
                "OverDown",
                "OverLeft",
                "OverRight",
                "UnderUp",
                "UnderDown",
                "UnderLeft",
                "UnderRight",
                "OverUpDown",
                "OverDownUp",
                "OverLeftRight",
                "OverRightLeft",
                "RotateLeft",
                "RotateRight",
                "RotateLeftRight",
            ]);
            transition_btn.set_tooltip("Transition");

            transition_btn.connect_selected_item_notify({
                let sm = sm.clone();
                move |m| {
                    let item = m.selected_item();
                    let Some(str_obj) = item.and_downcast::<gtk::StringObject>() else {
                        return;
                    };
                    let transition_type = str_obj.string().to_string();

                    let transition = match transition_type.as_str() {
                        "None" => gtk::StackTransitionType::None,
                        "Crossfade" => gtk::StackTransitionType::Crossfade,
                        "SlideRight" => gtk::StackTransitionType::SlideRight,
                        "SlideLeft" => gtk::StackTransitionType::SlideLeft,
                        "SlideUp" => gtk::StackTransitionType::SlideUp,
                        "SlideDown" => gtk::StackTransitionType::SlideDown,
                        "SlideLeftRight" => gtk::StackTransitionType::SlideLeftRight,
                        "SlideUpDown" => gtk::StackTransitionType::SlideUpDown,
                        "OverUp" => gtk::StackTransitionType::OverUp,
                        "OverDown" => gtk::StackTransitionType::OverDown,
                        "OverLeft" => gtk::StackTransitionType::OverLeft,
                        "OverRight" => gtk::StackTransitionType::OverRight,
                        "UnderUp" => gtk::StackTransitionType::UnderUp,
                        "UnderDown" => gtk::StackTransitionType::UnderDown,
                        "UnderLeft" => gtk::StackTransitionType::UnderLeft,
                        "UnderRight" => gtk::StackTransitionType::UnderRight,
                        "OverUpDown" => gtk::StackTransitionType::OverUpDown,
                        "OverDownUp" => gtk::StackTransitionType::OverDownUp,
                        "OverLeftRight" => gtk::StackTransitionType::OverLeftRight,
                        "OverRightLeft" => gtk::StackTransitionType::OverRightLeft,
                        "RotateLeft" => gtk::StackTransitionType::RotateLeft,
                        "RotateRight" => gtk::StackTransitionType::RotateRight,
                        "RotateLeftRight" => gtk::StackTransitionType::RotateLeftRight,
                        _ => gtk::StackTransitionType::None,
                    };

                    let Some(slide) = sm.current_slide() else {
                        return;
                    };
                    slide.set_transition(transition);
                }
            });

            transition_btn
        }

        fn build_color(_: &Self, sm: &SlideManager) -> gtk::ColorDialogButton {
            let color_btn = gtk::ColorDialogButton::new(Some(gtk::ColorDialog::new()));
            color_btn.set_tooltip("Background color");
            color_btn.set_rgba(&gtk::gdk::RGBA::parse("#383E41").expect("valid color"));

            color_btn.connect_rgba_notify({
                let sm = sm.clone();
                move |c| {
                    let Some(canvas) = sm.current_slide().and_then(|v| v.canvas()) else {
                        return;
                    };

                    canvas.set_background_color(c.rgba().to_hex());
                    canvas.style();
                }
            });

            color_btn
        }

        fn build_bg_image(_: &Self, sm: &SlideManager) -> gtk::Button {
            let image_btn = gtk::Button::builder()
                .icon_name("picture")
                .tooltip_text("Background image")
                .hexpand(true)
                .build();

            image_btn.connect_clicked({
                let sm = sm.clone();
                move |btn| {
                    let win = btn.toplevel_window();
                    let Some(image_file) = FileManager::open_image(win.as_ref()) else {
                        return;
                    };

                    let Some(path) =
                        FileManager::file_to_link(&image_file, AppConfigDir::SlideMedia)
                    else {
                        return;
                    };

                    let Some(canvas) = sm.current_slide().and_then(|v| v.canvas()) else {
                        return;
                    };

                    canvas.set_background_pattern(path);
                    canvas.style();
                }
            });

            image_btn
        }

        fn build_rmbg_image(_: &Self, sm: &SlideManager) -> gtk::Button {
            let remove_image_btn = gtk::Button::builder()
                .icon_name("remove-picture")
                .tooltip_text("Remove background image")
                .hexpand(true)
                .build();

            remove_image_btn.connect_clicked({
                let sm = sm.clone();
                move |_| {
                    if let Some(canvas) = sm.current_slide().and_then(|v| v.canvas()) {
                        canvas.set_background_pattern("");
                        canvas.style();
                    };
                }
            });

            remove_image_btn
        }
    }
}

glib::wrapper! {
pub struct CanvasTool(ObjectSubclass<imp::CanvasTool>)
        @extends  gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for CanvasTool {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl CanvasTool {
    pub fn new(sm: &SlideManager) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().build_canvas_section(sm);

        obj
    }

    pub fn update_props(&self, _: &CanvasItem, sm: &SlideManager) {
        let imp = self.imp();
        let Some(slide) = sm.current_slide() else {
            return;
        };

        let Some(c) = slide.canvas() else {
            return;
        };

        if let Ok(color) = gdk::RGBA::parse(c.background_color()) {
            imp.color_btn.borrow().set_rgba(&color);
        }

        imp.transition_btn
            .borrow()
            .set_selected(utils::transition_to_int(slide.transition()));
    }
}
