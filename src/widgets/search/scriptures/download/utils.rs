use futures_util::StreamExt;
use std::{cell::RefCell, io::Write, rc::Rc};

use rusqlite::Connection;

use crate::{
    config::AppConfigDir,
    db::{
        connection::{BibleTranslation, BibleVerse, DatabaseConnection},
        query::Query,
    },
};

use super::download_modal::BibleDownload;

pub async fn import_bible<F>(
    conn: Rc<RefCell<Option<DatabaseConnection>>>,
    bible: &BibleDownload,
    callback: F,
) -> Option<String>
where
    F: Fn(String),
{
    println!("SELCETIONS {:?}", bible);

    let download_url = match &bible.download_url {
        Some(d) => d,
        None => {
            eprintln!("ERROR: no download_url");
            return None;
        }
    };

    println!("DOWNLOAD STARTED");
    let response = reqwest::get(download_url).await.expect("Request error");
    // let response = reqwest::get(download_url).await.expect("Request error");

    let content_size = response
        .content_length()
        .expect("Error getting content size");
    let mut download_stream = response.bytes_stream();

    let mut downloaded_size: u64 = 0;
    let path_str = AppConfigDir::dir_path(AppConfigDir::DOWNLOADS);
    let file_path = path_str.join(bible.name.clone());
    let file = std::fs::File::create_new(&file_path);
    let mut file = match file {
        Ok(f) => f,
        Err(e) => {
            eprintln!("failed to create file {:?}", e);
            return None;
        }
    };

    while let Some(item) = download_stream.next().await {
        let chunk = item.expect("Error while downloading file");
        file.write_all(&chunk).expect("Error while writing to file");
        let percentage = ((downloaded_size as f64 / content_size as f64) * 100.0).round();

        println!(
            "DOWNLOAD PROGRESS \ndownloaded_size = {downloaded_size} \ntotal = {content_size} \npercentage = {}",
            percentage
        );

        callback(format!("Downloading {percentage}%"));

        downloaded_size = u64::min(downloaded_size + chunk.len() as u64, content_size);
    }

    callback("Installing...".to_string());

    let _db_conn = match Connection::open(&file_path) {
        Ok(conn) => conn,
        Err(e) => {
            println!("Error opening file: {:?}", e);
            return None;
        }
    };

    let translation_query = _db_conn.query_row(
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
    let translation: BibleTranslation = match translation_query {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "SQL ERROR: error getting downloaded translation info \n{:?}",
                e
            );
            return None;
        }
    };

    let translation_name = match bible.name.split(".").collect::<Vec<&str>>().first() {
        Some(name) => name.to_string(),
        None => {
            eprintln!("ERROR: failed to get file translation name");
            return None;
        }
    };

    let translation_verses_query = _db_conn.prepare(&format!(
        "SELECT id, book_id, chapter, verse, text FROM {}_verses",
        translation_name
    ));

    let mut verses_sql = match translation_verses_query {
        Ok(s) => s,
        Err(e) => {
            eprintln!("SQL ERROR: error getting downloaded verses \n{:?}", e);
            return None;
        }
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

    let bible_verse = match verses_query {
        Ok(a) => a,
        Err(e) => {
            eprintln!("SQL ERROR: error getting downloaded verses \n{:?}", e);
            return None;
        }
    };

    let mut verses_vec = Vec::new();
    for row in bible_verse {
        match row {
            Ok(r) => {
                // check if book is not part of the 66 books
                if r.1.book_id > 66 {
                    eprintln!("SQL ERROR: Book too large \n{:?}", verses_vec.first());
                    return None;
                }

                verses_vec.push(r);
            }
            Err(e) => {
                eprintln!("SQL ERROR: error extracting downloaded verses \n{:?}", e);
                return None;
            }
        };
    }

    let translation_name = translation.translation.clone();
    let res = Query::insert_verse(conn, translation, verses_vec);
    println!("INSERTING VERESES DONE: {:?}", res);

    if let Err(e) = std::fs::remove_file(&file_path) {
        eprintln!("FILE ERROR: error removing downloaded verses \n{:?}", e);
        return None;
    }

    callback("Done".to_string());

    Some(translation_name)
}
