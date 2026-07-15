# Design: Workspace Sync (True Reconcile)

**Date:** 2026-07-15
**Status:** Approved
**Approach:** Reconcile (A) + Missing-file confirmation UI (A3) + (size, mtime) rename rebinding (R2)

## Goal

Replace the current destructive `rescan_workspace` (delete-all + reinsert) with a reconcile-style sync that:

1. Preserves user metadata (tags, ratings, notes, last_viewed_at) on files that are unchanged, modified, renamed, or moved.
2. Adds genuinely new files.
3. Marks files that disappeared from disk and lets the user decide whether to delete them, instead of silently wiping them.
4. Reports meaningful progress (`new_files` / `modified_files` counts that match the names) and stays correct under interruption.

## Non-Goals

- SHA-256 hashing during sync. Hashes remain deferred to the dedup path. (R2 heuristic uses only `file_size` + `created_at`.)
- Auto-deletion of files missing from disk. The user confirms via dialog.
- Workspace recycle bin / soft-delete with `deleted_at` column. Out of scope; schema stays unchanged.
- Background file-system watcher. Sync remains an explicit user action.
- Frontend test framework. No vitest / RTL setup; coverage stays manual.

## Decisions Locked In

| Question | Choice | Rationale |
|---|---|---|
| Sync semantics | **A — true reconcile** | Preserves metadata; foundation for future incremental features. |
| Disk-missing handling | **A3 — UI confirmation** | Schema unchanged; user keeps control; recoverable from "external drive unplugged" scenarios. |
| Rename / move rebinding | **R2 — (size, mtime) heuristic** | Zero extra I/O (both fields already stored); collision rate in personal libraries negligible. |

---

## Current Behavior

`workspaceStore.scan(id)` (Pinia store) → `invoke('rescan_workspace', { id })` → backend (`commands/scan.rs`):

1. `DELETE FROM images WHERE workspace_id = ?1` — hard-wipes all rows for the workspace.
2. `scan_workspace` walks the directory with `std::fs::read_dir`, builds `Vec<ImageRecord>` with `file_hash = None`, `width/height/thumbnail_path/notes/rating/last_viewed_at = NULL/None`.
3. `INSERT ... ON CONFLICT(workspace_id, relative_path) DO UPDATE` in 50-row transactions.

Failure modes observed:

- `image_tags` rows are deleted via `ON DELETE CASCADE`. Tags themselves stay in `tags` (global), but every chip in the sidebar for that workspace disappears.
- `rating`, `notes`, `thumbnail_path`, `last_viewed_at` are wiped.
- `ScanProgress.modified_files` is hardcoded to `0`; `new_files` is just `processed`. Both fields are unused by any UI.
- Path-not-exists check runs **after** the DELETE, so unplugging an external drive and clicking Sync leaves the workspace empty with an error.
- File rename / move is treated as "old image deleted + new image inserted" — metadata is lost.
- Disk thumbnails (`{id}.jpg`) become orphans; never regenerated.

---

## Proposed Behavior

### Backend: `commands/sync.rs`

Two Tauri commands:

```rust
#[derive(Serialize)]
pub struct SyncResult {
    pub totals: SyncTotals,
    pub deleted_candidates: Vec<DeletedCandidate>,
}

#[derive(Serialize)]
pub struct SyncTotals {
    pub kept: i64,
    pub modified: i64,
    pub added: i64,
    pub rebound: i64,
    pub deleted_candidates: i64,
    pub duration_ms: i64,
}

#[derive(Serialize)]
pub struct DeletedCandidate {
    pub id: i64,
    pub relative_path: String,
    pub file_name: String,
    pub rating: Option<i32>,
    pub last_viewed_at: Option<String>,
    pub tag_names: Vec<String>,
}

#[tauri::command]
pub fn sync_workspace(state: State<AppState>, app: AppHandle, id: i64)
    -> Result<SyncResult, String>;

#[tauri::command]
pub fn apply_sync_deletions(state: State<AppState>, ids: Vec<i64>)
    -> Result<usize, String>;
```

`rescan_workspace` and `scan_workspace` are **retained** unchanged. `rescan_workspace` is exposed only through a new "Advanced → Full reset" submenu (see Frontend).

### Backend: `scanner/` split

`scanner/mod.rs` is reorganized:

- `collect_image_files(dir, &mut Vec<PathBuf>)` — unchanged.
- `FsEntry { relative_path, file_name, size, mtime, file_created_at, format }` — new struct.
- `scan_directory(workspace_id, workspace_path) -> Vec<FsEntry>` — new; replaces the `Vec<ImageRecord>`-producing path. Does **not** call any DB code. Returns the FS truth.
- `reconcile(workspace_id, fs: Vec<FsEntry>, db: HashMap<String, DbEntry>) -> ReconcilePlan` — new pure function. Owns all classification logic.
- `DbEntry { id, size, mtime, thumbnail_path }` — new struct (subset of `ImageRecord`).
- `ReconcilePlan { kept: Vec<i64>, modified: Vec<(i64, FsEntry)>, added: Vec<FsEntry>, rebound: Vec<(i64, String, FsEntry)>, deleted: Vec<(String, DbEntry)> }` — new struct. `deleted` carries `(relative_path, DbEntry)` so the caller can build UI-facing `DeletedCandidate` rows without re-querying the DB.
- `apply_plan(conn, plan, workspace_id) -> Result<()>` — new; runs all writes inside one transaction.
- `load_index(conn, workspace_id) -> Result<HashMap<String, DbEntry>>` — new query in `db/queries.rs`.

`scan_directory_individually` is kept as a thin wrapper that calls `scan_directory` then `reconcile` then `apply_plan` for callers that still want the legacy all-in-one behavior — but `commands/scan.rs::rescan_workspace` is the only remaining caller, and the front-end no longer reaches it.

### Reconcile algorithm

Two-pass classification. No DB access inside the algorithm.

**Pass 1 — exact path match**

```
for fs in fs_entries:
  if db_by_path.contains(fs.relative_path):
    db = db_by_path.remove(fs.relative_path)
    if fs.size == db.size AND fs.mtime == db.mtime:
      plan.kept.push(db.id)
    else:
      plan.modified.push((db.id, fs))
  else:
    leftover_fs.push(fs)
```

**Pass 2 — (size, mtime) rebind (R2)**

```
db_unmatched = db_by_path.values().collect()
for fs in leftover_fs:
  candidates = db_unmatched.iter()
                           .filter(|db| db.size == fs.size && db.mtime == fs.mtime)
                           .collect::<Vec<_>>()
  match candidates.len() {
    1 => {
      let db = candidates[0];
      plan.rebound.push((db.id, fs.relative_path.clone(), fs));
      db_unmatched.retain(|d| d.id != db.id);
    }
    _ => plan.added.push(fs),  // 0 hits = truly new; >=2 = ambiguous → safe fallback
  }
plan.deleted = db_unmatched;  // anything still unmatched → "missing on disk" candidate
```

`plan.deleted` is **never** written by `apply_plan`. It is returned to the front-end for confirmation.

### `apply_plan` — single transaction, all-or-nothing

| Plan bucket | SQL | Effect on existing columns |
|---|---|---|
| `kept` | none | 0 I/O |
| `modified` | `UPDATE images SET file_size=?, created_at=?, updated_at=datetime('now'), thumbnail_path=NULL WHERE id=?` | `rating`, `notes`, `image_tags`, `last_viewed_at`, `format` preserved; `thumbnail_path` cleared so the background batch regens it |
| `added` | `INSERT INTO images (workspace_id, relative_path, file_name, file_size, file_hash, width, height, format, thumbnail_path, notes, rating, created_at, file_created_at, updated_at, indexed_at, last_viewed_at) VALUES (?, ?, ?, ?, NULL, NULL, NULL, ?, NULL, ?, NULL, ?, ?, datetime('now'), datetime('now'), NULL)` | `rating = 0`, `notes = ''`, `thumbnail_path = NULL` (column defaults) |
| `rebound` | `UPDATE images SET relative_path=?, file_name=?, updated_at=datetime('now') WHERE id=?` | `rating`, `notes`, `image_tags`, `last_viewed_at`, `thumbnail_path`, `file_hash` all preserved; `thumbnail_path` stays because thumbnails are stored as `{id}.jpg`, not by path |

`apply_sync_deletions(ids)` runs `DELETE FROM images WHERE id IN (...)` in one transaction. `image_tags.image_id ON DELETE CASCADE` handles tag membership cleanup. Tag definitions in the global `tags` table are **not** touched (orphans remain, per AGENTS.md invariant).

### Event emission

`sync_workspace` reuses the existing `scan-progress` event name and `ScanProgress` payload so `BrowserView.vue`'s existing listener (`unlistenScan`) works without changes:

```rust
app.emit("scan-progress", ScanProgress {
    total: fs_count,
    processed: cumulative_done,
    new_files: plan.added.len() as i64,
    modified_files: plan.modified.len() as i64,
    finished: true_or_false,
});
```

`ScanProgress` itself does not get new fields — the front-end does not currently render them; this just makes the data correct if it ever does.

### Frontend flow

```
User clicks [Rescan] on WorkspaceCard
  -> confirm(workspace.syncConfirm)
  -> workspaceStore.sync(id)
       |- listen('scan-progress')  -> updates workspaceStore.scanProgress
       |- invoke('sync_workspace', { id }) -> SyncResult
       |- if SyncResult.deletedCandidates.length > 0:
       |    MissingFilesDialog opens with the candidate list
       |      |- [Delete selected]  -> invoke('apply_sync_deletions', { ids }) -> toast
       |      |- [Keep all]         -> dialog closes; DB rows remain
       |      \- [Cancel]           -> same as Keep all
       \- scanProgress = finished
```

`MissingFilesDialog.vue` (new component under `src/components/workspace/`) reuses the existing `.dialog-overlay` / `.dialog` styles from `MoveToWorkspaceDialog.vue`. Layout:

```
+- Missing files (12) --------------------------------[x]-+
|  Disk no longer has these files. Review before          |
|  deleting. Tags/ratings will be lost.                   |
|  +----------------------------------------------------+  |
|  | [v] IMG_0001.jpg   ****  travel, scenery           |  |
|  | [v] IMG_0002.jpg   ***                             |  |
|  | [ ] IMG_0003.jpg                                   |  |
|  | ...                                                |  |
|  +----------------------------------------------------+  |
|                          [Keep all]  [Delete selected]  |
+----------------------------------------------------------+
```

### Store changes

`src/stores/workspace.ts`:

```ts
async function sync(id: number): Promise<SyncResult>  // renamed from scan()
async function applyDeletions(ids: number[]): Promise<number>
```

Callers updated: `src/views/WorkspaceHome.vue:43` (`store.scan` -> `store.sync`). `BrowserView.vue`'s listener on `scan-progress` is unchanged.

### Destructive entry retained

`rescan_workspace` (full wipe) is preserved for the rare "DB is wrong, start over" case. Wired through a new `WorkspaceCard` submenu "Advanced -> Full reset" which fires a two-stage confirmation:

1. `confirm(workspace.advancedReset)` — warns about metadata loss.
2. `confirm(workspace.fullResetConfirm)` — requires user to click through.

No new commands; just a UI affordance over the existing `invoke('rescan_workspace', { id })`.

### Error handling matrix

| Failure | Behavior | UX |
|---|---|---|
| Workspace path missing | `Err` returned before reconcile; no DB writes, no events | toast "Workspace path not accessible: ..." |
| `fs::read_dir` permission denied on subfolder | `collect_image_files` silently skips (matches current code) | affected files appear in `deletedCandidates`; user can recognize this as a permission issue |
| `load_index` DB error | Pass-through `Err` | toast + state resets to idle |
| `apply_plan` mid-failure | Single transaction `ROLLBACK`; DB returns to pre-sync state | toast "Sync failed, rolled back" + reason; retry is safe |
| Partial `apply_sync_deletions` success | Single transaction; returns actual row count | toast "Deleted N of M selected" |
| User picks Keep all in missing dialog | No `apply_sync_deletions` call; rows remain in DB | dialog closes; tags/ratings/notes intact; next sync re-lists them as candidates |
| Double-click on [Rescan] | New `syncingIds: Set<number>` guard in `WorkspaceHome.vue` | button shows spinner / is disabled while syncing |
| Metadata edit during sync | SQLite single-writer; either pre-applies (preserved) or post-applies (no loss) | brief delay; no functional regression |
| Many-to-many (size, mtime) collisions | No heuristic; both sides go to `added` / `deleted` candidates | user reviews in dialog |
| Empty workspace (FS empty + DB empty) | Empty plan; `deletedCandidates.length == 0` | no dialog; toast "No changes" |

### Concurrency / interruption safety

- `apply_plan` is one transaction. App crash mid-write -> ROLLBACK; FS unchanged; retry is idempotent (`kept` is 0 I/O; `modified`/`rebound` produce the same end state).
- `reconcile()` is pure (no locks, no I/O). `load_index` takes the DB mutex once for a short SELECT and releases before `apply_plan`.
- `apply_sync_deletions` is independent of sync — runs its own short transaction; safe to invoke at any time.

### Performance

- `load_index` for 10k rows: ~50-100 ms (single SELECT, in-memory projection of 5 fields).
- `fs::read_dir` walk for 10k files: SSD ~1-2 s, HDD ~5-10 s. Bottleneck unchanged from current code.
- `reconcile()` for 10k + 10k: O(n + m) HashMap ops, ~10-50 ms.
- `apply_plan` for 10k INSERT/UPDATE in 50-row batches: ~1 s on SSD.
- Total sync time for a 10k-image workspace: 2-12 s depending on disk. 100k images: 20-120 s, acceptable for a one-shot operation under user supervision.

### i18n

New keys in `src/i18n/locales/zh-CN.ts` and `en.ts` (style consistent with existing `workspace.*` / `common.*` namespaces):

```
workspace.syncConfirm          # Top-level confirm message
workspace.syncResultToast      # Post-sync summary toast
workspace.advancedReset        # "Advanced -> Full reset" submenu
workspace.fullResetConfirm     # Destructive confirm text
missing.title                  # "Missing files (N)"
missing.description            # "Disk no longer has these files..."
missing.keepAll
missing.deleteSelected         # "Delete selected (N)"
missing.cancel
missing.empty                  # "Nothing selected"
sync.counts.added              # "Added N"
sync.counts.modified
sync.counts.kept
sync.counts.rebound
sync.counts.deletedCandidates
```

---

## Files Touched

**Backend (Rust)**

- `src-tauri/src/scanner/mod.rs` — split into `scan_directory` + new `reconcile.rs` module. Add `FsEntry`, `DbEntry`, `ReconcilePlan` structs.
- `src-tauri/src/scanner/reconcile.rs` — new file; `reconcile()` pure function + `#[cfg(test)] mod tests`.
- `src-tauri/src/db/queries.rs` — add `load_index(conn, workspace_id) -> Result<HashMap<String, DbEntry>>`.
- `src-tauri/src/commands/sync.rs` — new file; `sync_workspace`, `apply_sync_deletions`, types.
- `src-tauri/src/commands/scan.rs` — keep `rescan_workspace` and `scan_workspace` unchanged. They become the "Full reset" path.
- `src-tauri/src/lib.rs` (or wherever commands are registered) — register the two new commands.
- `src-tauri/Cargo.toml` — add `tempfile` as a dev-dependency (for integration tests).

**Frontend (Vue / TypeScript)**

- `src/stores/workspace.ts` — rename `scan` -> `sync`; add `applyDeletions`; add `syncingIds` set + `isSyncing(id)` helper.
- `src/views/WorkspaceHome.vue` — `store.scan(id)` -> `store.sync(id)`; add guard against double-clicks; add Advanced submenu.
- `src/components/workspace/MissingFilesDialog.vue` — new file.
- `src/components/workspace/WorkspaceCard.vue` — add Advanced -> Full reset submenu; surface `syncing` spinner.
- `src/i18n/locales/zh-CN.ts`, `src/i18n/locales/en.ts` — add new keys.

**Unchanged**

- `src-tauri/src/db/schema.rs` (0 schema changes).
- FTS5 triggers (0 changes).
- `src/components/browser/BrowserView.vue` (event listener unchanged).
- `src/stores/images.ts`, `src/stores/tags.ts` (filters watcher unchanged).
- `src/components/browser/ThumbnailGrid.vue` and thumbnail generator (reused as-is; thumbnail regen triggered by existing `finished=true` handler).
- All other workspace / browser / dedup views.

---

## Testing

### Unit tests — `src-tauri/src/scanner/reconcile.rs`

Twelve cases:

1. Empty FS + empty DB -> empty plan.
2. FS has rows, DB empty -> all `added`.
3. DB has rows, FS empty -> all `deleted`.
4. Same path + same (size, mtime) -> `kept`.
5. Same path, size differs -> `modified`.
6. Same path, mtime differs -> `modified`.
7. Path differs, (size, mtime) matches exactly one DB row -> `rebound`.
8. File moved to subdirectory, (size, mtime) matches -> `rebound`.
9. (size, mtime) collisions: many-to-one -> ambiguous, both sides fall back to `added` / `deleted`.
10. (size, mtime) matches nothing -> `added`.
11. Mixed plan: `kept` + `modified` + `added` + `rebound` + `deleted` all present simultaneously.
12. `workspace_id` is passed through unchanged on every bucket.

### Integration test — `src-tauri/src/commands/sync.rs` (in-module `#[cfg(test)]`)

Uses `tempfile::tempdir()` + a fresh SQLite file:

- After a sync with mixed plan, verify:
  - `kept` rows: byte-identical to pre-sync state.
  - `modified` rows: `file_size`, `created_at`, `thumbnail_path` updated; `rating`, `notes`, `image_tags`, `last_viewed_at` preserved.
  - `added` rows: present with `rating = 0`, `notes = ''`, `thumbnail_path = NULL`.
  - `rebound` rows: `relative_path` updated; everything else preserved.
- `apply_sync_deletions` cascade: `image_tags` rows referencing deleted images are gone; `tags` definitions remain.
- Crash simulation: kill the connection mid-`apply_plan` (close `Connection` before commit) -> DB returns to pre-sync state (no orphan rows, no missing rows).

### Frontend

No automated tests added (consistent with project). Manual verification:

- WorkspaceCard [Rescan] on a workspace with edits outside the app -> missing dialog lists exactly those files.
- WorkspaceCard [Rescan] after editing a file's content (size changes) -> `modified` counter increments, no dialog.
- WorkspaceCard [Rescan] after renaming a folder -> `rebound` counter increments, no dialog, tags intact.
- WorkspaceCard [Advanced -> Full reset] -> two confirms -> DB wiped for that workspace.
- Sync on a workspace pointing at an unmounted drive -> toast error, no data loss.

---

## Migration / Backward Compatibility

- 0 schema changes. Existing DBs work without migration.
- `rescan_workspace` command retained; reachable only via the Advanced submenu.
- `scan()` -> `sync()` rename in `workspaceStore` is internal; `BrowserView`'s `scan-progress` listener is unchanged.
- Any user that had tags / ratings on a workspace and triggers the new sync: metadata is preserved (improvement over current behavior).
- Any user that relied on sync being destructive: must now use Advanced -> Full reset (intentional).

## Out of Scope (Future Work)

- SHA-256-based rename detection (R3). Can be added later as a `reconcile_with_hashes()` variant that runs after the (size, mtime) pass for ambiguous cases.
- File-system watcher for automatic sync triggers.
- Background thumbnail regeneration scheduling (currently piggybacks on the existing `generate_thumbnails_batch` call from `BrowserView`).
- Soft-delete / workspace recycle bin with `deleted_at` column.
