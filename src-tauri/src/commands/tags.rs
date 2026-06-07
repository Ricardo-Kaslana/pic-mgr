use tauri::State;
use crate::models::{Tag, TagWithCount};
use crate::AppState;

#[tauri::command]
pub fn get_tags(state: State<AppState>, workspace_id: i64) -> Result<Vec<TagWithCount>, String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_tags(&conn, workspace_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_tag(
    state: State<AppState>,
    name: String,
    color: Option<String>,
) -> Result<Tag, String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::create_tag(&conn, &name, color.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_tag(state: State<AppState>, tag_id: i64) -> Result<(), String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::delete_tag(&conn, tag_id).map_err(|e| e.to_string())
}
