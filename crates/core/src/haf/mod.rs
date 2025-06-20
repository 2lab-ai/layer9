//! HAF (Hierarchical Architecture First) enforcement system
//! 
//! This module provides compile-time guarantees that code follows HAF principles.
//! It uses Rust's type system to prevent architectural violations.

use std::marker::PhantomData;

// Sub-modules
pub mod contracts;
pub mod l1_core;
pub mod l2_runtime;
pub mod l3_framework;
pub mod component;
pub mod compat;
pub mod vdom;
pub mod reactive;
#[cfg(test)]
pub mod tests;

// Re-export commonly used items
pub use contracts::HttpResponse;
pub use l1_core::*;
// Re-export specific items to avoid conflicts
pub use l2_runtime::{Runtime as L2Runtime, Scheduler as L2Scheduler};
pub use l3_framework::{App, Runtime as L3Runtime, Scheduler as L3Scheduler, 
                       router, hooks, http, websocket, devtools};

/// Layer markers - zero-sized types representing architectural layers
pub mod layers {
    /// Layer 1: Core - Pure business logic, no I/O, no dependencies
    pub struct L1;
    
    /// Layer 2: Runtime - Execution environment, manages side effects
    pub struct L2;
    
    /// Layer 3: Framework - External interfaces, user-facing APIs
    pub struct L3;
}

/// Trait that all layer types must implement
pub trait Layer: 'static {
    /// Layer depth - L1=1, L2=2, L3=3
    const DEPTH: u8;
    
    /// Layer name for debugging
    const NAME: &'static str;
}

impl Layer for layers::L1 {
    const DEPTH: u8 = 1;
    const NAME: &'static str = "Core";
}

impl Layer for layers::L2 {
    const DEPTH: u8 = 2;
    const NAME: &'static str = "Runtime";
}

impl Layer for layers::L3 {
    const DEPTH: u8 = 3;
    const NAME: &'static str = "Framework";
}

/// Component tagged with its layer
pub struct Component<L: Layer> {
    #[allow(dead_code)]
    pub(crate) inner: Box<dyn ComponentInner>,
    pub(crate) _layer: PhantomData<L>,
}

/// Internal component representation
pub trait ComponentInner: 'static {
    fn render(&self) -> VNode;
}

/// Virtual DOM node - pure data structure (L1)
#[derive(Clone, Debug, PartialEq)]
pub enum VNode {
    Text(String),
    Element {
        tag: String,
        props: Props,
        children: Vec<VNode>,
    },
}

/// Component properties - pure data (L1)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Props {
    pub attributes: Vec<(String, String)>,
}

/// Contract wrapper for layer translations
#[derive(Debug, Clone)]
pub struct Contract<Input, Output> {
    pub input: Input,
    pub output: Output,
}

impl<I, O> Contract<I, O> {
    pub fn new(input: I, output: O) -> Self {
        Self { input, output }
    }
}

/// Translation contract from L1 to L2
pub trait L1ToL2Contract {
    type L1Type;
    type L2Type;
    
    fn translate(l1: Self::L1Type) -> Self::L2Type;
}

/// Translation contract from L2 to L3
pub trait L2ToL3Contract {
    type L2Type;
    type L3Type;
    
    fn translate(l2: Self::L2Type) -> Self::L3Type;
}

/// Dependency direction enforcement
/// This trait can only be implemented for valid layer transitions
pub trait CanDepend<Target: Layer>: Layer {}

// L3 can depend on L2
impl CanDepend<layers::L2> for layers::L3 {}

// L3 can depend on L1
impl CanDepend<layers::L1> for layers::L3 {}

// L2 can depend on L1
impl CanDepend<layers::L1> for layers::L2 {}

// Self-dependencies are allowed
impl CanDepend<layers::L1> for layers::L1 {}
impl CanDepend<layers::L2> for layers::L2 {}
impl CanDepend<layers::L3> for layers::L3 {}

/// Service boundary with explicit layer
pub struct Service<L: Layer> {
    #[allow(dead_code)]
    pub(crate) name: &'static str,
    pub(crate) _layer: PhantomData<L>,
}

impl<L: Layer> Service<L> {
    pub const fn new(name: &'static str) -> Self {
        Service {
            name,
            _layer: PhantomData,
        }
    }
}

/// HAF-compliant function that enforces dependency rules
pub fn haf_function<From, To: Layer, F, R>(f: F) -> F
where
    From: Layer + CanDepend<To>,
    F: Fn() -> R,
{
    f
}

/// Macro to define HAF-compliant components
#[macro_export]
macro_rules! haf_component {
    // L1 component - pure, no side effects
    (L1, $name:ident, $props:ty, $body:expr) => {
        pub fn $name(props: $props) -> $crate::haf::Component<$crate::haf::layers::L1> {
            use $crate::haf::{Component, ComponentInner, VNode};
            
            struct Inner {
                props: $props,
            }
            
            impl ComponentInner for Inner {
                fn render(&self) -> VNode {
                    let props = &self.props;
                    $body
                }
            }
            
            Component {
                inner: Box::new(Inner { props }),
                _layer: std::marker::PhantomData,
            }
        }
    };
    
    // L2 component - can use runtime features
    (L2, $name:ident, $props:ty, $body:expr) => {
        pub fn $name(props: $props) -> $crate::haf::Component<$crate::haf::layers::L2> {
            use $crate::haf::{Component, ComponentInner, VNode};
            
            struct Inner {
                props: $props,
            }
            
            impl ComponentInner for Inner {
                fn render(&self) -> VNode {
                    let props = &self.props;
                    // L2 can access runtime features
                    $body
                }
            }
            
            Component {
                inner: Box::new(Inner { props }),
                _layer: std::marker::PhantomData,
            }
        }
    };
    
    // L3 component - full framework features
    (L3, $name:ident, $props:ty, $body:expr) => {
        pub fn $name(props: $props) -> $crate::haf::Component<$crate::haf::layers::L3> {
            use $crate::haf::{Component, ComponentInner, VNode};
            
            struct Inner {
                props: $props,
            }
            
            impl ComponentInner for Inner {
                fn render(&self) -> VNode {
                    let props = &self.props;
                    // L3 can access all features
                    $body
                }
            }
            
            Component {
                inner: Box::new(Inner { props }),
                _layer: std::marker::PhantomData,
            }
        }
    };
}

/// Service definition macro with layer enforcement
#[macro_export]
macro_rules! haf_service {
    ($layer:ident, $name:ident, {
        $(
            pub fn $fn_name:ident($($arg:ident: $arg_ty:ty),*) -> $ret:ty $body:block
        )*
    }) => {
        pub mod $name {
            use super::*;
            use $crate::haf::{Service, layers::$layer};
            
            pub const SERVICE: Service<$layer> = Service::new(stringify!($name));
            
            $(
                pub fn $fn_name($($arg: $arg_ty),*) -> $ret {
                    // Function is tagged with service layer
                    $body
                }
            )*
        }
    };
}

/// Contract definition macro
#[macro_export]
macro_rules! haf_contract {
    ($name:ident: $from:ident -> $to:ident {
        $(
            fn $method:ident($input:ty) -> $output:ty;
        )*
    }) => {
        pub trait $name {
            $(
                fn $method(input: $input) -> $output;
            )*
        }
        
        // Ensure contract follows dependency rules
        const _: () = {
            fn _check_contract<F: $crate::haf::Layer, T: $crate::haf::Layer>()
            where
                F: $crate::haf::CanDepend<T>
            {}
            
            fn _verify() {
                _check_contract::<$crate::haf::layers::$from, $crate::haf::layers::$to>();
            }
        };
    };
}
