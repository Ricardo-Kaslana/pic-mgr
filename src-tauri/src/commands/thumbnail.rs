use tauri::{State, Emitter};
use crate::AppState;
use std::sync::Mutex;
use std::collections::HashMap;

pub struct ThumbnailState {
    pub cache: Mutex<crate::scanner::thumbnail::ThumbnailCache>,
    /// Lazily-cached workspace paths keyed by workspace_id — avoids 2x DB query per thumbnail
    pub workspace_paths: Mutex<HashMap<i64, std::path::PathBuf>>,
}

/// Resolve workspace path from cache or DB. Caller must hold the DB lock.
fn get_workspace_path_cached(
    thumb_state: &ThumbnailState,
    conn: &rusqlite::Connection,
    workspace_id: i64,
) -> Result<std::path::PathBuf, String> {
    {
        let paths = thumb_state.workspace_paths.lock().map_err(|e| e.to_string())?;
        if let Some(path) = paths.get(&workspace_id) {
            return Ok(path.clone());
        }
    }
    let ws = crate::db::queries::get_workspace(conn, workspace_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Workspace {} not found", workspace_id))?;
    let path = std::path::PathBuf::from(&ws.path);
    let mut paths = thumb_state.workspace_paths.lock().map_err(|e| e.to_string())?;
    paths.insert(workspace_id, path.clone());
    Ok(path)
}

#[tauri::command]
pub fn get_thumbnail(
    state: State<AppState>,
    thumb_state: State<ThumbnailState>,
    image_id: i64,
) -> Result<Vec<u8>, String> {
    // Fast path: thumbnail already cached on disk
    {
        let cache = thumb_state.cache.lock().map_err(|e| e.to_string())?;
        if let Some(bytes) = cache.get_bytes(image_id) {
            return Ok(bytes);
        }
    }

    // Slow path: need to generate
    let gen_result: Result<(String, u32, u32), String> = {
        let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
        let record = crate::db::queries::get_image_detail(&conn, image_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Image not found".to_string())?;
        let ws_path = get_workspace_path_cached(&thumb_state, &conn, record.workspace_id)?;
        let source_path = ws_path.join(&record.relative_path);
        drop(conn);

        let cache = thumb_state.cache.lock().map_err(|e| e.to_string())?;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            cache.generate(&source_path, image_id)
        }));
        match result {
            Ok(Ok(out)) => Ok(out),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(format!("Failed to process image: {}", source_path.display())),
        }
    };

    if let Ok((ref path, w, h)) = gen_result {
        let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
        crate::db::queries::update_thumbnail_path(&conn, image_id, path, w, h)
            .map_err(|e| e.to_string())?;
    }

    let cache = thumb_state.cache.lock().map_err(|e| e.to_string())?;
    cache.get_bytes(image_id).ok_or_else(|| "Failed to read generated thumbnail".to_string())
}

/// Batch thumbnail loading: single IPC call for multiple image IDs.
#[derive(serde::Serialize)]
pub struct ThumbnailBatchEntry {
    pub image_id: i64,
    pub bytes: Vec<u8>,
}

#[tauri::command]
pub fn get_thumbnails_batch(
    state: State<AppState>,
    thumb_state: State<ThumbnailState>,
    image_ids: Vec<i64>,
) -> Result<Vec<ThumbnailBatchEntry>, String> {
    let mut results = Vec::with_capacity(image_ids.len());
    let mut miss_ids: Vec<i64> = Vec::new();

    // Phase 1: Collect cache hits, gather miss IDs
    {
        let cache = thumb_state.cache.lock().map_err(|e| e.to_string())?;
        for &id in &image_ids {
            if let Some(bytes) = cache.get_bytes(id) {
                results.push(ThumbnailBatchEntry { image_id: id, bytes });
            } else {
                miss_ids.push(id);
            }
        }
    }

    if miss_ids.is_empty() {
        return Ok(results);
    }

    // Phase 2: Single-query path resolution (replaces 1 + N queries).
    // Pre-filter: skip ids whose thumbnail file already exists on disk
    // so we don't read the image just to find the cache is already populated.
    let (miss_records, _ws_paths) = {
        let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
        let all_paths = crate::db::queries::get_image_paths(&conn, &miss_ids)
            .map_err(|e| e.to_string())?;

        // Resolve every distinct workspace_id in one pass, reusing the cache.
        let mut ws_paths: HashMap<i64, std::path::PathBuf> = HashMap::new();
        for &(_, ws_id, _) in &all_paths {
            if !ws_paths.contains_key(&ws_id) {
                let p = get_workspace_path_cached(&thumb_state, &conn, ws_id)?;
                ws_paths.insert(ws_id, p);
            }
        }

        // Filter out images whose thumbnail already exists on disk.
        let cache = thumb_state.cache.lock().map_err(|e| e.to_string())?;
        let mut records: Vec<(i64, std::path::PathBuf)> = Vec::with_capacity(all_paths.len());
        for (id, ws_id, rel) in all_paths {
            let thumb_path = cache.cache_path(id);
            if thumb_path.exists() {
                // The cache file is on disk; let the frontend see it as a hit
                // by reading it back. (Cheaper than re-decoding the source.)
                if let Ok(bytes) = std::fs::read(&thumb_path) {
                    results.push(ThumbnailBatchEntry { image_id: id, bytes });
                }
                continue;
            }
            if let Some(p) = ws_paths.get(&ws_id) {
                records.push((id, p.join(&rel)));
            }
        }
        (records, ws_paths)
    };

    if miss_records.is_empty() {
        return Ok(results);
    }

    // Phase 3: Generate thumbnails sequentially under the cache lock.
    // This serializes image decoding but keeps memory bounded and avoids
    // spawning threads that all need to hold the lock anyway.
    let mut paths_to_update: Vec<(i64, String, u32, u32)> = Vec::with_capacity(miss_records.len());
    {
        let cache = thumb_state.cache.lock().map_err(|e| e.to_string())?;
        for (id, source_path) in &miss_records {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                cache.generate(source_path, *id)
            }));
            if let Ok(Ok((thumb_path, w, h))) = result {
                paths_to_update.push((*id, thumb_path, w, h));
            }
        }
    }

    // Phase 4: Persist thumbnail_path updates + read back generated bytes.
    if !paths_to_update.is_empty() {
        let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
        for (id, path, w, h) in &paths_to_update {
            crate::db::queries::update_thumbnail_path(&conn, *id, path, *w, *h).ok();
        }
        drop(conn);
        let cache = thumb_state.cache.lock().map_err(|e| e.to_string())?;
        for (id, _, _, _) in &paths_to_update {
            if let Some(bytes) = cache.get_bytes(*id) {
                results.push(ThumbnailBatchEntry { image_id: *id, bytes });
            }
        }
    }

    Ok(results)
}

#[tauri::command]
pub fn generate_thumbnail(
    state: State<AppState>,
    thumb_state: State<ThumbnailState>,
    image_id: i64,
) -> Result<(), String> {
    let gen_result: Result<(String, u32, u32), String> = {
        let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
        let record = crate::db::queries::get_image_detail(&conn, image_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Image not found".to_string())?;
        let ws = get_workspace_path_cached(&thumb_state, &conn, record.workspace_id)?;
        let source_path = ws.join(&record.relative_path);
        drop(conn);

        let cache = thumb_state.cache.lock().map_err(|e| e.to_string())?;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            cache.generate(&source_path, image_id)
        }));
        match result {
            Ok(Ok(out)) => Ok(out),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(format!("Failed to process image: {}", source_path.display())),
        }
    };

    if let Ok((ref path, w, h)) = gen_result {
        let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
        crate::db::queries::update_thumbnail_path(&conn, image_id, path, w, h)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Background thumbnail generation after scan: generates thumbnails for all
/// image_ids, emitting progress events. Skips files whose thumbnails already
/// exist on disk.
#[tauri::command]
pub fn generate_thumbnails_batch(
    state: State<AppState>,
    thumb_state: State<ThumbnailState>,
    app: tauri::AppHandle,
    image_ids: Vec<i64>,
) -> Result<(), String> {
    if image_ids.is_empty() {
        return Ok(());
    }

    // Single-query path fetch — replaces 1 + N queries for the per-image
    // workspace_id + relative_path lookup. We also skip ids whose
    // thumbnail already exists on disk in the same pass.
    let records: Vec<(i64, std::path::PathBuf)> = {
        let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
        let all_paths = crate::db::queries::get_image_paths(&conn, &image_ids)
            .map_err(|e| e.to_string())?;

        // Resolve distinct workspace paths once.
        let mut ws_paths: HashMap<i64, std::path::PathBuf> = HashMap::new();
        for &(_, ws_id, _) in &all_paths {
            if !ws_paths.contains_key(&ws_id) {
                let p = get_workspace_path_cached(&thumb_state, &conn, ws_id)?;
                ws_paths.insert(ws_id, p);
            }
        }

        let cache = thumb_state.cache.lock().map_err(|e| e.to_string())?;
        let mut recs = Vec::with_capacity(all_paths.len());
        for (id, ws_id, rel) in all_paths {
            let thumb_path = cache.cache_path(id);
            if thumb_path.exists() {
                continue;
            }
            if let Some(p) = ws_paths.get(&ws_id) {
                recs.push((id, p.join(&rel)));
            }
        }
        recs
    };

    if records.is_empty() {
        return Ok(());
    }

    let total = records.len() as i64;
    let mut generated: i64 = 0;
    let mut failed: i64 = 0;
    let flush_interval = 10;

    for chunk in records.chunks(flush_interval) {
        let mut paths_to_update: Vec<(i64, String, u32, u32)> = Vec::with_capacity(chunk.len());

        {
            let cache = thumb_state.cache.lock().map_err(|e| e.to_string())?;
            for &(id, ref src_path) in chunk {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    cache.generate(src_path, id)
                }));
                match result {
                    Ok(Ok((thumb_path, w, h))) => {
                        generated += 1;
                        paths_to_update.push((id, thumb_path, w, h));
                    }
                    _ => { failed += 1; }
                }
            }
        }

        if !paths_to_update.is_empty() {
            let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
            for (id, path, w, h) in &paths_to_update {
                crate::db::queries::update_thumbnail_path(&conn, *id, path, *w, *h).ok();
            }
        }

        app.emit("thumbnail-progress", crate::models::ThumbnailProgress {
            total,
            generated,
            failed,
            finished: generated + failed >= total,
        }).map_err(|e| e.to_string())?;
    }

    Ok(())
}