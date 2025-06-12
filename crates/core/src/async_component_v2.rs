//! Simplified Async Components with Suspense Support - L5
//! 
//! This module provides a simpler async component implementation
//! that works within current WASM and lifetime constraints.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen_futures::spawn_local;

use crate::component::{Component, Element, Props, use_state};
use crate::error::ErrorBoundary;
use crate::hooks::use_effect;
use crate::reactive_v2::queue_current_render;

/// Loading state for async components
#[derive(Clone, Debug)]
pub enum AsyncState<T> {
    Loading,
    Success(T),
    Error(String),
}

/// Suspense boundary component
pub struct Suspense {
    children: Box<dyn Component>,
    fallback: Box<dyn Component>,
}

impl Suspense {
    pub fn new(children: impl Component + 'static) -> Self {
        Suspense {
            children: Box::new(children),
            fallback: Box::new(DefaultLoadingComponent),
        }
    }

    pub fn fallback(mut self, fallback: impl Component + 'static) -> Self {
        self.fallback = Box::new(fallback);
        self
    }
}

impl Component for Suspense {
    fn render(&self) -> Element {
        // For now, just render children directly
        // In a full implementation, we'd track loading state
        self.children.render()
    }
}

/// Default loading component
struct DefaultLoadingComponent;

impl Component for DefaultLoadingComponent {
    fn render(&self) -> Element {
        Element::Node {
            tag: "div".to_string(),
            props: Props {
                class: Some("layer9-loading".to_string()),
                ..Default::default()
            },
            children: vec![
                Element::Text("Loading...".to_string())
            ],
        }
    }
}

/// Async data container
pub struct AsyncData<T: Clone + 'static> {
    state: Rc<RefCell<AsyncState<T>>>,
}

impl<T: Clone + 'static> AsyncData<T> {
    pub fn new() -> Self {
        AsyncData {
            state: Rc::new(RefCell::new(AsyncState::Loading)),
        }
    }
    
    pub fn load<F>(&self, f: F)
    where
        F: FnOnce() + 'static,
    {
        f();
    }
    
    pub fn set_loading(&self) {
        *self.state.borrow_mut() = AsyncState::Loading;
        queue_current_render();
    }
    
    pub fn set_success(&self, data: T) {
        *self.state.borrow_mut() = AsyncState::Success(data);
        queue_current_render();
    }
    
    pub fn set_error(&self, error: String) {
        *self.state.borrow_mut() = AsyncState::Error(error);
        queue_current_render();
    }
    
    pub fn get(&self) -> AsyncState<T> {
        self.state.borrow().clone()
    }
}

impl<T: Clone + 'static> Clone for AsyncData<T> {
    fn clone(&self) -> Self {
        AsyncData {
            state: self.state.clone(),
        }
    }
}

/// Hook for async data loading with manual control
pub fn use_async_data<T: Clone + 'static>() -> AsyncData<T> {
    let state = use_state(|| AsyncData::new());
    state.get()
}

/// Example component that loads data asynchronously
pub struct AsyncExample;

impl Component for AsyncExample {
    fn render(&self) -> Element {
        let data = use_async_data::<String>();
        
        // Load data on mount
        use_effect((), {
            let data = data.clone();
            move || {
                // Start loading
                data.set_loading();
                
                // Spawn async task
                spawn_local(async move {
                    // Simulate delay
                    gloo_timers::future::TimeoutFuture::new(1000).await;
                    
                    // Set success
                    data.set_success("Hello from async!".to_string());
                });
                
                // Cleanup
                || {}
            }
        });
        
        match data.get() {
            AsyncState::Loading => Element::Text("Loading...".to_string()),
            AsyncState::Success(text) => Element::Text(text),
            AsyncState::Error(err) => Element::Text(format!("Error: {}", err)),
        }
    }
}

/// Helper to create a component with error boundary
pub fn with_error_boundary(
    children: impl Component + 'static,
    error_fallback: impl Fn(&crate::error::ErrorInfo) -> Element + 'static,
) -> impl Component {
    ErrorBoundary::new(children).fallback(error_fallback)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_async_state() {
        let loading: AsyncState<String> = AsyncState::Loading;
        assert!(matches!(loading, AsyncState::Loading));
        
        let success = AsyncState::Success("data".to_string());
        assert!(matches!(success, AsyncState::Success(_)));
        
        let error: AsyncState<String> = AsyncState::Error("error".to_string());
        assert!(matches!(error, AsyncState::Error(_)));
    }
    
    #[test]
    fn test_async_data() {
        let data = AsyncData::<String>::new();
        assert!(matches!(data.get(), AsyncState::Loading));
        
        data.set_success("test".to_string());
        assert!(matches!(data.get(), AsyncState::Success(s) if s == "test"));
        
        data.set_error("error".to_string());
        assert!(matches!(data.get(), AsyncState::Error(e) if e == "error"));
    }
}