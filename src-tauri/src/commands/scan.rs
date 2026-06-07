use tauri::State;
use crate::models::ScanProgress;
use crate::AppState;

#[tauri::command]
pub async fn scan_workspace(state: State<'_, AppState>, id: i64) -> Result<ScanProgress, String> {
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

    // Step 3: Insert in batches (lock per batch, not per entire scan)
    let batch_size = 50;
    let mut processed = 0i64;

    for chunk in image_records.chunks(batch_size) {
        let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
        for record in chunk {
            crate::db::queries::insert_image(&conn, record).map_err(|e| e.to_string())?;
            processed += 1;
        }
        // Lock drops at end of this block, allowing other operations
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
