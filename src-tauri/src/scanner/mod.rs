pub mod thumbnail;

use walkdir::WalkDir;
use sha2::{Sha256, Digest};
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;
use crate::models::ImageRecord;

const SUPPORTED_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff", "tif",
    "cr2", "nef", "arw",
];

fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn compute_file_hash(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Some(format!("{:x}", hasher.finalize()))
}

fn file_modified_at(path: &Path) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    let duration = modified.duration_since(UNIX_EPOCH).ok()?;
    let dt = chrono_lite(duration.as_secs() as i64);
    Some(dt)
}

fn chrono_lite(unix_ts: i64) -> String {
    let secs_per_day = 86400;
    let days = unix_ts / secs_per_day;
    let remaining = unix_ts % secs_per_day;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;
    let (y, m, d) = days_since_epoch_to_date(days);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}", y, m, d, hours, minutes, seconds)
}

fn days_since_epoch_to_date(days: i64) -> (i64, i64, i64) {
    let mut days = days;
    let mut year = 1970;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year { break; }
        days -= days_in_year;
        year += 1;
    }
    let month_lengths = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1;
    for &len in month_lengths.iter() {
        if days < len { break; }
        days -= len;
        month += 1;
    }
    (year, month, days + 1)
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

fn chrono_lite_now() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    chrono_lite(now.as_secs() as i64)
}

pub fn scan_directory_individually(workspace_id: i64, workspace_path: &Path) -> Vec<ImageRecord> {
    WalkDir::new(workspace_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && is_image_file(e.path()))
        .map(|entry| {
            let path = entry.path();
            let relative = path.strip_prefix(workspace_path).unwrap_or(path);
            let relative_str = relative.to_string_lossy().to_string();
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            let format = path.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());
            let file_size = fs::metadata(path).ok().map(|m| m.len() as i64);
            let file_hash = compute_file_hash(path);
            let created_at = file_modified_at(path);
            let now = chrono_lite_now();

            let (width, height) = image::image_dimensions(path)
                .map(|(w, h)| (Some(w as i32), Some(h as i32)))
                .unwrap_or((None, None));

            ImageRecord {
                id: 0,
                workspace_id,
                relative_path: relative_str,
                file_name,
                file_size,
                file_hash,
                width,
                height,
                format,
                thumbnail_path: None,
                notes: None,
                rating: None,
                created_at,
                updated_at: now.clone(),
                indexed_at: now,
                last_viewed_at: None,
            }
        })
        .collect()
}
