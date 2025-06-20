//! Layer 3: Framework - External interfaces
//! 
//! This layer provides user-facing APIs and handles external communication.
//! It depends on L2 and L1 but nothing depends on it.

use super::VNode;

/// Application builder - the main entry point for users
pub struct App {
    root_component: Option<Box<dyn Fn() -> VNode + Send + Sync>>,
    routes: Vec<Route>,
}

pub struct Route {
    pub path: String,
    pub component: Box<dyn Fn() -> VNode + Send + Sync>,
}

/// Runtime for the framework layer
pub struct Runtime {
    #[cfg(target_arch = "wasm32")]
    pub dom_nodes: std::cell::RefCell<std::collections::HashMap<usize, web_sys::Node>>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        Runtime {
            #[cfg(target_arch = "wasm32")]
            dom_nodes: std::cell::RefCell::new(std::collections::HashMap::new()),
        }
    }
    
    /// Apply DOM operations
    pub fn apply_dom_ops(&self, _ops: Vec<super::contracts::DomOp>) {
        // TODO: Implement DOM operations
    }
}

/// Scheduler for effects
pub struct Scheduler {
    // TODO: Implement scheduler
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {}
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Create a new Layer9 application
    pub fn new() -> Self {
        App {
            root_component: None,
            routes: Vec::new(),
        }
    }
    
    /// Set the root component
    pub fn component<F>(mut self, component: F) -> Self 
    where
        F: Fn() -> VNode + Send + Sync + 'static
    {
        self.root_component = Some(Box::new(component));
        self
    }
    
    /// Add a route
    pub fn route<F>(mut self, path: &str, component: F) -> Self
    where
        F: Fn() -> VNode + Send + Sync + 'static
    {
        self.routes.push(Route {
            path: path.to_string(),
            component: Box::new(component),
        });
        self
    }
    
    /// Mount the application to a DOM element (browser)
    #[cfg(target_arch = "wasm32")]
    pub fn mount(self, selector: &str) {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let root_element = document.query_selector(selector).unwrap().unwrap();
        
        // Clear existing content
        root_element.set_inner_html("");
        
        // Render root component
        if let Some(root_fn) = self.root_component {
            let vnode = root_fn();
            let _runtime = Runtime::new();
            // For now, skip the contract translation in WASM
            // TODO: Implement proper DOM operations
            let _vnode = vnode;
            // _runtime.apply_dom_ops(ops);
            
            // TODO: Implement proper DOM mounting
            // For now, just create a simple text node
            let text_node = document.create_text_node("Layer9 App");
            root_element.append_child(&text_node).unwrap();
        }
    }
    
    /// Run as server (SSR)
    #[cfg(feature = "ssr")]
    pub async fn serve(self, addr: &str) {
        use axum::{Router, routing::get};
        
        let app_state = AppState {
            app: std::sync::Arc::new(self),
        };
        
        let router = Router::new()
            .route("/*path", get(handle_request))
            .with_state(app_state);
        
        println!("Server running at http://{}", addr);
        
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, router).await.unwrap();
    }
}

/// HTTP server state
#[cfg(feature = "ssr")]
#[derive(Clone)]
struct AppState {
    app: std::sync::Arc<App>,
}

/// Handle HTTP requests
#[cfg(feature = "ssr")]
async fn handle_request(
    axum::extract::State(state): axum::extract::State<AppState>,
    axum::extract::Path(path): axum::extract::Path<String>,
) -> crate::haf::HttpResponse {
    // Find matching route
    for route in &state.app.routes {
        if route.path == path || route.path == format!("/{}", path) {
            let vnode = (route.component)();
            
            // Render to HTML
            let html = format!("<div>{:?}</div>", vnode); // TODO: Implement proper SSR
            
            return crate::haf::HttpResponse {
                status: 200,
                headers: vec![("Content-Type".to_string(), "text/html".to_string())],
                body: format!(
                    "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><script type=\"module\" src=\"/app.js\"></script></head><body><div id=\"app\">{}</div></body></html>",
                    html
                ),
            };
        }
    }
    
    // 404
    crate::haf::HttpResponse {
        status: 404,
        headers: vec![("Content-Type".to_string(), "text/html".to_string())],
        body: "Not Found".to_string(),
    }
}

/// Hook APIs for components
pub mod hooks {
    use super::*;
    use std::rc::Rc;
    
    /// State hook
    pub fn use_state<T: Clone + 'static>(initial: T) -> (T, Rc<dyn Fn(T)>) {
        // Get current component context
        // In real implementation, this would use thread-local storage
        let signal_id = 0; // TODO: Implement proper signal ID allocation
        let runtime = Rc::new(Runtime::new());
        
        let value = initial.clone();
        
        let setter = {
            let _runtime = runtime.clone();
            Rc::new(move |_new_value: T| {
                // Update signal in runtime
                // This would trigger re-render
                println!("Updating state to: {:?}", signal_id);
            }) as Rc<dyn Fn(T)>
        };
        
        (value, setter)
    }
    
    /// Effect hook
    pub fn use_effect<F, D>(_dependencies: D, _effect: F)
    where
        F: Fn() + 'static,
        D: PartialEq + 'static,
    {
        // Register effect with runtime
        println!("Registering effect");
    }
    
    /// Memo hook
    pub fn use_memo<T, F, D>(_dependencies: D, compute: F) -> T
    where
        T: Clone + 'static,
        F: Fn() -> T,
        D: PartialEq + 'static,
    {
        compute()
    }
}

/// Router API
pub mod router {
    /// Navigate to a route
    pub fn navigate(path: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().unwrap();
            let history = window.history().unwrap();
            history.push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(path)).unwrap();
            
            // Trigger re-render
            // In real implementation, would notify router
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            println!("Navigate to: {}", path);
        }
    }
    
    /// Get current route
    pub fn use_route() -> String {
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().unwrap();
            let location = window.location();
            location.pathname().unwrap()
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            "/".to_string()
        }
    }
    
    /// Get route parameters
    pub fn use_params() -> std::collections::HashMap<String, String> {
        std::collections::HashMap::new()
    }
}

/// HTTP client API
pub mod http {
    
    #[derive(Clone)]
    pub struct Request {
        pub method: String,
        pub url: String,
        pub headers: Vec<(String, String)>,
        pub body: Option<String>,
    }
    
    /// Fetch data from URL
    pub async fn fetch(_url: &str) -> Result<String, String> {
        #[cfg(target_arch = "wasm32")]
        {
            // Use browser fetch API
            // Implementation would use wasm-bindgen-futures
            Ok("Mock response".to_string())
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Use reqwest or similar
            Ok("Mock response".to_string())
        }
    }
    
    /// POST JSON data
    pub async fn post_json<T: serde::Serialize>(url: &str, data: &T) -> Result<String, String> {
        let _body = serde_json::to_string(data).map_err(|e| e.to_string())?;
        
        // Make request
        fetch(url).await
    }
}

/// WebSocket API
pub mod websocket {
    
    pub struct WebSocket {
        #[allow(dead_code)]
        url: String,
        #[cfg(target_arch = "wasm32")]
        ws: Option<web_sys::WebSocket>,
    }
    
    impl WebSocket {
        pub fn connect(url: &str) -> Result<Self, String> {
            #[cfg(target_arch = "wasm32")]
            {
                let ws = web_sys::WebSocket::new(url).map_err(|_| "Failed to create WebSocket")?;
                Ok(WebSocket {
                    url: url.to_string(),
                    ws: Some(ws),
                })
            }
            
            #[cfg(not(target_arch = "wasm32"))]
            {
                Ok(WebSocket {
                    url: url.to_string(),
                })
            }
        }
        
        pub fn send(&self, message: &str) -> Result<(), String> {
            #[cfg(target_arch = "wasm32")]
            {
                if let Some(ws) = &self.ws {
                    ws.send_with_str(message).map_err(|_| "Failed to send")?;
                }
            }
            
            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = message;
            }
            
            Ok(())
        }
        
        pub fn on_message<F>(&self, _handler: F)
        where
            F: Fn(String) + 'static
        {
            #[cfg(target_arch = "wasm32")]
            {
                // Set up message handler
                // Implementation would use Closure
            }
        }
    }
}

/// Development tools
pub mod devtools {
    /// Enable React DevTools integration
    pub fn enable_devtools() {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::log_1(&"Layer9 DevTools enabled".into());
        }
    }
    
    /// Log component tree
    pub fn log_component_tree() {
        println!("Component tree logging");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_app_builder() {
        let app = App::new()
            .route("/", || VNode::Text("Home".to_string()))
            .route("/about", || VNode::Text("About".to_string()));
        
        assert_eq!(app.routes.len(), 2);
    }
    
    #[test]
    fn test_hooks() {
        let (value, setter) = hooks::use_state(42);
        assert_eq!(value, 42);
        
        // In real implementation, setter would trigger re-render
        setter(100);
    }
}