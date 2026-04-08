use std::{
    cell::RefCell,
    collections::{HashSet, VecDeque},
    fs,
};

use futures_util::TryFutureExt;
use gtk::{
    FileFilter, gdk,
    gio::{
        self,
        prelude::{FileExt, FileExtManual, ListModelExtManual},
    },
    glib,
};
use sha2::{Digest, Sha256};

use crate::{
    app_config::{self, AppConfigDir},
    widgets::canvas::serialise::SlideManagerData,
};

mod imp {

    use gtk::glib::{
        self,
        subclass::{object::ObjectImpl, types::ObjectSubclass},
    };

    #[derive(Default)]
    pub struct FileManager {}

    #[glib::object_subclass]
    impl ObjectSubclass for FileManager {
        const NAME: &'static str = "FileManager";
        type Type = super::FileManager;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for FileManager {}
}

glib::wrapper! {
    pub struct FileManager(ObjectSubclass<imp::FileManager>);
}

impl FileManager {
    fn save_user_file(
        title: String,
        accept_button_label: String,
        filters: &mut glib::List<gtk::FileFilter>,
        window: Option<&gtk::Window>,
        data: &[u8],
    ) -> Option<gio::File> {
        let has_all = filters.iter().any(|f| f.name() == Some("All Files".into()));
        if !has_all {
            let all_files = FileFilter::new();
            all_files.set_name(Some("All Files"));
            all_files.add_pattern("*");
            filters.push_back(all_files);
        }

        let mut list_store = gtk::gio::ListStore::new::<gtk::FileFilter>();
        list_store.extend(filters);

        let filter_model = gtk::FilterListModel::new(Some(list_store), None::<FileFilter>);

        let dialog = gtk::FileDialog::builder()
            .modal(true)
            .accept_label(accept_button_label)
            .filters(&filter_model)
            .title(title)
            .build();

        let ctx = glib::MainContext::default();
        ctx.block_on(async move {
            let res = dialog.save_future(window).await;
            if let Ok(user_file) = &res {
                Self::create_file_if_not_exists(user_file);

                let Some(path) = user_file.path() else {
                    return None;
                };

                std::fs::write(path, data).expect("failed to write file");
            }

            let res = res.inspect_err(|e| eprintln!("Error opening file in dialog: {:?}", e));
            res.ok()
        })
    }
    fn get_file_from_user(
        title: String,
        accept_button_label: String,
        filters: &mut glib::List<gtk::FileFilter>,
        window: Option<&gtk::Window>,
    ) -> Option<gio::File> {
        let has_all = filters.iter().any(|f| f.name() == Some("All Files".into()));
        if !has_all {
            let all_files = FileFilter::new();
            all_files.set_name(Some("All Files"));
            all_files.add_pattern("*");
            filters.push_back(all_files);
        }

        let mut list_store = gtk::gio::ListStore::new::<gtk::FileFilter>();
        list_store.extend(filters);

        let filter_model = gtk::FilterListModel::new(Some(list_store), None::<FileFilter>);

        let dialog = gtk::FileDialog::builder()
            .modal(true)
            .accept_label(accept_button_label)
            .filters(&filter_model)
            .title(title)
            .build();

        let ctx = glib::MainContext::default();
        ctx.block_on(async move {
            let res = dialog.open_future(window).await;
            let res = res.inspect_err(|e| eprintln!("Error opening file in dialog: {:?}", e));
            res.ok()
        })
    }

    fn get_multiple_files_from_user(
        title: String,
        accept_button_label: String,
        filters: &mut glib::List<gtk::FileFilter>,
        window: Option<&gtk::Window>,
    ) -> gio::ListModel {
        let has_all = filters.iter().any(|f| f.name() == Some("All Files".into()));
        if !has_all {
            let all_files = FileFilter::new();
            all_files.set_name(Some("All Files"));
            all_files.add_pattern("*");
            filters.push_back(all_files);
        }

        let mut list_store = gtk::gio::ListStore::new::<gtk::FileFilter>();
        list_store.extend(filters);

        let filter_model = gtk::FilterListModel::new(Some(list_store), None::<FileFilter>);

        let dialog = gtk::FileDialog::builder()
            .modal(true)
            .accept_label(accept_button_label)
            .filters(&filter_model)
            .title(title)
            .build();

        let ctx = glib::MainContext::default();
        ctx.block_on(async move {
            let res = dialog.open_multiple_future(window).await;
            let res = res.inspect_err(|e| eprintln!("Error opening file in dialog: {:?}", e));
            res.ok()
        })
        .unwrap_or(gio::ListStore::new::<gio::File>().into())
    }
    pub fn open_image() -> Option<gio::File> {
        let mut filters: glib::List<gtk::FileFilter> = glib::List::new();
        let filter = gtk::FileFilter::new();
        filter.set_name(Some("Images"));
        filter.add_mime_type("image/*");

        filters.push_back(filter);

        FileManager::get_file_from_user(
            String::from("Open Image"),
            String::from("Open"),
            &mut filters,
            None,
        )
    }
    pub fn open_files(
        title: &str,
        accept_button_label: &str,
        filters: &mut glib::List<gtk::FileFilter>,
    ) -> gio::ListModel {
        FileManager::get_multiple_files_from_user(
            title.to_string(),
            accept_button_label.to_string(),
            filters,
            None,
        )
    }

    pub fn open_schedule_file() -> Option<Vec<SlideManagerData>> {
        let mut filters = glib::List::new();
        let filter = gtk::FileFilter::new();
        filter.set_name(Some("Openworship schedule file"));
        filter.add_pattern(app_config::APP_EXT);
        filters.push_back(filter);

        let files = Self::open_files("Open Schedule", "Ok", &mut filters);
        files
            .iter::<gio::File>()
            .flatten()
            .next()
            .as_ref()
            .and_then(FileManager::get_data)
            .and_then(|v| String::from_utf8(v).ok())
            .and_then(|v| Self::parse_schedule_file(v))

        // NOTE: we will have to append a head before saving
    }

    pub fn parse_schedule_file(data: String) -> Option<Vec<SlideManagerData>> {
        let mut payload = match serde_json::from_str::<Vec<SlideManagerData>>(&data) {
            Ok(data) => data,
            Err(e) => {
                glib::g_warning!("FileManager", "serder error: {:?}", e);
                return None;
            }
        };

        //
        let mut images = HashSet::new();
        let mut decom = |v: &mut SlideManagerData| {
            for slide in v.slides.iter_mut() {
                let Some(bg) = slide.canvas_data.background_pattern.clone() else {
                    slide.canvas_data.background_pattern = None;
                    continue;
                };

                if bg.is_empty() {
                    slide.canvas_data.background_pattern = None;
                    continue;
                };

                let bg64 = bg.split(&[':', ';', ','][..]).collect::<Vec<_>>();

                let content_type = if let Some(content_type) = bg64.get(1)
                    && content_type.contains("image")
                {
                    content_type.replace("image/", "")
                } else {
                    slide.canvas_data.background_pattern = None;
                    continue;
                };
                let Some(content) = bg64.get(3).map(|v| v.to_string()) else {
                    slide.canvas_data.background_pattern = None;
                    continue;
                };

                let checksum = hex::encode(Sha256::digest(content.as_bytes()));
                let mut path = AppConfigDir::dir_path(AppConfigDir::SlideMedia);
                path.push(format!("{checksum}.{content_type}"));
                slide.canvas_data.background_pattern = Some(path.display().to_string());

                images.insert((path, content));
            }
        };

        for i in payload.iter_mut() {
            decom(i);
        }

        for (path, content) in images {
            let bytes = glib::base64_decode(&content);
            if std::path::Path::new(&path).exists() {
                glib::g_warning!("FileManager", "file already exists");
                continue;
            };

            let _ = fs::write(path, bytes).map_err(|e| {
                glib::g_warning!("FileManager", "Error: writing slide image: {:?}", e)
            });
        }
        //

        Some(payload)
    }

    pub fn save_file(
        title: &str,
        accept_button_label: &str,
        filters: &mut glib::List<gtk::FileFilter>,
        data: &[u8],
    ) -> Option<gio::File> {
        FileManager::save_user_file(
            title.to_string(),
            accept_button_label.to_string(),
            filters,
            None,
            data,
        )
    }

    pub fn save_schedule_file(payload: Vec<SlideManagerData>) {
        if payload.is_empty() {
            return;
        };

        // NOTE: very inefficient
        // alllow for duplicate base64 images across slides
        let mut payload = payload;
        for item in &mut payload {
            for slide in &mut item.slides {
                if let Some(bg) = &slide.canvas_data.background_pattern {
                    let p = std::path::Path::new(bg);
                    let file = gio::File::for_path(p);
                    let Some(content_type) =
                        gio::content_type_get_mime_type(&gio::content_type_guess(Some(p), &[]).0)
                    else {
                        return;
                    };
                    let Some(b64) = Self::file_to_base64(&file) else {
                        return;
                    };
                    let image_b64 = format!("data:{content_type};base64,{b64}");
                    slide.canvas_data.background_pattern = Some(image_b64);
                }
            }
        }

        let mut filters = glib::List::new();
        let filter = gtk::FileFilter::new();
        filter.set_name(Some("Openworship schedule file"));
        filter.add_pattern(app_config::APP_EXT);
        filters.push_back(filter);

        let mut list_store = gtk::gio::ListStore::new::<gtk::FileFilter>();
        list_store.extend(filters);

        let filter_model = gtk::FilterListModel::new(Some(list_store), None::<FileFilter>);

        let dialog = gtk::FileDialog::builder()
            .modal(true)
            // .accept_label("Save")
            .filters(&filter_model)
            .title("Save Schedule")
            .build();

        let ctx = glib::MainContext::default();
        ctx.block_on(async move {
            let res = dialog.save_future(None::<&gtk::Window>).await;

            let res = res.inspect_err(|e| eprintln!("Error opening file in dialog: {:?}", e));
            let user_file = match res {
                Ok(f) => f,
                Err(e) => {
                    println!("E: {:?}", e);
                    return;
                }
            };
            Self::create_file_if_not_exists(&user_file);

            let Some(path) = user_file.path() else {
                return;
            };

            let content = match serde_json::to_string(&payload) {
                Ok(c) => c,
                Err(e) => {
                    println!("Err: {:?}", e);
                    return;
                }
            };
            let mut data = Vec::new();
            // data.extend_from_slice(MAGIC_HEADER);
            data.extend_from_slice(content.as_bytes());
            std::fs::write(path, data).expect("failed to write file");
        });

        // NOTE: if file has head
        // we will have to verify the header validity and then skip forward
        // to file content
    }

    pub fn get_data(file: &gio::File) -> Option<Vec<u8>> {
        file.load_contents(None::<&gio::Cancellable>)
            .map(|(bytes, _)| bytes.to_vec())
            .map_err(|err| {
                glib::g_log!(
                    "FileManager",
                    glib::LogLevel::Warning,
                    "Could not read file contents: {}",
                    err
                );
            })
            .ok()
    }

    pub fn file_to_base64(file: &gio::File) -> Option<String> {
        let Some(bytes) = Self::get_data(file) else {
            return None;
        };

        return Some(glib::base64_encode(bytes.as_slice()).to_string());
    }

    pub fn base64_to_file(filename: &str, base64_data: String) -> String {
        let data = glib::base64_decode(&base64_data);
        match gio::File::for_path(&filename).replace_contents(
            &data,
            None,
            false,
            gio::FileCreateFlags::NONE,
            gio::Cancellable::NONE,
        ) {
            Ok(a) => a,
            Err(e) => {
                glib::g_log!(
                    "FileManager",
                    glib::LogLevel::Warning,
                    "Could not save data to file: {}",
                    e
                );

                return String::new();
            }
        };

        String::from(filename)
    }

    pub fn file_to_link(file: &gio::File, dir: AppConfigDir) -> Option<String> {
        if !file.query_exists(gio::Cancellable::NONE) {
            return None;
        };

        let Some(url) = file
            .clone()
            .path()
            .as_ref()
            .and_then(|v| v.to_str())
            .map(|s| s.to_string())
        else {
            return None;
        };

        let Some(filename) = std::path::Path::new(&url).file_name() else {
            return None;
        };

        let symlink_path = AppConfigDir::dir_path(dir).join(filename);
        let path = symlink_path.display().to_string();

        if symlink_path.exists() {
            return Some(path);
        }

        match std::fs::hard_link(url.clone(), &symlink_path) {
            Ok(path) => path,
            Err(err) => {
                eprintln!("Error creating sysmlink. {}", err);
                return None;
            }
        };

        Some(path)
    }

    pub fn create_file_if_not_exists(file: &gio::File) {
        if !file.query_exists(gio::Cancellable::NONE) {
            match file.create(
                gio::FileCreateFlags::REPLACE_DESTINATION,
                gio::Cancellable::NONE,
            ) {
                Ok(_) => (),
                Err(e) => eprintln!("Could not write file: {:?}", e),
            };
        }
    }

    pub fn load_backgrounds() -> Vec<String> {
        let mut path_list = Vec::new();
        let dir = match AppConfigDir::dir_path(AppConfigDir::Backgrounds).read_dir() {
            Ok(d) => d,
            Err(e) => {
                println!(
                    "ERROR: could not read {:?} = {:?}",
                    AppConfigDir::Backgrounds,
                    e
                );
                return path_list;
            }
        };

        for entry in dir {
            let file_entry = match entry {
                Ok(f) => f,
                Err(_) => continue,
            };

            match file_entry.metadata() {
                Ok(entry) if entry.is_file() => (),
                _ => continue,
            };

            let guess = gio::content_type_guess(Some(file_entry.path()), &[]);
            if let Some(mime_type) = gio::content_type_get_mime_type(&guess.0)
                && !mime_type.contains("image")
            {
                continue;
            };

            path_list.push(file_entry.path().display().to_string());
        }

        path_list
    }

    pub fn get_background_image<F: FnOnce(Option<gdk::Texture>) + 'static>(
        path: &std::path::Path,
        size: Option<(i32, i32)>,
        cb: F,
    ) {
        let path = path.display().to_string();
        let texture = LOADED_BACKGROUND_IMAGES.with_borrow_mut({
            let path = path.clone();
            move |v| {
                if let Some(item) = v
                    .iter()
                    .position(|(s, _)| *s == path)
                    .and_then(|p| v.remove(p))
                {
                    v.push_back(item.clone());
                    return Some(item.1);
                }

                None
            }
        });

        if texture.is_some() {
            cb(texture);
            return;
        }

        glib::spawn_future_local(async move {
            let texture = gio::spawn_blocking({
                let path = path.clone();
                let size = size.clone().unwrap_or((1920, 1080));
                move || {
                    let pixbuf =
                        gtk::gdk_pixbuf::Pixbuf::from_file_at_scale(&path, size.0, size.1, true)
                            .map_err(|e| glib::g_warning!("FileManager", "Failed to load: {:?}", e))
                            .ok()?;
                    Some(gdk::Texture::for_pixbuf(&pixbuf))
                }
            })
            .map_err(|e| glib::g_warning!("FileManager", "Error in spawn_blocking: {:?}", e))
            .await
            .ok();

            if let Some(texture) = texture {
                LOADED_BACKGROUND_IMAGES.with_borrow_mut(|v| {
                    if size.is_some() {
                        return;
                    };
                    let Some(texture) = texture.clone() else {
                        return;
                    };
                    if v.len() >= 5 {
                        v.pop_front();
                    }
                    v.push_back((path, texture));
                });
                cb(texture);
            }
        });
    }
}

thread_local! {
    static  LOADED_BACKGROUND_IMAGES:RefCell<VecDeque<(String, gdk::Texture)>> = RefCell::new(VecDeque::with_capacity(5));
}
