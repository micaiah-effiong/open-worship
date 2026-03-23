use gtk;
use gtk::glib;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::{EditableExt, TextBufferExt, TextViewExt};

use crate::services::alert::Alert;

mod imp {

    use std::cell::RefCell;

    use gtk::{
        glib::{
            self, Properties,
            object::CastNone,
            subclass::{
                object::{ObjectImpl, ObjectImplExt},
                types::{ObjectSubclass, ObjectSubclassExt},
            },
            types::StaticType,
        },
        prelude::{
            AccessibleExtManual, EditableExt, ObjectExt, TextBufferExt, TextViewExt, WidgetExt,
        },
        subclass::{
            box_::BoxImpl,
            prelude::DerivedObjectProperties,
            widget::{
                CompositeTemplateCallbacksClass, CompositeTemplateClass,
                CompositeTemplateInitializingExt, WidgetClassExt, WidgetImpl,
            },
        },
    };

    use crate::{
        services::{alert::Alert, settings::ApplicationSettings},
        utils::TextBufferExtraExt,
        widgets::message_alert_editor_window::MessageAlertEditorWindow,
    };

    #[derive(Debug, Default, Properties, gtk::CompositeTemplate)]
    #[template(resource = "/com/openworship/app/ui/message_alert_editor.ui")]
    #[properties(wrapper_type=super::MessageAlertEditor)]
    pub struct MessageAlertEditor {
        #[template_child]
        pub(super) alert_name: gtk::TemplateChild<gtk::Entry>,
        #[template_child]
        pub(super) alert_message: gtk::TemplateChild<gtk::TextView>,

        #[property(get=Self::message_name_)]
        pub message_name: RefCell<String>,
        #[property(get=Self::message_)]
        pub message: RefCell<String>,
        #[property(get, set)]
        pub alert: RefCell<Alert>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageAlertEditor {
        const NAME: &'static str = "MessageAlertEditor";
        type Type = super::MessageAlertEditor;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MessageAlertEditor {
        fn constructed(&self) {
            self.parent_constructed();
            let settings = ApplicationSettings::get_instance();
            self.obj().alert().set_count(settings.alert_count());

            self.alert_message.buffer().connect_changed(glib::clone!(
                #[weak(rename_to=imp)]
                self,
                move |buf| {
                    imp.obj().alert().set_message(buf.full_text().to_string());
                }
            ));

            self.alert_name.connect_changed(glib::clone!(
                #[weak(rename_to=imp)]
                self,
                move |buf| {
                    imp.obj().alert().set_name(buf.text().to_string());
                }
            ));
        }
    }
    impl WidgetImpl for MessageAlertEditor {}
    impl BoxImpl for MessageAlertEditor {}

    #[gtk::template_callbacks]
    impl MessageAlertEditor {
        #[template_callback]
        fn handler_use_alert(&self, _: &gtk::Button) {
            let obj = self.obj();
            let mut valid = true;

            if obj.message_name().is_empty() {
                self.alert_name.add_css_class("error");
                valid = false
            } else {
                self.alert_name.remove_css_class("error");
            }
            if obj.message().is_empty() {
                self.alert_message.add_css_class("error");
                valid = false
            } else {
                self.alert_message.remove_css_class("error");
            }

            if !valid {
                return;
            }

            if let Some(win) = self
                .obj()
                .ancestor(MessageAlertEditorWindow::static_type())
                .and_downcast_ref::<MessageAlertEditorWindow>()
            {
                win.emit_use_alert(&self.obj().alert());
            }
        }
    }

    impl MessageAlertEditor {
        fn message_(&self) -> String {
            self.obj().alert().message()
        }
        fn message_name_(&self) -> String {
            self.obj().alert().name()
        }
    }
}

glib::wrapper! {
    pub struct MessageAlertEditor(ObjectSubclass<imp::MessageAlertEditor>)
    @extends  gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl MessageAlertEditor {
    pub fn new() -> Self {
        glib::Object::new()
    }
    pub fn from_alert(alert: &Alert) -> Self {
        let obj: Self = glib::Object::new();

        obj.imp().alert_message.buffer().set_text(&alert.message());
        obj.imp().alert_name.set_text(&alert.name());
        obj.set_alert(alert);

        obj
    }
}
