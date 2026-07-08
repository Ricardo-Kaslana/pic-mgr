use tauri::State;
use crate::models::{ImagePage, ImageSummaryPage, ImageDetail, ImageFilters};
use crate::AppState;

#[derive(serde::Serialize)]
pub struct DuplicateGroup {
    pub hash: String,
    pub images: Vec<crate::models::ImageRecord>,
}

fn find_duplicates_inner(
    state: &AppState,
    workspace_id: i64,
) -> Result<Vec<DuplicateGroup>, String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    // Single-query duplicate detection via CTE — replaces the previous
    // 1 + N query loop (one per duplicate hash).
    let groups = crate::db::queries::find_duplicate_groups(&conn, workspace_id)
        .map_err(|e| e.to_string())?;
    Ok(groups
        .into_iter()
        .map(|(hash, images)| DuplicateGroup { hash, images })
        .collect())
}

#[tauri::command]
pub fn find_duplicates(
    state: State<AppState>,
    workspace_id: i64,
) -> Result<Vec<DuplicateGroup>, String> {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        find_duplicates_inner(&state, workspace_id)
    }));
    match result {
        Ok(inner) => inner,
        Err(_) => Err("find_duplicates panicked".to_string()),
    }
}

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

/// Lightweight grid query — the hot path for first-load of 100+ image
/// workspaces. Returns ~3x less data than `get_images` and avoids the
/// fields the grid never reads (file_hash, relative_path, notes,
/// last_viewed_at, file_created_at).
#[tauri::command]
pub fn get_images_light(
    state: State<AppState>,
    workspace_id: i64,
    filters: ImageFilters,
    page: i32,
    page_size: i32,
) -> Result<ImageSummaryPage, String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_images_light(&conn, workspace_id, &filters, page, page_size)
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