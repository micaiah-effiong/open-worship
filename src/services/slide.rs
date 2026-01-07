use gtk::glib::object::Cast;
use gtk::glib::{self, object::ObjectExt, subclass::types::ObjectSubclassIsExt};
use gtk::prelude::WidgetExt;
use gtk::{gdk, gdk_pixbuf};
use std::time::Duration;

use crate::utils::{self, WidgetChildrenExt};
use crate::widgets::canvas::canvas::Canvas;
use crate::widgets::canvas::canvas_item::{CanvasItem, CanvasItemExt};
use crate::widgets::canvas::serialise::{CanvasData, SlideData};

const VISIBLE_CHANGED: &str = "visible-changed";
const EMPTY_SLIDE: &str = "{\"background-color\":\"#000000\", \"background-pattern\":\"\" , \"items\": [{\"x\": 0,\"y\": 0,\"w\": 720,\"h\": 510, \"type\": \"color\", \"background_color\": \"#000000\", \"border-radius\": 0 }], \"notes\":\"\", \"preview\": \"iVBORw0KGgoAAAANSUhEUgAAAQsAAACWCAYAAADJ2q17AAAABmJLR0QA/wD/AP+gvaeTAAACM0lEQVR4nO3UMWpbURQAUd3Hlwpt1qndxztOVOjDcx8ITCcZzlnBVDNrrY/jOL5m5n4B+Mfe+3Ge5+fcbrc/z+fzvvd+dRPwhmbmcr1e/66ZMQrgv/bel5m5r1eHAD+DWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQCJWQDJ2ns/ZubVHcCbmpnL3vsxa61fx3H8npn7q6OA97P3fpzn+fkNRKooH6vPau0AAAAASUVORK5CYII=\"}";

mod imp {
    use std::cell::{Cell, RefCell};
    use std::sync::OnceLock;

    use gtk::glib::subclass::Signal;
    use gtk::glib::subclass::types::ObjectSubclass;
    use gtk::glib::{self, Properties};
    use gtk::subclass::prelude::*;
    use gtk::{StackTransitionType, prelude::*};

    use crate::widgets::canvas::canvas::Canvas;
    use crate::widgets::canvas::serialise::SlideData;

    #[derive(Properties)]
    #[properties(wrapper_type = super::Slide)]
    pub struct SlideImp {
        pub save_data: RefCell<Option<SlideData>>,
        pub canvas: RefCell<Option<Canvas>>,
        pub preview: RefCell<gtk::Picture>,

        #[property(set, get, default_value = "")]
        pub preview_data: RefCell<String>,
        #[property(set, get, default_value = "")]
        pub notes: RefCell<String>,
        #[property(set, get, builder(gtk::StackTransitionType::None))]
        pub trasition: RefCell<gtk::StackTransitionType>,

        #[property(get, set/* =Self::set_visible_ */, default_value=true, construct)]
        pub visible: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SlideImp {
        const NAME: &'static str = "OwSlide";
        type Type = super::Slide;
    }

    #[glib::derived_properties]
    impl ObjectImpl for SlideImp {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder(super::VISIBLE_CHANGED)
                        .param_types([bool::static_type()])
                        .build(),
                ]
            })
        }
    }

    impl Default for SlideImp {
        fn default() -> Self {
            Self {
                save_data: RefCell::new(None),
                canvas: RefCell::new(None),
                preview: RefCell::new(gtk::Picture::default()),
                preview_data: RefCell::new(String::default()),
                notes: RefCell::new(String::default()),
                trasition: RefCell::new(StackTransitionType::None),
                visible: Cell::new(true),
            }
        }
    }

    impl SlideImp {
        pub fn set_visible_(&self, value: bool) {
            self.visible.set(value);
            if let Some(c) = self.canvas.borrow().clone() {
                c.set_visible(value);
            }
            self.obj().emit_visible_changed(value);
        }
    }
}

glib::wrapper! {
    pub struct Slide(ObjectSubclass<imp::SlideImp>);
}

impl Default for Slide {
    fn default() -> Self {
        glib::Object::new::<Slide>()
    }
}

impl Slide {
    fn emit_visible_changed(&self, value: bool) {
        self.emit_by_name::<()>(VISIBLE_CHANGED, &[&value]);
    }

    pub fn connect_visible_changed<F: Fn(bool) + 'static>(&self, f: F) {
        self.connect_closure(
            VISIBLE_CHANGED,
            false,
            glib::closure_local!(move |_: &Self, value: bool| {
                f(value);
            }),
        );
    }

    pub fn new(/* window: &SpiceWindow, */ save_data: Option<SlideData>) -> Self {
        let slide = glib::Object::new::<Slide>();

        let canvas_data: Option<CanvasData> = match save_data.clone() {
            Some(d) => Some(d.into()),
            None => None,
        };

        slide.imp().save_data.replace(save_data.clone());
        let canvas = Canvas::new(/* window, */ canvas_data);
        slide.imp().canvas.replace(Some(canvas.clone()));

        canvas.connect_request_draw_preview(glib::clone!(
            #[weak]
            slide,
            move || slide.reload_preview_data()
        ));

        slide
            .bind_property("visible", &canvas, "visible")
            .bidirectional()
            .build();

        slide
    }

    pub fn empty(/* window: &SpiceWindow */) -> Self {
        let slide = Self::default();
        let data: Option<SlideData> = serde_json::from_str(EMPTY_SLIDE).ok();
        slide.imp().save_data.replace(data.clone());

        let canvas_data: Option<CanvasData> = match data {
            Some(d) => Some(d.into()),
            None => None,
        };

        slide
            .imp()
            .canvas
            .replace(Some(Canvas::new(/* window, */ canvas_data)));

        if let Some(canvas) = slide.canvas() {
            slide
                .bind_property("visible", &canvas, "visible")
                .bidirectional()
                .build();
        }

        slide.load_data();
        slide.set_visible(false);

        slide
    }

    // pub fn without_canvas(save_data: SlideData) -> Self {
    //     let slide = Self::default();
    //
    //     slide.imp().save_data.replace(Some(save_data));
    //     slide.imp().canvas.replace(None);
    //     slide.load_data();
    //
    //     slide
    // }

    pub fn load_slide(&self) {
        let Some(save_data) = self.imp().save_data.borrow().clone() else {
            return;
        };

        let canvas = match self.imp().canvas.borrow().clone() {
            Some(canvas) => {
                canvas.clear_all();
                canvas
            }
            None => return,
        };

        let _ = save_data.items.iter().for_each(|raw| {
            if let Some(item) = utils::canvas_item_from_data(raw.clone(), Some(&canvas)) {
                self.add_item(item, false, false);
            } else {
                println!("> ITEM ERROR: could not create canvas item from");
            }
        });

        self.imp().save_data.replace(None);
    }

    ///
    /// * `select_item` - default false
    pub fn add_item(&self, canvas_item: CanvasItem, select_item: bool, save_history: bool) {
        let canvas = if let Some(canvas) = self.imp().canvas.borrow().clone() {
            canvas.add_item(canvas_item.clone(), save_history);
            canvas
        } else {
            return;
        };

        if select_item {
            canvas.emit_item_clicked(Some(canvas_item));
        }
    }

    pub fn serialise(&self) -> SlideData {
        let imp = self.imp();
        if let Some(save_data) = imp.save_data.borrow().clone() {
            return save_data;
        }

        let canvas = match imp.canvas.borrow().clone() {
            Some(canvas) => canvas,
            None => return SlideData::default(),
        };

        let mut c_item_data = Vec::new();
        let iter = canvas.imp().widget.borrow().get_children::<CanvasItem>();
        for ci in iter {
            if !ci.is_visible() {
                continue;
            }

            c_item_data.push(ci.serialise());
        }

        //
        let raw_notes = glib::base64_encode(self.notes().as_bytes());
        // format!(
        //     "{{{}, \"transition\": {}, \"items\": [{}], \"notes\": \"{}\", \"preview\": \"{}\"}}\n",
        //     canvas.serialise(),
        //     i32::from(self.trasition()),
        //     data,
        //     raw_notes,
        //     self.preview_data()
        // );
        SlideData::new(
            utils::transition_to_int(self.trasition()),
            c_item_data,
            self.preview_data(),
            canvas.serialise(),
        )
    }

    pub fn delete(&self) {
        let Some(canvas) = self.canvas() else {
            return;
        };
        // let Some(window) = canvas.imp().window.upgrade() else {
        //     return;
        // };
        //
        // let action = TypedHistoryAction::slide_changed(&self.clone(), "visible");
        // window
        //     .history_manager()
        //     .add_undoable_action(action.into(), Some(true));

        self.set_visible(false);
    }

    pub fn destroy(&self) {
        if let Some(c) = self.imp().canvas.borrow().clone() {
            // TODO: dispose canvas
            c.unparent();
        }
    }

    pub fn reload_preview_data(&self) {
        // let s = self.clone();
        //
        // glib::timeout_add_local(Duration::from_millis(110), move || {
        //     let canvas = s.imp().canvas.borrow().clone();
        //     let canvas_buffer_surface = match canvas {
        //         Some(c) => c.imp().surface.borrow().clone(),
        //         None => return glib::ControlFlow::Continue,
        //     };
        //     if let Some(surface) = &canvas_buffer_surface
        //         && let Ok(pix) = surface.load_to_pixbuf()
        //     {
        //         let pixbuf = pix.scale_simple(
        //             SlideList::width(),
        //             SlideList::height(),
        //             gdk_pixbuf::InterpType::Bilinear,
        //         );
        //
        //         if let Some(pixbuf) = pixbuf {
        //             let t = gdk::Texture::for_pixbuf(&pixbuf).upcast::<gdk::Paintable>();
        //             s.imp().preview.borrow().set_paintable(Some(&t));
        //             if let Some(p) = utils::pixbuf_to_base64(&pixbuf) {
        //                 s.set_preview_data(p);
        //             }
        //         }
        //
        //         return glib::ControlFlow::Break;
        //     }
        //
        //     glib::ControlFlow::Continue
        // });
    }

    fn load_data(&self) {
        println!("???? LOAD_DATA");
        let Some(save_data) = self.imp().save_data.borrow().clone() else {
            return;
        };

        self.set_preview_data(save_data.preview);

        println!("???? PREVIEW {:?}", self.preview_data());

        if !self.preview_data().is_empty() {
            let pix_buf = utils::base64_to_pixbuf(&self.preview_data().clone());

            // TODO:
            // if let Some(pix_buf) = pix_buf
            //     && let Some(pix) = pix_buf.scale_simple(
            //         SlideList::width(),
            //         SlideList::height(),
            //         gdk_pixbuf::InterpType::Nearest,
            //     )
            // {
            //     let t = gdk::Texture::for_pixbuf(&pix).upcast::<gdk::Paintable>();
            //     self.imp().preview.borrow().set_paintable(Some(&t));
            // }
        }

        self.set_trasition(utils::int_to_transition(save_data.transition));
        // self.set_notes(save_data.notes);
    }

    pub fn canvas(&self) -> Option<Canvas> {
        self.imp().canvas.borrow().clone()
    }
}
