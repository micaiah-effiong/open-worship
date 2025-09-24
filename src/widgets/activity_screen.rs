use pango::{FontDescription, Layout, WrapMode};
use std::cell::RefCell;
use std::rc::Rc;

use gtk::pango;
use gtk::prelude::*;
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
    // screen: gtk::Box,
    screen_drawing_area: gtk::DrawingArea,
    // screen_label: gtk::Label,
}

const MIN_GRID_HEIGHT: i32 = 300;

impl ActivityScreenModel {
    fn format_bg_style(image: String) -> String {
        let mut style = "background-size: cover;
            background-position: center center;
            background-color: black;"
            .to_string();

        if !image.is_empty() {
            let bg_image_style = format!("background-image: url(\"file://{}\");", image);
            style = style + &bg_image_style;
        }

        style
    }

    fn update_display_image(&mut self, image_src: String) {
        if image_src.is_empty() {
            return;
        }

        println!("display bg {image_src}");
        *self.background_image.borrow_mut() = Some(image_src);
        {
            let bg = self.background_image.borrow().clone();
            if let Some(img) = bg {
                self.bg_style = ActivityScreenModel::format_bg_style(img);
                // println!("bg-style {}", self.bg_style);
            }
        }
    }

    fn register_drawing_screen(&self) {
        let text = self.display_text.clone();
        // let label = self.screen_label.clone();
        self.screen_drawing_area
            .set_draw_func(move |_area, ctx, width, height| {
                // Background
                ctx.set_source_rgb(0.0, 0.0, 0.0);
                ctx.rectangle(0.0, 0.0, width as f64, height as f64);
                match ctx.fill() {
                    Ok(_) => (),
                    Err(e) => eprintln!("{:?}", e),
                }

                // Padding inside the board
                let padding = 16.0;
                let avail_w = (width as f64).max(1.0) - 2.0 * padding;
                let avail_h = (height as f64).max(1.0) - 2.0 * padding;
                if avail_w <= 0.0 || avail_h <= 0.0 {
                    return; // nothing to draw
                }

                // Create a Pango layout for measuring and drawing
                let pango_ctx = pangocairo::functions::create_context(ctx); //.expect("create pango context");
                let layout = Layout::new(&pango_ctx);
                layout.set_alignment(pango::Alignment::Center);
                layout.set_wrap(WrapMode::Word);
                layout.set_width((avail_w * pango::SCALE as f64) as i32); // width in Pango units
                println!("[SCALE] {avail_w} {}", pango::SCALE);

                // text to draw
                let txt = text.borrow().clone();
                layout.set_text(&txt);

                // Choose a font family; we'll adjust the size
                let family = "Sans";

                // Binary search font size (in points) between min and max to fit both width and height
                let min_pt = 4.0f64;
                let max_pt = 200.0f64; //4.0f64 * 3.0; // max should be 200
                let user_value = f64::min(0.7, 1.0);
                let tolerance = 0.05f64; // stop when close enough

                let best_size = Self::binary_search_font_size(
                    min_pt,
                    max_pt,
                    avail_w,
                    avail_h,
                    tolerance,
                    Self::font_fits(layout.clone(), family.to_string()),
                );

                // Apply the chosen font and draw
                let font =
                    FontDescription::from_string(&format!("{} {}", family, best_size * user_value));
                // you can set weight/italic if you want: font.set_weight(pango::Weight::Bold);
                layout.set_font_description(Some(&font));

                // center text vertically and horizontally (approx using measured size)
                let (px_width, px_height) = layout.pixel_size();
                let x = padding + ((avail_w - px_width as f64) / 2.0).max(0.0);
                let y = padding + ((avail_h - px_height as f64) / 2.0).max(0.0);

                ctx.set_source_rgb(1.0, 1.0, 1.0); // text color
                ctx.move_to(x, y);

                pangocairo::functions::show_layout(ctx, &layout);

                // ctx.set_source_rgb(1.0, 0.0, 0.0); // text color
                // ctx.rectangle(x, y, px_width as f64, px_height as f64);
                println!(
                    "width: {px_width}\nheight: {px_height}\npw: {}\nph: {}\nsize: {:?}",
                    layout.width(),
                    layout.height(),
                    layout.size()
                );
                let _ = ctx.stroke();
            });
    }

    /// - `layout` is a Pango layout already configured with text and wrap width.
    /// - family is a font family string (e.g. "Sans").
    fn font_fits(layout: pango::Layout, family: String) -> impl Fn(f64, f64, f64) -> bool {
        move |pt: f64, avail_w: f64, avail_h: f64| -> bool {
            let font_desc = FontDescription::from_string(&format!("{family} {pt}"));
            layout.set_font_description(Some(&font_desc));
            let (w, h) = layout.pixel_size();
            (w as f64) <= avail_w + 0.5 && (h as f64) <= avail_h + 0.5
        }
    }

    /// Binary-search for the largest font size (in points) that fits within (avail_w, avail_h).
    ///   Returns the chosen font size in points (floating).
    fn binary_search_font_size(
        min_pt: f64,
        max_pt: f64,
        avail_w: f64,
        avail_h: f64,
        tol: f64,
        fits: impl Fn(f64, f64, f64) -> bool,
    ) -> f64 {
        let mut lo = min_pt;
        let mut hi = max_pt;
        let mut best = min_pt;

        if fits(hi, avail_w, avail_h) {
            return hi;
        }

        while hi - lo > tol {
            let mid = (lo + hi) / 2.0;
            if fits(mid, avail_w, avail_h) {
                best = mid;
                lo = mid;
            } else {
                hi = mid;
            }
        }

        best
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
                    set_child= &screen_drawing_area -> gtk::DrawingArea{
                        set_vexpand: true,
                        set_hexpand: true,
                        // set_visible: false
                    },

                    #[local_ref]
                    add_overlay = &screen_box -> gtk::Box {
                        // set_homogeneous: true,
                        set_css_classes: &[/* "brown_box", */  "fade-in-image", "black_bg_box" ],
                        set_overflow: gtk::Overflow::Hidden,
                        set_width_request: 200,
                        set_visible: false,

                        #[watch]
                        inline_css: &model.bg_style,

                        // #[local_ref]
                        // append = &screen_label -> gtk::Label {
                        //     #[watch]
                        //     set_text: &model.display_text.borrow(),
                        //
                        //     set_overflow: gtk::Overflow::Hidden,
                        //     set_justify: gtk::Justification::Center,
                        //     // set_align: gtk::Align::Center,
                        //     set_wrap: true,
                        //     set_vexpand: true,
                        //     set_hexpand: true,
                        //     set_wrap_mode: gtk::pango::WrapMode::Word,
                        //     set_css_classes: &["white", /*"yellow_box",   "screen_label" */]
                        // },
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
        let screen_drawing_area = gtk::DrawingArea::builder().build();
        // let screen_label = gtk::Label::builder().build();

        let model = ActivityScreenModel {
            display_text: Rc::new(RefCell::new(String::new())),
            background_image: Rc::new(RefCell::new(None)),
            bg_style: ActivityScreenModel::format_bg_style(String::new()),
            is_cleared: false,
            // screen: screen_box.clone(),
            // screen_label: screen_label.clone(),
            screen_drawing_area: screen_drawing_area.clone(),
        };
        model.register_drawing_screen();

        let widgets = view_output!();

        relm4::ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            ActivityScreenInput::DisplayUpdate(display_data) => {
                *self.display_text.borrow_mut() = display_data.text;
                self.screen_drawing_area.queue_draw();

                if let Some(img) = display_data.background_image {
                    self.update_display_image(img);
                }
            }
            ActivityScreenInput::DisplayBackground(image_src) => {
                self.update_display_image(image_src);
            }
            ActivityScreenInput::ClearDisplay => {
                self.is_cleared = !self.is_cleared;
                self.screen_drawing_area.set_visible(!self.is_cleared);
            }
        };
    }
}
