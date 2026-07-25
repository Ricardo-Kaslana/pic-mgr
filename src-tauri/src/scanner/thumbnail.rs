use image::codecs::jpeg::JpegEncoder;
use image::{ExtendedColorType, GenericImageView};
use std::io::Cursor;
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
        self.cache_dir.join(format!("{}.jpg", image_id))
    }

    /// Returns (cache_path, original_width, original_height) so callers can
    /// backfill the width/height columns, which are NULL at scan time.
    pub fn generate(&self, source_path: &Path, image_id: i64) -> Result<(String, u32, u32), String> {
        // Read file once into buffer (was 3 separate reads: metadata + magic + decode)
        let buf = std::fs::read(source_path)
            .map_err(|e| format!("Cannot read file: {}", e))?;
        let file_size = buf.len();

        let ext = source_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("none");

        let magic = if buf.len() >= 4 {
            format!("{:02X} {:02X} {:02X} {:02X}", buf[0], buf[1], buf[2], buf[3])
        } else {
            "file too small".to_string()
        };

        // Decode from buffer instead of re-reading from disk
        let img = image::ImageReader::new(Cursor::new(&buf))
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
        let rgb = thumbnail.to_rgb8();
        let cache_path = self.cache_path(image_id);

        let mut output = Vec::new();
        // JPEG quality 80 — visually identical for thumbnails, 10-20x faster than WebP lossless
        let mut encoder = JpegEncoder::new_with_quality(&mut output, 80);
        encoder
            .encode(rgb.as_raw(), actual_w, actual_h, ExtendedColorType::Rgb8)
            .map_err(|e| format!("Cannot encode JPEG: {}", e))?;

        std::fs::write(&cache_path, &output)
            .map_err(|e| format!("Cannot write thumbnail: {}", e))?;

        Ok((cache_path.to_string_lossy().to_string(), w, h))
    }

    pub fn get_bytes(&self, image_id: i64) -> Option<Vec<u8>> {
        let path = self.cache_path(image_id);
        std::fs::read(&path).ok()
    }
}
