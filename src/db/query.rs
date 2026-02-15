use gtk::glib;
use rusqlite::{Result as RuResult, params};

use crate::{
    db::connection::BibleVerse,
    dto::{SongData, SongVerse},
    widgets::canvas::serialise::{CanvasItemType, SlideData},
};

use super::connection::{BibleTranslation, DatabaseConnection};

/// Query
pub struct Query {}

impl Query {
    pub fn search_by_partial_text_query(
        translation: String,
        text: String,
    ) -> RuResult<Vec<BibleVerse>> {
        let sql = format!(
            r#"
            SELECT book_id, chapter, verse, text, books.name AS book 
            FROM {translation}_verses
            JOIN bible_books AS books ON books.id = {translation}_verses.book_id
            WHERE {translation}_verses.text LIKE ?1
            LIMIT 100
            "#
        );
        // println!("SEARCH \nbook: {book}, chapter: {chapter}, transaction: {translation}");

        let rows = DatabaseConnection::with_db(|conn| {
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params![format!("%{text}%")], |r| {
                Ok(BibleVerse {
                    book_id: r.get::<_, u32>(0)?,
                    chapter: r.get::<_, u32>(1)?,
                    verse: r.get::<_, u32>(2)?,
                    text: r.get::<_, String>(3)?,
                    book: r.get::<_, String>(4)?,
                })
            })?;
            let mut verses_vec = Vec::new();
            for row in rows {
                verses_vec.push(row.unwrap());
            }

            Ok(verses_vec)
        });

        rows
    }
    pub fn search_by_chapter_query(
        translation: String,
        book: String,
        chapter: u32,
    ) -> RuResult<Vec<BibleVerse>> {
        let sql = format!(
            r#"
            SELECT book_id, chapter, verse, text, books.name AS book 
            FROM {translation}_verses
            JOIN bible_books AS books ON books.id = {translation}_verses.book_id
            WHERE {translation}_verses.book_id =(SELECT id FROM bible_books WHERE name LIKE ?1) 
                AND {translation}_verses.chapter = ?2 
            "#
        );
        // println!("SEARCH \nbook: {book}, chapter: {chapter}, transaction: {translation}");

        let rows = DatabaseConnection::with_db(|conn| {
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params![format!("%{book}%"), chapter], |r| {
                Ok(BibleVerse {
                    book_id: r.get::<_, u32>(0)?,
                    chapter: r.get::<_, u32>(1)?,
                    verse: r.get::<_, u32>(2)?,
                    text: r.get::<_, String>(3)?,
                    book: r.get::<_, String>(4)?,
                })
            })?;

            let mut verses_vec = Vec::new();
            for row in rows {
                verses_vec.push(row.unwrap());
            }

            Ok(verses_vec)
        });

        rows
    }

    pub fn insert_song(song: SongData) -> RuResult<()> {
        let song_sql = r#"
            INSERT INTO songs(title) VALUES(?1)
        "#;

        let song_verse_sql = r#"
            INSERT INTO song_verses(song_id,verse,text,tag,slide) VALUES(?1,?2,?3,?4,jsonb(?5))
        "#;

        let r = DatabaseConnection::with_mut_db(|conn| {
            let tx = conn.transaction()?;

            tx.execute(song_sql, [&song.title])?;
            let song_id =
                tx.query_row("SELECT id from songs WHERE title = ?1", [song.title], |r| {
                    r.get::<_, u32>(0)
                })?;

            for (i, verse) in song.verses.iter().enumerate() {
                tx.execute(
                    song_verse_sql,
                    (
                        &song_id,
                        &i.saturating_add(1),
                        &verse.text,
                        &verse.tag,
                        &verse.slide,
                    ),
                )?;
            }

            tx.commit()
        });

        r
    }

    pub fn update_song(song: SongData) -> RuResult<()> {
        let song_sql = "UPDATE songs SET title=?1 WHERE id = ?2";
        let clear_song_verses_sql = "DELETE FROM song_verses WHERE song_id = ?1";

        let song_verse_sql = r#"
            INSERT INTO song_verses(song_id,verse,text,tag,slide) VALUES(?1,?2,?3,?4,jsonb(?5))
        "#;

        let r = DatabaseConnection::with_mut_db(|conn| {
            let tx = conn.transaction()?;
            tx.execute(song_sql, (&song.title, &song.song_id))?;
            tx.execute(clear_song_verses_sql, [&song.song_id])?;

            for (i, verse) in song.verses.iter().enumerate() {
                tx.execute(
                    song_verse_sql,
                    (
                        &song.song_id,
                        &i.saturating_add(1),
                        &verse.text,
                        &verse.tag,
                        &verse.slide,
                    ),
                )?;
            }

            tx.commit()
        });

        r
    }

    pub fn delete_song(song: SongData) -> RuResult<()> {
        let song_sql = "DELETE FROM songs WHERE id = ?1";
        let song_verses_sql = "DELETE FROM song_verses WHERE song_id = ?1";

        let r = DatabaseConnection::with_mut_db(|conn| {
            let tx = conn.transaction()?;
            tx.execute(song_verses_sql, [&song.song_id])?;
            tx.execute(song_sql, [&song.song_id])?;

            tx.commit()
        });

        r
    }

    pub fn get_songs(search_text: String) -> RuResult<Vec<SongData>> {
        let r = DatabaseConnection::with_mut_db(|conn| {
            let mut songs_sql =
                conn.prepare("SELECT id, title FROM songs WHERE title LIKE ?1 ORDER BY title ASC")?;
            let mut songs_verses_sql = conn.prepare(
                "SELECT verse, text, tag, json(slide) FROM song_verses WHERE song_id = ?1",
            )?;

            let songs_query = songs_sql.query_map([format!("%{search_text}%")], |r| {
                Ok((r.get::<_, u32>(0)?, r.get::<_, String>(1)?))
            })?;
            let db_songs = songs_query
                .map(|i| i.unwrap())
                .collect::<Vec<(u32, String)>>();

            let mut songs = Vec::new();
            for song in db_songs {
                let verses_query = songs_verses_sql.query_map([&song.0], |r| {
                    let text = r.get::<_, Option<String>>(1)?;
                    let tag = r.get::<_, Option<String>>(2)?;
                    let slide = r.get::<_, Option<String>>(3)?;

                    let default_slide = serde_json::to_string(&SlideData::from_default()).ok();

                    let slide = slide.as_ref().or(default_slide.as_ref()).and_then(|s| {
                        serde_json::from_str::<SlideData>(s)
                            .ok()
                            .and_then(|mut ss| {
                                ss.items
                                    .iter_mut()
                                    .find_map(|item| match &mut item.item_type {
                                        CanvasItemType::Text(text_item) => {
                                            text_item.text_data = glib::base64_encode(
                                                text.clone().unwrap_or_default().as_bytes(),
                                            )
                                            .into();
                                            Some(())
                                        }
                                        _ => None,
                                    })?;
                                serde_json::to_string(&ss).ok()
                            })
                    });

                    Ok(SongVerse::new(text.unwrap_or_default(), tag, slide))
                })?;

                let verses = verses_query.map(|v| v.unwrap()).collect::<Vec<SongVerse>>();
                songs.push(SongData::new(song.0, song.1, verses))
            }

            Ok(songs)
        });

        r
    }

    pub fn get_all_songs() -> RuResult<Vec<SongData>> {
        Self::get_songs(String::new())
    }

    pub fn insert_verse(
        bible_translation: BibleTranslation,
        bible_verse: Vec<(u32, BibleVerse)>,
    ) -> RuResult<()> {
        let translations_sql = "INSERT OR IGNORE INTO `translations` (`translation`, `title`, `license`) VALUES (?1, ?2, ?3);";

        let table_sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS `{}_verses` (
                `id` INT AUTO_INCREMENT PRIMARY KEY,
                `book_id` INT,
                `chapter` INT,
                `verse` INT,
                `text` TEXT,
                FOREIGN KEY (book_id) REFERENCES `bible_books`(id)
            );
        "#,
            bible_translation.translation, //bible_translation.translation
        );

        let mut vec_sql_verse = Vec::new();
        for item in bible_verse {
            let (id, book) = item;
            let verse_sql = format!(
                "INSERT OR IGNORE INTO `{}_verses` (`id`, `book_id`, `chapter`, `verse`, `text`) VALUES (?1, ?2, ?3, ?4, ?5);",
                bible_translation.translation
            );

            // println!("INSERTING VERESES id:{id}");
            vec_sql_verse.push((verse_sql, id, book));
        }

        let r = DatabaseConnection::with_mut_db(|conn| {
            let tx = conn.transaction()?;

            tx.execute(&table_sql, [])?;
            tx.execute(
                translations_sql,
                [
                    &bible_translation.translation,
                    &bible_translation.title,
                    &bible_translation.license,
                ],
            )?;

            for verse in vec_sql_verse.iter() {
                let (sql, id, book) = verse;
                match tx.execute(
                    sql,
                    (id, book.book_id, book.chapter, book.verse, &book.text),
                ) {
                    Ok(s) => s,
                    Err(e) => {
                        println!(">>INSERTING VERESES ERROR: {:?}", verse);
                        return Err(e);
                    }
                };
            }

            tx.commit()
        });

        r
    }

    pub fn get_translations() -> RuResult<Vec<String>> {
        let r = DatabaseConnection::with_mut_db(|conn| {
            let sql = "SELECT translation FROM translations";
            let mut stmt = conn.prepare(sql)?;

            let query_result = stmt.query_map([], |r| r.get::<_, String>(0))?;

            let mut translation_list = Vec::new();
            for i in query_result.into_iter() {
                match i {
                    Ok(i) => translation_list.push(i),
                    Err(e) => eprintln!("SQL ERROR: {:?}", e),
                }
            }

            Ok(translation_list)
        });

        r
    }

    pub fn delete_bible_translation(translation: String) -> RuResult<()> {
        let delete_translations_sql = "DELETE FROM translations WHERE translation = ?1";
        let drop_translation_table_sql = format!("DROP TABLE IF EXISTS {translation}_verses"); // <name>_verses

        let r = DatabaseConnection::with_mut_db(|conn| {
            let trx = conn.transaction()?;
            trx.execute(delete_translations_sql, [&translation])?;
            trx.execute(&drop_translation_table_sql, [])?;

            trx.commit()
        });

        r
    }
}
