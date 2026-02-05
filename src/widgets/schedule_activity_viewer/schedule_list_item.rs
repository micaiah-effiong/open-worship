use gtk::{
    glib::{self},
    prelude::*,
};

pub mod imp {
    use gtk::{
        glib::{
            Properties,
            subclass::{
                object::{ObjectImpl, ObjectImplExt},
                types::ObjectSubclass,
            },
        },
        subclass::{
            box_::BoxImpl,
            prelude::WidgetClassExt,
            widget::{CompositeTemplateClass, CompositeTemplateInitializingExt, WidgetImpl},
        },
    };

    use gtk::subclass::prelude::DerivedObjectProperties;

    use super::*;

    #[derive(Default, Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type=super::ScheduleListItem)]
    #[template(resource = "/com/openworship/app/ui/schedule_listitem.ui")]
    pub struct ScheduleListItem {
        #[template_child]
        #[property(get)]
        pub label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub preview_box: gtk::TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScheduleListItem {
        const NAME: &'static str = "ScheduleListItem";
        type Type = super::ScheduleListItem;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for ScheduleListItem {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for ScheduleListItem {}
    impl BoxImpl for ScheduleListItem {}
}

glib::wrapper! {
    pub struct ScheduleListItem(ObjectSubclass<imp::ScheduleListItem>)
    @extends  gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for ScheduleListItem {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl ScheduleListItem {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
