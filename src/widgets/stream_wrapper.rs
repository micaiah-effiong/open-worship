use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, gsk};

mod imp {
    use gtk::glib::Properties;

    use super::*;
    use std::cell::{Cell, RefCell};

    #[derive(Properties)]
    #[properties(wrapper_type=super::WidgetMediaStream)]
    pub struct WidgetMediaStream {
        #[property(get, construct_only)]
        pub widget: RefCell<gtk::Widget>,

        #[property(get, construct_only)]
        pub paintable: RefCell<gtk::WidgetPaintable>,

        pub tick_id: RefCell<Option<gtk::TickCallbackId>>,

        /// Timestamp of the frame clock at the moment play() was called,
        /// used to compute a relative position.
        pub start_time: Cell<i64>,

        pub sig_contents: RefCell<Option<glib::SignalHandlerId>>,
        pub sig_size: RefCell<Option<glib::SignalHandlerId>>,
    }

    impl Default for WidgetMediaStream {
        fn default() -> Self {
            let b = gtk::Box::builder().build();
            Self {
                widget: RefCell::new(b.clone().into()),
                paintable: RefCell::new(gtk::WidgetPaintable::new(Some(&b))),
                tick_id: RefCell::new(None),
                start_time: Cell::new(0),
                sig_contents: RefCell::new(None),
                sig_size: RefCell::new(None),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WidgetMediaStream {
        const NAME: &'static str = "WidgetMediaStream";
        type Type = super::WidgetMediaStream;
        type ParentType = gtk::MediaStream;
        type Interfaces = (gdk::Paintable,);
    }

    #[glib::derived_properties]
    impl ObjectImpl for WidgetMediaStream {
        fn dispose(&self) {
            self.disconnect_paintable_signals();
        }
    }

    impl PaintableImpl for WidgetMediaStream {
        fn flags(&self) -> gdk::PaintableFlags {
            gdk::PaintableFlags::empty()
        }

        fn intrinsic_width(&self) -> i32 {
            self.paintable.borrow().intrinsic_width().max(0)
        }

        fn intrinsic_height(&self) -> i32 {
            self.paintable.borrow().intrinsic_height().max(0)
        }

        fn intrinsic_aspect_ratio(&self) -> f64 {
            self.paintable.borrow().intrinsic_aspect_ratio().max(0.0)
        }

        fn snapshot(&self, snapshot: &gdk::Snapshot, width: f64, height: f64) {
            self.paintable.borrow().snapshot(snapshot, width, height);
        }
        fn current_image(&self) -> gdk::Paintable {
            self.paintable.borrow().current_image()
        }
    }

    impl MediaStreamImpl for WidgetMediaStream {
        fn play(&self) -> bool {
            let widget = self.widget.borrow();

            self.start_time.set(glib::monotonic_time());
            let stream = self.obj().clone();

            let tick_id = widget.add_tick_callback(move |_w, frame_clock| {
                let imp = stream.imp();
                let position = frame_clock.frame_time() - imp.start_time.get();

                stream.update(position.max(0));
                stream.invalidate_contents();
                stream.invalidate_size();
                //

                if let Some(ndi_frame) = stream.extract_frame(position) {
                    //
                };

                glib::ControlFlow::Continue
            });

            self.tick_id.replace(Some(tick_id));
            true
        }

        fn pause(&self) {
            if let Some(id) = self.tick_id.take() {
                id.remove();
            }
        }

        fn seek(&self, _timestamp: i64) {
            self.obj().seek_success();
        }
    }

    impl WidgetMediaStream {
        pub(super) fn connect_paintable_signals(&self, paintable: &gtk::WidgetPaintable) {
            let stream = self.obj().clone();
            let id_contents = paintable.connect_invalidate_contents(move |_| {
                stream.invalidate_contents();
            });

            let stream = self.obj().clone();
            let id_size = paintable.connect_invalidate_size(move |_| {
                stream.invalidate_size();
            });

            self.sig_contents.replace(Some(id_contents));
            self.sig_size.replace(Some(id_size));
        }

        fn disconnect_paintable_signals(&self) {
            let paintable = self.paintable.borrow();
            if let Some(id) = self.sig_contents.take() {
                paintable.disconnect(id);
            }
            if let Some(id) = self.sig_size.take() {
                paintable.disconnect(id);
            }
        }
    }
}

glib::wrapper! {

    /// WidgetMediaStream — wrap any GtkWidget as a GdkPaintable / GtkMediaStream.
    ///
    /// The stream snapshots the widget on every timer tick, so whatever the widget
    /// draws becomes the video frame
    pub struct WidgetMediaStream(ObjectSubclass<imp::WidgetMediaStream>)
        @extends gtk::MediaStream,
        @implements gdk::Paintable;
}

impl WidgetMediaStream {
    pub fn new<W: IsA<gtk::Widget>>(widget: &W) -> Self {
        let paintable = gtk::WidgetPaintable::new(Some(widget));

        let obj: Self = glib::Object::builder()
            .property("widget", widget)
            .property("paintable", paintable.clone())
            .build();

        obj.imp().connect_paintable_signals(&paintable);

        obj.stream_prepared(
            false, // no audio
            true,  // has video
            false, // not seekable
            0,     // no duration — live
        );

        // Once the window is shown and source_box is realised, start the stream.
        // connect_map wait until the widget has a real allocation.
        widget.connect_map(glib::clone!(
            #[weak]
            obj,
            move |_| {
                if !obj.is_playing() {
                    obj.play();
                }
            }
        ));

        widget.connect_unmap(glib::clone!(
            #[weak]
            obj,
            move |_| obj.pause()
        ));

        obj
    }

    fn extract_frame(&self, pts: i64) -> Option<NdiFrame> {
        let image = self.current_image();
        let w = image.intrinsic_width();
        let h = image.intrinsic_height();
        if w == 0 || h == 0 {
            return None;
        }

        let snapshot = gtk::Snapshot::new();
        image.snapshot((&snapshot).into(), w as f64, h as f64);
        let node = snapshot.to_node()?;

        // Reuse the widget's display for the renderer so we stay on the
        // same GPU context — avoids an extra copy.
        let renderer = gsk::CairoRenderer::new();
        renderer
            .realize(None)
            .map_err(|e| println!("Error renderer realize: {:?}", e))
            .ok()?;
        let texture = renderer.render_texture(&node, None);
        renderer.unrealize();

        // w * h * RGBA
        let mut data = vec![0u8; (w * h * 4) as usize];
        texture.download(&mut data, (w * 4) as usize);

        // NDI expects BGRA — swap R and B channels in place.
        for pixel in data.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }

        Some(NdiFrame {
            data,
            width: w as u32,
            height: h as u32,
            pts,
        })
    }
}

struct NdiFrame {
    data: Vec<u8>,
    width: u32,
    height: u32,
    pts: i64, // stream position in µs
}
