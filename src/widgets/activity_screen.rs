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
        let text_len = text.len() as f64;
        if text_len.eq(&0.0) {
            return;
        }

        let (_x, _y, w, h) = match self.screen_label.bounds() {
            Some(r) => r,
            None => return,
        };
        let max_font_size = calculate_max_font_size_for_rect(w, h, text_len);

        let fixed_font = 14.0;
        let max_set_len = calc_max_len(w, h, fixed_font);
        let dyn_font_size = calculate_max_font_size_for_rect(w, h, max_set_len);
        println!("FONT-SIZE {max_font_size}, {dyn_font_size}, len = {max_set_len}");

        self.screen_label.inline_css(&format!(
            "font-size: {}px",
            f64::min(dyn_font_size, max_font_size)
        ));
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
            // if let Some(t) = model_c.screen_label.toplevel_window() {
            //     if t.is_maximized() {
            //         return;
            //     }
            // }

            model_c.resize_font();
            println!("RESIZE {_w}, {_h}");
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

/// Calculate max-font-size with an unsigned float.
/// Computes `cbrt(area / txt.len()) * 2`,
/// returning the the max-font-size for a  given area
fn calculate_max_font_size_for_rect(w: i32, h: i32, text_length: f64) -> f64 {
    let w = w.saturating_sub(10);
    let h = h.saturating_sub(10);
    let area = w.saturating_mul(h) as f64;
    let max_font_size = (area / text_length).cbrt() * 2.0;

    return max_font_size;
}

/// Calculate for the maximum number of text needed to fill a rect
/// with a given font-size
/// ```
/// a = area
/// l = length
/// m = max font size
/// m = ³√(a/l) * 2.0
///
/// find l when m = 10
/// m = ³√(a/l) * 2
/// l = 4a/m³
/// ```
fn calc_max_len(w: i32, h: i32, font_size: f64) -> f64 {
    let area = (w * h) as f64;
    return (8.0 * area) / (font_size.powf(3.0));
}
