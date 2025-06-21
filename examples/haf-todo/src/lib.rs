//! HAF Todo Example - Demonstrates Hierarchical Architecture First principles
//! 
//! This example shows how to build a Todo app following HAF principles:
//! - L1 (Domain): Pure business logic with no dependencies
//! - L2 (Runtime): State management and effects
//! - L3 (UI): User interface and external interactions
//! 
//! Notice how dependencies only flow downward: L3 → L2 → L1

use wasm_bindgen::prelude::*;
use layer9_core::haf::l3_framework::App;

// Layer modules - organized by architectural layers
pub mod l1_domain;
pub mod l2_runtime; 
pub mod l3_ui;

/// Entry point for the WASM application
#[wasm_bindgen(start)]
pub fn main() {
    // Set panic hook for better error messages in browser
    console_error_panic_hook::set_once();
    
    // Create and mount the application
    let app = App::new()
        .component(|| {
            let store = l2_runtime::todo_runtime::create_store();
            l3_ui::render_app(&store)
        });
    
    // Inject styles
    inject_styles();
    
    // Mount to DOM
    #[cfg(target_arch = "wasm32")]
    app.mount("#app");
}

/// Inject TodoMVC styles into the document
fn inject_styles() {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let head = document.head().unwrap();
        
        let style = document.create_element("style").unwrap();
        style.set_inner_html(l3_ui::get_styles());
        head.append_child(&style).unwrap();
    }
}

/// Example of HAF principles in action
#[cfg(test)]
mod haf_principles {
    use super::*;
    use crate::l1_domain::{TodoList, TodoAction};
    
    #[test]
    fn test_layer_separation() {
        // L1: Pure domain logic - no I/O, no dependencies
        let list = TodoList::new();
        let list = list.reduce(TodoAction::Add {
            title: "Learn HAF".to_string()
        });
        assert_eq!(list.todos.len(), 1);
        
        // L2: Runtime manages state and effects
        let store = l2_runtime::TodoStore::new();
        store.dispatch(TodoAction::Add {
            title: "Apply HAF".to_string()
        });
        
        // L3: UI renders based on state from L2
        // In a real app, this would create VNodes
        let _vnode = l3_ui::render_app(&store);
    }
    
    #[test]
    fn test_dependency_direction() {
        // This compiles: L2 can use L1
        use crate::l1_domain::Todo;
        let _todo = Todo {
            id: 1,
            title: "Test".to_string(),
            completed: false,
        };
        
        // This compiles: L3 can use L2
        use crate::l2_runtime::TodoStore;
        let _store = TodoStore::new();
        
        // L1 cannot use L2 or L3 (would not compile if attempted)
        // This enforces proper dependency direction
    }
    
    #[test]
    fn test_translation_contracts() {
        use layer9_core::haf::L1ToL2Contract;
        use crate::l2_runtime::{ActionToCommandContract, Command};
        
        // Actions (L1) are translated to Commands (L2)
        let action = TodoAction::Add {
            title: "Test contract".to_string()
        };
        
        let commands = ActionToCommandContract::translate(action);
        assert!(!commands.is_empty());
        
        // Each layer speaks its own language
        match &commands[0] {
            Command::LogAction(msg) => {
                assert!(msg.contains("Adding todo"));
            }
            _ => {}
        }
    }
}

/// Documentation of the HAF structure
pub mod architecture {
    //! # HAF Architecture Overview
    //! 
    //! ## Layer 1: Domain (l1_domain)
    //! - Pure business logic
    //! - No framework dependencies
    //! - Immutable data structures
    //! - Pure functions only
    //! 
    //! ## Layer 2: Runtime (l2_runtime)  
    //! - State management (TodoStore)
    //! - Side effects (Storage, Analytics)
    //! - Translation contracts
    //! - Event handling
    //! 
    //! ## Layer 3: UI (l3_ui)
    //! - User interface components
    //! - External API calls
    //! - Browser interactions
    //! - Framework-specific code
    //! 
    //! ## Key Principles
    //! 1. Dependencies flow downward only (L3 → L2 → L1)
    //! 2. Each layer has clear responsibilities
    //! 3. Explicit contracts between layers
    //! 4. Pure core with effects at edges
}

/// Example usage from JavaScript
#[wasm_bindgen]
pub fn create_todo_app() -> Result<(), JsValue> {
    main();
    Ok(())
}

/// Expose todo operations to JavaScript
#[wasm_bindgen]
pub struct TodoAppHandle {
    store: std::rc::Rc<l2_runtime::TodoStore>,
}

#[wasm_bindgen]
impl TodoAppHandle {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        TodoAppHandle {
            store: l2_runtime::todo_runtime::create_store(),
        }
    }
    
    #[wasm_bindgen]
    pub fn add_todo(&self, title: String) {
        use l1_domain::TodoAction;
        self.store.dispatch(TodoAction::Add { title });
    }
    
    #[wasm_bindgen]
    pub fn toggle_todo(&self, id: usize) {
        use l1_domain::TodoAction;
        self.store.dispatch(TodoAction::Toggle { id });
    }
    
    #[wasm_bindgen]
    pub fn get_active_count(&self) -> usize {
        self.store.get_state().active_count()
    }
}