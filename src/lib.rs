//! Layer9 Framework - Main entry point
//!
//! Re-exports all core functionality

pub use layer9_core::*;
pub use layer9_macro::*;

pub mod prelude {
    pub use layer9_core::{
        app::*, component::*, layers::*, router::*, server::*, state::*, ui::*, vdom::*,
    };

    pub use layer9_macro::*;
}

// Re-export commonly used external dependencies
pub use js_sys;
pub use wasm_bindgen;
pub use web_sys;
