#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use pic_mgr_lib::commands;
use pic_mgr_lib::db::Database;
use pic_mgr_lib::AppState;
use pic_mgr_lib::scanner::thumbnail::ThumbnailCache;
use pic_mgr_lib::commands::thumbnail::ThumbnailState;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
            let database = Database::new(&app_data_dir)
                .expect("Failed to initialize database");

            let cache_dir = app_data_dir.join("thumbnails");
            let thumb_cache = ThumbnailCache::new(cache_dir);

            app.manage(AppState {
                db: database,
                app_data_dir,
            });
            app.manage(ThumbnailState {
                cache: Mutex::new(thumb_cache),
                workspace_paths: Mutex::new(std::collections::HashMap::new()),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::workspace::create_workspace,
            commands::workspace::list_workspaces,
            commands::workspace::delete_workspace,
            commands::scan::scan_workspace,
            commands::scan::rescan_workspace,
            commands::sync::sync_workspace,
            commands::sync::apply_sync_deletions,
            commands::images::get_images,
            commands::images::get_images_light,
            commands::images::get_image_detail,
            commands::images::search_images,
            commands::images::find_duplicates,
            commands::metadata::update_notes,
            commands::metadata::update_rating,
            commands::metadata::batch_update_tags,
            commands::metadata::open_in_explorer,
            commands::metadata::delete_image,
            commands::metadata::trash_images,
            commands::metadata::move_images,
            commands::metadata::read_file_bytes,
            commands::tags::get_tags,
            commands::tags::get_all_tags,
            commands::tags::create_tag,
            commands::tags::delete_tag,
            commands::thumbnail::get_thumbnail,
            commands::thumbnail::get_thumbnails_batch,
            commands::thumbnail::generate_thumbnail,
            commands::thumbnail::generate_thumbnails_batch,
            commands::export::export_metadata,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
