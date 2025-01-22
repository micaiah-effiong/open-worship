use rusqlite::{Connection, Result as RuResult};

use crate::db::connection::BibleVerse;

use super::connection::DatabaseConnection;

/// Query
pub struct Query<'a> {
    pub database: &'a DatabaseConnection,
}

impl<'a> Query<'a> {
    pub fn new(database: &'a DatabaseConnection) -> Query<'a> {
        return Query {
            database: &database,
        };
    }

    fn get_connection(&self) -> &Connection {
        return &self.database.connection;
    }

    pub fn get_chapter_query(
        &self,
        translation: String,
        book_id: u32,
        chapter: u32,
    ) -> RuResult<Vec<BibleVerse>> {
        let sql = format!(
            r#"
            SELECT book_id, chapter, verse, text, kb.name AS book 
            FROM {translation}_verses
            JOIN {translation}_books as kb
            WHERE {translation}_verses.book_id = ?1 
                AND {translation}_verses.chapter = ?2 
            "#
        );

        let mut stmt = self.get_connection().prepare(&sql)?;
        let rows = stmt.query_map([book_id, chapter], |r| {
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
}
