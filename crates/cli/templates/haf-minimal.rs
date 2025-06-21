//! Minimal HAF Layer9 Application Template
//! 
//! This template demonstrates the hierarchical architecture with proper layer separation.

use layer9::prelude::*;
use layer9::haf::{layers::*, component::*, vdom::*, Contract};

// ==================== L1: Pure Business Logic ====================

/// Application state - pure data
#[derive(Clone, Debug)]
struct AppState {
    count: i32,
    message: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            count: 0,
            message: "Welcome to HAF Layer9!".to_string(),
        }
    }
}

/// Pure state transitions
fn increment_count(state: &AppState) -> AppState {
    AppState {
        count: state.count + 1,
        ..state.clone()
    }
}

fn decrement_count(state: &AppState) -> AppState {
    AppState {
        count: state.count - 1,
        ..state.clone()
    }
}

/// Counter component - pure render function
struct Counter;

impl Default for Counter {
    fn default() -> Self {
        Self
    }
}

impl PureComponent<L1> for Counter {
    type Props = AppState;
    
    fn render(&self, props: &Self::Props) -> VNode<L1> {
        VNode::Element {
            tag: "div".to_string(),
            props: VProps {
                class: Some("counter".to_string()),
                ..Default::default()
            },
            children: vec![
                VNode::Element {
                    tag: "h1".to_string(),
                    props: VProps::default(),
                    children: vec![VNode::Text(props.message.clone())],
                },
                VNode::Element {
                    tag: "div".to_string(),
                    props: VProps {
                        class: Some("count-display".to_string()),
                        ..Default::default()
                    },
                    children: vec![
                        VNode::Text(format!("Count: {}", props.count))
                    ],
                },
                VNode::Element {
                    tag: "div".to_string(),
                    props: VProps {
                        class: Some("buttons".to_string()),
                        ..Default::default()
                    },
                    children: vec![
                        VNode::Element {
                            tag: "button".to_string(),
                            props: VProps {
                                events: vec![("click".to_string(), EventId(1))],
                                ..Default::default()
                            },
                            children: vec![VNode::Text("Increment".to_string())],
                        },
                        VNode::Element {
                            tag: "button".to_string(),
                            props: VProps {
                                events: vec![("click".to_string(), EventId(2))],
                                ..Default::default()
                            },
                            children: vec![VNode::Text("Decrement".to_string())],
                        },
                    ],
                },
            ],
        }
    }
}

// ==================== L2: Runtime Layer ====================

/// Application runtime managing state and effects
pub struct AppRuntime {
    state: AppState,
    component_runtime: ComponentRuntime<L2>,
    patch_runtime: PatchRuntime<L2>,
}

impl AppRuntime {
    pub fn new() -> Self {
        Self {
            state: AppState::default(),
            component_runtime: ComponentRuntime::new(),
            patch_runtime: PatchRuntime::new(),
        }
    }
    
    /// Handle state updates
    pub fn dispatch(&mut self, action: Action) -> Vec<Contract<Patch<L1>, DomOperation>> {
        // Update state based on action
        let new_state = match action {
            Action::Increment => increment_count(&self.state),
            Action::Decrement => decrement_count(&self.state),
        };
        
        // Generate diff
        let old_vnode = self.render_app(&self.state);
        let new_vnode = self.render_app(&new_state);
        
        let diff = VDomDiff::<L1>::new();
        let patches = diff.diff(&old_vnode, &new_vnode);
        
        // Update state
        self.state = new_state;
        
        // Convert patches to DOM operations
        self.patch_runtime.apply_patches(patches, DomHandle { node_id: 0 })
    }
    
    fn render_app(&self, state: &AppState) -> VNode<L1> {
        let counter = Counter::default();
        counter.render(state)
    }
    
    /// Register event handlers
    pub fn setup_event_handlers(&mut self) {
        // Register increment handler
        self.patch_runtime.register_event(EventId(1), Box::new(|| {
            // In real app, would send Action::Increment to dispatcher
            println!("Increment clicked");
        }));
        
        // Register decrement handler
        self.patch_runtime.register_event(EventId(2), Box::new(|| {
            // In real app, would send Action::Decrement to dispatcher
            println!("Decrement clicked");
        }));
    }
}

/// Actions that can modify state
#[derive(Clone, Debug)]
pub enum Action {
    Increment,
    Decrement,
}

// ==================== L3: Framework Layer ====================

/// Application entry point
#[wasm_bindgen(start)]
pub fn main() {
    // Set up panic hook for better error messages
    console_error_panic_hook::set_once();
    
    // Create app instance
    let mut app = ComponentApp::<L3>::new();
    
    // Create initial app state
    let initial_state = AppState::default();
    let counter = Counter::default();
    let initial_vnode = counter.render(&initial_state);
    
    // Mount to DOM
    app.mount("#app", initial_vnode);
}

// ==================== Styles ====================

/// CSS for the application
pub fn styles() -> &'static str {
    r#"
    body {
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        background: #0a0a0a;
        color: #ffffff;
        margin: 0;
        padding: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        min-height: 100vh;
    }
    
    .counter {
        text-align: center;
        padding: 2rem;
        background: rgba(255, 255, 255, 0.05);
        border-radius: 1rem;
        border: 1px solid rgba(255, 255, 255, 0.1);
    }
    
    h1 {
        margin: 0 0 2rem 0;
        font-size: 2rem;
        background: linear-gradient(to right, #60a5fa, #a78bfa);
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
    }
    
    .count-display {
        font-size: 3rem;
        font-weight: bold;
        margin: 2rem 0;
    }
    
    .buttons {
        display: flex;
        gap: 1rem;
        justify-content: center;
    }
    
    button {
        background: rgba(99, 102, 241, 0.2);
        border: 1px solid rgb(99, 102, 241);
        color: rgb(199, 210, 254);
        padding: 0.75rem 1.5rem;
        border-radius: 0.5rem;
        font-size: 1rem;
        cursor: pointer;
        transition: all 0.2s;
    }
    
    button:hover {
        background: rgba(99, 102, 241, 0.3);
        transform: translateY(-2px);
    }
    
    button:active {
        transform: translateY(0);
    }
    "#
}