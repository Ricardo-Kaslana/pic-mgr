use rusqlite::{params, Connection, Result, Row};
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

fn row_to_workspace(row: &Row) -> Result<Workspace> {
    Ok(Workspace {
        id: row.get(0)?,
        name: row.get(1)?,
        path: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
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
    let mut sql = String::from(
        "SELECT i.id, i.workspace_id, i.relative_path, i.file_name, i.file_size, i.file_hash,
                i.width, i.height, i.format, i.thumbnail_path, i.notes, i.rating,
                i.created_at, i.file_created_at, i.updated_at, i.indexed_at, i.last_viewed_at
         FROM images i WHERE i.workspace_id = ?1"
    );
    let mut param_idx = 2;
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(workspace_id)];

    if let Some(ref tag_ids) = filters.tag_ids {
        if !tag_ids.is_empty() {
            let placeholders: Vec<String> = tag_ids.iter().enumerate()
                .map(|(i, _)| format!("?{}", param_idx + i))
                .collect();
            sql.push_str(&format!(
                " AND i.id IN (SELECT image_id FROM image_tags WHERE tag_id IN ({}))",
                placeholders.join(",")
            ));
            for &tid in tag_ids {
                param_values.push(Box::new(tid));
                param_idx += 1;
            }
        }
    }

    if let Some(ref format) = filters.format {
        sql.push_str(&format!(" AND i.format = ?{}", param_idx));
        param_values.push(Box::new(format.clone()));
        param_idx += 1;
    }

    if let Some(rating_min) = filters.rating_min {
        sql.push_str(&format!(" AND i.rating >= ?{}", param_idx));
        param_values.push(Box::new(rating_min));
        param_idx += 1;
    }

    if let Some(ref date_from) = filters.date_from {
        sql.push_str(&format!(" AND i.created_at >= ?{}", param_idx));
        param_values.push(Box::new(date_from.clone()));
        param_idx += 1;
    }

    if let Some(ref date_to) = filters.date_to {
        sql.push_str(&format!(" AND i.created_at <= ?{}", param_idx));
        param_values.push(Box::new(date_to.clone()));
        param_idx += 1;
    }

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
    sql.push_str(&format!(" ORDER BY {} {}", sort_col, sort_dir));

    let count_sql = sql.replacen(
        "SELECT i.id, i.workspace_id, i.relative_path, i.file_name, i.file_size, i.file_hash,
                i.width, i.height, i.format, i.thumbnail_path, i.notes, i.rating,
                i.created_at, i.file_created_at, i.updated_at, i.indexed_at, i.last_viewed_at",
        "SELECT COUNT(*)",
        1,
    );
    let total: i64 = {
        let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
        conn.query_row(&count_sql, params_refs.as_slice(), |row| row.get(0))?
    };

    let offset = (page - 1) * page_size;
    sql.push_str(&format!(" LIMIT ?{} OFFSET ?{}", param_idx, param_idx + 1));
    param_values.push(Box::new(page_size));
    param_values.push(Box::new(offset));

    let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_refs.as_slice(), row_to_image_record)?;
    let images: Result<Vec<ImageRecord>> = rows.collect();

    Ok(ImagePage {
        images: images?,
        total,
        page,
        page_size,
    })
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

pub fn get_tags(conn: &Connection, workspace_id: i64) -> Result<Vec<TagWithCount>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color, COUNT(it.image_id) as cnt
         FROM tags t
         LEFT JOIN image_tags it ON t.id = it.tag_id
         LEFT JOIN images i ON it.image_id = i.id AND i.workspace_id = ?1
         GROUP BY t.id
         HAVING cnt > 0
         ORDER BY cnt DESC"
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

// ── Helpers ──

fn chrono_now(conn: &Connection) -> String {
    conn.query_row("SELECT datetime('now')", [], |row| row.get(0))
        .unwrap_or_else(|_| "1970-01-01T00:00:00".to_string())
}
