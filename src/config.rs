use std::{fs, path::PathBuf};

use crate::db;

const APP_DIR_NAME: &str = "openworship";
const APP_DATA_DIRS: [&str; 4] = ["backgrounds", "database", "media", "downloads"];

pub struct AppConfig {
    //
}

impl AppConfig {
    pub fn init() {
        AppConfig::setup_config_dir();
        AppConfig::setup_database();
    }
    ///  setup directories
    fn setup_config_dir() {
        let sys_config_dir = dirs::config_dir().expect("Could not get config directory");

        // create app data dir if not exist
        let app_config_path = sys_config_dir.join(APP_DIR_NAME);
        if !app_config_path.exists() {
            fs::create_dir_all(app_config_path.clone())
                .expect("An error occurred while creating app directory");
        }

        for dir in APP_DATA_DIRS {
            if AppConfigDir::from(dir.to_string()).is_none() {
                eprintln!("ERROR: Invalid File/Dir, name: {}", &dir);
                continue;
            }

            let err_msg = format!("ERROR: Could not create {:?}", app_config_path.join(dir));
            if !app_config_path.join(dir).exists() {
                fs::create_dir_all(app_config_path.join(dir)).expect(&err_msg);
            }
        }
    }

    fn setup_database() {
        let _ = db::connection::load_db(AppConfig::get_db_path()); //.expect("Error setting up database");
    }

    pub fn get_db_path() -> String {
        let db_path = AppConfigDir::dir_path(AppConfigDir::DATABASE)
            .join("db.sqlite")
            .display()
            .to_string();

        return db_path;
    }
}

pub enum AppConfigDir {
    DOWNLOADS,
    DATABASE,
    MEDIA,
    BACKGROUNDS,
}

impl AppConfigDir {
    pub fn from(val: String) -> Option<AppConfigDir> {
        match val.as_str() {
            "media" => Some(Self::MEDIA),
            "database" => Some(Self::DATABASE),
            "downloads" => Some(Self::DOWNLOADS),
            "backgrounds" => Some(Self::BACKGROUNDS),
            _ => None,
        }
    }

    pub fn to(val: AppConfigDir) -> String {
        match val {
            Self::MEDIA => String::from("media"),
            Self::DATABASE => String::from("database"),
            Self::DOWNLOADS => String::from("downloads"),
            Self::BACKGROUNDS => String::from("backgrounds"),
        }
    }

    pub fn dir_path(val: AppConfigDir) -> PathBuf {
        let sys_config_dir = dirs::config_dir()
            .expect("Could not get config directory")
            .join(APP_DIR_NAME);
        return sys_config_dir.join(AppConfigDir::to(val));
    }
}
