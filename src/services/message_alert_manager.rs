use gtk::gio;
use gtk::gio::prelude::ListModelExt;
use gtk::glib;
use gtk::glib::object::CastNone;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::{GtkWindowExt, WidgetExt};
use gtk::{self};

use crate::{
    services::alert::Alert,
    widgets::{message_alert::MessageAlert, message_alert_viewer::MessageAlertViewer},
};

mod imp {
    use std::cell::RefCell;

    use gtk::prelude::ObjectExt;
    use gtk::{
        glib::{
            self, Properties,
            subclass::{
                object::{ObjectImpl, ObjectImplExt},
                prelude::DerivedObjectProperties,
                types::{ObjectSubclass, ObjectSubclassExt},
            },
        },
        prelude::WidgetExt,
    };

    use crate::widgets::message_alert_editor_window::MessageAlertEditorWindow;

    use super::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type=super::MessageAlertManager)]
    pub struct MessageAlertManager {
        #[property(get)]
        marquee: RefCell<MessageAlert>,
        #[property(get)]
        alerts: RefCell<gtk::SingleSelection>,
        #[property(get)]
        viewer: RefCell<MessageAlertViewer>,
        #[property(get)]
        editor: RefCell<MessageAlertEditorWindow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageAlertManager {
        const NAME: &'static str = "MessageAlertManager";
        type Type = super::MessageAlertManager;
    }

    #[glib::derived_properties]
    impl ObjectImpl for MessageAlertManager {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            let store = gio::ListStore::new::<Alert>();
            obj.alerts().set_model(Some(&store));
            obj.viewer().imp().alert_manager.set(Some(&obj));

            obj.editor().connect_close_request(|win| {
                win.set_visible(false);
                glib::Propagation::Proceed
            });

            obj.marquee().connect_request_next_message(glib::clone!(
                #[strong]
                obj,
                move |_marquee| {
                    obj.request_next_message();
                }
            ));
            obj.marquee().connect_stop_marquee_request(glib::clone!(
                #[strong]
                obj,
                move |_marquee| {
                    obj.stop();
                }
            ));

            obj.editor().connect_use_alert(glib::clone!(
                #[strong]
                obj,
                move |_, alert| {
                    obj.add_alert(alert.clone());
                }
            ));
        }
    }
}

glib::wrapper! {
    pub struct MessageAlertManager(ObjectSubclass<imp::MessageAlertManager>) ;
}

impl Default for MessageAlertManager {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl MessageAlertManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_alert(&self, alert: Alert) {
        let alerts = self.alerts();
        let Some(model) = alerts.model().and_downcast::<gio::ListStore>() else {
            return;
        };
        model.append(&alert);
        self.viewer().add_alert(&alert);
    }
    pub fn remove_alert(&self, postion: u32) {
        let alerts = self.alerts();
        let Some(model) = alerts.model().and_downcast::<gio::ListStore>() else {
            return;
        };
        model.remove(postion);
        self.viewer().remove_alert(postion);
    }

    pub fn request_next_message(&self) {
        let alerts = self.alerts();
        let Some(model) = alerts.model().and_downcast::<gio::ListStore>() else {
            return;
        };

        if alerts.n_items() == 0 {
            println!("empty");
            alerts.set_selected(gtk::INVALID_LIST_POSITION);
            return;
        };

        let current = alerts.selected();

        let Some(current_item) = alerts.selected_item().and_downcast::<Alert>() else {
            return;
        };

        let mut next = (current + 1) % alerts.n_items();

        let next_alert = loop {
            let Some(next_alert) = alerts.item(next).and_downcast::<Alert>() else {
                return;
            };
            if next_alert.count() > 0 {
                break next_alert;
            }

            self.remove_alert(next);
            let len = model.n_items();
            if len == 0 {
                println!("empty");
                alerts.set_selected(gtk::INVALID_LIST_POSITION);
                return;
            }
            next = next % len;
        };

        alerts.set_selected(next);
        current_item.set_active(false);
        next_alert.set_active(true);

        next_alert.set_count(next_alert.count().saturating_sub(1));
        self.marquee().add_message(&next_alert.message());

        if current_item.count() == 0
            && let Some(c_pos) = model.find(&current_item)
        {
            self.remove_alert(c_pos);
        }
    }

    pub fn show(&self) {
        let alerts = self.alerts();

        let alert = loop {
            let Some(alert) = alerts.selected_item().and_downcast::<Alert>() else {
                return;
            };

            if alert.count() > 0 {
                break alert;
            } else {
                self.remove_alert(alerts.selected());
            }
        };

        if alert.count() == 0 {
            self.remove_alert(alerts.selected());
            return;
        }

        alert.set_active(true);
        self.marquee().add_message(&alert.message());
        self.marquee().start_marquee();

        alert.set_count(alert.count().saturating_sub(1));
        // if alert.count() == 0 {
        //     self.remove_alert(alerts.selected());
        // }
    }
    pub fn hide(&self) {
        self.marquee().stop_marquee();
    }

    /// handles [STOP_MARQUEE_REQUEST](crate::widgets::message_alert::signals)
    pub fn stop(&self) {
        self.viewer().set_alert_active(false);
    }
    pub fn open_editor(&self) {
        self.editor().present();
        self.editor().set_visible(true);
    }
}
