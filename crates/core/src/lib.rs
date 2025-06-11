//! Layer9 Core - Hierarchical Web Framework
//! 
//! L9 Philosophy: Consciousness-aware web development
//! L8 Architecture: Enforced layer separation
//! L7 Application: Business logic isolation
//! L6 Features: Modular feature system
//! L5 Components: Type-safe reactive components
//! L4 Services: Server/Client boundary
//! L3 Runtime: Layer9SM/JS execution
//! L2 Platform: Next.js compatibility
//! L1 Infrastructure: Build and deploy

pub mod layers;
pub mod component;
pub mod router;
pub mod router_v2;
pub mod server;
pub mod vdom;
pub mod app;
pub mod auth;
pub mod styles;
pub mod ui;
pub mod ssr;
pub mod fetch;
pub mod state;
pub mod error;
pub mod form;
pub mod upload;
pub mod websocket;
pub mod middleware;
pub mod env;
pub mod image;
pub mod test;
pub mod db;
pub mod i18n;
pub mod cache;
pub mod security;
pub mod monitoring;
pub mod api_docs;

pub mod prelude {
    pub use crate::layers::*;
    pub use crate::component::{Component, Element, view, Props};
    pub use crate::router::{Page, Route, RouteHandler};
    pub use crate::router_v2::{init_router, use_router, use_route, Link, navigate, route};
    pub use crate::server::{ServerFunction, Response};
    pub use crate::app::{Layer9App, run_app};
    pub use crate::auth::{AuthService, use_auth, Protected};
    pub use crate::styles::{style, StyleBuilder, inject_global_styles};
    pub use crate::ui::*;
    pub use crate::ssr::{SSRApp, SSRComponent, create_ssr_server, hydrate_app, SSG};
    pub use crate::fetch::{FetchBuilder, get, post, SWR, Method};
    pub use crate::state::{create_atom, use_atom, use_selector, create_app_store, AppState, AppAction};
    pub use crate::error::{ErrorBoundary, use_error_handler};
    pub use crate::form::{use_form, Form, FormConfig};
    pub use crate::upload::{use_upload, UploadConfig};
    pub use crate::websocket::{use_websocket, WsMessage, WsState};
    pub use crate::middleware::{MiddlewareStack, Middleware, Context};
    pub use crate::env::{env, env_or, is_production, is_development};
    pub use crate::image::{Image, Picture};
    pub use crate::test::{TestContext, TestResult, TestUtils};
    pub use crate::db::{use_db, use_repository, Model, QueryBuilder};
    pub use crate::i18n::{use_i18n, t, plural, Locale};
    pub use crate::cache::{use_cache, use_http_cache, InvalidationStrategy};
    pub use crate::security::{use_security, use_csrf_token, XssProtection};
    pub use crate::monitoring::{use_metrics, use_performance, use_analytics};
    pub use crate::api_docs::{OpenApiBuilder, ApiDoc, SchemaBuilder};
    pub use layer9_macro::{layer9_app, page, component, server};
}

// Layer validation at compile time
#[macro_export]
macro_rules! enforce_layer {
    ($from:expr, $to:expr) => {
        const _: () = {
            assert!($from >= $to, "Invalid layer access: higher layers cannot access lower layers directly");
        };
    };
}