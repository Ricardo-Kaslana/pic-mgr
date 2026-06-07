pub mod commands;
pub mod db;
pub mod models;
pub mod scanner;

use db::Database;
use std::path::PathBuf;

pub struct AppState {
    pub db: Database,
    pub app_data_dir: PathBuf,
}
