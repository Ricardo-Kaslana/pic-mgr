use std::collections::HashMap;

use rusqlite::{params, Connection, Result};

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
    // Stable-match algorithm: an fs-db pair only rebinds when neither side has
    // any alternative. If two fs entries both match the same db entry (or one
    // fs entry matches two db entries), the situation is ambiguous -- we cannot
    // tell which fs is the renamed version, so both fs go to `added` and the
    // db row stays in `deleted`. The user reviews the deleted list and decides.
    let mut unmatched_db: Vec<(String, DbEntry)> = db.into_iter().collect();

    // Step 2a: for each leftover fs, find db candidates (matching size+mtime).
    let fs_candidates: Vec<Option<usize>> = leftover_fs.iter().map(|entry| {
        let mut hits = unmatched_db.iter().enumerate()
            .filter(|(_, (_, d))| d.size == entry.size && d.mtime == entry.mtime)
            .map(|(i, _)| i);
        let first = hits.next();
        let second = hits.next();
        match (first, second) {
            (Some(i), None) => Some(i),
            _ => None,  // 0 or >=2 candidates -> ambiguous, skip rebind
        }
    }).collect();

    // Step 2b: count how many fs entries claim each db entry.
    let mut db_claim_count: HashMap<usize, usize> = HashMap::new();
    for opt in &fs_candidates {
        if let Some(j) = opt {
            *db_claim_count.entry(*j).or_insert(0) += 1;
        }
    }

    // Step 2c: commit rebinds where both sides have exactly one partner.
    let mut rebound_fs_idx: HashSet<usize> = HashSet::new();
    for (i, opt) in fs_candidates.iter().enumerate() {
        if let Some(j) = opt {
            if db_claim_count[j] == 1 {
                let entry = leftover_fs[i].clone();
                let (_old_rel, db_entry) = unmatched_db.remove(*j);
                plan.rebound.push((db_entry.id, entry.relative_path.clone(), entry));
                rebound_fs_idx.insert(i);
            }
        }
    }

    // Anything not rebound is `added`.
    for (i, entry) in leftover_fs.into_iter().enumerate() {
        if !rebound_fs_idx.contains(&i) {
            plan.added.push(entry);
        }
    }

    plan.deleted = unmatched_db;
    plan
}

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
        // added: insert fresh row; rating and notes are omitted from the column
        // list so SQLite applies the schema defaults (0 and ''). Explicitly
        // passing NULL would override the defaults.
        {
            let mut stmt = conn.prepare(
                "INSERT INTO images (
                    workspace_id, relative_path, file_name, file_size,
                    file_hash, width, height, format,
                    thumbnail_path,
                    created_at, file_created_at,
                    updated_at, indexed_at, last_viewed_at
                ) VALUES (
                    ?1, ?2, ?3, ?4,
                    NULL, NULL, NULL, ?5,
                    NULL,
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

use std::collections::HashSet;

#[cfg(test)]
mod helpers {
    use super::*;

    pub fn fs(rel: &str, size: i64, mtime: &str) -> FsEntry {
        FsEntry {
            relative_path: rel.into(),
            file_name: rel.rsplit('/').next().unwrap_or(rel).into(),
            size,
            mtime: mtime.into(),
            file_created_at: None,
            format: None,
        }
    }

    pub fn entry(id: i64, size: i64, mtime: &str) -> DbEntry {
        DbEntry { id, size, mtime: mtime.into(), thumbnail_path: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::helpers::{fs, entry};

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
        let mut index = HashMap::new();
        index.insert("a/1.jpg".to_string(), entry(7, 100, "2026-01-01T00:00:00"));
        let plan = reconcile(1, vec![], index);
        assert_eq!(plan.deleted.len(), 1);
        assert_eq!(plan.deleted[0].0, "a/1.jpg");
        assert_eq!(plan.deleted[0].1.id, 7);
    }

    #[test]
    fn same_path_and_size_and_mtime_is_kept() {
        let mut index = HashMap::new();
        index.insert("a/1.jpg".to_string(), entry(7, 100, "2026-01-01T00:00:00"));
        let fs = vec![fs("a/1.jpg", 100, "2026-01-01T00:00:00")];
        let plan = reconcile(1, fs, index);
        assert_eq!(plan.kept, vec![7]);
        assert!(plan.modified.is_empty() && plan.added.is_empty()
                && plan.rebound.is_empty() && plan.deleted.is_empty());
    }

    #[test]
    fn same_path_size_changed_is_modified() {
        let mut index = HashMap::new();
        index.insert("a/1.jpg".to_string(), entry(7, 100, "2026-01-01T00:00:00"));
        let fs = vec![fs("a/1.jpg", 200, "2026-01-01T00:00:00")];
        let plan = reconcile(1, fs, index);
        assert_eq!(plan.modified.len(), 1);
        assert_eq!(plan.modified[0].0, 7);
        assert_eq!(plan.modified[0].1.size, 200);
    }

    #[test]
    fn same_path_mtime_changed_is_modified() {
        let mut index = HashMap::new();
        index.insert("a/1.jpg".to_string(), entry(7, 100, "2026-01-01T00:00:00"));
        let fs = vec![fs("a/1.jpg", 100, "2026-01-02T00:00:00")];
        let plan = reconcile(1, fs, index);
        assert_eq!(plan.modified.len(), 1);
        assert_eq!(plan.modified[0].0, 7);
    }

    #[test]
    fn renamed_file_with_size_mtime_match_is_rebound() {
        let mut index = HashMap::new();
        index.insert("old/1.jpg".to_string(), entry(7, 100, "2026-01-01T00:00:00"));
        let fs = vec![fs("new/1.jpg", 100, "2026-01-01T00:00:00")];
        let plan = reconcile(1, fs, index);
        assert_eq!(plan.rebound.len(), 1);
        assert_eq!(plan.rebound[0].0, 7);
        assert_eq!(plan.rebound[0].1, "new/1.jpg");
        assert_eq!(plan.rebound[0].2.relative_path, "new/1.jpg");
    }

    #[test]
    fn moved_to_subdirectory_with_size_mtime_match_is_rebound() {
        let mut index = HashMap::new();
        index.insert("1.jpg".to_string(), entry(7, 100, "2026-01-01T00:00:00"));
        let fs = vec![fs("archive/2026/1.jpg", 100, "2026-01-01T00:00:00")];
        let plan = reconcile(1, fs, index);
        assert_eq!(plan.rebound.len(), 1);
        assert_eq!(plan.rebound[0].1, "archive/2026/1.jpg");
    }

    #[test]
    fn many_to_one_size_mtime_collision_falls_back_to_added_and_deleted() {
        // Two fs entries with same (size, mtime), one db row with the same.
        // Per spec: ambiguous -> fs entries go to `added`, db row goes to `deleted`.
        let mut index = HashMap::new();
        index.insert("a/1.jpg".to_string(), entry(7, 100, "2026-01-01T00:00:00"));
        let fs = vec![
            fs("b/1.jpg", 100, "2026-01-01T00:00:00"),
            fs("b/2.jpg", 100, "2026-01-01T00:00:00"),
        ];
        let plan = reconcile(1, fs, index);
        assert_eq!(plan.added.len(), 2);
        assert_eq!(plan.deleted.len(), 1);
        assert!(plan.rebound.is_empty());
    }

    #[test]
    fn unmatched_size_mtime_is_truly_new() {
        let mut index = HashMap::new();
        index.insert("a/1.jpg".to_string(), entry(7, 100, "2026-01-01T00:00:00"));
        let fs = vec![fs("a/2.jpg", 999, "2030-01-01T00:00:00")];
        let plan = reconcile(1, fs, index);
        assert_eq!(plan.added.len(), 1);
        assert_eq!(plan.added[0].relative_path, "a/2.jpg");
        assert_eq!(plan.deleted.len(), 1);
    }

    #[test]
    fn mixed_plan_covers_all_buckets() {
        let mut index = HashMap::new();
        index.insert("kept.jpg".to_string(),     entry(7, 100, "2026-01-01T00:00:00"));
        index.insert("modified.jpg".to_string(), entry(8, 100, "2026-01-01T00:00:00"));
        index.insert("old_name.jpg".to_string(), entry(9, 200, "2026-02-01T00:00:00"));
        let fs = vec![
            fs("kept.jpg",          100, "2026-01-01T00:00:00"),
            fs("modified.jpg",      150, "2026-01-02T00:00:00"),
            fs("renamed.jpg",       200, "2026-02-01T00:00:00"),
            fs("brand_new.jpg",     300, "2026-03-01T00:00:00"),
        ];
        let plan = reconcile(1, fs, index);
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
        let mut index = HashMap::new();
        index.insert("a/1.jpg".to_string(), entry(7, 100, "2026-01-01T00:00:00"));
        let fs = vec![fs("a/1.jpg", 100, "2026-01-01T00:00:00")];

        let p1 = reconcile(1, fs.clone(), index.clone());
        let p2 = reconcile(42, fs, index);
        assert_eq!(p1.kept, p2.kept);
        assert_eq!(p1.added, p2.added);
        assert_eq!(p1.modified, p2.modified);
        assert_eq!(p1.rebound, p2.rebound);
        assert_eq!(p1.deleted, p2.deleted);
    }
}

#[cfg(test)]
mod apply_plan_tests {
    use super::*;
    use super::helpers::{fs, entry};
    use rusqlite::Connection;
    use tempfile::tempdir;

    fn fresh_db() -> Connection {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.db");
        std::mem::forget(dir);
        let conn = Connection::open(&path).unwrap();
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

        let mut index = HashMap::new();
        index.insert("a.jpg".to_string(), entry(7, 100, "2026-01-01T00:00:00"));
        let plan = ReconcilePlan { kept: vec![id], ..Default::default() };
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

        let mut index = HashMap::new();
        index.insert("a.jpg".to_string(), entry(7, 100, "2026-01-01T00:00:00"));
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

        let mut index = HashMap::new();
        index.insert("old.jpg".to_string(), entry(7, 100, "2026-01-01T00:00:00"));
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
    fn apply_plan_runs_inside_one_transaction() {
        let conn = fresh_db();
        let plan = ReconcilePlan {
            added: vec![fs("a.jpg", 1, "2026-01-01T00:00:00")],
            ..Default::default()
        };
        apply_plan(&conn, 1, &plan).unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM images", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
    }
}
