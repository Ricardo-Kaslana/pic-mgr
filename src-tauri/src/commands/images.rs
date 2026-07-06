use tauri::State;
use crate::models::{ImagePage, ImageDetail, ImageFilters};
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

    let hashes: Vec<String> = {
        let mut stmt = conn.prepare(
            "SELECT file_hash FROM images WHERE workspace_id = ?1 AND file_hash IS NOT NULL
             GROUP BY file_hash HAVING COUNT(*) > 1"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map(rusqlite::params![workspace_id], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        rows.filter_map(|r| r.ok()).collect()
    };

    // Prepare statement once, reuse for all hash lookups
    let mut img_stmt = conn.prepare(
        "SELECT id, workspace_id, relative_path, file_name, file_size, file_hash, width, height, format, thumbnail_path, notes, rating, created_at, file_created_at, updated_at, indexed_at, last_viewed_at
         FROM images WHERE file_hash = ?1 AND workspace_id = ?2"
    ).map_err(|e| e.to_string())?;

    let mut groups = Vec::new();
    for hash in hashes {
        let rows = img_stmt.query_map(
            rusqlite::params![hash, workspace_id],
            crate::db::queries::row_to_image_record,
        ).map_err(|e| e.to_string())?;
        let images: Vec<crate::models::ImageRecord> = rows.filter_map(|r| r.ok()).collect();

        if !images.is_empty() {
            groups.push(DuplicateGroup { hash, images });
        }
    }

    Ok(groups)
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
