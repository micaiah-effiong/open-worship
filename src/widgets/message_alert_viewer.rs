use gtk;
use gtk::gio;
use gtk::glib;
use gtk::glib::object::CastNone;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::ToggleButtonExt;

use crate::services::alert::Alert;

mod imp {

    use gtk::{
        gio,
        glib::{
            self, Properties,
            object::CastNone,
            subclass::{
                object::{ObjectImpl, ObjectImplExt},
                types::ObjectSubclass,
            },
            types::StaticTypeExt,
        },
        prelude::{ButtonExt, ToggleButtonExt},
        subclass::{
            box_::BoxImpl,
            prelude::DerivedObjectProperties,
            widget::{
                CompositeTemplateCallbacksClass, CompositeTemplateClass,
                CompositeTemplateInitializingExt, WidgetClassExt, WidgetImpl,
            },
        },
    };

    use crate::services::{alert::Alert, message_alert_manager::MessageAlertManager};

    #[derive(Debug, Default, Properties, gtk::CompositeTemplate)]
    #[template(resource = "/com/openworship/app/ui/message_alert_viewer.ui")]
    #[properties(wrapper_type=super::MessageAlertViewer)]
    pub struct MessageAlertViewer {
        #[template_child]
        pub(super) column_view: gtk::TemplateChild<gtk::ColumnView>,
        pub alert_manager: glib::WeakRef<MessageAlertManager>,

        #[template_child]
        pub(super) toggle_alert_btn: gtk::TemplateChild<gtk::ToggleButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageAlertViewer {
        const NAME: &'static str = "MessageAlertViewer";
        type Type = super::MessageAlertViewer;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Alert::ensure_type();
            klass.bind_template();
            klass.bind_template_callbacks();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MessageAlertViewer {
        fn constructed(&self) {
            self.parent_constructed();
            let column_view = self.column_view.clone();
            Self::build_column_view(&column_view);
        }
    }
    impl WidgetImpl for MessageAlertViewer {}
    impl BoxImpl for MessageAlertViewer {}

    #[gtk::template_callbacks(functions)]
    impl MessageAlertViewer {
        #[template_callback(function = false)]
        fn handle_toggle_alert(&self, btn: &gtk::ToggleButton) {
            let label = match btn.is_active() {
                true => "Hide",
                false => "Show",
            };
            btn.set_label(label);

            if let Some(manager) = self.alert_manager.upgrade() {
                if btn.is_active() {
                    manager.show();
                } else {
                    manager.hide();
                }
            }
        }

        #[template_callback(function = false)]
        fn handle_inc_count(&self, _: &gtk::Button) {
            let Some(model) = self
                .column_view
                .model()
                .and_downcast::<gtk::SingleSelection>()
            else {
                return;
            };

            let Some(obj) = model.selected_item().and_downcast::<Alert>() else {
                return;
            };

            obj.set_count(obj.count().saturating_add(1));
        }

        #[template_callback(function = false)]
        fn handle_dec_count(&self, _: &gtk::Button) {
            let Some(model) = self
                .column_view
                .model()
                .and_downcast::<gtk::SingleSelection>()
            else {
                return;
            };

            let Some(obj) = model.selected_item().and_downcast::<Alert>() else {
                return;
            };

            obj.set_count(obj.count().saturating_sub(1));
        }

        #[template_callback(function = false)]
        fn handle_open_editor_window(&self, _: &gtk::Button) {
            if let Some(manager) = self.alert_manager.upgrade() {
                manager.open_editor();
            }
        }

        #[template_callback]
        fn get_active_icon(#[rest] values: &[glib::Value]) -> String {
            let active = values[0].get::<bool>().expect("Should be a bool");
            let icon_name = if active { "plus" } else { "minus" };

            icon_name.into()
        }
    }
    impl MessageAlertViewer {
        fn build_column_view(column_view: &gtk::ColumnView) {
            let store = gio::ListStore::new::<Alert>();
            let Some(selection_model) = column_view.model().and_downcast::<gtk::SingleSelection>()
            else {
                return;
            };
            selection_model.set_model(Some(&store));
        }
    }
}

glib::wrapper! {
    pub struct MessageAlertViewer(ObjectSubclass<imp::MessageAlertViewer>)
    @extends  gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for MessageAlertViewer {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl MessageAlertViewer {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn add_alert(&self, alert: &Alert) {
        let Some(model) = self
            .imp()
            .column_view
            .model()
            .and_downcast_ref::<gtk::SingleSelection>()
            .cloned()
        else {
            return;
        };

        let Some(store) = model.model().and_downcast::<gio::ListStore>() else {
            return;
        };

        store.append(alert);
    }

    pub fn remove_alert(&self, position: u32) {
        let Some(model) = self
            .imp()
            .column_view
            .model()
            .and_downcast_ref::<gtk::SingleSelection>()
            .cloned()
        else {
            return;
        };

        let Some(store) = model.model().and_downcast::<gio::ListStore>() else {
            return;
        };

        store.remove(position);
    }

    pub fn set_alert_active(&self, active: bool) {
        self.imp().toggle_alert_btn.set_active(active);
    }
}
