use gtk::{
    glib::{
        self,
        object::{Cast, ObjectExt},
        subclass::types::ObjectSubclassIsExt,
    },
    pango,
    prelude::{
        DrawingAreaExtManual, EventControllerExt, GestureExt, GridExt, TextBufferExt, TextViewExt,
        WidgetExt,
    },
};

use crate::{
    // services::history_manager::history_action::TypedHistoryAction,
    services::settings::ApplicationSettings,
    utils::{self, buffer_markup::TextBufferExtra},
    widgets::canvas::{
        canvas::Canvas,
        canvas_item::{CanvasItem, CanvasItemExt},
        serialise::CanvasItemData,
    },
};

const PLACEHOLDER_TEXT: &str = "Click to add text...";

mod stacks {
    pub const EDIT: &str = "edit";
    pub const DISPLAY: &str = "display";
}

mod imp {
    use std::cell::{Cell, RefCell};
    use std::u32;

    use glib::subclass::object::ObjectImpl;
    use glib::subclass::types::ObjectSubclass;
    use gtk::glib::{self, Properties, prelude::*};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use super::*;
    use crate::utils::{self};
    use crate::widgets::canvas::canvas_item::{
        CanvasItem, CanvasItemExt, CanvasItemImpl, CanvasItemImplExt,
    };
    use crate::widgets::canvas::serialise::{CanvasItemType, TextItemData};

    #[derive(Properties, Debug, Default)]
    #[properties(wrapper_type = super::TextItem)]
    pub struct TextItem {
        pub entry: RefCell<gtk::TextView>,
        pub drawing_area: RefCell<gtk::DrawingArea>,
        pub stack: RefCell<gtk::Stack>,

        #[property(get, set, default_value = 1, construct)]
        pub justification: Cell<u32>,
        #[property(get, set, default_value = 1, construct)]
        pub align: Cell<u32>,
        #[property(get, set, default_value = 16.0, construct)]
        pub font_size: Cell<f32>,
        pub display_font_size: Cell<f32>,
        #[property(get, set, default_value = "", construct)]
        pub font: RefCell<String>,
        #[property(get, set, default_value = false, construct)]
        pub text_shadow: Cell<bool>,
        #[property(get, set, default_value = false, construct)]
        pub text_outline: Cell<bool>,

        #[property(get=Self::get_editing_, set=Self::set_editing_)]
        pub editing: Cell<bool>,

        pub setting_text: Cell<bool>,
        pub first_change_in_edit: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TextItem {
        const NAME: &'static str = "TextItem";
        type Type = super::TextItem;
        type ParentType = CanvasItem;
    }

    impl WidgetImpl for TextItem {}
    impl CanvasItemImpl for TextItem {
        fn load_item_data(&self) {
            let Some(json_data) = self.obj().get_save_data() else {
                return;
            };

            let text_data = match json_data.item_type {
                CanvasItemType::Text(text_item_data) => text_item_data,
                _ => return,
            };

            let t = text_data.text_data.clone();
            if !t.is_empty()
                && let Ok(t) = String::from_utf8(glib::base64_decode(&t))
            {
                let mut iter = self.entry.borrow().buffer().start_iter();
                self.entry.borrow().buffer().insert_markup(&mut iter, &t);
                self.drawing_area.borrow().queue_draw();
            }

            self.obj().set_font_size(text_data.font_size);
            self.obj().set_font(text_data.font);
            self.obj().set_justification(text_data.justification);
            self.obj().set_align(text_data.align);
            self.obj().set_text_outline(text_data.text_outline);
            self.obj().set_text_shadow(text_data.text_shadow);
        }

        fn serialise_item(&self) -> CanvasItemType {
            let entry = self.entry.borrow();

            let text = entry.buffer().markup();

            let encoded = glib::base64_encode(text.as_bytes()).to_string();
            let font_size = self.obj().font_size();

            let obj = self.obj();

            let data = TextItemData {
                font: obj.font(),
                font_size,
                justification: obj.justification(),
                align: obj.align(),
                text_data: encoded.clone(),
                text_outline: obj.text_outline(),
                text_shadow: obj.text_shadow(),
            };
            let data = CanvasItemType::Text(data);

            data
        }

        fn style(&self) {
            self.parent_style();
            let obj = self.obj().clone();

            // NOTE: allow label to capture tag update from buffer
            obj.imp().first_change_in_edit.set(false);
            obj.imp().drawing_area.borrow().queue_draw();

            let css = self.style_css();
            if !css.is_empty() {
                utils::set_style(&obj.clone(), &css);
                utils::set_style(&self.entry.borrow().clone(), &css);
            }

            let entry = self.entry.borrow().clone();
            match obj.justification() {
                0 => {
                    entry.set_justification(gtk::Justification::Left);
                }
                1 => {
                    entry.set_justification(gtk::Justification::Center);
                }
                2 => {
                    entry.set_justification(gtk::Justification::Right);
                }
                3 => {
                    entry.set_justification(gtk::Justification::Fill);
                }
                4_u32..=u32::MAX => {}
            };

            match self.align.get() {
                0 => self.entry.borrow().set_valign(gtk::Align::Start),

                1 => self.entry.borrow().set_valign(gtk::Align::Center),

                2 => self.entry.borrow().set_valign(gtk::Align::End),

                3_u32..=u32::MAX => (),
            };

            self.resize_entry();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for TextItem {}

    impl BoxImpl for TextItem {}

    impl TextItem {
        pub(super) fn placeholder_text(&self) -> String {
            let ci = self.obj().clone().upcast::<CanvasItem>();
            match ci.imp().is_presentation_mode() {
                true => " ",
                false => PLACEHOLDER_TEXT,
            }
            .to_string()
        }

        fn set_editing_(&self, value: bool) {
            self.entry.borrow().set_editable(value);
            self.entry.borrow().set_cursor_visible(value);

            if value {
                self.entry.borrow().grab_focus();
            }
        }

        fn get_editing_(&self) -> bool {
            self.entry.borrow().is_editable()
        }

        pub fn resize_entry(&self) {
            let obj = self.obj();

            gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(80), {
                let textitem = obj.downgrade().clone();
                move || {
                    if let Some(textitem) = textitem.upgrade() {
                        // WARN: this could lead to a text overwrite
                        // textitem.imp().set_text(t.to_string());
                        textitem.imp().entry.borrow().queue_resize();
                    };
                }
            });
        }

        pub(super) fn handle_unselect(&self) {
            self.obj().set_editing(false);
            self.entry.borrow().emit_select_all(false);
            self.drawing_area.borrow().queue_draw();

            self.stack.borrow().set_visible_child_name(stacks::DISPLAY);
        }

        fn style_css(&self) -> String {
            let obj = self.obj().clone();
            let ci = obj.upcast_ref::<CanvasItem>().clone();
            let Some(canvas) = ci.canvas() else {
                return String::new();
            };

            let font_size = self.obj().check_font_size();

            let converted_font_size = font_size as f64; //5.3 * canvas.current_ratio() * font_size as f64;

            if converted_font_size <= 0.0 {
                return String::new();
            }

            let font_css = obj.get_font_css(obj.font(), converted_font_size);

            let ratio = canvas.current_ratio();
            let outline = if obj.text_outline() {
                let offset = 3.0 * ratio;
                let radius = 1.0 * ratio;
                &format!(
                    "#000 {} #000 {} #000 {} #000 {}",
                    format!("-{}px -{}px {}px,", offset, offset, radius),
                    format!("{}px -{}px {}px,", offset, offset, radius),
                    format!("-{}px  {}px {}px,", offset, offset, radius),
                    format!("{}px {}px {}px", offset, offset, radius)
                )
            } else {
                "#0000 0px  0px 0px"
            };

            let shadow = if obj.text_shadow() {
                let offset = 8.0 * ratio;
                let radius = 8.0 * ratio;
                &format!("{outline}, -{}px {}px {}px black", offset, offset, radius)
            } else {
                outline
            };

            let css = format!(
                ".colored, textview.view {{
                    font: {font_css};
                    color: white;
                    padding: 0px;
                    background: 0;
                    /* letter-spacing: {}px; */
                    text-shadow: {shadow};
                }}",
                2.0 * ratio
            );

            css
        }
    }
}

glib::wrapper! {
    pub struct TextItem(ObjectSubclass<imp::TextItem>)
        @extends CanvasItem, gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for TextItem {
    fn default() -> Self {
        let ti: TextItem = glib::Object::new();
        Self::from_instance(ti)
    }
}

impl TextItem {
    pub fn new(canvas: Option<&Canvas>, save_data: Option<CanvasItemData>) -> Self {
        let data = match save_data {
            Some(d) => serde_json::to_string(&d).ok(),
            None => None,
        };

        let ti: TextItem = glib::Object::builder()
            .property("canvas", canvas)
            .property("save-data", data)
            .build();
        let ti = Self::from_instance(ti);

        ti
    }

    fn from_instance(ti: Self) -> Self {
        let binding = ti.clone();
        let imp = binding.imp();
        imp.display_font_size.set(16.0);

        ti.set_font(ApplicationSettings::get_instance().song_font());

        let buffer = gtk::TextBuffer::new(Some(&Self::build_tag_table()));

        let textview = gtk::TextView::builder()
            .justification(gtk::Justification::Center)
            .wrap_mode(gtk::WrapMode::WordChar)
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Fill)
            .can_focus(true)
            .buffer(&buffer)
            .vexpand(true)
            .hexpand(true)
            .build();

        imp.entry.replace(textview.clone());
        {
            // NOTE: this consumes click event on the textview
            // and allow double clicking on textview to highlight items
            let g = gtk::GestureClick::new();
            g.set_propagation_phase(gtk::PropagationPhase::Bubble);
            g.connect_begin(|g, _| {
                g.set_state(gtk::EventSequenceState::Claimed);
            });

            textview.add_controller(g);
        }

        *imp.stack.borrow_mut() = gtk::Stack::builder()
            .vhomogeneous(false)
            .hhomogeneous(false)
            .vexpand(true)
            .hexpand(true)
            .build();

        ti.setup_drawing_area();
        let da = imp.drawing_area.borrow().clone();

        let stack = imp.stack.borrow();
        stack.add_named(&da, Some(stacks::DISPLAY));
        stack.add_named(&imp.entry.borrow().clone(), Some(stacks::EDIT));
        stack.set_visible_child_name(stacks::DISPLAY);

        let ci: CanvasItem = ti.clone().upcast();

        if ci.imp().grid.borrow().is::<gtk::Grid>() {
            ci.imp().grid.borrow().attach(&stack.clone(), 0, 0, 1, 1);
        }

        let esc = gtk::EventControllerKey::new();
        esc.set_propagation_phase(gtk::PropagationPhase::Bubble);
        esc.connect_key_pressed({
            let obj = ti.clone();
            move |_, k, _, _| {
                let stack = obj.imp().stack.borrow();
                if k == gtk::gdk::Key::Escape
                    && let Some(name) = stack.visible_child_name()
                    && name.as_str() == stacks::EDIT
                {
                    obj.imp().handle_unselect();
                    return glib::Propagation::Stop;
                }

                glib::Propagation::Proceed
            }
        });
        ti.add_controller(esc);

        ti.connect_double_clicked(glib::clone!(
            #[weak]
            ti,
            move |_| {
                let binding = ti.clone();
                let imp = binding.imp();

                if !ti.editing() {
                    imp.first_change_in_edit.set(true);
                    ti.grab_focus();

                    let entry = imp.entry.borrow().clone();

                    let buf = entry.buffer();

                    // NOTE:
                    // When a TextBuffer is shared across views, an active selection in
                    // one view can invalidate iters held by another
                    // this resolves remove tags on active selection iters
                    if let Some((_, end)) = buf.selection_bounds() {
                        buf.place_cursor(&end);
                    }

                    if buf.markup().as_str() == PLACEHOLDER_TEXT {
                        entry.buffer().set_text("");
                    }

                    imp.stack.borrow().set_visible_child_name(stacks::EDIT);
                    ti.style();

                    glib::timeout_add_local_once(std::time::Duration::from_millis(80), {
                        let entry_clone = entry.clone();
                        move || {
                            entry_clone.grab_focus();
                            entry_clone.set_size_request(-1, -1); // gtk3 0,0
                        }
                    });
                    entry.queue_resize();
                    ti.set_editing(true);
                }
            }
        ));

        ti.connect_unselect(move |ci| {
            ci.downcast_ref::<Self>()
                .map(|ti| ti.imp().handle_unselect());
        });

        ti.set_editing(false);
        ti.load_data();

        imp.entry.borrow().buffer().connect_changed(glib::clone!(
            #[weak]
            ti,
            move |_| {
                if ti.imp().setting_text.get() {
                    return;
                }

                if let Some(name) = ti.imp().stack.borrow().visible_child_name()
                    && name.as_str() == stacks::EDIT
                {
                    ti.calculate_size();
                }

                // let action = TypedHistoryAction::item_changed(&ti, "text");
                // let Some(window) = ci.canvas().and_then(|c| c.imp().window.upgrade()) else {
                //     return;
                // };
                //
                // window
                //     .history_manager()
                //     .add_undoable_action(action.into(), Some(ti.imp().first_change_in_edit.get()));

                ti.imp().first_change_in_edit.set(false);
                ti.imp().drawing_area.borrow().queue_draw();
                ti.style();
                // ti.imp().previous_text.replace(label.label().to_string());
            }
        ));

        if let Some(canvas) = ci.canvas() {
            canvas.connect_ratio_changed({
                let ti = ti.clone();
                move |_| ti.style()
            });

            ti.style();
        }

        ti
    }

    // #[trace]
    fn get_font_css(&self, font: String, font_size: f64) -> String {
        let font_size_text = font_size.to_string().replace(",", ".");
        let style = format!("normal 400 {font_size_text}px '{font}'");

        style
    }

    pub fn buffer(&self) -> gtk::TextBuffer {
        self.imp().entry.borrow().buffer()
    }

    fn check_font_size(&self) -> f32 {
        // self.font_size()
        // self.imp().alt_font_size.get().min(self.font_size())
        self.imp().display_font_size.get()
    }

    fn build_tag_table() -> gtk::TextTagTable {
        let table = gtk::TextTagTable::new();
        let bold = gtk::TextTag::builder()
            .name(utils::text_tags::BOLD)
            .weight(700)
            .build();
        table.add(&bold);

        let italic = gtk::TextTag::builder()
            .name(utils::text_tags::ITALIC)
            .style(gtk::pango::Style::Italic)
            .build();
        table.add(&italic);

        let underline = gtk::TextTag::builder()
            .name(utils::text_tags::UNDERLINE)
            .underline(gtk::pango::Underline::Single)
            .build();
        table.add(&underline);

        table
    }

    fn setup_drawing_area(&self) {
        let da = gtk::DrawingArea::builder()
            .vexpand(true)
            .hexpand(true)
            .build();
        let ti = self.clone();

        da.set_draw_func(glib::clone!(
            #[weak]
            ti,
            move |_da, cr, _width, height| {
                let ci: CanvasItem = ti.clone().upcast();
                let Some(canvas) = ci.canvas() else {
                    return;
                };

                let Some((size, mut font_desc, layout, _, text)) = ti.calculate_size() else {
                    return;
                };

                font_desc.set_size((size * pango::SCALE as f64) as i32);
                layout.set_font_description(Some(&font_desc));
                layout.set_markup(&text);

                let (_, layout_height) = layout.pixel_size();

                let yalign_offset = match ti.align() {
                    0 => 0,                            // top
                    1 => (height - layout_height) / 2, // center
                    2 => height - layout_height,       // bottom
                    _ => 0,
                };

                let ratio = canvas.current_ratio();
                if ti.text_outline() {
                    let radius = 6.0 * ratio;

                    cr.set_source_rgba(0.0, 0.0, 0.0, 1.0);
                    cr.set_line_width(radius * 2.0);
                    cr.set_line_join(gtk::cairo::LineJoin::Round);
                    cr.move_to(0.0, yalign_offset as f64);

                    pangocairo::functions::layout_path(cr, &layout);
                    cr.stroke().ok();
                }

                if ti.text_shadow() {
                    let offset = 10.0 * ratio;

                    let rg = regex::Regex::new(r#"\s*foreground\s*=\s*"[^"]*""#).unwrap();
                    let text = text.clone();
                    let text = rg.replace_all(&text, "");

                    // NOTE: this copies all the values.
                    // below are the ones I need
                    // - font_description
                    // - width
                    // - wrap
                    // - alignment
                    let shadow_layout = layout.copy();
                    shadow_layout.set_markup(&text);

                    // draw shadow by offsetting a black copy underneath
                    cr.set_source_rgba(0.0, 0.0, 0.0, 1.0);
                    cr.move_to(offset, (yalign_offset as f64) + offset);
                    pangocairo::functions::show_layout(cr, &shadow_layout);
                }

                cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
                cr.move_to(0.0, yalign_offset as f64);
                pangocairo::functions::show_layout(cr, &layout);
            }
        ));

        // Redraw when style changes (font, size, justification etc.)
        ti.connect_font_notify(|ti| ti.imp().drawing_area.borrow().queue_draw());
        ti.connect_justification_notify(|ti| ti.imp().drawing_area.borrow().queue_draw());
        ti.connect_font_size_notify(|ti| {
            ti.imp().drawing_area.borrow().queue_draw();
            ti.calculate_size();
            ti.style();
        });
        ti.connect_align_notify(|ti| ti.imp().drawing_area.borrow().queue_draw());

        ti.imp().drawing_area.replace(da.clone());
    }

    fn calculate_size(
        &self,
    ) -> Option<(
        f64,
        pango::FontDescription,
        pango::Layout,
        pango::Alignment,
        String,
    )> {
        let ti = self.clone();
        let imp = self.imp();
        let da = imp.drawing_area.borrow();
        let width = da.width();
        let height = da.height();

        let ci: CanvasItem = ti.clone().upcast();
        let Some(canvas) = ci.canvas() else {
            return None;
        };

        let buf = ti.buffer();
        let text = buf.markup().to_string();
        let placeholder = ti.imp().placeholder_text();

        let text = match text.is_empty() {
            true => placeholder,
            false => text,
        };

        let layout = da.create_pango_layout(None);

        let px = 5.3 * canvas.current_ratio() * ti.font_size() as f64;

        let mut font_desc = pango::FontDescription::new();
        font_desc.set_family(&ti.font());
        font_desc.set_size((px * pango::SCALE as f64) as i32);
        layout.set_font_description(Some(&font_desc));
        layout.set_markup(&text);

        layout.set_width(width * pango::SCALE);
        layout.set_wrap(pango::WrapMode::WordChar);

        // Match justification
        let alignment = match ti.justification() {
            0 => pango::Alignment::Left,
            1 => pango::Alignment::Center,
            2 => pango::Alignment::Right,
            _ => pango::Alignment::Left,
        };
        layout.set_alignment(alignment);

        let size = fit_text_layout(&layout, px, width, height);

        ti.imp().display_font_size.set(size as f32);

        Some((size, font_desc, layout, alignment, text))
    }
}

fn fit_text_layout(layout: &pango::Layout, max_px: f64, width: i32, height: i32) -> f64 {
    if width <= 0 || height <= 0 {
        return max_px;
    }

    let mut lo = 6.0_f64;
    let mut hi = max_px;
    let mut best = lo;

    for _ in 0..20 {
        let mid = (lo + hi) / 2.0;

        // temporarily set size to measure
        if let Some(mut fd) = layout.font_description() {
            fd.set_size((mid * pango::SCALE as f64) as i32);
            layout.set_font_description(Some(&fd));
        }

        let (pw, ph) = layout.pixel_size();
        if pw <= width && ph <= height {
            best = mid;
            lo = mid;
        } else {
            hi = mid;
        }
    }

    best
}
