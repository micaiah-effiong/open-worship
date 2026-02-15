use core::panic;
use std::sync::{Mutex, MutexGuard, OnceLock};

use rusqlite::Connection;

use crate::app_config::AppConfig;

#[derive(Debug)]
pub struct DatabaseConnection {
    pub connection: Connection,
}

impl DatabaseConnection {
    fn open(path: String) -> DatabaseConnection {
        DatabaseConnection {
            connection: Connection::open(path).expect("Cound not open the database file"),
        }
    }
    pub fn _close(self) -> Result<(), (Connection, rusqlite::Error)> {
        self.connection.close()
    }

    fn instance() -> MutexGuard<'static, DatabaseConnection> {
        let db = DB.get_or_init(|| Mutex::new(DatabaseConnection::open(AppConfig::get_db_path())));
        let conn = db.lock().unwrap();
        conn
    }

    pub fn with_db<F, R>(f: F) -> Result<R, rusqlite::Error>
    where
        F: FnOnce(&Connection) -> Result<R, rusqlite::Error>,
    {
        let conn = Self::instance();

        f(&conn.connection)
    }

    pub fn with_mut_db<F, R>(f: F) -> Result<R, rusqlite::Error>
    where
        F: FnOnce(&mut Connection) -> Result<R, rusqlite::Error>,
    {
        let mut conn = Self::instance();

        f(&mut conn.connection)
    }
}

static DB: OnceLock<Mutex<DatabaseConnection>> = OnceLock::new();

/// open db
/// run setup sql
/// close db
pub fn load_db() {
    create_songs_table();
    create_song_verses_table();
    create_bible_books_table();
    create_translations_table();
    insert_bible_books();

    let _ = DatabaseConnection::with_db(|c| c.pragma_update(None, "journal_mode", "WAL"));
}

pub fn create_bible_books_table() {
    let sql = "CREATE TABLE IF NOT EXISTS bible_books (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL
        )"
    .to_string();

    let ex = DatabaseConnection::with_db(|c| c.execute(&sql, ()));

    if ex.is_err() {
        panic!("Could not create bible_books table {:?}", ex);
    }
}

pub fn create_bible_book_verses_table(translation: String) {
    let sql = format!(
        "CREATE TABLE IF NOT EXISTS {translation}_verses (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            book_id INTEGER NOT NULL,
            chapter INTEGER NOT NULL,
            verse INTEGER NOT NULL,
            text TEXT NOT NULL,
            FOREIGN KEY (book_id) REFERENCES {translation}_books(id)
        )"
    );

    let ex = DatabaseConnection::with_db(|c| c.execute(&sql, ()));

    if ex.is_err() {
        panic!("Could not create {translation}_verses table {:?}", ex);
    }
}

pub fn create_translations_table() {
    let sql = "CREATE TABLE IF NOT EXISTS translations (
            translation TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            license TEXT
        )"
    .to_string();

    let ex = DatabaseConnection::with_db(|c| c.execute(&sql, ()));
    if ex.is_err() {
        panic!("Could not create translations table {:?}", ex);
    }
}

pub fn create_songs_table() {
    let sql = "CREATE TABLE IF NOT EXISTS songs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL
        )"
    .to_string();

    let ex = DatabaseConnection::with_db(|c| c.execute(&sql, ()));
    if ex.is_err() {
        panic!("Could not create song verses table {:?}", ex);
    }
}

pub fn create_song_verses_table() {
    let sql = "CREATE TABLE IF NOT EXISTS song_verses (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            song_id INTEGER NOT NULL,
            verse INTEGER NOT NULL,
            text TEXT NOT NULL,
            tag TEXT,
            slide BLOB,
            FOREIGN KEY (song_id) REFERENCES songs(id)
        )"
    .to_string();

    let ex = DatabaseConnection::with_db(|c| c.execute(&sql, ()));
    if ex.is_err() {
        panic!("Could not create song verses table {:?}", ex);
    }
}

fn insert_bible_books() {
    let bible_books_sql = include_str!("sql/bible_books.sql");
    let sql = bible_books_sql.to_string();

    let ex = DatabaseConnection::with_db(|c| c.execute_batch(&sql));
    if ex.is_err() {
        panic!("Could not create song verses table {:?}", ex);
    }
}

#[derive(Debug)]
pub struct BibleVerse {
    pub book: String,
    pub book_id: u32,
    pub chapter: u32,
    pub text: String,
    pub verse: u32,
}

pub struct BibleBook {
    pub id: u32,
    pub name: String,
}

pub struct BibleTranslation {
    /// translations like
    /// KJV AMP NIV
    pub translation: String,
    pub title: String,
    pub license: String,
}

// search book
// SELECT book_id, chapter, verse, text, kb.name book from kjv_verses
// join kjv_books as kb
// where kjv_verses.book_id = kb.id and ...

// search book
// SELECT * from kjv_books where name like "Ge%"

// fetch translations
// SELECT * from translations
