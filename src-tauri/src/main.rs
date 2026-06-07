#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use pic_mgr_lib::commands;
use pic_mgr_lib::AppState;
use std::path::PathBuf;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
            let database = pic_mgr_lib::db::Database::new(&app_data_dir)
                .expect("Failed to initialize database");
            app.manage(AppState {
                db: database,
                app_data_dir,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::workspace::create_workspace,
            commands::workspace::list_workspaces,
            commands::workspace::delete_workspace,
            commands::scan::scan_workspace,
            commands::scan::rescan_workspace,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
