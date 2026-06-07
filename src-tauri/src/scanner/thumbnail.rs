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
        let img = image::open(source_path).map_err(|e| format!("Cannot open image: {}", e))?;
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
