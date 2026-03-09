use gtk::{
    FileFilter,
    gio::{
        self,
        prelude::{FileExt, FileExtManual, ListModelExtManual},
    },
    glib,
};

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
            .and_then(|v| serde_json::from_str::<Vec<SlideManagerData>>(&v).ok())

        // NOTE: we will have to append a head before saving
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
}
