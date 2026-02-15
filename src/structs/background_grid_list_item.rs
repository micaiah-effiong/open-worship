use gtk::{
    self,
    glib::{self, subclass::types::ObjectSubclassIsExt},
};

//
mod imp {
    use gtk::{
        glib::{
            self,
            subclass::{object::ObjectImpl, types::ObjectSubclass},
        },
        subclass::{
            box_::BoxImpl,
            widget::{
                CompositeTemplateClass, CompositeTemplateInitializingExt, WidgetClassExt,
                WidgetImpl,
            },
        },
    };

    #[derive(Default, Debug, gtk::CompositeTemplate)]
    #[template(resource = "/com/openworship/app/ui/background_list_item.ui")]
    pub struct BackgroundListItem {
        #[template_child]
        pub(super) bg_picture: gtk::TemplateChild<gtk::Picture>,
        #[template_child]
        pub(super) title_label: gtk::TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BackgroundListItem {
        const NAME: &'static str = "BackgroundListItem";
        type Type = super::BackgroundListItem;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for BackgroundListItem {}
    impl WidgetImpl for BackgroundListItem {}
    impl BoxImpl for BackgroundListItem {}
}

glib::wrapper! {
    pub struct BackgroundListItem(ObjectSubclass<imp::BackgroundListItem>)
    @extends  gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for BackgroundListItem {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl BackgroundListItem {
    pub fn set_label(&self, text: &str) {
        self.imp().title_label.set_label(text);
    }
    pub fn set_picture_src(&self, text: &str) {
        self.imp().bg_picture.set_filename(Some(text));
    }
}
