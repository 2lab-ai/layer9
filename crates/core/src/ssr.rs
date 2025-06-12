//! Server-Side Rendering - L2/L3

use crate::prelude::*;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::collections::HashMap;
use wasm_bindgen::prelude::wasm_bindgen;

/// SSR Context
pub struct SSRContext {
    pub url: String,
    pub params: HashMap<String, String>,
    pub query: HashMap<String, String>,
    pub headers: HashMap<String, String>,
}

/// Trait for SSR-capable components
pub trait SSRComponent: Component {
    /// Server-side data fetching
    async fn get_server_props(_ctx: &SSRContext) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({}))
    }

    /// Render to HTML string
    fn render_to_string(&self) -> String {
        let element = self.render();
        element_to_html(&element)
    }
}

/// Convert Element to HTML string
fn element_to_html(element: &Element) -> String {
    match element {
        Element::Text(text) => html_escape(text),
        Element::Node {
            tag,
            props,
            children,
        } => {
            let mut html = format!("<{}", tag);

            // Add attributes
            if let Some(class) = &props.class {
                html.push_str(&format!(r#" class="{}""#, html_escape(class)));
            }
            if let Some(id) = &props.id {
                html.push_str(&format!(r#" id="{}""#, html_escape(id)));
            }
            for (key, value) in &props.attributes {
                html.push_str(&format!(r#" {}="{}""#, key, html_escape(value)));
            }

            html.push('>');

            // Add children
            for child in children {
                html.push_str(&element_to_html(child));
            }

            html.push_str(&format!("</{}>", tag));
            html
        }
        Element::Component(component) => element_to_html(&component.render()),
    }
}

/// HTML escape
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// SSR Configuration
pub struct SSRConfig {
    pub wasm_dir: String,
    pub bundle_name: String,
    pub static_dir: Option<String>,
}

impl Default for SSRConfig {
    fn default() -> Self {
        SSRConfig {
            wasm_dir: "pkg".to_string(),
            bundle_name: "layer9_bundle".to_string(),
            static_dir: Some("static".to_string()),
        }
    }
}

/// SSR App trait
pub trait SSRApp: Layer9App + Sync {
    /// Get SSR configuration
    fn ssr_config(&self) -> SSRConfig {
        SSRConfig::default()
    }
    
    /// Get HTML template
    fn html_template(&self) -> &'static str {
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>{styles}</style>
    <script>window.__Layer9_PROPS__ = {props};</script>
</head>
<body>
    <div id="layer9-root">{content}</div>
    <script type="module">
        import init from '/{wasm_dir}/{bundle_name}.js';
        init('/{wasm_dir}/{bundle_name}_bg.wasm').then(() => {
            if (window.__Layer9_HYDRATE__) {
                window.__Layer9_HYDRATE__();
            }
        });
    </script>
</body>
</html>"#
    }

    /// Render page on server
    fn render_page(
        &self,
        route: &str,
        ctx: SSRContext,
    ) -> impl std::future::Future<Output = Result<String, StatusCode>> + Send {
        async move {
            // Find matching route
            let routes = self.routes();
            let route_match = routes.iter().find(|r| r.path == route);

            if let Some(route) = route_match {
                match &route.handler {
                    RouteHandler::Page(page_fn) => {
                        let page = page_fn();

                        // Render component to HTML
                        let element = page.component.render();
                        let content = element_to_html(&element);

                    // Get server props
                    let props = serde_json::json!({
                        "url": ctx.url,
                        "params": ctx.params,
                        "query": ctx.query,
                    });

                    // Build final HTML
                    let config = self.ssr_config();
                    let html = self
                        .html_template()
                        .replace("{title}", &page.title)
                        .replace("{styles}", &get_critical_css())
                        .replace("{props}", &props.to_string())
                        .replace("{content}", &content)
                        .replace("{wasm_dir}", &config.wasm_dir)
                        .replace("{bundle_name}", &config.bundle_name);

                        Ok(html)
                    }
                    _ => Err(StatusCode::NOT_FOUND),
                }
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
    }
}

/// Get critical CSS for initial render
fn get_critical_css() -> String {
    // In production, this would extract only used styles
    r#"
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body { 
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        background: #000; 
        color: #fff; 
        line-height: 1.5; 
    }
    .loading { 
        display: flex; 
        align-items: center; 
        justify-content: center; 
        min-height: 100vh; 
    }
    "#
    .to_string()
}

/// Create Axum server for SSR
pub fn create_ssr_server<T: SSRApp + Send + Sync + 'static>(app: T) -> Router {
    use tower_http::services::ServeDir;
    
    let config = app.ssr_config();
    let app = std::sync::Arc::new(app);

    let mut router = Router::new()
        // Serve static assets (WASM, JS, CSS)
        .nest_service(&format!("/{}", config.wasm_dir), ServeDir::new(&config.wasm_dir));
    
    // Add static directory if configured
    if let Some(static_dir) = config.static_dir {
        router = router.nest_service("/static", ServeDir::new(static_dir));
    }
    
    // Mount database API if database is initialized
    #[cfg(feature = "ssr")]
    {
        if let Ok(db_conn) = crate::db_sqlx::get_db_connection() {
            router = router.nest("/api/db", crate::db_api::create_db_api_router(db_conn));
        } else {
            router = router.route("/api/db/*path", get(|| async { 
                (StatusCode::SERVICE_UNAVAILABLE, "Database not initialized")
            }));
        }
    }
    
    router
        // Other API routes if needed
        .route("/api/*path", get(|| async { StatusCode::NOT_IMPLEMENTED }))
        // Catch-all route for SSR (must be last)
        .fallback(
            get(move |Path(path): Path<String>| {
                let app = app.clone();
                async move {
                    let ctx = SSRContext {
                        url: format!("/{}", path),
                        params: HashMap::new(),
                        query: HashMap::new(),
                        headers: HashMap::new(),
                    };

                    match app.render_page(&format!("/{}", path), ctx).await {
                        Ok(html) => Html(html).into_response(),
                        Err(status) => status.into_response(),
                    }
                }
            }),
        )
}

/// Hydration helper for client side
/// Note: This function should be called from JavaScript after WASM is loaded
pub fn hydrate_app_internal(app: impl Layer9App + 'static) {
    // Get props from server
    let window = web_sys::window().unwrap();
    let _props: Option<serde_json::Value> = js_sys::Reflect::get(&window, &"__Layer9_PROPS__".into())
        .ok()
        .and_then(|v| serde_wasm_bindgen::from_value(v).ok());

    // Initialize app with server props
    run_app(app);
}

/// Export hydration function for WASM
#[wasm_bindgen]
pub fn hydrate() {
    // This would be implemented by the specific app
    // For now, just log
    web_sys::console::log_1(&"Layer9 hydration called".into());
}

/// Static Site Generation
pub struct SSG {
    routes: Vec<String>,
    output_dir: String,
}

impl SSG {
    pub fn new(output_dir: impl Into<String>) -> Self {
        SSG {
            routes: vec![],
            output_dir: output_dir.into(),
        }
    }

    pub fn add_route(mut self, route: impl Into<String>) -> Self {
        self.routes.push(route.into());
        self
    }

    pub async fn generate<T: SSRApp>(self, app: T) -> Result<(), String> {
        use tokio::fs;

        // Create output directory
        fs::create_dir_all(&self.output_dir)
            .await
            .map_err(|e| e.to_string())?;

        // Generate each route
        for route in self.routes {
            let ctx = SSRContext {
                url: route.clone(),
                params: HashMap::new(),
                query: HashMap::new(),
                headers: HashMap::new(),
            };

            let html = app
                .render_page(&route, ctx)
                .await
                .map_err(|_| "Failed to render page")?;

            // Write to file
            let file_path = if route == "/" {
                format!("{}/index.html", self.output_dir)
            } else {
                format!("{}{}.html", self.output_dir, route)
            };

            fs::write(&file_path, html)
                .await
                .map_err(|e| e.to_string())?;

            println!("Generated: {}", file_path);
        }

        Ok(())
    }
}

/// Incremental Static Regeneration
pub struct ISR {
    cache: HashMap<String, (String, std::time::Instant)>,
    revalidate_after: std::time::Duration,
}

impl ISR {
    pub fn new(revalidate_seconds: u64) -> Self {
        ISR {
            cache: HashMap::new(),
            revalidate_after: std::time::Duration::from_secs(revalidate_seconds),
        }
    }

    pub async fn get_or_generate<T: SSRApp>(
        &mut self,
        app: &T,
        route: &str,
        ctx: SSRContext,
    ) -> Result<String, StatusCode> {
        // Check cache
        if let Some((html, generated_at)) = self.cache.get(route) {
            if generated_at.elapsed() < self.revalidate_after {
                return Ok(html.clone());
            }
        }

        // Generate new HTML
        let html = app.render_page(route, ctx).await?;

        // Update cache
        self.cache
            .insert(route.to_string(), (html.clone(), std::time::Instant::now()));

        Ok(html)
    }
}
