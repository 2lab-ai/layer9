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

pub mod api_docs;
pub mod app;
// pub mod async_component; // Using v2 instead
pub mod async_component_v2;
pub mod auth;
pub mod cache;
pub mod component;
pub mod db;
#[cfg(feature = "ssr")]
pub mod db_api;
#[cfg(feature = "ssr")]
pub mod db_sqlx;
pub mod env;
pub mod error;
pub mod fetch;
pub mod form;
pub mod form_traits;
pub mod form_builder;
pub mod hooks;
pub mod i18n;
pub mod image;
pub mod layers;
pub mod middleware;
pub mod monitoring;
// pub mod reactive; // Using v2 to fix borrowing issues
pub mod reactive_v2;
pub mod router;
pub mod router_v2;
pub mod security;
pub mod server;
#[cfg(feature = "ssr")]
pub mod ssr;
pub mod state;
pub mod styles;
pub mod test;
pub mod ui;
pub mod upload;
pub mod vdom;
pub mod websocket;

pub mod prelude {
    pub use crate::api_docs::{ApiDoc, OpenApiBuilder, SchemaBuilder};
    pub use crate::app::{run_app, Layer9App};
    pub use crate::async_component_v2::{
        use_async_data, with_error_boundary, 
        AsyncData, AsyncState, Suspense
    };
    pub use crate::auth::{use_auth, AuthService, Protected};
    pub use crate::cache::{use_cache, use_http_cache, InvalidationStrategy};
    pub use crate::component::{use_state, view, Component, Element, Props, State};
    pub use crate::db::{use_db, use_repository, Model, QueryBuilder};
    pub use crate::env::{env, env_or, is_development, is_production};
    pub use crate::error::{use_error_handler, ErrorBoundary};
    pub use crate::fetch::{get, post, FetchBuilder, Method, SWR};
    pub use crate::form::{use_form, Form, FormConfig};
    pub use crate::hooks::{
        use_state as use_state_hook, use_reducer, use_effect, use_memo, use_callback, 
        use_ref, use_layout_effect, use_context, provide_context, Context as HookContext,
        use_counter, use_previous, use_debounce
    };
    pub use crate::i18n::{use_i18n, Locale};
    pub use crate::image::{Image, Picture};
    pub use crate::layers::*;
    pub use crate::middleware::{Context, Middleware, MiddlewareStack};
    pub use crate::monitoring::{use_analytics, use_metrics, use_performance};
    pub use crate::reactive_v2::{init_renderer, mount, queue_current_render};
    pub use crate::router::{Page, Route, RouteHandler};
    pub use crate::router_v2::{init_router, navigate, route, use_route, use_router, Link};
    pub use crate::security::{use_csrf_token, use_security, XssProtection};
    pub use crate::server::{Response, ServerError, ServerFunction};
    #[cfg(feature = "ssr")]
    pub use crate::ssr::{create_ssr_server, hydrate, hydrate_app_internal, SSRApp, SSRComponent, SSG};
    pub use crate::state::{
        create_app_store, create_atom, use_atom, use_selector, AppAction, AppState,
        // Note: use_effect is now provided by hooks module
    };
    pub use crate::styles::{inject_global_styles, style, StyleBuilder};
    pub use crate::test::{TestContext, TestResult, TestUtils};
    pub use crate::ui::*;
    pub use crate::upload::{FileUpload, FileUploadManager, UploadStatus};
    pub use crate::websocket::{use_websocket, WsMessage, WsState};
}

// Layer validation at compile time
#[macro_export]
macro_rules! enforce_layer {
    ($from:expr, $to:expr) => {
        const _: () = {
            assert!(
                $from >= $to,
                "Invalid layer access: higher layers cannot access lower layers directly"
            );
        };
    };
}
