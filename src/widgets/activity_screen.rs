use gtk::glib::subclass::types::ObjectSubclassIsExt;
use std::cell::{Cell, RefCell};

use gtk::{glib, prelude::*};
use relm4::prelude::*;

use crate::dto;
use crate::services::slide::Slide;
use crate::utils::WidgetChildrenExt;
use crate::widgets::canvas::text_item::TextItem;

#[derive(Debug)]
pub enum ActivityScreenInput {
    DisplayUpdate(dto::DisplayPayload),
    DisplayBackground(String),
    ClearDisplay,
}

#[derive(Debug, Clone)]
pub struct ActivityScreenModel {
    is_cleared: Cell<bool>,
    slide: RefCell<Slide>,
    text: RefCell<String>,
}

const MIN_GRID_HEIGHT: i32 = 300;

impl ActivityScreenModel {}

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
                set_ratio: 16.0 / 9.0, //TODO: use device ratio
                set_obey_child:false,
                set_child = canvas.as_ref(),
            }

        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let slide = Slide::empty();
        slide.set_presentation_mode(true);
        slide.set_visible(true);
        slide.load_slide();
        let canvas = slide.canvas();

        let model = ActivityScreenModel {
            is_cleared: Cell::new(false),
            slide: RefCell::new(slide),
            text: RefCell::new(String::new()),
        };

        let widgets = view_output!();

        relm4::ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            ActivityScreenInput::DisplayUpdate(display_data) => {
                let text = display_data.text.clone();
                let slide = self.slide.borrow().clone();

                let Some(c) = slide.canvas() else {
                    return;
                };

                for c in c.widget().get_children::<TextItem>() {
                    if !self.is_cleared.get() {
                        c.imp().entry.borrow().buffer().set_text(&text);
                    }
                    self.text.replace(text);
                    break;
                }

                if let Some(img) = display_data.background_image {
                    c.set_background_pattern(img.clone());
                    c.style();
                }
            }
            ActivityScreenInput::DisplayBackground(image_src) => {
                let slide = self.slide.borrow();
                let Some(c) = slide.canvas() else {
                    return;
                };
                c.set_background_pattern(image_src);
                c.style();
            }
            ActivityScreenInput::ClearDisplay => {
                self.is_cleared.set(!self.is_cleared.get());

                let slide = self.slide.borrow().clone();

                let Some(c) = slide.canvas() else {
                    return;
                };
                for c in c.widget().get_children::<TextItem>() {
                    let buf = c.imp().entry.borrow().buffer();

                    if self.is_cleared.get() {
                        buf.set_text(" ");
                    } else {
                        buf.set_text(&self.text.borrow().clone());
                    }
                    break;
                }
                //
            }
        };
    }
}

// Activity Screen Widget
mod imp {

    use gtk::{
        glib::subclass::{
            object::{ObjectImpl, ObjectImplExt},
            types::{ObjectSubclass, ObjectSubclassExt},
        },
        subclass::{box_::BoxImpl, widget::WidgetImpl},
    };

    use super::*;

    #[derive(Debug, Default)]
    pub struct ActivityScreen {
        pub is_cleared: Cell<bool>,
        pub slide: RefCell<Slide>,
        pub text: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ActivityScreen {
        const NAME: &'static str = "ActivityScreen";
        type Type = super::ActivityScreen;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for ActivityScreen {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            let slide = Slide::empty();
            self.slide.replace(slide.clone());
            slide.set_presentation_mode(true);
            slide.set_visible(true);
            slide.load_slide();

            let aspect_frame = gtk::AspectFrame::builder()
                .css_name("pink_box")
                .ratio(16.0 / 9.0)
                .obey_child(false)
                .build();

            aspect_frame.set_child(slide.canvas().as_ref());

            let frame = gtk::Frame::builder()
                .height_request(MIN_GRID_HEIGHT)
                .hexpand(true)
                .vexpand(true)
                .child(&aspect_frame)
                .build();

            obj.add_css_class("gray_bg_box");
            obj.append(&frame);
        }
    }

    impl WidgetImpl for ActivityScreen {}
    impl BoxImpl for ActivityScreen {}
}

glib::wrapper! {
pub struct ActivityScreen (ObjectSubclass<imp::ActivityScreen>)
    @extends gtk::Widget, gtk::Box,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl Default for ActivityScreen {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl ActivityScreen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn display_update(&self, display_data: dto::DisplayPayload) {
        let imp = self.imp();
        let text = display_data.text.clone();
        let slide = imp.slide.borrow().clone();

        let Some(c) = slide.canvas() else {
            return;
        };

        for c in c.widget().get_children::<TextItem>() {
            if !imp.is_cleared.get() {
                c.imp().entry.borrow().buffer().set_text(&text);
            }
            imp.text.replace(text);
            break;
        }

        if let Some(img) = display_data.background_image {
            c.set_background_pattern(img.clone());
            c.style();
        }
    }

    pub fn display_background(&self, image_src: String) {
        let slide = self.imp().slide.borrow();
        let Some(c) = slide.canvas() else {
            return;
        };
        c.set_background_pattern(image_src);
        c.style();
    }

    pub fn clear_display(&self, clear: bool) {
        let imp = self.imp();
        if clear == imp.is_cleared.get() {
            return;
        };

        imp.is_cleared.set(clear);
        let slide = imp.slide.borrow().clone();
        let Some(c) = slide.canvas() else {
            return;
        };

        for c in c.widget().get_children::<TextItem>() {
            let buf = c.imp().entry.borrow().buffer();

            if imp.is_cleared.get() {
                buf.set_text(" ");
            } else {
                buf.set_text(&imp.text.borrow().clone());
            }
            break;
        }
    }
}
