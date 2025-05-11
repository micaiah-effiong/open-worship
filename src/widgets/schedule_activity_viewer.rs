use std::{cell::RefCell, rc::Rc};

use gtk::{
    gio::{ActionEntry, MenuItem, SimpleActionGroup},
    glib::clone,
    prelude::*,
    SingleSelection,
};
use relm4::{prelude::*, typed_view::list::TypedListView};

use crate::{
    dto::{self, ListPayload},
    structs::schedule_list_item::ScheduleListItemModel,
};

const MIN_GRID_HEIGHT: i32 = 300;
// const MIN_GRID_WIDTH: i32 = 300;

#[derive(Debug)]
pub enum ScheduleViewerInput {
    NewItem(ListPayload),
    RemoveItem(u32),
}
#[derive(Debug)]
pub enum ScheduleViewerOutput {
    Activated(dto::ListPayload),
}

#[derive(Debug, Clone)]
pub struct ScheduleData {
    list: Vec<String>,
    title: String,
}

#[derive(Clone)]
pub struct ScheduleViewerModel {
    title: String,
    list_view_wrapper: Rc<RefCell<TypedListView<ScheduleListItemModel, SingleSelection>>>,
}

impl ScheduleViewerModel {
    fn new(list_data: Option<Vec<ScheduleData>>) -> Self {
        let list = list_data.unwrap_or_default();

        let mut t_view = TypedListView::new();
        for item in list.clone() {
            t_view.append(ScheduleListItemModel::new(item.title, item.list, None));
        }

        ScheduleViewerModel {
            title: String::from("Schedule"),
            list_view_wrapper: Rc::new(RefCell::new(t_view)),
        }
    }

    fn register_context_menu(&self, sender: &ComponentSender<ScheduleViewerModel>) {
        let list_view = self.list_view_wrapper.borrow().view.clone();

        let remove_action = ActionEntry::builder("remove_item")
            .activate(clone!(
                #[strong]
                list_view,
                #[strong]
                sender,
                move |_g: &SimpleActionGroup, _sa, _v| {
                    if let Some(m) = list_view.model() {
                        sender.input(ScheduleViewerInput::RemoveItem(m.selection().nth(0)));
                    };
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
        popover_menu.set_position(gtk::PositionType::Right);
        popover_menu.set_parent(&list_view);

        let gesture_click = gtk::GestureClick::new();
        gesture_click.set_button(gtk::gdk::BUTTON_SECONDARY);
        gesture_click.connect_pressed(clone!(
            #[strong]
            popover_menu,
            move |gc, _n, x, y| {
                let rect = gtk::gdk::Rectangle::new(x as i32, y as i32, 10, 10);
                popover_menu.set_pointing_to(Some(&rect));
                popover_menu.popup();
                gc.set_state(gtk::EventSequenceState::Claimed);
            }
        ));

        list_view.add_controller(gesture_click);
        list_view.insert_action_group("schedule", Some(&menu_action_group));
    }

    fn register_activate(&self, sender: &ComponentSender<Self>) {
        let list_view_wrapper = self.list_view_wrapper.clone();
        let list_view = list_view_wrapper.borrow().view.clone();

        list_view.connect_activate(clone!(
            #[strong]
            list_view_wrapper,
            #[strong]
            sender,
            move |list_view, _| {
                let model = match list_view.model() {
                    Some(m) => m,
                    None => return,
                };

                let ss_model = match model.downcast_ref::<gtk::SingleSelection>() {
                    Some(ss) => ss,
                    None => return,
                };

                let pos = ss_model.selected();
                println!("activate-preview {:?}", &pos);

                let data_list = match list_view_wrapper.borrow().get(pos) {
                    Some(list_item) => list_item,
                    None => return,
                };

                let data_list = data_list.borrow();

                let payload = dto::ListPayload {
                    text: data_list.title.to_string(),
                    list: data_list.list.clone(),
                    position: pos,
                    background_image: data_list.backgound_image.clone(),
                };
                let _ = sender.output(ScheduleViewerOutput::Activated(payload));
            }
        ));
    }
}

#[relm4::component(pub)]
impl SimpleComponent for ScheduleViewerModel {
    type Input = ScheduleViewerInput;
    type Output = ScheduleViewerOutput;
    type Init = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_vexpand: true,
            set_height_request: MIN_GRID_HEIGHT,
            set_css_classes: &["pink_box", "ow-listview"],

            gtk::Label {
                set_label: &model.title
            },

            gtk::ScrolledWindow {
                set_vexpand: true,

                #[local_ref]
                list_view -> gtk::ListView{},
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = ScheduleViewerModel::new(None);
        let list_view = model.list_view_wrapper.borrow().view.clone();

        model.register_activate(&sender);
        model.register_context_menu(&sender);

        let widgets = view_output!();

        relm4::ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            ScheduleViewerInput::RemoveItem(position) => {
                self.list_view_wrapper.borrow_mut().remove(position);
            }
            ScheduleViewerInput::NewItem(payload) => {
                let list_item = ScheduleListItemModel::new(
                    payload.text,
                    payload.list,
                    payload.background_image,
                );
                self.list_view_wrapper.borrow_mut().append(list_item);
                self.list_view_wrapper.borrow().view.clone().grab_focus();
            }
        };
    }
}
