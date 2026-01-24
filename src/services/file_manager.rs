use gtk::{
    FileFilter,
    gio::{
        self,
        prelude::{FileExt, FileExtManual},
    },
    glib,
};

use crate::config::AppConfigDir;

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
    fn get_file_from_user(
        title: String,
        accept_button_label: String,
        filters: &mut glib::List<gtk::FileFilter>,
        window: Option<&gtk::Window>,
        chooser_action: gtk::FileChooserAction,
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
            let res = match chooser_action {
                gtk::FileChooserAction::Open => dialog.open_future(window).await,
                gtk::FileChooserAction::Save => dialog.save_future(window).await,
                _ => return None,
            };

            let res = res.inspect_err(|e| eprintln!("Error opening file in dialog: {:?}", e));
            res.ok()
        })
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
            gtk::FileChooserAction::Open,
        )
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
}
