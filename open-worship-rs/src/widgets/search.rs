use std::{cell::RefCell, rc::Rc};

use gtk::prelude::*;
use relm4::{prelude::*, typed_view::grid::TypedGridView};

use crate::structs::background_grid_list_item::BackgroundGridListItem;

const MIN_GRID_HEIGHT: i32 = 300;
// const MIN_GRID_WIDTH: i32 = 300;

// search area (notebook)
#[derive(Debug)]
pub enum SearchInput {}

#[derive(Debug)]
pub enum SearchOutput {
    PreviewBackground(String),
}

#[derive(Debug, Clone)]
pub struct SearchModel {
    image_src_list: Rc<RefCell<Vec<String>>>,
}

pub struct SearchInit {
    pub image_src_list: Vec<String>,
}

#[relm4::component(pub)]
impl SimpleComponent for SearchModel {
    type Init = SearchInit;
    type Output = SearchOutput;
    type Input = SearchInput;

    view! {
        #[root]
        gtk::Box{
            set_orientation: gtk::Orientation::Vertical,
            set_height_request: MIN_GRID_HEIGHT,
            set_hexpand: true,
            set_homogeneous: true,

            #[name="tab_box"]
            gtk::Box {
                set_orientation:gtk::Orientation::Horizontal,
                set_spacing: 3,
                set_css_classes: &["purple_box", "ow-listview"],
                set_height_request: 48,

                gtk::Notebook {
                    set_hexpand: true,

                    append_page[Some(&gtk::Label::new(Some("Songs")))] = &gtk::Box {
                        set_orientation:gtk::Orientation::Vertical,
                        set_vexpand: true,
                        add_css_class: "blue_box",

                        #[name="search_field"]
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 2,
                            set_height_request: 48,
                            add_css_class: "green_double_box",

                            gtk::SearchEntry {
                                set_placeholder_text: Some("Search..."),
                                set_hexpand: true
                            }
                        },

                        gtk::ScrolledWindow {
                            set_vexpand: true,

                            #[wrap(Some)]
                            set_child = &gtk::ListView {

                                #[wrap(Some)]
                                set_model = &gtk::SingleSelection{
                                    set_model: Some(&(0..1000).map(|_| LIST_VEC[0]).collect::<gtk::StringList>()),
                                },

                                #[wrap(Some)]
                                set_factory = &gtk::SignalListItemFactory {
                                    connect_setup => move |_, list_item|{
                                        let label = gtk::Label::builder()
                                        .ellipsize(gtk::pango::EllipsizeMode::End)
                                        .single_line_mode(true)
                                        .halign(gtk::Align::Start)
                                        .justify(gtk::Justification::Fill).build();

                                        list_item
                                            .downcast_ref::<gtk::ListItem>()
                                            .expect("Must be a list item")
                                            .set_child(Some(&label));

                                        list_item
                                            .property_expression("item")
                                            .chain_property::<gtk::StringObject>("string")
                                            .bind(&label, "label", gtk::Widget::NONE);
                                    }
                                }
                            }
                        }

                    },

                    append_page[Some(&gtk::Label::new(Some("Bible")))] = &gtk::Box {
                        set_orientation:gtk::Orientation::Vertical,
                        set_vexpand: true,
                        add_css_class: "blue_box",

                        // #[name="search_field"]
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 2,
                            set_height_request: 48,
                            add_css_class: "green_double_box",

                            gtk::SearchEntry {
                                set_placeholder_text: Some("Search..."),
                                set_hexpand: true
                            }
                        },

                        gtk::ScrolledWindow {
                            set_vexpand: true,

                            #[wrap(Some)]
                            set_child = &gtk::ListView {
                                #[wrap(Some)]
                                set_model = &gtk::SingleSelection{
                                    set_model: Some(&(0..1000).map(|_| LIST_VEC[0]).collect::<gtk::StringList>()),
                                },

                                #[wrap(Some)]
                                set_factory = &gtk::SignalListItemFactory {
                                    connect_setup => move |_, list_item|{
                                        let label = gtk::Label::builder()
                                        .ellipsize(gtk::pango::EllipsizeMode::End)
                                        .single_line_mode(true)
                                        .halign(gtk::Align::Start)
                                        .justify(gtk::Justification::Fill).build();

                                        list_item
                                            .downcast_ref::<gtk::ListItem>()
                                            .expect("Must be a list item")
                                            .set_child(Some(&label));

                                        list_item
                                            .property_expression("item")
                                            .chain_property::<gtk::StringObject>("string")
                                            .bind(&label, "label", gtk::Widget::NONE);
                                    }
                                }
                            }
                        }

                    },


                    append_page[Some(&gtk::Label::new(Some("Backgrounds")))] = &gtk::Box {
                        set_orientation:gtk::Orientation::Vertical,
                        set_vexpand: true,
                        add_css_class: "blue_box",

                        // #[name="search_field"]
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 2,
                            set_height_request: 48,
                            add_css_class: "green_double_box",

                            gtk::SearchEntry {
                                set_placeholder_text: Some("Search..."),
                                set_hexpand: true
                            }
                        },

                        gtk::ScrolledWindow {
                            set_vexpand: true,

                            #[wrap(Some)]
                            #[local_ref]
                            set_child = &bg_grid_view -> gtk::GridView {
                                connect_activate[sender] => move |grid_view, _|{
                                    let s_model = match grid_view.model() {
                                        Some(model)=>model,
                                        None=> return
                                    };

                                    let ss_model = match s_model.downcast_ref::<gtk::SingleSelection>() {
                                        Some(model)=>model,
                                        None=> return
                                    };

                                    let selected_pos = ss_model.selected();

                                    println!("bg activated => {}", selected_pos);

                                    let fullpath_image = match std::env::current_dir(){
                                        Ok(path)=>format!("{}/data/background/image.jpg", path.display()),
                                        Err(_)=>return,
                                    };

                                    // let abs_path = match std::path::absolute( std::path::Path::new(&fullpath_image)){
                                    //     Ok(path)=>path,
                                    //     Err(_)=>return
                                    // };

                                    let _ = sender.output(SearchOutput::PreviewBackground(fullpath_image.to_string()));
                                },
                                // #[wrap(Some)]
                                // set_model = &gtk::SingleSelection{
                                //     set_model: Some(&(0..1000).map(|_| LIST_VEC[0]).collect::<gtk::StringList>()),
                                // },
                                //
                                // #[wrap(Some)]
                                // set_factory = &gtk::SignalListItemFactory {
                                //     connect_setup => move |_, list_item|{
                                //         let label = gtk::Label::builder()
                                //         .ellipsize(gtk::pango::EllipsizeMode::End)
                                //         .single_line_mode(true)
                                //         .halign(gtk::Align::Start)
                                //         .justify(gtk::Justification::Fill).build();
                                //
                                //         list_item
                                //             .downcast_ref::<gtk::ListItem>()
                                //             .expect("Must be a list item")
                                //             .set_child(Some(&label));
                                //
                                //         list_item
                                //             .property_expression("item")
                                //             .chain_property::<gtk::StringObject>("string")
                                //             .bind(&label, "label", gtk::Widget::NONE);
                                //     }
                                // }
                            }
                        }

                    },
                }
            }

        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let mut background_grid_view: TypedGridView<BackgroundGridListItem, gtk::SingleSelection> =
            TypedGridView::new();

        for _i in 0..31 {
            background_grid_view.append(BackgroundGridListItem::new(
                "clean imageclean imageclean image".to_string(),
                "data/background/image.jpg".to_string(),
            ));
        }

        let bg_grid_view = background_grid_view.view;

        let model = SearchModel {
            image_src_list: Rc::new(RefCell::new(init.image_src_list)),
        };
        let widgets = view_output!();

        return relm4::ComponentParts { model, widgets };
    }
}

const LIST_VEC: [&str; 13] = [
    "Golden sun, a radiant masterpiece, paints the canvas of the morning sky. With hues of pink and softest blue, a breathtaking, ethereal sight,",
    "A gentle breeze, a whispered lullaby, carries softly through and through, Enveloping the world in calm as morning dew begins to fall anew.",
    "Dew-kissed flowers, adorned with sparkling gems, open wide to greet the day, Unfurling petals, soft and sweet, in a vibrant, colorful display,",
    "Nature's beauty, a masterpiece, unfolds before our wondering eyes,",
    "Inviting us to pause and breathe, beneath the endless, open skies.",
    "Children laugh, their joy infectious, as they chase their dreams so high,",
    "Imaginations soar and fly, reaching for the boundless sky,",
    "Hopeful wishes, like tiny stars, twinkle brightly in their hearts,",
    "As golden moments slip away, leaving precious, lasting marks.",
    "Hand in hand, we'll journey on, through life's winding, twisting road,",
    "With courage, strength, and hearts aflame, carrying hope's precious load,",
    "Brighter days, a promised land, await us just beyond the bend,",
    "As love and friendship's bonds endure, forever and without an end.",
];
