use core::panic;

use rusqlite::Connection;

#[derive(Debug)]
pub struct DatabaseConnection {
    pub connection: Connection,
}

impl DatabaseConnection {
    pub fn open(path: String) -> DatabaseConnection {
        DatabaseConnection {
            connection: Connection::open(path).expect("Cound not open the database file"),
        }
    }
    pub fn close(self) -> Result<(), (Connection, rusqlite::Error)> {
        self.connection.close()
    }
}

/// open db
/// run setup sql
/// close db
pub fn load_db(path: String) {
    let conn = Connection::open(path).expect("Cound not open the database file");

    create_songs_table(&conn);
    create_song_verses_table(&conn);
    create_bible_books_table(&conn);
    create_translations_table(&conn);
    insert_bible_books(&conn);

    let _ = conn.pragma_update(None, "journal_mode", "WAL");
    let _ = conn.close();
}

pub fn create_bible_books_table(conn: &Connection) {
    let sql = "CREATE TABLE IF NOT EXISTS bible_books (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL
        )".to_string();

    let ex = conn.execute(&sql, ());
    if ex.is_err() {
        panic!("Could not create bible_books table {:?}", ex);
    }
}

pub fn create_bible_book_verses_table(conn: Connection, translation: String) {
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

    let ex = conn.execute(&sql, ());
    if ex.is_err() {
        panic!("Could not create {translation}_verses table {:?}", ex);
    }
}

pub fn create_translations_table(conn: &Connection) {
    let sql = "CREATE TABLE IF NOT EXISTS translations (
            translation TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            license TEXT
        )".to_string();

    let ex = conn.execute(&sql, ());
    if ex.is_err() {
        panic!("Could not create translations table {:?}", ex);
    }
}

pub fn create_songs_table(conn: &Connection) {
    let sql = "CREATE TABLE IF NOT EXISTS songs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL
        )".to_string();

    let ex = conn.execute(&sql, ());
    if ex.is_err() {
        panic!("Could not create song verses table {:?}", ex);
    }
}

pub fn create_song_verses_table(conn: &Connection) {
    let sql = "CREATE TABLE IF NOT EXISTS song_verses (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            song_id INTEGER NOT NULL,
            verse INTEGER NOT NULL,
            text TEXT NOT NULL,
            tag TEXT,
            FOREIGN KEY (song_id) REFERENCES songs(id)
        )".to_string();

    let ex = conn.execute(&sql, ());
    if ex.is_err() {
        panic!("Could not create song verses table {:?}", ex);
    }
}

fn insert_bible_books(conn: &Connection) {
    let bible_books_sql = include_str!("sql/bible_books.sql");
    let sql = bible_books_sql.to_string();

    let ex = conn.execute_batch(&sql);
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
