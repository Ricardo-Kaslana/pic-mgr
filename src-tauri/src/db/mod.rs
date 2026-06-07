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
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        conn.execute_batch(schema::CREATE_TABLES_SQL)?;
        Ok(Database {
            conn: Mutex::new(conn),
        })
    }
}
