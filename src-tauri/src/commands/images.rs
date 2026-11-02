use crate::models::{ImageDetail, ImageFilters, ImagePage, ImageSummaryPage};
use crate::AppState;
use tauri::State;

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
    let (mut record, tags, ws_path) = {
        let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
        let record = crate::db::queries::get_image_detail(&conn, image_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Image not found".to_string())?;
        let tags =
            crate::db::queries::get_tags_for_image(&conn, image_id).map_err(|e| e.to_string())?;
        // Rows indexed before dimension backfill existed (or whose thumbnail
        // already sat on disk, skipping generate()) have NULL width/height.
        let ws_path = if record.width.is_none() || record.height.is_none() {
            crate::db::queries::get_workspace(&conn, record.workspace_id)
                .ok()
                .flatten()
                .map(|ws| std::path::PathBuf::from(ws.path))
        } else {
            None
        };
        (record, tags, ws_path)
    };

    if let Some(ws) = ws_path {
        let source = ws.join(&record.relative_path);
        // into_dimensions parses only the header (no full decode);
        // catch_unwind guards against image-rs panics on malformed files.
        let dims = std::panic::catch_unwind(|| {
            image::ImageReader::open(&source)
                .ok()?
                .with_guessed_format()
                .ok()?
                .into_dimensions()
                .ok()
        })
        .ok()
        .flatten();
        if let Some((w, h)) = dims {
            let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
            // Deliberately no updated_at bump: a passive read should not
            // reshuffle the "recently updated" sort order.
            conn.execute(
                "UPDATE images SET width = ?1, height = ?2 WHERE id = ?3",
                rusqlite::params![w, h, image_id],
            )
            .ok();
            record.width = Some(w as i32);
            record.height = Some(h as i32);
        }
    }

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
