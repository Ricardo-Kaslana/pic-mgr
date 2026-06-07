use tauri::State;
use crate::models::ScanProgress;
use crate::AppState;

#[tauri::command]
pub async fn scan_workspace(state: State<'_, AppState>, id: i64) -> Result<ScanProgress, String> {
    let workspace = {
        let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
        crate::db::queries::get_workspace(&conn, id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Workspace not found".to_string())?
    };

    let ws_path = std::path::PathBuf::from(&workspace.path);
    if !ws_path.exists() {
        return Err(format!("Path does not exist: {}", workspace.path));
    }

    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    let image_records = crate::scanner::scan_directory_individually(id, &ws_path);
    let total = image_records.len() as i64;
    let mut processed = 0i64;

    for record in &image_records {
        crate::db::queries::insert_image(&conn, record).map_err(|e| e.to_string())?;
        processed += 1;
    }

    Ok(ScanProgress {
        total,
        processed,
        new_files: total,
        modified_files: 0,
        finished: true,
    })
}

#[tauri::command]
pub async fn rescan_workspace(state: State<'_, AppState>, id: i64) -> Result<ScanProgress, String> {
    {
        let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM images WHERE workspace_id = ?1", rusqlite::params![id])
            .map_err(|e| e.to_string())?;
    }
    scan_workspace(state, id).await
}
