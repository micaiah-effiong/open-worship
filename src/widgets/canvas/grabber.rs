use gtk::glib::object::{CastNone, ObjectExt};
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::{ButtonExt, EventControllerExt, GestureExt, GestureSingleExt, WidgetExt};
use gtk::{EventControllerMotion, GestureClick};
use gtk::{gdk, glib};

mod signals {
    pub(super) const GRABBED: &str = "grabbed";
    pub(super) const GRABBED_STOPPED: &str = "grabbed-stopped";
    pub(super) const GRABBED_MOTION: &str = "grabbed-motion";
}
mod g_ctl {
    pub(super) const CLICK: &str = "g-click";
    pub(super) const MOTION: &str = "g-motion";
}

/// AspectRatio equivalent
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "GrabberMode")]
pub enum GrabberMode {
    #[default]
    Drag,
    Move,
}

mod imp {

    use std::cell::{Cell, RefCell};
    use std::sync::OnceLock;

    use gtk::glib::subclass::Signal;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use super::*;

    #[derive(Default)]
    // #[properties(wrapper_type = super::Grabber)]
    pub struct Grabber {
        pub(super) core_id: Cell<u32>,
        pub id: Cell<u32>,
        pub mode: RefCell<GrabberMode>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Grabber {
        const NAME: &'static str = "Grabber";
        type Type = super::Grabber;
        type ParentType = gtk::Button;
    }

    impl WidgetImpl for Grabber {}

    // #[glib::derived_properties]
    impl ObjectImpl for Grabber {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder(signals::GRABBED)
                        .param_types([
                            gtk::GestureClick::static_type(),
                            u32::static_type(),
                            f64::static_type(),
                            f64::static_type(),
                        ])
                        .build(),
                    Signal::builder(signals::GRABBED_STOPPED)
                        .param_types([
                            gtk::GestureClick::static_type(),
                            u32::static_type(),
                            f64::static_type(),
                            f64::static_type(),
                        ])
                        .build(),
                    Signal::builder(signals::GRABBED_MOTION)
                        .param_types([
                            gtk::EventControllerMotion::static_type(),
                            u32::static_type(),
                            f64::static_type(),
                            f64::static_type(),
                        ])
                        .build(),
                ]
            })
        }
    }

    impl ButtonImpl for Grabber {}
}

glib::wrapper! {
    pub struct Grabber(ObjectSubclass<imp::Grabber>)
        @extends gtk::Button, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable,  gtk::Buildable, gtk::ConstraintTarget;
}

impl Grabber {
    pub fn new(id: u32) -> Self {
        let g = glib::Object::new::<Grabber>();
        g.imp().id.set(id);
        g.imp().core_id.set(id);

        let g_click = gtk::GestureClick::builder().name(g_ctl::CLICK).build();
        g_click.set_propagation_phase(gtk::PropagationPhase::Bubble);
        {
            g_click.set_button(gdk::BUTTON_PRIMARY);
            g_click.connect_pressed(glib::clone!(
                #[weak]
                g,
                move |gesture, _n, x, y| {
                    let id = g.imp().id.get();
                    g.emit_by_name::<()>(signals::GRABBED, &[gesture, &id, &x, &y]);
                    gesture.set_state(gtk::EventSequenceState::Claimed);
                }
            ));

            g_click.set_button(gdk::BUTTON_PRIMARY);
            g_click.connect_released(glib::clone!(
                #[weak]
                g,
                move |gesture, _n, x, y| {
                    let id = g.imp().id.get();
                    g.emit_by_name::<()>(signals::GRABBED_STOPPED, &[gesture, &id, &x, &y]);
                    gesture.set_state(gtk::EventSequenceState::Claimed);
                }
            ));

            g.add_controller(g_click);
        }

        {
            let motion = gtk::EventControllerMotion::new();
            motion.set_propagation_phase(gtk::PropagationPhase::Bubble);
            motion.set_name(Some(g_ctl::MOTION));
            motion.connect_motion(glib::clone!(
                #[weak]
                g,
                move |motion, x, y| {
                    let id = g.imp().id.get();
                    g.emit_by_name::<()>(signals::GRABBED_MOTION, &[motion, &id, &x, &y]);
                }
            ));

            g.add_controller(motion);
        }

        //->

        g.remove_css_class("button");
        g.add_css_class("ow-grabber");
        g.set_icon_name("drag-symbolic");
        g.set_cursor_from_id(id as u32);

        if let Some(image) = g.child().and_downcast_ref::<gtk::Image>() {
            image.set_pixel_size(13);
        }

        g
    }

    pub fn connect_grabbed<F: Fn(&GestureClick, u32, f64, f64) + 'static>(&self, f: F) {
        self.connect_closure(
            signals::GRABBED,
            false,
            glib::closure_local!(
                move |_g: Grabber, evt: GestureClick, id: u32, x: f64, y: f64| {
                    f(&evt, id, x, y);
                }
            ),
        );
    }

    pub fn connect_grabbed_stopped<F: Fn(&GestureClick, u32, f64, f64) + 'static>(&self, f: F) {
        self.connect_closure(
            signals::GRABBED_STOPPED,
            false,
            glib::closure_local!(
                move |_g: Grabber, evt: GestureClick, id: u32, x: f64, y: f64| {
                    f(&evt, id, x, y);
                }
            ),
        );
    }

    pub fn connect_grabbed_motion<F: Fn(&EventControllerMotion, u32, f64, f64) + 'static>(
        &self,
        f: F,
    ) {
        self.connect_closure(
            signals::GRABBED_MOTION,
            false,
            glib::closure_local!(move |_g: Grabber,
                                       evt: EventControllerMotion,
                                       id: u32,
                                       x: f64,
                                       y: f64| {
                f(&evt, id, x, y);
            }),
        );
    }

    pub fn switch_mode(&self) {
        let mode = self.imp().mode.borrow().clone();
        let id = match mode {
            GrabberMode::Drag => {
                self.imp().mode.replace(GrabberMode::Move);
                self.set_icon_name("drag-square-symbolic");
                0
            }
            GrabberMode::Move => {
                self.imp().mode.replace(GrabberMode::Drag);
                self.set_icon_name("drag-symbolic");
                self.imp().core_id.get()
            }
        };

        self.imp().id.set(id);
        self.set_cursor_from_id(id);
        println!(
            "mode switched {:?} {}",
            self.imp().mode.borrow(),
            self.imp().id.get()
        )
    }

    fn set_cursor_from_id(&self, holding_id: u32) {
        let cursor_name = match holding_id {
            0 => "move",
            1 => "nw-resize",
            2 => "n-resize",
            3 => "ne-resize",
            4 => "e-resize",
            5 => "se-resize",
            6 => "s-resize",
            7 => "sw-resize",
            8 => "w-resize",
            9_u32..=u32::MAX => "default",
        };

        if let Some(c) = self.first_child().and_downcast_ref::<gtk::Image>() {
            c.set_cursor_from_name(Some(cursor_name));
        }
    }
}
