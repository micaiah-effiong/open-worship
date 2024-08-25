use std::{cell::RefCell, rc::Rc, usize};

use gtk::{glib::clone, prelude::*};
use relm4::{prelude::*, typed_view::grid::TypedGridView};

use crate::structs::background_grid_list_item::BackgroundGridListItem;

const MIN_GRID_HEIGHT: i32 = 300;
// const MIN_GRID_WIDTH: i32 = 300;

// search area (notebook)
#[derive(Debug)]
pub enum SearchInput {
    NewBackgroundImages(Vec<String>),
}

#[derive(Debug)]
pub enum SearchOutput {
    PreviewBackground(String),
}

#[derive(Debug, Clone)]
pub struct SearchModel {
    image_src_list: Rc<RefCell<Vec<String>>>,
    view: Rc<RefCell<TypedGridView<BackgroundGridListItem, gtk::SingleSelection>>>,
}

pub struct SearchInit {
    pub image_src_list: Vec<String>,
}

impl SearchModel {
    fn append_background(&mut self, bg: Vec<String>) {
        let mut view = self.view.borrow_mut();
        let mut list = self.image_src_list.borrow_mut();

        for path in bg {
            view.append(BackgroundGridListItem::new(path.clone(), None));
            list.push(path);
        }
    }

    fn register_backgroud_chooser(sender: ComponentSender<SearchModel>) -> gtk::FileChooserDialog {
        let file_filter = gtk::FileFilter::new();
        file_filter.add_mime_type("image/png");
        file_filter.add_mime_type("image/jpeg");

        let fc = gtk::FileChooserDialog::builder()
            .select_multiple(true)
            .maximized(false)
            .modal(true)
            .title("Import background")
            .action(gtk::FileChooserAction::Open)
            .filter(&file_filter)
            .build();

        fc.add_button("Open", gtk::ResponseType::Ok);
        fc.add_button("Cancel", gtk::ResponseType::Cancel);

        fc.connect_response(clone!(
            @strong sender,
            => move |f, r| {
                let list = match r {
                    gtk::ResponseType::Ok => f.files(),
                    gtk::ResponseType::Cancel => {
                        f.close();
                        return;
                    }
                    _ => return,
                };

                let mut new_images:Vec<String> = vec![];

                for item in &list {
                    if item.is_err() {
                        continue;
                    }

                    let file = match item.unwrap().downcast::<gtk::gio::File>() {
                        Ok(file) => file,
                        Err(_) => continue,
                    };

                    println!("file -> {:?}", &file.path());
                    if let Some(path) = file.path() {
                        // print!("{}", path.display().to_string());
                        new_images.push(path.display().to_string());
                    }
                }

                sender.input(SearchInput::NewBackgroundImages(new_images));

                f.close();
            }
        ));

        return fc;
    }
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
                                connect_activate[sender, model] => move |grid_view, _| {
                                    let s_model = match grid_view.model() {
                                        Some(model)=>model,
                                        None=> return
                                    };

                                    let ss_model = match s_model.downcast_ref::<gtk::SingleSelection>() {
                                        Some(model)=>model,
                                        None=> return
                                    };

                                    let selected_pos = ss_model.selected();
                                    let list = model.image_src_list.borrow();
                                    let path = list.get(selected_pos as usize);

                                    let path = match path{
                                        Some(path)=>path,
                                        None=>return,
                                    };

                                    let _ = sender.output(SearchOutput::PreviewBackground(path.to_string()));
                                },
                            }
                        },

                        gtk::Box {
                            gtk::Button {
                                set_icon_name: "plus",
                                set_tooltip: "Add background",

                                connect_clicked[sender] => move |btn|{
                                    let window = match btn.toplevel_window(){
                                        Some(win)=>win,
                                        None=>return
                                    };

                                    let file_chooser = SearchModel::register_backgroud_chooser(sender.clone());
                                    file_chooser.set_transient_for(Some(&window));
                                    file_chooser.show();
                                }
                            }
                        },

                    }
                }
            }

        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = SearchModel {
            image_src_list: Rc::new(RefCell::new(Vec::new())),
            view: Rc::new(RefCell::new(TypedGridView::new())),
        };

        let bg_grid_view = model.view.borrow().view.clone();

        let widgets = view_output!();

        return relm4::ComponentParts { model, widgets };
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            SearchInput::NewBackgroundImages(list) => {
                self.append_background(list);
            }
        };
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
