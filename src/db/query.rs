use std::{cell::RefCell, rc::Rc};

use rusqlite::{params, Result as RuResult};

use crate::{
    db::connection::BibleVerse,
    dto::{Song, SongVerse},
};

use super::connection::{BibleTranslation, DatabaseConnection};

/// Query
pub struct Query {}

impl Query {
    pub fn search_by_partial_text_query(
        conn: Rc<RefCell<Option<DatabaseConnection>>>,
        translation: String,
        text: String,
    ) -> RuResult<Vec<BibleVerse>> {
        let database_connection = conn.borrow();
        let conn = match *database_connection {
            Some(ref conn) => &conn.connection,
            None => {
                return Err(rusqlite::Error::UnwindingPanic);
            }
        };

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
    }
    pub fn search_by_chapter_query(
        conn: Rc<RefCell<Option<DatabaseConnection>>>,
        translation: String,
        book: String,
        chapter: u32,
    ) -> RuResult<Vec<BibleVerse>> {
        let database_connection = conn.borrow();
        let conn = match *database_connection {
            Some(ref conn) => &conn.connection,
            None => {
                return Err(rusqlite::Error::UnwindingPanic);
            }
        };

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
    }

    pub fn insert_song(conn: Rc<RefCell<Option<DatabaseConnection>>>, song: Song) -> RuResult<()> {
        let mut database_connection = conn.borrow_mut();
        let conn = match *database_connection {
            Some(ref mut conn) => &mut conn.connection,
            None => {
                return Err(rusqlite::Error::UnwindingPanic);
            }
        };
        let song_sql = r#"
            INSERT INTO songs(title) VALUES(?1)
        "#;

        let song_verse_sql = r#"
            INSERT INTO song_verses(song_id,verse,text,tag) VALUES(?1,?2,?3,?4)
        "#;

        let tx = conn.transaction()?;

        tx.execute(song_sql, [&song.title])?;
        let song_id = tx.query_row("SELECT id from songs WHERE title = ?1", [song.title], |r| {
            r.get::<_, u32>(0)
        })?;

        for (i, verse) in song.verses.iter().enumerate() {
            tx.execute(
                song_verse_sql,
                (&song_id, &i.saturating_add(1), &verse.text, &verse.tag),
            )?;
        }

        tx.commit()
    }

    pub fn update_song(conn: Rc<RefCell<Option<DatabaseConnection>>>, song: Song) -> RuResult<()> {
        let mut database_connection = conn.borrow_mut();
        let conn = match *database_connection {
            Some(ref mut conn) => &mut conn.connection,
            None => {
                return Err(rusqlite::Error::UnwindingPanic);
            }
        };
        let song_sql = "UPDATE songs SET title=?1 WHERE id = ?2";
        let clear_song_verses_sql = "DELETE FROM song_verses WHERE song_id = ?1";

        let song_verse_sql = r#"
            INSERT INTO song_verses(song_id,verse,text,tag) VALUES(?1,?2,?3,?4)
        "#;

        let tx = conn.transaction()?;
        tx.execute(song_sql, (&song.title, &song.song_id))?;
        tx.execute(clear_song_verses_sql, [&song.song_id])?;

        for (i, verse) in song.verses.iter().enumerate() {
            tx.execute(
                song_verse_sql,
                (&song.song_id, &i.saturating_add(1), &verse.text, &verse.tag),
            )?;
        }

        tx.commit()
    }

    pub fn delete_song(conn: Rc<RefCell<Option<DatabaseConnection>>>, song: Song) -> RuResult<()> {
        let mut database_connection = conn.borrow_mut();
        let conn = match *database_connection {
            Some(ref mut conn) => &mut conn.connection,
            None => {
                return Err(rusqlite::Error::UnwindingPanic);
            }
        };
        let song_sql = "DELETE FROM songs WHERE id = ?1";
        let song_verses_sql = "DELETE FROM song_verses WHERE song_id = ?1";

        let tx = conn.transaction()?;
        tx.execute(song_verses_sql, [&song.song_id])?;
        tx.execute(song_sql, [&song.song_id])?;

        tx.commit()
    }

    pub fn get_songs(
        conn: Rc<RefCell<Option<DatabaseConnection>>>,
        search_text: String,
    ) -> RuResult<Vec<Song>> {
        let database_connection = conn.borrow();
        let conn = match *database_connection {
            Some(ref conn) => &conn.connection,
            None => {
                return Err(rusqlite::Error::UnwindingPanic);
            }
        };

        let mut songs_sql =
            conn.prepare("SELECT id, title FROM songs WHERE title LIKE ?1 ORDER BY title ASC")?;
        let mut songs_verses_sql =
            conn.prepare("SELECT verse, text, tag FROM song_verses WHERE song_id = ?1")?;

        let songs_query = songs_sql.query_map([format!("%{search_text}%")], |r| {
            Ok((r.get::<_, u32>(0)?, r.get::<_, String>(1)?))
        })?;
        let db_songs = songs_query
            .map(|i| i.unwrap())
            .collect::<Vec<(u32, String)>>();

        let mut songs = Vec::new();
        for song in db_songs {
            let verses_query = songs_verses_sql.query_map([&song.0], |r| {
                let t = r.get::<_, String>(1)?;
                let tag = match t.is_empty() {
                    true => None,
                    false => Some(t),
                };

                Ok(SongVerse::new(r.get::<_, String>(1)?, tag))
            })?;

            let verses = verses_query.map(|v| v.unwrap()).collect::<Vec<SongVerse>>();
            songs.push(Song::from_verses(song.0, song.1, verses))
        }

        Ok(songs)
    }

    pub fn insert_verse(
        conn: Rc<RefCell<Option<DatabaseConnection>>>,
        bible_translation: BibleTranslation,
        bible_verse: Vec<(u32, BibleVerse)>,
    ) -> RuResult<()> {
        let mut database_connection = conn.borrow_mut();
        let conn = match *database_connection {
            Some(ref mut conn) => &mut conn.connection,
            None => {
                return Err(rusqlite::Error::UnwindingPanic);
            }
        };

        println!("INSERTING VERESES");
        let translations_sql =
            "INSERT OR IGNORE INTO `translations` (`translation`, `title`, `license`) VALUES (?1, ?2, ?3);";

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
            let verse_sql = format!("INSERT OR IGNORE INTO `{}_verses` (`id`, `book_id`, `chapter`, `verse`, `text`) VALUES (?1, ?2, ?3, ?4, ?5);", bible_translation.translation);

            // println!("INSERTING VERESES id:{id}");
            vec_sql_verse.push((verse_sql, id, book));
        }

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
    }

    pub fn get_translations(
        conn: Rc<RefCell<Option<DatabaseConnection>>>,
    ) -> RuResult<Vec<String>> {
        let database_connection = conn.borrow();
        let conn = match *database_connection {
            Some(ref conn) => &conn.connection,
            None => {
                return Err(rusqlite::Error::UnwindingPanic);
            }
        };
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
    }

    pub fn delete_bible_translation(
        conn: Rc<RefCell<Option<DatabaseConnection>>>,
        translation: String,
    ) -> RuResult<()> {
        let mut database_connection = conn.borrow_mut();
        let conn = match *database_connection {
            Some(ref mut conn) => &mut conn.connection,
            None => {
                return Err(rusqlite::Error::UnwindingPanic);
            }
        };
        let delete_translations_sql = "DELETE FROM translations WHERE translation = ?1";
        let drop_translation_table_sql = format!("DROP TABLE IF EXISTS {translation}_verses"); // <name>_verses

        let trx = conn.transaction()?;
        trx.execute(delete_translations_sql, [&translation])?;
        trx.execute(&drop_translation_table_sql, [])?;

        trx.commit()
    }
}
