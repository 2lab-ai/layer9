//! Error Boundaries - L5/L6

use crate::prelude::*;
use std::cell::RefCell;
use std::panic;
use std::rc::Rc;

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
        // Try to render children
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| self.children.render()));

        match result {
            Ok(element) => element,
            Err(err) => {
                // Extract error message
                let message = if let Some(s) = err.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = err.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown error".to_string()
                };

                let error_info = ErrorInfo {
                    message,
                    stack: None,
                    component_stack: vec![], // TODO: Implement component stack tracking
                };

                self.state.borrow_mut().error = Some(error_info.clone());
                self.state.borrow_mut().error_count += 1;

                if let Some(handler) = &self.on_error {
                    handler(&error_info);
                }

                (self.fallback)(&error_info)
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
    use crate::component::{Element, Props};

    let mut props = Props::default();
    props.class = Some("error-boundary-fallback".to_string());

    Element::Node {
        tag: "div".to_string(),
        props,
        children: vec![
            Element::Node {
                tag: "h2".to_string(),
                props: Props::default(),
                children: vec![Element::Text("Something went wrong".to_string())],
            },
            Element::Node {
                tag: "p".to_string(),
                props: Props::default(),
                children: vec![Element::Text(error.message.clone())],
            },
        ],
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
static ERROR_LOGGER: once_cell::sync::Lazy<Option<ErrorLogger>> =
    once_cell::sync::Lazy::new(|| None);

pub fn init_error_logging(_endpoint: impl Into<String>, _api_key: impl Into<String>) {
    // This is a simplification - in real code you'd use a Mutex
    // ERROR_LOGGER = Some(ErrorLogger::new(endpoint, api_key));
}

/// Catch async errors
pub async fn catch_async<T, E>(
    future: impl std::future::Future<Output = Result<T, E>>,
) -> Result<T, String>
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

/// Hook to handle errors in components
pub fn use_error_handler() -> impl Fn(String) {
    move |error: String| {
        web_sys::console::error_1(&format!("Error: {}", error).into());
    }
}

// Re-exports
use crate::fetch::{FetchBuilder, FetchError, Method};
use web_sys::window;
