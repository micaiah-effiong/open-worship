use std::{cell::RefCell, rc::Rc};

use gtk::{
    gio::{ActionEntry, MenuItem, SimpleActionGroup},
    glib::clone,
    prelude::*,
    subclass::popover,
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
    background_image: Rc<RefCell<Option<String>>>,
    list_view_wrapper: Rc<RefCell<TypedListView<ScheduleListItemModel, SingleSelection>>>,
}

impl ScheduleViewerModel {
    fn new(list_data: Option<Vec<ScheduleData>>) -> Self {
        let list = match list_data {
            Some(list) => list,
            None => Vec::new(),
        };

        let mut t_view = TypedListView::new();
        for item in list.clone() {
            t_view.append(ScheduleListItemModel::new(item.title, item.list));
        }

        return ScheduleViewerModel {
            background_image: Rc::new(RefCell::new(None)),
            title: String::from("Schedule"),
            list_view_wrapper: Rc::new(RefCell::new(t_view)),
        };
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
        gesture_click.set_button(gtk::gdk::ffi::GDK_BUTTON_SECONDARY as u32);
        gesture_click.connect_pressed(clone!(
            #[strong]
            popover_menu,
            move |_gc, _n, x, y| {
                let rect = gtk::gdk::Rectangle::new(x as i32, y as i32, 10, 10);
                popover_menu.set_pointing_to(Some(&rect));
                popover_menu.popup();
            }
        ));

        list_view.add_controller(gesture_click);
        list_view.insert_action_group("schedule", Some(&menu_action_group));
    }

    fn register_activate(&self, sender: &ComponentSender<Self>) {
        let list_view_wrapper = self.list_view_wrapper.clone();
        let list_view = list_view_wrapper.borrow().view.clone();
        let background_image = self.background_image.clone();

        list_view.connect_activate(clone!(
            #[strong]
            list_view_wrapper,
            #[strong]
            background_image,
            #[strong]
            sender,
            move |list_view, _| {
                let selection_model = match list_view.model() {
                    Some(m) => m,
                    None => return,
                };

                let ss_model = match selection_model.downcast_ref::<gtk::SingleSelection>() {
                    Some(ss) => ss,
                    None => return,
                };

                let pos = ss_model.selected();
                println!("activate-preview {:?}", &pos);

                let schedule_data = list_view_wrapper.borrow();

                let data_list = match schedule_data.get(pos) {
                    Some(txt) => txt,
                    None => return,
                };

                let data_list = data_list.borrow();

                let payload = dto::ListPayload {
                    text: data_list.title.to_string(),
                    list: data_list.list.clone(),
                    position: pos,
                    background_image: background_image.borrow().clone(),
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
        let model = ScheduleViewerModel::new(Some(get_default_data()));
        let list_view = model.list_view_wrapper.borrow().view.clone();

        model.register_activate(&sender);
        model.register_context_menu(&sender);

        let widgets = view_output!();

        return relm4::ComponentParts { model, widgets };
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            ScheduleViewerInput::RemoveItem(position) => {
                self.list_view_wrapper.borrow_mut().remove(position);
            }
            ScheduleViewerInput::NewItem(payload) => {
                self.list_view_wrapper
                    .borrow_mut()
                    .append(ScheduleListItemModel::new(payload.text, payload.list));
                self.list_view_wrapper.borrow().view.clone().grab_focus();
            }
        };
    }
}

fn get_default_data() -> Vec<ScheduleData> {
    return Vec::from([
        ScheduleData{
            title: "Echoes of the Soul".to_string(),
            list:Vec::from(
                [
                    "In depths of silence, where thoughts reside,\nA melody whispers, a soulful tide.\nEmotions painted in hues of night,\nSearching for solace, a guiding light.".to_string(),
                    "Lost in the echoes of yesterday's dream,\nA fragile heart, a delicate scheme.\nYearning for moments, pure and true,\nFinding strength within, a different view.".to_string(),
                    "With every heartbeat, a rhythm's grace,\nA journey inward, a soulful space.\nThrough shadows and light, the spirit soars,\nDiscovering treasures, unlocking doors.".to_string(),
                    "In harmony's embrace, the soul finds peace,\nA gentle whisper, a sweet release.\nWith hope as a compass, a steady hand,\nWalking the path, through this mortal land.".to_string()
                ]
            )
        },

        ScheduleData{
            title: "City Lights and Lonely Nights".to_string(),
            list: Vec::from([
                "Neon signs blink, a dazzling display,\nBut shadows deepen as the night creeps in.\nA bustling city, a vibrant scene,\nYet solitude lingers, a haunting refrain.".to_string(),
                "Lost in the crowd, a faceless name,\nSearching for connection, a fading flame.\nThe rhythm of life, a constant chase,\nA longing for love, a warm embrace.".to_string(),
                "Dreams and aspirations, a fragile art,\nA fragile heart, torn apart.\nThe city's allure, a deceptive guise,\nHiding the yearning, beneath the disguise.".to_string(),
                "In the quiet moments, when silence descends,\nA soul yearns for peace, where solace transcends. \nA flicker of hope, a distant star,\nGuiding the way, from near or far.".to_string()  
            ])
        },
        ScheduleData{
            title: "Whispers of the Wind".to_string(),
            list: Vec::from([
                "The wind whispers secrets, through rustling leaves,\nCarrying stories, of life's reprieves.\nA gentle caress, a soothing sound,\nNature's symphony, all around.".to_string(),
                "From distant lands, it carries dreams,\nOf endless horizons, and sparkling streams.\nA touch of magic, a playful breeze,\nDancing with freedom, through ancient trees.".to_string(),
                "It sings of love, of loss, and pain,\nOf hope and resilience, again and again.\nA constant companion, a faithful friend,\nA gentle reminder, that life will transcend.".to_string(),
                "With every gust, a promise of new,\nA chance to start, with a different view.\nIn its embrace, find solace and grace,\nLet the wind's wisdom, fill your space.".to_string()
            ])
        },
        ScheduleData{
            title: "Fragments of Time".to_string(),
            list: Vec::from([
                "Time slips away, like grains of sand,\nLeaving footprints, on life's vast land.\nMemories linger, a bittersweet art,\nShaping the soul, from the very start.".to_string(),
                "In the tapestry of years gone by,\nMoments of joy, and reasons to cry.\nLessons learned, through trials and strife,\nBuilding resilience, for a stronger life.".to_string(),
                "The clock ticks on, an endless race,\nChasing dreams, at a frantic pace.\nYet, in stillness, find inner peace,\nA sanctuary of calm, a sweet release.".to_string(),
                "Embrace the present, with open heart, Let go of the past, make a brand new start. Time is a gift, a precious art,\nUse it wisely, from the very start.".to_string()
            ])
        },
        ScheduleData{
            title: "Ocean's Lullaby".to_string(),
            list: Vec::from([
                "Waves crash gently, on sandy shores,\nCarrying secrets, from ocean's cores.\nThe moon's soft glow, a silvered sea,\nA tranquil beauty, wild and free.".to_string(),
                "Beneath the surface, a world unknown,\nCreatures of wonder, a mystic zone.\nThe rhythm of tides, a constant flow,\nA dance of nature, a wondrous show.".to_string(),
                "In solitude's embrace, the soul finds peace,\nAs ocean's melody, offers release.\nThe vast expanse, a mirror of mind,\nReflecting depths, where answers reside.".to_string(),
                "With every tide, a chance to renew,\nTo wash away worries, old and new.\nIn ocean's wisdom, find strength to be,\nA harmonious part, of eternity.".to_string()
            ])
        }
    ]);
}
