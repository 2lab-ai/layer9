//! WARP Core - Hierarchical Web Framework
//! 
//! L9 Philosophy: Consciousness-aware web development
//! L8 Architecture: Enforced layer separation
//! L7 Application: Business logic isolation
//! L6 Features: Modular feature system
//! L5 Components: Type-safe reactive components
//! L4 Services: Server/Client boundary
//! L3 Runtime: WASM/JS execution
//! L2 Platform: Next.js compatibility
//! L1 Infrastructure: Build and deploy

pub mod layers;
pub mod component;
pub mod router;
pub mod server;
pub mod vdom;
pub mod app;
pub mod auth;
pub mod styles;
pub mod ui;

pub mod prelude {
    pub use crate::layers::*;
    pub use crate::component::{Component, Element, view, Props};
    pub use crate::router::{Page, Route, RouteHandler};
    pub use crate::server::{ServerFunction, Response};
    pub use crate::app::{WarpApp, run_app};
    pub use crate::auth::{AuthService, use_auth, Protected};
    pub use crate::styles::{style, StyleBuilder, inject_global_styles};
    pub use crate::ui::*;
    pub use warp_macro::{warp_app, page, component, server};
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