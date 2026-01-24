use gtk::{
    glib::{self, subclass::types::ObjectSubclassIsExt},
    prelude::BoxExt,
};

use crate::services::slide_manager::SlideManager;

mod views {
    pub(super) const TEXT: &str = "text";
    pub(super) const CANVAS: &str = "canvas";
}

mod imp {
    use std::cell::RefCell;

    use gtk::{
        glib::{
            self,
            object::CastNone,
            subclass::{object::ObjectImpl, types::ObjectSubclass},
        },
        prelude::{BoxExt, ButtonExt, ToggleButtonExt, WidgetExt},
        subclass::{box_::BoxImpl, widget::WidgetImpl},
    };
    use relm4::RelmWidgetExt;

    use crate::{
        config::AppConfigDir,
        services::{file_manager::FileManager, slide_manager::SlideManager},
        widgets::canvas::canvas_item::CanvasItemExt,
    };

    #[derive(Debug, Default)]
    pub struct SongEditorToolbar {
        pub slide_manager: RefCell<SlideManager>,
        pub stack: RefCell<gtk::Stack>,
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

    impl SongEditorToolbar {
        pub(super) fn build_text_toolbar(&self) -> gtk::Box {
            let toolbar = gtk::Box::builder()
                .height_request(35)
                .spacing(8)
                .name("toolbar-box")
                .css_classes([/* "brown_box", */ "edit-toolbar-box"])
                .build();

            toolbar.set_margin_all(6);

            let font_btn = gtk::FontDialogButton::new(Some(gtk::FontDialog::new()));
            {
                font_btn.set_tooltip("Font family");
                toolbar.append(&font_btn);
                toolbar.append(&self.build_font_size_btn());
                font_btn.set_level(gtk::FontLevel::Family);
                if let Some(btn) = font_btn.first_child().and_downcast::<gtk::Button>() {
                    btn.add_css_class("flat");
                }

                let mut font_desc = gtk::pango::FontDescription::new();
                font_desc.set_family("Tahoma");
                font_btn.set_font_desc(&font_desc);

                font_btn.connect_font_desc_notify({
                    let sm = self.slide_manager.borrow().clone();
                    move |f| {
                        let Some(d) = f.font_desc() else {
                            return;
                        };
                        println!("F {:?}", d.family());
                        let Some(ti) = sm.current_slide().and_then(|v| v.text_item()) else {
                            return;
                        };

                        if let Some(family) = d.family() {
                            ti.set_font(family);
                        }
                        ti.style();
                    }
                });
            }

            /* let superscript_btn = gtk::ToggleButton::new();
            {
                wrapper.append(&superscript_btn);
                superscript_btn.set_active(false);
                superscript_btn.set_icon_name("text-superscript-regular");
                superscript_btn.add_css_class("flat");
            };

            let subscript_btn = gtk::ToggleButton::new();
            {
                wrapper.append(&subscript_btn);
                subscript_btn.set_active(false);
                subscript_btn.set_icon_name("text-subscript-regular");
                subscript_btn.add_css_class("flat");
                superscript_btn.set_group(Some(&subscript_btn));
            }; */

            let color_btn = gtk::ColorDialogButton::new(Some(gtk::ColorDialog::new()));
            {
                color_btn.set_tooltip("Text color");
                color_btn.set_rgba(&gtk::gdk::RGBA::new(255.0, 255.0, 255.0, 1.0));
                toolbar.append(&color_btn);
                if let Some(btn) = color_btn.first_child().and_downcast::<gtk::Button>() {
                    btn.add_css_class("flat");
                }
                color_btn.connect_rgba_notify({
                    let sm = self.slide_manager.borrow().clone();
                    move |c| {
                        println!("Color activate {}", c.rgba());
                        let Some(ti) = sm.current_slide().and_then(|v| v.text_item()) else {
                            return;
                        };

                        ti.set_font_color(c.rgba().to_str());
                        ti.style();
                    }
                });
            }
            let bold_btn = gtk::ToggleButton::new();
            {
                bold_btn.set_tooltip("Bold");
                toolbar.append(&bold_btn);
                bold_btn.set_icon_name("text-bold-filled");
                bold_btn.add_css_class("flat");

                bold_btn.connect_toggled({
                    let sm = self.slide_manager.borrow().clone();
                    move |t| {
                        let Some(ti) = sm.current_slide().and_then(|v| v.text_item()) else {
                            glib::g_warning!("SongEditorToolbar", "No current item");
                            return;
                        };

                        ti.set_font_weight(if t.is_active() { "bold" } else { "regular" });
                        ti.style();

                        println!("Current item {:?}", sm.current_item());
                    }
                });
            };

            let italics_btn = gtk::ToggleButton::new();
            {
                italics_btn.set_tooltip("Italic");
                toolbar.append(&italics_btn);
                italics_btn.set_icon_name("text-italic-filled");
                italics_btn.add_css_class("flat");
                italics_btn.connect_toggled({
                    let sm = self.slide_manager.borrow().clone();
                    move |t| {
                        let Some(ti) = sm.current_slide().and_then(|v| v.text_item()) else {
                            return;
                        };

                        ti.set_font_style(if t.is_active() { "italic" } else { "normal" });
                        ti.style();
                    }
                });
            };

            let underline_btn = gtk::ToggleButton::new();
            {
                underline_btn.set_tooltip("Underline");
                toolbar.append(&underline_btn);
                underline_btn.set_icon_name("text-underline-filled");
                underline_btn.add_css_class("flat");
                underline_btn.connect_toggled({
                    let sm = self.slide_manager.borrow().clone();
                    move |t| {
                        let Some(ti) = sm.current_slide().and_then(|v| v.text_item()) else {
                            return;
                        };

                        ti.set_text_underline(t.is_active());
                        ti.style();
                    }
                });
            };

            let shadow_btn = gtk::ToggleButton::new();
            {
                shadow_btn.set_tooltip("Text shadow");
                toolbar.append(&shadow_btn);
                shadow_btn.set_icon_name("text-shadow-filled");
                shadow_btn.add_css_class("flat");
                shadow_btn.connect_toggled({
                    let sm = self.slide_manager.borrow().clone();
                    move |t| {
                        let Some(ti) = sm.current_slide().and_then(|v| v.text_item()) else {
                            return;
                        };

                        ti.set_text_shadow(t.is_active());
                        ti.style();
                    }
                });
            };

            let outline_btn = gtk::ToggleButton::new();
            {
                outline_btn.set_tooltip("Text outline");
                toolbar.append(&outline_btn);
                outline_btn.set_icon_name("text-outline-filled");
                outline_btn.add_css_class("flat");
                outline_btn.connect_toggled({
                    let sm = self.slide_manager.borrow().clone();
                    move |t| {
                        let Some(ti) = sm.current_slide().and_then(|v| v.text_item()) else {
                            return;
                        };

                        ti.set_text_outline(t.is_active());
                        ti.style();
                    }
                });
            };

            toolbar.append(&gtk::Separator::new(gtk::Orientation::Vertical));

            let justify_left_btn = gtk::ToggleButton::new();
            {
                justify_left_btn.set_tooltip("Justify left");
                toolbar.append(&justify_left_btn);
                justify_left_btn.set_icon_name("text-justify-left");
                justify_left_btn.add_css_class("flat");
                justify_left_btn.connect_toggled({
                    let sm = self.slide_manager.borrow().clone();
                    move |_| {
                        let Some(ti) = sm.current_slide().and_then(|v| v.text_item()) else {
                            return;
                        };

                        ti.set_justification(0);
                        ti.style();
                    }
                });
            };

            let justify_center_btn = gtk::ToggleButton::new();
            {
                justify_center_btn.set_tooltip("Justify center");
                toolbar.append(&justify_center_btn);
                justify_center_btn.set_icon_name("text-justify-center");
                justify_center_btn.add_css_class("flat");
                justify_center_btn.set_group(Some(&justify_left_btn));
                justify_center_btn.connect_toggled({
                    let sm = self.slide_manager.borrow().clone();
                    move |_| {
                        let Some(ti) = sm.current_slide().and_then(|v| v.text_item()) else {
                            return;
                        };

                        ti.set_justification(1);
                        ti.style();
                    }
                });
            };

            let justify_right_btn = gtk::ToggleButton::new();
            {
                justify_right_btn.set_tooltip("Justify right");
                toolbar.append(&justify_right_btn);
                justify_right_btn.set_icon_name("text-justify-right");
                justify_right_btn.add_css_class("flat");
                justify_right_btn.set_group(Some(&justify_left_btn));
                justify_right_btn.connect_toggled({
                    let sm = self.slide_manager.borrow().clone();
                    move |_| {
                        let Some(ti) = sm.current_slide().and_then(|v| v.text_item()) else {
                            return;
                        };

                        ti.set_justification(2);
                        ti.style();
                    }
                });
            };

            toolbar.append(&gtk::Separator::new(gtk::Orientation::Vertical));

            let align_top_btn = gtk::ToggleButton::new();
            {
                align_top_btn.set_tooltip("Align top");
                toolbar.append(&align_top_btn);
                align_top_btn.add_css_class("flat");
                align_top_btn.set_icon_name("align-top");
                align_top_btn.connect_toggled({
                    let sm = self.slide_manager.borrow().clone();
                    move |_| {
                        let Some(ti) = sm.current_slide().and_then(|v| v.text_item()) else {
                            return;
                        };

                        ti.set_align(0);
                        ti.style();
                    }
                });
            };

            let align_middle_btn = gtk::ToggleButton::new();
            {
                align_middle_btn.set_tooltip("Align middle");
                toolbar.append(&align_middle_btn);
                align_middle_btn.set_icon_name("align-middle");
                align_middle_btn.add_css_class("flat");
                align_middle_btn.set_group(Some(&align_top_btn));
                align_middle_btn.connect_toggled({
                    let sm = self.slide_manager.borrow().clone();
                    move |_| {
                        let Some(ti) = sm.current_slide().and_then(|v| v.text_item()) else {
                            return;
                        };

                        ti.set_align(1);
                        ti.style();
                    }
                });
            };

            let align_bottom_btn = gtk::ToggleButton::new();
            {
                align_bottom_btn.set_tooltip("Align bottom");
                toolbar.append(&align_bottom_btn);
                align_bottom_btn.add_css_class("flat");
                align_bottom_btn.set_icon_name("align-bottom");
                align_bottom_btn.set_group(Some(&align_top_btn));
                align_bottom_btn.connect_toggled({
                    let sm = self.slide_manager.borrow().clone();
                    move |_| {
                        let Some(ti) = sm.current_slide().and_then(|v| v.text_item()) else {
                            return;
                        };

                        ti.set_align(2);
                        ti.style();
                    }
                });
            };

            toolbar
        }

        fn build_font_size_btn(&self) -> gtk::DropDown {
            let font_btn = gtk::DropDown::from_strings(&[
                "8", "10", "12", "14", "16", "18", "20", "24", "28", "32",
            ]);
            font_btn.set_tooltip("Font size");

            font_btn.connect_selected_item_notify({
                let sm = self.slide_manager.borrow().clone();
                move |m| {
                    let item = m.selected_item();
                    let Some(str_obj) = item.and_downcast::<gtk::StringObject>() else {
                        return;
                    };
                    let size = str_obj.string().parse::<u32>().unwrap_or_default();

                    let slide = sm.current_slide();
                    let Some(ti) = slide.and_then(|v| v.text_item()) else {
                        return;
                    };

                    ti.set_font_size(size);
                    ti.style();
                }
            });

            font_btn
        }

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
            transition_btn.set_tooltip("Slide transition");

            transition_btn.connect_selected_item_notify({
                let sm = self.slide_manager.borrow().clone();
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
                    //
                    // ti.set_font_size(size);
                    // ti.style();
                }
            });

            transition_btn
        }

        pub(super) fn build_canvas_toolbar(&self) -> gtk::Box {
            let toolbar = gtk::Box::builder()
                .height_request(35)
                .spacing(8)
                .name("toolbar-box")
                .css_classes([/* "brown_box", */ "edit-toolbar-box"])
                .build();

            toolbar.set_margin_all(6);

            let color_btn = gtk::ColorDialogButton::new(Some(gtk::ColorDialog::new()));
            color_btn.set_tooltip("Background color");
            {
                color_btn.set_rgba(&gtk::gdk::RGBA::new(255.0, 255.0, 255.0, 1.0));
                toolbar.append(&color_btn);
                if let Some(btn) = color_btn.first_child().and_downcast::<gtk::Button>() {
                    btn.add_css_class("flat");
                }
                color_btn.connect_rgba_notify({
                    let sm = self.slide_manager.borrow().clone();
                    move |c| {
                        println!("Color activate {}", c.rgba());
                        let Some(canvas) = sm.current_slide().and_then(|v| v.canvas()) else {
                            return;
                        };

                        canvas.set_background_color(c.rgba().to_str());
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
                    let sm = self.slide_manager.borrow().clone();
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

            toolbar.append(&self.build_transition_btn());

            toolbar
        }
    }
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

        let stack = gtk::Stack::new();
        obj.imp().stack.replace(stack.clone());

        slide_manager.connect_item_clicked(glib::clone!(
            #[weak]
            obj,
            move |_, item| {
                let stack = obj.imp().stack.borrow();
                let Some(_item) = item else {
                    stack.set_visible_child_name(views::CANVAS);
                    return;
                };
                stack.set_visible_child_name(views::TEXT);
            }
        ));

        let text_toolbar = obj.imp().build_text_toolbar();
        stack.add_named(&text_toolbar, Some(views::TEXT));
        stack.set_visible_child_name(views::TEXT);

        let canvas_toolbar = obj.imp().build_canvas_toolbar();
        stack.add_named(&canvas_toolbar, Some(views::CANVAS));
        stack.set_visible_child_name(views::CANVAS);

        obj.append(&stack);

        obj
    }
}
