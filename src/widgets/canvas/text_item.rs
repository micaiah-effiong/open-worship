use gtk::{
    glib::{
        self, clone,
        object::{Cast, ObjectExt},
        subclass::types::ObjectSubclassIsExt,
    },
    pango,
    prelude::{EventControllerExt, GestureExt, GridExt, TextBufferExt, TextViewExt, WidgetExt},
};

use crate::{
    // services::history_manager::history_action::TypedHistoryAction,
    utils::TextBufferExtraExt,
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
    use crate::utils::{self, TextBufferExtraExt};
    use crate::widgets::canvas::canvas_item::{CanvasItem, CanvasItemExt, CanvasItemImpl};
    use crate::widgets::canvas::serialise::{CanvasItemType, TextItemData};

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
        #[property(get, set, default_value = 16, construct)]
        pub font_size: Cell<u32>,
        pub alt_font_size: Cell<u32>,
        // pub display_font_size: Cell<u32>,
        #[property(get, set, default_value = "Open Sans", construct)]
        pub font: RefCell<String>,
        #[property(get, set, default_value = "#fff", construct)]
        pub font_color: RefCell<String>,
        #[property(get, set, default_value = "normal", construct)]
        pub font_style: RefCell<String>,
        #[property(get, set, default_value = "regular", construct)]
        pub font_weight: RefCell<String>,
        #[property(get, set, default_value = false, construct)]
        pub text_underline: Cell<bool>,
        #[property(get, set, default_value = false, construct)]
        pub text_shadow: Cell<bool>,
        #[property(get, set, default_value = false, construct)]
        pub text_outline: Cell<bool>,

        pub setting_text: Cell<bool>,
        pub first_change_in_edit: Cell<bool>,
        pub previous_text: RefCell<String>,

        // #[property(get=Self::get_text_, set=Self::set_text_)]
        // pub text: RefCell<String>,
        #[property(get=Self::get_editing_, set=Self::set_editing_)]
        pub editing: Cell<bool>,
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

            println!("TEXT_DATA {:?}", text_data);

            let t = text_data.text_data.clone();
            if !t.is_empty()
                && let Ok(t) = String::from_utf8(glib::base64_decode(&t))
            {
                self.entry.borrow().buffer().set_text(&t);
                self.set_text(t);
            }

            self.obj().set_font_size(text_data.font_size);
            self.obj().set_font(text_data.font);
            self.obj().set_font_style(text_data.font_style);
            self.obj().set_justification(text_data.justification);
            self.obj().set_align(text_data.align);
            self.obj().set_font_color(text_data.color);
            self.obj().set_font_weight(text_data.font_weight);
            self.obj().set_text_underline(text_data.text_underline);
            self.obj().set_text_outline(text_data.text_outline);
            self.obj().set_text_shadow(text_data.text_shadow);
        }

        fn serialise_item(&self) -> CanvasItemType {
            let entry = self.entry.borrow();

            let text = entry.buffer().full_text();

            let encoded = glib::base64_encode(text.as_bytes()).to_string();

            let obj = self.obj();

            let data = TextItemData {
                font: obj.font(),
                color: obj.font_color(),
                font_size: obj.font_size(),
                font_style: obj.font_style(),
                font_weight: obj.font_weight(),
                justification: obj.justification(),
                align: obj.align(),
                text_data: encoded.clone(),
                text_underline: obj.text_underline(),
                text_outline: obj.text_outline(),
                text_shadow: obj.text_shadow(),
            };
            let data = CanvasItemType::Text(data);

            data

            // format!(
            //     "\"type\":\"text\",\"text\": \"\",\"text-data\": \"{}\",\"font\": \"{}\",\"color\": \"{}\",\"font-size\": {}, \"font-style\":\"{}\", \"justification\": {}, \"align\": {} ",
            //     encoded,
            //     self.obj().font(),
            //     self.obj().font_color(),
            //     self.obj().font_size(),
            //     self.obj().font_style(),
            //     self.obj().justification(),
            //     self.obj().align()
            // )
        }

        fn style(&self) {
            let obj = self.obj().clone();

            {
                let ci: CanvasItem = obj.clone().upcast();
                let Some(canvas) = ci.canvas() else { return };
                let tv = obj.imp().entry.borrow();

                let f = Self::calculate_font_scale(
                    &tv,
                    tv.buffer().full_text().to_string(),
                    obj.font_size() as f32,
                    canvas.current_ratio() as f32,
                    ci.rectangle(),
                ) as u32;

                obj.imp().alt_font_size.set(f);
            }
            // glib::g_message!("TextItem", "FONT CSS \n{css}");

            let css = self.style_css();
            if !css.is_empty() {
                utils::set_style(&obj.clone(), &css);
                utils::set_style(&self.entry.borrow().clone(), &css);
            }
            // glib::g_message!("TextItem", "color {}", self.obj().font_color());

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

            let font_css = obj.get_font_css(
                obj.font(),
                obj.font_style(),
                obj.font_weight(),
                converted_font_size,
            );

            let underline = if obj.text_underline() {
                "underline"
            } else {
                "none"
            };

            let outline = if obj.text_outline() {
                "#000 -1px -1px 1px, 
                #000 1px -1px 1px, 
                #000 -1px  1px 1px,
                #000 1px  1px 1px"
            } else {
                "#0000 0px  0px 0px"
            };

            let shadow = if obj.text_shadow() {
                &format!("{outline}, -2px 2px 1px black")
            } else {
                outline
            };

            let css = format!(
                ".colored, textview.view {{
                        color: {};
                        font: {font_css};
                        padding: 0px;
                        background: 0;
                        text-decoration: {underline} {};
                        text-shadow: {shadow};
                    }}",
                obj.font_color(),
                obj.font_color(),
            );
            glib::g_message!("TextItem", "FONT CSS \n{css}");

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

            if value.is_empty() {
                self.label.borrow().set_label(&self.placeholder_text());
            } else {
                self.label.borrow().set_label(&value);
            }

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
            let text = self.entry.borrow().buffer().full_text();

            gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(80), {
                let textitem = obj.downgrade().clone();
                let t = text.clone();
                move || {
                    let Some(textitem) = textitem.upgrade() else {
                        return;
                    };
                    textitem.imp().set_text(t.to_string());
                    textitem.imp().entry.borrow().queue_resize();
                }
            });
        }

        fn calculate_font_scale(
            // &self,
            widget: &gtk::TextView,
            text: String,
            desired_font_size: f32,
            scale: f32,
            w_rect: utils::rect::Rect,
        ) -> f32 {
            let ctx = widget.pango_context();

            let family = "Tahoma";
            let (w, h) = (w_rect.width as f32 * scale, w_rect.height as f32 * scale);
            let layout = {
                let layout = gtk::pango::Layout::new(&ctx);

                let mut font_desc = gtk::pango::FontDescription::new();
                font_desc.set_family(family);
                font_desc.set_style(pango::Style::Normal);
                font_desc.set_weight(pango::Weight::Normal);
                font_desc.set_size(desired_font_size as i32 * gtk::pango::SCALE);

                println!("w={w}, h={h}");
                layout.set_font_description(Some(&font_desc));
                layout.set_alignment(pango::Alignment::Left);
                layout.set_width(w as i32 * gtk::pango::SCALE);
                // layout.set_height(h as i32 * gtk::pango::SCALE);
                layout.set_wrap(gtk::pango::WrapMode::WordChar);
                layout.set_text(&text);
                layout
            };

            let (_, layout_rect) = layout.pixel_extents();
            let layout_w = layout_rect.width() as f32 /* / gtk::pango::SCALE */ ;
            let layout_h = layout_rect.height()  as f32 /* / gtk::pango::SCALE */;
            // println!("bound={w} x {h}");
            // println!("layout={layout_w} x {layout_h}");

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
        imp.alt_font_size.set(16);

        let textview = gtk::TextView::builder()
            .justification(gtk::Justification::Center)
            .wrap_mode(gtk::WrapMode::WordChar)
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Fill)
            .can_focus(true)
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
            .build();

        *imp.stack.borrow_mut() = gtk::Stack::builder()
            .vhomogeneous(false)
            .hhomogeneous(false)
            .vexpand(true)
            .hexpand(true)
            .build();

        let stack = imp.stack.borrow();
        stack.add_named(&imp.label.borrow().clone(), Some("label"));
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
                glib::g_log!(
                    "TextItem",
                    glib::LogLevel::Message,
                    "RUNING connect_clicked"
                );

                let binding = ti.clone();
                let imp = binding.imp();

                if !ti.editing() {
                    imp.first_change_in_edit.set(true);

                    let entry = imp.entry.borrow();

                    let buf = entry.buffer();
                    let text = buf.full_text();

                    glib::g_message!("TextItem", "{text}");
                    if text.as_str() == PLACEHOLDER_TEXT {
                        entry.buffer().set_text("");
                    }

                    imp.stack.borrow().set_visible_child_name("entry");

                    let w = imp.label.borrow().width();
                    let h = imp.label.borrow().height();
                    glib::g_message!(
                        "TextItem",
                        "width: {w}, height: {h}\na_width: {}, a_height: {}",
                        imp.label.borrow().width(),
                        imp.label.borrow().height()
                    );

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

                let text = entry.buffer().full_text();
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

                ti.style();

                // let action = TypedHistoryAction::item_changed(&ti, "text");
                // let Some(window) = ci.canvas().and_then(|c| c.imp().window.upgrade()) else {
                //     return;
                // };
                //
                // window
                //     .history_manager()
                //     .add_undoable_action(action.into(), Some(ti.imp().first_change_in_edit.get()));

                ti.imp().first_change_in_edit.set(false);

                let text = buf.full_text().to_string();
                let label = ti.imp().label.borrow();
                if text.is_empty() {
                    label.set_label(&ti.imp().placeholder_text());
                } else {
                    label.set_label(&text);
                }
                // ti.imp().previous_text.replace(text.clone());
                ti.imp().previous_text.replace(label.label().to_string());
            }
        ));

        if let Some(canvas) = ci.canvas() {
            canvas.connect_ratio_changed({
                let ti_clone = ti.clone();
                move |_| ti_clone.style()
            });

            ti.style();
        }

        ti
    }

    // #[trace]
    fn get_font_css(
        &self,
        font: String,
        font_style: String,
        font_weight: String,
        font_size: f64,
    ) -> String {
        let font_size_text = font_size.to_string().replace(",", ".");
        let font_style = font_style.to_lowercase();
        let font_weight = font_weight.to_lowercase();

        let mut font_weight = font_weight.replace("black", "900");
        font_weight = font_weight.replace("extrabold", "800");
        font_weight = font_weight.replace("semibold", "600");
        font_weight = font_weight.replace("bold", "700");
        font_weight = font_weight.replace("medium", "500");
        font_weight = font_weight.replace("regular", "400");
        font_weight = font_weight.replace("extralight", "300");
        font_weight = font_weight.replace("light", "200");
        font_weight = font_weight.replace("thin", "100");

        let style = format!("{font_style} {font_weight} {font_size_text}px '{font}'",);

        println!("{style}");

        style
    }

    pub fn buffer(&self) -> gtk::TextBuffer {
        self.imp().entry.borrow().buffer()
    }

    fn check_font_size(&self) -> u32 {
        self.imp().alt_font_size.get().min(self.font_size())
    }
}
