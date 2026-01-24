use std::{fs, path::PathBuf};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::db;

const APP_DIR_NAME: &str = "openworship";

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

        for dir in AppConfigDir::iter() {
            let path: String = dir.into();

            let err_msg = format!("ERROR: Could not create {:?}", app_config_path.join(&path));
            if !app_config_path.join(&path).exists() {
                fs::create_dir_all(app_config_path.join(path)).expect(&err_msg);
            }
        }
    }

    fn setup_database() {
        db::connection::load_db();
    }

    pub fn get_db_path() -> String {
        let db_path = AppConfigDir::dir_path(AppConfigDir::Database)
            .join("db.sqlite")
            .display()
            .to_string();

        db_path
    }
}

#[derive(Debug, EnumIter)]
pub enum AppConfigDir {
    Downloads,
    Database,
    Media,
    SlideMedia,
    Backgrounds,
}

impl Into<String> for AppConfigDir {
    fn into(self) -> String {
        match self {
            Self::Media => String::from("media"),
            Self::Database => String::from("database"),
            Self::Downloads => String::from("downloads"),
            Self::Backgrounds => String::from("backgrounds"),
            Self::SlideMedia => String::from("slide_media"),
        }
    }
}
impl TryFrom<String> for AppConfigDir {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "media" => Ok(Self::Media),
            "database" => Ok(Self::Database),
            "downloads" => Ok(Self::Downloads),
            "backgrounds" => Ok(Self::Backgrounds),
            "slide_media" => Ok(Self::SlideMedia),
            _ => Err(()),
        }
    }
}
impl AppConfigDir {
    pub fn dir_path(val: AppConfigDir) -> PathBuf {
        let sys_config_dir = dirs::config_dir()
            .expect("Could not get config directory")
            .join(APP_DIR_NAME);

        let path: String = val.into();
        sys_config_dir.join(path)
    }
}
