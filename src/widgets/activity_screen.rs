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

pub struct ActivityScreenModel {
    display_text: String,
    background_image: Rc<RefCell<Option<String>>>,
    bg_style: String,
    is_cleared: bool,
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
}

#[relm4::component(pub)]
impl SimpleComponent for ActivityScreenModel {
    type Init = ();
    type Input = ActivityScreenInput;
    type Output = ();

    view! {
        #[root]
        gtk::Frame {
            #[wrap(Some)]
            set_child = &gtk::Box {
                set_homogeneous: true,
                set_height_request: MIN_GRID_HEIGHT,
                set_css_classes: &["brown_box",  "fade-in-image", "black_bg_box" ],
                set_overflow: gtk::Overflow::Hidden,
                #[watch]
                inline_css: &model.bg_style,

                if !&model.is_cleared {
                    gtk::Label {
                        #[watch]
                        set_label: &model.display_text,
                        set_justify: gtk::Justification::Center,
                        set_wrap: true,
                        set_wrap_mode: gtk::pango::WrapMode::Word,
                        set_css_classes: &["red_box", "white", "yellow_box"]
                    }
                }else {
                    gtk::Label {
                        set_css_classes: &["red_box", "white", "yellow_box"]
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = ActivityScreenModel {
            display_text: String::from(""),
            background_image: Rc::new(RefCell::new(None)),
            bg_style: ActivityScreenModel::format_bg_style(String::new()),
            is_cleared: false,
        };
        let widgets = view_output!();

        println!("cwd => {:?}", std::env::current_dir());

        return relm4::ComponentParts { model, widgets };
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            ActivityScreenInput::DisplayUpdate(display_data) => {
                self.display_text = display_data.text;
                if let Some(img) = display_data.background_image {
                    self.update_display_image(img);
                }
            }
            ActivityScreenInput::DisplayBackground(image_src) => {
                self.update_display_image(image_src);
            }
            ActivityScreenInput::ClearDisplay => {
                self.is_cleared = !self.is_cleared;
            }
        };
    }
}
