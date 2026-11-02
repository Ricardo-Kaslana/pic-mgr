use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn export_metadata(
    state: State<AppState>,
    workspace_id: i64,
    image_ids: Option<Vec<i64>>,
    format: String,
) -> Result<String, String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;

    // Step 1: Resolve IDs
    let ids: Vec<i64> = match image_ids {
        Some(ids) => ids,
        None => {
            let mut stmt = conn
                .prepare("SELECT id FROM images WHERE workspace_id = ?1")
                .map_err(|e| e.to_string())?;
            let result: Vec<i64> = stmt
                .query_map(rusqlite::params![workspace_id], |row| row.get(0))
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok())
                .collect();
            result
        }
    };

    if ids.is_empty() {
        return match format.as_str() {
            "json" => Ok("[]".to_string()),
            "csv" => Ok("id,file_name,relative_path,format,width,height,file_size,rating,notes,tags,created_at,file_created_at\n".to_string()),
            _ => Err("Unsupported format. Use 'json' or 'csv'".to_string()),
        };
    }

    // Step 2: Batch-fetch all images in one query (was N+1)
    let placeholders: Vec<String> = ids
        .iter()
        .enumerate()
        .map(|(i, _)| format!("?{}", i + 1))
        .collect();
    let sql = format!(
        "SELECT id, workspace_id, relative_path, file_name, file_size, file_hash,
                width, height, format, thumbnail_path, notes, rating,
                created_at, file_created_at, updated_at, indexed_at, last_viewed_at
         FROM images WHERE id IN ({})",
        placeholders.join(",")
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let params: Vec<Box<dyn rusqlite::types::ToSql>> = ids
        .iter()
        .map(|&id| Box::new(id) as Box<dyn rusqlite::types::ToSql>)
        .collect();
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let records: Vec<crate::models::ImageRecord> = stmt
        .query_map(
            param_refs.as_slice(),
            crate::db::queries::row_to_image_record,
        )
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    // Step 3: Batch-fetch all tags in one query (was N queries)
    let id_placeholders: Vec<String> = ids
        .iter()
        .enumerate()
        .map(|(i, _)| format!("?{}", i + 1))
        .collect();
    let tag_sql = format!(
        "SELECT it.image_id, t.id, t.name, t.color FROM tags t
         INNER JOIN image_tags it ON t.id = it.tag_id
         WHERE it.image_id IN ({})",
        id_placeholders.join(",")
    );
    let mut tag_stmt = conn.prepare(&tag_sql).map_err(|e| e.to_string())?;
    let tag_params: Vec<Box<dyn rusqlite::types::ToSql>> = ids
        .iter()
        .map(|&id| Box::new(id) as Box<dyn rusqlite::types::ToSql>)
        .collect();
    let tag_param_refs: Vec<&dyn rusqlite::types::ToSql> =
        tag_params.iter().map(|p| p.as_ref()).collect();
    let mut tags_by_image: std::collections::HashMap<i64, Vec<String>> =
        std::collections::HashMap::new();
    let tag_rows = tag_stmt
        .query_map(tag_param_refs.as_slice(), |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(2)?))
        })
        .map_err(|e| e.to_string())?;
    for row in tag_rows {
        if let Ok((image_id, tag_name)) = row {
            tags_by_image.entry(image_id).or_default().push(tag_name);
        }
    }

    // Step 4: Build export records
    let export_records: Vec<serde_json::Value> = records
        .iter()
        .map(|r| {
            let tags = tags_by_image.get(&r.id).cloned().unwrap_or_default();
            serde_json::json!({
                "id": r.id,
                "file_name": r.file_name,
                "relative_path": r.relative_path,
                "file_size": r.file_size,
                "width": r.width,
                "height": r.height,
                "format": r.format,
                "notes": r.notes,
                "rating": r.rating,
                "tags": tags,
                "created_at": r.created_at,
                "file_created_at": r.file_created_at,
                "updated_at": r.updated_at,
                "indexed_at": r.indexed_at,
            })
        })
        .collect();

    match format.as_str() {
        "json" => serde_json::to_string_pretty(&export_records).map_err(|e| e.to_string()),
        "csv" => {
            let mut csv = String::from("id,file_name,relative_path,format,width,height,file_size,rating,notes,tags,created_at,file_created_at\n");
            for r in &export_records {
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{},{},\"{}\",\"{}\",{},{}\n",
                    r["id"],
                    csv_escape(r["file_name"].as_str().unwrap_or("")),
                    csv_escape(r["relative_path"].as_str().unwrap_or("")),
                    csv_escape(r["format"].as_str().unwrap_or("")),
                    r["width"].as_i64().unwrap_or(0),
                    r["height"].as_i64().unwrap_or(0),
                    r["file_size"].as_i64().unwrap_or(0),
                    r["rating"].as_i64().unwrap_or(0),
                    r["notes"].as_str().unwrap_or("").replace("\"", "\"\""),
                    r["tags"]
                        .as_array()
                        .map(|a| a
                            .iter()
                            .filter_map(|v| v.as_str())
                            .collect::<Vec<_>>()
                            .join(";"))
                        .unwrap_or_default(),
                    r["created_at"].as_str().unwrap_or(""),
                    r["file_created_at"].as_str().unwrap_or(""),
                ));
            }
            Ok(csv)
        }
        _ => Err("Unsupported format. Use 'json' or 'csv'".to_string()),
    }
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
