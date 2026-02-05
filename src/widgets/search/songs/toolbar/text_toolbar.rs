use gtk::glib::{self, subclass::types::ObjectSubclassIsExt};

use crate::services::slide_manager::SlideManager;

mod imp {
    use std::cell::RefCell;

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
        prelude::{BoxExt, ButtonExt, ToggleButtonExt, WidgetExt},
        subclass::{box_::BoxImpl, widget::WidgetImpl},
    };
    use relm4::RelmWidgetExt;

    use crate::{
        services::slide_manager::SlideManager,
        utils::ColorDialogButtonExtra,
        widgets::{
            canvas::{canvas_item::CanvasItemExt, text_item::TextItem},
            entry_combo::{self, EntryCombo},
            group_toggle_button::GroupToggleButton,
        },
    };

    #[derive(Debug, Default)]
    pub struct TextToolbar {
        pub slide_manager: glib::WeakRef<SlideManager>,
        //
        pub font: RefCell<gtk::FontDialogButton>,
        pub font_size: RefCell<EntryCombo>,
        pub color: RefCell<gtk::ColorDialogButton>,
        pub bold: RefCell<gtk::ToggleButton>,
        pub italic: RefCell<gtk::ToggleButton>,
        pub underline: RefCell<gtk::ToggleButton>,
        pub shadow: RefCell<gtk::ToggleButton>,
        pub outline: RefCell<gtk::ToggleButton>,
        pub justification: RefCell<GroupToggleButton>,
        pub alignment: RefCell<GroupToggleButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TextToolbar {
        const NAME: &'static str = "TextToolbar";
        type Type = super::TextToolbar;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for TextToolbar {}
    impl WidgetImpl for TextToolbar {}
    impl BoxImpl for TextToolbar {}

    impl TextToolbar {
        pub(super) fn build_ui(&self) {
            let obj = self.obj();

            obj.set_height_request(35);
            obj.set_spacing(8);
            obj.set_widget_name("text-toolbar-box");
            obj.set_css_classes(&["edit-toolbar-box"]);
            obj.set_margin_all(6);

            let Some(sm) = self.slide_manager.upgrade() else {
                return;
            };

            let font_btn = gtk::FontDialogButton::new(Some(gtk::FontDialog::new()));

            {
                self.font.replace(font_btn.clone());
                font_btn.set_tooltip("Font family");
                obj.append(&font_btn);
                obj.append(&self.build_font_size_btn());
                font_btn.set_level(gtk::FontLevel::Family);
                if let Some(btn) = font_btn.first_child().and_downcast::<gtk::Button>() {
                    btn.add_css_class("flat");
                }

                let mut font_desc = gtk::pango::FontDescription::new();
                font_desc.set_family("Tahoma");
                font_btn.set_font_desc(&font_desc);

                font_btn.connect_font_desc_notify({
                    let sm = sm.clone();
                    move |f| {
                        let Some(d) = f.font_desc() else {
                            return;
                        };

                        let Some(ti) = sm.current_item().and_downcast::<TextItem>() else {
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
                self.color.replace(color_btn.clone());
                color_btn.set_tooltip("Text color");
                color_btn.set_rgba(&gtk::gdk::RGBA::new(255.0, 255.0, 255.0, 1.0));
                obj.append(&color_btn);
                if let Some(btn) = color_btn.first_child().and_downcast::<gtk::Button>() {
                    btn.add_css_class("flat");
                }
                color_btn.connect_rgba_notify({
                    let sm = sm.clone();
                    move |c| {
                        let Some(ti) = sm.current_item().and_downcast::<TextItem>() else {
                            return;
                        };
                        ti.set_font_color(c.hex());
                        ti.style();
                    }
                });
            }

            let bold_btn = gtk::ToggleButton::new();
            {
                self.bold.replace(bold_btn.clone());
                bold_btn.set_tooltip("Bold");
                obj.append(&bold_btn);
                bold_btn.set_icon_name("text-bold-filled");
                bold_btn.add_css_class("flat");

                bold_btn.connect_toggled({
                    let sm = sm.clone();
                    move |t| {
                        let Some(ti) = sm.current_item().and_downcast::<TextItem>() else {
                            return;
                        };

                        ti.set_font_weight(if t.is_active() { "bold" } else { "regular" });
                        ti.style();
                    }
                });
            };

            let italics_btn = gtk::ToggleButton::new();
            {
                self.italic.replace(italics_btn.clone());
                italics_btn.set_tooltip("Italic");
                obj.append(&italics_btn);
                italics_btn.set_icon_name("text-italic-filled");
                italics_btn.add_css_class("flat");
                italics_btn.connect_toggled({
                    let sm = sm.clone();
                    move |t| {
                        let Some(ti) = sm.current_item().and_downcast::<TextItem>() else {
                            return;
                        };

                        ti.set_font_style(if t.is_active() { "italic" } else { "normal" });
                        ti.style();
                    }
                });
            };

            let underline_btn = gtk::ToggleButton::new();
            {
                self.underline.replace(italics_btn.clone());
                underline_btn.set_tooltip("Underline");
                obj.append(&underline_btn);
                underline_btn.set_icon_name("text-underline-filled");
                underline_btn.add_css_class("flat");
                underline_btn.connect_toggled({
                    let sm = sm.clone();
                    move |t| {
                        let Some(ti) = sm.current_item().and_downcast::<TextItem>() else {
                            return;
                        };

                        ti.set_text_underline(t.is_active());
                        ti.style();
                    }
                });
            };

            let shadow_btn = gtk::ToggleButton::new();
            {
                self.shadow.replace(italics_btn.clone());
                shadow_btn.set_tooltip("Text shadow");
                obj.append(&shadow_btn);
                shadow_btn.set_icon_name("text-shadow-filled");
                shadow_btn.add_css_class("flat");
                shadow_btn.connect_toggled({
                    let sm = sm.clone();
                    move |t| {
                        let Some(ti) = sm.current_item().and_downcast::<TextItem>() else {
                            return;
                        };

                        ti.set_text_shadow(t.is_active());
                        ti.style();
                    }
                });
            };

            let outline_btn = gtk::ToggleButton::new();
            {
                self.outline.replace(italics_btn.clone());
                outline_btn.set_tooltip("Text outline");
                obj.append(&outline_btn);
                outline_btn.set_icon_name("text-outline-filled");
                outline_btn.add_css_class("flat");
                outline_btn.connect_toggled({
                    let sm = sm.clone();
                    move |t| {
                        let Some(ti) = sm.current_item().and_downcast::<TextItem>() else {
                            return;
                        };

                        ti.set_text_outline(t.is_active());
                        ti.style();
                    }
                });
            };

            obj.append(&gtk::Separator::new(gtk::Orientation::Vertical));

            let justifcation_btn = GroupToggleButton::new();
            {
                self.justification.replace(justifcation_btn.clone());
                obj.append(&justifcation_btn);
                justifcation_btn.set_height_request(35);
                justifcation_btn.set_spacing(8);
                justifcation_btn.connect_mode_changed({
                    let sm = sm.clone();
                    move |t, _| {
                        let Some(ti) = sm.current_item().and_downcast::<TextItem>() else {
                            return;
                        };

                        match t.selected() {
                            0 | 1 | 2 => {
                                ti.set_justification(t.selected() as u32);
                                ti.style();
                            }
                            _ => (),
                        };
                    }
                });
            };

            let justify_left_btn = gtk::ToggleButton::new();
            {
                justifcation_btn.append_toggle_button(&justify_left_btn);
                justify_left_btn.set_tooltip("Justify left");
                justify_left_btn.set_icon_name("text-justify-left");
                justify_left_btn.add_css_class("flat");
            }

            let justify_center_btn = gtk::ToggleButton::new();
            {
                justifcation_btn.append_toggle_button(&justify_center_btn);
                justify_center_btn.set_tooltip("Justify center");
                justify_center_btn.set_icon_name("text-justify-center");
                justify_center_btn.add_css_class("flat");
            };

            let justify_right_btn = gtk::ToggleButton::new();
            {
                justifcation_btn.append_toggle_button(&justify_right_btn);
                justify_right_btn.set_tooltip("Justify right");
                justify_right_btn.set_icon_name("text-justify-right");
                justify_right_btn.add_css_class("flat");
            };

            obj.append(&gtk::Separator::new(gtk::Orientation::Vertical));

            let alignment_btn = GroupToggleButton::new();
            {
                self.alignment.replace(alignment_btn.clone());
                obj.append(&alignment_btn);
                alignment_btn.set_height_request(35);
                alignment_btn.set_spacing(8);
                alignment_btn.connect_mode_changed({
                    let sm = sm.clone();
                    move |t, _| {
                        let Some(ti) = sm.current_item().and_downcast::<TextItem>() else {
                            return;
                        };

                        match t.selected() {
                            0 | 1 | 2 => {
                                ti.set_align(t.selected() as u32);
                                ti.style();
                            }
                            _ => (),
                        };
                    }
                });
            }

            let align_top_btn = gtk::ToggleButton::new();
            {
                alignment_btn.append_toggle_button(&align_top_btn);
                align_top_btn.set_tooltip("Align top");
                align_top_btn.add_css_class("flat");
                align_top_btn.set_icon_name("align-top");
            };

            let align_middle_btn = gtk::ToggleButton::new();
            {
                align_middle_btn.set_tooltip("Align middle");
                alignment_btn.append_toggle_button(&align_middle_btn);
                align_middle_btn.set_icon_name("align-middle");
                align_middle_btn.add_css_class("flat");
                align_middle_btn.set_group(Some(&align_top_btn));
            };

            let align_bottom_btn = gtk::ToggleButton::new();
            {
                align_bottom_btn.set_tooltip("Align bottom");
                alignment_btn.append_toggle_button(&align_bottom_btn);
                align_bottom_btn.add_css_class("flat");
                align_bottom_btn.set_icon_name("align-bottom");
                align_bottom_btn.set_group(Some(&align_top_btn));
            };
        }

        fn build_font_size_btn(&self) -> EntryCombo {
            let m =
                gtk::StringList::new(&["8", "10", "12", "14", "16", "18", "20", "24", "28", "32"]);
            let font_btn = EntryCombo::new(Some(&m));
            self.font_size.replace(font_btn.clone());
            font_btn.set_tooltip("Font size");

            font_btn.connect_changed({
                let sm = self.slide_manager.clone();
                move |_, text| {
                    let Some(sm) = sm.upgrade() else {
                        return;
                    };

                    let Some(ti) = sm.current_item().and_downcast::<TextItem>() else {
                        return;
                    };

                    let size = text.parse::<u32>().unwrap_or_default();

                    ti.set_font_size(size);
                    ti.style();
                }
            });

            font_btn
        }

        pub(super) fn update_props(&self) {
            let Some(sm) = self.slide_manager.upgrade() else {
                return;
            };

            let Some(ti) = sm.current_item().and_downcast::<TextItem>() else {
                return;
            };

            {
                let font_desc = gtk::pango::FontDescription::from_string(
                    format!("{} {}", ti.font(), ti.font_size()).as_str(),
                );
                self.font.borrow().set_font_desc(&font_desc);
            }

            self.font_size
                .borrow()
                .clone()
                .set_text(ti.font_size().to_string());

            if let Ok(color) = gdk::RGBA::parse(&ti.font_color()) {
                self.color.borrow().set_rgba(&color);
            }

            self.bold
                .borrow()
                .set_active(ti.font_weight().contains("bold"));
            self.italic
                .borrow()
                .set_active(ti.font_style().contains("italic"));
            self.underline.borrow().set_active(ti.text_underline());
            self.shadow.borrow().set_active(ti.text_shadow());
            self.outline.borrow().set_active(ti.text_outline());
            self.justification.borrow().set_active(ti.justification());
            self.alignment.borrow().set_active(ti.align());
        }
    }
}

glib::wrapper! {
    pub struct TextToolbar(ObjectSubclass<imp::TextToolbar>)
        @extends  gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for TextToolbar {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl TextToolbar {
    pub fn new(slide_manager: &SlideManager) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().slide_manager.set(Some(&slide_manager));
        obj.imp().build_ui();

        slide_manager.connect_item_clicked(glib::clone!(
            #[weak]
            obj,
            move |_, _| obj.imp().update_props()
        ));

        obj.imp().update_props();
        obj
    }
}
