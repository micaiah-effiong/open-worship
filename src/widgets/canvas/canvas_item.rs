mod imp {
    use std::cell::{Cell, RefCell};
    use std::sync::OnceLock;
    use std::{i32, u32};

    use glib::subclass::object::ObjectImpl;
    use glib::subclass::types::ObjectSubclass;
    use gtk::glib::subclass::Signal;
    use gtk::glib::{self, Properties};
    use gtk::subclass::prelude::*;
    use gtk::{TextView, prelude::*};
    use serde_json::Value as JsonValue;

    use crate::{utils, widgets};
    // use crate::services::history_manager::history_action::{HistoryAction, TypedHistoryAction};
    // use crate::services::utils::{self, rect};
    use crate::widgets::canvas::canvas::Canvas;
    use crate::widgets::canvas::grabber::{self, Grabber};
    use crate::widgets::canvas::serialise::CanvasItemData;

    use super::*;

    pub const MIN_SIZE: i32 = 40;
    pub const CSS: &str = ".colored.selected { border: 2px dotted white; }";

    #[derive(Properties, Debug, Default)]
    #[properties(wrapper_type = super::CanvasItem)]
    pub struct CanvasItem {
        //
        #[property(get, set, default_value = 0)]
        pub delta_x: Cell<i32>,
        #[property(get, set, default_value = 0)]
        pub delta_y: Cell<i32>,

        // pub(super) undo_move_action: RefCell<HistoryAction>,
        #[property(get=Self::item_visible_, set=Self::set_item_visible_)]
        pub item_visible: Cell<bool>,

        #[property(get=Self::rectangle_, set=Self::set_rectangle_)]
        pub rectangle: RefCell<utils::rect::Rect>,

        // NOTE: this is saved as string because non-unit values are not allowed
        // as glib::Enum
        #[property(get, construct_only)]
        pub save_data: RefCell<Option<String>>,
        pub start_x: Cell<f64>,
        pub start_y: Cell<f64>,
        pub start_w: Cell<i32>,
        pub start_h: Cell<i32>,

        /// more like grabber id
        pub holding: Cell<bool>,
        pub holding_id: Cell<u32>,

        pub real_width: Cell<i32>,
        pub real_height: Cell<i32>,
        pub real_x: Cell<i32>,
        pub real_y: Cell<i32>,

        pub grid: RefCell<gtk::Grid>,
        pub grabber_revealer: RefCell<gtk::Revealer>,

        #[property(get, construct_only, nullable)]
        pub canvas: glib::WeakRef<Canvas>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CanvasItem {
        const NAME: &'static str = "CanvasItem";
        type Type = super::CanvasItem;
        type ParentType = gtk::Box;
        type Class = super::Class;
        const ABSTRACT: bool = true;

        /// Initialize the class struct with the default implementations of the
        /// virtual methods.
        fn class_init(klass: &mut Self::Class) {
            klass.load_item_data = |obj| obj.imp().load_item_data_default();
            klass.serialise_item = |obj| obj.imp().serialise_item_default();
            klass.style = |obj| obj.imp().style_default();
        }
    }

    impl WidgetImpl for CanvasItem {}

    #[glib::derived_properties]
    impl ObjectImpl for CanvasItem {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj().clone();

            self.real_width.set(720);
            self.real_height.set(510);

            obj.add_css_class("colored");
            obj.add_css_class("ow-canvas-item");

            utils::set_style(&obj.clone(), CSS);

            let grabber_revealer = gtk::Revealer::new();
            grabber_revealer.set_transition_duration(0);

            let grabber_grid = gtk::Grid::new();
            self.grid.replace(grabber_grid.clone());
            grabber_grid.add_css_class("ow-canvas-item-grid");
            grabber_grid.set_row_homogeneous(true);
            grabber_grid.set_column_homogeneous(true);

            self.grabber_revealer.replace(grabber_revealer);

            let overlay = gtk::Overlay::new();
            obj.append(&overlay);
            overlay.set_hexpand(true);
            overlay.set_vexpand(true);
            overlay.set_child(Some(&self.grid.borrow().clone()));

            /*
             * Grabber Pos:
             * 1 2 3
             * 8   4
             * 7 6 5
             */
            let grabber_1 = self.make_grabber(1, gtk::Align::Start, gtk::Align::Start, &overlay);
            let grabber_2 = self.make_grabber(2, gtk::Align::Center, gtk::Align::Start, &overlay);
            let grabber_3 = self.make_grabber(3, gtk::Align::End, gtk::Align::Start, &overlay);
            let grabber_4 = self.make_grabber(4, gtk::Align::End, gtk::Align::Center, &overlay);
            let grabber_5 = self.make_grabber(5, gtk::Align::End, gtk::Align::End, &overlay);
            let grabber_6 = self.make_grabber(6, gtk::Align::Center, gtk::Align::End, &overlay);
            let grabber_7 = self.make_grabber(7, gtk::Align::Start, gtk::Align::End, &overlay);
            let grabber_8 = self.make_grabber(8, gtk::Align::Start, gtk::Align::Center, &overlay);

            let grabber_list = [
                grabber_1.clone(),
                grabber_2.clone(),
                grabber_3.clone(),
                grabber_4.clone(),
                grabber_5.clone(),
                grabber_6.clone(),
                grabber_7.clone(),
                grabber_8.clone(),
            ];

            obj.connect_clicked({
                let grabber_list = grabber_list.clone();
                let grabber_grid = grabber_grid.clone();
                move |obj| {
                    println!("obj.connect_clicked");
                    if obj.imp().is_presentation_mode() {
                        return;
                    }

                    for g in &grabber_list {
                        g.set_visible(true);
                    }
                    grabber_grid.add_css_class("ow-canvas-item-grid-select");
                }
            });

            obj.connect_unselect({
                let grabber_list = grabber_list.clone();
                let grabber_grid = grabber_grid.clone();
                move |obj| {
                    println!("obj.connect_unselect");
                    // if obj.imp().is_presentation_mode() {
                    //     return;
                    // }

                    for g in &grabber_list {
                        g.set_visible(false);
                    }
                    grabber_grid.remove_css_class("ow-canvas-item-grid-select");
                }
            });

            {
                let clicked = gtk::GestureClick::builder().name("CanvasItemClick").build();
                clicked.set_button(gtk::gdk::BUTTON_PRIMARY);
                clicked.set_propagation_phase(gtk::PropagationPhase::Bubble);

                clicked.connect_pressed(glib::clone!(
                    #[weak(rename_to=ci)]
                    self,
                    move |g, _, _, _| {
                        if ci.is_presentation_mode() {
                            return;
                        }

                        ci.button_press_event(g);
                        g.set_state(gtk::EventSequenceState::Claimed);
                    }
                ));
                clicked.connect_released(glib::clone!(
                    #[weak(rename_to=ci)]
                    self,
                    move |g, _, _, _| {
                        if ci.is_presentation_mode() {
                            return;
                        }

                        ci.button_release_event(g);
                        g.set_state(gtk::EventSequenceState::Claimed);
                    }
                ));

                clicked.connect_cancel(glib::clone!(
                    #[weak(rename_to=ci)]
                    self,
                    move |g, _| {
                        if ci.is_presentation_mode() {
                            return;
                        }

                        ci.button_release_event(g);
                        g.set_state(gtk::EventSequenceState::Claimed);
                    }
                ));

                let right_click = gtk::GestureClick::new();
                right_click.set_button(gtk::gdk::BUTTON_SECONDARY);
                right_click.set_propagation_phase(gtk::PropagationPhase::Bubble);
                right_click.connect_pressed(glib::clone!(
                    #[weak(rename_to=ci)]
                    self,
                    #[strong]
                    grabber_list,
                    move |g, _, _, _| {
                        if ci.is_presentation_mode() {
                            return;
                        }

                        for g in &grabber_list {
                            g.switch_mode();
                        }
                        g.set_state(gtk::EventSequenceState::Claimed);
                    }
                ));

                let motion = gtk::EventControllerMotion::new();
                motion.set_propagation_phase(gtk::PropagationPhase::Bubble);
                motion.connect_motion(glib::clone!(
                    #[weak(rename_to=ci)]
                    self,
                    move |g, _, _| {
                        if ci.is_presentation_mode() {
                            return;
                        }

                        ci.motion_notify_event(g);
                    }
                ));

                obj.add_controller(motion);
                obj.add_controller(clicked);
                obj.add_controller(right_click);
            }

            if !self.is_presentation_mode() {
                // NOTE: this registers that the new item has been selected
                obj.emit_clicked();
            }
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();

            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder(signals::CLICKED).build(),
                    Signal::builder(signals::UN_SELECT).build(),
                    Signal::builder(signals::SET_AS_PRIMARY).build(),
                    Signal::builder(signals::MOVE_ITEM)
                        .param_types([i32::static_type(), i32::static_type()])
                        .build(),
                    Signal::builder(signals::CHECK_POSITION).build(),
                    Signal::builder(signals::ACTIVE_CHANGED).build(),
                ]
            })
        }
    }

    impl BoxImpl for CanvasItem {}

    impl CanvasItem {
        fn emit_checkposition(&self) {
            self.obj().emit_by_name::<()>(signals::CHECK_POSITION, &[]);
        }

        fn emit_unselect(&self) {
            self.obj().emit_by_name::<()>(signals::UN_SELECT, &[]);
        }

        fn emit_move_item(&self, x: i32, y: i32) {
            self.obj().emit_by_name::<()>(signals::MOVE_ITEM, &[&x, &y]);
        }

        pub(super) fn get_save_data(&self) -> Option<CanvasItemData> {
            let Some(data) = self.save_data.borrow().clone() else {
                return None;
            };
            serde_json::from_str(&data).ok()
        }

        fn rectangle_(&self) -> utils::rect::Rect {
            utils::rect::Rect::new(
                self.real_x.get(),
                self.real_y.get(),
                self.real_width.get(),
                self.real_height.get(),
            )
        }

        fn set_rectangle_(&self, rect: utils::rect::Rect) {
            self.real_x.set(rect.x);
            self.real_y.set(rect.y);
            self.real_width.set(rect.width);
            self.real_height.set(rect.height);

            self.emit_checkposition();
        }

        fn fix_size(size: f64) -> i32 {
            size.max(MIN_SIZE as f64) as i32
        }

        fn fix_position(&self, delta: i32, _length: i32, initial_length: i32) -> i32 {
            let Some(canvas) = self.canvas.upgrade() else {
                return delta;
            };
            let scaled_delta = (initial_length) as f64 * canvas.current_ratio();
            delta.min(scaled_delta as i32)
        }

        pub(super) fn load_data(&self) {
            let Some(save_data) = self.get_save_data() else {
                return;
            };

            self.real_width.set(save_data.w);
            self.real_height.set(save_data.h);
            self.real_x.set(save_data.x);
            self.real_y.set(save_data.y);

            self.obj().load_item_data();
            self.emit_checkposition();
        }

        fn motion_notify_event(&self, event: &gtk::EventControllerMotion) {
            if !self.holding.get() {
                return;
            }

            let Some((x, y)) = event.current_event().and_then(|v| v.position()) else {
                return;
            };

            let x: f64 = x.into();
            let y: f64 = y.into();

            let x = (x - self.start_x.get()) as i32;
            let y = (y - self.start_y.get()) as i32;

            let Some(canvas) = self.canvas.upgrade() else {
                return;
            };

            let canvas_current_ratio = canvas.current_ratio();

            let real_width = self.real_width.get();
            let real_height = self.real_height.get();
            let start_w = self.start_w.get();
            let start_h = self.start_h.get();

            match self.holding_id.get() {
                0 => {
                    // Moving
                    self.delta_x.set(x);
                    self.delta_y.set(y);
                }
                1 => {
                    // Top left
                    self.delta_x.set(self.fix_position(x, real_width, start_w));
                    self.delta_y.set(self.fix_position(y, real_height, start_h));
                    self.real_height.set(Self::fix_size(
                        start_h as f64 - 1.0 / canvas_current_ratio * y as f64,
                    ));
                    self.real_width.set(Self::fix_size(
                        start_w as f64 - 1.0 / canvas_current_ratio * x as f64,
                    ));
                }
                2 => {
                    // Top
                    self.delta_y.set(self.fix_position(y, real_height, start_h));
                    self.real_height.set(Self::fix_size(
                        start_h as f64 - 1.0 / canvas_current_ratio * y as f64,
                    ));
                }
                3 => {
                    // Top right
                    self.delta_y.set(self.fix_position(y, real_height, start_h));
                    self.real_height.set(Self::fix_size(
                        start_h as f64 - 1.0 / canvas_current_ratio * y as f64,
                    ));
                    self.real_width.set(Self::fix_size(
                        start_w as f64 + 1.0 / canvas_current_ratio * x as f64,
                    ));
                }
                4 => {
                    // Right
                    self.real_width.set(Self::fix_size(
                        start_w as f64 + 1.0 / canvas_current_ratio * x as f64,
                    ));
                }
                5 => {
                    // Bottom Right
                    self.real_width.set(Self::fix_size(
                        start_w as f64 + 1.0 / canvas_current_ratio * x as f64,
                    ));
                    self.real_height.set(Self::fix_size(
                        start_h as f64 + 1.0 / canvas_current_ratio * y as f64,
                    ));
                }
                6 => {
                    // Bottom
                    self.real_height.set(Self::fix_size(
                        start_h as f64 + 1.0 / canvas_current_ratio * y as f64,
                    ));
                }
                7 => {
                    // Bottom left
                    self.real_height.set(Self::fix_size(
                        start_h as f64 + 1.0 / canvas_current_ratio * y as f64,
                    ));
                    self.real_width.set(Self::fix_size(
                        start_w as f64 - 1.0 / canvas_current_ratio * x as f64,
                    ));
                    self.delta_x.set(self.fix_position(x, real_width, start_w));
                }
                8 => {
                    // Left
                    self.real_width.set(Self::fix_size(
                        start_w as f64 - 1.0 / canvas_current_ratio * x as f64,
                    ));
                    self.delta_x.set(self.fix_position(x, real_width, start_w));
                }
                9_u32..=u32::MAX => panic!("Invalid holding id"),
            }

            self.emit_checkposition();

            return;
        }

        fn button_press_event(&self, _event: &gtk::GestureClick) -> bool {
            let Some(canvas) = self.canvas.upgrade() else {
                return false;
            };

            // if let Some(window) = canvas.imp().window.upgrade()
            //     && window.is_presenting()
            // {
            //     return false;
            // };

            let Some((x, y)) = _event.current_event().and_then(|v| v.position()) else {
                return false;
            };

            let x: f64 = x.into();
            let y: f64 = y.into();

            if self.holding.get() {
                return true;
            }

            // let undo_move_action = TypedHistoryAction::item_moved(&self.obj().clone());
            // self.undo_move_action.replace(undo_move_action.into());

            self.start_x.set(x);
            self.start_y.set(y);

            self.start_w.set(self.real_width.get());
            self.start_h.set(self.real_height.get());

            self.holding.set(true);

            self.obj().emit_clicked();
            self.set_cursor(self.holding_id.get());

            true
        }

        fn button_release_event(&self, _event: &gtk::GestureClick) -> bool {
            println!("button_released");
            if !self.holding.get() {
                return false;
            }

            // utils::set_cursor("default");

            self.holding.set(false);
            self.holding_id.set(0);

            if self.delta_x.get() == 0
                && self.delta_y.get() == 0
                && (self.start_w.get() == self.real_width.get())
                && (self.start_h.get() == self.real_height.get())
            {
                return false;
            }

            // if let Some(canvas) = self.canvas.upgrade()
            //     && let Some(window) = canvas.imp().window.upgrade()
            // {
            //     window
            //         .history_manager()
            //         .add_undoable_action(self.undo_move_action.borrow().clone(), None);
            // }

            self.emit_move_item(self.delta_x.get(), self.delta_y.get());
            self.delta_x.set(0);
            self.delta_y.set(0);

            false
        }

        fn make_grabber(
            &self,
            id: u32,
            halign: gtk::Align,
            valign: gtk::Align,
            overlay: &gtk::Overlay,
        ) -> Grabber {
            let g = Grabber::new(id);
            g.set_halign(halign);
            g.set_valign(valign);
            g.set_visible(false);

            self.connect_grabber(&g);
            overlay.add_overlay(&g);

            g
        }
        fn connect_grabber(&self, grabber: &Grabber) {
            // if let Some(c) = self.canvas.upgrade()
            //     && c.presentation_mode()
            // {
            //     return;
            // }

            grabber.connect_grabbed(glib::clone!(
                #[weak(rename_to=ci)]
                self,
                move |event, id, _, _| {
                    if ci.is_presentation_mode() {
                        return;
                    }
                    ci.button_press_event(event);
                    ci.set_holding(id as u32);
                    event.set_state(gtk::EventSequenceState::Claimed);
                }
            ));

            grabber.connect_grabbed_stopped(glib::clone!(
                #[weak(rename_to=ci)]
                self,
                move |event, _, _, _| {
                    if ci.is_presentation_mode() {
                        return;
                    }
                    ci.button_release_event(event);
                    event.set_state(gtk::EventSequenceState::Claimed);
                }
            ));

            grabber.connect_grabbed_motion(glib::clone!(
                #[weak(rename_to=ci)]
                self,
                move |event, _, _, _| {
                    if ci.is_presentation_mode() {
                        return;
                    }
                    ci.motion_notify_event(event);
                }
            ));
        }

        fn set_holding(&self, id: u32) {
            self.holding.set(true);
            self.holding_id.set(id);
            self.set_cursor(id);
        }

        fn get_grabber(w: &gtk::Widget) -> Option<Grabber> {
            println!("get_grabber {:?}", w);
            if let Some(g) = w.downcast_ref::<Grabber>() {
                return Some(g.clone());
            }

            w.downcast_ref::<gtk::Image>()?
                .parent()
                .and_downcast::<Grabber>()
        }

        #[doc(alias = "set_grabbing_cursor")]
        fn set_cursor(&self, holding_id: u32) {
            let cursor_name = match holding_id {
                0 => "grabbing",
                1 => "nw-resize",
                2 => "n-resize",
                3 => "ne-resize",
                4 => "e-resize",
                5 => "se-resize",
                6 => "s-resize",
                7 => "sw-resize",
                8 => "w-resize",
                9_u32..=u32::MAX => panic!("Invalid holding id"),
            };

            glib::g_message!(
                "CanvasItem",
                "Set cursor in butten_event_press {holding_id} {cursor_name}"
            );

            // utils::set_cursor(cursor_name);
        }

        pub fn unselect(&self) {
            if !self.holding.get() {
                self.emit_unselect();
            }
        }

        pub fn delete(&self) {
            let Some(canvas) = self.canvas.upgrade() else {
                return;
            };
            // let Some(window) = canvas.imp().window.upgrade() else {
            //     return;
            // };
            // let obj = self.obj().clone();
            // let action = TypedHistoryAction::item_changed(&obj, "item-visible");
            // window
            //     .history_manager()
            //     .add_undoable_action(action.into(), Some(true));

            self.obj().set_item_visible(false);
        }

        fn item_visible_(&self) -> bool {
            self.obj().is_visible()
        }
        fn set_item_visible_(&self, value: bool) {
            self.obj().set_visible(value);
        }

        pub(super) fn serialise(&self) -> CanvasItemData {
            let x = self.real_x.get();
            let y = self.real_y.get();
            let w = self.real_width.get();
            let h = self.real_height.get();
            let item_type = self.obj().serialise_item();
            let data = CanvasItemData::new(x, y, w, h, item_type);
            data
            // format!("{{\"x\": {},\"y\": {},\"w\": {},\"h\": {},{}}}",)
        }

        /// virtuals
        fn load_item_data_default(&self) {}
        fn serialise_item_default(&self) -> CanvasItemType {
            panic!("Implement virtual method `serialise_item` for your widget")
        }
        fn style_default(&self) {
            panic!("Implement virtual method `style` for your widget")
        }

        pub fn is_presentation_mode(&self) -> bool {
            self.canvas
                .upgrade()
                .and_then(|c| Some(c.presentation_mode()))
                .unwrap_or(false)
        }
    }
}

use gtk::glib::object::{IsA, ObjectExt};
use gtk::glib::subclass::types::{
    ClassStruct, IsSubclassable, ObjectSubclass, ObjectSubclassIsExt,
};
use gtk::glib::{self};
use gtk::prelude::*;
use gtk::subclass::box_::BoxImpl;
use gtk::subclass::prelude::*;

use crate::widgets::canvas::serialise::{CanvasItemData, CanvasItemType};

pub(super) mod signals {
    pub const CLICKED: &str = "clicked";
    pub const UN_SELECT: &str = "un-select";
    pub const SET_AS_PRIMARY: &str = "set-as-primary";
    pub const MOVE_ITEM: &str = "move-item";
    pub const CHECK_POSITION: &str = "check-position";
    pub const ACTIVE_CHANGED: &str = "active-changed";
}

glib::wrapper! {
pub struct CanvasItem(ObjectSubclass<imp::CanvasItem>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for CanvasItem {
    fn default() -> Self {
        glib::Object::new()
    }
}

pub trait CanvasItemExt: IsA<CanvasItem> {
    fn get_save_data(&self) -> Option<CanvasItemData> {
        self.upcast_ref::<CanvasItem>().imp().get_save_data()
    }
    fn load_data(&self) {
        self.upcast_ref::<CanvasItem>().imp().load_data();
    }
    fn serialise(&self) -> CanvasItemData {
        self.upcast_ref::<CanvasItem>().imp().serialise()
    }
    fn unselect(&self) {
        self.upcast_ref::<CanvasItem>().imp().unselect();
    }
    fn delete(&self) {
        self.upcast_ref::<CanvasItem>().imp().delete();
    }

    fn emit_clicked(&self) {
        self.emit_by_name::<()>(signals::CLICKED, &[]);
    }
    fn connect_clicked<F: Fn(&CanvasItem) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            signals::CLICKED,
            false,
            glib::closure_local!(move |c: &CanvasItem| {
                f(c);
            }),
        )
    }

    fn connect_checkposition<F: Fn(&CanvasItem) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            signals::CHECK_POSITION,
            false,
            glib::closure_local!(move |c: &CanvasItem| {
                f(c);
            }),
        )
    }

    fn connect_move_item<F: Fn(&CanvasItem, i32, i32) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            signals::MOVE_ITEM,
            false,
            glib::closure_local!(move |c: &CanvasItem, x: i32, y: i32| {
                f(c, x, y);
            }),
        )
    }

    fn connect_unselect<F: Fn(&CanvasItem) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            signals::UN_SELECT,
            false,
            glib::closure_local!(move |c: &CanvasItem| {
                f(c);
            }),
        )
    }

    // NOTE: VIIRTUAL methods below

    // const TEXT_STYLE_CSS: &str = "";
    fn load_item_data(&self) {
        let obj = self.upcast_ref::<CanvasItem>();
        (obj.class().as_ref().load_item_data)(obj);
    }

    fn serialise_item(&self) -> CanvasItemType {
        let obj = self.upcast_ref::<CanvasItem>();
        (obj.class().as_ref().serialise_item)(obj)
    }
    fn style(&self) {
        let obj = self.upcast_ref::<CanvasItem>();
        (obj.class().as_ref().style)(obj);
    }
    fn style_css(&self) -> String {
        let obj = self.upcast_ref::<CanvasItem>();
        (obj.class().as_ref().style_css)(obj)
    }
}

impl<O: IsA<CanvasItem>> CanvasItemExt for O {}

pub trait CanvasItemImpl: BoxImpl + ObjectSubclass<Type: IsA<CanvasItem>> {
    fn style_css(&self) -> String {
        self.parent_style_css()
    }
    fn load_item_data(&self) {
        self.parent_load_item_data()
    }

    fn serialise_item(&self) -> CanvasItemType {
        self.parent_serialise_item()
    }
    fn style(&self) {
        self.parent_style()
    }
}

pub trait CanvasItemImplExt: CanvasItemImpl {
    fn parent_load_item_data(&self) {
        unsafe {
            let data = Self::type_data();
            let parent_class = &*(data.as_ref().parent_class() as *mut Class);
            (parent_class.load_item_data)(self.obj().unsafe_cast_ref())
        }
    }

    fn parent_serialise_item(&self) -> CanvasItemType {
        unsafe {
            let data = Self::type_data();
            let parent_class = &*(data.as_ref().parent_class() as *mut Class);
            (parent_class.serialise_item)(self.obj().unsafe_cast_ref())
        }
    }
    fn parent_style(&self) {
        unsafe {
            let data = Self::type_data();
            let parent_class = &*(data.as_ref().parent_class() as *mut Class);
            (parent_class.style)(self.obj().unsafe_cast_ref())
        }
    }
    fn parent_style_css(&self) -> String {
        unsafe {
            let data = Self::type_data();
            let parent_class = &*(data.as_ref().parent_class() as *mut Class);
            (parent_class.style_css)(self.obj().unsafe_cast_ref())
        }
    }
}

impl<T: CanvasItemImpl> CanvasItemImplExt for T {}

unsafe impl<T: CanvasItemImpl> IsSubclassable<T> for CanvasItem {
    fn class_init(class: &mut glib::Class<Self>) {
        Self::parent_class_init::<T>(class.upcast_ref_mut());

        let klass = class.as_mut();
        klass.load_item_data = |obj| unsafe {
            let imp = obj.unsafe_cast_ref::<T::Type>().imp();
            imp.load_item_data()
        };
        klass.serialise_item = |obj| unsafe {
            let imp = obj.unsafe_cast_ref::<T::Type>().imp();
            imp.serialise_item()
        };
        klass.style = |obj| unsafe {
            let imp = obj.unsafe_cast_ref::<T::Type>().imp();
            imp.style()
        };
        klass.style_css = |obj| unsafe {
            let imp = obj.unsafe_cast_ref::<T::Type>().imp();
            imp.style_css()
        };
    }
    // fn class_init(class: &mut glib::Class<Self>) {
    //     <glib::Object as IsSubclassable<T>>::class_init(class);
    // }
    //
    // fn instance_init(instance: &mut glib::subclass::InitializingObject<T>) {
    //     <glib::Object as IsSubclassable<T>>::instance_init(instance);
    // }
}

/// GObject class struct with the function pointers for the virtual methods.
///
/// This must be `#[repr(C)]`.
#[repr(C)]
pub struct Class {
    pub parent_class: gtk::ffi::GtkBoxClass,
    // If these functions are meant to be called from C, you need to make these functions
    // `unsafe extern "C" fn` & use FFI-safe types (usually raw pointers).
    pub load_item_data: fn(&CanvasItem),
    pub serialise_item: fn(&CanvasItem) -> CanvasItemType,
    pub style: fn(&CanvasItem),
    pub style_css: fn(&CanvasItem) -> String,
}

/// Make it possible to use this struct as class struct in an `ObjectSubclass`
/// trait implementation.
///
/// This is `unsafe` to enforce that the struct is `#[repr(C)]`.
unsafe impl ClassStruct for Class {
    type Type = imp::CanvasItem;
}

/// Deref directly to the parent class' class struct.
impl std::ops::Deref for Class {
    type Target = glib::Class<<<Self as ClassStruct>::Type as ObjectSubclass>::ParentType>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(&self.parent_class as *const _ as *const _) }
    }
}
