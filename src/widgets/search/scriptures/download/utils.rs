use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::{fmt, fs};

use anyhow::anyhow;
use futures_util::AsyncReadExt;
use futures_util::future::{Abortable, Future, abortable};
use futures_util::stream::AbortHandle;
use gtk::glib;
use rusqlite::Connection;

use crate::app_config::AppConfigDir;
use crate::db::connection::{BibleTranslation, BibleVerse};
use crate::db::query::Query;
use crate::widgets::search::scriptures::download::download_page::BibleDownload;

pub enum ImportBibleStatus {
    Init,
    Progress(u64),
    Installation,
    Done(String),
}

pub struct DownloadedBytes(u64);
impl fmt::Display for DownloadedBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format_bytes(self.0))
    }
}

pub struct TotalBytes(u64);
impl fmt::Display for TotalBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format_bytes(self.0))
    }
}

pub enum DownloadStatus {
    Init,
    Progress(u64, DownloadedBytes, TotalBytes),
    Done(PathBuf),
}

struct FileCleanupGuard {
    path: PathBuf,
    success: bool,
    temp: PathBuf,
}

impl FileCleanupGuard {
    fn new(path: PathBuf) -> Self {
        let temp_dir = std::env::temp_dir();

        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let name = format!("openworship-{}-{}", std::process::id(), nanos);
        let temp_path = temp_dir.join(name);

        FileCleanupGuard {
            path,
            success: false,
            temp: temp_path,
        }
    }
    fn mark_success(&mut self) {
        self.success = true;
    }
}

impl Drop for FileCleanupGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.temp);
        if !self.success {
            println!("Cleaning up partial file: {:?}", self.path);
            let _ = fs::remove_file(&self.path);
        }
    }
}

pub fn import_bible<F>(bible: BibleDownload, callback: F) -> AbortHandle
where
    F: Fn(Result<ImportBibleStatus, ()>) + 'static,
{
    let callback = Arc::new(callback);
    let callback_clone = callback.clone();

    let (fut, abort_handle) = abortable(async move {
        callback(Ok(ImportBibleStatus::Init));

        let path = AppConfigDir::dir_path(AppConfigDir::Downloads).join(bible.name());
        let mut guard = FileCleanupGuard::new(path);

        match fs::exists(guard.path.clone()) {
            Ok(true) => {
                callback(Ok(ImportBibleStatus::Progress(100)));
                callback(Ok(ImportBibleStatus::Installation));
                write_to_db(guard.path.clone(), &bible);
                guard.mark_success();
                callback(Ok(ImportBibleStatus::Done(bible.name())));
                return;
            }
            Ok(false) => (),
            Err(_) => {
                callback(Err(()));
                return;
            }
        }

        let mut response = match surf::get(bible.download_url()).await {
            Ok(r) => r,
            Err(_) => {
                callback(Err(()));
                return;
            }
        };

        let content_size = response
            .header("content-length")
            .and_then(|v| v.as_str().parse::<u64>().ok())
            .unwrap_or(0);

        let mut file = match fs::File::create(&guard.temp) {
            Ok(f) => f,
            Err(_) => {
                callback(Err(()));
                return;
            }
        };

        let mut downloaded: u64 = 0;
        let mut buffer = vec![0u8; 8192];

        loop {
            let len = match response.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => {
                    callback(Err(()));
                    return;
                }
            };

            if file.write_all(&buffer[..len]).is_err() {
                callback(Err(()));
                return;
            }

            downloaded += len as u64;
            if content_size > 0 {
                let percent = (downloaded * 100) / content_size;
                println!("DOWNLOAD PROGRESS = {}%", percent);
                callback(Ok(ImportBibleStatus::Progress(percent)));
            }
        }

        callback(Ok(ImportBibleStatus::Installation));
        if fs::rename(guard.temp.clone(), guard.path.clone()).is_err() {
            return callback(Err(()));
        };
        write_to_db(guard.path.clone(), &bible);

        guard.mark_success();
        callback(Ok(ImportBibleStatus::Done(bible.name())));
    });

    glib::spawn_future_local(async move {
        match fut.await {
            Ok(_) => println!("Download finished or stopped via return"),
            Err(_) => {
                println!("Download was INSTANTLY aborted");
                callback_clone(Err(()))
            }
        }
    });

    abort_handle
}

fn write_to_db(file_path: std::path::PathBuf, bible: &BibleDownload) -> Option<String> {
    let Ok(db_conn) =
        Connection::open(&file_path).map_err(|e| println!("Error opening file: {:?}", e))
    else {
        return None;
    };

    let translation_query = db_conn.query_row(
        "SELECT translation, title, license FROM translations",
        [],
        |r| {
            let bt = BibleTranslation {
                translation: r.get::<_, String>(0)?,
                title: r.get::<_, String>(1)?,
                license: r.get::<_, String>(2)?,
            };

            Ok(bt)
        },
    );
    let Ok(bible_translation) = translation_query.map_err(|e| eprintln!("SQL ERROR: \n{:?}", e))
    else {
        return None;
    };

    let translation_name = bible.name();
    let translation_verses_query = db_conn.prepare(&format!(
        "SELECT id, book_id, chapter, verse, text FROM {}_verses",
        translation_name
    ));

    let Ok(mut verses_sql) = translation_verses_query
        .map_err(|e| eprintln!("SQL ERROR: error getting downloaded verses \n{:?}", e))
    else {
        return None;
    };

    let verses_query = verses_sql.query_map([], |r| {
        let bv = (
            r.get::<_, u32>(0)?, // id
            BibleVerse {
                book: "".to_string(),
                book_id: r.get::<_, u32>(1)?, // book_id
                chapter: r.get::<_, u32>(2)?, // chapter
                verse: r.get::<_, u32>(3)?,   // verse
                text: r.get::<_, String>(4)?, // text
            },
        );

        Ok(bv)
    });

    let Ok(bible_verse) =
        verses_query.map_err(|e| eprintln!("SQL ERROR: error getting downloaded verses \n{:?}", e))
    else {
        return None;
    };

    let mut verses_vec = Vec::new();
    for row in bible_verse {
        let Ok(r) =
            row.map_err(|e| eprintln!("SQL ERROR: error extracting downloaded verses \n{:?}", e))
        else {
            return None;
        };
        verses_vec.push(r);
    }

    let translation_name = bible_translation.translation.clone();
    let res = Query::insert_verse(bible_translation, verses_vec);
    println!("INSERTING VERESES DONE: {:?}", res);

    Some(translation_name)
}

pub fn download_something<F>(
    url: String,
    path: PathBuf,
    callback: F,
) -> (
    Abortable<impl Future<Output = Result<(), anyhow::Error>>>,
    AbortHandle,
)
where
    F: Fn(DownloadStatus),
{
    let a = async move {
        callback(DownloadStatus::Init);
        let mut guard = FileCleanupGuard::new(path);

        match fs::exists(&guard.path) {
            Ok(true) => {
                // callback(DownloadStatus::Progress(100));
                callback(DownloadStatus::Done(guard.path.clone()));
                guard.mark_success();
                return Ok::<(), anyhow::Error>(());
            }
            Ok(false) => (),
            Err(e) => {
                return Err(anyhow::Error::from(e));
            }
        }

        let client = surf::client().with(surf::middleware::Redirect::default());
        let mut response = match client.get(url).await {
            Ok(r) if !r.status().is_success() => {
                return Err(anyhow!("Request failed with status: {}", r.status()));
            }
            Ok(r) => r,
            Err(e) => {
                return Err(anyhow!("{}", e));
            }
        };

        let content_size = response
            .header("content-length")
            .and_then(|v| v.as_str().parse::<u64>().ok())
            .unwrap_or(0);

        let mut file = match fs::File::create(&guard.temp) {
            Ok(f) => f,
            Err(e) => {
                return Err(anyhow::Error::from(e));
            }
        };

        let mut downloaded: u64 = 0;
        let mut buffer = vec![0u8; 8192];

        loop {
            let len = match response.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => {
                    return Err(anyhow::Error::from(e));
                }
            };

            if let Err(e) = file.write_all(&buffer[..len]) {
                return Err(anyhow::Error::from(e));
            }

            downloaded += len as u64;
            if content_size > 0 {
                let percent = (downloaded * 100) / content_size;
                callback(DownloadStatus::Progress(
                    percent,
                    DownloadedBytes(downloaded),
                    TotalBytes(content_size),
                ));
            }
        }

        if downloaded < content_size {
            return Err(anyhow!(
                "downloaded ({downloaded}) and content size ({content_size}) mismatch "
            ));
        }

        if let Err(e) = fs::rename(&guard.temp, &guard.path) {
            return Err(anyhow::Error::from(e));
        };

        callback(DownloadStatus::Done(guard.path.clone()));
        guard.mark_success();

        Ok(())
    };

    abortable(a)
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];

    let mut value = bytes as f64;
    let mut unit = 0;

    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    if value.fract() == 0.0 {
        format!("{} {}", value as u64, UNITS[unit])
    } else {
        format!("{:.1} {}", value, UNITS[unit])
    }
}

// NOTE: if you want to be fancy add resumeable download
