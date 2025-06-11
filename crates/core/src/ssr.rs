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

/// SSR App trait
pub trait SSRApp: Layer9App {
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
        import init from '/layer9_bundle.js';
        init().then(() => {
            window.__Layer9_HYDRATE__();
        });
    </script>
</body>
</html>"#
    }

    /// Render page on server
    async fn render_page(&self, route: &str, ctx: SSRContext) -> Result<String, StatusCode> {
        // Find matching route
        let routes = self.routes();
        let route_match = routes.iter().find(|r| r.path == route);

        if let Some(route) = route_match {
            match &route.handler {
                RouteHandler::Page(page_fn) => {
                    let page = page_fn();

                    // Render component to HTML
                    let content = page.component.render_to_string();

                    // Get server props
                    let props = serde_json::json!({
                        "url": ctx.url,
                        "params": ctx.params,
                        "query": ctx.query,
                    });

                    // Build final HTML
                    let html = self
                        .html_template()
                        .replace("{title}", &page.title)
                        .replace("{styles}", &get_critical_css())
                        .replace("{props}", &props.to_string())
                        .replace("{content}", &content);

                    Ok(html)
                }
                _ => Err(StatusCode::NOT_FOUND),
            }
        } else {
            Err(StatusCode::NOT_FOUND)
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
    let app = std::sync::Arc::new(app);

    Router::new()
        // Catch-all route for SSR
        .route(
            "/*path",
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
        // Static files
        .route(
            "/layer9_bundle.js",
            get(|| async {
                // In production, serve the actual Layer9SM bundle
                "// Layer9 bundle placeholder"
            }),
        )
}

/// Hydration helper for client side
#[wasm_bindgen]
pub fn hydrate_app(app: impl Layer9App + 'static) {
    // Get props from server
    let window = web_sys::window().unwrap();
    let props = js_sys::Reflect::get(&window, &"__Layer9_PROPS__".into())
        .ok()
        .and_then(|v| serde_wasm_bindgen::from_value(v).ok());

    // Initialize app with server props
    run_app(app);
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
