use gtk;
use gtk::glib;

use crate::services::message_alert_manager::MessageAlertManager;
use crate::services::slide_manager::SlideManager;

mod imp {

    use gtk::{
        glib::{
            self, Properties,
            object::{Cast, CastNone},
            subclass::{
                object::{ObjectImpl, ObjectImplExt},
                types::{ObjectSubclass, ObjectSubclassExt, ObjectSubclassIsExt},
            },
            value::ToValue,
        },
        prelude::{BoxExt, ObjectExt, WidgetExt},
        subclass::{box_::BoxImpl, prelude::DerivedObjectProperties, widget::WidgetImpl},
    };

    use crate::{
        services::{
            message_alert_manager::MessageAlertManager, settings::ApplicationSettings,
            slide_manager::SlideManager,
        },
        utils::WidgetChildrenExt,
        widgets::message_alert::MessageAlert,
    };

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type=super::MessageAlertWapper)]
    pub struct MessageAlertWapper {
        #[property(get, construct_only)]
        pub(super) slide_manager: glib::WeakRef<SlideManager>,
        #[property(get, construct_only)]
        pub(super) alert_manager: glib::WeakRef<MessageAlertManager>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageAlertWapper {
        const NAME: &'static str = "MessageAlertWapper";
        type Type = super::MessageAlertWapper;
        type ParentType = gtk::Box;
    }

    #[glib::derived_properties]
    impl ObjectImpl for MessageAlertWapper {
        fn constructed(&self) {
            self.parent_constructed();

            let Some(alert_manager) = self.alert_manager.upgrade() else {
                return;
            };
            let Some(sm) = self.slide_manager.upgrade() else {
                return;
            };
            let overlay = gtk::Overlay::new();
            overlay.set_child(Some(&sm.slideshow()));

            {
                // its simpler than it looks
                let b = gtk::Box::new(gtk::Orientation::Vertical, 0);
                b.set_widget_name("b");
                let b1 = gtk::Box::builder().name("b1").build();

                let settings = ApplicationSettings::get_instance();
                settings
                    .bind_alert_position(&b1, "vexpand")
                    .mapping(|v, _| {
                        let value: u32 = v.get().expect("The variant needs to be of type `u32`.");
                        Some((value != 0).to_value())
                    })
                    .set_mapping(|_, _| None)
                    .build();
                let b2 = gtk::Box::new(gtk::Orientation::Vertical, 0);
                b2.set_widget_name("b2");
                b2.append(&alert_manager.marquee());
                b.append(&b1);
                b.append(&b2);
                overlay.add_overlay(&b);
            }

            overlay.set_overflow(gtk::Overflow::Hidden);
            overlay.connect_get_child_position({
                let sm = sm.clone();
                move |_v, w| {
                    let Some(message_alert) = w
                        .downcast_ref::<gtk::Box>()
                        .and_then(|b| b.children().nth(1).and_downcast_ref::<gtk::Box>().cloned())
                        .and_then(|v| v.first_child().and_downcast_ref::<MessageAlert>().cloned())
                    else {
                        return None;
                    };

                    let Some(canvas) = sm.current_slide().and_then(|v| v.canvas()) else {
                        return None;
                    };
                    message_alert.set_font_scale(canvas.current_ratio() as f32);

                    let hh = message_alert.imp().scrolled_box.height();
                    message_alert.set_height_request(hh);

                    None
                }
            });

            self.obj().append(&overlay);
            // alert_manager.show();
        }
    }
    impl WidgetImpl for MessageAlertWapper {}
    impl BoxImpl for MessageAlertWapper {}
}

glib::wrapper! {
    pub struct MessageAlertWapper(ObjectSubclass<imp::MessageAlertWapper>)
    @extends  gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for MessageAlertWapper {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl MessageAlertWapper {
    pub fn new(slide_manager: &SlideManager, alert_manager: &MessageAlertManager) -> Self {
        let obj: Self = glib::Object::builder()
            .property("alert_manager", alert_manager)
            .property("slide_manager", slide_manager)
            .build();

        obj
    }
}
