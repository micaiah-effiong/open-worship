use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use futures_util::AsyncReadExt;
use futures_util::future::abortable;
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
    Instalation,
    Done(String),
}

struct FileCleanupGuard {
    path: PathBuf,
    success: bool,
}

impl Drop for FileCleanupGuard {
    fn drop(&mut self) {
        if !self.success {
            println!("Cleaning up partial file: {:?}", self.path);
            let _ = fs::remove_file(&self.path);
        }
    }
}

pub fn import_bible2<F>(bible: BibleDownload, callback: F) -> AbortHandle
where
    F: Fn(Result<ImportBibleStatus, ()>) + 'static,
{
    let callback = Arc::new(callback);
    let callback_clone = callback.clone();

    let (fut, abort_handle) = abortable(async move {
        callback(Ok(ImportBibleStatus::Init));

        let path = AppConfigDir::dir_path(AppConfigDir::Downloads).join(bible.name());

        let mut guard = FileCleanupGuard {
            path: path.clone(),
            success: false,
        };

        match fs::exists(path.clone()) {
            Ok(true) => {
                callback(Ok(ImportBibleStatus::Progress(100)));
                callback(Ok(ImportBibleStatus::Instalation));
                write_to_db(guard.path.clone(), &bible);
                guard.success = true;
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

        let mut file = match fs::File::create(&guard.path) {
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

        callback(Ok(ImportBibleStatus::Instalation));
        write_to_db(guard.path.clone(), &bible);

        guard.success = true;
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

    // if let Err(e) = std::fs::remove_file(&file_path) {
    //     eprintln!("FILE ERROR: error removing downloaded verses \n{:?}", e);
    //     return None;
    // }

    return Some(translation_name);
}
