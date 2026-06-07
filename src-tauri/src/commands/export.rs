use tauri::State;
use crate::AppState;

#[tauri::command]
pub fn export_metadata(
    state: State<AppState>,
    workspace_id: i64,
    format: String,
) -> Result<String, String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare("SELECT id FROM images WHERE workspace_id = ?1")
        .map_err(|e| e.to_string())?;
    let image_ids: Vec<i64> = stmt.query_map(rusqlite::params![workspace_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut records = Vec::new();
    for id in image_ids {
        if let Ok(Some(record)) = crate::db::queries::get_image_detail(&conn, id) {
            let tags = crate::db::queries::get_tags_for_image(&conn, id).unwrap_or_default();
            records.push(serde_json::json!({
                "id": record.id,
                "file_name": record.file_name,
                "relative_path": record.relative_path,
                "file_size": record.file_size,
                "width": record.width,
                "height": record.height,
                "format": record.format,
                "notes": record.notes,
                "rating": record.rating,
                "tags": tags.iter().map(|t| &t.name).collect::<Vec<_>>(),
                "created_at": record.created_at,
                "updated_at": record.updated_at,
                "indexed_at": record.indexed_at,
            }));
        }
    }

    match format.as_str() {
        "json" => serde_json::to_string_pretty(&records).map_err(|e| e.to_string()),
        "csv" => {
            let mut csv = String::from("id,file_name,relative_path,format,width,height,file_size,rating,notes,tags,created_at\n");
            for r in &records {
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{},{},\"{}\",\"{}\",{}\n",
                    r["id"], r["file_name"].as_str().unwrap_or(""),
                    r["relative_path"].as_str().unwrap_or(""),
                    r["format"].as_str().unwrap_or(""),
                    r["width"].as_i64().unwrap_or(0),
                    r["height"].as_i64().unwrap_or(0),
                    r["file_size"].as_i64().unwrap_or(0),
                    r["rating"].as_i64().unwrap_or(0),
                    r["notes"].as_str().unwrap_or("").replace("\"", "\"\""),
                    r["tags"].as_array().map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(";")).unwrap_or_default(),
                    r["created_at"].as_str().unwrap_or(""),
                ));
            }
            Ok(csv)
        }
        _ => Err("Unsupported format. Use 'json' or 'csv'".to_string()),
    }
}
