//! Image transformation and optimization module
//! Provides server-side and client-side image processing capabilities

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use parking_lot::RwLock;

#[cfg(not(target_arch = "wasm32"))]
use image::{ImageFormat, ImageError};

/// Supported image formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    Jpeg,
    Png,
    WebP,
    Avif,
    Gif,
}

impl Format {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => Some(Format::Jpeg),
            "png" => Some(Format::Png),
            "webp" => Some(Format::WebP),
            "avif" => Some(Format::Avif),
            "gif" => Some(Format::Gif),
            _ => None,
        }
    }

    pub fn to_mime_type(&self) -> &'static str {
        match self {
            Format::Jpeg => "image/jpeg",
            Format::Png => "image/png",
            Format::WebP => "image/webp",
            Format::Avif => "image/avif",
            Format::Gif => "image/gif",
        }
    }

    pub fn to_extension(&self) -> &'static str {
        match self {
            Format::Jpeg => "jpg",
            Format::Png => "png",
            Format::WebP => "webp",
            Format::Avif => "avif",
            Format::Gif => "gif",
        }
    }
}

/// Image transformation parameters
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransformParams {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: Option<Format>,
    pub quality: u8,
    pub blur: Option<u8>,
}

impl Default for TransformParams {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            format: None,
            quality: 85,
            blur: None,
        }
    }
}

impl TransformParams {
    /// Parse transform parameters from URL query string
    pub fn from_query(query: &str) -> Self {
        let mut params = Self::default();
        
        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                match key {
                    "w" | "width" => {
                        params.width = value.parse().ok();
                    }
                    "h" | "height" => {
                        params.height = value.parse().ok();
                    }
                    "f" | "format" => {
                        params.format = Format::from_extension(value);
                    }
                    "q" | "quality" => {
                        params.quality = value.parse().unwrap_or(85).clamp(1, 100);
                    }
                    "blur" => {
                        params.blur = value.parse().ok();
                    }
                    _ => {}
                }
            }
        }
        
        params
    }

    /// Generate cache key for these parameters
    pub fn cache_key(&self, source: &str) -> String {
        let mut key = source.to_string();
        
        if let Some(w) = self.width {
            key.push_str(&format!("_w{}", w));
        }
        if let Some(h) = self.height {
            key.push_str(&format!("_h{}", h));
        }
        if let Some(f) = &self.format {
            key.push_str(&format!("_{}", f.to_extension()));
        }
        key.push_str(&format!("_q{}", self.quality));
        if let Some(b) = self.blur {
            key.push_str(&format!("_blur{}", b));
        }
        
        key
    }
}

/// Image cache entry
pub struct CacheEntry {
    pub data: Vec<u8>,
    pub format: Format,
    pub width: u32,
    pub height: u32,
}

/// Image transformer with caching
pub struct ImageTransformer {
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    max_cache_size: usize,
}

impl ImageTransformer {
    pub fn new(max_cache_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_cache_size,
        }
    }

    /// Transform image with given parameters (server-side only)
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn transform(
        &self,
        source_data: &[u8],
        source_path: &str,
        params: &TransformParams,
    ) -> Result<CacheEntry, ImageError> {
        let cache_key = params.cache_key(source_path);
        
        // Check cache first
        {
            let cache = self.cache.read();
            if let Some(entry) = cache.get(&cache_key) {
                return Ok(CacheEntry {
                    data: entry.data.clone(),
                    format: entry.format,
                    width: entry.width,
                    height: entry.height,
                });
            }
        }
        
        // Load and process image
        let img = image::load_from_memory(source_data)?;
        let mut result = img;
        
        // Resize if needed
        if params.width.is_some() || params.height.is_some() {
            let (orig_width, orig_height) = (result.width(), result.height());
            let (new_width, new_height) = calculate_dimensions(
                orig_width,
                orig_height,
                params.width,
                params.height,
            );
            
            result = result.resize(
                new_width,
                new_height,
                image::imageops::FilterType::Lanczos3,
            );
        }
        
        // Apply blur if requested
        if let Some(blur_radius) = params.blur {
            result = result.blur(blur_radius as f32);
        }
        
        // Determine output format
        let output_format = params.format.unwrap_or_else(|| {
            // Auto-detect from source path
            Path::new(source_path)
                .extension()
                .and_then(|ext| ext.to_str())
                .and_then(Format::from_extension)
                .unwrap_or(Format::Jpeg)
        });
        
        // Encode to desired format
        let mut output = Vec::new();
        let image_format = match output_format {
            Format::Jpeg => ImageFormat::Jpeg,
            Format::Png => ImageFormat::Png,
            Format::WebP => ImageFormat::WebP,
            Format::Gif => ImageFormat::Gif,
            Format::Avif => {
                // AVIF support requires additional crate
                // For now, fallback to WebP
                ImageFormat::WebP
            }
        };
        
        // Apply quality settings for lossy formats
        match image_format {
            ImageFormat::Jpeg => {
                use image::codecs::jpeg::JpegEncoder;
                use std::io::Cursor;
                let mut cursor = Cursor::new(&mut output);
                let encoder = JpegEncoder::new_with_quality(&mut cursor, params.quality);
                result.write_with_encoder(encoder)?;
            }
            ImageFormat::WebP => {
                // WebP encoding requires webp crate
                // For now, use PNG as fallback
                result.write_to(&mut std::io::Cursor::new(&mut output), ImageFormat::Png)?;
            }
            _ => {
                result.write_to(&mut std::io::Cursor::new(&mut output), image_format)?;
            }
        }
        
        let entry = CacheEntry {
            data: output.clone(),
            format: output_format,
            width: result.width(),
            height: result.height(),
        };
        
        // Store in cache
        {
            let mut cache = self.cache.write();
            
            // Simple cache eviction: remove oldest entries if cache is too large
            if cache.len() >= self.max_cache_size {
                // In production, use LRU eviction
                cache.clear();
            }
            
            cache.insert(cache_key, CacheEntry {
                data: output,
                format: output_format,
                width: result.width(),
                height: result.height(),
            });
        }
        
        Ok(entry)
    }

    /// Generate blur placeholder (server-side only)
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn generate_blur_placeholder(
        &self,
        source_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<String, ImageError> {
        let img = image::load_from_memory(source_data)?;
        
        // Resize to small size for placeholder
        let placeholder = img.resize(
            width.min(20),
            height.min(20),
            image::imageops::FilterType::Gaussian,
        );
        
        // Apply heavy blur
        let blurred = placeholder.blur(5.0);
        
        // Encode to base64
        let mut png_data = Vec::new();
        blurred.write_to(&mut std::io::Cursor::new(&mut png_data), ImageFormat::Png)?;
        
        use base64::Engine as _;
        Ok(format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(&png_data)
        ))
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        self.cache.write().clear();
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.read().len()
    }
}

/// Calculate target dimensions preserving aspect ratio
fn calculate_dimensions(
    orig_width: u32,
    orig_height: u32,
    target_width: Option<u32>,
    target_height: Option<u32>,
) -> (u32, u32) {
    match (target_width, target_height) {
        (Some(w), Some(h)) => (w, h),
        (Some(w), None) => {
            let ratio = w as f32 / orig_width as f32;
            (w, (orig_height as f32 * ratio) as u32)
        }
        (None, Some(h)) => {
            let ratio = h as f32 / orig_height as f32;
            ((orig_width as f32 * ratio) as u32, h)
        }
        (None, None) => (orig_width, orig_height),
    }
}

/// Generate responsive srcset
pub fn generate_srcset(base_url: &str, widths: &[u32]) -> String {
    widths
        .iter()
        .map(|&w| format!("{}?w={} {}w", base_url, w, w))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Generate sizes attribute for responsive images
pub fn generate_sizes(breakpoints: &[(u32, u32)]) -> String {
    breakpoints
        .iter()
        .map(|(breakpoint, width)| {
            format!("(max-width: {}px) {}px", breakpoint, width)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Image optimization configuration
#[derive(Debug, Clone)]
pub struct ImageConfig {
    /// Base URL for image optimization service
    pub base_url: String,
    /// Supported widths for responsive images
    pub responsive_widths: Vec<u32>,
    /// Default quality
    pub default_quality: u8,
    /// Enable WebP auto-conversion
    pub auto_webp: bool,
    /// Enable AVIF auto-conversion
    pub auto_avif: bool,
    /// Cache directory for optimized images
    pub cache_dir: Option<String>,
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            base_url: "/_layer9/image".to_string(),
            responsive_widths: vec![320, 640, 768, 1024, 1280, 1536, 1920, 2560],
            default_quality: 85,
            auto_webp: true,
            auto_avif: false,
            cache_dir: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_params_from_query() {
        let params = TransformParams::from_query("w=800&h=600&q=90&format=webp");
        assert_eq!(params.width, Some(800));
        assert_eq!(params.height, Some(600));
        assert_eq!(params.quality, 90);
        assert_eq!(params.format, Some(Format::WebP));
    }

    #[test]
    fn test_cache_key_generation() {
        let params = TransformParams {
            width: Some(800),
            height: Some(600),
            format: Some(Format::WebP),
            quality: 90,
            blur: None,
        };
        
        let key = params.cache_key("/images/test.jpg");
        assert_eq!(key, "/images/test.jpg_w800_h600_webp_q90");
    }

    #[test]
    fn test_dimension_calculation() {
        // Test width only
        let (w, h) = calculate_dimensions(1000, 800, Some(500), None);
        assert_eq!(w, 500);
        assert_eq!(h, 400);
        
        // Test height only
        let (w, h) = calculate_dimensions(1000, 800, None, Some(400));
        assert_eq!(w, 500);
        assert_eq!(h, 400);
        
        // Test both
        let (w, h) = calculate_dimensions(1000, 800, Some(600), Some(600));
        assert_eq!(w, 600);
        assert_eq!(h, 600);
    }
}