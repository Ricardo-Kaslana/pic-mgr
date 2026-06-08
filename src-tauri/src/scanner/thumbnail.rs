use image::GenericImageView;
use std::path::{Path, PathBuf};

pub struct ThumbnailCache {
    pub cache_dir: PathBuf,
    pub size: u32,
}

impl ThumbnailCache {
    pub fn new(cache_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&cache_dir).ok();
        ThumbnailCache {
            cache_dir,
            size: 300,
        }
    }

    pub fn cache_path(&self, image_id: i64) -> PathBuf {
        self.cache_dir.join(format!("{}.webp", image_id))
    }

    pub fn generate(&self, source_path: &Path, image_id: i64) -> Result<String, String> {
        // Log file info for debugging
        let file_size = std::fs::metadata(source_path)
            .map(|m| m.len())
            .unwrap_or(0);
        let ext = source_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("none");

        // Read first 16 bytes to check magic bytes
        let magic = std::fs::read(source_path)
            .map(|buf| {
                if buf.len() >= 4 {
                    format!("{:02X} {:02X} {:02X} {:02X}", buf[0], buf[1], buf[2], buf[3])
                } else {
                    "file too small".to_string()
                }
            })
            .unwrap_or_else(|_| "cannot read".to_string());

        // Use Reader with with_guessed_format() to detect format from file header
        // instead of relying on extension
        let img = image::io::Reader::open(source_path)
            .map_err(|e| format!("Cannot open for reading: {}", e))?
            .with_guessed_format()
            .map_err(|e| format!("Cannot guess format: {}", e))?
            .decode()
            .map_err(|e| {
                let msg = format!(
                    "Cannot decode: {} | path={} ext={} size={}B magic=[{}]",
                    e,
                    source_path.display(),
                    ext,
                    file_size,
                    magic,
                );
                eprintln!("[thumbnail] {}", msg);
                msg
            })?;
        let (w, h) = img.dimensions();
        let aspect = w as f64 / h as f64;

        let (new_w, new_h) = if w > h {
            (self.size, (self.size as f64 / aspect) as u32)
        } else {
            ((self.size as f64 * aspect) as u32, self.size)
        };

        let thumbnail = img.thumbnail(new_w, new_h);
        // Use actual dimensions from thumbnail (may differ slightly from requested)
        let (actual_w, actual_h) = thumbnail.dimensions();
        let rgba = thumbnail.to_rgba8();
        let cache_path = self.cache_path(image_id);

        let mut output = Vec::new();
        let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut output);
        encoder
            .encode(rgba.as_raw(), actual_w, actual_h, image::ColorType::Rgba8.into())
            .map_err(|e| format!("Cannot encode WebP: {}", e))?;

        std::fs::write(&cache_path, &output)
            .map_err(|e| format!("Cannot write thumbnail: {}", e))?;

        Ok(cache_path.to_string_lossy().to_string())
    }

    pub fn get_bytes(&self, image_id: i64) -> Option<Vec<u8>> {
        let path = self.cache_path(image_id);
        std::fs::read(&path).ok()
    }
}
