//! Server-Side Rendering support for Layer9

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json;
use async_trait::async_trait;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use axum::{Router, routing::get, response::Html, extract::Query};

#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

// Always need Arc for SSRRoute
use std::sync::Arc as ArcAlways;

/// SSR Context for rendering components server-side
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSRContext {
    pub props: HashMap<String, String>,
    pub initial_state: Option<String>,
    pub meta_tags: Vec<String>,
    pub route: String,
    pub query_params: HashMap<String, String>,
    #[serde(skip)]
    pub request_headers: HashMap<String, String>,
}

impl SSRContext {
    pub fn new() -> Self {
        Self {
            props: HashMap::new(),
            initial_state: None,
            meta_tags: Vec::new(),
            route: "/".to_string(),
            query_params: HashMap::new(),
            request_headers: HashMap::new(),
        }
    }
    
    pub fn with_props(mut self, props: HashMap<String, String>) -> Self {
        self.props = props;
        self
    }
    
    pub fn with_state(mut self, state: String) -> Self {
        self.initial_state = Some(state);
        self
    }
    
    pub fn add_meta_tag(mut self, tag: String) -> Self {
        self.meta_tags.push(tag);
        self
    }
    
    pub fn with_route(mut self, route: String) -> Self {
        self.route = route;
        self
    }
    
    pub fn with_query_params(mut self, params: HashMap<String, String>) -> Self {
        self.query_params = params;
        self
    }
    
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.request_headers = headers;
        self
    }
}

/// Trait for SSR-capable components
#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
pub trait SSRComponent: Send + Sync {
    fn render_to_string(&self, ctx: &SSRContext) -> String;
    
    async fn get_server_props(&self, _ctx: &SSRContext) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({}))
    }
    
    fn get_data_requirements(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Trait for SSR-capable components (server-side)
#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
pub trait SSRComponent: Send + Sync {
    fn render_to_string(&self, ctx: &SSRContext) -> String;
    
    async fn get_server_props(&self, _ctx: &SSRContext) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({}))
    }
    
    fn get_data_requirements(&self) -> Vec<String> {
        Vec::new()
    }
}

/// SSR Renderer
pub struct SSRRenderer {
    components: Vec<Box<dyn SSRComponent>>,
    template: String,
    enable_hydration: bool,
}

impl SSRRenderer {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            template: Self::default_template(),
            enable_hydration: true,
        }
    }
    
    pub fn with_template(mut self, template: String) -> Self {
        self.template = template;
        self
    }
    
    pub fn add_component(&mut self, component: Box<dyn SSRComponent>) {
        self.components.push(component);
    }
    
    pub fn enable_hydration(mut self, enable: bool) -> Self {
        self.enable_hydration = enable;
        self
    }
    
    pub async fn render(&self, ctx: &SSRContext) -> String {
        let mut body = String::new();
        
        for component in &self.components {
            body.push_str(&component.render_to_string(ctx));
        }
        
        let mut html = self.template.clone();
        html = html.replace("{{content}}", &body);
        
        // Add meta tags
        let meta_tags = ctx.meta_tags.join("\n    ");
        html = html.replace("{{meta}}", &meta_tags);
        
        // Add initial state and hydration data
        let mut state_scripts = Vec::new();
        
        if let Some(state) = &ctx.initial_state {
            state_scripts.push(format!(
                r#"<script>window.__INITIAL_STATE__ = {};</script>"#,
                state
            ));
        }
        
        // Add SSR context for hydration
        if self.enable_hydration {
            let ssr_context = serde_json::to_string(&ctx).unwrap_or_default();
            state_scripts.push(format!(
                r#"<script>window.__SSR_CONTEXT__ = {};</script>"#,
                ssr_context
            ));
        }
        
        html = html.replace("{{state}}", &state_scripts.join("\n    "));
        
        // Add hydration script only if hydration is enabled
        if self.enable_hydration {
            html = html.replace("{{hydration_script}}", r#"
    <script type="module">
        import init, { hydrate_app } from '/pkg/layer9_app.js';
        init().then(() => {
            if (window.__SSR_CONTEXT__) {
                hydrate_app();
            }
        });
    </script>"#);
        } else {
            html = html.replace("{{hydration_script}}", "");
        }
        
        html
    }
    
    fn default_template() -> String {
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Layer9 SSR</title>
    {{meta}}
</head>
<body>
    <div id="app">{{content}}</div>
    {{state}}
    {{hydration_script}}
</body>
</html>"#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ssr_context() {
        let ctx = SSRContext::new()
            .with_state(r#"{"counter": 0}"#.to_string())
            .add_meta_tag(r#"<meta name="description" content="Test">"#.to_string());
        
        assert!(ctx.initial_state.is_some());
        assert_eq!(ctx.meta_tags.len(), 1);
    }
}

/// SSR-enabled application trait
#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
pub trait SSRApp: Send + Sync + 'static {
    /// Get routes for the application
    fn routes(&self) -> Vec<SSRRoute>;
    
    /// Get the HTML template for the application
    fn html_template(&self) -> &'static str {
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Layer9 SSR</title>
    {{meta}}
</head>
<body>
    <div id="app">{{content}}</div>
    {{state}}
    {{hydration_script}}
</body>
</html>"#
    }
    
    /// Handle 404 errors
    async fn handle_not_found(&self, _ctx: &SSRContext) -> String {
        "<h1>404 - Page Not Found</h1>".to_string()
    }
}

/// SSR Route definition
#[derive(Clone)]
pub struct SSRRoute {
    pub path: String,
    pub handler: ArcAlways<dyn SSRRouteHandler>,
}

/// SSR Route handler trait
#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
pub trait SSRRouteHandler: Send + Sync {
    async fn handle(&self, ctx: SSRContext) -> Result<String, String>;
}

/// Create an SSR server for the given app
#[cfg(not(target_arch = "wasm32"))]
pub fn create_ssr_server<A: SSRApp>(app: Arc<A>) -> Router {
    let mut router = Router::new();
    
    // Add routes from the app
    for route in app.routes() {
        let app_clone = Arc::clone(&app);
        let handler = route.handler.clone();
        let path = route.path.clone();
        let path_clone = path.clone();
        
        router = router.route(&path, get(move |
            Query(params): Query<HashMap<String, String>>,
            headers: axum::http::HeaderMap,
        | {
            let _app = Arc::clone(&app_clone);
            let handler = handler.clone();
            
            async move {
                // Build SSR context
                let mut ctx = SSRContext::new()
                    .with_route(path_clone)
                    .with_query_params(params);
                
                // Add headers to context
                let mut req_headers = HashMap::new();
                for (key, value) in headers.iter() {
                    if let Ok(v) = value.to_str() {
                        req_headers.insert(key.to_string(), v.to_string());
                    }
                }
                ctx = ctx.with_headers(req_headers);
                
                // Handle the route
                match handler.handle(ctx).await {
                    Ok(html) => Html(html),
                    Err(e) => Html(format!("<h1>Error</h1><p>{}</p>", e)),
                }
            }
        }));
    }
    
    // Add static file serving for WASM app
    router = router.route("/pkg/*path", get(serve_wasm_files));
    
    // Add database API endpoints if configured
    #[cfg(feature = "ssr")]
    {
        // Create a simple database connection for the API
        if let Ok(conn) = crate::db_sqlx::get_db_connection() {
            router = router.nest("/api/db", crate::db_api::create_db_api_router(conn));
        }
    }
    
    router
}

/// Serve WASM files
#[cfg(not(target_arch = "wasm32"))]
async fn serve_wasm_files(
    axum::extract::Path(path): axum::extract::Path<String>
) -> impl axum::response::IntoResponse {
    use axum::http::StatusCode;
    
    // In production, these would be served from a CDN or static file server
    // For development, we'll return a placeholder
    (
        StatusCode::OK,
        [("content-type", "application/javascript")],
        format!("// Placeholder for WASM file: {}", path)
    )
}

/// Hydration support for client-side
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn hydrate_app() {
    // Get SSR context from window
    let window = web_sys::window().expect("no global window");
    let ssr_context = js_sys::Reflect::get(&window, &"__SSR_CONTEXT__".into())
        .ok()
        .and_then(|v| v.as_string())
        .and_then(|s| serde_json::from_str::<SSRContext>(&s).ok());
    
    if let Some(ctx) = ssr_context {
        // Initialize router with current route
        if let Err(e) = crate::router_v2::navigate(&ctx.route) {
            web_sys::console::error_1(&format!("Failed to hydrate route: {}", e).into());
        }
        
        // Mount the app with hydration mode
        crate::reactive_v2::init_renderer();
        
        web_sys::console::log_1(&"SSR hydration complete".into());
    } else {
        web_sys::console::warn_1(&"No SSR context found, starting fresh".into());
    }
}

/// Data fetching hook for SSR
pub struct SSRData<T> {
    pub data: Option<T>,
    pub error: Option<String>,
    pub is_loading: bool,
}

impl<T> SSRData<T> {
    pub fn loading() -> Self {
        Self {
            data: None,
            error: None,
            is_loading: true,
        }
    }
    
    pub fn success(data: T) -> Self {
        Self {
            data: Some(data),
            error: None,
            is_loading: false,
        }
    }
    
    pub fn error(error: String) -> Self {
        Self {
            data: None,
            error: Some(error),
            is_loading: false,
        }
    }
}

/// Use SSR data hook
#[cfg(target_arch = "wasm32")]
pub fn use_ssr_data<T: serde::de::DeserializeOwned + 'static>(key: &str) -> SSRData<T> {
    // Check if we have SSR data in window
    let window = web_sys::window().expect("no global window");
    
    if let Ok(initial_state) = js_sys::Reflect::get(&window, &"__INITIAL_STATE__".into()) {
        if let Some(state_str) = initial_state.as_string() {
            if let Ok(state_obj) = serde_json::from_str::<serde_json::Value>(&state_str) {
                if let Some(data) = state_obj.get(key) {
                    if let Ok(typed_data) = serde_json::from_value::<T>(data.clone()) {
                        return SSRData::success(typed_data);
                    }
                }
            }
        }
    }
    
    SSRData::loading()
}

/// Example SSR component implementation
pub struct ExampleSSRComponent {
    pub title: String,
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl SSRComponent for ExampleSSRComponent {
    fn render_to_string(&self, ctx: &SSRContext) -> String {
        format!(
            r#"<div class="example">
                <h1>{}</h1>
                <p>Route: {}</p>
                <p>Query params: {:?}</p>
            </div>"#,
            self.title, ctx.route, ctx.query_params
        )
    }
    
    async fn get_server_props(&self, _ctx: &SSRContext) -> Result<serde_json::Value, String> {
        // Fetch data from database or API
        // Fetch data from database or API
        Ok(serde_json::json!({
            "title": self.title,
            "timestamp": "2025-01-11T12:00:00Z",
        }))
    }
}

#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
impl SSRComponent for ExampleSSRComponent {
    fn render_to_string(&self, ctx: &SSRContext) -> String {
        format!(
            r#"<div class="example">
                <h1>{}</h1>
                <p>Route: {}</p>
            </div>"#,
            self.title, ctx.route
        )
    }
}
