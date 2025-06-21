//! HTTP handler for image optimization requests

#[cfg(feature = "ssr")]
use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[cfg(feature = "ssr")]
use crate::image_transform::{ImageTransformer, TransformParams, Format};

/// Query parameters for image optimization
#[derive(Debug, Deserialize, Serialize)]
pub struct ImageQuery {
    /// Source image URL or path
    pub src: String,
    /// Target width
    pub w: Option<u32>,
    /// Target height
    pub h: Option<u32>,
    /// Output format
    pub f: Option<String>,
    /// Quality (1-100)
    pub q: Option<u8>,
    /// Blur radius
    pub blur: Option<u8>,
}

impl From<ImageQuery> for TransformParams {
    fn from(query: ImageQuery) -> Self {
        TransformParams {
            width: query.w,
            height: query.h,
            format: query.f.as_deref().and_then(Format::from_extension),
            quality: query.q.unwrap_or(85),
            blur: query.blur,
        }
    }
}

/// Shared application state for image handler
#[cfg(feature = "ssr")]
pub struct ImageHandlerState {
    pub transformer: Arc<ImageTransformer>,
    pub allowed_domains: Vec<String>,
    pub cache_dir: Option<String>,
}

/// Handle image optimization requests
#[cfg(feature = "ssr")]
pub async fn handle_image_request(
    Query(query): Query<ImageQuery>,
    State(state): State<Arc<ImageHandlerState>>,
) -> Result<Response, StatusCode> {
    // Validate source URL
    if query.src.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Security: Validate allowed domains for remote URLs
    if query.src.starts_with("http") {
        let is_allowed = state.allowed_domains.iter().any(|domain| {
            query.src.contains(domain)
        });
        
        if !is_allowed {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // Fetch source image
    let image_data = if query.src.starts_with("http") {
        // Fetch remote image
        match fetch_remote_image(&query.src).await {
            Ok(data) => data,
            Err(_) => return Err(StatusCode::NOT_FOUND),
        }
    } else {
        // Load local image
        match load_local_image(&query.src, state.cache_dir.as_deref()).await {
            Ok(data) => data,
            Err(_) => return Err(StatusCode::NOT_FOUND),
        }
    };

    // Transform image
    let src = query.src.clone();
    let params = query.into();
    match state.transformer.transform(&image_data, &src, &params).await {
        Ok(result) => {
            // Build response with proper headers
            let response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, result.format.to_mime_type())
                .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
                .header("X-Layer9-Width", result.width.to_string())
                .header("X-Layer9-Height", result.height.to_string())
                .body(result.data.into())
                .unwrap();
                
            Ok(response)
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Fetch image from remote URL
#[cfg(feature = "ssr")]
async fn fetch_remote_image(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let response = reqwest::get(url).await?;
    if !response.status().is_success() {
        return Err("Failed to fetch image".into());
    }
    
    let bytes = response.bytes().await?;
    Ok(bytes.to_vec())
}

/// Load image from local filesystem
#[cfg(feature = "ssr")]
async fn load_local_image(
    path: &str,
    base_dir: Option<&str>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use tokio::fs;
    
    let full_path = if let Some(base) = base_dir {
        format!("{}/{}", base, path.trim_start_matches('/'))
    } else {
        format!("./public{}", path)
    };
    
    // Prevent directory traversal
    if full_path.contains("..") {
        return Err("Invalid path".into());
    }
    
    let data = fs::read(&full_path).await?;
    Ok(data)
}

/// Create image optimization router
#[cfg(feature = "ssr")]
pub fn create_image_router(
    transformer: Arc<ImageTransformer>,
    allowed_domains: Vec<String>,
    cache_dir: Option<String>,
) -> axum::Router {
    use axum::routing::get;
    
    let state = Arc::new(ImageHandlerState {
        transformer,
        allowed_domains,
        cache_dir,
    });
    
    axum::Router::new()
        .route("/_layer9/image", get(handle_image_request))
        .with_state(state)
}

/// Client-side image URL builder
pub struct ImageUrlBuilder {
    base_url: String,
    src: String,
    params: TransformParams,
}

impl ImageUrlBuilder {
    pub fn new(src: impl Into<String>) -> Self {
        Self {
            base_url: "/_layer9/image".to_string(),
            src: src.into(),
            params: TransformParams::default(),
        }
    }

    pub fn width(mut self, width: u32) -> Self {
        self.params.width = Some(width);
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.params.height = Some(height);
        self
    }

    pub fn format(mut self, format: Format) -> Self {
        self.params.format = Some(format);
        self
    }

    pub fn quality(mut self, quality: u8) -> Self {
        self.params.quality = quality;
        self
    }

    pub fn blur(mut self, blur: u8) -> Self {
        self.params.blur = Some(blur);
        self
    }

    pub fn build(&self) -> String {
        let mut params = vec![format!("src={}", urlencoding::encode(&self.src))];
        
        if let Some(w) = self.params.width {
            params.push(format!("w={}", w));
        }
        if let Some(h) = self.params.height {
            params.push(format!("h={}", h));
        }
        if let Some(f) = &self.params.format {
            params.push(format!("f={}", f.to_extension()));
        }
        if self.params.quality != 85 {
            params.push(format!("q={}", self.params.quality));
        }
        if let Some(b) = self.params.blur {
            params.push(format!("blur={}", b));
        }
        
        format!("{}?{}", self.base_url, params.join("&"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_url_builder() {
        let url = ImageUrlBuilder::new("/images/hero.jpg")
            .width(800)
            .height(600)
            .quality(90)
            .build();
            
        assert_eq!(url, "/_layer9/image?src=%2Fimages%2Fhero.jpg&w=800&h=600&q=90");
    }

    #[test]
    fn test_transform_params_from_query() {
        let query = ImageQuery {
            src: "/test.jpg".to_string(),
            w: Some(1024),
            h: Some(768),
            f: Some("webp".to_string()),
            q: Some(80),
            blur: None,
        };
        
        let params: TransformParams = query.into();
        assert_eq!(params.width, Some(1024));
        assert_eq!(params.height, Some(768));
        assert_eq!(params.format, Some(Format::WebP));
        assert_eq!(params.quality, 80);
        assert_eq!(params.blur, None);
    }
}