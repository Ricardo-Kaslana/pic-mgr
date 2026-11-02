use crate::models::Workspace;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn create_workspace(
    state: State<AppState>,
    name: String,
    path: String,
) -> Result<Workspace, String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::insert_workspace(&conn, &name, &path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_workspaces(state: State<AppState>) -> Result<Vec<Workspace>, String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::list_workspaces(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_workspace(state: State<AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::delete_workspace(&conn, id).map_err(|e| e.to_string())
}
