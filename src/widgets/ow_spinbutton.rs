use gtk::{
    gdk,
    glib::{
        self,
        object::{Cast, CastNone, ObjectExt},
        property::PropertySet,
        subclass::types::ObjectSubclassIsExt,
    },
    prelude::{
        AccessibleExtManual, AdjustmentExt, ButtonExt, EditableExt, EventControllerExt, GestureExt,
        GestureSingleExt, WidgetExt,
    },
};
use icu::decimal::DecimalFormatter;
use icu::decimal::input::Decimal;
use icu::locale::Locale;

const MAX_TIMER_CALLS: i32 = 5;
const EPSILON: f64 = 1e-10;
const MAX_DIGITS: i32 = 20;
const TIMEOUT_INITIAL: u64 = 500;
const TIMEOUT_REPEAT: u64 = 50;
const GTK_ERROR_INPUT: i32 = -1;

macro_rules! g_assert {
    ($domain:expr, $cond:expr) => {{
        match $cond.partial_cmp(&true) {
            Some(std::cmp::Ordering::Equal) => true,
            _ => {
                let message = format!("assertion `{}` failed", stringify!($cond));
                glib::g_critical!($domain, "{}", message);
                panic!("{}", message);
            }
        };
    }};

    ($domain:expr, $cond:expr, $($arg:tt)*) => {{
        match $cond.partial_cmp(&true) {
            Some(std::cmp::Ordering::Equal) => true,
            _ => {
                let message = format!(
                    "assertion `{}` failed: {}",
                    stringify!($cond),
                    format_args!($($arg)*)
                );

                glib::g_critical!($domain, "{}", &message);
                panic!("{}", message);
            }
        }
    }}
}

mod signals {
    pub(super) const INPUT: &str = "input";
    pub(super) const OUTPUT: &str = "output";
    pub(super) const VALUE_CHANGED: &str = "value-changed";
    pub(super) const ACTIVATE: &str = "activate";
    pub(super) const CHANGE_VALUE: &str = "change-value";
    pub(super) const WRAPPED: &str = "wrapped";

    // int (*input)  (GtkSpinButton *spin_button,
    //                double        *new_value);
    // int (*output) (GtkSpinButton *spin_button);
    // void (*value_changed) (GtkSpinButton *spin_button);
    // void (*activate)      (GtkSpinButton *spin_button);
    //
    // /* Action signals for keybindings, do not connect to these */
    // void (*change_value) (GtkSpinButton *spin_button,
    //                       GtkScrollType  scroll);
    //
    // void (*wrapped) (GtkSpinButton *spin_button);
}

mod imp {
    use std::{
        cell::{Cell, RefCell},
        sync::OnceLock,
    };

    use crate::utils::WidgetChildrenExt;

    use super::*;

    use gtk::{
        gdk,
        glib::{
            self, Properties,
            object::Cast,
            subclass::{
                Signal,
                object::{ObjectImpl, ObjectImplExt},
                types::{ObjectSubclass, ObjectSubclassExt, ObjectSubclassIsExt},
            },
            translate::IntoGlib,
            types::StaticType,
            variant::ToVariant,
        },
        prelude::{
            AdjustmentExt, BoxExt, /*CellEditableExt,*/ EditableExt, EditableExtManual,
            GestureExt, OrientableExt, WidgetExt,
        },
        subclass::{
            prelude::{
                AccessibleImpl, AccessibleRangeImpl,
                /*CellEditableImpl,*/ DerivedObjectProperties, EditableImpl,
            },
            widget::{WidgetClassExt, WidgetImpl, WidgetImplExt},
        },
    };

    #[derive(Properties)]
    #[properties(wrapper_type = super::OwSpinButton)]
    pub struct OwSpinButton {
        #[property(name = "value", type=f64, get=Self::_get_value, set=Self::_set_value, explicit_notify)]
        #[property(get, set, explicit_notify)]
        pub(super) adjustment: RefCell<gtk::Adjustment>,
        pub(super) adjustment_handlers: RefCell<Vec<glib::SignalHandlerId>>,

        pub(super) entry: RefCell<gtk::Text>,

        pub(super) up_button: RefCell<gtk::Button>,
        pub(super) down_button: RefCell<gtk::Button>,
        pub(super) button_box: RefCell<gtk::Box>,

        pub(super) click_child: RefCell<Option<gtk::Button>>,

        pub(super) timer: RefCell<Option<glib::SourceId>>,

        #[property(
            get,
            set,
            explicit_notify,
            builder(gtk::SpinButtonUpdatePolicy::Always)
        )]
        pub(super) update_policy: RefCell<gtk::SpinButtonUpdatePolicy>,
        #[property(get, set, explicit_notify)]
        pub(super) climb_rate: Cell<f64>,
        pub(super) timer_step: Cell<f64>,
        pub(super) swipe_remainder: Cell<f64>,

        #[property(get, set=Self::_set_orientation, builder(gtk::Orientation::Vertical))]
        pub(super) orientation: RefCell<gtk::Orientation>,

        #[property(get, set=Self::_set_digits, explicit_notify)]
        pub(super) digits: Cell<i32>,
        pub(super) need_timer: Cell<bool>,
        #[property(get, set=Self::_set_numeric, explicit_notify)]
        pub(super) numeric: Cell<bool>,
        #[property(get, set=Self::_set_snap_to_tick, explicit_notify)]
        pub(super) snap_to_ticks: Cell<bool>,
        pub(super) timer_calls: Cell<i32>,
        #[property(get, set=Self::_set_wrap, explicit_notify)]
        pub(super) wrap: Cell<bool>,
        // #[property(get, set=Self::_set_activates_default, override_interface=gtk::CellEditable)]
        // pub(super) editing_canceled: Cell<bool>,
        pub(super) edited: Cell<bool>,
        #[property(get, set, explicit_notify)]
        pub(super) activates_default: Cell<bool>,

        pub(super) insert_text_handler: RefCell<Option<glib::SignalHandlerId>>,
    }

    impl Default for OwSpinButton {
        fn default() -> Self {
            Self {
                adjustment: RefCell::new(gtk::Adjustment::default()),
                adjustment_handlers: RefCell::new(Vec::new()),
                entry: RefCell::new(gtk::Text::default()),
                button_box: RefCell::new(gtk::Box::new(gtk::Orientation::Vertical, 0)),
                up_button: RefCell::new(Self::build_spin_button_widget("pan-up-symbolic", "up")),
                down_button: RefCell::new(Self::build_spin_button_widget(
                    "pan-down-symbolic",
                    "down",
                )),
                click_child: RefCell::new(None),
                timer: RefCell::new(None),
                update_policy: RefCell::new(gtk::SpinButtonUpdatePolicy::Always),
                climb_rate: Cell::new(f64::default()),
                timer_step: Cell::new(f64::default()),
                swipe_remainder: Cell::new(f64::default()),
                orientation: RefCell::new(gtk::Orientation::Vertical),
                digits: Cell::new(i32::default()),
                need_timer: Cell::new(bool::default()),
                numeric: Cell::new(bool::default()),
                snap_to_ticks: Cell::new(bool::default()),
                timer_calls: Cell::new(i32::default()),
                wrap: Cell::new(bool::default()),
                // editing_canceled: Cell::new(bool::default()),
                edited: Cell::new(bool::default()),
                activates_default: Cell::new(bool::default()),
                insert_text_handler: RefCell::new(None),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for OwSpinButton {
        const NAME: &'static str = "OwSpinButton";
        type Type = super::OwSpinButton;
        type ParentType = gtk::Widget;
        type Interfaces = (
            gtk::Editable,
            gtk::AccessibleRange, /*, gtk::CellEditable*/
        );

        fn class_init(klass: &mut Self::Class) {
            let add_binding = |klass: &mut glib::subclass::basic::ClassStruct<OwSpinButton>,
                               keyval: gdk::Key,
                               mask: gdk::ModifierType,
                               scroll: gtk::ScrollType| {
                let trigger = gtk::KeyvalTrigger::new(keyval, mask);
                let action = gtk::SignalAction::new(signals::CHANGE_VALUE);
                let shortcut = gtk::Shortcut::builder()
                    .trigger(&trigger)
                    .action(&action)
                    .arguments(&(scroll.into_glib(),).to_variant())
                    .build();
                klass.add_shortcut(&shortcut);
            };

            add_binding(
                klass,
                gdk::Key::Up,
                gdk::ModifierType::empty(),
                gtk::ScrollType::StepUp,
            );
            add_binding(
                klass,
                gdk::Key::KP_Up,
                gdk::ModifierType::empty(),
                gtk::ScrollType::StepUp,
            );
            add_binding(
                klass,
                gdk::Key::Down,
                gdk::ModifierType::empty(),
                gtk::ScrollType::StepDown,
            );
            add_binding(
                klass,
                gdk::Key::KP_Down,
                gdk::ModifierType::empty(),
                gtk::ScrollType::StepDown,
            );
            add_binding(
                klass,
                gdk::Key::Page_Up,
                gdk::ModifierType::empty(),
                gtk::ScrollType::PageUp,
            );
            add_binding(
                klass,
                gdk::Key::Page_Down,
                gdk::ModifierType::empty(),
                gtk::ScrollType::PageDown,
            );
            add_binding(
                klass,
                gdk::Key::End,
                gdk::ModifierType::CONTROL_MASK,
                gtk::ScrollType::End,
            );
            add_binding(
                klass,
                gdk::Key::Home,
                gdk::ModifierType::CONTROL_MASK,
                gtk::ScrollType::Start,
            );
            add_binding(
                klass,
                gdk::Key::Page_Up,
                gdk::ModifierType::CONTROL_MASK,
                gtk::ScrollType::PageUp,
            );
            add_binding(
                klass,
                gdk::Key::Page_Down,
                gdk::ModifierType::CONTROL_MASK,
                gtk::ScrollType::PageDown,
            );

            klass.set_activate_signal_from_name(signals::ACTIVATE);
            klass.set_layout_manager_type::<gtk::BoxLayout>();
            klass.set_css_name("spinbutton");
            klass.set_accessible_role(gtk::AccessibleRole::SpinButton);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            unsafe { obj.as_ref() }.init_delegate();
        }
    }

    impl ObjectImpl for OwSpinButton {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }
        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                editable_property @ ("text" | "cursor-position" | "selection-bound"
                | "editable" | "width-chars" | "max-width-chars"
                | "xalign" | "enable-undo") => {
                    self.entry
                        .borrow()
                        .set_property_from_value(pspec.name(), value);
                    if editable_property == "width-chars" {
                        self.update_width_chars();
                    }
                }
                _ => self.derived_set_property(id, value, pspec),
            }
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "text" | "cursor-position" | "selection-bound" | "editable" | "width-chars"
                | "max-width-chars" | "xalign" | "enable-undo" => {
                    self.entry.borrow().property_value(pspec.name())
                }
                _ => self.derived_property(id, pspec),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj().clone();

            obj.set_width_chars(0);
            obj.set_overflow(gtk::Overflow::Hidden);
            self.set_max_width_chars(0);

            let entry = self.entry.borrow().clone();
            entry.set_hexpand(true);
            entry.set_vexpand(true);
            entry.connect_activate(glib::clone!(
                #[weak]
                obj,
                move |entry| {
                    //spin_activate
                    if !entry.is_editable() {
                        return;
                    }
                    let was_edited = obj.imp().edited.get();
                    obj.update();

                    if !was_edited {
                        obj.emit_activate();
                    }
                }
            ));
            entry.connect_changed(glib::clone!(
                #[weak]
                obj,
                move |_| obj.imp().edited.set(true)
            ));

            let insert_text_handler_id = entry.connect_insert_text(glib::clone!(
                #[weak]
                obj,
                move |entry, text, pos| obj.on_insert_text(entry, text, pos)
            ));
            self.insert_text_handler
                .replace(Some(insert_text_handler_id));

            entry.set_parent(&obj);

            {
                let seperator = gtk::Separator::new(gtk::Orientation::Vertical);
                seperator.set_parent(&obj);
            }

            let button_box = self.button_box.borrow().clone();
            button_box.set_parent(&obj);
            button_box.set_width_request(30);

            let up_btn = self.up_button.borrow().clone();
            up_btn.set_parent(&button_box);
            {
                let seperator = gtk::Separator::new(gtk::Orientation::Horizontal);
                seperator.set_parent(&button_box);
            }
            let down_btn = self.down_button.borrow().clone();
            down_btn.set_parent(&button_box);

            let down_click_gesture = gtk::GestureClick::new();
            down_click_gesture.set_button(0);
            down_click_gesture.set_touch_only(false);
            down_click_gesture.set_propagation_phase(gtk::PropagationPhase::Capture);
            down_click_gesture.connect_pressed(glib::clone!(
                #[weak]
                obj,
                move |g, n, x, y| obj.button_pressed_cb(g, n, x, y)
            ));
            down_click_gesture.connect_released(glib::clone!(
                #[weak]
                obj,
                move |g, n, x, y| obj.button_released_cb(g, n, x, y)
            ));
            down_click_gesture.connect_cancel(glib::clone!(
                #[weak]
                obj,
                move |g, s| obj.button_cancel_cb(g, s)
            ));

            down_btn.add_controller(self.build_btn_gesture());
            up_btn.add_controller(self.build_btn_gesture());

            // TODO: set adjustment to null

            let swipe_gesture = gtk::GestureSwipe::new();
            swipe_gesture.set_touch_only(true);
            swipe_gesture.set_propagation_phase(gtk::PropagationPhase::Capture);
            swipe_gesture.connect_begin(glib::clone!(
                #[weak]
                obj,
                move |gesture, event| obj.swipe_gesture_begin(gesture, event)
            ));
            swipe_gesture.connect_update(glib::clone!(
                #[weak]
                obj,
                move |gesture, event| obj.swipe_gesture_update(gesture, event)
            ));
            obj.text_widget().add_controller(swipe_gesture);

            let scroll_ctl = gtk::EventControllerScroll::new(
                gtk::EventControllerScrollFlags::VERTICAL
                    | gtk::EventControllerScrollFlags::DISCRETE,
            );
            scroll_ctl.connect_scroll(glib::clone!(
                #[weak]
                obj,
                #[upgrade_or]
                glib::Propagation::Proceed,
                move |event, dx, dy| obj.imp().scroll_controller_scroll(event, dx, dy)
            ));
            obj.add_controller(scroll_ctl);

            let key_ctl = gtk::EventControllerKey::new();
            key_ctl.connect_key_released(glib::clone!(
                #[weak]
                obj,
                move |_, _, _, _| {
                    let imp = obj.imp();
                    imp.timer_step.set(obj.adjustment().step_increment());
                    imp.timer_calls.set(0);
                }
            ));
            obj.add_controller(key_ctl);

            let focus_ctl = gtk::EventControllerFocus::new();
            focus_ctl.connect_leave(glib::clone!(
                #[weak]
                obj,
                move |event| obj.key_ctl_focus_out(event)
            ));
            obj.add_controller(focus_ctl);
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();

            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder(signals::INPUT)
                        .param_types([f64::static_type()])
                        .return_type::<i32>()
                        .run_last()
                        .build(),
                    Signal::builder(signals::OUTPUT)
                        .run_last()
                        .return_type::<bool>()
                        .build(),
                    Signal::builder(signals::VALUE_CHANGED).run_last().build(),
                    Signal::builder(signals::ACTIVATE)
                        .run_last()
                        .action()
                        .class_handler(|_, args| {
                            // real_button_activate
                            let spin = args[0]
                                .get::<super::OwSpinButton>()
                                .expect("expected OwSpinButton");
                            if spin.imp().activates_default.get() {
                                let _ = spin.activates_default();
                            }
                            None
                        })
                        .build(),
                    Signal::builder(signals::CHANGE_VALUE)
                        .action()
                        .run_last()
                        .class_handler(|_, args| {
                            let spin = args[0]
                                .get::<super::OwSpinButton>()
                                .expect("expected OwSpinButton");
                            let scroll = args[1]
                                .get::<gtk::ScrollType>()
                                .expect("expected ScrollType");
                            spin.real_change_value(scroll);
                            None
                        })
                        .param_types([gtk::ScrollType::static_type()])
                        .build(),
                    Signal::builder(signals::WRAPPED).run_last().build(),
                ]
            })
        }

        fn dispose(&self) {
            let obj = self.obj();
            obj.stop_spinning();
            self.unset_adjustment();

            obj.finish_delegate();
            obj.text_widget().unparent();

            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for OwSpinButton {
        fn grab_focus(&self) -> bool {
            self.entry.borrow().grab_focus()
        }

        fn mnemonic_activate(&self, _group_cycling: bool) -> bool {
            self.entry.borrow().grab_focus()
        }

        fn realize(&self) {
            self.parent_realize();
            let obj = self.obj();
            let r_val = obj.emit_output();
            let text = obj.text_widget().text();

            if !r_val && (obj.numeric() || text.is_empty()) {
                obj.default_output();
            }
        }

        fn state_flags_changed(&self, state_flags: &gtk::StateFlags) {
            let obj = self.obj();
            if !obj.is_sensitive() {
                obj.stop_spinning();
            }

            self.parent_state_flags_changed(state_flags);
        }
    }

    impl AccessibleImpl for OwSpinButton {
        fn platform_state(&self, state: gtk::AccessiblePlatformState) -> bool {
            self.obj().delegate_get_accessible_platform_state(state)
        }
    }

    impl EditableImpl for OwSpinButton {
        fn delegate(&self) -> Option<gtk::Editable> {
            Some(self.entry.borrow().clone().upcast())
        }
        fn set_selection_bounds(&self, start_position: i32, end_position: i32) {
            self.entry
                .borrow()
                .select_region(start_position, end_position);
        }
    }

    impl AccessibleRangeImpl for OwSpinButton {
        fn set_current_value(&self, value: f64) -> bool {
            self.obj().set_value(value);
            true
        }
    }
    // impl CellEditableImpl for OwSpinButton {
    //     fn start_editing(&self, _event: Option<&gdk::Event>) {
    //         self.obj().text_widget().connect_activate(glib::clone!(
    //             #[weak(rename_to=imp)]
    //             self,
    //             move |_| imp.cell_editable_spinbutton_activated()
    //         ));
    //
    //         let key_controller = gtk::EventControllerKey::new();
    //         key_controller.connect_key_pressed(glib::clone!(
    //             #[weak(rename_to=imp)]
    //             self,
    //             #[upgrade_or]
    //             glib::Propagation::Proceed,
    //             move |key, keyval, keycode, modifier| {
    //                 imp.cell_editable_spinbutton_key_pressed(key, keyval, keycode, modifier)
    //             }
    //         ));
    //
    //         self.obj().text_widget().add_controller(key_controller);
    //     }
    // }

    impl OwSpinButton {
        fn _set_value(&self, value: f64) {
            let obj = self.obj();

            if (value - obj.adjustment().value()).abs() > EPSILON
                || value < obj.adjustment().lower()
                || value > obj.adjustment().upper()
            {
                obj.adjustment().set_value(value);
            } else {
                let r_val = obj.emit_output();
                if !r_val {
                    obj.default_output();
                }
            }

            self.edited.set(false);
        }

        fn _get_value(&self) -> f64 {
            self.obj().adjustment().value()
        }

        pub(super) fn _set_orientation(&self, orientation: gtk::Orientation) {
            let btn_box = self.button_box.borrow().clone();
            let Some(b_seperator) = btn_box.get_children::<gtk::Separator>().next() else {
                glib::g_critical!("OwSpinButton", "Could not find button seperator");
                return;
            };

            self.orientation.set(orientation);
            btn_box.set_orientation(orientation);

            let up_btn = self.up_button.borrow().clone();
            let down_btn = self.down_button.borrow().clone();

            while let Some(v) = btn_box.first_child() {
                v.unparent();
            }

            match orientation {
                gtk::Orientation::Horizontal => {
                    up_btn.set_icon_name("pan-end-symbolic");
                    down_btn.set_icon_name("pan-start-symbolic");
                    btn_box.append(&down_btn);
                    btn_box.append(&b_seperator);
                    btn_box.append(&up_btn);
                }
                gtk::Orientation::Vertical => {
                    up_btn.set_icon_name("pan-up-symbolic");
                    down_btn.set_icon_name("pan-down-symbolic");
                    btn_box.append(&up_btn);
                    btn_box.append(&b_seperator);
                    btn_box.append(&down_btn);
                }
                _ => return,
            };

            self.obj().notify_orientation();
        }

        // fn set_width_chars(&self, value: i32) {
        //     self.width_chars.set(value);
        //     self.update_width_chars();
        // }
        fn set_max_width_chars(&self, value: i32) {
            self.entry.borrow().set_max_width_chars(value);
        }

        fn _set_numeric(&self, numeric: bool) {
            if self.numeric.get() != numeric {
                self.numeric.set(numeric);
                let input = if numeric {
                    (gtk::InputPurpose::Number, gtk::InputHints::NO_EMOJI)
                } else {
                    (gtk::InputPurpose::FreeForm, gtk::InputHints::NONE)
                };

                self.entry.borrow().set_input_purpose(input.0);
                self.entry.borrow().set_input_hints(input.1);

                if numeric {
                    self.entry.borrow().set_direction(gtk::TextDirection::Ltr);
                    self.obj().notify_numeric();
                }
            }
        }

        fn _set_adjustment(&self, adj: gtk::Adjustment) {
            self.configure(Some(adj), self.climb_rate.get(), self.digits.get());
        }

        fn _set_digits(&self, value: i32) {
            self.configure(Some(self.obj().adjustment()), self.climb_rate.get(), value);
        }

        fn unset_adjustment(&self) {
            // TODO: take adjustment
            let adjustment = self.adjustment.borrow();
            for id in self.adjustment_handlers.borrow_mut().drain(..) {
                adjustment.disconnect(id);
            }
        }

        pub(super) fn configure(
            &self,
            adjustment: Option<gtk::Adjustment>,
            climb_rate: f64,
            digits: i32,
        ) {
            let adj = adjustment.unwrap_or_else(|| self.adjustment.borrow().clone());

            let obj = self.obj();
            let _ = obj.freeze_notify();

            if obj.adjustment() != adj {
                self.unset_adjustment();
                obj.set_adjustment(adj.clone());

                adj.connect_value_changed(glib::clone!(
                    #[weak]
                    obj,
                    move |adjustment| obj.value_changed(adjustment)
                ));
                adj.connect_changed(glib::clone!(
                    #[weak]
                    obj,
                    move |adjustment| obj.adjustment_change_cb(adjustment)
                ));

                self.timer_step.set(adj.step_increment());
                obj.notify_adjustment();
                obj.queue_resize();
            }

            if obj.digits() != digits {
                obj.set_digits(digits);
                obj.notify_digits();
            }

            if obj.climb_rate() != climb_rate {
                obj.set_climb_rate(climb_rate);
                obj.notify_climb_rate();
            }

            self.update_width_chars();
            obj.update_property(&[
                gtk::accessible::Property::ValueMax(adj.upper() - adj.page_size()),
                gtk::accessible::Property::ValueMin(adj.lower()),
                gtk::accessible::Property::ValueNow(adj.value()),
            ]);

            obj.value_changed(&adj);
        }

        // fn _set_editing_canceled(&self, value: bool) {
        //     if self.editing_canceled.get() != value {
        //         self.editing_canceled.set(value);
        //         // NOTE: maybe notify?
        //         self.obj().notify("editing_canceled");
        //     }
        // }

        fn _set_activates_default(&self, value: bool) {
            if self.activates_default.get() != value {
                self.activates_default.set(value);
                self.obj().notify_activates_default();
            }
        }

        fn _set_snap_to_tick(&self, value: bool) {
            if self.snap_to_ticks.get() != value {
                self.snap_to_ticks.set(value);
                if value {
                    self.obj().update();
                }
                self.obj().notify_snap_to_ticks();
            }
        }

        fn _set_wrap(&self, value: bool) {
            if self.wrap.get() != value {
                self.wrap.set(value);
                self.obj().notify_wrap();
                self.obj().update_button_sensitivity();
            }
        }

        fn _set_update_policy(&self, value: gtk::SpinButtonUpdatePolicy) {
            if *self.update_policy.borrow() != value {
                self.update_policy.replace(value);
                self.obj().notify_update_policy();
            }
        }
    }

    impl OwSpinButton {
        // fn cell_editable_spinbutton_activated(&self) {
        //     let obj = self.obj();
        //     obj.editing_done();
        //     obj.remove_widget();
        // }

        // fn cell_editable_spinbutton_key_pressed(
        //     &self,
        //     _key: &gtk::EventControllerKey,
        //     keyval: gdk::Key,
        //     _keycode: u32,
        //     _modifier: gdk::ModifierType,
        // ) -> glib::Propagation {
        //     let spin = self.obj();
        //     if keyval == gdk::Key::Escape {
        //         spin.set_editing_canceled(true);
        //         spin.editing_done();
        //         spin.remove_widget();
        //
        //         return glib::Propagation::Stop;
        //     }
        //
        //     if keyval == gdk::Key::Up {
        //         spin.editing_done();
        //         spin.remove_widget();
        //         return glib::Propagation::Stop;
        //     }
        //
        //     glib::Propagation::Proceed
        // }

        fn scroll_controller_scroll(
            &self,
            _event: &gtk::EventControllerScroll,
            _dx: f64,
            dy: f64,
        ) -> glib::Propagation {
            let spin = self.obj();
            if spin.has_focus() {
                spin.grab_focus();
            }

            spin.real_spin(-dy * spin.adjustment().step_increment());

            glib::Propagation::Stop
        }

        fn update_width_chars(&self) {
            let mut width_chars = 0;
            let obj = self.obj();
            let ed = obj.clone().upcast::<gtk::Editable>();

            if ed.width_chars() == -1 {
                let value = obj.adjustment().lower();
                let data = self.format_for_value(value);
                width_chars = width_chars.max(data.len() as i32);
                width_chars = width_chars.min(10);
            } else {
                width_chars = ed.width_chars();
            }

            self.entry.borrow().set_width_chars(width_chars);
        }

        pub(super) fn format_for_value(&self, value: f64) -> String {
            let digits = self.digits.get() as usize;
            let buf = format!("{:.*}", digits, value);

            weed_out_neg_zero(buf, digits)
        }

        fn build_spin_button_widget(icon_name: &str, css_class: &str) -> gtk::Button {
            let button = gtk::Button::builder()
                .accessible_role(gtk::AccessibleRole::None) // decorative; entry carries the real role
                .icon_name(icon_name)
                .build();
            button.add_css_class(css_class);
            button.set_css_classes(&["spin-flat"]);
            button.set_can_focus(false);
            if let Some(img) = button.child().and_downcast::<gtk::Image>() {
                img.set_pixel_size(14);
            };
            button.set_vexpand(true);

            button
        }

        fn build_btn_gesture(&self) -> gtk::GestureClick {
            let obj = self.obj();
            let gesture = gtk::GestureClick::new();
            gesture.set_button(0);
            gesture.set_touch_only(false);
            gesture.set_propagation_phase(gtk::PropagationPhase::Capture);
            gesture.connect_pressed(glib::clone!(
                #[weak]
                obj,
                move |g, n, x, y| obj.button_pressed_cb(g, n, x, y)
            ));
            gesture.connect_released(glib::clone!(
                #[weak]
                obj,
                move |g, n, x, y| obj.button_released_cb(g, n, x, y)
            ));
            gesture.connect_cancel(glib::clone!(
                #[weak]
                obj,
                move |g, s| obj.button_cancel_cb(g, s)
            ));
            gesture
        }
    }
}

glib::wrapper! {
pub struct OwSpinButton(ObjectSubclass<imp::OwSpinButton>)
        @extends gtk::Widget,
        @implements gtk::Accessible,  gtk::AccessibleRange, gtk::Editable/*, gtk::CellEditable*/;
}

impl Default for OwSpinButton {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl OwSpinButton {
    pub fn new(adj: Option<gtk::Adjustment>, climb_rate: f64, digits: i32) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().configure(adj, climb_rate, digits);

        obj
    }

    pub fn with_range(min: f64, max: f64, step: f64) -> Self {
        g_assert!("OwSpinButton", min <= max);
        g_assert!("OwSpinButton", step != 0.0);

        let obj: Self = glib::Object::new();

        let adj = gtk::Adjustment::new(min, min, max, step, 10.0 * step, 0.0);

        let digits = if step.abs() >= 1.0 || step == 0.0 {
            0
        } else {
            let t = step.abs().log10().floor().abs() as i32;
            t.min(MAX_DIGITS)
        };

        obj.imp().configure(Some(adj), step, digits);
        obj.set_numeric(true);

        obj
    }
}

impl OwSpinButton {
    pub fn emit_input(&self, value: f64) -> i32 {
        self.emit_by_name::<i32>(signals::INPUT, &[&value])
    }
    pub fn emit_output(&self) -> bool {
        self.emit_by_name::<bool>(signals::OUTPUT, &[])
    }
    pub fn emit_value_change(&self) {
        self.emit_by_name::<()>(signals::VALUE_CHANGED, &[])
    }
    pub fn emit_activate(&self) {
        self.emit_by_name::<()>(signals::ACTIVATE, &[])
    }
    pub fn emit_change_value(&self, scroll_type: &gtk::ScrollType) {
        self.emit_by_name::<()>(signals::CHANGE_VALUE, &[scroll_type])
    }
    pub fn emit_wrapped(&self) {
        self.emit_by_name::<()>(signals::WRAPPED, &[])
    }

    pub fn connect_input<F: Fn(&Self, f64) -> i32 + 'static>(
        &self,
        f: F,
    ) -> gtk::glib::SignalHandlerId {
        self.connect_closure(
            signals::INPUT,
            true,
            glib::closure_local!(|obj: &Self, data: f64| f(obj, data)),
        )
    }
    pub fn connect_output<F: Fn(&Self) -> bool + 'static>(
        &self,
        f: F,
    ) -> gtk::glib::SignalHandlerId {
        self.connect_closure(
            signals::OUTPUT,
            true,
            glib::closure_local!(|obj: &Self| f(obj)),
        )
    }
    pub fn connect_value_changed<F: Fn(&Self) + 'static>(
        &self,
        f: F,
    ) -> gtk::glib::SignalHandlerId {
        self.connect_closure(
            signals::VALUE_CHANGED,
            true,
            glib::closure_local!(|obj: &Self| f(obj)),
        )
    }
    pub fn connect_activate<F: Fn(&Self) + 'static>(&self, f: F) -> gtk::glib::SignalHandlerId {
        self.connect_closure(
            signals::ACTIVATE,
            true,
            glib::closure_local!(|obj: &Self| f(obj)),
        )
    }
    pub fn connect_change_value<F: Fn(&Self, &gtk::ScrollType) + 'static>(
        &self,
        f: F,
    ) -> gtk::glib::SignalHandlerId {
        self.connect_closure(
            signals::CHANGE_VALUE,
            true,
            glib::closure_local!(|obj: &Self, scroll_type: gtk::ScrollType| f(obj, &scroll_type)),
        )
    }
    pub fn connect_wrapped<F: Fn(&Self) + 'static>(&self, f: F) -> gtk::glib::SignalHandlerId {
        self.connect_closure(
            signals::WRAPPED,
            true,
            glib::closure_local!(|obj: &Self| f(obj)),
        )
    }
}

impl OwSpinButton {
    fn real_change_value(&self, scroll: gtk::ScrollType) {
        if !self.imp().entry.borrow().is_editable() {
            self.error_bell();
            return;
        }
        self.update();

        let old_value = self.adjustment().value();

        match scroll {
            gtk::ScrollType::StepBackward
            | gtk::ScrollType::StepDown
            | gtk::ScrollType::StepLeft => {
                let timer_step = self.imp().timer_step.get();
                let timer_calls = self.imp().timer_calls.get();
                self.real_spin(-timer_step);

                if self.climb_rate() > 0.0 && timer_step < self.adjustment().page_increment() {
                    if timer_calls < MAX_TIMER_CALLS {
                        self.imp().timer_calls.set(timer_calls + 1);
                    } else {
                        self.imp().timer_calls.set(0);
                        self.imp().timer_step.set(timer_step + self.climb_rate());
                    }
                }
            }

            gtk::ScrollType::StepForward | gtk::ScrollType::StepUp | gtk::ScrollType::StepRight => {
                let timer_step = self.imp().timer_step.get();
                let timer_calls = self.imp().timer_calls.get();
                self.real_spin(timer_step);

                if self.climb_rate() > 0.0 && timer_step < self.adjustment().page_increment() {
                    if timer_calls < MAX_TIMER_CALLS {
                        self.imp().timer_calls.set(timer_calls + 1);
                    } else {
                        self.imp().timer_calls.set(0);
                        self.imp().timer_step.set(timer_step + self.climb_rate());
                    }
                }
            }

            gtk::ScrollType::PageBackward
            | gtk::ScrollType::PageDown
            | gtk::ScrollType::PageLeft => self.real_spin(-self.adjustment().page_increment()),
            gtk::ScrollType::PageForward | gtk::ScrollType::PageUp | gtk::ScrollType::PageRight => {
                self.real_spin(self.adjustment().page_increment())
            }

            gtk::ScrollType::Start => {
                let diff = self.adjustment().value() - self.adjustment().lower();
                if diff > EPSILON {
                    self.real_spin(-diff);
                }
            }
            gtk::ScrollType::End => {
                let diff = self.adjustment().upper() - self.adjustment().value();
                if diff > EPSILON {
                    self.real_spin(diff);
                }
            }

            gtk::ScrollType::None | gtk::ScrollType::Jump | _ => glib::g_warning!(
                "OwSpinButton",
                "Invalid scroll type {scroll:?} for GtkSpinButton::change-value"
            ),
        }

        self.update();
        if self.adjustment().value() == old_value {
            self.error_bell();
        }
    }

    fn update(&self) {
        let mut val: f64 = 0.0;
        let mut error = false;

        let emit_val = self.emit_input(val);
        if emit_val == 0 {
            match self.default_input() {
                Ok(v) => val = v,
                Err(v) => error = v == GTK_ERROR_INPUT,
            };
        } else if emit_val == GTK_ERROR_INPUT {
            error = true;
        }

        if self.update_policy() == gtk::SpinButtonUpdatePolicy::Always {
            if val < self.adjustment().lower() {
                val = self.adjustment().lower();
            } else if val > self.adjustment().upper() {
                val = self.adjustment().upper();
            }
        } else if self.update_policy() == gtk::SpinButtonUpdatePolicy::IfValid
            && (error || val < self.adjustment().lower() || val > self.adjustment().upper())
        {
            self.value_changed(&self.adjustment());
            return;
        }

        if self.snap_to_ticks() {
            self.snap(val);
        } else {
            self.set_value(val);
        }
    }

    fn real_spin(&self, increment: f64) {
        let mut wrapped = false;
        let adjustment = self.adjustment();
        let mut new_value = adjustment.value() + increment;

        let upper = adjustment.upper();
        let lower = adjustment.lower();
        if increment > 0.0 {
            if self.wrap() {
                if (adjustment.value() - upper).abs() > EPSILON {
                    new_value = lower;
                    wrapped = true;
                } else if new_value > upper {
                    new_value = upper
                }
            } else {
                new_value = new_value.min(upper);
            }
        } else if increment < 0.0 {
            if self.wrap() {
                if (adjustment.value() - lower).abs() < EPSILON {
                    new_value = upper;
                    wrapped = true;
                } else if new_value < lower {
                    new_value = lower
                }
            } else {
                new_value = new_value.max(lower)
            }
        }

        if (new_value - adjustment.value()).abs() > EPSILON {
            adjustment.set_value(new_value);
        }

        if wrapped {
            self.emit_wrapped();
        }
    }

    fn default_input(&self) -> Result<f64, i32> {
        let text = self.text_widget().text();
        let text = text.trim();

        if let Ok(t) = text.parse::<f64>() {
            return Ok(t);
        };

        let mut val = 0i32;
        let mut sign = 1i32;
        let mut iter = text.char_indices();
        for (i, ch) in &mut iter {
            if i == 0 && ch == '-' {
                sign = -1;
                continue;
            }

            // val.checked_mul(10).and_then(|v| v.checked_add(0));
            match ch.to_digit(10) {
                Some(d) => val = val * 10 + d as i32,
                None => break,
            };
        }

        if iter.next().is_some() {
            return Err(GTK_ERROR_INPUT);
        }

        Ok((sign * val) as f64)
    }

    fn default_output(&self) {
        let buf = self.imp().format_for_value(self.adjustment().value());

        let entry = self.text_widget();
        if buf != entry.text() {
            entry.set_text(&buf);
        }
    }

    fn value_changed(&self, adjustment: &gtk::Adjustment) {
        let r_val = self.emit_output();
        if !r_val {
            self.default_output();
        }
        self.emit_value_change();

        self.update_property(&[gtk::accessible::Property::ValueNow(adjustment.value())]);
        self.update_button_sensitivity();
        self.notify_value();
        self.imp().edited.set(false);
    }

    fn snap(&self, mut val: f64) {
        let adjustment = self.adjustment();
        let inc = adjustment.step_increment();

        if inc != 0.0 {
            let tmp = (val - self.adjustment().lower()) / inc;

            if tmp - tmp.floor() < tmp.ceil() - tmp {
                val = adjustment.lower() + tmp.floor() * inc;
            } else {
                val = adjustment.lower() + tmp.ceil() * inc;
            }
        }

        self.adjustment().set_value(val)
    }

    fn text_widget(&self) -> gtk::Text {
        self.imp().entry.borrow().clone()
    }

    fn update_button_sensitivity(&self) {
        let lower = self.adjustment().lower();
        let upper = self.adjustment().upper();
        let value = self.adjustment().value();
        let imp = self.imp();

        imp.up_button
            .borrow()
            .set_sensitive(self.wrap() || upper - value > EPSILON);
        imp.down_button
            .borrow()
            .set_sensitive(self.wrap() || value - lower > EPSILON);
    }

    fn adjustment_change_cb(&self, adj: &gtk::Adjustment) {
        self.imp().timer_step.set(adj.step_increment());
        self.update_button_sensitivity();

        self.update_property(&[
            gtk::accessible::Property::ValueMax(adj.upper() - adj.page_size()),
            gtk::accessible::Property::ValueMin(adj.lower()),
            gtk::accessible::Property::ValueNow(adj.value()),
        ]);
        self.queue_resize();
    }

    fn stop_spinning(&self) -> bool {
        let mut did_spin = false;
        let imp = self.imp();

        if let Some(id) = imp.timer.take() {
            id.remove();
            imp.need_timer.set(false);
            did_spin = true
        }

        imp.timer_step.set(self.adjustment().step_increment());
        imp.timer_calls.set(0);
        imp.click_child.replace(None);

        did_spin
    }

    fn start_spinning(&self, click_child: &gtk::Button, step: f64) {
        let imp = self.imp();

        imp.click_child.replace(Some(click_child.clone()));

        if imp.timer.borrow().is_none() {
            imp.timer_step.set(step);
            imp.need_timer.set(true);

            let timer = glib::timeout_add_local(
                std::time::Duration::from_millis(TIMEOUT_INITIAL),
                glib::clone!(
                    #[weak(rename_to=obj)]
                    self,
                    #[upgrade_or]
                    glib::ControlFlow::Break,
                    move || obj._timer()
                ),
            );

            imp.timer.replace(Some(timer));
        }

        let inc = match *click_child == imp.up_button.borrow().clone() {
            true => step,
            false => -step,
        };

        self.real_spin(inc);
    }

    fn _timer(&self) -> glib::ControlFlow {
        let imp = self.imp();

        if imp.timer.borrow().is_some() {
            let click_child = imp.click_child.borrow().clone();
            let step = imp.timer_step.get();

            let clicked_up_btn = click_child.is_some_and(|v| v == *imp.up_button.borrow());
            let delta = if clicked_up_btn { step } else { -step };
            self.real_spin(delta);

            if imp.need_timer.get() {
                imp.need_timer.set(false);
                let timer = glib::timeout_add_local(
                    std::time::Duration::from_millis(TIMEOUT_REPEAT),
                    glib::clone!(
                        #[weak(rename_to=obj)]
                        self,
                        #[upgrade_or]
                        glib::ControlFlow::Break,
                        move || obj._timer()
                    ),
                );

                imp.timer.set(Some(timer));
            } else {
                if imp.climb_rate.get() > 0.0
                    && imp.timer_step.get() < self.adjustment().page_increment()
                {
                    if imp.timer_calls.get() < MAX_TIMER_CALLS {
                        imp.timer_calls.set(imp.timer_calls.get() + 1);
                    } else {
                        imp.timer_calls.set(0);
                        imp.timer_step
                            .set(imp.timer_step.get() + imp.climb_rate.get());
                    }
                }

                return glib::ControlFlow::Continue;
            }
        }

        glib::ControlFlow::Break
    }

    fn button_pressed_cb(&self, gesture: &gtk::GestureClick, _n_presses: i32, _x: f64, _y: f64) {
        let Some(pressed_button) = gesture.widget().and_downcast::<gtk::Button>() else {
            return;
        };

        self.grab_focus();
        if self.text_widget().is_editable() {
            let mouse_btn = gesture.current_button();
            self.update();

            if mouse_btn == gdk::BUTTON_PRIMARY {
                self.start_spinning(&pressed_button, self.adjustment().step_increment());
            } else if mouse_btn == gdk::BUTTON_MIDDLE {
                self.start_spinning(&pressed_button, self.adjustment().page_increment());
            }
            gesture.set_state(gtk::EventSequenceState::Claimed);
        } else {
            self.error_bell();
        }
    }

    fn button_released_cb(&self, gesture: &gtk::GestureClick, _n_presses: i32, _x: f64, _y: f64) {
        let mouse_btn = gesture.current_button();
        self.stop_spinning();
        let imp = self.imp();

        if mouse_btn == gdk::BUTTON_SECONDARY {
            let Some(btn_widget) = gesture.widget() else {
                return;
            };
            let diff;

            if imp.down_button.borrow().clone().upcast::<gtk::Widget>() == btn_widget {
                diff = self.adjustment().value() - self.adjustment().lower();
                if diff > EPSILON {
                    self.real_spin(-diff);
                }
            } else if imp.up_button.borrow().clone().upcast::<gtk::Widget>() == btn_widget {
                diff = self.adjustment().upper() - self.adjustment().value();
                if diff > EPSILON {
                    self.real_spin(diff);
                }
            }
        }
    }

    fn button_cancel_cb(
        &self,
        _gesture: &gtk::GestureClick,
        _sequesnce: Option<&gdk::EventSequence>,
    ) {
        self.stop_spinning();
    }

    fn key_ctl_focus_out(&self, _: &gtk::EventControllerFocus) {
        if self.text_widget().is_editable() {
            self.update();
        }
    }

    fn swipe_gesture_begin(
        &self,
        gesture: &gtk::GestureSwipe,
        _event: Option<&gdk::EventSequence>,
    ) {
        gesture.set_state(gtk::EventSequenceState::Claimed);
        self.grab_focus();
        self.imp().swipe_remainder.set(0.0);
    }

    fn swipe_gesture_update(
        &self,
        gesture: &gtk::GestureSwipe,
        _event: Option<&gdk::EventSequence>,
    ) {
        let Some(g) = gesture.downcast_ref::<gtk::GestureSwipe>() else {
            return;
        };

        let Some((_, vel_y)) = g.velocity() else {
            return;
        };

        let step = (-vel_y / 20.0) + self.imp().swipe_remainder.get();
        let swipe_remainder = step % self.adjustment().step_increment();
        self.imp().swipe_remainder.set(swipe_remainder);
        self.real_spin(step - self.imp().swipe_remainder.get());
    }

    fn on_insert_text(&self, entry: &gtk::Text, new_text: &str, position: &mut i32) {
        entry.stop_signal_emission_by_name("insert-text");

        if self.numeric() {
            let mut dotpos = -1;
            let entry_text = entry.text();
            let entry_len = entry_text.len() as i32;
            let decimal_point = locale_decimal_point(None);

            let mut sign = entry_text
                .chars()
                .collect::<Vec<_>>()
                .iter()
                .any(|&c| c == '-' || c == '+');

            if sign && *position == 0 {
                return;
            }

            if let Some(v) = entry_text.find(decimal_point) {
                dotpos = v as i32;
            }

            if dotpos > -1
                && *position > dotpos
                && self.digits() - entry_len + dotpos - new_text.len() as i32 + 1 < 0
            {
                return;
            }

            for (i, ch) in new_text.chars().enumerate() {
                let i = i as i32;
                if ch == '-' || ch == '+' {
                    if sign || *position != 0 || i != 0 {
                        return;
                    }
                    sign = true
                } else if ch == decimal_point {
                    if self.digits() == 0
                        || dotpos > -1
                        || (new_text.len() as i32 - 1 - i + entry_len - *position > self.digits())
                    {
                        return;
                    }

                    dotpos = *position + i;
                } else if !ch.is_ascii_digit() {
                    return;
                }
            }
        }

        if let Some(id) = self.imp().insert_text_handler.take() {
            entry.block_signal(&id);
            entry.insert_text(new_text, position);
            entry.unblock_signal(&id);
            self.imp().insert_text_handler.replace(Some(id));
        }
    }
}

fn weed_out_neg_zero(mut s: String, digits: usize) -> String {
    if s.starts_with("-") && s == format!("{:.*}", digits, -0.0f64) {
        s.remove(0);
    }
    s
}

fn current_locale() -> Locale {
    sys_locale::get_locale()
        .and_then(|s| s.parse().ok())
        .unwrap_or(icu::locale::locale!("en"))
}

pub fn locale_decimal_point(locale: Option<Locale>) -> char {
    let locale = locale.unwrap_or_else(current_locale);
    let formatter = DecimalFormatter::try_new(locale.clone().into(), Default::default())
        .unwrap_or_else(|_| {
            DecimalFormatter::try_new(icu::locale::locale!("en").into(), Default::default())
                .expect("en locale must always be available")
        });

    let mut decimal = Decimal::from(1);
    decimal.multiply_pow10(-1); // 0.1, forces a separator into the output

    let formatted = formatter.format(&decimal).to_string();
    formatted
        .chars()
        .find(|c| !c.is_ascii_digit())
        .unwrap_or('.')
}
