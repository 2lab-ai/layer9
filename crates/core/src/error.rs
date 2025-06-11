//! Error Boundaries - L5/L6

use crate::prelude::*;
use std::panic;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

/// Error boundary state
#[derive(Clone)]
pub struct ErrorBoundaryState {
    pub error: Option<ErrorInfo>,
    pub error_count: u32,
}

#[derive(Clone)]
pub struct ErrorInfo {
    pub message: String,
    pub stack: Option<String>,
    pub component_stack: Vec<String>,
}

/// Error boundary component
pub struct ErrorBoundary {
    children: Box<dyn Component>,
    fallback: Box<dyn Fn(&ErrorInfo) -> Element>,
    on_error: Option<Box<dyn Fn(&ErrorInfo)>>,
    state: Rc<RefCell<ErrorBoundaryState>>,
}

impl ErrorBoundary {
    pub fn new(children: impl Component + 'static) -> Self {
        ErrorBoundary {
            children: Box::new(children),
            fallback: Box::new(default_error_fallback),
            on_error: None,
            state: Rc::new(RefCell::new(ErrorBoundaryState {
                error: None,
                error_count: 0,
            })),
        }
    }
    
    pub fn fallback(mut self, fallback: impl Fn(&ErrorInfo) -> Element + 'static) -> Self {
        self.fallback = Box::new(fallback);
        self
    }
    
    pub fn on_error(mut self, handler: impl Fn(&ErrorInfo) + 'static) -> Self {
        self.on_error = Some(Box::new(handler));
        self
    }
    
    fn catch_error(&self) -> Element {
        // Set up panic hook to catch errors
        let state = self.state.clone();
        let prev_hook = panic::take_hook();
        
        panic::set_hook(Box::new(move |panic_info| {
            let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown error".to_string()
            };
            
            let location = panic_info.location()
                .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
                .unwrap_or_else(|| "Unknown location".to_string());
            
            let error_info = ErrorInfo {
                message: format!("{} at {}", message, location),
                stack: Some(format!("{:?}", panic_info)),
                component_stack: vec![], // TODO: Implement component stack tracking
            };
            
            state.borrow_mut().error = Some(error_info);
            state.borrow_mut().error_count += 1;
        }));
        
        // Try to render children
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            self.children.render()
        }));
        
        // Restore previous panic hook
        panic::set_hook(prev_hook);
        
        match result {
            Ok(element) => element,
            Err(_) => {
                // Render fallback
                if let Some(error_info) = &self.state.borrow().error {
                    if let Some(handler) = &self.on_error {
                        handler(error_info);
                    }
                    (self.fallback)(error_info)
                } else {
                    Element::Text("An error occurred".to_string())
                }
            }
        }
    }
}

impl Component for ErrorBoundary {
    fn render(&self) -> Element {
        // Check if we already have an error
        if let Some(error_info) = &self.state.borrow().error {
            return (self.fallback)(error_info);
        }
        
        // Try to render children with error catching
        self.catch_error()
    }
}

/// Default error fallback UI
fn default_error_fallback(error: &ErrorInfo) -> Element {
    view! {
        <div class="error-boundary-fallback">
            <h2>"Something went wrong"</h2>
            <details style="white-space: pre-wrap">
                <summary>"Error details"</summary>
                <p>{&error.message}</p>
                {if let Some(stack) = &error.stack {
                    view! { <pre>{stack}</pre> }
                } else {
                    view! { <div /> }
                }}
            </details>
            <button onclick="window.location.reload()">
                "Reload page"
            </button>
        </div>
    }
}

/// Error logger service
pub struct ErrorLogger {
    endpoint: String,
    api_key: String,
}

impl ErrorLogger {
    pub fn new(endpoint: impl Into<String>, api_key: impl Into<String>) -> Self {
        ErrorLogger {
            endpoint: endpoint.into(),
            api_key: api_key.into(),
        }
    }
    
    pub async fn log_error(&self, error: &ErrorInfo) -> Result<(), FetchError> {
        let payload = serde_json::json!({
            "message": error.message,
            "stack": error.stack,
            "component_stack": error.component_stack,
            "timestamp": js_sys::Date::now(),
            "user_agent": window().unwrap().navigator().user_agent().unwrap(),
            "url": window().unwrap().location().href().unwrap(),
        });
        
        FetchBuilder::new(&self.endpoint)
            .method(Method::POST)
            .header("X-API-Key", &self.api_key)
            .json(&payload)?
            .send()
            .await?;
        
        Ok(())
    }
}

/// Global error handler
static ERROR_LOGGER: once_cell::sync::Lazy<Option<ErrorLogger>> = once_cell::sync::Lazy::new(|| None);

pub fn init_error_logging(endpoint: impl Into<String>, api_key: impl Into<String>) {
    // This is a simplification - in real code you'd use a Mutex
    // ERROR_LOGGER = Some(ErrorLogger::new(endpoint, api_key));
}

/// Catch async errors
pub async fn catch_async<T, E>(future: impl std::future::Future<Output = Result<T, E>>) -> Result<T, String> 
where
    E: std::fmt::Display,
{
    match future.await {
        Ok(value) => Ok(value),
        Err(e) => {
            let error_msg = e.to_string();
            
            // Log error if logger is configured
            if let Some(logger) = ERROR_LOGGER.as_ref() {
                let error_info = ErrorInfo {
                    message: error_msg.clone(),
                    stack: None,
                    component_stack: vec![],
                };
                
                let _ = logger.log_error(&error_info).await;
            }
            
            Err(error_msg)
        }
    }
}

// Re-exports
use crate::fetch::{FetchBuilder, FetchError, Method};
use web_sys::window;