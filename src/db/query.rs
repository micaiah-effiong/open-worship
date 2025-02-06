use std::collections::HashMap;

use relm4::factory::Position;
use rusqlite::{params, Connection, Result as RuResult};

use crate::{
    db::connection::BibleVerse,
    dto::{Song, SongVerse},
};

/// Query
pub struct Query {}

impl Query {
    pub fn get_chapter_query(
        conn: &Connection,
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

        return Ok(verses_vec);
    }

    pub fn insert_song(conn: &mut Connection, song: Song) -> RuResult<()> {
        let song_sql = r#"
            INSERT INTO songs(title) VALUES(?1)
        "#;

        let song_verse_sql = r#"
            INSERT INTO song_verses(song_id,verse,text,tag) VALUES(?1,?2,?3,?4)
        "#;

        let tx = conn.transaction()?;

        tx.execute(song_sql, [song.title])?;
        let song_id = tx.query_row("SELECT id from songs ORDER BY id ASC LIMIT 1", [], |r| {
            r.get::<_, u32>(0)
        })?;

        for (i, verse) in song.verses.iter().enumerate() {
            tx.execute(
                song_verse_sql,
                params![song_id, i.saturating_add(1), verse.text, verse.tag],
            )?;
        }

        return tx.commit();
    }

    pub fn update_song(conn: &mut Connection, song: Song) -> RuResult<()> {
        let song_sql = "UPDATE songs SET title=?1 WHERE id = ?2";
        let clear_song_verses_sql = "DELETE FROM song_verses WHERE song_id = ?1";

        let song_verse_sql = r#"
            INSERT INTO song_verses(song_id,verse,text,tag) VALUES(?1,?2,?3,?4)
        "#;

        println!("SONG_ID {:?}", song.song_id);
        let tx = conn.transaction()?;
        tx.execute(song_sql, (&song.title, &song.song_id))?;
        tx.execute(clear_song_verses_sql, [&song.song_id])?;

        for (i, verse) in song.verses.iter().enumerate() {
            println!(
                "VERSES UPDATE {:?}",
                tx.execute(
                    song_verse_sql,
                    (&song.song_id, &i.saturating_add(1), &verse.text, &verse.tag),
                )
            );
        }

        return tx.commit();
    }

    pub fn delete_song(conn: &mut Connection, song: Song) -> RuResult<()> {
        let song_sql = "DELETE FROM songs WHERE id = ?1";
        let song_verses_sql = "DELETE FROM song_verses WHERE song_id = ?1";

        let tx = conn.transaction()?;
        tx.execute(song_verses_sql, [&song.song_id])?;
        tx.execute(song_sql, [&song.song_id])?;

        return tx.commit();
    }

    pub fn get_songs(conn: &Connection, search_text: String) -> RuResult<Vec<Song>> {
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

        return Ok(songs);
    }
}
