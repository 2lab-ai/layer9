//! Async Components with Suspense Support - L5
//! 
//! This module provides async component loading with Suspense boundaries
//! for handling loading states and error boundaries for error handling.

use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

use wasm_bindgen_futures::spawn_local;

use crate::component::{Component, Element, Props};
use crate::error::ErrorBoundary;
use crate::hooks::{use_state, use_effect};
use crate::reactive::queue_current_render;

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
        // Check if any child component is in loading state
        let is_loading = SUSPENSE_CONTEXT.with(|ctx| {
            ctx.borrow().is_loading
        });

        if is_loading {
            self.fallback.render()
        } else {
            // Render children within suspense context
            with_suspense_context(|| {
                self.children.render()
            })
        }
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

/// Suspense context for tracking loading states
#[derive(Clone)]
struct SuspenseContext {
    is_loading: bool,
    pending_count: u32,
}

thread_local! {
    static SUSPENSE_CONTEXT: RefCell<SuspenseContext> = RefCell::new(SuspenseContext {
        is_loading: false,
        pending_count: 0,
    });
}

/// Run a closure within a suspense context
fn with_suspense_context<T>(f: impl FnOnce() -> T) -> T {
    SUSPENSE_CONTEXT.with(|ctx| {
        let prev_state = ctx.borrow().clone();
        ctx.borrow_mut().is_loading = false;
        ctx.borrow_mut().pending_count = 0;
        
        let result = f();
        
        // Restore previous state
        *ctx.borrow_mut() = prev_state;
        
        result
    })
}

/// Mark current suspense boundary as loading
fn mark_suspense_loading() {
    SUSPENSE_CONTEXT.with(|ctx| {
        ctx.borrow_mut().is_loading = true;
        ctx.borrow_mut().pending_count += 1;
    });
}

/// Mark async operation as complete
fn mark_suspense_complete() {
    SUSPENSE_CONTEXT.with(|ctx| {
        let mut context = ctx.borrow_mut();
        if context.pending_count > 0 {
            context.pending_count -= 1;
        }
        if context.pending_count == 0 {
            context.is_loading = false;
        }
    });
}

/// Async component trait
pub trait AsyncComponent: Component {
    type Data: Clone + 'static;
    
    /// Load async data
    fn load(&self) -> Pin<Box<dyn Future<Output = Result<Self::Data, String>>>>;
    
    /// Render with loaded data
    fn render_with_data(&self, data: &Self::Data) -> Element;
}

/// Wrapper to make async components work with the Component trait
pub struct AsyncComponentWrapper<C: AsyncComponent> {
    inner: C,
    state: Rc<RefCell<AsyncState<C::Data>>>,
}

impl<C: AsyncComponent> AsyncComponentWrapper<C> {
    pub fn new(component: C) -> Self {
        AsyncComponentWrapper {
            inner: component,
            state: Rc::new(RefCell::new(AsyncState::Loading)),
        }
    }
}

impl<C: AsyncComponent + 'static> Component for AsyncComponentWrapper<C> {
    fn render(&self) -> Element {
        let state = self.state.clone();
        let inner = &self.inner;
        
        // Use effect to load data on mount
        use_effect((), {
            let state = state.clone();
            let future = inner.load();
            
            move || {
                // Mark suspense as loading
                mark_suspense_loading();
                
                // Spawn async task
                spawn_local(async move {
                    match future.await {
                        Ok(data) => {
                            *state.borrow_mut() = AsyncState::Success(data);
                        }
                        Err(err) => {
                            *state.borrow_mut() = AsyncState::Error(err);
                        }
                    }
                    
                    // Mark as complete and trigger re-render
                    mark_suspense_complete();
                    queue_current_render();
                });
                
                // Cleanup function
                || {}
            }
        });
        
        // Render based on current state
        match &*self.state.borrow() {
            AsyncState::Loading => {
                // Suspense will handle this
                Element::Text("".to_string())
            }
            AsyncState::Success(data) => {
                self.inner.render_with_data(data)
            }
            AsyncState::Error(err) => {
                // Error boundary will catch this
                panic!("Async component error: {}", err);
            }
        }
    }
}

/// Hook for async data loading
pub fn use_async<T, F>(loader: F) -> AsyncState<T>
where
    T: Clone + 'static,
    F: FnOnce() -> Pin<Box<dyn Future<Output = Result<T, String>>>> + Clone + 'static,
{
    let (state, set_state) = use_state(|| AsyncState::<T>::Loading);
    
    use_effect((), move || {
        // Mark suspense as loading
        mark_suspense_loading();
        
        let future = loader();
        
        // Spawn async task
        spawn_local(async move {
            match future.await {
                Ok(data) => {
                    set_state(AsyncState::Success(data));
                }
                Err(err) => {
                    set_state(AsyncState::Error(err));
                }
            }
            
            // Mark as complete
            mark_suspense_complete();
        });
        
        // Cleanup
        || {}
    });
    
    state
}

/// Create an async component
pub fn async_component<C: AsyncComponent + 'static>(component: C) -> impl Component {
    AsyncComponentWrapper::new(component)
}

/// Example async component
pub struct AsyncDataComponent {
    url: String,
}

impl AsyncDataComponent {
    pub fn new(url: impl Into<String>) -> Self {
        AsyncDataComponent {
            url: url.into(),
        }
    }
}

impl Component for AsyncDataComponent {
    fn render(&self) -> Element {
        // Use async hook
        let data = use_async({
            let url = self.url.clone();
            move || {
                Box::pin(async move {
                    // Simulate async data fetching - for now just return dummy data
                    // TODO: Implement proper fetch when async runtime is ready
                    Ok(format!("Data from {}", url))
                })
            }
        });
        
        match data {
            AsyncState::Loading => Element::Text("Loading data...".to_string()),
            AsyncState::Success(text) => {
                Element::Node {
                    tag: "div".to_string(),
                    props: Props::default(),
                    children: vec![
                        Element::Text(format!("Data: {}", text))
                    ],
                }
            }
            AsyncState::Error(err) => {
                Element::Node {
                    tag: "div".to_string(),
                    props: Props {
                        class: Some("error".to_string()),
                        ..Default::default()
                    },
                    children: vec![
                        Element::Text(format!("Error: {}", err))
                    ],
                }
            }
        }
    }
}

/// Helper to create a Suspense boundary with error handling
pub fn with_async_boundary(
    children: impl Component + 'static,
    loading: impl Component + 'static,
    error_fallback: impl Fn(&crate::error::ErrorInfo) -> Element + 'static,
) -> impl Component {
    ErrorBoundary::new(
        Suspense::new(children).fallback(loading)
    ).fallback(error_fallback)
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
}