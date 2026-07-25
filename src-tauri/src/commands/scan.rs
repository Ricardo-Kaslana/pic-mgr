use tauri::{State, AppHandle, Emitter};
use crate::models::ScanProgress;
use crate::AppState;

#[tauri::command]
pub fn scan_workspace(state: State<AppState>, app: AppHandle, id: i64) -> Result<ScanProgress, String> {
    // Step 1: Get workspace path (short lock)
    let ws_path = {
        let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
        let workspace = crate::db::queries::get_workspace(&conn, id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Workspace not found".to_string())?;
        std::path::PathBuf::from(&workspace.path)
    };

    if !ws_path.exists() {
        return Err(format!("Path does not exist: {}", ws_path.display()));
    }

    // Step 2: Scan directory WITHOUT holding DB lock
    let image_records = crate::scanner::scan_directory_individually(id, &ws_path);
    let total = image_records.len() as i64;

    // Step 3: Insert in batches, emitting progress after each batch
    let batch_size = 50;
    let mut processed = 0i64;

    for chunk in image_records.chunks(batch_size) {
        let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
        conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;
        for record in chunk {
            crate::db::queries::insert_image(&conn, record).map_err(|e| e.to_string())?;
            processed += 1;
        }
        conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;

        app.emit("scan-progress", ScanProgress {
            total,
            processed,
            new_files: processed,
            modified_files: 0,
            finished: false,
        }).map_err(|e| e.to_string())?;
    }

    let final_progress = ScanProgress {
        total,
        processed,
        new_files: total,
        modified_files: 0,
        finished: true,
    };

    app.emit("scan-progress", final_progress.clone()).map_err(|e| e.to_string())?;

    Ok(final_progress)
}

#[tauri::command]
pub fn rescan_workspace(
    state: State<AppState>,
    thumb_state: State<crate::commands::thumbnail::ThumbnailState>,
    app: AppHandle,
    id: i64,
) -> Result<ScanProgress, String> {
    // Verify the path is reachable BEFORE wiping rows -- otherwise a
    // disconnected drive turns "full reset" into total data loss.
    let old_ids: Vec<i64> = {
        let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
        let workspace = crate::db::queries::get_workspace(&conn, id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Workspace not found".to_string())?;
        let ws_path = std::path::PathBuf::from(&workspace.path);
        if !ws_path.exists() {
            return Err(format!("Path does not exist: {}", ws_path.display()));
        }
        let mut stmt = conn
            .prepare("SELECT id FROM images WHERE workspace_id = ?1")
            .map_err(|e| e.to_string())?;
        let ids: Vec<i64> = stmt
            .query_map(rusqlite::params![id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        conn.execute("DELETE FROM images WHERE workspace_id = ?1", rusqlite::params![id])
            .map_err(|e| e.to_string())?;
        ids
    };
    // Rescan inserts rows with fresh AUTOINCREMENT ids, so the old {id}.jpg
    // cache files would never be reclaimed without this sweep.
    if let Ok(cache) = thumb_state.cache.lock() {
        crate::commands::metadata::delete_thumbnail_files(&cache, &old_ids);
    }
    scan_workspace(state, app, id)
}
