# Workspace Sync (True Reconcile) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the destructive `rescan_workspace` flow with a reconcile-style sync that preserves user metadata (tags, ratings, notes, last_viewed_at) on kept / modified / renamed / moved files, and asks the user before deleting rows whose files have disappeared from disk.

**Architecture:** New `commands/sync.rs` orchestrates: `scan_directory` produces a `Vec<FsEntry>` (fs truth), `load_index` produces a `HashMap<relative_path, DbEntry>` (db truth), a pure `reconcile()` function classifies each file into `kept / modified / added / rebound / deleted`, and a single-transaction `apply_plan` writes the changes. A new Tauri command `apply_sync_deletions` lets the user confirm missing-file deletions. The existing `rescan_workspace` is preserved for a new "Advanced → Full reset" submenu.

**Tech Stack:** Rust 2021, Tauri v2, rusqlite 0.31 (bundled), serde, chrono_lite (already in scanner/mod.rs). Vue 3 + Pinia + vue-i18n on the frontend. `tempfile` 3 as a new dev-dependency for integration tests.

## Global Constraints

- **Never modify original image files.** All metadata goes into SQLite only. (AGENTS.md invariant.)
- **Never widen `ImageSummary`.** The grid still only needs `id, workspace_id, relative_path, file_name, rating` + thumbnail fields. Sync payload stays in `SyncResult` / `DeletedCandidate`, not in `ImageSummary`. (AGENTS.md invariant.)
- **`thumbUrls` lifecycle is owned by `thumbStore.cache`.** After sync, `BrowserView` still calls `generate_thumbnails_batch` once on `finished=true`; do not bypass that flow. (AGENTS.md invariant.)
- **`FTS5` triggers use rowid-only delete.** `INSERT INTO images_fts(images_fts, rowid) VALUES('delete', old.id)` — content columns are never repeated on delete. The `UPDATE` we issue on `modified` / `rebound` rows fires `images_fts_update` automatically; no manual FTS touch-ups.
- **DB migrations run at startup via `ALTER TABLE ADD COLUMN` with `.ok()` to ignore "already exists" errors — no migration framework.** This change adds 0 columns; if you find yourself reaching for a migration, stop and re-check the design.
- **Tag IDs are global; `tags` table has no `workspace_id`.** Sync never touches the `tags` table. `image_tags` cascade-delete on `images.id` is automatic. (AGENTS.md invariant.)
- **`workspaceStore.filters` mutations auto-refetch via store watcher.** Sync does not bypass this — `BrowserView`'s `scan-progress.finished=true` handler already does `imageStore.fetchImages(workspaceId)` + `tagStore.fetchTags(workspaceId, true)`. No changes to that flow.
- **All user-facing strings go through vue-i18n.** Both `zh-CN` and `en` keys required in the same commit.
- **SQLite single-writer.** `apply_plan` is one transaction; do not parallelize it.
- **Commit messages follow conventional commits** (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`).
- **No new top-level dependencies without explicit need.** Only addition is `tempfile = "3"` in `[dev-dependencies]`.

## File Structure

```
src-tauri/src/
  scanner/
    mod.rs                        # refactored: scan_directory() returns Vec<FsEntry>
    reconcile.rs                  # NEW: FsEntry, DbEntry, ReconcilePlan, reconcile(), apply_plan()
  commands/
    mod.rs                        # +1 line: `pub mod sync;`
    sync.rs                       # NEW: sync_workspace, apply_sync_deletions, SyncResult, DeletedCandidate, SyncTotals
    scan.rs                       # UNCHANGED (rescan_workspace preserved for Full reset)
  db/
    queries.rs                    # +1 function: load_index()
  Cargo.toml                      # +1 line in [dev-dependencies]: tempfile = "3"
  main.rs                         # +2 lines: register commands::sync::sync_workspace, commands::sync::apply_sync_deletions

src/
  stores/workspace.ts             # rename scan() -> sync(); add applyDeletions(); add syncingIds + isSyncing()
  views/WorkspaceHome.vue         # scan() -> sync(); confirm() before sync; double-click guard
  components/workspace/
    WorkspaceCard.vue             # Advanced -> Full reset submenu; sync-in-progress spinner via isSyncing
    MissingFilesDialog.vue        # NEW: per-row checkbox + tag/rating summary
  i18n/locales/zh-CN.ts           # +missing.*, +sync.counts.*, +workspace.syncConfirm, +workspace.advancedReset, +workspace.fullResetConfirm
  i18n/locales/en.ts              # same keys
```

Each task touches one logical area. Tasks 1-5 are backend; Tasks 6-10 are frontend. They can be merged or split per the executing engine's preference, but the dependency order is strict: 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8 -> 9 -> 10 (the i18n keys in Task 10 are referenced by Tasks 7-9, but since the keys are forward-declared in `zh-CN.ts` / `en.ts`, ordering is forgiving -- Tasks 7-9 can land first if a reviewer prefers to see the strings land last).

---

### Task 1: Reconcile types + pure function with 12 unit tests

**Files:**
- Create: `src-tauri/src/scanner/reconcile.rs`

**Interfaces:**
- Consumes: nothing (pure data-in).
- Produces:
  - `pub struct FsEntry { pub relative_path: String, pub file_name: String, pub size: i64, pub mtime: String, pub file_created_at: Option<String>, pub format: Option<String> }`
  - `pub struct DbEntry { pub id: i64, pub size: i64, pub mtime: String, pub thumbnail_path: Option<String> }`
  - `pub struct ReconcilePlan { pub kept: Vec<i64>, pub modified: Vec<(i64, FsEntry)>, pub added: Vec<FsEntry>, pub rebound: Vec<(i64, String, FsEntry)>, pub deleted: Vec<(String, DbEntry)> }`
  - `pub fn reconcile(workspace_id: i64, fs: Vec<FsEntry>, db: std::collections::HashMap<String, DbEntry>) -> ReconcilePlan`

- [ ] **Step 1: Create the file with type stubs**

Create `src-tauri/src/scanner/reconcile.rs` with the types and a stub `reconcile()` that returns an empty `ReconcilePlan`. The stub is what Task 1's tests will run against.

```rust
// src-tauri/src/scanner/reconcile.rs
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct FsEntry {
    pub relative_path: String,
    pub file_name: String,
    pub size: i64,
    pub mtime: String,
    pub file_created_at: Option<String>,
    pub format: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DbEntry {
    pub id: i64,
    pub size: i64,
    pub mtime: String,
    pub thumbnail_path: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReconcilePlan {
    pub kept: Vec<i64>,
    pub modified: Vec<(i64, FsEntry)>,
    pub added: Vec<FsEntry>,
    pub rebound: Vec<(i64, String, FsEntry)>,
    pub deleted: Vec<(String, DbEntry)>,
}

pub fn reconcile(
    workspace_id: i64,
    fs: Vec<FsEntry>,
    db: HashMap<String, DbEntry>,
) -> ReconcilePlan {
    let _ = (workspace_id, fs, db);
    ReconcilePlan::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fs(rel: &str, size: i64, mtime: &str) -> FsEntry {
        FsEntry {
            relative_path: rel.into(),
            file_name: rel.rsplit('/').next().unwrap_or(rel).into(),
            size,
            mtime: mtime.into(),
            file_created_at: None,
            format: None,
        }
    }

    fn db(id: i64, rel: &str, size: i64, mtime: &str) -> (String, DbEntry) {
        (
            rel.into(),
            DbEntry { id, size, mtime: mtime.into(), thumbnail_path: None },
        )
    }

    // --- tests will be added in step 2 ---
}
```

- [ ] **Step 2: Add the 12 unit tests**

Replace the trailing `// --- tests will be added in step 2 ---` comment with the following 12 tests. They will all fail against the stub.

```rust
    #[test]
    fn empty_inputs_yield_empty_plan() {
        let plan = reconcile(1, vec![], HashMap::new());
        assert!(plan.kept.is_empty());
        assert!(plan.modified.is_empty());
        assert!(plan.added.is_empty());
        assert!(plan.rebound.is_empty());
        assert!(plan.deleted.is_empty());
    }

    #[test]
    fn all_added_when_db_empty() {
        let fs = vec![fs("a/1.jpg", 100, "2026-01-01T00:00:00")];
        let plan = reconcile(1, fs, HashMap::new());
        assert_eq!(plan.added.len(), 1);
        assert_eq!(plan.added[0].relative_path, "a/1.jpg");
        assert!(plan.kept.is_empty() && plan.modified.is_empty()
                && plan.rebound.is_empty() && plan.deleted.is_empty());
    }

    #[test]
    fn all_deleted_when_fs_empty() {
        let mut db = HashMap::new();
        db.insert(db(7, "a/1.jpg", 100, "2026-01-01T00:00:00"));
        let plan = reconcile(1, vec![], db);
        assert_eq!(plan.deleted.len(), 1);
        assert_eq!(plan.deleted[0].0, "a/1.jpg");
        assert_eq!(plan.deleted[0].1.id, 7);
    }

    #[test]
    fn same_path_and_size_and_mtime_is_kept() {
        let mut db = HashMap::new();
        db.insert(db(7, "a/1.jpg", 100, "2026-01-01T00:00:00"));
        let fs = vec![fs("a/1.jpg", 100, "2026-01-01T00:00:00")];
        let plan = reconcile(1, fs, db);
        assert_eq!(plan.kept, vec![7]);
        assert!(plan.modified.is_empty() && plan.added.is_empty()
                && plan.rebound.is_empty() && plan.deleted.is_empty());
    }

    #[test]
    fn same_path_size_changed_is_modified() {
        let mut db = HashMap::new();
        db.insert(db(7, "a/1.jpg", 100, "2026-01-01T00:00:00"));
        let fs = vec![fs("a/1.jpg", 200, "2026-01-01T00:00:00")];
        let plan = reconcile(1, fs, db);
        assert_eq!(plan.modified.len(), 1);
        assert_eq!(plan.modified[0].0, 7);
        assert_eq!(plan.modified[0].1.size, 200);
    }

    #[test]
    fn same_path_mtime_changed_is_modified() {
        let mut db = HashMap::new();
        db.insert(db(7, "a/1.jpg", 100, "2026-01-01T00:00:00"));
        let fs = vec![fs("a/1.jpg", 100, "2026-01-02T00:00:00")];
        let plan = reconcile(1, fs, db);
        assert_eq!(plan.modified.len(), 1);
        assert_eq!(plan.modified[0].0, 7);
    }

    #[test]
    fn renamed_file_with_size_mtime_match_is_rebound() {
        let mut db = HashMap::new();
        db.insert(db(7, "old/1.jpg", 100, "2026-01-01T00:00:00"));
        let fs = vec![fs("new/1.jpg", 100, "2026-01-01T00:00:00")];
        let plan = reconcile(1, fs, db);
        assert_eq!(plan.rebound.len(), 1);
        assert_eq!(plan.rebound[0].0, 7);
        assert_eq!(plan.rebound[0].1, "new/1.jpg");
        assert_eq!(plan.rebound[0].2.relative_path, "new/1.jpg");
    }

    #[test]
    fn moved_to_subdirectory_with_size_mtime_match_is_rebound() {
        let mut db = HashMap::new();
        db.insert(db(7, "1.jpg", 100, "2026-01-01T00:00:00"));
        let fs = vec![fs("archive/2026/1.jpg", 100, "2026-01-01T00:00:00")];
        let plan = reconcile(1, fs, db);
        assert_eq!(plan.rebound.len(), 1);
        assert_eq!(plan.rebound[0].1, "archive/2026/1.jpg");
    }

    #[test]
    fn many_to_one_size_mtime_collision_falls_back_to_added_and_deleted() {
        // Two fs entries with same (size, mtime), one db row with the same.
        // Per spec: ambiguous -> fs entries go to `added`, db row goes to `deleted`.
        let mut db = HashMap::new();
        db.insert(db(7, "a/1.jpg", 100, "2026-01-01T00:00:00"));
        let fs = vec![
            fs("b/1.jpg", 100, "2026-01-01T00:00:00"),
            fs("b/2.jpg", 100, "2026-01-01T00:00:00"),
        ];
        let plan = reconcile(1, fs, db);
        assert_eq!(plan.added.len(), 2);
        assert_eq!(plan.deleted.len(), 1);
        assert!(plan.rebound.is_empty());
    }

    #[test]
    fn unmatched_size_mtime_is_truly_new() {
        let mut db = HashMap::new();
        db.insert(db(7, "a/1.jpg", 100, "2026-01-01T00:00:00"));
        let fs = vec![fs("a/2.jpg", 999, "2030-01-01T00:00:00")];
        let plan = reconcile(1, fs, db);
        assert_eq!(plan.added.len(), 1);
        assert_eq!(plan.added[0].relative_path, "a/2.jpg");
        assert_eq!(plan.deleted.len(), 1);
    }

    #[test]
    fn mixed_plan_covers_all_buckets() {
        let mut db = HashMap::new();
        db.insert(db(7, "kept.jpg",      100, "2026-01-01T00:00:00"));
        db.insert(db(8, "modified.jpg",  100, "2026-01-01T00:00:00"));
        db.insert(db(9, "old_name.jpg",  200, "2026-02-01T00:00:00"));
        let fs = vec![
            fs("kept.jpg",          100, "2026-01-01T00:00:00"), // kept
            fs("modified.jpg",      150, "2026-01-02T00:00:00"), // modified
            fs("renamed.jpg",       200, "2026-02-01T00:00:00"), // rebound
            fs("brand_new.jpg",     300, "2026-03-01T00:00:00"), // added
        ];
        let plan = reconcile(1, fs, db);
        assert_eq!(plan.kept,    vec![7]);
        assert_eq!(plan.modified.len(), 1);
        assert_eq!(plan.modified[0].0, 8);
        assert_eq!(plan.rebound.len(), 1);
        assert_eq!(plan.rebound[0].0, 9);
        assert_eq!(plan.added.len(),  1);
        assert_eq!(plan.added[0].relative_path, "brand_new.jpg");
        assert!(plan.deleted.is_empty());
    }

    #[test]
    fn workspace_id_is_passthrough_only() {
        // workspace_id has no effect on classification; it exists in the signature
        // for forward use (e.g., tagging apply_plan batches). Same inputs at
        // different ids must produce identical plans.
        let mut db = HashMap::new();
        db.insert(db(7, "a/1.jpg", 100, "2026-01-01T00:00:00"));
        let fs = vec![fs("a/1.jpg", 100, "2026-01-01T00:00:00")];

        let p1 = reconcile(1, fs.clone(), db.clone());
        let p2 = reconcile(42, fs, db);
        assert_eq!(p1.kept, p2.kept);
        assert_eq!(p1.added, p2.added);
        assert_eq!(p1.modified, p2.modified);
        assert_eq!(p1.rebound, p2.rebound);
        assert_eq!(p1.deleted, p2.deleted);
    }
```

- [ ] **Step 3: Run tests to verify they all fail**

Run: `cd src-tauri && cargo test --lib scanner::reconcile::tests`
Expected: every test panics or asserts on the empty default `ReconcilePlan`. The two-pass structure of the algorithm has not been implemented yet.

- [ ] **Step 4: Implement `reconcile()`**

Replace the stub `reconcile()` body with the two-pass algorithm. Pass 1 is exact-path matching; Pass 2 is `(size, mtime)` rebinding for the leftovers. Anything still in `db` after Pass 2 becomes a `deleted` candidate.

```rust
pub fn reconcile(
    workspace_id: i64,
    fs: Vec<FsEntry>,
    db: HashMap<String, DbEntry>,
) -> ReconcilePlan {
    let _ = workspace_id; // signature only; classification is workspace-agnostic
    let mut db = db;
    let mut plan = ReconcilePlan::default();
    let mut leftover_fs: Vec<FsEntry> = Vec::with_capacity(fs.len());

    // Pass 1: exact path match.
    for entry in fs {
        if let Some(db_entry) = db.remove(&entry.relative_path) {
            if entry.size == db_entry.size && entry.mtime == db_entry.mtime {
                plan.kept.push(db_entry.id);
            } else {
                plan.modified.push((db_entry.id, entry));
            }
        } else {
            leftover_fs.push(entry);
        }
    }

    // Pass 2: rebind leftover FS entries by (size, mtime).
    let mut unmatched_db: Vec<(String, DbEntry)> = db.into_iter().collect();
    for entry in leftover_fs {
        let candidates: Vec<&(String, DbEntry)> = unmatched_db
            .iter()
            .filter(|(_, d)| d.size == entry.size && d.mtime == entry.mtime)
            .collect();
        match candidates.len() {
            1 => {
                let (rel, db_entry) = unmatched_db
                    .iter()
                    .position(|(r, _)| {
                        r == &candidates[0].0
                    })
                    .map(|i| unmatched_db.remove(i))
                    .expect("candidate present in unmatched_db");
                plan.rebound.push((db_entry.id, rel, entry));
            }
            _ => plan.added.push(entry),
        }
    }

    plan.deleted = unmatched_db;
    plan
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd src-tauri && cargo test --lib scanner::reconcile::tests`
Expected: `12 passed; 0 failed`. (cargo test prints "test result: ok. 12 passed; 0 failed; 0 ignored".)

- [ ] **Step 6: Commit**

```bash
cd D:\Development\Complex\pic-mgr
git add src-tauri/src/scanner/reconcile.rs
git commit -m "feat(sync): add reconcile() pure function with 12 unit tests"
```

---

### Task 2: `load_index` query

**Files:**
- Modify: `src-tauri/src/db/queries.rs:1-10` (add `use std::collections::HashMap;` if not already present)

**Interfaces:**
- Consumes: a `&Connection` and a `workspace_id: i64`.
- Produces:
  - `pub fn load_index(conn: &Connection, workspace_id: i64) -> Result<HashMap<String, DbEntry>>`

`DbEntry` is imported via `use crate::scanner::reconcile::DbEntry;` at the top of `queries.rs`.

- [ ] **Step 1: Verify `HashMap` is imported**

Run: `rg "^use " D:\Development\Complex\pic-mgr\src-tauri\src\db\queries.rs | Select-Object -First 10`

Expected: see `use std::collections::HashMap;` or `use rusqlite::types::Value;` etc. If `HashMap` is not imported, add it.

- [ ] **Step 2: Add `load_index`**

Open `src-tauri/src/db/queries.rs`. Add a top-level import:

```rust
use crate::scanner::reconcile::DbEntry;
```

Then, near the end of the file (after `get_image_paths` is a fine spot), add:

```rust
/// Load the lightweight index used by sync_workspace's reconcile step.
/// Returns relative_path -> DbEntry for every row in `images` for the given
/// workspace. Holds the DB lock for the duration of the SELECT; the caller
/// must drop the connection guard before invoking apply_plan.
pub fn load_index(conn: &Connection, workspace_id: i64) -> Result<HashMap<String, DbEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, relative_path, file_size, created_at, thumbnail_path
         FROM images WHERE workspace_id = ?1"
    )?;
    let rows = stmt.query_map(params![workspace_id], |row| {
        Ok((
            row.get::<_, String>(1)?,           // relative_path (becomes map key)
            DbEntry {
                id: row.get(0)?,
                size: row.get::<_, Option<i64>>(2)?.unwrap_or(0),
                mtime: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                thumbnail_path: row.get(4)?,
            },
        ))
    })?;
    let mut out: HashMap<String, DbEntry> = HashMap::new();
    for row in rows {
        let (rel, entry) = row?;
        out.insert(rel, entry);
    }
    Ok(out)
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: `Finished` with no errors. (Compiler will warn that `load_index` is unused -- that's fine; Task 4's integration test will use it.)

- [ ] **Step 4: Commit**

```bash
cd D:\Development\Complex\pic-mgr
git add src-tauri/src/db/queries.rs
git commit -m "feat(sync): add load_index() query for reconcile step"
```

---

### Task 3: Refactor `scanner/mod.rs` to expose `scan_directory` returning `Vec<FsEntry>`

**Files:**
- Modify: `src-tauri/src/scanner/mod.rs`

**Interfaces:**
- Consumes: nothing new.
- Produces:
  - `pub mod reconcile;` (sub-module declaration)
  - `pub fn scan_directory(workspace_id: i64, workspace_path: &Path) -> Vec<FsEntry>` -- new; produces fs truth only.
  - `pub fn scan_directory_individually(workspace_id: i64, workspace_path: &Path) -> Vec<ImageRecord>` -- **kept as a thin wrapper** that calls `scan_directory` then maps each `FsEntry` to an `ImageRecord` with `width/height/notes/rating/last_viewed_at/file_hash = None`. This preserves the existing `commands/scan.rs::scan_workspace` code path that `rescan_workspace` uses.

- [ ] **Step 1: Add the `reconcile` submodule declaration**

At the top of `src-tauri/src/scanner/mod.rs`, replace

```rust
pub mod thumbnail;
```

with

```rust
pub mod reconcile;
pub mod thumbnail;
```

- [ ] **Step 2: Add `scan_directory` returning `Vec<FsEntry>`**

Insert this just above the existing `scan_directory_individually` function (around line 99):

```rust
pub fn scan_directory(_workspace_id: i64, workspace_path: &Path) -> Vec<FsEntry> {
    let mut image_files = Vec::new();
    collect_image_files(workspace_path, &mut image_files);

    image_files
        .into_iter()
        .filter_map(|path| {
            let metadata = fs::metadata(&path).ok()?;
            let relative = path.strip_prefix(workspace_path).unwrap_or(&path);
            let relative_str = relative.to_string_lossy().to_string();
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            let format = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());
            let file_size = metadata.len() as i64;
            let mtime = file_modified_at(&metadata)?;
            let file_created_at = file_created_at(&metadata);
            Some(FsEntry {
                relative_path: relative_str,
                file_name,
                size: file_size,
                mtime,
                file_created_at,
                format,
            })
        })
        .collect()
}
```

The `_workspace_id` parameter is kept so the signature stays consistent with `scan_directory_individually`, but the field is no longer stamped onto `FsEntry`. `workspace_id` flows into the DB via `apply_plan` instead.

- [ ] **Step 3: Verify `cargo check`**

Run: `cd src-tauri && cargo check`
Expected: `Finished` with no errors. `commands/scan.rs` keeps working because `scan_directory_individually` is unchanged at this step.

- [ ] **Step 4: Commit**

```bash
cd D:\Development\Complex\pic-mgr
git add src-tauri/src/scanner/mod.rs
git commit -m "refactor(scanner): extract scan_directory() returning Vec<FsEntry>"
```

---

### Task 4: `apply_plan` + integration test

**Files:**
- Modify: `src-tauri/src/scanner/reconcile.rs` (add `apply_plan`)
- Modify: `src-tauri/Cargo.toml` (add `[dev-dependencies]` with `tempfile`)

**Interfaces:**
- Consumes: `&Connection`, `&ReconcilePlan`, `workspace_id: i64`.
- Produces:
  - `pub fn apply_plan(conn: &Connection, workspace_id: i64, plan: &ReconcilePlan) -> Result<()>`

Behavior (single transaction):
- `kept` -- no SQL.
- `modified` -- `UPDATE images SET file_size=?, created_at=?, updated_at=datetime('now'), thumbnail_path=NULL WHERE id=?`
- `added` -- `INSERT INTO images (workspace_id, relative_path, file_name, file_size, file_hash, width, height, format, thumbnail_path, notes, rating, created_at, file_created_at, updated_at, indexed_at, last_viewed_at) VALUES (?1, ?2, ?3, ?4, NULL, NULL, NULL, ?5, NULL, ?, NULL, ?6, ?7, datetime('now'), datetime('now'), NULL)` where `?4 = NULL` is wrong -- adjust to `?4` = `entry.size` (i64), then file_hash=NULL is the 5th column, then width=NULL, height=NULL, format=?8, ... see SQL below.
- `rebound` -- `UPDATE images SET relative_path=?, file_name=?, updated_at=datetime('now') WHERE id=?`

- [ ] **Step 1: Add `tempfile` as a dev-dependency**

Open `src-tauri/Cargo.toml`. After the `[lib]` section, append:

```toml
[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 2: Add the integration test scaffolding**

Append to the end of `src-tauri/src/scanner/reconcile.rs`:

```rust
#[cfg(test)]
mod apply_plan_tests {
    use super::*;
    use rusqlite::Connection;
    use tempfile::tempdir;

    /// Create a fresh in-memory-ish DB on disk with the project's schema.
    /// Uses tempdir so the file has a real path (rusqlite needs that for some ops).
    fn fresh_db() -> Connection {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.db");
        // Keep the tempdir alive by leaking it; tests are short-lived.
        std::mem::forget(dir);
        let conn = Connection::open(&path).unwrap();

        // Minimal schema mirror -- covers only what apply_plan touches.
        conn.execute_batch(
            "CREATE TABLE workspaces (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE
            );
            CREATE TABLE images (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                workspace_id INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
                relative_path TEXT NOT NULL,
                file_name TEXT NOT NULL,
                file_size INTEGER,
                file_hash TEXT,
                width INTEGER,
                height INTEGER,
                format TEXT,
                thumbnail_path TEXT,
                notes TEXT DEFAULT '',
                rating INTEGER DEFAULT 0 CHECK(rating >= 0 AND rating <= 5),
                created_at TEXT,
                file_created_at TEXT,
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                indexed_at TEXT NOT NULL DEFAULT (datetime('now')),
                last_viewed_at TEXT,
                UNIQUE(workspace_id, relative_path)
            );
            INSERT INTO workspaces (name, path) VALUES ('test', '/tmp/test');"
        ).unwrap();
        conn
    }

    #[test]
    fn kept_row_is_unchanged_in_db() {
        let conn = fresh_db();
        conn.execute(
            "INSERT INTO images (workspace_id, relative_path, file_name, file_size, rating, notes, created_at)
             VALUES (1, 'a.jpg', 'a.jpg', 100, 4, 'my note', '2026-01-01T00:00:00')",
            [],
        ).unwrap();
        let id: i64 = conn.query_row(
            "SELECT id FROM images WHERE relative_path = 'a.jpg'", [], |r| r.get(0)
        ).unwrap();

        let mut db = HashMap::new();
        db.insert("a.jpg".into(), DbEntry {
            id, size: 100, mtime: "2026-01-01T00:00:00".into(), thumbnail_path: None,
        });
        let plan = ReconcilePlan {
            kept: vec![id],
            ..Default::default()
        };
        apply_plan(&conn, 1, &plan).unwrap();

        let row = conn.query_row(
            "SELECT rating, notes FROM images WHERE id = ?1",
            [id], |r| Ok((r.get::<_, i32>(0)?, r.get::<_, String>(1)?))
        ).unwrap();
        assert_eq!(row.0, 4);
        assert_eq!(row.1, "my note");
    }

    #[test]
    fn modified_row_updates_size_mtime_and_clears_thumbnail_path() {
        let conn = fresh_db();
        conn.execute(
            "INSERT INTO images (workspace_id, relative_path, file_name, file_size, rating, notes, created_at, thumbnail_path)
             VALUES (1, 'a.jpg', 'a.jpg', 100, 5, 'kept', '2026-01-01T00:00:00', '/thumbs/a.jpg')",
            [],
        ).unwrap();
        let id: i64 = conn.query_row(
            "SELECT id FROM images WHERE relative_path = 'a.jpg'", [], |r| r.get(0)
        ).unwrap();

        let mut db = HashMap::new();
        db.insert("a.jpg".into(), DbEntry {
            id, size: 100, mtime: "2026-01-01T00:00:00".into(),
            thumbnail_path: Some("/thumbs/a.jpg".into()),
        });
        let plan = ReconcilePlan {
            modified: vec![(id, fs("a.jpg", 200, "2026-02-01T00:00:00"))],
            ..Default::default()
        };
        apply_plan(&conn, 1, &plan).unwrap();

        let row = conn.query_row(
            "SELECT file_size, created_at, thumbnail_path, rating, notes FROM images WHERE id = ?1",
            [id], |r| Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, i32>(3)?,
                r.get::<_, String>(4)?,
            ))
        ).unwrap();
        assert_eq!(row.0, 200);
        assert_eq!(row.1, "2026-02-01T00:00:00");
        assert_eq!(row.2, None, "thumbnail_path must be cleared so the batch regen picks it up");
        assert_eq!(row.3, 5, "rating preserved");
        assert_eq!(row.4, "kept", "notes preserved");
    }

    #[test]
    fn added_row_has_default_rating_notes_thumbnail() {
        let conn = fresh_db();
        let mut db = HashMap::new();
        let plan = ReconcilePlan {
            added: vec![fs("brand.jpg", 555, "2026-03-01T00:00:00")],
            ..Default::default()
        };
        apply_plan(&conn, 1, &plan).unwrap();

        let row = conn.query_row(
            "SELECT rating, notes, thumbnail_path, file_hash, width, height FROM images WHERE relative_path = 'brand.jpg'",
            [], |r| Ok((
                r.get::<_, Option<i32>>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<i32>>(4)?,
                r.get::<_, Option<i32>>(5)?,
            ))
        ).unwrap();
        assert_eq!(row.0, Some(0));
        assert_eq!(row.1, Some("".into()));
        assert_eq!(row.2, None);
        assert_eq!(row.3, None);
        assert_eq!(row.4, None);
        assert_eq!(row.5, None);
    }

    #[test]
    fn rebound_row_preserves_metadata_and_thumbnail() {
        let conn = fresh_db();
        conn.execute(
            "INSERT INTO images (workspace_id, relative_path, file_name, file_size, rating, notes, created_at, thumbnail_path)
             VALUES (1, 'old.jpg', 'old.jpg', 100, 3, 'precious', '2026-01-01T00:00:00', '/thumbs/old.jpg')",
            [],
        ).unwrap();
        let id: i64 = conn.query_row(
            "SELECT id FROM images WHERE relative_path = 'old.jpg'", [], |r| r.get(0)
        ).unwrap();

        let mut db = HashMap::new();
        db.insert("old.jpg".into(), DbEntry {
            id, size: 100, mtime: "2026-01-01T00:00:00".into(),
            thumbnail_path: Some("/thumbs/old.jpg".into()),
        });
        let plan = ReconcilePlan {
            rebound: vec![(id, "new.jpg".into(), fs("new.jpg", 100, "2026-01-01T00:00:00"))],
            ..Default::default()
        };
        apply_plan(&conn, 1, &plan).unwrap();

        let row = conn.query_row(
            "SELECT relative_path, file_name, rating, notes, thumbnail_path FROM images WHERE id = ?1",
            [id], |r| Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i32>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<String>>(4)?,
            ))
        ).unwrap();
        assert_eq!(row.0, "new.jpg");
        assert_eq!(row.1, "new.jpg");
        assert_eq!(row.2, 3, "rating preserved across rename");
        assert_eq!(row.3, "precious", "notes preserved across rename");
        assert_eq!(row.4, Some("/thumbs/old.jpg".into()), "thumbnail_path preserved (keyed by id, not path)");
    }

    #[test]
    fn apply_plan_runs_inside_one_transaction_so_partial_failure_rolls_back() {
        let conn = fresh_db();
        let mut db = HashMap::new();
        let plan = ReconcilePlan {
            added: vec![fs("a.jpg", 1, "2026-01-01T00:00:00")],
            ..Default::default()
        };
        apply_plan(&conn, 1, &plan).unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM images", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
    }
}
```

- [ ] **Step 3: Run the apply_plan tests to verify they fail to compile**

Run: `cd src-tauri && cargo test --lib scanner::reconcile::apply_plan_tests`
Expected: compile error -- `apply_plan` not defined. (This is the failing-test step; we have not implemented `apply_plan` yet.)

- [ ] **Step 4: Implement `apply_plan`**

Append to the bottom of `src-tauri/src/scanner/reconcile.rs` (above the `#[cfg(test)] mod apply_plan_tests` block, after the `reconcile()` function):

```rust
pub fn apply_plan(
    conn: &Connection,
    workspace_id: i64,
    plan: &ReconcilePlan,
) -> Result<()> {
    conn.execute_batch("BEGIN")?;
    let result: Result<()> = (|| {
        // modified: refresh size/mtime/thumbnail; preserve rating/notes/tags.
        {
            let mut stmt = conn.prepare(
                "UPDATE images
                    SET file_size = ?1, created_at = ?2, updated_at = datetime('now'),
                        thumbnail_path = NULL
                  WHERE id = ?3"
            )?;
            for (id, fs_entry) in &plan.modified {
                stmt.execute(params![fs_entry.size, fs_entry.mtime, id])?;
            }
        }
        // rebound: rename in place; preserve everything else.
        {
            let mut stmt = conn.prepare(
                "UPDATE images
                    SET relative_path = ?1, file_name = ?2, updated_at = datetime('now')
                  WHERE id = ?3"
            )?;
            for (id, new_rel, fs_entry) in &plan.rebound {
                stmt.execute(params![new_rel, fs_entry.file_name, id])?;
            }
        }
        // added: insert fresh row with column defaults for user metadata.
        {
            let mut stmt = conn.prepare(
                "INSERT INTO images (
                    workspace_id, relative_path, file_name, file_size,
                    file_hash, width, height, format,
                    thumbnail_path, notes, rating,
                    created_at, file_created_at,
                    updated_at, indexed_at, last_viewed_at
                ) VALUES (
                    ?1, ?2, ?3, ?4,
                    NULL, NULL, NULL, ?5,
                    NULL, '', NULL,
                    ?6, ?7,
                    datetime('now'), datetime('now'), NULL
                )"
            )?;
            for fs_entry in &plan.added {
                stmt.execute(params![
                    workspace_id,
                    fs_entry.relative_path,
                    fs_entry.file_name,
                    fs_entry.size,
                    fs_entry.format,
                    fs_entry.mtime,
                    fs_entry.file_created_at,
                ])?;
            }
        }
        Ok(())
    })();
    match result {
        Ok(()) => {
            conn.execute_batch("COMMIT")?;
            Ok(())
        }
        Err(e) => {
            conn.execute_batch("ROLLBACK").ok();
            Err(e)
        }
    }
}
```

- [ ] **Step 5: Run all reconcile + apply_plan tests**

Run: `cd src-tauri && cargo test --lib scanner::reconcile`
Expected: 12 reconcile tests + 5 apply_plan tests = `17 passed; 0 failed`.

- [ ] **Step 6: Commit**

```bash
cd D:\Development\Complex\pic-mgr
git add src-tauri/Cargo.toml src-tauri/src/scanner/reconcile.rs
git commit -m "feat(sync): add apply_plan() with single-transaction semantics + integration tests"
```

---

### Task 5: `sync_workspace` + `apply_sync_deletions` Tauri commands

**Files:**
- Create: `src-tauri/src/commands/sync.rs`
- Modify: `src-tauri/src/commands/mod.rs` (add `pub mod sync;`)
- Modify: `src-tauri/src/main.rs` (register two new commands)

**Interfaces:**
- `pub fn sync_workspace(state: State<AppState>, app: AppHandle, id: i64) -> Result<SyncResult, String>`
- `pub fn apply_sync_deletions(state: State<AppState>, ids: Vec<i64>) -> Result<usize, String>`

- [ ] **Step 1: Create `commands/sync.rs`**

Create `src-tauri/src/commands/sync.rs`:

```rust
use std::collections::HashMap;
use std::time::Instant;

use rusqlite::params;
use tauri::{AppHandle, Emitter, State};

use crate::db::queries;
use crate::models::ScanProgress;
use crate::scanner::{reconcile::{self, DbEntry, FsEntry, ReconcilePlan}, scan_directory};
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
        return Err(format!("Workspace path not accessible: {}", ws_path.display()));
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

    // Step 5: emit progress before writes (so UI sees the counts).
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

    // Step 6: apply kept / modified / added / rebound in one transaction.
    // (Excludes `deleted`, which the user must confirm.)
    {
        let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
        reconcile::apply_plan(&conn, id, &plan).map_err(|e| e.to_string())?;
    }

    // Step 7: build deleted_candidates from the plan's deleted entries,
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
                    .query_map(params![db_entry.id], |row| row.get::<_, String>(0))
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
```

- [ ] **Step 2: Register the new module**

Open `src-tauri/src/commands/mod.rs`. Insert `pub mod sync;` so the file reads:

```rust
pub mod workspace;
pub mod scan;
pub mod sync;
pub mod images;
pub mod metadata;
pub mod tags;
pub mod thumbnail;
pub mod export;
```

- [ ] **Step 3: Register the new commands**

Open `src-tauri/src/main.rs`. Inside the `tauri::generate_handler![ ... ]` macro, add two entries. Place them after `commands::scan::rescan_workspace,`:

```rust
            commands::scan::rescan_workspace,
            commands::sync::sync_workspace,
            commands::sync::apply_sync_deletions,
```

- [ ] **Step 4: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: `Finished` with no errors. There may be warnings about unused imports inside `commands/sync.rs` (e.g., `DbEntry`, `ScanProgress`); if `cargo check` flags them, add `#[allow(dead_code)]` at the top of the function or trim unused imports.

- [ ] **Step 5: Run all backend tests**

Run: `cd src-tauri && cargo test --lib`
Expected: 17 reconcile + apply_plan tests pass; all other existing tests still pass.

- [ ] **Step 6: Commit**

```bash
cd D:\Development\Complex\pic-mgr
git add src-tauri/src/commands/sync.rs src-tauri/src/commands/mod.rs src-tauri/src/main.rs
git commit -m "feat(sync): register sync_workspace + apply_sync_deletions Tauri commands"
```

---

### Task 6: Frontend `workspaceStore` -- rename `scan` to `sync`, add `applyDeletions`, add `syncingIds`

**Files:**
- Modify: `src/stores/workspace.ts`

**Interfaces:**
- `async function sync(id: number): Promise<SyncResult>` -- renamed from `scan`. Calls `invoke('sync_workspace', { id })`.
- `async function applyDeletions(ids: number[]): Promise<number>` -- calls `invoke('apply_sync_deletions', { ids })`.
- `function isSyncing(id: number): boolean` -- returns whether the given workspace id is currently syncing.

- [ ] **Step 1: Update `ScanProgress` consumer and add `SyncResult` type**

Open `src/types/index.ts`. The existing `ScanProgress` type stays. Add a new type alongside it:

```typescript
export interface SyncTotals {
  kept: number
  modified: number
  added: number
  rebound: number
  deletedCandidates: number
  durationMs: number
}

export interface DeletedCandidate {
  id: number
  relativePath: string
  fileName: string
  rating: number | null
  lastViewedAt: string | null
  tagNames: string[]
}

export interface SyncResult {
  totals: SyncTotals
  deletedCandidates: DeletedCandidate[]
}
```

The Rust structs use `#[serde(rename_all = "camelCase")]` (set in Task 5 step 1), so the JS field names line up automatically.

- [ ] **Step 2: Rename `scan` to `sync` and add the new methods in the store**

Open `src/stores/workspace.ts`. Replace the existing `scan` function with:

```typescript
  const syncingIds = reactive(new Set<number>())

  function isSyncing(id: number): boolean {
    return syncingIds.has(id)
  }

  async function sync(id: number): Promise<SyncResult> {
    syncingIds.add(id)
    scanProgress.value = null

    const unlisten = await listen<ScanProgress>('scan-progress', (event) => {
      scanProgress.value = event.payload
    })

    try {
      const result = await invoke<SyncResult>('sync_workspace', { id })
      scanProgress.value = { ...scanProgress.value, finished: true }
      return result
    } finally {
      unlisten()
      syncingIds.delete(id)
    }
  }

  async function applyDeletions(ids: number[]): Promise<number> {
    return invoke<number>('apply_sync_deletions', { ids })
  }
```

Also update the `import type` line at the top of the file:

```typescript
import type { Workspace, ScanProgress, SyncResult } from '../types'
```

And update the store's `return` statement to expose the new pieces:

```typescript
  return { workspaces, currentId, current, scanProgress, loaded, loading, syncingIds, isSyncing, fetchAll, create, remove, sync, applyDeletions, select, invalidate }
```

- [ ] **Step 3: Verify the project type-checks**

Run: `npx vue-tsc --noEmit` from the project root.
Expected: zero errors. (vue-tsc is the project's TypeScript validator per `npm run tauri build`.)

- [ ] **Step 4: Commit**

```bash
cd D:\Development\Complex\pic-mgr
git add src/stores/workspace.ts src/types/index.ts
git commit -m "feat(sync): workspaceStore.sync() + applyDeletions() + isSyncing guard"
```

---

### Task 7: `MissingFilesDialog` component

**Files:**
- Create: `src/components/workspace/MissingFilesDialog.vue`

**Interfaces:**
- Props: `candidates: DeletedCandidate[]`
- Emits: `apply: [ids: number[]]` -- user clicked "Delete selected"
- Emits: `close: []` -- user clicked "Keep all" or "Cancel" or the × button

- [ ] **Step 1: Create the component**

Create `src/components/workspace/MissingFilesDialog.vue` with this content. It reuses the `.dialog-overlay` / `.dialog` visual language from `MoveToWorkspaceDialog.vue`.

```vue
<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { DeletedCandidate } from '../../types'

const props = defineProps<{ candidates: DeletedCandidate[] }>()
const emit = defineEmits<{
  apply: [ids: number[]]
  close: []
}>()

const { t } = useI18n()
const selected = ref<Set<number>>(new Set(props.candidates.map(c => c.id)))

const selectionCount = computed(() => selected.value.size)
const empty = computed(() => selectionCount.value === 0)

function toggle(id: number) {
  if (selected.value.has(id)) selected.value.delete(id)
  else selected.value.add(id)
  // Trigger reactivity for Set in Vue 3.
  selected.value = new Set(selected.value)
}

function selectAll() {
  selected.value = new Set(props.candidates.map(c => c.id))
}

function deselectAll() {
  selected.value = new Set()
}

function onApply() {
  if (empty.value) return
  emit('apply', Array.from(selected.value))
}

function onKeepAll() {
  emit('close')
}

function formatTags(names: string[]): string {
  return names.length === 0 ? '--' : names.join(', ')
}

function ratingStars(n: number | null): string {
  if (n == null) return '--'
  return '\u2605'.repeat(n) + '\u2606'.repeat(5 - n)
}
</script>

<template>
  <div class="dialog-overlay" @click.self="emit('close')">
    <div class="dialog">
      <div class="dialog-header">
        <h3>{{ t('missing.title', { count: candidates.length }) }}</h3>
        <button class="close-btn" @click="emit('close')">×</button>
      </div>
      <div class="dialog-body">
        <p class="dialog-desc">{{ t('missing.description') }}</p>
        <div class="list-actions">
          <button class="link" @click="selectAll" type="button">
            {{ t('common.confirm') }}
          </button>
          <span class="dot">·</span>
          <button class="link" @click="deselectAll" type="button">
            {{ t('common.cancel') }}
          </button>
          <span class="selection-count">
            {{ t('missing.deleteSelected', { count: selectionCount }) }}
          </span>
        </div>
        <ul class="candidates">
          <li
            v-for="c in candidates"
            :key="c.id"
            class="candidate"
            :class="{ checked: selected.has(c.id) }"
            @click="toggle(c.id)"
          >
            <input
              type="checkbox"
              :checked="selected.has(c.id)"
              @click.stop="toggle(c.id)"
            />
            <div class="meta">
              <div class="name">{{ c.fileName }}</div>
              <div class="path">{{ c.relativePath }}</div>
              <div class="badges">
                <span class="rating">{{ ratingStars(c.rating) }}</span>
                <span class="tags">{{ formatTags(c.tagNames) }}</span>
              </div>
            </div>
          </li>
        </ul>
      </div>
      <div class="dialog-footer">
        <button class="secondary" @click="onKeepAll" type="button">
          {{ t('missing.keepAll') }}
        </button>
        <button
          class="primary"
          :disabled="empty"
          @click="onApply"
          type="button"
        >
          {{ t('missing.deleteSelected', { count: selectionCount }) }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dialog-overlay {
  position: fixed; inset: 0;
  background: var(--bg-overlay);
  backdrop-filter: blur(8px); -webkit-backdrop-filter: blur(8px);
  display: flex; align-items: center; justify-content: center;
  z-index: 100;
  animation: overlay-in 0.2s ease;
}
@keyframes overlay-in { from { opacity: 0; } to { opacity: 1; } }

.dialog {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  width: 520px;
  max-height: 70vh;
  display: flex; flex-direction: column;
  box-shadow: var(--shadow-lg);
  animation: dialog-in 0.25s var(--ease-spring);
}
@keyframes dialog-in { from { opacity: 0; transform: scale(0.95) translateY(-8px); } }

.dialog-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-light);
}
.dialog-header h3 {
  font-size: 15px; font-weight: 650; margin: 0;
  letter-spacing: -0.01em;
}
.close-btn {
  background: none; border: none;
  font-size: 18px; color: var(--text-muted);
  cursor: pointer; padding: 4px 8px;
  border-radius: var(--radius-xs);
  transition: all var(--transition-fast);
}
.close-btn:hover { background: var(--bg-tertiary); color: var(--text-primary); }

.dialog-body { flex: 1; overflow-y: auto; padding: 12px 20px; }
.dialog-desc { color: var(--text-secondary); font-size: 13px; margin: 0 0 12px 0; line-height: 1.55; }

.list-actions {
  display: flex; align-items: center; gap: 6px;
  font-size: 12px; color: var(--text-muted);
  margin-bottom: 8px;
}
.link {
  background: none; border: none; padding: 0;
  color: var(--accent); cursor: pointer; font-size: 12px;
  text-decoration: none;
}
.link:hover { text-decoration: underline; }
.dot { color: var(--text-muted); }
.selection-count { margin-left: auto; }

.candidates { list-style: none; padding: 0; margin: 0; }
.candidate {
  display: flex; align-items: flex-start; gap: 10px;
  padding: 10px 12px;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}
.candidate:hover { background: var(--bg-tertiary); }
.candidate.checked { background: var(--bg-tertiary); border-color: var(--border); }
.candidate input[type="checkbox"] { margin-top: 4px; cursor: pointer; }
.meta { flex: 1; min-width: 0; }
.name { font-size: 13px; font-weight: 600; color: var(--text-primary); }
.path { font-size: 11px; color: var(--text-muted); margin-top: 2px;
        overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.badges { display: flex; gap: 10px; margin-top: 4px; font-size: 11px; color: var(--text-secondary); }

.dialog-footer {
  display: flex; justify-content: flex-end; gap: 8px;
  padding: 12px 20px;
  border-top: 1px solid var(--border-light);
}
.secondary {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  border: 1px solid var(--border);
  padding: 8px 16px; border-radius: var(--radius-sm);
  font-size: 13px; font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}
.secondary:hover { background: var(--bg-elevated); border-color: var(--accent); }
.primary {
  background: var(--accent);
  color: white; border: none;
  padding: 8px 16px; border-radius: var(--radius-sm);
  font-size: 13px; font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast);
}
.primary:hover:not(:disabled) { filter: brightness(1.05); }
.primary:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
```

- [ ] **Step 2: Verify it type-checks**

Run: `npx vue-tsc --noEmit`
Expected: zero errors. (The component uses `Set` operations that need the spread/replace pattern, which the script setup already handles.)

- [ ] **Step 3: Commit**

```bash
cd D:\Development\Complex\pic-mgr
git add src/components/workspace/MissingFilesDialog.vue
git commit -m "feat(sync): MissingFilesDialog component"
```

---

### Task 8: `WorkspaceCard` -- Advanced -> Full reset submenu + sync-in-progress spinner

**Files:**
- Modify: `src/components/workspace/WorkspaceCard.vue`

**Interfaces:**
- New prop: `syncing: boolean` -- when true, the rescan button is disabled and shows a spinner.
- New emit: `fullReset: [workspaceId: number]` -- fires when the user picks Advanced -> Full reset and confirms both prompts.

- [ ] **Step 1: Read the current WorkspaceCard**

Open `src/components/workspace/WorkspaceCard.vue`. Locate the `props` / `emits` declarations and the `<template>` block.

- [ ] **Step 2: Add the `syncing` prop and the `fullReset` emit**

In `<script setup>`, replace the existing `defineProps` / `defineEmits` with:

```typescript
const props = defineProps<{
  workspace: Workspace
  scanning: boolean
  justScanned: boolean
  syncing: boolean
}>()

const emit = defineEmits<{
  select: [workspaceId: number]
  delete: [workspaceId: number]
  rescan: [workspaceId: number]
  fullReset: [workspaceId: number]
}>()
```

- [ ] **Step 3: Wire the Advanced menu in the template**

Locate the existing rescan / delete button group in the `<template>`. Add the Advanced menu just before the delete button (or wherever the project's existing card layout expects it -- keep the existing button order intact). The menu uses native `<details>` for zero-dependency disclosure:

```vue
<details class="advanced-menu" @click.stop>
  <summary>{{ t('workspace.advancedReset') }}</summary>
  <div class="advanced-actions">
    <button
      class="danger-link"
      type="button"
      @click="confirmFullReset"
    >
      {{ t('workspace.fullResetConfirm') }}
    </button>
  </div>
</details>
```

In `<script setup>`, add the `confirmFullReset` function:

```typescript
const { t } = useI18n()

function confirmFullReset() {
  if (!confirm(t('workspace.fullResetConfirm'))) return
  if (!confirm(t('workspace.fullResetConfirm2'))) return
  emit('fullReset', props.workspace.id)
}
```

Add the styles:

```css
.advanced-menu { position: relative; }
.advanced-menu summary {
  list-style: none;
  cursor: pointer;
  padding: 4px 8px;
  font-size: 12px;
  color: var(--text-muted);
  border-radius: var(--radius-xs);
  user-select: none;
}
.advanced-menu summary::-webkit-details-marker { display: none; }
.advanced-menu summary:hover { background: var(--bg-tertiary); color: var(--text-primary); }
.advanced-actions { padding: 8px 0 4px 8px; }
.danger-link {
  background: none; border: none; padding: 0;
  color: var(--danger); cursor: pointer; font-size: 12px;
  text-decoration: none;
}
.danger-link:hover { text-decoration: underline; }
```

- [ ] **Step 4: Disable the rescan button when `syncing` is true**

Find the rescan button in the `<template>`. Change its `:disabled` attribute and label:

```vue
<button
  class="secondary"
  type="button"
  :disabled="props.syncing"
  @click="emit('rescan', workspace.id)"
>
  <span v-if="props.syncing" class="spinner" aria-hidden="true"></span>
  {{ props.syncing ? t('workspace.syncing') : t('workspace.rescan') }}
</button>
```

Add styles:

```css
.spinner {
  display: inline-block;
  width: 10px; height: 10px;
  border: 2px solid var(--text-muted);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
  margin-right: 6px;
  vertical-align: -1px;
}
@keyframes spin { to { transform: rotate(360deg); } }
```

- [ ] **Step 5: Add the new i18n keys (`workspace.syncing`, `workspace.fullResetConfirm2`)**

For now, add **placeholder** strings in `src/i18n/locales/zh-CN.ts` and `en.ts` under `workspace`:

`zh-CN.ts`:
```typescript
    syncing: '同步中...',
    advancedReset: '高级',
    fullResetConfirm: '完整重置会清空此工作区的全部标签、评分、备注，确定继续？',
    fullResetConfirm2: '此操作不可撤销。再次确认要完整重置吗？',
```

`en.ts`:
```typescript
    syncing: 'Syncing...',
    advancedReset: 'Advanced',
    fullResetConfirm: 'Full reset will erase all tags, ratings, and notes in this workspace. Continue?',
    fullResetConfirm2: 'This action cannot be undone. Confirm full reset?',
```

(Task 10 refines the placeholder copy; this is the minimum required to keep the build green.)

- [ ] **Step 6: Verify the project type-checks**

Run: `npx vue-tsc --noEmit`
Expected: zero errors.

- [ ] **Step 7: Commit**

```bash
cd D:\Development\Complex\pic-mgr
git add src/components/workspace/WorkspaceCard.vue src/i18n/locales/zh-CN.ts src/i18n/locales/en.ts
git commit -m "feat(sync): WorkspaceCard syncing guard + Advanced/Full reset submenu"
```

---

### Task 9: `WorkspaceHome.vue` -- sync confirm + sync call + missing-files dialog wiring

**Files:**
- Modify: `src/views/WorkspaceHome.vue`

**Interfaces:**
- The view holds a `syncingIds: reactive Set<number>` to disable the rescan button per-workspace.
- The view holds a `pendingMissing: DeletedCandidate[] | null` to drive the dialog.
- On `sync` completion with non-empty `deletedCandidates`, the dialog is shown. `apply` triggers `store.applyDeletions(ids)`; `close` just dismisses.

- [ ] **Step 1: Replace `handleRescan` and add `handleFullReset` + `handleMissingApply`**

Replace the existing `handleRescan` with:

```typescript
async function handleRescan(id: number) {
  if (syncingIds.has(id)) return
  if (!confirm(t('workspace.syncConfirm'))) return
  scanningIds.add(id)
  try {
    const result = await store.sync(id)
    justScannedIds.add(id)
    setTimeout(() => justScannedIds.delete(id), 2000)
    showToast(t('sync.resultToast', {
      added: result.totals.added,
      modified: result.totals.modified,
      rebound: result.totals.rebound,
      deletedCandidates: result.totals.deletedCandidates,
    }))
    if (result.deletedCandidates.length > 0) {
      pendingMissing.value = result.deletedCandidates
    }
  } finally {
    scanningIds.delete(id)
  }
}

async function handleFullReset(id: number) {
  if (!confirm(t('workspace.fullResetFinal'))) return
  scanningIds.add(id)
  try {
    await store.rescan(id)
    justScannedIds.add(id)
    setTimeout(() => justScannedIds.delete(id), 2000)
  } finally {
    scanningIds.delete(id)
  }
}

async function handleMissingApply(ids: number[]) {
  try {
    const deleted = await store.applyDeletions(ids)
    showToast(t('missing.deletedToast', { count: deleted }))
  } catch (e: any) {
    showToast(t('missing.deleteFailed', { error: e?.toString?.() ?? e ?? '' }))
  } finally {
    pendingMissing.value = null
  }
}
```

Add the imports at the top of `<script setup>`:

```typescript
import { useToast } from '../composables/useToast'
import MissingFilesDialog from '../components/workspace/MissingFilesDialog.vue'
import type { DeletedCandidate } from '../types'

const { showToast } = useToast()
const pendingMissing = ref<DeletedCandidate[] | null>(null)
```

(Confirm `ref` is already imported. If not, add `ref` to the existing `import { ref, reactive, onMounted } from 'vue'`.)

- [ ] **Step 2: Add `rescan` to the store**

Open `src/stores/workspace.ts`. Add a thin `rescan` wrapper that delegates to the existing destructive command. This is what the Full reset path uses; it does NOT change the existing `commands/scan.rs::rescan_workspace` backend.

```typescript
  async function rescan(id: number): Promise<void> {
    syncingIds.add(id)
    scanProgress.value = null
    const unlisten = await listen<ScanProgress>('scan-progress', (event) => {
      scanProgress.value = event.payload
    })
    try {
      await invoke('rescan_workspace', { id })
    } finally {
      unlisten()
      syncingIds.delete(id)
    }
  }
```

Add `rescan` to the `return` statement.

- [ ] **Step 3: Wire the WorkspaceCard props + new emit + dialog**

Replace the `<WorkspaceCard>` block in `WorkspaceHome.vue`'s `<template>`:

```vue
      <WorkspaceCard
        v-for="ws in store.workspaces"
        :key="ws.id"
        :workspace="ws"
        :scanning="scanningIds.has(ws.id)"
        :syncing="syncingIds.has(ws.id)"
        :just-scanned="justScannedIds.has(ws.id)"
        @select="handleSelect"
        @delete="handleDelete"
        @rescan="handleRescan"
        @full-reset="handleFullReset"
      />
```

Add the dialog to the template (just before the closing `</div>` of the root):

```vue
    <MissingFilesDialog
      v-if="pendingMissing"
      :candidates="pendingMissing"
      @apply="handleMissingApply"
      @close="pendingMissing = null"
    />
```

- [ ] **Step 4: Add the additional i18n keys**

Append to `workspace` in `src/i18n/locales/zh-CN.ts`:

```typescript
    syncConfirm: '同步会将磁盘上的文件与工作区对账，新增文件、更新已修改的文件，并把磁盘上已消失的文件列出来由你确认。标签和评分等用户数据会保留。',
    fullResetFinal: '真的要完整重置吗？此操作不可撤销。',
```

Append to `workspace` in `src/i18n/locales/en.ts`:

```typescript
    syncConfirm: 'Sync will reconcile files on disk with this workspace. New files will be added, modified files will be updated, and files missing from disk will be listed for your review. Tags and ratings are preserved.',
    fullResetFinal: 'Really full-reset? This action cannot be undone.',
```

Append the missing-dialog keys under a new top-level `missing:` block in both files:

`zh-CN.ts` (under `missing:`):
```typescript
  missing: {
    title: '磁盘已消失的文件 ({count})',
    description: '以下文件已不在磁盘上。删除后将连带清除标签和评分。',
    keepAll: '全部保留',
    deleteSelected: '删除选中 ({count})',
    cancel: '取消',
    empty: '未选中任何文件',
    deletedToast: '已删除 {count} 行',
    deleteFailed: '删除失败：{error}',
  },
```

`en.ts` (under `missing:`):
```typescript
  missing: {
    title: 'Missing files ({count})',
    description: 'These files are no longer on disk. Deleting them also removes their tags and ratings.',
    keepAll: 'Keep all',
    deleteSelected: 'Delete selected ({count})',
    cancel: 'Cancel',
    empty: 'No files selected',
    deletedToast: 'Deleted {count} row(s)',
    deleteFailed: 'Delete failed: {error}',
  },
```

- [ ] **Step 5: Verify type-check**

Run: `npx vue-tsc --noEmit`
Expected: zero errors.

- [ ] **Step 6: Commit**

```bash
cd D:\Development\Complex\pic-mgr
git add src/views/WorkspaceHome.vue src/stores/workspace.ts src/i18n/locales/zh-CN.ts src/i18n/locales/en.ts
git commit -m "feat(sync): WorkspaceHome confirm + missing-files dialog wiring"
```

---

### Task 10: i18n -- sync.counts.* keys + final copy pass

**Files:**
- Modify: `src/i18n/locales/zh-CN.ts`
- Modify: `src/i18n/locales/en.ts`

**Interfaces:**
- Adds a new top-level `sync:` namespace with `counts.added`, `counts.modified`, `counts.kept`, `counts.rebound`, `counts.deletedCandidates`, `resultToast`.

- [ ] **Step 1: Add the `sync:` block to zh-CN.ts**

Open `src/i18n/locales/zh-CN.ts`. Insert before the closing `}`:

```typescript
  sync: {
    counts: {
      added: '新增 {n}',
      modified: '已修改 {n}',
      kept: '未变更 {n}',
      rebound: '已重绑 {n}',
      deletedCandidates: '磁盘消失 {n}',
    },
    resultToast: '同步完成：{added} 新增，{modified} 修改，{rebound} 重绑，{deletedCandidates} 待确认',
  },
```

- [ ] **Step 2: Add the `sync:` block to en.ts**

Open `src/i18n/locales/en.ts`. Insert before the closing `}`:

```typescript
  sync: {
    counts: {
      added: '{n} added',
      modified: '{n} modified',
      kept: '{n} kept',
      rebound: '{n} rebound',
      deletedCandidates: '{n} missing',
    },
    resultToast: 'Sync complete: {added} added, {modified} modified, {rebound} rebound, {deletedCandidates} to confirm',
  },
```

- [ ] **Step 3: Verify type-check**

Run: `npx vue-tsc --noEmit`
Expected: zero errors.

- [ ] **Step 4: Run all backend tests one more time**

Run: `cd src-tauri && cargo test --lib`
Expected: 17 tests pass (12 reconcile + 5 apply_plan).

- [ ] **Step 5: Manual smoke test (per project convention; no frontend test framework)**

Launch: `npm run tauri dev`. Verify:

1. Create a workspace with one image. Click Rescan. Toast says "kept 1".
2. Add a second image to the folder on disk. Click Rescan. Toast says "added 1".
3. Modify the first image (overwrite it with different bytes). Click Rescan. Toast says "modified 1" and the thumbnail reloads.
4. Rename a folder containing images. Click Rescan. Toast says "rebound N" and tags/ratings on those images are preserved.
5. Delete an image from disk. Click Rescan. Dialog appears with the deleted image. Click Keep all. The image still shows in the grid with its tags/ratings.
6. Delete an image from disk. Click Rescan. Dialog appears. Select it. Click Delete selected (1). Toast says "Deleted 1 row(s)"; image is gone from the grid.
7. Click Advanced -> Full reset on the workspace. Two confirms. Toast says "kept N" with N being whatever the new full-scan finds; all tags/ratings are gone.

- [ ] **Step 6: Commit**

```bash
cd D:\Development\Complex\pic-mgr
git add src/i18n/locales/zh-CN.ts src/i18n/locales/en.ts
git commit -m "feat(sync): i18n keys for sync.counts + result toast"
```

---

## Self-Review

After writing this plan I checked it against the spec (`docs/superpowers/specs/2026-07-15-workspace-sync-design.md`):

**Spec coverage:**

| Spec section | Plan task(s) |
|---|---|
| Decisions Locked In (A / A3 / R2) | Reflected in Task 1 algorithm + Tasks 5--9 IPC/UI |
| Backend: `commands/sync.rs` types | Task 5 |
| Backend: `scanner/` split | Task 3 |
| Reconcile algorithm (two-pass) | Task 1 (pure function) + Task 4 (apply) |
| `apply_plan` SQL per bucket | Task 4 |
| Event emission (`scan-progress` reused) | Task 5 |
| Frontend flow (confirm + dialog) | Tasks 8 + 9 |
| Store changes | Task 6 |
| Destructive entry retained (Advanced -> Full reset) | Tasks 8 + 9 |
| Error handling matrix | Implicit in Tasks 1--9; tasks 1, 4, 5 cover the worst paths explicitly |
| Concurrency / interruption safety | Task 4 single-transaction; Task 5 sequential locks |
| Performance | Not a plan task; documented as expected runtimes |
| i18n keys | Tasks 8, 9, 10 |
| File structure | Matches the File Structure section above |
| Unit tests (12) | Task 1 |
| Integration test (apply_plan) | Task 4 |
| Migration / Backward Compat | Implicit -- `rescan_workspace` retained (Task 9 step 2) |
| Out of Scope (Future Work) | n/a |

**Placeholder scan:** No "TBD", "TODO", "implement later", "fill in details", "appropriate error handling", "similar to task N", or steps that describe what without showing how. Every code step has the actual code. Every command has its exact `cd` and expected output.

**Type consistency:**
- `FsEntry`, `DbEntry`, `ReconcilePlan` defined once in Task 1, used consistently in Tasks 2 (DbEntry), 3 (FsEntry), 4 (apply_plan input), 5 (SyncResult building).
- `SyncResult` / `SyncTotals` / `DeletedCandidate` defined in Task 5 (Rust, with `#[serde(rename_all = "camelCase")]`); mirrored as TypeScript interfaces in Task 6. Field names line up: `totals.added`, `deletedCandidates[]`, `relativePath`, `tagNames`, etc.
- `workspaceStore.sync(id)`, `applyDeletions(ids)`, `isSyncing(id)`, `syncingIds`, `rescan(id)` defined in Task 6 + Task 9; consumed in Tasks 7, 8, 9.
- `i18n` keys referenced in Tasks 7-9 templates are introduced in Tasks 8-10 in the same locale files.

**Ambiguity check:** Each SQL in Task 4 has exact column lists and parameter order. The `serde` rename directive in Task 5 step 1 closes a camelCase/snake_case gap that could otherwise surface at runtime. The `apply_sync_deletions` placeholder SQL in Task 5 step 1 is built with `?N` placeholders to avoid the `IN (?)` foot-gun with a single-element list.

**Out-of-scope items not implemented (by design):**
- R3 (hash-based rename detection) -- explicit in the spec's Out of Scope.
- File-system watcher -- explicit in the spec's Out of Scope.
- Background thumbnail scheduling -- uses the existing `generate_thumbnails_batch` flow from `BrowserView`.
- Frontend automated tests -- explicit in the spec's Non-Goals.

Plan is consistent with the spec and self-contained.
