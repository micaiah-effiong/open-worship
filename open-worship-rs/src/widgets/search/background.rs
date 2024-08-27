use std::{cell::RefCell, rc::Rc, usize};

use gtk::{glib::clone, prelude::*};
use relm4::{prelude::*, typed_view::grid::TypedGridView};

use crate::structs::background_grid_list_item::BackgroundGridListItem;

// search area (notebook)
#[derive(Debug)]
pub enum SearchBackgroundInput {
    NewBackgroundImages(Vec<String>),
    RemoveBackgroundImages(Vec<u32>),
}

#[derive(Debug)]
pub enum SearchBacgroundOutput {
    SendPreviewBackground(String),
}

#[derive(Debug, Clone)]
pub struct SearchBackgroundModel {
    image_src_list: Rc<RefCell<Vec<String>>>,
    view: Rc<RefCell<TypedGridView<BackgroundGridListItem, gtk::SingleSelection>>>,
}

pub struct SearchBackgroundInit {
    // pub image_src_list: Vec<String>,
}

impl SearchBackgroundModel {
    fn join_path(path: std::path::PathBuf) -> std::path::PathBuf {
        return path.join("data").join("background");
    }

    fn load_backgrounds() -> Vec<String> {
        let cwd = match std::env::current_dir() {
            Ok(cw) => SearchBackgroundModel::join_path(cw),
            Err(_) => {
                panic!("Could not get current working directory");
            }
        };

        let dir = cwd.read_dir().expect("read_dir call failed");
        println!("dir {:?}", dir);

        let mut path_list = Vec::new();
        for entry in dir {
            if let Ok(entry) = entry {
                let path = entry.path();
                path_list.push(path.display().to_string());
            }
        }

        println!("load dir {:?}", &path_list);
        return path_list;
    }

    fn append_background(&mut self, bg: Vec<String>) {
        let mut view = self.view.borrow_mut();
        let mut list = self.image_src_list.borrow_mut();

        for path in bg {
            // create symlink
            let filename = match std::path::Path::new(&path).file_name() {
                Some(f_path) => f_path,
                None => continue,
            };
            let cwd = match std::env::current_dir() {
                Ok(cwd) => cwd.display().to_string(),
                Err(_) => continue,
            };

            let f = std::path::Path::new(&cwd);
            let sym_path = SearchBackgroundModel::join_path(f.to_path_buf()).join(filename);
            println!("sym_path -> {:?}", sym_path);

            match std::fs::hard_link(path, &sym_path) {
                Ok(path) => path,
                Err(err) => {
                    eprintln!("Error creating sysmlink. {}", err);
                    return;
                }
            };

            let path = sym_path.display().to_string();
            view.append(BackgroundGridListItem::new(path.clone(), None));
            list.push(path);
        }
    }

    fn register_backgroud_chooser(
        sender: ComponentSender<SearchBackgroundModel>,
    ) -> gtk::FileChooserDialog {
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
                        new_images.push(path.display().to_string());
                    }
                }

                sender.input(SearchBackgroundInput::NewBackgroundImages(new_images));

                f.close();
            }
        ));

        return fc;
    }
}

#[relm4::component(pub)]
impl SimpleComponent for SearchBackgroundModel {
    type Init = SearchBackgroundInit;
    type Output = SearchBacgroundOutput;
    type Input = SearchBackgroundInput;

    view! {
        #[root]
        gtk::Box {
            set_orientation:gtk::Orientation::Vertical,
            set_vexpand: true,
            add_css_class: "blue_box",

            // #[name="search_field"]
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 2,
                set_height_request: 48,
                set_width_request: 58,
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
                    set_min_columns: 2,
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

                        let _ = sender.output(SearchBacgroundOutput::SendPreviewBackground(path.to_string()));
                    },
                }
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 3,
                gtk::Button {
                    set_icon_name: "plus",
                    set_tooltip: "Add background",

                    connect_clicked[sender] => move |btn|{
                        let window = match btn.toplevel_window(){
                            Some(win)=>win,
                            None=>return
                        };

                        let file_chooser = SearchBackgroundModel::register_backgroud_chooser(sender.clone());
                        file_chooser.set_transient_for(Some(&window));
                        file_chooser.show();
                    }
                },
                gtk::Button {
                    set_icon_name: "minus",
                    set_tooltip: "Remove background",

                    connect_clicked[sender, model] => move |_|{
                        let grid_view = model.view.borrow().view.clone();

                        let s_model = match grid_view.model() {
                            Some(model)=>model,
                            None=> return
                        };

                        let ss_model = match s_model.downcast_ref::<gtk::SingleSelection>() {
                            Some(model)=>model,
                            None=> return
                        };

                        let selected_pos = ss_model.selected();
                        sender.input(SearchBackgroundInput::RemoveBackgroundImages(vec![selected_pos]));

                    }
                }
            },

        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let view = Rc::new(RefCell::new(TypedGridView::new()));

        let bg_list = SearchBackgroundModel::load_backgrounds();
        for item in bg_list.clone() {
            view.borrow_mut()
                .append(BackgroundGridListItem::new(item, None));
        }

        let model = SearchBackgroundModel {
            image_src_list: Rc::new(RefCell::new(bg_list)),
            view,
        };

        let bg_grid_view = model.view.borrow().view.clone();

        let widgets = view_output!();
        return relm4::ComponentParts { model, widgets };
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            SearchBackgroundInput::NewBackgroundImages(list) => {
                self.append_background(list);
            }
            SearchBackgroundInput::RemoveBackgroundImages(position_list) => {
                for index in position_list.clone() {
                    let removed_item = self.image_src_list.borrow_mut().remove(index as usize);
                    self.view.borrow_mut().remove(index);

                    let _ = std::fs::remove_file(removed_item);
                }
            }
        };
    }
}
