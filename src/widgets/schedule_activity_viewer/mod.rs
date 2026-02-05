mod schedule_list_item;

use gtk::{
    gdk,
    gio::{ActionEntry, MenuItem, SimpleActionGroup},
    glib::{self, clone, subclass::types::ObjectSubclassIsExt},
    prelude::*,
};

use crate::{
    dto::{self, schedule_data},
    utils::{ListViewExtra, WidgetChildrenExt},
    widgets::canvas::serialise::SlideManagerData,
};

mod signals {
    pub const ACTIVATE: &str = "activate";
}

mod imp {
    use std::{cell::RefCell, sync::OnceLock};

    use crate::{utils::WidgetChildrenExt, widgets::canvas::serialise::SlideManagerData};

    use super::*;
    use dto::schedule_data::ScheduleData;
    use gtk::{
        glib::{
            self,
            subclass::{
                Signal,
                object::{ObjectImpl, ObjectImplExt},
                types::{ObjectSubclass, ObjectSubclassExt},
            },
        },
        subclass::{
            box_::BoxImpl,
            prelude::WidgetClassExt,
            widget::{CompositeTemplateClass, CompositeTemplateInitializingExt, WidgetImpl},
        },
    };
    use relm4::RelmWidgetExt;
    use schedule_list_item::ScheduleListItem;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/openworship/app/ui/schedule_activity_viewer.ui")]
    pub struct ScheduleActivityViewer {
        #[template_child]
        pub listview: gtk::TemplateChild<gtk::ListView>,
        #[template_child]
        pub title_label: gtk::TemplateChild<gtk::Label>,
        moved_item: RefCell<Option<ScheduleData>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScheduleActivityViewer {
        const NAME: &'static str = "ScheduleActivityViewer";
        type Type = super::ScheduleActivityViewer;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ScheduleActivityViewer {
        fn constructed(&self) {
            self.parent_constructed();

            self.title_label.set_halign(gtk::Align::Start);

            let factory = gtk::SignalListItemFactory::new();
            let store_model = gtk::gio::ListStore::new::<ScheduleData>();
            let selection_model = gtk::SingleSelection::new(Some(store_model));
            self.listview.set_model(Some(&selection_model));
            self.listview.set_factory(Some(&factory));
            self.listview.set_focus_on_click(true);

            factory.connect_setup(move |_, list_item| {
                let li_widget = ScheduleListItem::new();
                let li = list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Needs to be ListItem");

                li.set_child(Some(&li_widget));
            });

            factory.connect_bind(move |_, list_item| {
                let slide = list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Needs to be ListItem")
                    .item()
                    .and_downcast::<ScheduleData>()
                    .expect("The item has to be an `Slide`.");

                let view = list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Needs to be ListItem")
                    .child()
                    .and_downcast::<ScheduleListItem>()
                    .expect("The child has to be a `Box`.");

                view.label().set_label(&slide.title());
            });

            self.register_activate();
            self.register_context_menu();
            self.register_drop();
            self.register_drag();
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();

            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder(signals::ACTIVATE)
                        .param_types([SlideManagerData::static_type()])
                        .build(),
                ]
            })
        }
    }
    impl WidgetImpl for ScheduleActivityViewer {}
    impl BoxImpl for ScheduleActivityViewer {}

    impl ScheduleActivityViewer {
        fn register_activate(&self) {
            let obj = self.obj();
            let listview = self.listview.clone();

            listview.connect_activate(clone!(
                #[weak]
                obj,
                move |list_view, _| {
                    let Some(model) = list_view.model().and_downcast::<gtk::SingleSelection>()
                    else {
                        return;
                    };

                    let Some(data) = model.selected_item().and_downcast::<ScheduleData>() else {
                        return;
                    };

                    obj.emit_activate(&data.slide_data());
                }
            ));
        }

        fn register_context_menu(&self) {
            let list_view = self.listview.clone();

            let remove_action = ActionEntry::builder("remove_item")
                .activate(clone!(
                    #[strong]
                    list_view,
                    move |_g: &SimpleActionGroup, _sa, _v| {
                        list_view.remove_selected_items();
                    }
                ))
                .build();

            let menu_action_group = SimpleActionGroup::new();
            menu_action_group.add_action_entries([remove_action]);

            let menu = gtk::gio::Menu::new();
            let remove_schedule = MenuItem::new(Some("Remove Item"), Some("schedule.remove_item"));
            menu.insert_item(0, &remove_schedule);

            let popover_menu = gtk::PopoverMenu::from_model(Some(&menu));
            popover_menu.set_has_arrow(false);
            popover_menu.set_position(gtk::PositionType::Bottom);
            popover_menu.set_align(gtk::Align::Start);
            popover_menu.set_parent(&list_view);

            let gesture_click = gtk::GestureClick::new();
            gesture_click.set_button(gtk::gdk::BUTTON_SECONDARY);
            gesture_click.connect_pressed(clone!(
                #[weak]
                popover_menu,
                move |gc, _, x, y| {
                    let rect = gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1);
                    popover_menu.set_pointing_to(Some(&rect));
                    popover_menu.popup();
                    gc.set_state(gtk::EventSequenceState::Claimed);
                }
            ));

            list_view.add_controller(gesture_click);
            list_view.insert_action_group("schedule", Some(&menu_action_group));
        }

        fn register_drag(&self) {
            let listview = self.listview.clone();

            let drag_source = gtk::DragSource::new();
            drag_source.set_actions(gtk::gdk::DragAction::COPY);
            drag_source.connect_prepare(glib::clone!(
                #[weak(rename_to=imp)]
                self,
                #[strong]
                listview,
                #[upgrade_or]
                None,
                move |_, _, _| {
                    let Some(model) = listview.model().and_downcast::<gtk::SingleSelection>()
                    else {
                        return None;
                    };

                    let Some(data) = model.selected_item().and_downcast::<ScheduleData>() else {
                        return None;
                    };

                    let sm_data = data.slide_data();
                    imp.moved_item.replace(Some(data));

                    let content = gtk::gdk::ContentProvider::for_value(&sm_data.to_value());
                    Some(content)
                }
            ));

            drag_source.connect_drag_begin(move |_, _| {
                // let item_text = item_text.to_string();
                // drag.set_icon_name(Some("document-properties"), 0, 0);
            });
            drag_source.connect_drag_end(glib::clone!(
                #[weak(rename_to=imp)]
                self,
                #[strong]
                listview,
                move |_, _, _| {
                    if let Some(item) = imp.moved_item.take() {
                        listview.remove_item(&item);
                    }
                    imp.moved_item.replace(None);
                }
            ));
            drag_source.connect_drag_cancel(glib::clone!(
                #[weak(rename_to=imp)]
                self,
                #[strong]
                listview,
                #[upgrade_or]
                true,
                move |_, _, _| {
                    if let Some(item) = imp.moved_item.take() {
                        listview.remove_item(&item);
                    }
                    imp.moved_item.replace(None);

                    true
                }
            ));

            listview.add_controller(drag_source);
        }

        fn register_drop(&self) {
            let obj = self.obj().clone();

            let drop_target =
                gtk::DropTarget::new(SlideManagerData::static_type(), gdk::DragAction::COPY);

            drop_target.connect_drop(glib::clone!(
                #[weak]
                obj,
                #[upgrade_or]
                false,
                move |_, value, x, y| {
                    let item = match value.get::<SlideManagerData>() {
                        Ok(t) => t,
                        Err(e) => {
                            glib::g_critical!(
                                "schedule_activity_viewer",
                                "Error while dropping => {:?}",
                                e
                            );
                            return false;
                        }
                    };

                    let lv = obj.imp().listview.clone();
                    for item in lv.children() {
                        if item.accessible_role() == gtk::AccessibleRole::ListItem {
                            item.remove_css_class("drop-over");
                        }
                    }

                    let child = obj.imp().listview.pick(x, y, gtk::PickFlags::DEFAULT);
                    let items = obj
                        .imp()
                        .listview
                        .children()
                        .filter_map(|v| v.first_child().and_downcast::<ScheduleListItem>())
                        .collect::<Vec<_>>();

                    let Some(child) = child else {
                        return true;
                    };

                    let Some(listitem) = child
                        .ancestor(ScheduleListItem::static_type())
                        .and_then(|w| w.downcast::<ScheduleListItem>().ok())
                        .or_else(|| {
                            child
                                .first_child()
                                .and_then(|w| w.downcast::<ScheduleListItem>().ok())
                        })
                    else {
                        obj.add_new_item(&item);
                        obj.imp().listview.grab_focus();
                        return true;
                    };

                    let over_item = items.iter().position(|v| *v == listitem);

                    obj.add_new_item_at(over_item.unwrap_or(items.len()) as u32, &item);
                    obj.imp().listview.grab_focus();

                    return true;
                }
            ));
            drop_target.connect_motion(glib::clone!(
                #[weak]
                obj,
                #[upgrade_or]
                gdk::DragAction::COPY,
                move |_, x, y| {
                    let child = obj.imp().listview.pick(x, y, gtk::PickFlags::DEFAULT);

                    let Some(child) = child else {
                        return gdk::DragAction::COPY;
                    };

                    if let Some(li) = child
                        .ancestor(ScheduleListItem::static_type())
                        .and_then(|w| w.downcast::<ScheduleListItem>().ok())
                        .or_else(|| {
                            child
                                .first_child()
                                .and_then(|w| w.downcast::<ScheduleListItem>().ok())
                        })
                        && let Some(li) = li.parent()
                    {
                        let lv = obj.imp().listview.clone();
                        for item in lv.children() {
                            if item.accessible_role() == gtk::AccessibleRole::ListItem {
                                item.remove_css_class("drop-over");
                            }
                        }
                        li.add_css_class("drop-over");
                    };

                    return gdk::DragAction::COPY;
                }
            ));

            self.listview.add_controller(drop_target);
        }
    }
}

glib::wrapper! {
    pub struct ScheduleActivityViewer(ObjectSubclass<imp::ScheduleActivityViewer>)
        @extends  gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for ScheduleActivityViewer {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl ScheduleActivityViewer {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn add_new_item(&self, payload: &SlideManagerData) {
        let imp = self.imp();

        if let Some(model) = imp.listview.model() {
            self.add_new_item_at(model.n_items(), payload);
        };
    }

    pub fn add_new_item_at(&self, position: u32, payload: &SlideManagerData) {
        let imp = self.imp();
        let listview = imp.listview.clone();

        let Some(model) = listview.model() else {
            return;
        };

        let data = schedule_data::ScheduleData::new(payload.title.clone(), payload.clone());
        listview.insert_item(position, &data);
        model.select_item(position, true);
        for (index, listitem) in listview.children().enumerate() {
            if index == position as usize
                && listitem.accessible_role() == gtk::AccessibleRole::ListItem
            {
                listview.set_focus_child(Some(&listitem));
                break;
            }
        }
    }

    pub fn connect_activate<F: Fn(&Self, &SlideManagerData) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            signals::ACTIVATE,
            false,
            glib::closure_local!(move |obj: &Self, slide_data: &SlideManagerData| f(
                obj, slide_data
            )),
        )
    }

    pub fn emit_activate(&self, data: &SlideManagerData) {
        self.emit_by_name::<()>(signals::ACTIVATE, &[data]);
    }
}
