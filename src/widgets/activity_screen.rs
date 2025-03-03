use std::{cell::RefCell, rc::Rc};

use gtk::{glib::property::PropertySet, prelude::*};
use relm4::prelude::*;

use crate::dto;

// actrivity screen
#[derive(Debug)]
pub enum ActivityScreenInput {
    DisplayUpdate(dto::DisplayPayload),
    DisplayBackground(String),
    ClearDisplay,
}

#[derive(Debug, Clone)]
pub struct ActivityScreenModel {
    display_text: Rc<RefCell<String>>,
    background_image: Rc<RefCell<Option<String>>>,
    bg_style: String,
    is_cleared: bool,
    screen: gtk::Box,
    screen_label: gtk::Label,
}

const MIN_GRID_HEIGHT: i32 = 300;

impl ActivityScreenModel {
    fn format_bg_style(image: String) -> String {
        let mut style = format!(
            "background-size: cover; background-position: center center; background-color: black;",
        );

        if !image.is_empty() {
            let bg_image_style = format!("background-image: url(\"file://{}\");", image);
            style = style + &bg_image_style;
        }

        return style;
    }

    fn update_display_image(&mut self, image_src: String) {
        if image_src.is_empty() {
            return;
        }

        println!("display bg {image_src}");
        self.background_image.set(Some(image_src));
        {
            let bg = self.background_image.borrow().clone();
            if let Some(img) = bg {
                self.bg_style = ActivityScreenModel::format_bg_style(img);
                println!("bg-style {}", self.bg_style);
            }
        }
    }

    fn _get_user_text_size(text: String, _user_font_size: f64) -> f64 {
        let text_lines = text.lines();
        let text_w = text_lines.clone().count();
        let text_h = text_lines.clone().fold(0, |acc, e| acc.max(e.len().into()));
        let text_len = text.len();

        let size = text_w.saturating_mul(text_h).saturating_div(text_len);

        return size as f64;
    }

    fn resize_font(&self) {
        let text = self.display_text.borrow();
        println!("RESIZE TEXT LEN {}", text.len());

        let text_len = text.len() as f64;
        let font_size = match self.screen_label.bounds() {
            Some((_x, _y, w, h)) => calculate_max_font_size_for_rect(w, h, text_len),
            None => 1.0,
        };

        // let font_size = font_size.max(user_font_size);

        let mut desc = gtk::pango::FontDescription::new();
        desc.set_size((font_size as i32).saturating_mul(gtk::pango::SCALE));
        let attrs = gtk::pango::AttrList::new();
        let attr = gtk::pango::AttrFontDesc::new(&desc);
        attrs.insert(attr);

        self.screen_label.set_attributes(Some(&attrs));
    }
}

#[relm4::component(pub)]
impl SimpleComponent for ActivityScreenModel {
    type Init = ();
    type Input = ActivityScreenInput;
    type Output = ();

    view! {
        #[root]
        gtk::Frame {
            set_height_request: MIN_GRID_HEIGHT,
            add_css_class: "gray_bg_box",
            set_hexpand: true,
            set_vexpand: true,

            #[wrap(Some)]
            set_child = &gtk::AspectFrame {
                add_css_class: "pink_box",
                // set_homogeneous: true,
                // set_valign:gtk::Align::Center,
                // set_halign:gtk::Align::Center,
                set_ratio: 16.0 / 9.0,
                set_obey_child:false,

                #[name="screen_overlay"]
                #[wrap(Some)]
                set_child = &gtk::Overlay {


                    set_css_classes: &["green_box"],

                    #[local_ref]
                    #[wrap(Some)]
                    set_child = &screen_box -> gtk::Box {
                        // set_homogeneous: true,
                        set_css_classes: &[/* "brown_box", */  "fade-in-image", "black_bg_box" ],
                        set_overflow: gtk::Overflow::Hidden,
                        set_width_request: 200,

                        #[watch]
                        inline_css: &model.bg_style,


                        #[local_ref]
                        append = &screen_label -> gtk::Label {
                            #[watch]
                            set_markup: &model.display_text.borrow(),

                            set_overflow: gtk::Overflow::Hidden,
                            set_justify: gtk::Justification::Center,
                            // set_align: gtk::Align::Center,
                            set_wrap: true,
                            set_vexpand: true,
                            set_hexpand: true,
                            set_wrap_mode: gtk::pango::WrapMode::Word,
                            set_css_classes: &["white", /*"yellow_box",   "screen_label" */]
                        },
                    }
                },
            }

        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let screen_box = gtk::Box::builder().build();
        let screen_label = gtk::Label::builder().build();

        let model = ActivityScreenModel {
            display_text: Rc::new(RefCell::new(String::new())),
            background_image: Rc::new(RefCell::new(None)),
            bg_style: ActivityScreenModel::format_bg_style(String::new()),
            is_cleared: false,
            screen: screen_box.clone(),
            screen_label: screen_label.clone(),
        };

        let widgets = view_output!();

        let d = gtk::DrawingArea::new();
        widgets.screen_overlay.add_overlay(&d);
        let model_c = model.clone();
        d.connect_resize(move |_, _w, _h| {
            model_c.resize_font();
        });

        return relm4::ComponentParts { model, widgets };
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            ActivityScreenInput::DisplayUpdate(display_data) => {
                *self.display_text.borrow_mut() = display_data.text;
                self.resize_font();

                if let Some(img) = display_data.background_image {
                    self.update_display_image(img);
                }
            }
            ActivityScreenInput::DisplayBackground(image_src) => {
                self.update_display_image(image_src);
            }
            ActivityScreenInput::ClearDisplay => {
                self.is_cleared = !self.is_cleared;
                self.screen_label.set_visible(!self.is_cleared);
            }
        };
    }
}

fn calculate_max_font_size_for_rect(w: i32, h: i32, text_length: f64) -> f64 {
    let w = w.saturating_add(10);
    let h = h.saturating_add(10);
    let area = w.saturating_mul(h) as f64;
    let max_font_size = (area / text_length).sqrt();

    return max_font_size;
}
