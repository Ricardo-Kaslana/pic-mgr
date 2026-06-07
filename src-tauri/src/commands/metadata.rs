use tauri::State;
use crate::AppState;

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
    conn.execute("DELETE FROM image_tags WHERE image_id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM images_fts WHERE rowid = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM images WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
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
