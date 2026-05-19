use gtk::gdk::prelude::{PaintableExt, TextureExt};
use gtk::gdk_pixbuf;
use gtk::gio;
use gtk::glib::object::Cast;
use gtk::glib::{self, object::ObjectExt, subclass::types::ObjectSubclassIsExt};
use gtk::gsk::prelude::GskRendererExt;
use gtk::prelude::{SnapshotExt, WidgetExt};

use crate::app_config::AppConfig;
use crate::utils::{self, WidgetChildrenExt};
use crate::widgets::canvas::canvas::Canvas;
use crate::widgets::canvas::canvas_item::{CanvasItem, CanvasItemExt};
use crate::widgets::canvas::serialise::{CanvasData, SlideData};
use crate::widgets::canvas::text_item::TextItem;

mod imp {
    use std::cell::{Cell, RefCell};

    use gtk::glib::subclass::types::ObjectSubclass;
    use gtk::glib::{self, Properties};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::widgets::canvas::canvas::Canvas;
    use crate::widgets::canvas::serialise::SlideData;

    #[derive(Properties)]
    #[properties(wrapper_type = super::Slide)]
    pub struct SlideImp {
        pub save_data: RefCell<Option<SlideData>>,
        pub canvas: RefCell<Option<Canvas>>,

        #[property(get)]
        pub preview: RefCell<gtk::Picture>,

        #[property(set, get)]
        pub preview_data: RefCell<glib::Bytes>,
        #[property(set, get, default_value = "")]
        pub notes: RefCell<String>,
        #[property(set, get, builder(gtk::StackTransitionType::None))]
        pub transition: RefCell<gtk::StackTransitionType>,

        #[property(get, set/* =Self::set_visible_ */, default_value=true, construct)]
        pub visible: Cell<bool>,

        #[property(get, set, construct, default_value = false)]
        pub presentation_mode: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SlideImp {
        const NAME: &'static str = "OwSlide";
        type Type = super::Slide;
    }

    #[glib::derived_properties]
    impl ObjectImpl for SlideImp {}

    impl Default for SlideImp {
        fn default() -> Self {
            Self {
                save_data: RefCell::new(None),
                canvas: RefCell::new(None),
                preview: RefCell::new(gtk::Picture::default()),
                preview_data: RefCell::new(glib::Bytes::from(&[])),
                notes: RefCell::new(String::default()),
                transition: RefCell::new(gtk::StackTransitionType::None),
                visible: Cell::new(true),
                presentation_mode: Cell::new(false),
            }
        }
    }

    impl SlideImp {}
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
    const PREVIEW_WIDTH: i32 = 200;
    pub fn preview_height() -> i32 {
        (Self::PREVIEW_WIDTH as f32 / AppConfig::aspect_ratio()) as i32
    }
    pub fn preview_width() -> i32 {
        Self::PREVIEW_WIDTH
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
            move |_| slide.reload_preview_data()
        ));

        slide
            .bind_property("visible", &canvas, "visible")
            .bidirectional()
            .sync_create()
            .build();

        slide
            .bind_property("presentation_mode", &canvas, "presentation_mode")
            .bidirectional()
            .sync_create()
            .build();

        slide.load_data();

        slide
    }

    #[doc = "visbility is false by default"]
    pub fn empty(/* window: &SpiceWindow */) -> Self {
        let slide = glib::Object::new::<Slide>();

        let data = SlideData::from_default();
        slide.imp().save_data.replace(Some(data.clone()));

        let canvas = Canvas::new(/* window, */ Some(data.into()));

        slide
            .bind_property("visible", &canvas, "visible")
            .bidirectional()
            .sync_create()
            .build();

        slide
            .bind_property("presentation_mode", &canvas, "presentation_mode")
            .bidirectional()
            .sync_create()
            .build();

        slide.imp().canvas.replace(Some(canvas));

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

        for raw in save_data.items {
            if let Some(item) = utils::canvas_item_from_data(raw.clone(), Some(&canvas)) {
                self.add_item(item, true, false);
            } else {
                println!("> ITEM ERROR: could not create canvas item from");
            }
        }

        self.set_transition(utils::int_to_transition(save_data.transition));

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

        let raw_notes = glib::base64_encode(self.notes().as_bytes());
        SlideData::new(
            utils::transition_to_int(self.transition()),
            c_item_data,
            self.preview_data().to_vec(),
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

    pub fn destroy(self) {
        if let Some(c) = self.imp().canvas.borrow().clone() {
            c.destroy();
        }
        self.imp().canvas.replace(None);
        drop(self);
    }

    pub fn reload_preview_data(&self) {
        let s = self.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(110), move || {
            let canvas = s.imp().canvas.borrow().clone();
            let Some(paintable) = canvas.and_then(|c| c.imp().surface.borrow().clone()) else {
                return glib::ControlFlow::Continue;
            };
            s.preview().set_paintable(Some(&paintable));

            let snapshot = gtk::Snapshot::new();
            let w = Self::preview_width() as f64;
            let h = Self::preview_height() as f64;
            paintable.snapshot(&snapshot, w, h);

            let Some(node) = snapshot.to_node() else {
                return glib::ControlFlow::Continue;
            };

            let renderer = gtk::gsk::CairoRenderer::new();
            if renderer.realize(None::<&gtk::gdk::Surface>).is_err() {
                return glib::ControlFlow::Continue;
            };

            let texture = renderer.render_texture(
                &node,
                Some(&gtk::graphene::Rect::new(0.0, 0.0, w as f32, h as f32)),
            );
            renderer.unrealize();

            let data = texture.save_to_png_bytes();
            s.set_preview_data(data);

            glib::ControlFlow::Break
        });
    }

    fn load_data(&self) {
        let Some(save_data) = self.imp().save_data.borrow().clone() else {
            return;
        };

        self.set_preview_data(glib::Bytes::from(&save_data.preview));

        if !self.preview_data().is_empty() {
            let pix_buf = gdk_pixbuf::Pixbuf::from_stream(
                &gio::MemoryInputStream::from_bytes(&self.preview_data()),
                gio::Cancellable::NONE,
            );

            if let Some(pix_buf) = pix_buf.ok()
                && let Some(pix) = pix_buf.scale_simple(
                    Self::preview_width(),
                    Self::preview_height(),
                    gdk_pixbuf::InterpType::Bilinear,
                )
            {
                let t = gtk::gdk::Texture::for_pixbuf(&pix).upcast::<gtk::gdk::Paintable>();
                self.imp().preview.borrow().set_paintable(Some(&t));
            }
        }

        self.set_transition(utils::int_to_transition(save_data.transition));
        // self.set_notes(save_data.notes);
    }

    pub fn canvas(&self) -> Option<Canvas> {
        self.imp().canvas.borrow().clone()
    }

    pub fn entry_buffer(&self) -> Option<gtk::TextBuffer> {
        if let Some(canvas) = self.canvas() {
            for t in canvas.widget().get_children::<TextItem>() {
                return Some(t.buffer().clone());
            }
        }

        None
    }
    pub fn text_item(&self) -> Option<TextItem> {
        if let Some(canvas) = self.canvas() {
            for t in canvas.widget().get_children::<TextItem>() {
                return Some(t);
            }
        }

        None
    }
}
