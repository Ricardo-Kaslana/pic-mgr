use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRecord {
    pub id: i64,
    pub workspace_id: i64,
    pub relative_path: String,
    pub file_name: String,
    pub file_size: Option<i64>,
    pub file_hash: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub format: Option<String>,
    pub thumbnail_path: Option<String>,
    pub notes: Option<String>,
    pub rating: Option<i32>,
    pub created_at: Option<String>,
    pub updated_at: String,
    pub indexed_at: String,
    pub last_viewed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageDetail {
    #[serde(flatten)]
    pub record: ImageRecord,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagWithCount {
    #[serde(flatten)]
    pub tag: Tag,
    pub image_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagePage {
    pub images: Vec<ImageRecord>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageFilters {
    pub tag_ids: Option<Vec<i64>>,
    pub format: Option<String>,
    pub rating_min: Option<i32>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub total: i64,
    pub processed: i64,
    pub new_files: i64,
    pub modified_files: i64,
    pub finished: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThumbnailProgress {
    pub total: i64,
    pub generated: i64,
    pub failed: i64,
    pub finished: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub format: String,
}
