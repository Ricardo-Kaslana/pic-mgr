use std::collections::HashMap;
use std::time::Instant;

use rusqlite::params;
use tauri::{AppHandle, Emitter, State};

use crate::db::queries;
use crate::models::ScanProgress;
use crate::scanner::reconcile::{self, DbEntry, FsEntry, ReconcilePlan};
use crate::scanner::scan_directory;
use crate::AppState;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub totals: SyncTotals,
    pub deleted_candidates: Vec<DeletedCandidate>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncTotals {
    pub kept: i64,
    pub modified: i64,
    pub added: i64,
    pub rebound: i64,
    pub deleted_candidates: i64,
    pub duration_ms: i64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletedCandidate {
    pub id: i64,
    pub relative_path: String,
    pub file_name: String,
    pub rating: Option<i32>,
    pub last_viewed_at: Option<String>,
    pub tag_names: Vec<String>,
}

#[tauri::command]
pub fn sync_workspace(
    state: State<AppState>,
    app: AppHandle,
    id: i64,
) -> Result<SyncResult, String> {
    let started = Instant::now();

    // Step 1: get workspace path (short lock).
    let ws_path = {
        let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
        let ws = queries::get_workspace(&conn, id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Workspace not found".to_string())?;
        std::path::PathBuf::from(&ws.path)
    };

    if !ws_path.exists() {
        return Err(format!(
            "Workspace path not accessible: {}",
            ws_path.display()
        ));
    }

    // Step 2: scan directory -- no DB lock held during fs walk.
    let fs: Vec<FsEntry> = scan_directory(id, &ws_path);

    // Step 3: load index -- short DB lock.
    let db_index: HashMap<String, DbEntry> = {
        let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
        queries::load_index(&conn, id).map_err(|e| e.to_string())?
    };

    // Step 4: classify (pure function).
    let plan: ReconcilePlan = reconcile::reconcile(id, fs, db_index);

    // Step 5: apply kept / modified / added / rebound in one transaction.
    // (Excludes `deleted`, which the user must confirm via MissingFilesDialog.)
    {
        let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
        reconcile::apply_plan(&conn, id, &plan).map_err(|e| e.to_string())?;
    }

    // Step 6: build deleted_candidates from the plan's deleted entries,
    // joining back to fetch rating / last_viewed_at / tag_names for the UI.
    let deleted_candidates = {
        let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
        let mut out = Vec::with_capacity(plan.deleted.len());
        for (rel, db_entry) in &plan.deleted {
            let rating: Option<i32> = conn
                .query_row(
                    "SELECT rating FROM images WHERE id = ?1",
                    params![db_entry.id],
                    |row| row.get(0),
                )
                .ok();
            let last_viewed_at: Option<String> = conn
                .query_row(
                    "SELECT last_viewed_at FROM images WHERE id = ?1",
                    params![db_entry.id],
                    |row| row.get(0),
                )
                .ok()
                .flatten();
            let tag_names: Vec<String> = {
                let mut stmt = conn
                    .prepare(
                        "SELECT t.name FROM tags t
                         INNER JOIN image_tags it ON t.id = it.tag_id
                         WHERE it.image_id = ?1
                         ORDER BY t.name",
                    )
                    .map_err(|e| e.to_string())?;
                let rows = stmt
                    .query_map(params![db_entry.id], |row| {
                        row.get::<_, String>(0)
                    })
                    .map_err(|e| e.to_string())?;
                rows.filter_map(|r| r.ok()).collect()
            };
            out.push(DeletedCandidate {
                id: db_entry.id,
                relative_path: rel.clone(),
                file_name: std::path::Path::new(rel)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(rel.as_str())
                    .to_string(),
                rating,
                last_viewed_at,
                tag_names,
            });
        }
        out
    };

    // Step 7: emit finished=true LAST so the UI's existing listener
    // (BrowserView.unlistenScan -> fetchImages + fetchTags + background
    // thumbnail regen) only fires after all writes have committed and the
    // deleted_candidates list is ready to be consumed.
    let total = plan.kept.len() as i64
        + plan.modified.len() as i64
        + plan.added.len() as i64
        + plan.rebound.len() as i64;
    let _ = app.emit(
        "scan-progress",
        ScanProgress {
            total,
            processed: total,
            new_files: plan.added.len() as i64,
            modified_files: plan.modified.len() as i64,
            finished: true,
        },
    );

    Ok(SyncResult {
        totals: SyncTotals {
            kept: plan.kept.len() as i64,
            modified: plan.modified.len() as i64,
            added: plan.added.len() as i64,
            rebound: plan.rebound.len() as i64,
            deleted_candidates: plan.deleted.len() as i64,
            duration_ms: started.elapsed().as_millis() as i64,
        },
        deleted_candidates,
    })
}

#[tauri::command]
pub fn apply_sync_deletions(
    state: State<AppState>,
    ids: Vec<i64>,
) -> Result<usize, String> {
    if ids.is_empty() {
        return Ok(0);
    }
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;
    let placeholders: Vec<String> = (0..ids.len()).map(|i| format!("?{}", i + 1)).collect();
    let sql = format!(
        "DELETE FROM images WHERE id IN ({})",
        placeholders.join(",")
    );
    let params_vec: Vec<&dyn rusqlite::ToSql> =
        ids.iter().map(|i| i as &dyn rusqlite::ToSql).collect();
    let result = conn.execute(&sql, params_vec.as_slice());
    match result {
        Ok(n) => {
            conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
            Ok(n)
        }
        Err(e) => {
            conn.execute_batch("ROLLBACK").ok();
            Err(e.to_string())
        }
    }
}
