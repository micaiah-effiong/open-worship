use gtk::{
    glib::{self, object::ObjectExt, subclass::types::ObjectSubclassIsExt},
    prelude::{TextViewExt, WidgetExt},
};

use crate::{services::slide::Slide, widgets::editor::EditorType};

/// This macro takes `imp` and `action_name`, and returns a tuple
/// 0 - gio::SimpleAction
/// 1 - gio::MenuItem
/// `imp` is [`EditorListItem`](imp::EditorListItem)
macro_rules! make_menu_item {
    ($imp:expr, $action_name: expr) => {{
        let action = gio::SimpleAction::new($action_name, None);
        let label = action.name();
        let cap = capitalize_first(&label);
        let menu_item =
            gio::MenuItem::new(Some(&cap), Some(&format!("slide.{}", label.to_lowercase())));

        action.connect_activate(glib::clone!(
            #[weak(rename_to=imp)]
            $imp,
            move |_, _| {
                if let Some(slide) = imp.slide.borrow().clone() {
                    slide.set_tag(cap.clone());
                }
            }
        ));

        (action, menu_item)
    }};
}

mod imp {
    use super::*;
    use std::{cell::RefCell, collections::HashMap};

    use gtk::{
        gio::{
            self,
            prelude::{ActionExt, ActionMapExt},
        },
        glib::{
            self,
            subclass::{
                object::{ObjectImpl, ObjectImplExt},
                types::{ObjectSubclass, ObjectSubclassExt},
            },
        },
        prelude::{BoxExt, OrientableExt, TextViewExt, WidgetExt},
        subclass::{box_::BoxImpl, widget::WidgetImpl},
    };

    #[derive(Default)]
    pub struct EditorListItem {
        pub(super) slide: RefCell<Option<Slide>>,
        pub(super) textview: RefCell<gtk::TextView>,
        pub(super) tag_label: RefCell<gtk::Label>,
        pub(super) binding: RefCell<Vec<glib::Binding>>,
        pub(super) signals: RefCell<HashMap<glib::Object, glib::SignalHandlerId>>,
        pub(super) editor_type: RefCell<EditorType>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EditorListItem {
        const NAME: &'static str = "EditorListItem";
        type Type = super::EditorListItem;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for EditorListItem {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj().clone();
            obj.set_vexpand(true);
            obj.set_hexpand(true);
            obj.set_orientation(gtk::Orientation::Vertical);

            let tv = self.textview.borrow().clone();
            tv.set_margin_start(8);
            tv.set_left_margin(6);
            tv.set_right_margin(6);
            tv.set_top_margin(6);
            tv.set_bottom_margin(6);
            tv.set_wrap_mode(gtk::WrapMode::Word);
            tv.set_vexpand(true);
            tv.set_hexpand(true);

            let tag_label = self.tag_label.borrow().clone();
            tag_label.set_visible(false);
            tag_label.set_css_classes(&["slide-tag"]);
            tag_label.set_xalign(0.0);

            obj.append(&tag_label);
            obj.append(&tv);
        }
    }
    impl WidgetImpl for EditorListItem {}
    impl BoxImpl for EditorListItem {}

    impl EditorListItem {
        pub(super) fn register_context_menu(&self) {
            if self.editor_type.borrow().clone() != EditorType::Song {
                return;
            };

            let tv = self.textview.borrow().clone();

            let main_menu = gio::Menu::new();
            tv.set_extra_menu(Some(&main_menu));

            let actions = gio::SimpleActionGroup::new();
            let menu = gio::Menu::new();
            let tag_menu = gio::Menu::new();
            main_menu.append_section(None, &menu);

            let items = [
                "intro",
                "verse",
                "pre-chorus",
                "chorus",
                "solo",
                "bridge",
                "middle",
                "other",
                "ending",
            ];

            for item in items {
                let (action, menu_item) = make_menu_item!(self, item);
                tag_menu.append_item(&menu_item);
                actions.add_action(&action);
            }

            menu.append_submenu(Some("Add tag"), &tag_menu);
            tv.insert_action_group("slide", Some(&actions));
        }
    }
}

glib::wrapper! {
pub struct EditorListItem(ObjectSubclass<imp::EditorListItem>)
    @extends gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,gtk::Orientable;
}

impl EditorListItem {
    pub fn new(r#type: EditorType) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().editor_type.replace(r#type);

        // NOTE: this fn uses EditorType and calling is in constructed()
        // will no work because constructed() executes before I can set editor_type
        // (inside glib::Object::new()) which will result in a default value
        obj.imp().register_context_menu();

        obj
    }

    pub fn bind(&self, slide: Slide) {
        let mut binding = Vec::new();

        let tv = self.imp().textview.borrow().clone();
        let tag_label = self.imp().tag_label.borrow().clone();

        let tag_binding = slide
            .bind_property("tag", &self.imp().tag_label.borrow().clone(), "label")
            .sync_create()
            .build();
        binding.push(tag_binding);

        if let Some(buf) = slide.entry_buffer() {
            tv.set_buffer(Some(&buf));
            glib::idle_add_local_once(glib::clone!(
                #[weak]
                tv,
                move || tv.queue_resize()
            ));
        }

        let t_visible_binding = tag_label
            .bind_property("label", &tag_label, "visible")
            .sync_create()
            .transform_to(|_, label: String| Some(!label.is_empty()))
            .build();
        binding.push(t_visible_binding);
        self.imp().binding.borrow_mut().extend(binding);

        let signal = slide.connect_visible_notify(glib::clone!(
            #[weak(rename_to=imp)]
            self,
            move |slide| {
                if let Some(parent) = imp.parent() {
                    parent.set_visible(slide.visible())
                }
            }
        ));
        self.imp()
            .signals
            .borrow_mut()
            .insert(slide.clone().into(), signal);

        tag_label.set_label(&slide.tag());
        self.imp().slide.replace(Some(slide));
    }

    pub fn unbind(&self) {
        let mut binding = self.imp().binding.borrow_mut();
        let mut signals = self.imp().signals.borrow_mut();

        for item in binding.drain(..) {
            item.unbind();
        }

        for (obj, signal) in signals.drain() {
            obj.disconnect(signal);
        }
    }

    pub fn textview(&self) -> gtk::TextView {
        self.imp().textview.borrow().clone()
    }
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
