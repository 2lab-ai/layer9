//! Middleware System V2 - Proper chaining implementation

use crate::auth::User;
use crate::prelude::*;
pub use crate::router_v2::RouteParams;
use async_trait::async_trait;
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

/// Middleware context
pub struct Context {
    pub request: Request,
    pub response: Response,
    pub state: State,
    pub params: RouteParams,
}

/// Request object
#[derive(Clone)]
pub struct Request {
    pub method: Method,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub user: Option<User>,
}

/// Response object
#[derive(Clone)]
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}

impl Response {
    pub fn new() -> Self {
        Response {
            status: 200,
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn with_status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn json<T: Serialize>(mut self, data: &T) -> Result<Self, String> {
        let json = serde_json::to_string(data).map_err(|e| e.to_string())?;
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        self.body = Some(json);
        Ok(self)
    }
}

/// State container for middleware
pub type State = HashMap<String, Box<dyn std::any::Any>>;

/// Next function type
pub type Next = Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = Result<Response, MiddlewareError>>>>>;

/// Middleware trait
#[async_trait(?Send)]
pub trait Middleware {
    async fn handle(&self, ctx: &mut Context, next: Next) -> Result<Response, MiddlewareError>;
}

/// Middleware error
#[derive(Debug, Clone)]
pub struct MiddlewareError {
    pub status: u16,
    pub message: String,
}

impl From<String> for MiddlewareError {
    fn from(message: String) -> Self {
        MiddlewareError {
            status: 500,
            message,
        }
    }
}

/// Middleware chain builder
pub struct MiddlewareStack {
    middlewares: Vec<Rc<dyn Middleware>>,
}

impl Default for MiddlewareStack {
    fn default() -> Self {
        Self::new()
    }
}

impl MiddlewareStack {
    pub fn new() -> Self {
        MiddlewareStack {
            middlewares: vec![],
        }
    }

    pub fn use_middleware(mut self, middleware: impl Middleware + 'static) -> Self {
        self.middlewares.push(Rc::new(middleware));
        self
    }

    #[allow(clippy::await_holding_refcell_ref)]
    pub async fn run(&self, ctx: Context) -> Result<Response, MiddlewareError> {
        // Create a shared context wrapped in Rc<RefCell>
        let ctx_rc = Rc::new(RefCell::new(ctx));
        
        // Create the final handler that returns the response from context
        let final_handler = {
            let ctx_clone = ctx_rc.clone();
            move || -> Pin<Box<dyn Future<Output = Result<Response, MiddlewareError>>>> {
                Box::pin(async move {
                    Ok(ctx_clone.borrow().response.clone())
                })
            }
        };
        
        // Build the middleware chain in reverse order
        let chain = self.middlewares.iter().rev().fold(
            Box::new(final_handler) as Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = Result<Response, MiddlewareError>>>>>,
            |next, middleware| {
                let middleware = middleware.clone();
                let ctx_clone = ctx_rc.clone();
                Box::new(move || {
                    Box::pin(async move {
                        let result = {
                            let mut ctx_mut = ctx_clone.borrow_mut();
                            middleware.handle(&mut ctx_mut, next).await
                        };
                        result
                    })
                })
            }
        );
        
        // Execute the chain
        chain().await
    }
}

// Re-exports
use serde::Serialize;
use std::collections::HashMap;