use gtk::glib::{self, subclass::types::ObjectSubclassIsExt};

use crate::dto::SongObject;

mod imp {
    use std::cell::RefCell;

    use gtk::{
        glib::{
            self,
            subclass::{
                object::{ObjectImpl, ObjectImplExt},
                types::{ObjectSubclass, ObjectSubclassExt},
            },
            value::ToValue,
        },
        prelude::{BoxExt, WidgetExt},
        subclass::{box_::BoxImpl, widget::WidgetImpl},
    };

    use crate::{dto::SongObject, widgets::canvas::serialise::SlideManagerData};

    #[derive(Default, Debug)]
    pub struct SongListItem {
        pub text: RefCell<gtk::Label>,
        pub(super) data: RefCell<SongObject>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SongListItem {
        const NAME: &'static str = "SongListItem";
        type Type = super::SongListItem;
        type ParentType = gtk::Box;
    }
    impl ObjectImpl for SongListItem {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            let label = gtk::Label::builder()
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .build();
            self.text.replace(label.clone());

            {
                let drag_source = gtk::DragSource::new();
                drag_source.set_actions(gtk::gdk::DragAction::COPY);
                drag_source.connect_prepare(glib::clone!(
                    #[weak(rename_to=imp)]
                    self,
                    #[upgrade_or]
                    None,
                    move |_, _, _| {
                        let li = imp.data.borrow().clone();
                        let sm_data: SlideManagerData = li.into();
                        let content = gtk::gdk::ContentProvider::for_value(&sm_data.to_value());

                        Some(content)
                    }
                ));

                // drag_source.connect_drag_begin(move |_, _| {
                // let item_text = item_text.to_string();
                // drag.set_icon_name(Some("document-properties"), 0, 0);
                // });

                obj.add_controller(drag_source);
            }

            obj.append(&label);
        }
    }
    impl WidgetImpl for SongListItem {}
    impl BoxImpl for SongListItem {}
}

glib::wrapper! {
    pub struct SongListItem(ObjectSubclass<imp::SongListItem>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for SongListItem {
    fn default() -> Self {
        glib::Object::new::<Self>()
    }
}

impl SongListItem {
    pub fn load_data(&self, song: SongObject) {
        self.imp().data.replace(song.clone());
        self.imp()
            .text
            .borrow()
            .set_label(&song.title().to_string());
    }
}
