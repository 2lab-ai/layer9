//! Middleware System - L3/L4

use crate::auth::User;
use crate::prelude::*;
use crate::router_v2::RouteParams;
use async_trait::async_trait;
use std::cell::RefCell;
use std::rc::Rc;

// Type alias to simplify complex types
type VerifyTokenFn = Box<dyn Fn(&str) -> Option<User>>;

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

/// Middleware trait - simplified version
#[async_trait(?Send)]
pub trait Middleware: 'static {
    async fn handle(&self, ctx: &mut Context) -> Result<Response, MiddlewareError>;
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

/// Middleware stack
pub struct MiddlewareStack {
    middlewares: Vec<Box<dyn Middleware>>,
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

    pub fn use_middleware(mut self, middleware: impl Middleware) -> Self {
        self.middlewares.push(Box::new(middleware));
        self
    }

    pub async fn run(&self, mut ctx: Context) -> Result<Response, MiddlewareError> {
        // Run each middleware in sequence
        for middleware in &self.middlewares {
            let result = middleware.handle(&mut ctx).await?;
            ctx.response = result;
        }
        
        // Return the final response
        Ok(ctx.response)
    }
}

/// Wrapper for middleware that needs to call next
pub struct ChainedMiddleware<M> {
    middleware: M,
    next: Option<Box<dyn Middleware>>,
}

impl<M: Middleware> ChainedMiddleware<M> {
    pub fn new(middleware: M) -> Self {
        ChainedMiddleware {
            middleware,
            next: None,
        }
    }
    
    pub fn chain(mut self, next: impl Middleware) -> Self {
        self.next = Some(Box::new(next));
        self
    }
}

#[async_trait(?Send)]
impl<M: Middleware> Middleware for ChainedMiddleware<M> {
    async fn handle(&self, ctx: &mut Context) -> Result<Response, MiddlewareError> {
        // Run this middleware
        let result = self.middleware.handle(ctx).await?;
        ctx.response = result;
        
        // Run next middleware if exists
        if let Some(next) = &self.next {
            next.handle(ctx).await
        } else {
            Ok(ctx.response.clone())
        }
    }
}

/// Authentication middleware
pub struct AuthMiddleware {
    verify_token: VerifyTokenFn,
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
    async fn handle(&self, ctx: &mut Context) -> Result<Response, MiddlewareError> {
        // Check for auth header
        if let Some(auth_header) = ctx.request.headers.get("Authorization") {
            if let Some(token) = auth_header.strip_prefix("Bearer ") {
                if let Some(user) = (self.verify_token)(token) {
                    ctx.request.user = Some(user);
                }
            }
        }

        // Return the existing response (middleware should be chained properly)
        Ok(ctx.response.clone())
    }
}

/// CORS middleware
pub struct CorsMiddleware {
    allowed_origins: Vec<String>,
    allowed_methods: Vec<String>,
    allowed_headers: Vec<String>,
}

impl Default for CorsMiddleware {
    fn default() -> Self {
        Self::new()
    }
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
    async fn handle(&self, ctx: &mut Context) -> Result<Response, MiddlewareError> {
        // Handle preflight
        if ctx.request.method == Method::OPTIONS {
            return Ok(Response::new()
                .with_status(204)
                .with_header("Access-Control-Allow-Origin", "*")
                .with_header(
                    "Access-Control-Allow-Methods",
                    self.allowed_methods.join(", "),
                )
                .with_header(
                    "Access-Control-Allow-Headers",
                    self.allowed_headers.join(", "),
                ));
        }

        // Add CORS headers to response
        ctx.response
            .headers
            .insert("Access-Control-Allow-Origin".to_string(), "*".to_string());

        Ok(ctx.response.clone())
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
    async fn handle(&self, ctx: &mut Context) -> Result<Response, MiddlewareError> {
        let now = js_sys::Date::now() as u64;
        let client_id = ctx
            .request
            .headers
            .get("X-Client-Id")
            .or_else(|| ctx.request.user.as_ref().map(|u| &u.id))
            .cloned()
            .unwrap_or_else(|| "anonymous".to_string());

        let current_count = {
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
            entry.count
        };

        // Add rate limit headers
        ctx.response.headers.insert(
            "X-RateLimit-Limit".to_string(),
            self.max_requests.to_string(),
        );
        ctx.response.headers.insert(
            "X-RateLimit-Remaining".to_string(),
            (self.max_requests - current_count).to_string(),
        );

        Ok(ctx.response.clone())
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
    async fn handle(&self, ctx: &mut Context) -> Result<Response, MiddlewareError> {
        let start = js_sys::Date::now();

        // Log after processing (in real chain, this would be after next())
        let duration_ms = js_sys::Date::now() - start;

        (self.logger)(LogEntry {
            method: format!("{:?}", ctx.request.method),
            url: ctx.request.url.clone(),
            status: ctx.response.status,
            duration_ms,
            user_id: ctx.request.user.as_ref().map(|u| u.id.clone()),
        });

        Ok(ctx.response.clone())
    }
}

/// Compression middleware
pub struct CompressionMiddleware;

#[async_trait(?Send)]
impl Middleware for CompressionMiddleware {
    async fn handle(&self, ctx: &mut Context) -> Result<Response, MiddlewareError> {
        // Check if client accepts gzip
        if let Some(accept_encoding) = ctx.request.headers.get("Accept-Encoding") {
            if accept_encoding.contains("gzip") && ctx.response.body.is_some() {
                // In real implementation, compress the body
                ctx.response
                    .headers
                    .insert("Content-Encoding".to_string(), "gzip".to_string());
            }
        }

        Ok(ctx.response.clone())
    }
}

/// Security headers middleware
pub struct SecurityMiddleware;

#[async_trait(?Send)]
impl Middleware for SecurityMiddleware {
    async fn handle(&self, ctx: &mut Context) -> Result<Response, MiddlewareError> {
        // Add security headers
        ctx.response
            .headers
            .insert("X-Content-Type-Options".to_string(), "nosniff".to_string());
        ctx.response
            .headers
            .insert("X-Frame-Options".to_string(), "DENY".to_string());
        ctx.response
            .headers
            .insert("X-XSS-Protection".to_string(), "1; mode=block".to_string());
        ctx.response.headers.insert(
            "Referrer-Policy".to_string(),
            "strict-origin-when-cross-origin".to_string(),
        );
        ctx.response.headers.insert(
            "Content-Security-Policy".to_string(),
            "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'".to_string()
        );

        Ok(ctx.response.clone())
    }
}

// Re-exports
use serde::Serialize;
use std::collections::HashMap;