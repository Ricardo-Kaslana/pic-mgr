pub mod schema;
pub mod queries;

use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_data_dir: &PathBuf) -> Result<Self> {
        std::fs::create_dir_all(app_data_dir).ok();
        let db_path = app_data_dir.join("pic-mgr.db");
        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        conn.execute_batch(schema::CREATE_TABLES_SQL)?;
        // Migrations: add columns that may not exist in older databases
        conn.execute_batch(
            "ALTER TABLE images ADD COLUMN file_created_at TEXT;"
        ).ok(); // ignore error if column already exists
        // Migration: update FTS triggers to use rowid-only delete (skip content verification)
        conn.execute_batch(
            "DROP TRIGGER IF EXISTS images_fts_delete;
             DROP TRIGGER IF EXISTS images_fts_update;
             CREATE TRIGGER IF NOT EXISTS images_fts_delete AFTER DELETE ON images BEGIN
                 INSERT INTO images_fts(images_fts, rowid) VALUES('delete', old.id);
             END;
             CREATE TRIGGER IF NOT EXISTS images_fts_update AFTER UPDATE ON images BEGIN
                 INSERT INTO images_fts(images_fts, rowid) VALUES('delete', old.id);
                 INSERT INTO images_fts(rowid, file_name, notes) VALUES (new.id, new.file_name, new.notes);
             END;"
        ).ok();
        Ok(Database {
            conn: Mutex::new(conn),
        })
    }
}
