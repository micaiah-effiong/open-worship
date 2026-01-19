use std::sync::atomic::{self, AtomicBool};

use gtk::glib::object::{Cast, IsA, ObjectExt};
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::{
    AccessibleExt, BoxExt, EventControllerExt, GestureExt, GestureSingleExt, WidgetExt,
};

use gtk::{EventControllerKey, GestureClick, Overlay, gdk, glib};

use crate::utils::{self, WidgetChildrenExt};
use crate::widgets::canvas::canvas_grid::CanvasGrid;
use crate::widgets::canvas::canvas_item::{CanvasItem, CanvasItemExt};
use crate::widgets::canvas::serialise::CanvasData;
use crate::widgets::canvas::text_item::TextItem;

mod imp {
    use gtk::glib::Properties;
    use gtk::glib::subclass::Signal;
    use gtk::glib::subclass::types::ObjectSubclass;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{GestureClick, glib};

    use core::f64;
    use std::cell::{Cell, RefCell};
    use std::sync::OnceLock;
    use std::usize;

    use crate::utils::WidgetChildrenExt;
    use crate::widgets::canvas::canvas_grid::CanvasGrid;
    use crate::widgets::canvas::canvas_item::CanvasItem;
    use crate::widgets::canvas::serialise::CanvasData;

    pub(super) const CANVAS_CSS: &str = "
    .view {
        background: {};
    }
";

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::Canvas)]
    pub struct ImpCanvas {
        #[property(get, set=Self::set_current_ratio_, construct, default_value=1.0, type = f64)]
        pub current_ratio: Cell<f64>,
        // _current_ratio: Cell<f64>,
        // pub window: glib::WeakRef<SpiceWindow>,
        pub current_allocated_width: Cell<f64>,
        pub current_allocated_height: Cell<f64>,
        pub default_x_margin: Cell<f64>,
        pub default_y_margin: Cell<f64>,

        // pub canvas_grid: RefCell<CanvasGrid>,
        pub grid: RefCell<Option<CanvasGrid>>,
        pub sava_data: RefCell<Option<CanvasData>>,

        #[property(get, set, default_value = "#383E41", construct)]
        pub background_color: RefCell<String>,
        #[property(get, set, default_value = "", construct)]
        pub background_pattern: RefCell<String>,

        pub widget: RefCell<gtk::Overlay>,
        // pub canvas_items: RefCell<Vec<CanvasItem>>,
        pub surface: RefCell<Option<gtk::cairo::ImageSurface>>,

        #[property(get, set, construct)]
        pub presentation_mode: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ImpCanvas {
        const NAME: &'static str = "Canvas";
        type Type = super::Canvas;
        type ParentType = gtk::Box;
    }

    impl WidgetImpl for ImpCanvas {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            self.parent_snapshot(snapshot);

            // Canvas::set_drawing_preview(true);

            self._snapshot_widget();

            // Canvas::set_drawing_preview(false);
        }
    }
    impl BoxImpl for ImpCanvas {}

    pub const REQUEST_DRAW_PREVIEW: &str = "request-draw-preview";
    pub const ITEM_CLICKED: &str = "item-clicked";
    pub const RATIO_CHANGED: &str = "ratio-changed";
    pub const NEXT_SLIDE: &str = "next-slide";
    pub const PREVIOUS_SLIDE: &str = "previous-slide";
    pub const CLICKED: &str = "clicked";

    #[glib::derived_properties]
    impl ObjectImpl for ImpCanvas {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNAL: OnceLock<Vec<Signal>> = OnceLock::new();

            SIGNAL.get_or_init(|| {
                vec![
                    Signal::builder(REQUEST_DRAW_PREVIEW).build(),
                    Signal::builder(ITEM_CLICKED)
                        .param_types([Option::<CanvasItem>::static_type()])
                        .build(),
                    Signal::builder(RATIO_CHANGED)
                        .param_types([f64::static_type()])
                        .build(),
                    Signal::builder(NEXT_SLIDE).build(),
                    Signal::builder(PREVIOUS_SLIDE).build(),
                    Signal::builder(CLICKED)
                        .param_types([GestureClick::static_type()])
                        .build(),
                ]
            })
        }
    }

    impl ImpCanvas {
        fn set_current_ratio_(&self, value: f64) {
            if value <= 0.0 {
                return;
            }

            if value != self.current_ratio.get() {
                self.current_ratio.set(value);
                self.obj().emit_ratio_changed(value);
            }
        }

        pub(super) fn calculate_ratio(&self) {
            let max_width = 1500.0;
            let max_height = 1500.0;
            // let max_height = 843.75;

            let widget = self.widget.borrow();
            // NOTE: may have to address this in the constructor
            // added .max(1) to ensure the size defaults to 1 not 0
            self.current_allocated_width
                .set(widget.width().max(1).into());
            self.current_allocated_height
                .set(widget.height().max(1).into());

            let current_allocated_height = self.current_allocated_height.get();
            let current_allocated_width = self.current_allocated_width.get();
            let ratio = current_allocated_height / max_height;
            let current_ratio = ratio - ratio * 0.016;
            self.obj().set_current_ratio(current_ratio); // 24/1500 = 0.016; Legacy offset;

            self.default_x_margin.set(
                ((current_allocated_width - max_width * self.current_ratio.get()) / 2.0) + 0.5,
            );

            self.default_y_margin.set(
                ((current_allocated_height - max_height * self.current_ratio.get()) / 2.0) + 0.5,
            );
        }

        pub(super) fn load_data(&self) {
            let save_data = match self.sava_data.borrow().clone() {
                Some(save_data) => save_data,
                None => return,
            };

            self.background_color
                .replace(save_data.background_color.clone());
            self.background_pattern
                .replace(save_data.background_pattern.clone());
        }

        pub(super) fn reorder_overlay<W: IsA<gtk::Widget>>(&self, child: &W, index: usize) {
            let overlay = &self.widget.borrow().clone();

            let mut children = overlay.get_children::<CanvasItem>().collect::<Vec<_>>();
            for child in &children {
                overlay.remove_overlay(child);
            }

            if let Some(from_index) = children.iter().position(|x| x == child) {
                let element = children.remove(from_index);
                children.insert(index.min(children.len()), element);
            }

            children.iter().for_each(|i| overlay.add_overlay(i));
        }

        fn _snapshot_widget(&self) {
            if !self.obj().is_realized() {
                return;
            }

            super::Canvas::set_drawing_preview(true);
            let s = gtk::Snapshot::new();
            self.obj().snapshot_child(&self.widget.borrow().clone(), &s);
            super::Canvas::set_drawing_preview(false);

            let Some((_, _, w, h)) = self.obj().bounds() else {
                glib::g_log!(
                    "Canvas Snapshot",
                    glib::LogLevel::Warning,
                    "Could not get bounds",
                );
                return;
            };

            let Some(node) = s.clone().to_node() else {
                glib::g_log!(
                    "Canvas Snapshot",
                    glib::LogLevel::Warning,
                    "Could not get node",
                );
                return;
            };

            // let Ok(buffer_surface) = BufferSurface::new(w, h) else {
            //     glib::g_log!(
            //         "Canvas Snapshot",
            //         glib::LogLevel::Warning,
            //         "Could not create buffer-surface"
            //     );
            //     return;
            // };
            //
            // if let Some(surface) = buffer_surface.surface() {
            //     let Ok(ctx) = gtk::cairo::Context::new(&surface) else {
            //         return;
            //     };
            //     node.draw(&ctx);
            // }

            // self.surface.replace(Some(buffer_surface));
            self.surface.replace(None);
        }
    }
}

glib::wrapper! {
    pub struct Canvas(ObjectSubclass<imp::ImpCanvas>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for Canvas {
    fn default() -> Self {
        glib::Object::new::<Self>()
    }
}

static DRAWING_PREVIEW: AtomicBool = AtomicBool::new(false);
impl Canvas {
    pub fn set_drawing_preview(value: bool) {
        DRAWING_PREVIEW.store(value, atomic::Ordering::SeqCst);
    }
    pub fn drawing_preview() -> bool {
        DRAWING_PREVIEW.load(atomic::Ordering::SeqCst)
    }

    pub fn new(/* window: &SpiceWindow, */ save_data: Option<CanvasData>) -> Self {
        let obj = glib::Object::new::<Canvas>();
        // obj.imp().window.set(Some(window));
        obj.imp().sava_data.replace(save_data);

        obj.connect_presentation_mode_notify(|c| {
            // println!("-NOTIFY C_PRESENTATION_MODE {}", c.presentation_mode());
            if c.presentation_mode() {
                c.unselect_all(None);
            }
        });

        let canvas_grid = CanvasGrid::new(obj.clone());

        let overlay = gtk::Overlay::new();
        obj.append(&overlay);
        // WARN: this breaks the aspect-ratio
        // should be implementated to scale by current ratio
        // overlay.set_size_request(500, 380);
        overlay.set_child(Some(&canvas_grid));
        overlay.add_overlay(&gtk::Label::builder().sensitive(false).build());
        overlay.set_child_visible(true);
        overlay.add_css_class("canvas");

        overlay.connect_get_child_position({
            let cc = obj.clone().downgrade();
            move |ov, widget| {
                let Some(cc) = cc.upgrade() else { return None };
                return cc.connect_get_child_position(ov, widget);
            }
        });

        let click = GestureClick::builder().name("CanvasClick").build();
        click.set_propagation_phase(gtk::PropagationPhase::Bubble);
        click.connect_pressed(glib::clone!(
            #[weak]
            obj,
            move |evt, _, _, _| {
                obj.emit_clicked(evt);
                evt.set_state(gtk::EventSequenceState::Claimed);
            }
        ));

        obj.add_controller(click.clone()); // canvas now recieves click event
        obj.connect_clicked(glib::clone!(
            #[weak]
            obj,
            move |evt| obj.button_press_event(evt)
        ));

        let esc = EventControllerKey::new();
        esc.set_propagation_phase(gtk::PropagationPhase::Bubble);
        esc.connect_key_pressed({
            let obj = obj.clone();
            move |_, k, c, _| {
                // NOTE: according to macos
                // 53 is the keycode for ESC
                // not yet tested on other devices
                if c == 53 {
                    obj.unselect_all(None);
                }

                glib::Propagation::Proceed
                //
            }
        });
        obj.add_controller(esc);

        obj.imp().grid.replace(Some(canvas_grid));
        obj.imp().widget.replace(overlay);

        obj.imp().calculate_ratio();
        obj.imp().load_data();
        obj.style();

        obj
    }

    pub fn style(&self) {
        if let Some(canvas_grid) = self.imp().grid.borrow().clone() {
            utils::set_style(
                &canvas_grid.clone(),
                &imp::CANVAS_CSS.replace("{}", &self.imp().background_color.borrow().clone()),
            );

            canvas_grid.style(self.imp().background_pattern.borrow().clone());
        }

        self.emit_request_draw_preview();
    }

    pub fn move_up(&self, item_: &CanvasItem, add_undo_action: Option<bool>) {
        let mut index = 0;

        for child in self.widget().get_children::<CanvasItem>() {
            if child == *item_ {
                break;
            }
            index += 1
        }

        // if add_undo_action.unwrap_or(true) {
        //     if let Some(window) = self.imp().window.upgrade() {
        //         let action = TypedHistoryAction::depth_changed(item_, self, true);
        //         window
        //             .history_manager()
        //             .add_undoable_action(action.into(), Some(true));
        //     };
        // }

        self.imp().reorder_overlay(item_, index + 1);
    }

    pub fn move_down(&self, item_: &CanvasItem, add_undo_action: Option<bool>) {
        let overlay = self.widget();
        // let mut index: i32 = 0;

        let index = overlay
            .get_children::<CanvasItem>()
            .position(|v| v == *item_);

        if let Some(index) = index {
            self.imp().reorder_overlay(item_, index.saturating_sub(1));
        }

        // if add_undo_action.unwrap_or(true) {
        //     if let Some(window) = self.imp().window.upgrade() {
        //         let action = TypedHistoryAction::depth_changed(item_, self, false);
        //         window
        //             .history_manager()
        //             .add_undoable_action(action.into(), Some(true));
        //     };
        // }
    }

    pub fn clear_all(&self) {
        let widget = self.widget();

        for child in widget.get_children::<CanvasItem>().rev() {
            child.unselect();
            widget.remove_overlay(&child);
            child.unparent();
        }
    }

    pub fn add_item(&self, canvas_item: CanvasItem, undoable_action: bool) -> CanvasItem {
        self.widget().add_overlay(&canvas_item);

        canvas_item.connect_checkposition({
            let ci = canvas_item.clone();
            move |_| ci.queue_resize() //ci.queue_allocate()
        });

        canvas_item.connect_clicked(glib::clone!(
            #[weak(rename_to=c)]
            self,
            move |ci| {
                c.unselect_except(ci, Some(false));
                c.emit_item_clicked(Some(ci.clone()));
            }
        ));

        canvas_item.connect_move_item(glib::clone!(
            #[weak(rename_to=c)]
            self,
            move |ci, delta_x, delta_y| {
                if c.presentation_mode() {
                    return;
                }

                let r: gdk::Rectangle = ci.rectangle().into();

                let x = (delta_x as f64 / c.current_ratio()) as i32 + r.x();
                let y = (delta_y as f64 / c.current_ratio()) as i32 + r.y();

                let rect = gdk::Rectangle::new(x, y, r.width(), r.height());
                ci.set_rectangle(utils::rect::Rect::from(rect));

                // ci.queue_allocate();
                ci.style(); // NOTE: allows font rescaling after item resize
                ci.queue_resize();
                c.emit_request_draw_preview();
            }
        ));

        // if undoable_action {
        //     canvas_item.set_visible(false);
        //     let action = TypedHistoryAction::item_changed(&canvas_item, "visible");
        //     if let Some(window) = self.imp().window.upgrade() {
        //         window
        //             .history_manager()
        //             .add_undoable_action(action.into(), None);
        //     };
        //     canvas_item.set_visible(true);
        // }

        self.emit_request_draw_preview();

        // println!("C_PRESENTATION_MODE {}", self.presentation_mode());
        if self.presentation_mode() {
            self.unselect_all(None);
        }

        //
        canvas_item
    }

    pub fn serialise(&self) -> CanvasData {
        CanvasData {
            background_color: self.imp().background_color.borrow().clone(),
            background_pattern: self.imp().background_pattern.borrow().clone(),
        }
        // format!(
        //     "\"background-color\":\"{}\", \"background-pattern\":\"{}\"",
        //     self.imp().background_color.borrow(),
        //     self.imp().background_pattern.borrow()
        // )
    }

    pub fn unselect_all(&self, reset_item: Option<bool>) {
        let reset_item = reset_item.unwrap_or(true);

        for child in self.widget().get_children::<CanvasItem>() {
            child.unselect();
        }

        if reset_item {
            self.emit_item_clicked(None);
        }

        self.emit_request_draw_preview();
    }
    pub fn unselect_except(&self, item: &impl IsA<CanvasItem>, reset_item: Option<bool>) {
        let reset_item = reset_item.unwrap_or(true);

        for child in self.widget().get_children::<CanvasItem>() {
            if child == *item {
                continue;
            }
            child.unselect();
        }

        if reset_item {
            self.emit_item_clicked(None);
        }

        self.emit_request_draw_preview();
    }

    fn button_press_event(&self, event: &GestureClick) {
        if self.presentation_mode() {
            return;
        }
        self.unselect_all(None);
    }

    pub fn emit_clicked(&self, gesture: &GestureClick) {
        self.emit_by_name::<()>(imp::CLICKED, &[gesture]);
    }

    fn emit_ratio_changed(&self, ratio: f64) {
        self.emit_by_name::<()>(imp::RATIO_CHANGED, &[&ratio]);
    }

    pub fn emit_item_clicked(&self, item: Option<CanvasItem>) {
        self.emit_by_name::<()>(imp::ITEM_CLICKED, &[&item]);
    }

    pub fn emit_request_draw_preview(&self) {
        self.emit_by_name::<()>(imp::REQUEST_DRAW_PREVIEW, &[]);
    }
    pub fn emit_next_slide(&self) {
        self.emit_by_name::<()>(imp::NEXT_SLIDE, &[]);
    }
    pub fn emit_previous_slide(&self) {
        self.emit_by_name::<()>(imp::PREVIOUS_SLIDE, &[]);
    }

    pub fn connect_clicked<F: Fn(&GestureClick) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            imp::CLICKED,
            false,
            glib::closure_local!(move |_: &Self, g: &GestureClick| f(g)),
        )
    }
    pub fn connect_request_draw_preview<F: Fn() + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            imp::REQUEST_DRAW_PREVIEW,
            false,
            glib::closure_local!(move |_: &Self| {
                f();
            }),
        )
    }
    pub fn connect_next_slide<F: Fn() + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            imp::NEXT_SLIDE,
            false,
            glib::closure_local!(move |_: &Self| {
                f();
            }),
        )
    }
    pub fn connect_previous_slide<F: Fn() + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            imp::PREVIOUS_SLIDE,
            false,
            glib::closure_local!(move |_: &Self| {
                f();
            }),
        )
    }
    pub fn connect_item_clicked<F: Fn(Option<CanvasItem>) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            imp::ITEM_CLICKED,
            false,
            glib::closure_local!(move |_: &Self, ci: Option<CanvasItem>| {
                f(ci);
            }),
        )
    }
    pub fn connect_ratio_changed<F: Fn(f64) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            imp::RATIO_CHANGED,
            false,
            glib::closure_local!(move |_: &Self, ratio: f64| {
                f(ratio);
            }),
        )
    }

    fn connect_get_child_position(
        &self,
        ov: &Overlay,
        widget: &gtk::Widget,
    ) -> Option<gtk::gdk::Rectangle> {
        let c = self.clone();
        if c.imp().current_allocated_width.get() != ov.width() as f64
            && c.imp().current_allocated_height.get() != ov.height() as f64
        {
            c.imp().calculate_ratio();
        }

        let Ok(ci_widget) = widget.clone().downcast::<CanvasItem>() else {
            return None;
        };

        // println!("POSITION>> {:?} {:?}", ov.bounds(), r);

        let r: gdk::Rectangle = ci_widget.rectangle().into();
        let ratio = c.current_ratio();
        let margin_x = c.imp().default_x_margin.get();
        let margin_y = c.imp().default_y_margin.get();

        let width = (r.width() as f64 * ratio + 0.5) as i32;
        let height = (r.height() as f64 * ratio + 0.5) as i32;
        let x = (margin_x + (r.x() as f64 * ratio + 0.5) + ci_widget.delta_x() as f64) as i32;
        let y = (margin_y + (r.y() as f64 * ratio + 0.5) + ci_widget.delta_y() as f64) as i32;

        let allocation = gtk::gdk::Rectangle::new(x, y, width, height);
        return Some(allocation);
        // return None;
    }

    pub fn widget(&self) -> gtk::Overlay {
        self.imp().widget.borrow().clone()
    }
}
