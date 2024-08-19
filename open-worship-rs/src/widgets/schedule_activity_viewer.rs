use std::{cell::RefCell, rc::Rc};

use gtk::prelude::*;
use relm4::{prelude::*, typed_view::list::TypedListView};

use crate::{
    dto::{self, ListPayload},
    structs::schedule_list_item::ScheduleListItem,
};

const MIN_GRID_HEIGHT: i32 = 300;
// const MIN_GRID_WIDTH: i32 = 300;

#[derive(Debug)]
pub enum ScheduleViewerInput {
    NewList(ListPayload),
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

pub struct ScheduleViewerData {
    pub title: String,
    pub list: Vec<ScheduleData>,
}

#[derive(Clone)]
pub struct ScheduleViewerModel {
    title: String,
    list: Rc<RefCell<Vec<ScheduleData>>>,
    // list_view: gtk::ListView,
}

#[relm4::component(pub)]
impl SimpleComponent for ScheduleViewerModel {
    type Input = ScheduleViewerInput;
    type Output = ScheduleViewerOutput;
    type Init = ScheduleViewerData;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_height_request: MIN_GRID_HEIGHT,
            set_css_classes: &["pink_box", "ow-listview"],

            gtk::Label {
                set_label: &model.title
            },

            gtk::ScrolledWindow {
                set_vexpand: true,

                #[wrap(Some)]
                #[local_ref]
                set_child = &list_view -> gtk::ListView{

                    connect_activate[sender, model] => move |list_view,_|{
                        let selection_model = match list_view.model() {
                            Some(m)=>m,
                            None=>return,
                        };

                        let ss_model = match selection_model.downcast_ref::<gtk::SingleSelection>(){
                            Some(ss)=>ss,
                            None => return,
                        };

                        let pos = ss_model.selected();
                        println!("activate-preview {:?}", &pos);

                        let schedule_data = model.list.borrow();

                        let data_list = match schedule_data.get(pos as usize) {
                            Some(txt) => txt,
                            None => return,
                        };

                        let payload = dto::ListPayload {
                            text: data_list.title.to_string(),
                            list: data_list.list.clone(),
                            position: pos
                        };
                        let _ = sender.output(ScheduleViewerOutput::Activated(payload));
                    },

                },
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let mut list_view: TypedListView<ScheduleListItem, gtk::SingleSelection> = TypedListView::new();

        let model = ScheduleViewerModel {
            title: init.title,
            list: Rc::new(RefCell::new(get_default_data())),
        };

        for item in model.list.borrow().clone() {
            list_view.append(ScheduleListItem::new(item.title, item.list))
        }

        let list_view = list_view.view;

        let widgets = view_output!();

        return relm4::ComponentParts { model, widgets };
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            ScheduleViewerInput::NewList(payload) => {
                // self.list.borrow_mut().clear();
                // self.list.borrow_mut().append(&mut payload.list.clone());

                println!("schedule new sli pos={}", payload.position);
            }
        };
    }
}

fn get_default_data ()->Vec<ScheduleData>{
    return Vec::from([
        ScheduleData{
            title: "Echoes of the Soul".to_string(),
            list:Vec::from(
                [
                    "In depths of silence, where thoughts reside, A melody whispers, a soulful tide. Emotions painted in hues of night, Searching for solace, a guiding light.".to_string(),
                    "Lost in the echoes of yesterday's dream, A fragile heart, a delicate scheme. Yearning for moments, pure and true, Finding strength within, a different view.".to_string(),
                    "With every heartbeat, a rhythm's grace, A journey inward, a soulful space. Through shadows and light, the spirit soars, Discovering treasures, unlocking doors.".to_string(),
                    "In harmony's embrace, the soul finds peace, A gentle whisper, a sweet release. With hope as a compass, a steady hand, Walking the path, through this mortal land.".to_string()
                ]
            ) 
        },

        ScheduleData{
            title: "City Lights and Lonely Nights".to_string(),
            list: Vec::from([
                "Neon signs blink, a dazzling display, But shadows deepen as the night creeps in. A bustling city, a vibrant scene, Yet solitude lingers, a haunting refrain.".to_string(),
                "Lost in the crowd, a faceless name, Searching for connection, a fading flame. The rhythm of life, a constant chase, A longing for love, a warm embrace.".to_string(),
                "Dreams and aspirations, a fragile art, A fragile heart, torn apart. The city's allure, a deceptive guise, Hiding the yearning, beneath the disguise.".to_string(),
                "In the quiet moments, when silence descends, A soul yearns for peace, where solace transcends. A flicker of hope, a distant star, Guiding the way, from near or far.".to_string()  
            ])
        },
        ScheduleData{
            title: "Whispers of the Wind".to_string(),
            list: Vec::from([
                "The wind whispers secrets, through rustling leaves, Carrying stories, of life's reprieves. A gentle caress, a soothing sound, Nature's symphony, all around.".to_string(),
                "From distant lands, it carries dreams, Of endless horizons, and sparkling streams. A touch of magic, a playful breeze, Dancing with freedom, through ancient trees.".to_string(),
                "It sings of love, of loss, and pain, Of hope and resilience, again and again. A constant companion, a faithful friend, A gentle reminder, that life will transcend.".to_string(),
                "With every gust, a promise of new, A chance to start, with a different view. In its embrace, find solace and grace, Let the wind's wisdom, fill your space.".to_string()
            ])
        },
        ScheduleData{
            title: "Fragments of Time".to_string(),
            list: Vec::from([
                "Time slips away, like grains of sand, Leaving footprints, on life's vast land. Memories linger, a bittersweet art, Shaping the soul, from the very start.".to_string(),
                "In the tapestry of years gone by, Moments of joy, and reasons to cry. Lessons learned, through trials and strife, Building resilience, for a stronger life.".to_string(),
                "The clock ticks on, an endless race, Chasing dreams, at a frantic pace. Yet, in stillness, find inner peace, A sanctuary of calm, a sweet release.".to_string(),
                "Embrace the present, with open heart, Let go of the past, make a brand new start. Time is a gift, a precious art, Use it wisely, from the very start.".to_string()
            ])
        },
        ScheduleData{
            title: "Ocean's Lullaby".to_string(),
            list: Vec::from([
                "Waves crash gently, on sandy shores, Carrying secrets, from ocean's cores. The moon's soft glow, a silvered sea, A tranquil beauty, wild and free.".to_string(),
                "Beneath the surface, a world unknown, Creatures of wonder, a mystic zone. The rhythm of tides, a constant flow, A dance of nature, a wondrous show.".to_string(),
                "In solitude's embrace, the soul finds peace, As ocean's melody, offers release. The vast expanse, a mirror of mind, Reflecting depths, where answers reside.".to_string(),
                "With every tide, a chance to renew, To wash away worries, old and new. In ocean's wisdom, find strength to be, A harmonious part, of eternity.".to_string()
            ])
        } 
    ]);
}
