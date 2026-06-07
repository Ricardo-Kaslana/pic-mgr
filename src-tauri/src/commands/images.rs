use tauri::State;
use crate::models::{ImagePage, ImageDetail, ImageFilters};
use crate::AppState;

#[tauri::command]
pub fn get_images(
    state: State<AppState>,
    workspace_id: i64,
    filters: ImageFilters,
    page: i32,
    page_size: i32,
) -> Result<ImagePage, String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_images(&conn, workspace_id, &filters, page, page_size)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_image_detail(state: State<AppState>, image_id: i64) -> Result<ImageDetail, String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    let record = crate::db::queries::get_image_detail(&conn, image_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Image not found".to_string())?;
    let tags = crate::db::queries::get_tags_for_image(&conn, image_id)
        .map_err(|e| e.to_string())?;
    Ok(ImageDetail { record, tags })
}

#[tauri::command]
pub fn search_images(
    state: State<AppState>,
    workspace_id: i64,
    query: String,
    page: i32,
    page_size: i32,
) -> Result<ImagePage, String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::search_images(&conn, workspace_id, &query, page, page_size)
        .map_err(|e| e.to_string())
}
