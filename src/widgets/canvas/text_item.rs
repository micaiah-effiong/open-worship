use std::{ops::Sub, sync::Once};

use gtk::{
    glib::{
        self, clone,
        object::{Cast, ObjectExt},
        subclass::types::ObjectSubclassIsExt,
    },
    prelude::{
        AdjustmentExt, BoxExt, EventControllerExt, GestureExt, GridExt, TextBufferExt, TextViewExt,
        WidgetExt,
    },
};

use crate::{
    // services::history_manager::history_action::TypedHistoryAction,
    services::settings::ApplicationSettings,
    utils::{self, TextBufferExtraExt, buffer_markup::TextBufferExtra},
    widgets::canvas::{
        canvas::Canvas,
        canvas_item::{CanvasItem, CanvasItemExt},
        serialise::CanvasItemData,
    },
};

const PLACEHOLDER_TEXT: &str = "Click to add text...";

mod imp {
    use std::cell::{Cell, RefCell};
    use std::u32;

    use glib::subclass::object::ObjectImpl;
    use glib::subclass::types::ObjectSubclass;
    use gtk::glib::{self, Properties, prelude::*};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use super::*;
    use crate::utils::{self, WidgetExtrasExt};
    use crate::widgets::canvas::canvas_item::{CanvasItem, CanvasItemExt, CanvasItemImpl};
    use crate::widgets::canvas::serialise::{CanvasItemType, TextItemData};
    use crate::widgets::extended_screen::ExtendedScreen;

    #[derive(Properties, Debug, Default)]
    #[properties(wrapper_type = super::TextItem)]
    pub struct TextItem {
        pub entry: RefCell<gtk::TextView>,
        pub label: RefCell<gtk::Label>,
        pub stack: RefCell<gtk::Stack>,

        #[property(get, set, default_value = 1, construct)]
        pub justification: Cell<u32>,
        #[property(get, set, default_value = 1, construct)]
        pub align: Cell<u32>,
        #[property(get, set, default_value = 16.0, construct)]
        pub font_size: Cell<f32>,
        pub alt_font_size: Cell<f32>,
        #[property(get, set, default_value = "", construct)]
        pub font: RefCell<String>,
        #[property(get, set, default_value = false, construct)]
        pub text_shadow: Cell<bool>,
        #[property(get, set, default_value = false, construct)]
        pub text_outline: Cell<bool>,

        pub setting_text: Cell<bool>,
        pub first_change_in_edit: Cell<bool>,
        pub previous_text: RefCell<String>,

        #[property(get=Self::get_editing_, set=Self::set_editing_)]
        pub editing: Cell<bool>,

        pub(super) label_scroll: RefCell<gtk::ScrolledWindow>,
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
                self.set_text(t);
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
            let font_size = self.obj().check_font_size();

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
            let obj = self.obj().clone();

            {
                // NOTE: allow label to capture tag update from buffer
                obj.imp().first_change_in_edit.set(false);

                let text = obj.buffer().markup();
                let label = obj.imp().label.borrow();
                if text.is_empty() {
                    label.set_markup(&obj.imp().placeholder_text());
                } else {
                    label.set_markup(&text);
                }
                obj.imp().previous_text.replace(text.clone());
            }

            let css = self.style_css();
            if !css.is_empty() {
                utils::set_style(&obj.clone(), &css);
                utils::set_style(&self.entry.borrow().clone(), &css);
            }

            let label = self.label.borrow().clone();
            let entry = self.entry.borrow().clone();
            match obj.justification() {
                0 => {
                    entry.set_justification(gtk::Justification::Left);
                    label.set_justify(gtk::Justification::Left);
                    label.set_halign(gtk::Align::Start);
                    label.set_xalign(0.0);
                }
                1 => {
                    entry.set_justification(gtk::Justification::Center);
                    label.set_justify(gtk::Justification::Center);
                    label.set_halign(gtk::Align::Center);
                    label.set_xalign(0.5);
                }
                2 => {
                    entry.set_justification(gtk::Justification::Right);
                    label.set_justify(gtk::Justification::Right);
                    label.set_halign(gtk::Align::End);
                    label.set_xalign(1.0);
                }
                3 => {
                    entry.set_justification(gtk::Justification::Fill);
                    label.set_justify(gtk::Justification::Fill);
                    label.set_halign(gtk::Align::Fill);
                    label.set_xalign(0.0);
                }
                4_u32..=u32::MAX => {}
            };

            match self.align.get() {
                0 => {
                    self.entry.borrow().set_valign(gtk::Align::Start);
                    self.label.borrow().set_valign(gtk::Align::Start);
                }
                1 => {
                    self.entry.borrow().set_valign(gtk::Align::Center);
                    self.label.borrow().set_valign(gtk::Align::Center);
                }
                2 => {
                    self.entry.borrow().set_valign(gtk::Align::End);
                    self.label.borrow().set_valign(gtk::Align::End);
                }
                3_u32..=u32::MAX => {}
            };

            self.resize_entry();
        }

        fn style_css(&self) -> String {
            let obj = self.obj().clone();
            let ci = obj.upcast_ref::<CanvasItem>().clone();
            let Some(canvas) = ci.canvas() else {
                return String::new();
            };

            let font_size = self.obj().check_font_size();

            let converted_font_size = 5.3 * canvas.current_ratio() * font_size as f64;

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
                &format!(
                    "{outline}, -{}px {}px {}px black",
                    8.0 * ratio,
                    8.0 * ratio,
                    12.0 * ratio
                )
            } else {
                outline
            };

            let css = format!(
                ".colored, textview.view {{
                    font: {font_css};
                    color: white;
                    padding: 0px;
                    background: 0;
                    letter-spacing: {}px;
                    text-shadow: {shadow};
                }}",
                2.0 * ratio
            );

            css
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for TextItem {}

    impl BoxImpl for TextItem {}

    impl TextItem {
        // fn get_text_(&self) -> String {
        //     self.previous_text.borrow().to_string()
        // }

        pub(super) fn placeholder_text(&self) -> String {
            let ci = self.obj().clone().upcast::<CanvasItem>();
            match ci.imp().is_presentation_mode() {
                true => " ",
                false => PLACEHOLDER_TEXT,
            }
            .to_string()
        }

        #[doc = "Does not set text in entry widget"]
        pub(super) fn set_text(&self, value: String) {
            self.setting_text.set(true);

            let val = if value.is_empty() {
                self.placeholder_text()
            } else {
                value
            };
            self.label.borrow().set_markup(&val);

            self.previous_text
                .replace(self.label.borrow().label().to_string());
            self.setting_text.set(false);
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
            // let text = self.entry.borrow().buffer().full_text();

            gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(80), {
                let textitem = obj.downgrade().clone();
                // let t = text.clone();
                move || {
                    let Some(textitem) = textitem.upgrade() else {
                        return;
                    };
                    // WARN: this could lead to a text overwrite
                    // textitem.imp().set_text(t.to_string());
                    textitem.imp().entry.borrow().queue_resize();
                }
            });
        }

        // TODO: remove or review
        pub(super) fn update_font_scale(&self) {
            // let obj = self.obj();
            // let Some(widget) = self.stack.borrow().clone().visible_child() else {
            //     return;
            // };
            // let text = self.entry.borrow().buffer().full_text().to_string();
            // let ci: CanvasItem = obj.clone().upcast();
            // let Some(canvas) = ci.canvas() else { return };
            //
            // let size = Self::calculate_font_scale(
            //     &widget,
            //     text.clone(),
            //     obj.font_size() as f32,
            //     canvas.current_ratio() as f32,
            //     ci.rectangle(),
            //     // utils::rect::Rect::new(0, 0, ci.width(), ci.height()),
            // );
            //
            // self.alt_font_size.set(size);
            // println!(
            //     "check = {}, alt = {size} prev = {} curr_text = {}",
            //     obj.check_font_size(),
            //     self.previous_text.borrow().len(),
            //     text.len()
            // );
            // // println!("{}=>{},{}", obj.font_size(), size, canvas.current_ratio());
            // obj.style();
        }

        // TODO: remove or review
        pub fn calculate_font_scale(
            widget: &impl IsA<gtk::Widget>,
            text: String,
            desired_font_size: f32,
            scale: f32,
            w_rect: utils::rect::Rect,
        ) -> f32 {
            let (w, h) = (w_rect.width as f32 * scale, w_rect.height as f32 * scale);
            let layout = {
                let layout = widget.create_pango_layout(Some(&text));
                layout.set_width((w * gtk::pango::SCALE as f32) as i32);
                layout.set_height((h * gtk::pango::SCALE as f32) as i32);

                println!(
                    "\n\n==========\n\nlayout font_description = {:?}",
                    layout.font_description()
                );
                // layout.set_height((h * gtk::pango::SCALE as f32) as i32);

                layout

                // let mut font_desc = gtk::pango::FontDescription::new();
                // font_desc.set_style(pango::Style::Normal);
                // font_desc.set_weight(pango::Weight::Normal);
                // font_desc.set_size(desired_font_size as i32 * gtk::pango::SCALE);
                //
                // layout.set_font_description(Some(&font_desc));
                // layout.set_alignment(pango::Alignment::Center);
                // layout.set_width((w * gtk::pango::SCALE as f32) as i32);
                // layout.set_height((h * gtk::pango::SCALE as f32) as i32);
                // layout.set_wrap(gtk::pango::WrapMode::WordChar);
                // layout.set_text(&text);
                // layout
            };

            let (layout_rect, _) = layout.pixel_extents();
            let layout_w = layout_rect.width() as f32 /* / gtk::pango::SCALE */ ;
            let layout_h = layout_rect.height()  as f32 /* / gtk::pango::SCALE */;
            // println!("bound={w} x {h}");

            //  Calculate scaling factors for width and height
            let w_scale = if layout_w > 0.0 {
                w as f32 / layout_w
            } else {
                1.0
            };
            let h_scale = if layout_h > 0.0 {
                h as f32 / layout_h
            } else {
                1.0
            };

            // get the minimum scale
            let font_scale_factor = f32::min(w_scale, h_scale);

            if !widget
                .toplevel_window()
                .and_downcast::<ExtendedScreen>()
                .is_some()
            {
                println!("w={w}, h={h}");
                println!("layout={layout_w} x {layout_h}");
                println!(
                    "factor={font_scale_factor}, desired_font_size={desired_font_size}, should_scale={}",
                    w / h
                );
                println!(
                    "font_size=16, alt={}",
                    font_scale_factor * desired_font_size
                );
            }

            // calculate the scaled font size
            let final_size = if font_scale_factor < 1.0 {
                desired_font_size * font_scale_factor //.max(8.0) // Don't go below 8pt
            } else {
                desired_font_size
            };

            // println!(
            //     "Scale factor: {font_scale_factor}, Scale ration {scale}, Desired font size: {desired_font_size} Final size: {final_size}"
            // );

            final_size
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
        imp.alt_font_size.set(16.0);

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

        *imp.label.borrow_mut() = gtk::Label::builder()
            .label(imp.placeholder_text())
            .wrap_mode(gtk::pango::WrapMode::WordChar)
            .vexpand(true)
            .hexpand(true)
            .wrap(true)
            .valign(gtk::Align::Fill)
            .halign(gtk::Align::Fill)
            .build();

        *imp.stack.borrow_mut() = gtk::Stack::builder()
            .vhomogeneous(false)
            .hhomogeneous(false)
            .vexpand(true)
            .hexpand(true)
            .build();

        let label_box = gtk::Box::default();
        label_box.append(&imp.label.borrow().clone());
        let label_box_scroll = gtk::ScrolledWindow::new();
        label_box_scroll.set_child(Some(&label_box));
        imp.label_scroll.replace(label_box_scroll.clone());

        let stack = imp.stack.borrow();
        stack.add_named(&label_box_scroll, Some("label"));
        stack.add_named(&imp.entry.borrow().clone(), Some("entry"));
        stack.set_visible_child_name("label");

        let ci: CanvasItem = ti.clone().upcast();

        if ci.imp().grid.borrow().is::<gtk::Grid>() {
            ci.imp().grid.borrow().attach(&stack.clone(), 0, 0, 1, 1);
        }

        ti.connect_clicked(clone!(
            #[weak]
            ti,
            move |_| {
                let binding = ti.clone();
                let imp = binding.imp();

                if !ti.editing() {
                    imp.first_change_in_edit.set(true);

                    let entry = imp.entry.borrow();

                    let buf = entry.buffer();
                    let text = buf.markup();

                    // NOTE:
                    // When a TextBuffer is shared across views, an active selection in
                    // one view can invalidate iters held by another
                    // this resolves remove tags on active selection iters
                    if let Some((_, end)) = buf.selection_bounds() {
                        buf.place_cursor(&end);
                    }

                    if text.as_str() == PLACEHOLDER_TEXT {
                        entry.buffer().set_text("");
                    }

                    imp.stack.borrow().set_visible_child_name("entry");

                    let w = imp.label.borrow().width();
                    let h = imp.label.borrow().height();

                    entry.set_size_request(w, h);

                    glib::timeout_add_local_once(std::time::Duration::from_millis(80), {
                        let entry_clone = entry.clone();
                        move || {
                            entry_clone.set_size_request(-1, -1); // gtk3 0,0
                        }
                    });
                    entry.queue_resize();
                    ti.set_editing(true);
                }
            }
        ));

        ti.connect_unselect(clone!(
            #[weak]
            ti,
            move |_| {
                ti.set_editing(false);
                let entry = ti.imp().entry.borrow().clone();
                entry.emit_select_all(false);

                let text = entry.buffer().markup();
                ti.imp().set_text(text.to_string());

                ti.imp().stack.borrow().set_visible_child_name("label");
            }
        ));

        ti.set_editing(false);
        ti.load_data();

        imp.entry.borrow().buffer().connect_changed(clone!(
            #[weak]
            ti,
            // #[weak]
            // ci,
            move |buf| {
                if ti.imp().setting_text.get() {
                    return;
                }

                ti.imp().update_font_scale();

                if ti.imp().previous_text.borrow().len().saturating_sub(10) > buf.full_text().len()
                {
                    ti.imp().alt_font_size.set(ti.font_size());
                    ti.style();
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

                let text = buf.markup();
                let label = ti.imp().label.borrow();
                if text.is_empty() {
                    label.set_markup(&ti.imp().placeholder_text());
                } else {
                    label.set_markup(&text);
                }
                ti.imp().previous_text.replace(text.clone());
                // ti.imp().previous_text.replace(label.label().to_string());
            }
        ));

        // Ensures the font scales correctly when the canvas ratio changes
        // after the initial mount. The scaling logic is executed only once
        // for ratio change to avoid repeatedly recalculating the font scale.
        // Without initial font may appear too small due to the initial ratio.
        let scale_font = Once::new();
        if let Some(canvas) = ci.canvas() {
            canvas.connect_ratio_changed({
                let ti = ti.clone();
                move |_| {
                    ti.style();
                    scale_font.call_once(|| {
                        ti.imp().update_font_scale();

                        let label_box_scroll = ti.imp().label_scroll.borrow();
                        let vadj = label_box_scroll.vadjustment();

                        let ti = ti.clone();
                        vadj.connect_changed(move |adj| {
                            let overflow = adj.upper() > adj.page_size();

                            if overflow {
                                let size = ti.imp().alt_font_size.get();
                                ti.imp().alt_font_size.set(size.sub(1.0));
                                ti.style();
                            }
                        });
                    });
                }
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
        self.imp().alt_font_size.get().min(self.font_size())
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
}
