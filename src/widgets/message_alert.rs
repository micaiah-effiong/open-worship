use gtk::glib;
use gtk::glib::object::ObjectExt;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::{AdjustmentExt, BoxExt, WidgetExt, WidgetExtManual};
use gtk::{self, Label};

use crate::utils::WidgetChildrenExt;

mod signals {
    pub(super) const REQUEST_NEXT_MESSAGE: &str = "request-next-message";
    pub(super) const STOP_MARQUEE_REQUEST: &str = "stop-marquee-request";
}

mod imp {
    use std::{
        cell::{Cell, RefCell},
        sync::OnceLock,
    };

    use gtk::{
        glib::{
            self, Properties,
            subclass::{
                Signal,
                object::{ObjectImpl, ObjectImplExt},
                types::ObjectSubclass,
            },
        },
        prelude::ObjectExt,
        subclass::{
            box_::BoxImpl,
            prelude::DerivedObjectProperties,
            widget::{
                CompositeTemplateClass, CompositeTemplateInitializingExt, WidgetClassExt,
                WidgetImpl,
            },
        },
    };

    use crate::{
        utils::{self, WidgetChildrenExt},
        widgets::message_alert::signals,
    };

    #[derive(Debug, Default, Properties, gtk::CompositeTemplate)]
    #[template(resource = "/com/openworship/app/ui/message_alert.ui")]
    #[properties(wrapper_type=super::MessageAlert)]
    pub struct MessageAlert {
        #[template_child]
        pub scrolled: gtk::TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub scrolled_box: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) label_box: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) stack: gtk::TemplateChild<gtk::Stack>,
        #[template_child]
        pub(super) front_spacer: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) back_spacer: gtk::TemplateChild<gtk::Box>,

        #[property(get, set=Self::set_font_scale_)]
        pub font_scale: Cell<f32>,

        pub(super) tick_id: RefCell<Option<gtk::TickCallbackId>>,
        pub(super) init_indentation: Cell<bool>,
        pub(super) request_next_message: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageAlert {
        const NAME: &'static str = "MessageAlert";
        type Type = super::MessageAlert;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MessageAlert {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder(signals::REQUEST_NEXT_MESSAGE).build(),
                    Signal::builder(signals::STOP_MARQUEE_REQUEST).build(),
                ]
            })
        }
    }
    impl WidgetImpl for MessageAlert {}
    impl BoxImpl for MessageAlert {}

    impl MessageAlert {
        fn set_font_scale_(&self, value: f32) {
            if value == self.font_scale.get() {
                return;
            };

            self.font_scale.set(value);
            self.style();
        }
        pub(super) fn style(&self) {
            let value = self.font_scale.get();

            let font_size = 16.0;
            let css = format!(
                ".alert-label-size {{
                    font-size: {}px;
                }}",
                5.3 * value * font_size
            );

            println!("font {value} {}", 5.3 * value * font_size);
            for item in self.scrolled_box.get_children::<gtk::Label>() {
                utils::set_style(&item, &css);
            }
        }
    }

    impl MessageAlert {}
}

glib::wrapper! {
    /// The Marquee
    pub struct MessageAlert(ObjectSubclass<imp::MessageAlert>)
        @extends  gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for MessageAlert {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl MessageAlert {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn add_message(&self, msg: &str) {
        let imp = self.imp();
        let message_label = gtk::Label::new(Some(&format!("{msg}...")));
        message_label.set_css_classes(&["alert-label", "alert-label-size"]);

        imp.scrolled_box.append(&message_label);
        imp.scrolled_box
            .reorder_child_after(&imp.back_spacer.clone(), Some(&message_label));
        imp.request_next_message.set(true);
        imp.style();
    }

    pub fn start_marquee(&self) {
        if self.imp().tick_id.borrow().is_some() {
            return;
        };

        self.imp()
            .stack
            .set_transition_type(gtk::StackTransitionType::SlideUp);
        self.imp().stack.set_visible_child_name("scroll");
        let scrolled = self.imp().scrolled.clone();

        if let Some(scrollbar) = scrolled.get_children::<gtk::Scrollbar>().next() {
            scrollbar.set_visible(false);
        };

        let tick_id = self.add_tick_callback(glib::clone!(
            #[strong(rename_to=sw)]
            scrolled,
            move |obj, _fc| {
                let adj = sw.hadjustment();
                let current = adj.value();
                let upper = adj.upper(); // visible area width
                let page = adj.page_size(); // visible area width
                let diff = upper - page;

                if !obj.imp().init_indentation.get() && sw.width() > 0 {
                    obj.imp().init_indentation.set(true);
                    obj.imp().front_spacer.set_width_request(sw.width() + 100);
                    obj.imp().back_spacer.set_width_request(sw.width() + 100);
                }

                if diff <= 0.0 {
                    return glib::ControlFlow::Continue;
                }

                let speed = 1.0;
                let next_adj_value = current + speed;

                if next_adj_value >= diff {
                    // adj.set_value(0.0);
                    // this should trigger to hide/stop_marquee
                    obj.emit_stop_marquee_request();
                    return glib::ControlFlow::Break;
                } else {
                    adj.set_value(next_adj_value);
                }

                if obj.imp().request_next_message.get()
                    && let Some(point) =
                        // obj.imp().back_spacer.translate_coordinates(&sw, 0.0, 0.0)
                        obj
                            .imp()
                            .back_spacer
                            .compute_point(&sw, &gtk::graphene::Point::new(0.0, 0.0))
                {
                    let space = point.x() - sw.width() as f32;

                    let threshold = 100.0;
                    if space < threshold {
                        obj.imp().request_next_message.set(false);
                        obj.emit_request_next_message();
                    }
                }

                glib::ControlFlow::Continue
            }
        ));

        self.imp().tick_id.replace(Some(tick_id));
    }

    pub fn stop_marquee(&self) {
        let tick_id = match self.imp().tick_id.take() {
            Some(id) => id,
            None => return,
        };
        let imp = self.imp();

        self.imp()
            .stack
            .set_transition_type(gtk::StackTransitionType::SlideDown);
        imp.stack.set_visible_child_name("label");
        let scrolled_box = imp.scrolled_box.clone();
        // NOTE: wait for transition to finish
        glib::timeout_add_local_once(std::time::Duration::from_secs(1), move || {
            scrolled_box
                .get_children::<Label>()
                .for_each(|v| v.unparent());
        });
        tick_id.remove();
    }

    fn emit_request_next_message(&self) {
        self.emit_by_name::<()>(signals::REQUEST_NEXT_MESSAGE, &[]);
    }
    fn emit_stop_marquee_request(&self) {
        self.emit_by_name::<()>(signals::STOP_MARQUEE_REQUEST, &[]);
    }

    pub fn connect_request_next_message<F: Fn(&Self) + 'static>(&self, f: F) {
        self.connect_closure(
            signals::REQUEST_NEXT_MESSAGE,
            false,
            glib::closure_local!(move |obj: &Self| {
                f(obj);
            }),
        );
    }
    pub fn connect_stop_marquee_request<F: Fn(&Self) + 'static>(&self, f: F) {
        self.connect_closure(
            signals::STOP_MARQUEE_REQUEST,
            false,
            glib::closure_local!(move |obj: &Self| {
                f(obj);
            }),
        );
    }
}
