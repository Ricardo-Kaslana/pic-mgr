use tauri::State;
use crate::AppState;
use std::sync::Mutex;

pub struct ThumbnailState {
    pub cache: Mutex<crate::scanner::thumbnail::ThumbnailCache>,
}

#[tauri::command]
pub fn get_thumbnail(
    state: State<AppState>,
    thumb_state: State<ThumbnailState>,
    image_id: i64,
) -> Result<Vec<u8>, String> {
    let cache = thumb_state.cache.lock().map_err(|e| e.to_string())?;
    if let Some(bytes) = cache.get_bytes(image_id) {
        return Ok(bytes);
    }

    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    let record = crate::db::queries::get_image_detail(&conn, image_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Image not found".to_string())?;

    let workspace = crate::db::queries::get_workspace(&conn, record.workspace_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Workspace not found".to_string())?;

    drop(conn);

    let source_path = std::path::PathBuf::from(&workspace.path).join(&record.relative_path);
    let thumb_path = cache.generate(&source_path, image_id)?;

    let conn2 = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::update_thumbnail_path(&conn2, image_id, &thumb_path)
        .map_err(|e| e.to_string())?;

    cache.get_bytes(image_id).ok_or_else(|| "Failed to read generated thumbnail".to_string())
}

#[tauri::command]
pub fn generate_thumbnail(
    state: State<AppState>,
    thumb_state: State<ThumbnailState>,
    image_id: i64,
) -> Result<(), String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    let record = crate::db::queries::get_image_detail(&conn, image_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Image not found".to_string())?;
    let workspace = crate::db::queries::get_workspace(&conn, record.workspace_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Workspace not found".to_string())?;
    drop(conn);

    let source_path = std::path::PathBuf::from(&workspace.path).join(&record.relative_path);
    let cache = thumb_state.cache.lock().map_err(|e| e.to_string())?;
    let thumb_path = cache.generate(&source_path, image_id)?;

    let conn2 = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::update_thumbnail_path(&conn2, image_id, &thumb_path)
        .map_err(|e| e.to_string())?;

    Ok(())
}
