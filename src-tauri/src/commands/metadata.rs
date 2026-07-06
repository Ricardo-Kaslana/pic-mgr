use tauri::State;
use crate::AppState;
use crate::models::{TrashResult, MoveResult};
use std::path::PathBuf;

#[tauri::command]
pub fn open_in_explorer(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg("/select,")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(std::path::Path::new(&path).parent().unwrap_or(std::path::Path::new(&path)))
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn delete_image(state: State<AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    delete_db_records(&conn, &[id])
}

/// Helper: delete DB records for a list of image IDs.
/// FTS cleanup is handled by AFTER DELETE trigger on images.
fn delete_db_records(conn: &rusqlite::Connection, ids: &[i64]) -> Result<(), String> {
    for &id in ids {
        conn.execute("DELETE FROM image_tags WHERE image_id = ?1", rusqlite::params![id])
            .map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM images WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Helper: delete thumbnail cache files for a list of image IDs
fn delete_thumbnail_files(cache: &crate::scanner::thumbnail::ThumbnailCache, ids: &[i64]) {
    for &id in ids {
        let path = cache.cache_path(id);
        if path.exists() {
            std::fs::remove_file(&path).ok();
        }
    }
}

/// Move files to system recycle bin and remove from DB
#[tauri::command]
pub fn trash_images(
    state: State<AppState>,
    thumb_state: State<super::thumbnail::ThumbnailState>,
    ids: Vec<i64>,
) -> Result<TrashResult, String> {
    if ids.is_empty() {
        return Ok(TrashResult { success: 0, failed: 0, errors: vec![] });
    }

    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;

    // Collect image paths grouped by workspace
    struct ImageInfo {
        id: i64,
        workspace_path: String,
        relative_path: String,
    }
    let mut images: Vec<ImageInfo> = Vec::new();
    for &id in &ids {
        let record = crate::db::queries::get_image_detail(&conn, id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Image {} not found", id))?;
        let workspace = crate::db::queries::get_workspace(&conn, record.workspace_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Workspace {} not found", record.workspace_id))?;
        images.push(ImageInfo {
            id,
            workspace_path: workspace.path,
            relative_path: record.relative_path,
        });
    }
    drop(conn);

    // Move each file to trash
    let mut success = 0;
    let mut errors: Vec<String> = Vec::new();
    let mut succeeded_ids: Vec<i64> = Vec::new();

    for img in &images {
        let abs_path = PathBuf::from(&img.workspace_path).join(&img.relative_path);
        match trash::delete(&abs_path) {
            Ok(()) => {
                success += 1;
                succeeded_ids.push(img.id);
            }
            Err(e) => {
                errors.push(format!("{}: {}", img.relative_path, e));
            }
        }
    }

    // Clean up DB and thumbnails for successfully trashed files
    if !succeeded_ids.is_empty() {
        let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
        delete_db_records(&conn, &succeeded_ids)?;
        drop(conn);

        let cache = thumb_state.cache.lock().map_err(|e| e.to_string())?;
        delete_thumbnail_files(&cache, &succeeded_ids);
    }

    Ok(TrashResult {
        success,
        failed: ids.len() - success,
        errors,
    })
}

/// Move files to a destination directory and update DB paths
#[tauri::command]
pub fn move_images(
    state: State<AppState>,
    thumb_state: State<super::thumbnail::ThumbnailState>,
    ids: Vec<i64>,
    dest_dir: String,
) -> Result<MoveResult, String> {
    if ids.is_empty() {
        return Ok(MoveResult { success: 0, failed: 0, errors: vec![] });
    }

    let dest = PathBuf::from(&dest_dir);
    if !dest.is_dir() {
        return Err(format!("目标文件夹不存在: {}", dest_dir));
    }

    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;

    struct ImageInfo {
        id: i64,
        workspace_path: String,
        relative_path: String,
        file_name: String,
    }
    let mut images: Vec<ImageInfo> = Vec::new();
    for &id in &ids {
        let record = crate::db::queries::get_image_detail(&conn, id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Image {} not found", id))?;
        let workspace = crate::db::queries::get_workspace(&conn, record.workspace_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Workspace {} not found", record.workspace_id))?;
        images.push(ImageInfo {
            id,
            workspace_path: workspace.path,
            relative_path: record.relative_path,
            file_name: record.file_name,
        });
    }
    drop(conn);

    let mut success = 0;
    let mut errors: Vec<String> = Vec::new();
    // Track actual destination paths for DB updates
    struct MovedInfo {
        id: i64,
        workspace_path: String,
        dest_file: PathBuf,
        new_file_name: String,
    }
    let mut moved: Vec<MovedInfo> = Vec::new();

    for img in &images {
        let src = PathBuf::from(&img.workspace_path).join(&img.relative_path);

        // Compute destination path, handle name conflicts
        let mut dest_file = dest.join(&img.file_name);
        let mut new_file_name = img.file_name.clone();
        if dest_file.exists() {
            let stem = PathBuf::from(&img.file_name).file_stem()
                .unwrap_or_default().to_string_lossy().to_string();
            let ext = PathBuf::from(&img.file_name).extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default();
            for i in 1..10000 {
                let candidate_name = format!("{}_{}{}", stem, i, ext);
                let candidate = dest.join(&candidate_name);
                if !candidate.exists() {
                    dest_file = candidate;
                    new_file_name = candidate_name;
                    break;
                }
            }
        }

        // Try rename first; fall back to copy+delete for cross-drive moves
        let move_result = std::fs::rename(&src, &dest_file).or_else(|_| {
            std::fs::copy(&src, &dest_file).and_then(|_| std::fs::remove_file(&src))
        });

        match move_result {
            Ok(()) => {
                success += 1;
                moved.push(MovedInfo {
                    id: img.id,
                    workspace_path: img.workspace_path.clone(),
                    dest_file,
                    new_file_name,
                });
            }
            Err(e) => {
                errors.push(format!("{}: {}", img.file_name, e));
            }
        }
    }

    // Update DB for successfully moved files
    if !moved.is_empty() {
        // Clear stale thumbnails
        let cache = thumb_state.cache.lock().map_err(|e| e.to_string())?;
        let moved_ids: Vec<i64> = moved.iter().map(|m| m.id).collect();
        delete_thumbnail_files(&cache, &moved_ids);
        drop(cache);

        let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
        for m in &moved {
            let ws_path = PathBuf::from(&m.workspace_path);
            // Check if destination is inside this workspace
            let new_relative = if let Ok(rel) = m.dest_file.strip_prefix(&ws_path) {
                Some(rel.to_string_lossy().replace('\\', "/"))
            } else {
                None
            };

            if let Some(rel_path) = new_relative {
                // File is still in workspace — update record in-place
                conn.execute(
                    "UPDATE images SET relative_path = ?1, file_name = ?2, updated_at = datetime('now') WHERE id = ?3",
                    rusqlite::params![rel_path, m.new_file_name, m.id],
                ).map_err(|e| e.to_string())?;
            } else {
                // File moved outside workspace — remove from DB
                delete_db_records(&conn, &[m.id])?;
            }
        }
    }

    Ok(MoveResult {
        success,
        failed: ids.len() - success,
        errors,
    })
}

/// Read a file from disk and return raw bytes (for full-size image preview)
#[tauri::command]
pub fn read_file_bytes(path: String) -> Result<Vec<u8>, String> {
    std::fs::read(&path).map_err(|e| format!("Cannot read {}: {}", path, e))
}

#[tauri::command]
pub fn update_notes(state: State<AppState>, image_id: i64, notes: String) -> Result<(), String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::update_notes(&conn, image_id, &notes).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_rating(state: State<AppState>, image_id: i64, rating: i32) -> Result<(), String> {
    if rating < 0 || rating > 5 {
        return Err("Rating must be 0-5".to_string());
    }
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::update_rating(&conn, image_id, rating).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn batch_update_tags(
    state: State<AppState>,
    image_ids: Vec<i64>,
    tag_ids: Vec<i64>,
    action: String,
) -> Result<(), String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::batch_update_tags(&conn, &image_ids, &tag_ids, &action)
        .map_err(|e| e.to_string())
}
