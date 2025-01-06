use std::{cell::RefCell, rc::Rc};

use gtk::{glib::property::PropertySet, prelude::*, TextBuffer};
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
    display_text: String,
    background_image: Rc<RefCell<Option<String>>>,
    bg_style: String,
    is_cleared: bool,
    text_buffer: TextBuffer,
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

        self.background_image.set(Some(image_src));
        {
            let bg = self.background_image.borrow().clone();
            if let Some(img) = bg {
                self.bg_style = ActivityScreenModel::format_bg_style(img);
            }
        }
    }

    fn register_buffer_tags(&self) {
        self.text_buffer
            .create_tag(Some("bold"), &[("weight", &700.to_value())]);
        self.text_buffer
            .create_tag(Some("color"), &[("foreground", &"white".to_value())]);
        self.text_buffer
            .create_tag(Some("size"), &[("size", &(40 * 1000).to_value())]);
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

                #[name="screen"]
                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_homogeneous: true,
                    set_css_classes: &[/* "brown_box", */  "fade-in-image", "black_bg_box" ],
                    // set_overflow: gtk::Overflow::Hidden,

                    #[watch]
                    inline_css: &model.bg_style,

                    #[name="text_view"]
                    gtk::TextView {
                        set_hexpand: true,
                        set_vexpand: true,
                        // set_can_target: false,
                        #[watch]
                        set_visible : !model.is_cleared,
                        set_css_classes: &["red_box", "white", "yellow_box"],
                        set_buffer: Some(&model.text_buffer)
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
        let model = ActivityScreenModel {
            display_text: String::from(""),
            background_image: Rc::new(RefCell::new(None)),
            bg_style: ActivityScreenModel::format_bg_style(String::new()),
            is_cleared: false,
            text_buffer: TextBuffer::new(None),
        };

        let widgets = view_output!();
        model.register_buffer_tags();

        widgets
            .text_view
            .set_justification(gtk::Justification::Center);
        widgets.text_view.set_valign(gtk::Align::Center);

        return relm4::ComponentParts { model, widgets };
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            ActivityScreenInput::DisplayUpdate(display_data) => {
                self.display_text = display_data.text.clone();
                self.text_buffer.set_text(&display_data.text);

                let (start, end) = self.text_buffer.bounds();
                self.text_buffer.apply_tag_by_name("bold", &start, &end);
                self.text_buffer.apply_tag_by_name("color", &start, &end);
                self.text_buffer.apply_tag_by_name("size", &start, &end);

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
