use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use relm4::prelude::*;
use relm4::typed_view::grid::TypedGridView;

use crate::config::AppConfigDir;
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
    grid_view_wrapper: Rc<RefCell<TypedGridView<BackgroundGridListItem, gtk::SingleSelection>>>,
}

pub struct SearchBackgroundInit {}

impl SearchBackgroundModel {
    fn load_backgrounds() -> Vec<String> {
        let dir = match AppConfigDir::dir_path(AppConfigDir::Backgrounds).read_dir() {
            Ok(d) => d,
            Err(_) => {
                println!(
                    "ERROR: could not read {:?}",
                    AppConfigDir::to(AppConfigDir::Backgrounds)
                );
                return [].to_vec();
            }
        };

        let mut path_list = Vec::new();
        for entry in dir {
            let entry = match entry {
                Ok(f) => f,
                Err(_) => continue,
            };

            if let Ok(entry) = entry.metadata() {
                if !entry.is_file() {
                    continue;
                }
            }

            path_list.push(entry.path().display().to_string());
        }

        path_list
    }

    fn append_background(&mut self, bg: Vec<String>) {
        let mut view = self.grid_view_wrapper.borrow_mut();

        for path in bg {
            // create symlink
            let filename = match std::path::Path::new(&path).file_name() {
                Some(f_path) => f_path,
                None => continue,
            };

            let symlink_path = AppConfigDir::dir_path(AppConfigDir::Backgrounds).join(filename);
            println!("sym_path -> {:?}", symlink_path);

            match std::fs::hard_link(path, &symlink_path) {
                Ok(path) => path,
                Err(err) => {
                    eprintln!("Error creating sysmlink. {}", err);
                    return;
                }
            };

            let path = symlink_path.display().to_string();
            view.append(BackgroundGridListItem::new(path.clone(), None));
            self.image_src_list.borrow_mut().push(path);
        }
    }

    fn register_backgroud_chooser(
        window: &gtk::Window,
        sender: ComponentSender<SearchBackgroundModel>,
    ) -> gtk::FileDialog {
        let file_filter = gtk::FileFilter::new();
        file_filter.add_mime_type("image/*");
        file_filter.set_name(Some("Images"));

        let list_store = gtk::gio::ListStore::new::<gtk::FileFilter>();
        list_store.append(&file_filter);
        let filter_model = gtk::FilterListModel::new(Some(list_store), Some(file_filter));

        let fc = gtk::FileDialog::builder()
            .modal(true)
            .filters(&filter_model)
            .title("Import background")
            .build();

        fc.open_multiple(Some(window), gtk::gio::Cancellable::NONE, move |files| {
            println!("OPENING FILES");

            if files.is_err() {
                // TODO: log or inform user about error
                eprintln!("ERROR: Could not open files\n{:?}", files);
                return;
            }

            let files = files.unwrap();
            let mut new_images: Vec<String> = vec![];

            for file in files.iter::<gtk::gio::File>().flatten() {
                if let Some(path) = file.path() {
                    let filename = path.display().to_string();
                    new_images.push(filename);
                }
            }

            sender.input(SearchBackgroundInput::NewBackgroundImages(new_images));
        });

        fc
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

                        let file_chooser = SearchBackgroundModel::register_backgroud_chooser(&window,sender.clone());
                        // file_chooser.set_transient_for(Some(&window));
                        // file_chooser.show();
                    }
                },
                gtk::Button {
                    set_icon_name: "minus",
                    set_tooltip: "Remove background",

                    connect_clicked[sender, model] => move |_|{
                        let grid_view = model.grid_view_wrapper.borrow().view.clone();

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
        let grid_view_wrapper = Rc::new(RefCell::new(TypedGridView::new()));

        let bg_list = SearchBackgroundModel::load_backgrounds();
        for item in bg_list.clone() {
            grid_view_wrapper
                .borrow_mut()
                .append(BackgroundGridListItem::new(item, None));
        }

        let model = SearchBackgroundModel {
            image_src_list: Rc::new(RefCell::new(bg_list)),
            grid_view_wrapper,
        };

        let bg_grid_view = model.grid_view_wrapper.borrow().view.clone();

        let widgets = view_output!();
        relm4::ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            SearchBackgroundInput::NewBackgroundImages(list) => {
                self.append_background(list);
            }
            SearchBackgroundInput::RemoveBackgroundImages(position_list) => {
                for index in position_list.clone() {
                    let item_to_remove = self.grid_view_wrapper.clone().borrow().get(index);

                    if let Some(item) = item_to_remove {
                        let item = item.borrow().clone();

                        self.grid_view_wrapper.borrow_mut().remove(index);
                        let _ = std::fs::remove_file(&item.src);
                    }
                }
            }
        };
    }
}
