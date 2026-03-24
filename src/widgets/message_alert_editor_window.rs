use gtk::glib;
use gtk::glib::object::ObjectExt;
use gtk::{self};

use crate::services::alert::Alert;
use crate::services::settings::ApplicationSettings;

mod signals {
    pub(super) const USE_ALERT: &str = "use-alert";
}

mod imp {

    use std::sync::OnceLock;

    use gtk::{
        gio::prelude::ListModelExt,
        glib::{
            self, Properties,
            object::{Cast, CastNone},
            subclass::{
                Signal,
                object::{ObjectImpl, ObjectImplExt},
                types::ObjectSubclass,
            },
            types::StaticType,
        },
        prelude::SelectionModelExt,
        subclass::{
            prelude::DerivedObjectProperties,
            widget::{
                CompositeTemplateCallbacksClass, CompositeTemplateClass,
                CompositeTemplateInitializingExt, WidgetClassExt, WidgetImpl,
            },
            window::{WindowImpl, WindowImplExt},
        },
    };

    use crate::{
        db::query,
        services::alert::Alert,
        widgets::{message_alert_editor::MessageAlertEditor, message_alert_editor_window::signals},
    };

    #[derive(Debug, Default, Properties, gtk::CompositeTemplate)]
    #[template(resource = "/com/openworship/app/ui/message_alert_editor_window.ui")]
    #[properties(wrapper_type=super::MessageAlertEditorWindow)]
    pub struct MessageAlertEditorWindow {
        #[template_child]
        stack: gtk::TemplateChild<gtk::Stack>,
        #[template_child]
        sidebar: gtk::TemplateChild<gtk::StackSidebar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageAlertEditorWindow {
        const NAME: &'static str = "MessageAlertEditorWindow";
        type Type = super::MessageAlertEditorWindow;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MessageAlertEditorWindow {
        fn constructed(&self) {
            self.parent_constructed();
            self.sidebar.set_stack(&self.stack.clone());

            match query::Query::get_alerts() {
                Ok(alerts) => {
                    alerts.iter().for_each(|alert| {
                        let editor = MessageAlertEditor::from_alert(alert);
                        self.stack.add_titled(
                            &editor,
                            Some(&editor.message_name()),
                            &Self::truncate(&editor.message_name(), None),
                        );
                        //
                    });
                    //
                }
                Err(e) => eprintln!("Error: {:?}", e),
            };
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();

            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder(signals::USE_ALERT)
                        .param_types([Alert::static_type()])
                        .build(),
                ]
            })
        }
    }
    impl WidgetImpl for MessageAlertEditorWindow {}
    impl WindowImpl for MessageAlertEditorWindow {
        fn close_request(&self) -> glib::Propagation {
            self.parent_close_request();
            println!("close");
            let pages = self.stack.pages();
            let (new, old): (Vec<_>, Vec<_>) = (0..pages.n_items())
                .filter_map(|i| {
                    pages
                        .item(i)
                        .and_downcast::<gtk::StackPage>()
                        .and_then(|v| v.child().downcast::<MessageAlertEditor>().ok())
                })
                .map(|item| item.alert())
                .partition(|v| v.id() == 0);

            let _ = query::Query::insert_alerts(new);
            let _ = query::Query::update_alerts(old);
            // TODO: save to db
            glib::Propagation::Stop
        }
    }

    #[gtk::template_callbacks]
    impl MessageAlertEditorWindow {
        #[template_callback]
        fn handle_add_alert(&self, _: &gtk::Button) {
            let msg = MessageAlertEditor::new();
            let page = self.stack.add_titled(&msg, None, "new");
            self.stack.set_visible_child(&msg);
            msg.alert().connect_name_notify(move |msg| {
                page.set_title(&Self::truncate(&msg.name(), None));
            });
        }

        #[template_callback]
        fn handle_remove_alert(&self, _: &gtk::Button) {
            let Some(message_alert) = self
                .stack
                .visible_child()
                .and_downcast::<MessageAlertEditor>()
            else {
                return;
            };

            let pages = self.stack.pages();

            let Some((_, curr)) = gtk::BitsetIter::init_first(&pages.selection()) else {
                return;
            };

            self.stack.remove(&message_alert);
            let _ = query::Query::delete_alerts(message_alert.alert().id());
            if pages.n_items() == 0 {
                return;
            };

            let next = curr % pages.n_items();
            pages.select_item(next, true);
        }
    }

    impl MessageAlertEditorWindow {
        fn truncate(s: &str, max_chars: Option<u32>) -> String {
            let max_chars = max_chars.unwrap_or(15) as usize;
            if s.chars().count() <= max_chars {
                s.to_string()
            } else {
                s.chars().take(max_chars - 3).collect::<String>() + "..."
            }
        }
    }
}

glib::wrapper! {
    pub struct MessageAlertEditorWindow(ObjectSubclass<imp::MessageAlertEditorWindow>)
    @extends gtk::Window, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,gtk::Native,gtk::Root, gtk::ShortcutManager;
}

impl Default for MessageAlertEditorWindow {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl MessageAlertEditorWindow {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn emit_use_alert(&self, alert: &Alert) {
        let settings = ApplicationSettings::get_instance();
        alert.set_count(settings.alert_count());
        self.emit_by_name::<()>(signals::USE_ALERT, &[alert]);
    }

    pub fn connect_use_alert<F: Fn(&Self, &Alert) + 'static>(&self, f: F) {
        self.connect_closure(
            signals::USE_ALERT,
            false,
            glib::closure_local!(move |obj: &Self, alert: &Alert| f(obj, alert)),
        );
    }
}
