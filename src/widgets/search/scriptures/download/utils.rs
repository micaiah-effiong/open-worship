use futures_util::AsyncReadExt;
use gtk::glib;
use std::io::Write;

use rusqlite::Connection;

use crate::{
    app_config::AppConfigDir,
    db::{
        connection::{BibleTranslation, BibleVerse},
        query::Query,
    },
    widgets::search::scriptures::download::download_page::BibleDownload,
};

pub async fn import_bible<F>(bible: &BibleDownload, callback: F) -> Option<String>
where
    F: Fn(Result<String, ()>),
{
    println!("SELCETIONS {:?}", bible);
    println!("URL {:?}", bible.download_url());

    let download_url = bible.download_url();

    let client = surf::Client::new();
    let req = surf::get(download_url).build();
    let mut response = match client.send(req).await {
        Ok(r) => r,
        Err(e) => {
            glib::g_critical!("Import Bible", "{:?}", e);
            callback(Err(()));
            return None;
        }
    };

    let content_size = response
        .header("content-length")
        .and_then(|v| v.as_str().parse::<u64>().ok())
        .unwrap_or(0);

    let mut downloaded_size: u64 = 0;
    let path_str = AppConfigDir::dir_path(AppConfigDir::Downloads);
    let file_path = path_str.join(bible.name());
    let file = std::fs::File::create_new(&file_path);
    let mut file = match file {
        Ok(f) => f,
        Err(e) => {
            eprintln!("failed to create file {:?}", e);
            return None;
        }
    };

    let mut buffer = vec![0u8; 8192]; // 2**13
    loop {
        let len = response.read(&mut buffer).await.expect("read error");

        if len == 0 {
            break;
        }

        file.write_all(&buffer[..len]).expect("write error");

        downloaded_size = u64::min(downloaded_size + len as u64, content_size);
        let percentage = ((downloaded_size as f64 / content_size as f64) * 100.0).round();

        println!(
            "DOWNLOAD PROGRESS \ndownloaded_size = {downloaded_size} \ntotal = {content_size} \npercentage = {}",
            percentage
        );

        callback(Ok(format!("{percentage}%")));
    }

    callback(Ok("Installing...".to_string()));

    let translation_name = write_to_db(file_path, bible);
    callback(Ok("Done".to_string()));

    translation_name
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

    if let Err(e) = std::fs::remove_file(&file_path) {
        eprintln!("FILE ERROR: error removing downloaded verses \n{:?}", e);
        return None;
    }

    return Some(translation_name);
}
