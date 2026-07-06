pub mod thumbnail;

use sha2::{Sha256, Digest};
use std::fs;
use std::io::Read;
use std::path::Path;
use std::time::UNIX_EPOCH;
use crate::models::ImageRecord;

const SUPPORTED_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff", "tif",
];

fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Streaming SHA-256 hash — reads file in 8KB chunks.
fn compute_file_hash(path: &Path) -> Option<String> {
    let file = fs::File::open(path).ok()?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    let mut reader = std::io::BufReader::new(file);
    loop {
        let n = match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => return None,
        };
        hasher.update(&buf[..n]);
    }
    Some(format!("{:x}", hasher.finalize()))
}

fn file_modified_at(metadata: &fs::Metadata) -> Option<String> {
    let modified = metadata.modified().ok()?;
    let duration = modified.duration_since(UNIX_EPOCH).ok()?;
    let dt = chrono_lite(duration.as_secs() as i64);
    Some(dt)
}

fn file_created_at(metadata: &fs::Metadata) -> Option<String> {
    let created = metadata.created().ok()?;
    let duration = created.duration_since(UNIX_EPOCH).ok()?;
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

/// Recursively collect image files using std::fs (no WalkDir dependency).
fn collect_image_files(dir: &Path, results: &mut Vec<std::path::PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if path.is_dir() {
            collect_image_files(&path, results);
        } else if path.is_file() && is_image_file(&path) {
            results.push(path);
        }
    }
}

pub fn scan_directory_individually(workspace_id: i64, workspace_path: &Path) -> Vec<ImageRecord> {
    let mut image_files = Vec::new();
    collect_image_files(workspace_path, &mut image_files);

    image_files
        .into_iter()
        .map(|path| {
            let relative = path.strip_prefix(workspace_path).unwrap_or(&path);
            let relative_str = relative.to_string_lossy().to_string();
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            let format = path.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());
            let metadata = fs::metadata(&path).ok();
            let file_size = metadata.as_ref().map(|m| m.len() as i64);
            let file_hash = compute_file_hash(&path);
            let created_at = metadata.as_ref().and_then(file_modified_at);
            let file_created_at = metadata.as_ref().and_then(file_created_at);
            let now = chrono_lite_now();

            ImageRecord {
                id: 0,
                workspace_id,
                relative_path: relative_str,
                file_name,
                file_size,
                file_hash,
                width: None,
                height: None,
                format,
                thumbnail_path: None,
                notes: None,
                rating: None,
                created_at,
                file_created_at,
                updated_at: now.clone(),
                indexed_at: now,
                last_viewed_at: None,
            }
        })
        .collect()
}
