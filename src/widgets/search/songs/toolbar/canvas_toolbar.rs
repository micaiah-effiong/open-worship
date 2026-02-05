use gtk::glib::{self, subclass::types::ObjectSubclassIsExt};

use crate::services::slide_manager::SlideManager;

mod imp {
    use std::cell::{Cell, RefCell};

    use gtk::{
        gdk,
        glib::{
            self,
            object::{Cast, CastNone},
            subclass::{
                object::ObjectImpl,
                types::{ObjectSubclass, ObjectSubclassExt},
            },
        },
        prelude::{BoxExt, ButtonExt, WidgetExt},
        subclass::{box_::BoxImpl, widget::WidgetImpl},
    };

    use crate::{
        config::AppConfigDir,
        services::{file_manager::FileManager, slide_manager::SlideManager},
        utils::{self, ColorDialogButtonExtra, WidgetExtrasExt},
    };

    #[derive(Debug, Default)]
    pub struct CanvasToolbar {
        pub slide_manager: glib::WeakRef<SlideManager>,
        //
        pub color: RefCell<gtk::ColorDialogButton>,
        pub transition: RefCell<gtk::DropDown>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CanvasToolbar {
        const NAME: &'static str = "CanvasToolbar";
        type Type = super::CanvasToolbar;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for CanvasToolbar {}
    impl WidgetImpl for CanvasToolbar {}
    impl BoxImpl for CanvasToolbar {}

    impl CanvasToolbar {
        fn build_transition_btn(&self) -> gtk::DropDown {
            let transition_btn = gtk::DropDown::from_strings(&[
                "None",
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
            self.transition.replace(transition_btn.clone());
            transition_btn.set_tooltip("Slide transition");

            if let Some(t) = transition_btn
                .first_child()
                .and_downcast::<gtk::ToggleButton>()
            {
                t.set_css_classes(&["flat"]);
            }

            transition_btn.connect_selected_item_notify({
                let sm = self.slide_manager.clone();
                move |m| {
                    let Some(sm) = sm.upgrade() else {
                        return;
                    };

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

        pub(super) fn build_canvas_toolbar(&self) {
            let toolbar = self.obj();
            toolbar.set_height_request(35);
            toolbar.set_spacing(8);
            toolbar.set_widget_name("text-toolbar-box");
            toolbar.set_css_classes(&["edit-toolbar-box"]);
            toolbar.set_margin_all(6);

            let Some(sm) = self.slide_manager.upgrade() else {
                return;
            };

            toolbar.append(&self.build_transition_btn());

            let color_btn = gtk::ColorDialogButton::new(Some(gtk::ColorDialog::new()));
            color_btn.set_tooltip("Background color");
            {
                self.color.replace(color_btn.clone());
                color_btn.set_rgba(&gtk::gdk::RGBA::new(255.0, 255.0, 255.0, 1.0));
                toolbar.append(&color_btn);
                if let Some(btn) = color_btn.first_child().and_downcast::<gtk::Button>() {
                    btn.add_css_class("flat");
                }
                color_btn.connect_rgba_notify({
                    let sm = sm.clone();
                    move |c| {
                        let Some(canvas) = sm.current_slide().and_then(|v| v.canvas()) else {
                            return;
                        };

                        canvas.set_background_color(c.hex());
                        canvas.style();
                    }
                });
            }

            let image_btn = gtk::Button::builder().icon_name("picture").build();
            image_btn.set_tooltip("Background image");
            {
                toolbar.append(&image_btn);
                image_btn.add_css_class("flat");
                image_btn.connect_clicked({
                    let sm = sm.clone();
                    move |_| {
                        let Some(image_file) = FileManager::open_image() else {
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
            }

            let remove_image_btn = gtk::Button::builder().icon_name("close").build();
            remove_image_btn.set_tooltip("Remove background image");
            {
                toolbar.append(&remove_image_btn);
                remove_image_btn.add_css_class("flat");
                remove_image_btn.connect_clicked({
                    let sm = self.slide_manager.clone();
                    move |_| {
                        let Some(sm) = sm.upgrade() else {
                            return;
                        };
                        let Some(canvas) = sm.current_slide().and_then(|v| v.canvas()) else {
                            return;
                        };

                        canvas.set_background_pattern("");
                        canvas.style();
                    }
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

            if let Ok(color) = gdk::RGBA::parse(&c.background_color()) {
                self.color.borrow().set_rgba(&color);
            }

            self.transition
                .borrow()
                .set_selected(utils::transition_to_int(slide.transition()));
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
        obj.imp().slide_manager.set(Some(&slide_manager));

        slide_manager.connect_current_slide_changed(glib::clone!(
            #[weak]
            obj,
            move |_, _| obj.imp().update_props()
        ));

        obj.imp().build_canvas_toolbar();

        obj
    }
}
