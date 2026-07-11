use rusqlite::{params, params_from_iter, Connection, Result, Row};
use crate::models::*;

// ── Row mapping helpers ──

pub(crate) fn row_to_image_record(row: &Row) -> Result<ImageRecord> {
    Ok(ImageRecord {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        relative_path: row.get(2)?,
        file_name: row.get(3)?,
        file_size: row.get(4)?,
        file_hash: row.get(5)?,
        width: row.get(6)?,
        height: row.get(7)?,
        format: row.get(8)?,
        thumbnail_path: row.get(9)?,
        notes: row.get(10)?,
        rating: row.get(11)?,
        created_at: row.get(12)?,
        file_created_at: row.get(13)?,
        updated_at: row.get(14)?,
        indexed_at: row.get(15)?,
        last_viewed_at: row.get(16)?,
    })
}

fn row_to_image_summary(row: &Row) -> Result<ImageSummary> {
    Ok(ImageSummary {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        relative_path: row.get(2)?,
        file_name: row.get(3)?,
        file_size: row.get(4)?,
        width: row.get(5)?,
        height: row.get(6)?,
        format: row.get(7)?,
        thumbnail_path: row.get(8)?,
        rating: row.get(9)?,
        updated_at: row.get(10)?,
        indexed_at: row.get(11)?,
        created_at: row.get(12)?,
    })
}

fn row_to_workspace(row: &Row) -> Result<Workspace> {
    Ok(Workspace {
        id: row.get(0)?,
        name: row.get(1)?,
        path: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

// ── WHERE + ORDER BY builder ──

/// Build the workspace-scoped WHERE + ORDER BY fragment for image listing
/// queries. Returns the SQL fragment (with leading "WHERE 1=1" stripped) and
/// the bound parameters in order, so the caller can splice it into both the
/// paginated SELECT and the COUNT(*) query.
/// `param_offset` is the number of parameters already bound in the caller's
/// SQL before the filter clause is spliced in. Every placeholder `?N` inside
/// the returned fragment is numbered starting from `param_offset + 1` so that
/// indices never collide with the caller's own `?1 … ?{param_offset}`.
fn build_filter_clause(filters: &ImageFilters, param_offset: usize) -> (String, Vec<rusqlite::types::Value>) {
    let mut clauses: Vec<String> = Vec::new();
    let mut params: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(ref tag_ids) = filters.tag_ids {
        if !tag_ids.is_empty() {
            // Subquery returns the set of image_ids that have ALL of the
            // requested tags — we count the rows per image and keep those
            // whose count equals the number of requested tags.
            let placeholders: Vec<String> = (0..tag_ids.len())
                .map(|i| format!("?{}", param_offset + i + 1))
                .collect();
            clauses.push(format!(
                "i.id IN (SELECT image_id FROM image_tags WHERE tag_id IN ({}) \
                 GROUP BY image_id HAVING COUNT(DISTINCT tag_id) = {})",
                placeholders.join(","),
                tag_ids.len()
            ));
            for tid in tag_ids {
                params.push(rusqlite::types::Value::from(*tid));
            }
        }
    }

    if let Some(ref format) = filters.format {
        params.push(rusqlite::types::Value::from(format.clone()));
        clauses.push(format!("i.format = ?{}", param_offset + params.len()));
    }
    if let Some(rating_min) = filters.rating_min {
        params.push(rusqlite::types::Value::from(rating_min));
        clauses.push(format!("i.rating >= ?{}", param_offset + params.len()));
    }
    if let Some(ref date_from) = filters.date_from {
        params.push(rusqlite::types::Value::from(date_from.clone()));
        clauses.push(format!("i.created_at >= ?{}", param_offset + params.len()));
    }
    if let Some(ref date_to) = filters.date_to {
        params.push(rusqlite::types::Value::from(date_to.clone()));
        clauses.push(format!("i.created_at <= ?{}", param_offset + params.len()));
    }

    let where_clause = if clauses.is_empty() {
        String::new()
    } else {
        format!(" AND {}", clauses.join(" AND "))
    };

    let sort_col = match filters.sort_by.as_deref() {
        Some("name") => "i.file_name",
        Some("date") => "i.created_at",
        Some("size") => "i.file_size",
        Some("rating") => "i.rating",
        Some("indexed_at") => "i.indexed_at",
        Some("updated_at") => "i.updated_at",
        Some("file_created_at") => "i.file_created_at",
        _ => "i.indexed_at",
    };
    let sort_dir = match filters.sort_dir.as_deref() {
        Some("asc") => "ASC",
        _ => "DESC",
    };
    let order_clause = format!(" ORDER BY {} {}", sort_col, sort_dir);

    (format!("{}{}", where_clause, order_clause), params)
}

// ── Workspace ──

pub fn insert_workspace(conn: &Connection, name: &str, path: &str) -> Result<Workspace> {
    conn.execute(
        "INSERT INTO workspaces (name, path) VALUES (?1, ?2)",
        params![name, path],
    )?;
    let id = conn.last_insert_rowid();
    Ok(Workspace {
        id,
        name: name.to_string(),
        path: path.to_string(),
        created_at: chrono_now(conn),
        updated_at: chrono_now(conn),
    })
}

pub fn list_workspaces(conn: &Connection) -> Result<Vec<Workspace>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, path, created_at, updated_at FROM workspaces ORDER BY updated_at DESC"
    )?;
    let rows = stmt.query_map([], row_to_workspace)?;
    rows.collect()
}

pub fn delete_workspace(conn: &Connection, id: i64) -> Result<()> {
    // Manual cascade: delete image_tags → images → workspace in a single transaction.
    // Avoids relying on ON DELETE CASCADE which can trigger complex nested operations
    // (FTS triggers + image_tags cascade) that may lead to SQLITE_CORRUPT on large datasets.
    conn.execute_batch("BEGIN")?;
    conn.execute(
        "DELETE FROM image_tags WHERE image_id IN (SELECT id FROM images WHERE workspace_id = ?1)",
        params![id],
    )?;
    conn.execute(
        "DELETE FROM images WHERE workspace_id = ?1",
        params![id],
    )?;
    conn.execute(
        "DELETE FROM workspaces WHERE id = ?1",
        params![id],
    )?;
    // Sweep orphan tags last so the cascade above can finish first.
    // Tags are global (no workspace_id column on `tags`): any tag whose only
    // references lived inside the workspace we just deleted now has zero
    // image_tags rows, so it is unreachable from the UI (get_tags INNER JOINs
    // image_tags) but still blocks create_tag() via the UNIQUE(name) constraint.
    // Removing it here keeps tag names reclaimable after workspace deletion
    // and remains safe for shared tags: a tag still referenced by *any*
    // workspace image_tags survives.
    conn.execute(
        "DELETE FROM tags WHERE id NOT IN (SELECT tag_id FROM image_tags)",
        [],
    )?;
    conn.execute_batch("COMMIT")?;
    Ok(())
}

pub fn get_workspace(conn: &Connection, id: i64) -> Result<Option<Workspace>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, path, created_at, updated_at FROM workspaces WHERE id = ?1"
    )?;
    let mut rows = stmt.query_map(params![id], row_to_workspace)?;
    Ok(rows.next().transpose()?)
}

// ── Images ──

pub fn insert_image(conn: &Connection, image: &ImageRecord) -> Result<i64> {
    conn.execute(
        "INSERT INTO images (workspace_id, relative_path, file_name, file_size, file_hash, width, height, format, thumbnail_path, notes, rating, created_at, file_created_at, updated_at, indexed_at, last_viewed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
         ON CONFLICT(workspace_id, relative_path) DO UPDATE SET
         file_size=excluded.file_size, file_hash=excluded.file_hash, width=excluded.width,
         height=excluded.height, format=excluded.format, updated_at=excluded.updated_at",
        params![
            image.workspace_id, image.relative_path, image.file_name, image.file_size,
            image.file_hash, image.width, image.height, image.format, image.thumbnail_path,
            image.notes, image.rating, image.created_at, image.file_created_at,
            image.updated_at, image.indexed_at, image.last_viewed_at,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_images(
    conn: &Connection,
    workspace_id: i64,
    filters: &ImageFilters,
    page: i32,
    page_size: i32,
) -> Result<ImagePage> {
    let (filter_clause, filter_params) = build_filter_clause(filters, 1);
    let count_sql = format!(
        "SELECT COUNT(*) FROM images i WHERE i.workspace_id = ?1{}",
        filter_clause
    );
    let mut all_params: Vec<rusqlite::types::Value> = Vec::with_capacity(filter_params.len() + 1);
    all_params.push(rusqlite::types::Value::from(workspace_id));
    all_params.extend(filter_params.iter().cloned());
    let total: i64 = conn.query_row(
        &count_sql,
        params_from_iter(all_params.iter()),
        |row| row.get(0),
    )?;

    let offset = (page - 1) * page_size;
    let sql = format!(
        "SELECT i.id, i.workspace_id, i.relative_path, i.file_name, i.file_size, i.file_hash,
                i.width, i.height, i.format, i.thumbnail_path, i.notes, i.rating,
                i.created_at, i.file_created_at, i.updated_at, i.indexed_at, i.last_viewed_at
         FROM images i WHERE i.workspace_id = ?1{}
         LIMIT ?{} OFFSET ?{}",
        filter_clause,
        all_params.len() + 1,
        all_params.len() + 2
    );
    all_params.push(rusqlite::types::Value::from(page_size));
    all_params.push(rusqlite::types::Value::from(offset));

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(all_params.iter()), row_to_image_record)?;
    let images: Result<Vec<ImageRecord>> = rows.collect();
    Ok(ImagePage {
        images: images?,
        total,
        page,
        page_size,
    })
}

/// Lightweight grid query: returns only the columns the grid actually renders
/// (no notes, no file_hash, no relative_path, no last_viewed_at, no file_created_at).
/// ~3x smaller payload, ~2x faster row decoding — the hot path on first load.
pub fn get_images_light(
    conn: &Connection,
    workspace_id: i64,
    filters: &ImageFilters,
    page: i32,
    page_size: i32,
) -> Result<ImageSummaryPage> {
    let (filter_clause, filter_params) = build_filter_clause(filters, 1);
    let count_sql = format!(
        "SELECT COUNT(*) FROM images i WHERE i.workspace_id = ?1{}",
        filter_clause
    );
    let mut all_params: Vec<rusqlite::types::Value> = Vec::with_capacity(filter_params.len() + 1);
    all_params.push(rusqlite::types::Value::from(workspace_id));
    all_params.extend(filter_params.iter().cloned());
    let total: i64 = conn.query_row(
        &count_sql,
        params_from_iter(all_params.iter()),
        |row| row.get(0),
    )?;

    let offset = (page - 1) * page_size;
    let sql = format!(
        "SELECT i.id, i.workspace_id, i.relative_path, i.file_name, i.file_size,
                i.width, i.height, i.format, i.thumbnail_path, i.rating,
                i.updated_at, i.indexed_at, i.created_at
         FROM images i WHERE i.workspace_id = ?1{}
         LIMIT ?{} OFFSET ?{}",
        filter_clause,
        all_params.len() + 1,
        all_params.len() + 2
    );
    all_params.push(rusqlite::types::Value::from(page_size));
    all_params.push(rusqlite::types::Value::from(offset));

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(all_params.iter()), row_to_image_summary)?;
    let images: Result<Vec<ImageSummary>> = rows.collect();
    Ok(ImageSummaryPage {
        images: images?,
        total,
        page,
        page_size,
    })
}

/// Fetch a batch of image summaries by id, preserving input order.
/// Used by the grid after a sort/filter change to refresh just the visible
/// set without re-paginating from scratch.
pub fn get_image_summaries_by_ids(
    conn: &Connection,
    ids: &[i64],
) -> Result<Vec<ImageSummary>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("?{}", i)).collect();
    let sql = format!(
        "SELECT i.id, i.workspace_id, i.relative_path, i.file_name, i.file_size,
                i.width, i.height, i.format, i.thumbnail_path, i.rating,
                i.updated_at, i.indexed_at, i.created_at
         FROM images i WHERE i.id IN ({})",
        placeholders.join(",")
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(ids.iter()), row_to_image_summary)?;
    let mut out: Vec<ImageSummary> = rows.collect::<Result<Vec<_>>>()?;
    // Preserve input order (SQLite does not guarantee IN(...) order).
    let pos: std::collections::HashMap<i64, usize> = ids.iter().enumerate().map(|(i, &id)| (id, i)).collect();
    out.sort_by_key(|s| pos.get(&s.id).copied().unwrap_or(usize::MAX));
    Ok(out)
}

pub fn get_image_detail(conn: &Connection, image_id: i64) -> Result<Option<ImageRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, relative_path, file_name, file_size, file_hash,
         width, height, format, thumbnail_path, notes, rating,
         created_at, file_created_at, updated_at, indexed_at, last_viewed_at
         FROM images WHERE id = ?1"
    )?;
    let mut rows = stmt.query_map(params![image_id], row_to_image_record)?;
    Ok(rows.next().transpose()?)
}

/// Sanitize user input for FTS5 MATCH — wrap in double quotes, escape internal quotes.
/// This treats the input as a literal phrase search, preventing syntax errors or query injection.
fn sanitize_fts_query(input: &str) -> String {
    let escaped = input.replace('"', "\"\"");
    format!("\"{}\"", escaped)
}

pub fn search_images(
    conn: &Connection,
    workspace_id: i64,
    query: &str,
    page: i32,
    page_size: i32,
) -> Result<ImagePage> {
    let fts_query = sanitize_fts_query(query);

    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM images_fts WHERE images_fts MATCH ?1 AND rowid IN (SELECT id FROM images WHERE workspace_id = ?2)",
        params![fts_query, workspace_id],
        |row| row.get(0),
    )?;

    let offset = (page - 1) * page_size;
    let mut stmt = conn.prepare(
        "SELECT i.id, i.workspace_id, i.relative_path, i.file_name, i.file_size, i.file_hash,
                i.width, i.height, i.format, i.thumbnail_path, i.notes, i.rating,
                i.created_at, i.file_created_at, i.updated_at, i.indexed_at, i.last_viewed_at
         FROM images i
         INNER JOIN images_fts fts ON i.id = fts.rowid
         WHERE images_fts MATCH ?1 AND i.workspace_id = ?2
         ORDER BY rank
         LIMIT ?3 OFFSET ?4"
    )?;
    let rows = stmt.query_map(params![fts_query, workspace_id, page_size, offset], row_to_image_record)?;
    let images: Result<Vec<ImageRecord>> = rows.collect();

    Ok(ImagePage {
        images: images?,
        total,
        page,
        page_size,
    })
}

/// Single-query duplicate detection.
///
/// Previous implementation ran 1 + N queries (one for hashes, then one per
/// hash to fetch its images). For a workspace with 50 duplicate groups this
/// meant 51 round-trips through the connection lock. This version uses a
/// single JOIN that streams all rows belonging to any duplicated hash in
/// the same workspace, sorted by hash so the frontend can group them
/// without re-sorting.
pub fn find_duplicate_groups(
    conn: &Connection,
    workspace_id: i64,
) -> Result<Vec<(String, Vec<ImageRecord>)>> {
    let mut stmt = conn.prepare(
        "WITH dup_hashes AS (
           SELECT file_hash FROM images
           WHERE workspace_id = ?1 AND file_hash IS NOT NULL
           GROUP BY file_hash HAVING COUNT(*) > 1
         )
         SELECT i.id, i.workspace_id, i.relative_path, i.file_name, i.file_size, i.file_hash,
                i.width, i.height, i.format, i.thumbnail_path, i.notes, i.rating,
                i.created_at, i.file_created_at, i.updated_at, i.indexed_at, i.last_viewed_at
         FROM images i
         INNER JOIN dup_hashes d ON i.file_hash = d.file_hash
         WHERE i.workspace_id = ?1
         ORDER BY i.file_hash, i.id"
    )?;
    let rows = stmt.query_map(params![workspace_id], row_to_image_record)?;
    let mut by_hash: std::collections::HashMap<String, Vec<ImageRecord>> = std::collections::HashMap::new();
    for row in rows {
        let r = row?;
        if let Some(h) = r.file_hash.clone() {
            by_hash.entry(h).or_default().push(r);
        }
    }
    let mut groups: Vec<(String, Vec<ImageRecord>)> = by_hash.into_iter().collect();
    // Stable order: largest groups first (more useful to the user).
    groups.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
    Ok(groups)
}

// ── Metadata ──

pub fn update_notes(conn: &Connection, image_id: i64, notes: &str) -> Result<()> {
    conn.execute(
        "UPDATE images SET notes = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![notes, image_id],
    )?;
    Ok(())
}

pub fn update_rating(conn: &Connection, image_id: i64, rating: i32) -> Result<()> {
    conn.execute(
        "UPDATE images SET rating = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![rating, image_id],
    )?;
    Ok(())
}

pub fn batch_update_tags(
    conn: &Connection,
    image_ids: &[i64],
    tag_ids: &[i64],
    action: &str,
) -> Result<()> {
    conn.execute_batch("BEGIN")?;
    let result = (|| -> Result<()> {
        match action {
            "add" => {
                for &img_id in image_ids {
                    for &tag_id in tag_ids {
                        conn.execute(
                            "INSERT OR IGNORE INTO image_tags (image_id, tag_id) VALUES (?1, ?2)",
                            params![img_id, tag_id],
                        )?;
                    }
                }
            }
            "remove" => {
                for &img_id in image_ids {
                    for &tag_id in tag_ids {
                        conn.execute(
                            "DELETE FROM image_tags WHERE image_id = ?1 AND tag_id = ?2",
                            params![img_id, tag_id],
                        )?;
                    }
                }
            }
            "set" => {
                for &img_id in image_ids {
                    conn.execute("DELETE FROM image_tags WHERE image_id = ?1", params![img_id])?;
                    for &tag_id in tag_ids {
                        conn.execute(
                            "INSERT OR IGNORE INTO image_tags (image_id, tag_id) VALUES (?1, ?2)",
                            params![img_id, tag_id],
                        )?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    })();
    if result.is_err() {
        conn.execute_batch("ROLLBACK").ok();
        return result;
    }
    conn.execute_batch("COMMIT")?;
    Ok(())
}

// ── Tags ──

/// Single-query tag list with image counts (per workspace).
///
/// Previous version used a correlated subquery per tag, so 100 tags = 100
/// COUNT(*) subqueries on image_tags × images. This version uses one
/// GROUP BY through the join, so the work is O(N_image_tags_in_workspace)
/// regardless of how many tags exist globally.
pub fn get_tags(conn: &Connection, workspace_id: i64) -> Result<Vec<TagWithCount>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color, COUNT(it.image_id) AS cnt
         FROM tags t
         INNER JOIN image_tags it ON t.id = it.tag_id
         INNER JOIN images i ON it.image_id = i.id
         WHERE i.workspace_id = ?1
         GROUP BY t.id, t.name, t.color
         ORDER BY cnt DESC, t.name ASC"
    )?;
    let rows = stmt.query_map(params![workspace_id], |row| {
        Ok(TagWithCount {
            tag: Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
            },
            image_count: row.get(3)?,
        })
    })?;
    rows.collect()
}

pub fn create_tag(conn: &Connection, name: &str, color: Option<&str>) -> Result<Tag> {
    conn.execute(
        "INSERT INTO tags (name, color) VALUES (?1, ?2)",
        params![name, color],
    )?;
    let id = conn.last_insert_rowid();
    Ok(Tag {
        id,
        name: name.to_string(),
        color: color.map(|s| s.to_string()),
    })
}

pub fn delete_tag(conn: &Connection, tag_id: i64) -> Result<()> {
    conn.execute("DELETE FROM tags WHERE id = ?1", params![tag_id])?;
    Ok(())
}

pub fn get_tags_for_image(conn: &Connection, image_id: i64) -> Result<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color FROM tags t
         INNER JOIN image_tags it ON t.id = it.tag_id
         WHERE it.image_id = ?1"
    )?;
    let rows = stmt.query_map(params![image_id], |row| {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
        })
    })?;
    rows.collect()
}

// ── Thumbnails ──

pub fn update_thumbnail_path(conn: &Connection, image_id: i64, path: &str) -> Result<()> {
    conn.execute(
        "UPDATE images SET thumbnail_path = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![path, image_id],
    )?;
    Ok(())
}

/// Fetch the (id, relative_path, workspace_id) tuples for a list of image ids
/// in a single SQL round-trip. The frontend invokes this before kicking off a
/// batch thumbnail generation so the Rust side can resolve workspace paths
/// and skip already-cached entries with a single query.
pub fn get_image_paths(conn: &Connection, ids: &[i64]) -> Result<Vec<(i64, i64, String)>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("?{}", i)).collect();
    let sql = format!(
        "SELECT id, workspace_id, relative_path FROM images WHERE id IN ({})",
        placeholders.join(",")
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(ids.iter()), |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, String>(2)?))
    })?;
    rows.collect()
}

// ── Helpers ──

fn chrono_now(conn: &Connection) -> String {
    conn.query_row("SELECT datetime('now')", [], |row| row.get(0))
        .unwrap_or_else(|_| "1970-01-01T00:00:00".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ImageFilters;

    #[test]
    fn build_filter_clause_with_tag_ids() {
        let filters = ImageFilters {
            tag_ids: Some(vec![123]),
            format: None,
            rating_min: None,
            date_from: None,
            date_to: None,
            sort_by: None,
            sort_dir: None,
        };
        let (clause, params) = build_filter_clause(&filters, 1);
        assert!(clause.contains("i.id IN (SELECT image_id FROM image_tags"), "clause: {clause}");
        assert!(clause.contains("tag_id IN (?2)"), "clause: {clause}");
        assert!(clause.contains("COUNT(DISTINCT tag_id) = 1"), "clause: {clause}");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], rusqlite::types::Value::from(123i64));
    }

    #[test]
    fn build_filter_clause_with_multiple_tags_intersect() {
        let filters = ImageFilters {
            tag_ids: Some(vec![10, 20]),
            format: None,
            rating_min: None,
            date_from: None,
            date_to: None,
            sort_by: None,
            sort_dir: None,
        };
        let (clause, params) = build_filter_clause(&filters, 1);
        assert!(clause.contains("tag_id IN (?2,?3)"), "clause: {clause}");
        assert!(clause.contains("COUNT(DISTINCT tag_id) = 2"), "clause: {clause}");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn build_filter_clause_with_empty_tag_ids_is_noop() {
        let filters = ImageFilters {
            tag_ids: Some(vec![]),
            format: None,
            rating_min: None,
            date_from: None,
            date_to: None,
            sort_by: None,
            sort_dir: None,
        };
        let (clause, params) = build_filter_clause(&filters, 1);
        assert!(!clause.contains("image_tags"), "empty tag list should be a no-op: {clause}");
        assert_eq!(params.len(), 0);
    }

    // ── Orphan tag cleanup on workspace delete ────────────────────────
    //
    // Tags are global (the `tags` table has no workspace_id column), so a
    // workspace deletion must also sweep tags whose only references lived
    // inside the deleted workspace. Otherwise the leftover rows are
    // unreachable from get_tags (which INNER JOINs through image_tags) but
    // still block create_tag via the UNIQUE(name) constraint — a deadlock
    // where the tag cannot be re-added and never reappears in the UI.

    fn fresh_db() -> Connection {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        conn.execute_batch(crate::db::schema::CREATE_TABLES_SQL)
            .expect("apply schema");
        conn
    }

    fn make_image(workspace_id: i64, name: &str) -> ImageRecord {
        ImageRecord {
            id: 0,
            workspace_id,
            relative_path: name.to_string(),
            file_name: name.to_string(),
            file_size: None,
            file_hash: None,
            width: None,
            height: None,
            format: None,
            thumbnail_path: None,
            notes: None,
            rating: None,
            created_at: None,
            file_created_at: None,
            updated_at: "1970-01-01T00:00:00".to_string(),
            indexed_at: "1970-01-01T00:00:00".to_string(),
            last_viewed_at: None,
        }
    }

    fn tag_exists(conn: &Connection, name: &str) -> bool {
        conn.query_row(
            "SELECT COUNT(*) FROM tags WHERE name = ?1",
            params![name],
            |row| row.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false)
    }

    #[test]
    fn delete_workspace_sweeps_orphan_tag() {
        let conn = fresh_db();
        let ws = insert_workspace(&conn, "w1", "/tmp/w1").unwrap();
        let img = insert_image(&conn, &make_image(ws.id, "a.jpg")).unwrap();
        let tag = create_tag(&conn, "beach", None).unwrap();
        batch_update_tags(&conn, &[img], &[tag.id], "add").unwrap();

        assert!(tag_exists(&conn, "beach"), "tag should exist before delete");
        delete_workspace(&conn, ws.id).unwrap();
        assert!(!tag_exists(&conn, "beach"), "orphan tag must be removed");

        // After cleanup the same name can be recreated — this is the
        // user-visible failure mode we are guarding against.
        let again = create_tag(&conn, "beach", None)
            .expect("re-creating 'beach' should succeed after sweep");
        assert!(again.id > 0);
    }

    #[test]
    fn delete_workspace_preserves_shared_tag() {
        let conn = fresh_db();
        let ws1 = insert_workspace(&conn, "w1", "/tmp/w1").unwrap();
        let ws2 = insert_workspace(&conn, "w2", "/tmp/w2").unwrap();
        let img1 = insert_image(&conn, &make_image(ws1.id, "a.jpg")).unwrap();
        let img2 = insert_image(&conn, &make_image(ws2.id, "b.jpg")).unwrap();
        let tag = create_tag(&conn, "beach", None).unwrap();
        batch_update_tags(&conn, &[img1, img2], &[tag.id], "add").unwrap();

        delete_workspace(&conn, ws1.id).unwrap();
        assert!(
            tag_exists(&conn, "beach"),
            "shared tag must survive ws1 deletion (still referenced by ws2)"
        );
    }
}
