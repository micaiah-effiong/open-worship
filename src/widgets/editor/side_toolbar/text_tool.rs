use gtk::{
    glib::{
        self,
        object::{Cast, ObjectExt},
        subclass::types::ObjectSubclassIsExt,
    },
    prelude::{TextBufferExt, TextTagExt, ToggleButtonExt, WidgetExt},
};

use crate::{
    services::slide_manager::SlideManager,
    widgets::canvas::{canvas_item::CanvasItem, text_item::TextItem},
};

mod imp {
    use super::*;
    use std::cell::{Cell, RefCell};

    use glib::{
        self, Properties,
        subclass::{object::ObjectImpl, types::ObjectSubclass},
    };
    use gtk::{
        glib::{
            object::CastNone,
            subclass::{object::ObjectImplExt, types::ObjectSubclassExt},
        },
        prelude::{BoxExt, ObjectExt, WidgetExt},
        subclass::{box_::BoxImpl, prelude::DerivedObjectProperties, widget::WidgetImpl},
    };

    use crate::{
        services::slide_manager::SlideManager,
        utils::{self, WidgetChildrenExt, WidgetExtrasExt, buffer_markup::TextBufferExtra},
        widgets::{canvas::canvas_item::CanvasItemExt, ow_spinbutton::OwSpinButton},
    };

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::TextTool)]
    pub struct TextTool {
        #[property(get, construct_only)]
        slide_manager: glib::WeakRef<SlideManager>,
        //
        pub(super) font_btn: RefCell<gtk::FontDialogButton>,
        pub(super) font_size_btn: RefCell<OwSpinButton>,
        pub(super) bold_btn: RefCell<gtk::ToggleButton>,
        pub(super) italic_btn: RefCell<gtk::ToggleButton>,
        pub(super) underline_btn: RefCell<gtk::ToggleButton>,
        pub(super) outline_btn: RefCell<gtk::ToggleButton>,
        pub(super) shadow_btn: RefCell<gtk::ToggleButton>,
        pub(super) color_btn: RefCell<gtk::ColorDialogButton>,
        pub(super) justify_btn: RefCell<adw::ToggleGroup>,
        pub(super) align_btn: RefCell<adw::ToggleGroup>,

        //
        pub(super) cursor_handler_id: RefCell<Option<(gtk::TextBuffer, glib::SignalHandlerId)>>,

        pub(super) checking_cursor_position: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TextTool {
        const NAME: &'static str = "TextTool";
        type Type = super::TextTool;
        type ParentType = gtk::Box;
    }

    #[glib::derived_properties]
    impl ObjectImpl for TextTool {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj().clone();

            if let Some(sm) = obj.slide_manager() {
                self.build_text_section(&sm);
            }
        }
    }

    impl WidgetImpl for TextTool {}
    impl BoxImpl for TextTool {}

    impl TextTool {
        fn build_text_section(&self, slide_manager: &SlideManager) {
            let content = gtk::Box::new(gtk::Orientation::Vertical, 7);
            content.set_css_classes(&["small-pad"]);
            self.obj().append(&content);

            self.build_font_section(&content, slide_manager);
            self.build_text_formatter(&content, slide_manager);
            self.build_justification(&content, slide_manager);
            self.build_alignment(&content, slide_manager);
        }

        fn build_font_section(&self, parent: &gtk::Box, sm: &SlideManager) {
            let font_content = gtk::Box::builder().spacing(3).build();
            parent.append(&font_content);
            let font = gtk::FontDialogButton::new(Some(gtk::FontDialog::new()));
            font.set_tooltip("Font Family");
            font.set_hexpand(true);
            font.set_level(gtk::FontLevel::Family);
            font_content.append(&font);
            font.connect_font_desc_notify({
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
            self.font_btn.replace(font);

            let font_size = OwSpinButton::with_range(0.0, 100.0, 1.0);
            font_size.set_tooltip("Font size");
            font_size.set_hexpand(true);
            font_content.append(&font_size);
            font_size.connect_value_changed({
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
            self.font_size_btn.replace(font_size);
        }

        fn build_text_formatter(&self, parent: &gtk::Box, sm: &SlideManager) {
            let format_box = gtk::Box::builder().spacing(3).build();
            parent.append(&format_box);

            let bold = Self::build_bold(self);
            format_box.append(&bold);
            self.bold_btn.replace(bold);

            let italics = Self::build_italic(self);
            format_box.append(&italics);
            self.italic_btn.replace(italics);

            let underline = Self::build_underline(self);
            format_box.append(&underline);
            self.underline_btn.replace(underline);

            let shadow = Self::build_shadow(self, sm);
            format_box.append(&shadow);
            self.shadow_btn.replace(shadow);

            let outline = Self::build_outline(self, sm);
            format_box.append(&outline);
            self.outline_btn.replace(outline);

            let color_btn = Self::build_color(self, sm);
            format_box.append(&color_btn);
            self.color_btn.replace(color_btn);
        }

        fn build_justification(&self, parent: &gtk::Box, sm: &SlideManager) {
            let justify = adw::ToggleGroup::new();
            parent.append(&justify);
            let j_left = adw::Toggle::builder()
                .tooltip("Justify left")
                .icon_name("text-justify-left")
                .build();
            justify.add(j_left);
            let j_center = adw::Toggle::builder()
                .tooltip("Justify center")
                .icon_name("text-justify-center")
                .build();
            justify.add(j_center);
            let j_right = adw::Toggle::builder()
                .tooltip("Justify right")
                .icon_name("text-justify-right")
                .build();
            justify.add(j_right);

            justify.connect_active_notify({
                let sm = sm.clone();
                move |t| {
                    if let Some(ti) = sm.current_item().and_downcast::<TextItem>()
                        && let 0..=2 = t.active()
                    {
                        ti.set_justification(t.active());
                        ti.style();
                    };
                }
            });

            self.justify_btn.replace(justify);
        }

        fn build_alignment(&self, parent: &gtk::Box, sm: &SlideManager) {
            let align_btn = adw::ToggleGroup::new();
            parent.append(&align_btn);
            let a_top = adw::Toggle::builder()
                .tooltip("Align top")
                .icon_name("align-top")
                .build();
            align_btn.add(a_top);
            let a_middle = adw::Toggle::builder()
                .tooltip("Align middle")
                .icon_name("align-middle")
                .build();
            align_btn.add(a_middle);
            let a_bottom = adw::Toggle::builder()
                .tooltip("Align bottom")
                .icon_name("align-bottom")
                .build();
            align_btn.add(a_bottom);

            align_btn.connect_active_notify({
                let sm = sm.clone();
                move |t| {
                    if let Some(ti) = sm.current_item().and_downcast::<TextItem>()
                        && let 0..=2 = t.active()
                    {
                        ti.set_align(t.active());
                        ti.style();
                    };
                }
            });
            self.align_btn.replace(align_btn);
        }

        pub(super) fn get_current_item(&self) -> Option<TextItem> {
            let sm = self.obj().slide_manager()?;
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

    impl TextTool {
        fn build_bold(imp: &Self) -> gtk::ToggleButton {
            let bold = gtk::ToggleButton::builder()
                .tooltip_text("Bold")
                .icon_name("text-bold-filled")
                .build();
            bold.connect_toggled(glib::clone!(
                #[weak]
                imp,
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

            bold
        }

        fn build_italic(imp: &Self) -> gtk::ToggleButton {
            let italic = gtk::ToggleButton::builder()
                .tooltip_text("Italic")
                .icon_name("text-italic-filled")
                .build();
            italic.connect_toggled(glib::clone!(
                #[weak]
                imp,
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

            italic
        }

        fn build_underline(imp: &Self) -> gtk::ToggleButton {
            let underline = gtk::ToggleButton::builder()
                .tooltip_text("Underline")
                .icon_name("text-underline-filled")
                .build();
            underline.connect_toggled(glib::clone!(
                #[weak]
                imp,
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

            underline
        }

        fn build_shadow(_: &Self, sm: &SlideManager) -> gtk::ToggleButton {
            let shadow = gtk::ToggleButton::builder()
                .tooltip_text("Text Shadow")
                .icon_name("text-shadow-filled")
                .build();
            shadow.connect_toggled({
                let sm = sm.clone();
                move |t| {
                    if let Some(ti) = sm.current_item().and_downcast::<TextItem>() {
                        ti.set_text_shadow(t.is_active());
                        ti.style();
                    };
                }
            });

            shadow
        }

        fn build_outline(_: &Self, sm: &SlideManager) -> gtk::ToggleButton {
            let outline = gtk::ToggleButton::builder()
                .tooltip_text("Text outline")
                .icon_name("text-outline-filled")
                .build();

            outline.connect_toggled({
                let sm = sm.clone();
                move |t| {
                    if let Some(ti) = sm.current_item().and_downcast::<TextItem>() {
                        ti.set_text_outline(t.is_active());
                        ti.style();
                    };
                }
            });

            outline
        }

        fn build_color(imp: &Self, _: &SlideManager) -> gtk::ColorDialogButton {
            let color = gtk::ColorDialogButton::new(Some(gtk::ColorDialog::new()));
            color.set_tooltip("Text color");
            color.set_rgba(&gtk::gdk::RGBA::new(255.0, 255.0, 255.0, 1.0));

            color.connect_rgba_notify(glib::clone!(
                #[weak]
                imp,
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

            color
        }
    }
}

glib::wrapper! {
pub struct TextTool(ObjectSubclass<imp::TextTool>)
        @extends  gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for TextTool {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl TextTool {
    pub fn new(sm: &SlideManager) -> Self {
        glib::Object::builder()
            .property("slide_manager", sm.clone())
            .build()
    }

    pub fn update_props(&self, ci: &Option<CanvasItem>, _sm: &SlideManager) {
        let imp = self.imp();

        let items = ci
            .as_ref()
            .and_then(|ci| ci.downcast_ref::<TextItem>().map(|ti| (ci.clone(), ti)));

        self.set_sensitive(items.is_some());
        let Some((ci, ti)) = items else {
            return;
        };

        let mut font_desc = gtk::pango::FontDescription::new();
        font_desc.set_size(ti.font_size() as i32);
        font_desc.set_family(&ti.font());
        imp.font_btn.borrow().set_font_desc(&font_desc);

        imp.font_size_btn
            .borrow()
            .clone()
            .set_value(ti.font_size() as f64);

        imp.shadow_btn.borrow().set_active(ti.text_shadow());
        imp.outline_btn.borrow().set_active(ti.text_outline());
        imp.justify_btn.borrow().set_active(ti.justification());
        imp.align_btn.borrow().set_active(ti.align());

        self.listen_to_cursor_move(ci.clone());
    }

    fn listen_to_cursor_move(&self, ci: CanvasItem) {
        if let Some((buff, id)) = self.imp().cursor_handler_id.take() {
            buff.disconnect(id);
        }
        let Some(ti) = ci
            .downcast::<TextItem>()
            .ok()
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
                    |(a, _)| imp.bold_btn.borrow().set_active(a),
                    &iter,
                );
                check_and_do(
                    |v| v.is_style_set(),
                    |(a, _)| imp.italic_btn.borrow().set_active(a),
                    &iter,
                );
                check_and_do(
                    |v| v.is_underline_set(),
                    |(a, _)| imp.underline_btn.borrow().set_active(a),
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

                        imp.color_btn.borrow().set_rgba(&rgba);
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
