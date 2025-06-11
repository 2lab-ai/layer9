//! Middleware System - L3/L4

use crate::prelude::*;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use async_trait::async_trait;

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
        let json = serde_json::to_string(data)
            .map_err(|e| e.to_string())?;
        self.headers.insert("Content-Type".to_string(), "application/json".to_string());
        self.body = Some(json);
        Ok(self)
    }
}

/// State container for middleware
pub type State = HashMap<String, Box<dyn std::any::Any>>;

/// Next function type
pub type Next = Box<dyn Fn() -> Pin<Box<dyn Future<Output = Result<Response, MiddlewareError>>>>>;

/// Middleware trait
#[async_trait(?Send)]
pub trait Middleware: 'static {
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
        MiddlewareError { status: 500, message }
    }
}

/// Middleware stack
pub struct MiddlewareStack {
    middlewares: Vec<Box<dyn Middleware>>,
}

impl MiddlewareStack {
    pub fn new() -> Self {
        MiddlewareStack {
            middlewares: vec![],
        }
    }
    
    pub fn use_middleware(mut self, middleware: impl Middleware) -> Self {
        self.middlewares.push(Box::new(middleware));
        self
    }
    
    pub async fn run(&self, mut ctx: Context) -> Result<Response, MiddlewareError> {
        self.run_middleware(0, &mut ctx).await
    }
    
    fn run_middleware<'a>(
        &'a self,
        index: usize,
        ctx: &'a mut Context,
    ) -> Pin<Box<dyn Future<Output = Result<Response, MiddlewareError>> + 'a>> {
        Box::pin(async move {
            if index >= self.middlewares.len() {
                // No more middleware, return response
                Ok(ctx.response.clone())
            } else {
                let middleware = &self.middlewares[index];
                let next: Next = Box::new(move || {
                    Box::pin(self.run_middleware(index + 1, ctx))
                });
                
                middleware.handle(ctx, next).await
            }
        })
    }
}

/// Common middleware implementations

/// Authentication middleware
pub struct AuthMiddleware {
    verify_token: Box<dyn Fn(&str) -> Option<User>>,
}

impl AuthMiddleware {
    pub fn new(verify_token: impl Fn(&str) -> Option<User> + 'static) -> Self {
        AuthMiddleware {
            verify_token: Box::new(verify_token),
        }
    }
}

#[async_trait(?Send)]
impl Middleware for AuthMiddleware {
    async fn handle(&self, ctx: &mut Context, next: Next) -> Result<Response, MiddlewareError> {
        // Check for auth header
        if let Some(auth_header) = ctx.request.headers.get("Authorization") {
            if let Some(token) = auth_header.strip_prefix("Bearer ") {
                if let Some(user) = (self.verify_token)(token) {
                    ctx.request.user = Some(user);
                }
            }
        }
        
        // Continue to next middleware
        next().await
    }
}

/// CORS middleware
pub struct CorsMiddleware {
    allowed_origins: Vec<String>,
    allowed_methods: Vec<String>,
    allowed_headers: Vec<String>,
}

impl CorsMiddleware {
    pub fn new() -> Self {
        CorsMiddleware {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            allowed_headers: vec!["Content-Type", "Authorization"]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
    
    pub fn allow_origin(mut self, origin: impl Into<String>) -> Self {
        self.allowed_origins.push(origin.into());
        self
    }
}

#[async_trait(?Send)]
impl Middleware for CorsMiddleware {
    async fn handle(&self, ctx: &mut Context, next: Next) -> Result<Response, MiddlewareError> {
        // Handle preflight
        if ctx.request.method == Method::OPTIONS {
            return Ok(Response::new()
                .with_status(204)
                .with_header("Access-Control-Allow-Origin", "*")
                .with_header("Access-Control-Allow-Methods", self.allowed_methods.join(", "))
                .with_header("Access-Control-Allow-Headers", self.allowed_headers.join(", ")));
        }
        
        // Add CORS headers to response
        let mut response = next().await?;
        response.headers.insert(
            "Access-Control-Allow-Origin".to_string(),
            "*".to_string(),
        );
        
        Ok(response)
    }
}

/// Rate limiting middleware
pub struct RateLimitMiddleware {
    store: Rc<RefCell<HashMap<String, RateLimitEntry>>>,
    max_requests: u32,
    window_ms: u64,
}

struct RateLimitEntry {
    count: u32,
    reset_at: u64,
}

impl RateLimitMiddleware {
    pub fn new(max_requests: u32, window_ms: u64) -> Self {
        RateLimitMiddleware {
            store: Rc::new(RefCell::new(HashMap::new())),
            max_requests,
            window_ms,
        }
    }
}

#[async_trait(?Send)]
impl Middleware for RateLimitMiddleware {
    async fn handle(&self, ctx: &mut Context, next: Next) -> Result<Response, MiddlewareError> {
        let now = js_sys::Date::now() as u64;
        let client_id = ctx.request.headers.get("X-Client-Id")
            .or_else(|| ctx.request.user.as_ref().map(|u| &u.id))
            .cloned()
            .unwrap_or_else(|| "anonymous".to_string());
        
        let mut store = self.store.borrow_mut();
        let entry = store.entry(client_id.clone()).or_insert(RateLimitEntry {
            count: 0,
            reset_at: now + self.window_ms,
        });
        
        // Reset if window expired
        if now > entry.reset_at {
            entry.count = 0;
            entry.reset_at = now + self.window_ms;
        }
        
        // Check rate limit
        if entry.count >= self.max_requests {
            return Err(MiddlewareError {
                status: 429,
                message: "Too many requests".to_string(),
            });
        }
        
        entry.count += 1;
        drop(store);
        
        // Add rate limit headers
        let mut response = next().await?;
        response.headers.insert("X-RateLimit-Limit".to_string(), self.max_requests.to_string());
        response.headers.insert("X-RateLimit-Remaining".to_string(), 
            (self.max_requests - entry.count).to_string());
        
        Ok(response)
    }
}

/// Logging middleware
pub struct LoggingMiddleware {
    logger: Box<dyn Fn(LogEntry)>,
}

pub struct LogEntry {
    pub method: String,
    pub url: String,
    pub status: u16,
    pub duration_ms: f64,
    pub user_id: Option<String>,
}

impl LoggingMiddleware {
    pub fn new(logger: impl Fn(LogEntry) + 'static) -> Self {
        LoggingMiddleware {
            logger: Box::new(logger),
        }
    }
}

#[async_trait(?Send)]
impl Middleware for LoggingMiddleware {
    async fn handle(&self, ctx: &mut Context, next: Next) -> Result<Response, MiddlewareError> {
        let start = js_sys::Date::now();
        
        let response = next().await?;
        
        let duration_ms = js_sys::Date::now() - start;
        
        (self.logger)(LogEntry {
            method: format!("{:?}", ctx.request.method),
            url: ctx.request.url.clone(),
            status: response.status,
            duration_ms,
            user_id: ctx.request.user.as_ref().map(|u| u.id.clone()),
        });
        
        Ok(response)
    }
}

/// Compression middleware
pub struct CompressionMiddleware;

#[async_trait(?Send)]
impl Middleware for CompressionMiddleware {
    async fn handle(&self, ctx: &mut Context, next: Next) -> Result<Response, MiddlewareError> {
        let mut response = next().await?;
        
        // Check if client accepts gzip
        if let Some(accept_encoding) = ctx.request.headers.get("Accept-Encoding") {
            if accept_encoding.contains("gzip") && response.body.is_some() {
                // In real implementation, compress the body
                response.headers.insert("Content-Encoding".to_string(), "gzip".to_string());
            }
        }
        
        Ok(response)
    }
}

/// Security headers middleware
pub struct SecurityMiddleware;

#[async_trait(?Send)]
impl Middleware for SecurityMiddleware {
    async fn handle(&self, ctx: &mut Context, next: Next) -> Result<Response, MiddlewareError> {
        let mut response = next().await?;
        
        // Add security headers
        response.headers.insert("X-Content-Type-Options".to_string(), "nosniff".to_string());
        response.headers.insert("X-Frame-Options".to_string(), "DENY".to_string());
        response.headers.insert("X-XSS-Protection".to_string(), "1; mode=block".to_string());
        response.headers.insert("Referrer-Policy".to_string(), "strict-origin-when-cross-origin".to_string());
        response.headers.insert(
            "Content-Security-Policy".to_string(),
            "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'".to_string()
        );
        
        Ok(response)
    }
}

// Re-exports
use std::collections::HashMap;
use serde::Serialize;