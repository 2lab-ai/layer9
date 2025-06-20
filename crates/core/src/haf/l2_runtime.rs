//! Layer 2: Runtime - Execution environment
//! 
//! This layer manages side effects and provides the runtime for L1's pure logic.
//! It can perform I/O but only depends on L1.

use super::{
    contracts::{DomOp, Effect, EffectAction},
    l1_core::{Patch, SignalValue},
};
use std::cell::RefCell;
use std::collections::HashMap;

/// Runtime context for managing application state
pub struct Runtime {
    /// Component instances
    components: RefCell<HashMap<usize, Box<dyn ComponentRuntime>>>,
    /// Signal values
    signals: RefCell<HashMap<usize, SignalValue>>,
    /// Registered effects
    effects: RefCell<HashMap<usize, Effect>>,
    /// DOM node references (for browser runtime)
    #[cfg(target_arch = "wasm32")]
    dom_nodes: RefCell<HashMap<usize, web_sys::Node>>,
}

/// Component runtime interface
pub trait ComponentRuntime {
    fn update(&self);
    fn get_id(&self) -> usize;
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        Runtime {
            components: RefCell::new(HashMap::new()),
            signals: RefCell::new(HashMap::new()),
            effects: RefCell::new(HashMap::new()),
            #[cfg(target_arch = "wasm32")]
            dom_nodes: RefCell::new(HashMap::new()),
        }
    }
    
    /// Execute DOM operations (L1 → L2 translation)
    pub fn apply_dom_ops(&self, ops: Vec<DomOp>) {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            let document = web_sys::window().unwrap().document().unwrap();
            
            for op in ops {
                match op {
                    DomOp::CreateElement { tag, id } => {
                        let element = document.create_element(&tag).unwrap();
                        self.dom_nodes.borrow_mut().insert(id, element.into());
                    }
                    DomOp::CreateText { text, id } => {
                        let text_node = document.create_text_node(&text);
                        self.dom_nodes.borrow_mut().insert(id, text_node.into());
                    }
                    DomOp::SetAttribute { id, name, value } => {
                        if let Some(node) = self.dom_nodes.borrow().get(&id) {
                            if let Some(element) = node.dyn_ref::<web_sys::Element>() {
                                element.set_attribute(&name, &value).unwrap();
                            }
                        }
                    }
                    DomOp::AppendChild { parent, child } => {
                        let nodes = self.dom_nodes.borrow();
                        if let (Some(parent_node), Some(child_node)) = 
                            (nodes.get(&parent), nodes.get(&child)) {
                            parent_node.append_child(child_node).unwrap();
                        }
                    }
                    // ... other operations
                    _ => {}
                }
            }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Server-side: collect ops for SSR
            println!("DOM ops (server-side): {:?}", ops);
        }
    }
    
    /// Apply patches to update DOM
    pub fn apply_patches(&self, patches: Vec<Patch>) {
        // Translate L1 patches to L2 operations
        let ops: Vec<DomOp> = patches.into_iter().map(|patch| {
            match patch {
                Patch::UpdateText { path, text } => {
                    DomOp::UpdateText { 
                        id: path_to_id(&path), 
                        text 
                    }
                }
                Patch::SetAttribute { path, name, value } => {
                    DomOp::SetAttribute {
                        id: path_to_id(&path),
                        name,
                        value,
                    }
                }
                // ... other patch types
                _ => todo!("Implement other patch translations"),
            }
        }).collect();
        
        self.apply_dom_ops(ops);
    }
    
    /// Update signal value and trigger effects
    pub fn update_signal(&self, signal_id: usize, value: SignalValue) {
        self.signals.borrow_mut().insert(signal_id, value.clone());
        
        // Find and execute effects that depend on this signal
        let effects_to_run: Vec<Effect> = self.effects
            .borrow()
            .values()
            .filter(|effect| effect.dependencies.contains(&signal_id))
            .cloned()
            .collect();
            
        for effect in effects_to_run {
            self.execute_effect(effect);
        }
    }
    
    /// Execute an effect
    fn execute_effect(&self, effect: Effect) {
        match effect.action {
            EffectAction::UpdateDom { target } => {
                // Re-render the target component
                if let Some(component) = self.components.borrow().get(&target) {
                    component.update();
                }
            }
            EffectAction::TriggerRender { component } => {
                if let Some(comp) = self.components.borrow().get(&component) {
                    comp.update();
                }
            }
            EffectAction::RunCallback { callback_id } => {
                // Execute registered callback
                println!("Running callback {}", callback_id);
            }
        }
    }
    
    /// Register a component with the runtime
    pub fn register_component(&self, id: usize, component: Box<dyn ComponentRuntime>) {
        self.components.borrow_mut().insert(id, component);
    }
    
    /// Register an effect
    pub fn register_effect(&self, effect: Effect) {
        self.effects.borrow_mut().insert(effect.effect_id, effect);
    }
}

/// Scheduler for batching updates
pub struct Scheduler {
    pending_updates: RefCell<Vec<Box<dyn FnOnce()>>>,
    is_scheduled: RefCell<bool>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            pending_updates: RefCell::new(Vec::new()),
            is_scheduled: RefCell::new(false),
        }
    }
    
    /// Schedule an update
    pub fn schedule_update<F: FnOnce() + 'static>(&self, update: F) {
        self.pending_updates.borrow_mut().push(Box::new(update));
        
        if !*self.is_scheduled.borrow() {
            *self.is_scheduled.borrow_mut() = true;
            self.schedule_flush();
        }
    }
    
    /// Schedule flush of pending updates
    fn schedule_flush(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            // Use requestAnimationFrame for browser
            // Implementation would use wasm-bindgen
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Immediate flush for server-side
            self.flush();
        }
    }
    
    /// Flush all pending updates
    #[cfg(any(not(target_arch = "wasm32"), test))]
    fn flush(&self) {
        let updates = std::mem::take(&mut *self.pending_updates.borrow_mut());
        *self.is_scheduled.borrow_mut() = false;
        
        for update in updates {
            update();
        }
    }
}

// Memory allocator service (L2)
crate::haf_service!(L2, memory_service, {
    pub fn allocate_component_id() -> usize {
        // Thread-local counter for component IDs
        thread_local! {
            static NEXT_ID: RefCell<usize> = const { RefCell::new(0) };
        }
        
        NEXT_ID.with(|id| {
            let current = *id.borrow();
            *id.borrow_mut() = current + 1;
            current
        })
    }
    
    pub fn allocate_signal_id() -> usize {
        thread_local! {
            static NEXT_ID: RefCell<usize> = const { RefCell::new(0) };
        }
        
        NEXT_ID.with(|id| {
            let current = *id.borrow();
            *id.borrow_mut() = current + 1;
            current
        })
    }
});

/// Helper to convert path to node ID
fn path_to_id(path: &[usize]) -> usize {
    // Simple implementation - in real app would maintain proper mapping
    path.iter().fold(0, |acc, &idx| acc * 100 + idx)
}

/// Server-side rendering context
#[cfg(feature = "ssr")]
pub struct SsrContext {
    _html_buffer: RefCell<String>,
    _head_buffer: RefCell<String>,
}

#[cfg(feature = "ssr")]
impl Default for SsrContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "ssr")]
impl SsrContext {
    pub fn new() -> Self {
        SsrContext {
            _html_buffer: RefCell::new(String::new()),
            _head_buffer: RefCell::new(String::new()),
        }
    }
    
    pub fn render_to_string(&self, component: &super::Component<super::layers::L1>) -> String {
        // Render component to HTML string
        let vnode = component.inner.render();
        self.vnode_to_html(&vnode)
    }
    
    #[allow(clippy::only_used_in_recursion)]
    fn vnode_to_html(&self, vnode: &super::VNode) -> String {
        match vnode {
            super::VNode::Text(text) => html_escape::encode_text(text).to_string(),
            super::VNode::Element { tag, props, children } => {
                let mut html = format!("<{}", tag);
                
                for (name, value) in &props.attributes {
                    html.push_str(&format!(" {}=\"{}\"", name, html_escape::encode_text(value)));
                }
                
                html.push('>');
                
                for child in children {
                    html.push_str(&self.vnode_to_html(child));
                }
                
                html.push_str(&format!("</{}>", tag));
                html
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    
    #[test]
    fn test_runtime_creation() {
        let runtime = Runtime::new();
        let signal_id = memory_service::allocate_signal_id();
        
        runtime.update_signal(signal_id, SignalValue::String("Hello".to_string()));
        
        assert_eq!(runtime.signals.borrow().len(), 1);
    }
    
    #[test]
    fn test_scheduler() {
        let scheduler = Scheduler::new();
        let counter = Rc::new(RefCell::new(0));
        
        let counter_clone = counter.clone();
        scheduler.schedule_update(move || {
            *counter_clone.borrow_mut() += 1;
        });
        
        // In test environment, flush immediately
        scheduler.flush();
        
        assert_eq!(*counter.borrow(), 1);
    }
}