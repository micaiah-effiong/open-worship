use gtk::{
    glib::{
        self,
        object::{CastNone, ObjectExt},
        subclass::types::ObjectSubclassIsExt,
    },
    prelude::{TextBufferExt, TextTagExt, ToggleButtonExt},
};

use crate::{
    services::slide_manager::SlideManager,
    widgets::canvas::{canvas_item::CanvasItem, text_item::TextItem},
};

mod imp {
    use std::{
        cell::{Cell, RefCell},
        i32,
    };

    use adw::subclass::prelude::ObjectImplExt;
    use gtk::{
        glib::{
            self,
            object::CastNone,
            subclass::{
                object::ObjectImpl,
                types::{ObjectSubclass, ObjectSubclassExt},
            },
        },
        prelude::{BoxExt, ButtonExt, TextBufferExt, TextTagExt, ToggleButtonExt, WidgetExt},
        subclass::{box_::BoxImpl, widget::WidgetImpl},
    };

    use crate::{
        services::slide_manager::SlideManager,
        utils::{self, WidgetChildrenExt, WidgetExtrasExt, buffer_markup::TextBufferExtra},
        widgets::{
            canvas::{
                canvas_item::{CanvasItem, CanvasItemExt},
                text_item::TextItem,
            },
            group_toggle_button::GroupToggleButton,
        },
    };

    #[derive(Debug, Default)]
    pub struct TextToolbar {
        pub slide_manager: glib::WeakRef<SlideManager>,
        //
        pub font: RefCell<gtk::FontDialogButton>,
        pub font_size: RefCell<gtk::SpinButton>,
        pub color: RefCell<gtk::ColorDialogButton>,
        pub bold: RefCell<gtk::ToggleButton>,
        pub italic: RefCell<gtk::ToggleButton>,
        pub underline: RefCell<gtk::ToggleButton>,
        pub shadow: RefCell<gtk::ToggleButton>,
        pub outline: RefCell<gtk::ToggleButton>,
        pub justification: RefCell<adw::ToggleGroup>,
        pub alignment: RefCell<adw::ToggleGroup>,

        //
        pub(super) cursor_handler_id: RefCell<Option<(gtk::TextBuffer, glib::SignalHandlerId)>>,

        pub(super) checking_cursor_position: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TextToolbar {
        const NAME: &'static str = "TextToolbar";
        type Type = super::TextToolbar;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for TextToolbar {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_css_classes(&["toolbar"]);
        }
    }
    impl WidgetImpl for TextToolbar {}
    impl BoxImpl for TextToolbar {}

    impl TextToolbar {
        pub(super) fn build_ui(&self) {
            let obj = self.obj();

            obj.set_height_request(35);
            obj.set_spacing(8);
            obj.set_widget_name("text-toolbar-box");
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

                let font_desc = gtk::pango::FontDescription::new();
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

            let color_btn = gtk::ColorDialogButton::new(Some(gtk::ColorDialog::new()));
            {
                self.color.replace(color_btn.clone());
                color_btn.set_tooltip("Text color");
                color_btn.set_rgba(&gtk::gdk::RGBA::new(255.0, 255.0, 255.0, 1.0));
                obj.append(&color_btn);

                color_btn.connect_rgba_notify(glib::clone!(
                    #[weak(rename_to=imp)]
                    self,
                    move |c| {
                        if imp.checking_cursor_position.get() {
                            return;
                        };
                        let Some(ti) = imp.get_current_item() else {
                            return;
                        };

                        let buffer = ti.buffer();
                        let Some((start, end)) = buffer.selection_bounds() else {
                            return;
                        };

                        if !buffer.cursor_is_between(&start, &end) {
                            return;
                        }

                        let tag_table = buffer.tag_table();

                        let color_tag = gtk::TextTag::builder().foreground_rgba(&c.rgba()).build();
                        tag_table.add(&color_tag);
                        buffer.apply_tag(&color_tag, &start, &end);

                        // ti.set_font_color(c.hex());
                        ti.style();
                    }
                ));
            }

            let linked_btn = gtk::Box::builder().css_classes(["linked"]).build();
            obj.append(&linked_btn);

            let bold_btn = gtk::ToggleButton::new();
            {
                self.bold.replace(bold_btn.clone());
                bold_btn.set_tooltip("Bold");
                linked_btn.append(&bold_btn);
                bold_btn.set_icon_name("text-bold-filled");

                bold_btn.connect_toggled(glib::clone!(
                    #[weak(rename_to=imp)]
                    self,
                    move |btn| {
                        if imp.checking_cursor_position.get() {
                            return;
                        };
                        let Some(ti) = imp.get_current_item() else {
                            return;
                        };

                        let buffer = ti.buffer();
                        let tag_table = buffer.tag_table();

                        let Some(tag) = tag_table.lookup(utils::text_tags::BOLD) else {
                            return;
                        };
                        let Some((start, end)) = buffer.selection_bounds() else {
                            return;
                        };

                        if !buffer.cursor_is_between(&start, &end) {
                            return;
                        }

                        if btn.is_active() {
                            buffer.apply_tag(&tag, &start, &end);
                        } else {
                            // buffer.remove_tag(&tag, &start, &end);
                            buffer.remove_tags_by(|v| v.is_weight_set(), &start, &end);
                        }

                        // ti.set_font_weight(if t.is_active() { "bold" } else { "regular" });
                        ti.style();
                    }
                ));
            };

            let italics_btn = gtk::ToggleButton::new();
            {
                self.italic.replace(italics_btn.clone());
                italics_btn.set_tooltip("Italic");
                linked_btn.append(&italics_btn);
                italics_btn.set_icon_name("text-italic-filled");

                italics_btn.connect_toggled(glib::clone!(
                    #[weak(rename_to=imp)]
                    self,
                    move |btn| {
                        if imp.checking_cursor_position.get() {
                            return;
                        };
                        let Some(ti) = imp.get_current_item() else {
                            return;
                        };

                        let buffer = ti.buffer();
                        let tag_table = buffer.tag_table();

                        let Some(tag) = tag_table.lookup(utils::text_tags::ITALIC) else {
                            return;
                        };
                        let Some((start, end)) = buffer.selection_bounds() else {
                            return;
                        };

                        if btn.is_active() {
                            buffer.apply_tag(&tag, &start, &end);
                        } else {
                            // buffer.remove_tag(&tag, &start, &end);
                            buffer.remove_tags_by(|v| v.is_style_set(), &start, &end);
                        }

                        // ti.set_font_style(if t.is_active() { "italic" } else { "normal" });
                        ti.style();
                    }
                ));
            };

            let underline_btn = gtk::ToggleButton::new();
            {
                self.underline.replace(underline_btn.clone());
                underline_btn.set_tooltip("Underline");
                linked_btn.append(&underline_btn);
                underline_btn.set_icon_name("text-underline-filled");

                underline_btn.connect_toggled(glib::clone!(
                    #[weak(rename_to=imp)]
                    self,
                    move |btn| {
                        if imp.checking_cursor_position.get() {
                            return;
                        };
                        let Some(ti) = imp.get_current_item() else {
                            return;
                        };

                        let buffer = ti.buffer();
                        let tag_table = buffer.tag_table();

                        let Some(tag) = tag_table.lookup(utils::text_tags::UNDERLINE) else {
                            return;
                        };
                        let Some((start, end)) = buffer.selection_bounds() else {
                            return;
                        };

                        if btn.is_active() {
                            buffer.apply_tag(&tag, &start, &end);
                        } else {
                            // buffer.remove_tag(&tag, &start, &end);
                            buffer.remove_tags_by(|v| v.is_underline_set(), &start, &end);
                        }

                        // ti.set_text_underline(t.is_active());
                        ti.style();
                    }
                ));
            };

            let shadow_btn = gtk::ToggleButton::new();
            {
                self.shadow.replace(shadow_btn.clone());
                shadow_btn.set_tooltip("Text shadow");
                linked_btn.append(&shadow_btn);
                shadow_btn.set_icon_name("text-shadow-filled");

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
                self.outline.replace(outline_btn.clone());
                outline_btn.set_tooltip("Text outline");
                linked_btn.append(&outline_btn);
                outline_btn.set_icon_name("text-outline-filled");

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

            let justifcation_btn = adw::ToggleGroup::new();
            {
                self.justification.replace(justifcation_btn.clone());
                obj.append(&justifcation_btn);
                justifcation_btn.set_height_request(35);
                // justifcation_btn.set_spacing(8);
                justifcation_btn.connect_active_notify({
                    let sm = sm.clone();
                    move |t| {
                        let Some(ti) = sm.current_item().and_downcast::<TextItem>() else {
                            return;
                        };

                        match t.active() {
                            0 | 1 | 2 => {
                                ti.set_justification(t.active());
                                ti.style();
                            }
                            _ => (),
                        };
                    }
                });
            };

            let justify_left_btn = adw::Toggle::new();
            {
                justify_left_btn.set_tooltip("Justify left");
                justify_left_btn.set_icon_name(Some("text-justify-left"));
                justifcation_btn.add(justify_left_btn);
            }

            let justify_center_btn = adw::Toggle::new();
            {
                justify_center_btn.set_tooltip("Justify center");
                justify_center_btn.set_icon_name(Some("text-justify-center"));
                justifcation_btn.add(justify_center_btn);
            };

            let justify_right_btn = adw::Toggle::new();
            {
                justify_right_btn.set_tooltip("Justify right");
                justify_right_btn.set_icon_name(Some("text-justify-right"));
                justifcation_btn.add(justify_right_btn);
            };

            obj.append(&gtk::Separator::new(gtk::Orientation::Vertical));

            let alignment_btn = adw::ToggleGroup::new();
            {
                self.alignment.replace(alignment_btn.clone());
                obj.append(&alignment_btn);
                alignment_btn.set_height_request(35);
                // alignment_btn.set_spacing(8);
                alignment_btn.connect_active_notify({
                    let sm = sm.clone();
                    move |t| {
                        let Some(ti) = sm.current_item().and_downcast::<TextItem>() else {
                            return;
                        };

                        match t.active() {
                            0 | 1 | 2 => {
                                ti.set_align(t.active());
                                ti.style();
                            }
                            _ => (),
                        };
                    }
                });
            }

            let align_top_btn = adw::Toggle::new();
            {
                align_top_btn.set_tooltip("Align top");
                align_top_btn.set_icon_name(Some("align-top"));
                alignment_btn.add(align_top_btn);
            };

            let align_middle_btn = adw::Toggle::new();
            {
                align_middle_btn.set_tooltip("Align middle");
                align_middle_btn.set_icon_name(Some("align-middle"));
                alignment_btn.add(align_middle_btn);
            };

            let align_bottom_btn = adw::Toggle::new();
            {
                align_bottom_btn.set_tooltip("Align bottom");
                align_bottom_btn.set_icon_name(Some("align-bottom"));
                alignment_btn.add(align_bottom_btn);
            };
        }

        fn build_font_size_btn(&self) -> gtk::SpinButton {
            let font_btn = gtk::SpinButton::with_range(0.0, 100.0, 1.0);
            self.font_size.replace(font_btn.clone());
            font_btn.set_tooltip("Font size");

            font_btn.connect_value_changed({
                let sm = self.slide_manager.clone();
                move |btn| {
                    let Some(sm) = sm.upgrade() else {
                        return;
                    };

                    let Some(ti) = sm.current_item().and_downcast::<TextItem>() else {
                        return;
                    };

                    let size = btn.value();

                    ti.set_font_size(size as f32);
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
                let mut font_desc = gtk::pango::FontDescription::new();
                font_desc.set_size(ti.font_size() as i32);
                font_desc.set_family(&ti.font());

                self.font.borrow().set_font_desc(&font_desc);
            }

            self.font_size
                .borrow()
                .clone()
                .set_value(ti.font_size().into());

            self.shadow.borrow().set_active(ti.text_shadow());
            self.outline.borrow().set_active(ti.text_outline());
            self.justification.borrow().set_active(ti.justification());
            self.alignment.borrow().set_active(ti.align());
        }

        pub(super) fn get_current_item(&self) -> Option<TextItem> {
            let Some(sm) = self.slide_manager.upgrade() else {
                return None;
            };
            sm.current_item()
                .or_else(|| {
                    sm.current_slide()
                        .and_then(|v| v.canvas())
                        .and_then(|v| v.widget().get_children::<TextItem>().next())
                        .and_upcast::<CanvasItem>()
                })
                .and_downcast::<TextItem>()
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
            move |_, ci| {
                obj.imp().update_props();
                obj.listen_to_cursor_move(ci);
            }
        ));

        obj
    }

    fn listen_to_cursor_move(&self, ci: Option<CanvasItem>) {
        if let Some((buff, id)) = self.imp().cursor_handler_id.take() {
            buff.disconnect(id);
        }
        let Some(ti) = ci
            .and_downcast::<TextItem>()
            .or_else(|| self.imp().get_current_item())
        else {
            return;
        };

        let id = ti.buffer().connect_cursor_position_notify(glib::clone!(
            #[weak(rename_to=obj)]
            self,
            move |buff| {
                let cursor = buff.cursor_position();
                let iter = buff.iter_at_offset(cursor);

                let imp = obj.imp();
                imp.checking_cursor_position.set(true);
                check_and_do(
                    |v| v.is_weight_set(),
                    |(a, _)| imp.bold.borrow().set_active(a),
                    &iter,
                );
                check_and_do(
                    |v| v.is_style_set(),
                    |(a, _)| imp.italic.borrow().set_active(a),
                    &iter,
                );
                check_and_do(
                    |v| v.is_underline_set(),
                    |(a, _)| imp.underline.borrow().set_active(a),
                    &iter,
                );
                check_and_do(
                    |v| v.is_foreground_set(),
                    |(_, t)| {
                        let rgba = if let Some(tag) = t
                            && let Some(rgba) = tag.foreground_rgba()
                        {
                            rgba
                        } else {
                            gtk::gdk::RGBA::new(1.0, 1.0, 1.0, 1.0)
                        };

                        imp.color.borrow().set_rgba(&rgba);
                    },
                    &iter,
                );

                imp.checking_cursor_position.set(false);
            }
        ));
        self.imp()
            .cursor_handler_id
            .replace(Some((ti.buffer(), id)));
    }
}

fn check_and_do<F: Fn(&gtk::TextTag) -> bool, A: Fn((bool, Option<gtk::TextTag>))>(
    tag_fn: F,
    action: A,
    iter: &gtk::TextIter,
) {
    let active = iter
        .tags()
        .into_iter()
        .filter(|t| tag_fn(t))
        .find(|tag| iter.has_tag(tag));

    action((active.is_some(), active))
}
