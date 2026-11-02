pub mod queries;
pub mod schema;

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
        // ── Performance PRAGMAs ──
        // WAL gives concurrent readers + 1 writer; the rest of the pragmas
        // tune OS-level caching, mmap, and write coalescing for our workload
        // (many small reads during browse + occasional bulk thumbnail batches).
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;          -- safe with WAL, ~2x faster than FULL
             PRAGMA temp_store=MEMORY;            -- keep temp indices in RAM
             PRAGMA cache_size=-20000;            -- 20 MB page cache
             PRAGMA mmap_size=268435456;          -- 256 MB memory-mapped I/O
             PRAGMA foreign_keys=ON;",
        )?;
        conn.execute_batch(schema::CREATE_TABLES_SQL)?;
        // Migrations: add columns that may not exist in older databases
        conn.execute_batch("ALTER TABLE images ADD COLUMN file_created_at TEXT;")
            .ok(); // ignore error if column already exists
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
        // Per-statement optimization: pre-compile hot queries + analyze statistics.
        // analyze gathers row-count distribution so the planner picks the right index
        // for the workspace_id+sort and workspace_id+format queries.
        conn.execute_batch("ANALYZE;").ok();
        Ok(Database {
            conn: Mutex::new(conn),
        })
    }
}
