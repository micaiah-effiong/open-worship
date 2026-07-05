use gobject_macro::gobject_props;
use gtk::glib;
use gtk::prelude::ObjectExt;

use crate::widgets::canvas::serialise::SlideManagerData;
mod presentation_listitem;

mod signals {
    pub(super) const SEND_PREVIEW_PRESENTATION: &str = "send-preview-presentation";
    pub(super) const SEND_TO_SCHEDULE: &str = "send-to-schedule";
}

#[gobject_props]
struct PresentationObj {
    pub id: u32,
    pub data: SlideManagerData,
}

impl Default for PresentationObj {
    fn default() -> Self {
        glib::Object::new()
    }
}

mod imp {
    use std::{cell::RefCell, collections::HashSet, sync::OnceLock};

    use gtk::{
        gio::{
            self,
            prelude::{ActionMapExt, ListModelExt},
        },
        glib::{
            self,
            object::{Cast, CastNone},
            subclass::{
                Signal,
                object::{ObjectImpl, ObjectImplExt},
                types::{ObjectSubclass, ObjectSubclassExt, ObjectSubclassIsExt},
            },
            types::StaticType,
            value::ToValue,
        },
        prelude::{
            EditableExt, FilterExt, GestureExt, GestureSingleExt, ListItemExt, PopoverExt,
            WidgetExt,
        },
        subclass::{
            box_::BoxImpl,
            widget::{
                CompositeTemplateCallbacksClass, CompositeTemplateClass,
                CompositeTemplateInitializingExt, WidgetClassExt, WidgetImpl,
            },
        },
    };

    use super::*;
    use crate::{
        db::query::Query,
        utils::ListViewExtra,
        widgets::search::{
            presentation::presentation_listitem::PresentationListItem,
            songs::edit_modal::SongEditWindow,
        },
    };

    #[derive(Default, Debug, gtk::CompositeTemplate)]
    #[template(resource = "/com/openworship/app/ui/search_presentation.ui")]
    pub struct SearchPresentation {
        #[template_child]
        search_text: gtk::TemplateChild<gtk::SearchEntry>,
        #[template_child]
        listview: gtk::TemplateChild<gtk::ListView>,
        #[template_child]
        add_btn: gtk::TemplateChild<gtk::Button>,
        #[template_child]
        remove_btn: gtk::TemplateChild<gtk::Button>,

        search_timeout_id: RefCell<Option<glib::SourceId>>,
        filter_set: RefCell<HashSet<u32>>,
        filter_model: RefCell<gtk::CustomFilter>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SearchPresentation {
        const NAME: &'static str = "SearchPresentation";
        type Type = super::SearchPresentation;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SearchPresentation {
        fn constructed(&self) {
            self.parent_constructed();

            let listview = self.listview.clone();

            let store = gio::ListStore::new::<PresentationObj>();
            let custom_filter = gtk::CustomFilter::new(glib::clone!(
                #[weak(rename_to=imp)]
                self,
                #[upgrade_or]
                false,
                move |obj| {
                    if imp.search_text.text().is_empty() {
                        return true;
                    }

                    let presentation = obj
                        .downcast_ref::<PresentationObj>()
                        .expect("Should be PresentationObj");
                    let set = imp.filter_set.borrow();

                    set.contains(&presentation.id())
                }
            ));
            self.filter_model.replace(custom_filter.clone());
            let filter_model = gtk::FilterListModel::new(Some(store), Some(custom_filter));

            let model = gtk::SingleSelection::new(Some(filter_model));
            listview.set_model(Some(&model));

            let factory = gtk::SignalListItemFactory::new();
            listview.set_factory(Some(&factory));

            factory.connect_setup(|_, list_item| {
                let li = list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Expected ListItem");

                let p = PresentationListItem::new();

                // let label = gtk::Label::default();
                // label.set_xalign(0.0);
                //
                // let pic = gtk::Picture::new();
                // pic.set_height_request(40);
                // pic.set_css_classes(&["bg-preview-box"]);
                //
                // let b = gtk::Box::new(gtk::Orientation::Horizontal, 4);
                // b.append(&pic);
                // b.append(&label);
                // println!("{:?} => {:?}", li, b);
                li.set_child(Some(&p));
            });
            factory.connect_bind(|_, list_item| {
                let li = list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Expected ListItem");
                let item = li
                    .item()
                    .and_downcast::<PresentationObj>()
                    .expect("Expected PresentationObj");
                let box_w = li
                    .child()
                    .and_downcast::<PresentationListItem>()
                    .expect("Expected PresentationListItem");

                box_w.bind(&item);
                // box_w
                //     .parent()
                //     .expect("Expected GtkListItemWidget")
                //     .set_css_classes(&["small-pad"]);
                //
                // let c = box_w.children().collect::<Vec<_>>();
                // assert_eq!(c.len(), 2, "How is this not 2 children");
                //
                // let (pic, label) = (
                //     c.get(0)
                //         .and_then(|v| v.downcast_ref::<gtk::Picture>())
                //         .expect("Expected Picture"),
                //     c.get(1)
                //         .and_then(|v| v.downcast_ref::<gtk::Label>())
                //         .expect("Expected Label"),
                // );
                //
                // let slide = item.data().slides.first().cloned().unwrap_or_default();
                //
                // match gtk::gdk::Texture::from_bytes(&glib::Bytes::from(&slide.preview.clone())) {
                //     Ok(t) => pic.set_paintable(Some(&t)),
                //     Err(e) => println!("Shit {:?}", e),
                // };
                //
                // label.set_label(&item.data().title.clone());
            });

            factory.connect_unbind(|_, list_item| {
                list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Expected ListItem")
                    .child()
                    .and_downcast::<PresentationListItem>()
                    .expect("Expected PresentationListItem")
                    .unbind();
            });

            self.register_context_menu();
            self.register_search_field_events();
            self.load_presentation();
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();

            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder(signals::SEND_PREVIEW_PRESENTATION)
                        .param_types([SlideManagerData::static_type()])
                        .build(),
                    Signal::builder(signals::SEND_TO_SCHEDULE)
                        .param_types([SlideManagerData::static_type()])
                        .build(),
                ]
            })
        }
    }

    impl WidgetImpl for SearchPresentation {}
    impl BoxImpl for SearchPresentation {}

    #[gtk::template_callbacks]
    impl SearchPresentation {
        #[template_callback]
        fn handle_search_activate(&self) {}

        #[template_callback]
        fn handle_preview_presentation(&self, _: u32, _: &gtk::ListView) {
            let model = self
                .listview
                .model()
                .and_downcast::<gtk::SingleSelection>()
                .expect("Expected gtk::SingleSelection");

            let Some(listitem) = model.selected_item().and_downcast::<PresentationObj>() else {
                return;
            };

            self.obj().emit_send_preview_presentation(&listitem.data())
        }

        #[template_callback]
        fn handle_add_presentation(&self, _: &gtk::Button) {
            self.open_editor(None);
        }

        #[template_callback]
        fn handle_remove_presentation(&self, _: &gtk::Button) {
            let model = self
                .listview
                .model()
                .and_downcast::<gtk::SingleSelection>()
                .expect("Expected gtk::SingleSelection");

            if model.n_items() == 0 {
                return;
            }
            let Some(listitem) = model.selected_item().and_downcast::<PresentationObj>() else {
                return;
            };

            let _ = Query::delete_presentation(listitem.id()).map(|_| self.load_presentation());
        }
    }
    impl SearchPresentation {
        fn register_context_menu(&self) {
            let listview = self.listview.clone();
            let model = listview
                .model()
                .and_downcast::<gtk::SingleSelection>()
                .expect("Expected gtk::SingleSelection");

            let add_action = gio::SimpleAction::new("add-presentation", None);
            add_action.connect_activate(glib::clone!(
                #[weak(rename_to=imp)]
                self,
                move |_sa, _v| imp.handle_add_presentation(&gtk::Button::default())
            ));

            let edit_action = gio::SimpleAction::new("edit", None);
            edit_action.connect_activate(glib::clone!(
                #[strong]
                model,
                #[weak(rename_to=imp)]
                self,
                move |_sa, _v| {
                    model
                        .selected_item()
                        .and_downcast::<PresentationObj>()
                        .map(|v| imp.open_editor(Some(v)));
                }
            ));

            let add_to_schedule_action = gio::SimpleAction::new("add-to-schedule", None);
            add_to_schedule_action.connect_activate(glib::clone!(
                #[strong]
                listview,
                #[weak(rename_to=imp)]
                self,
                move |_sa, _v| {
                    let model = listview
                        .model()
                        .and_downcast::<gtk::SingleSelection>()
                        .expect("Expected gtk::SingleSelection");

                    let Some(listitem) = model.selected_item().and_downcast::<PresentationObj>()
                    else {
                        return;
                    };

                    imp.obj().emit_send_to_schedule(&listitem.data())
                }
            ));

            let delete_action = gio::SimpleAction::new("delete", None);
            delete_action.connect_activate(glib::clone!(
                #[weak(rename_to=imp)]
                self,
                move |_sa, _v| imp.handle_remove_presentation(&gtk::Button::default())
            ));

            let menu_action_group = gio::SimpleActionGroup::new();
            listview.insert_action_group("presentation", Some(&menu_action_group));
            menu_action_group.add_action(&add_action);
            menu_action_group.add_action(&edit_action);
            menu_action_group.add_action(&add_to_schedule_action);
            menu_action_group.add_action(&delete_action);

            let menu = gtk::gio::Menu::new();
            let add_to_schedule = gio::MenuItem::new(
                Some("Add to schedule"),
                Some("presentation.add-to-schedule"),
            );

            menu.insert_item(1, &add_to_schedule);
            menu.insert_item(
                2,
                &gio::MenuItem::new(
                    Some("Add Presentation"),
                    Some("presentation.add-presentation"),
                ),
            );
            menu.insert_item(
                3,
                &gio::MenuItem::new(Some("Edit presentation"), Some("presentation.edit")),
            );
            menu.insert_item(
                4,
                &gio::MenuItem::new(Some("Delete presentation"), Some("presentation.delete")),
            );

            let popover_menu = gtk::PopoverMenu::from_model(Some(&menu));
            popover_menu.set_has_arrow(false);
            popover_menu.set_halign(gtk::Align::Start);
            popover_menu.set_valign(gtk::Align::Start);
            popover_menu.set_parent(&listview);

            let gesture_click = gtk::GestureClick::new();
            gesture_click.set_button(gtk::gdk::BUTTON_SECONDARY);
            gesture_click.connect_released(glib::clone!(
                #[weak]
                listview,
                move |gc, _, x, y| {
                    let model = listview
                        .model()
                        .and_downcast::<gtk::SingleSelection>()
                        .expect("Expected gtk::SingleSelection");

                    let item = model.selected_item().and_downcast::<PresentationObj>();

                    let enable = item.is_some();
                    edit_action.set_enabled(enable);
                    add_to_schedule_action.set_enabled(enable);
                    delete_action.set_enabled(enable);

                    let rect = gtk::gdk::Rectangle::new(x as i32, y as i32, 0, 0);
                    popover_menu.set_pointing_to(Some(&rect));
                    popover_menu.popup();
                    gc.set_state(gtk::EventSequenceState::Claimed);
                }
            ));

            listview.add_controller(gesture_click);
        }

        fn open_editor(&self, initial_data: Option<PresentationObj>) {
            let slide_data = initial_data.as_ref().map(|v| v.data());
            let edit_window = SongEditWindow::new(None);
            let obj = self.obj().clone();

            edit_window.connect_save(glib::clone!(
                #[weak]
                obj,
                move |w, data| {
                    let res = match (w.is_new(), initial_data.as_ref()) {
                        (true, _) => Query::insert_presentation(data),
                        (false, Some(initial)) => Query::update_presentation(initial.id(), data),
                        _ => return,
                    };

                    if let Err(e) = res {
                        glib::g_warning!(
                            "Insert/Update Presentation",
                            "Error adding/updating presentation: \n{:?}",
                            e
                        );
                        return;
                    };

                    obj.imp().load_presentation();
                }
            ));

            edit_window.show(slide_data);
        }

        fn load_presentation(&self) {
            let presentation = Query::search_presentations("");

            let Some(store) = self.listview.get_list_store() else {
                return;
            };

            match presentation {
                Ok(p) => {
                    self.listview.remove_all();
                    let song_slice: Vec<_> = p
                        .into_iter()
                        .map(|(id, data)| PresentationObj::new(id, data))
                        .collect();
                    store.extend_from_slice(&song_slice);
                }
                Err(e) => eprintln!("SQL ERROR: {:?}", e),
            }
        }

        fn register_search_field_events(&self) {
            let search = self.search_text.clone();
            let list = self.listview.clone();

            search.connect_changed(glib::clone!(
                #[weak(rename_to=imp)]
                self,
                move |search_widget| {
                    if let Some(id) = imp.search_timeout_id.take() {
                        id.remove();
                    }

                    let timeout_id = glib::timeout_add_local_once(
                        std::time::Duration::from_millis(300),
                        glib::clone!(
                            #[weak]
                            imp,
                            #[strong]
                            search_widget,
                            move || {
                                let query = Query::search_presentations(&search_widget.text());

                                let presentation_ids: HashSet<_> = match query {
                                    Ok(q) => q.into_iter().map(|(id, _)| id).collect(),
                                    Err(e) => {
                                        eprintln!("SQL ERROR: {:?}", e);
                                        HashSet::new()
                                    }
                                };
                                imp.filter_set.replace(presentation_ids);

                                //
                                imp.filter_model
                                    .borrow()
                                    .changed(gtk::FilterChange::Different);
                                imp.search_timeout_id.take();
                            }
                        ),
                    );

                    imp.search_timeout_id.replace(Some(timeout_id));
                }
            ));

            search.connect_activate(glib::clone!(
                #[strong]
                list,
                move |_se| {
                    println!("S ACTIVATE");
                    let Some(model) = list.model().and_downcast::<gtk::SingleSelection>() else {
                        return;
                    };

                    let selected = model.selected().to_value();
                    list.emit_by_name_with_values("activate", &[selected]);
                }
            ));
        }
    }
}

glib::wrapper! {
    pub struct SearchPresentation(ObjectSubclass<imp::SearchPresentation>)
    @extends  gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Orientable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for SearchPresentation {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl SearchPresentation {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn connect_send_preview_presentation<F: Fn(&Self, &SlideManagerData) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            signals::SEND_PREVIEW_PRESENTATION,
            false,
            glib::closure_local!(|obj: &Self, data: &SlideManagerData| f(obj, data)),
        )
    }
    fn emit_send_preview_presentation(&self, data: &SlideManagerData) {
        self.emit_by_name(signals::SEND_PREVIEW_PRESENTATION, &[data])
    }

    pub fn connect_send_send_to_schedule<F: Fn(&Self, &SlideManagerData) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            signals::SEND_TO_SCHEDULE,
            false,
            glib::closure_local!(|obj: &Self, data: &SlideManagerData| f(obj, data)),
        )
    }
    fn emit_send_to_schedule(&self, data: &SlideManagerData) {
        self.emit_by_name(signals::SEND_TO_SCHEDULE, &[data])
    }
}
