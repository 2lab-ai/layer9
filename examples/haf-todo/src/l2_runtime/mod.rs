//! Layer 2: Runtime - Todo application runtime
//! 
//! This layer manages state, effects, and coordinates between L1 and L3.
//! It can perform side effects but only depends on L1.

use crate::l1_domain::{Filter, Todo, TodoAction, TodoList};
use layer9_core::haf::layers::{L1, L2};
use layer9_core::haf::{Service, L1ToL2Contract};
use std::cell::RefCell;
use std::rc::Rc;

/// Todo store - manages application state
pub struct TodoStore {
    state: RefCell<TodoList>,
    subscribers: RefCell<Vec<Box<dyn Fn(&TodoList)>>>,
}

impl TodoStore {
    /// Create a new todo store
    pub fn new() -> Rc<Self> {
        let store = Rc::new(TodoStore {
            state: RefCell::new(TodoList::new()),
            subscribers: RefCell::new(Vec::new()),
        });
        
        // Load from storage if available
        if let Some(stored) = Storage::load() {
            *store.state.borrow_mut() = stored;
        }
        
        store
    }
    
    /// Dispatch an action to update state
    pub fn dispatch(&self, action: TodoAction) {
        // Apply pure reducer
        let new_state = self.state.borrow().clone().reduce(action);
        
        // Update state
        *self.state.borrow_mut() = new_state.clone();
        
        // Persist to storage
        Storage::save(&new_state);
        
        // Notify subscribers
        for subscriber in self.subscribers.borrow().iter() {
            subscriber(&new_state);
        }
    }
    
    /// Subscribe to state changes
    pub fn subscribe<F>(&self, callback: F) -> Subscription
    where
        F: Fn(&TodoList) + 'static,
    {
        let id = self.subscribers.borrow().len();
        self.subscribers.borrow_mut().push(Box::new(callback));
        
        Subscription {
            id,
            store: self as *const TodoStore,
        }
    }
    
    /// Get current state
    pub fn get_state(&self) -> TodoList {
        self.state.borrow().clone()
    }
}

/// Subscription handle
pub struct Subscription {
    id: usize,
    store: *const TodoStore,
}

impl Drop for Subscription {
    fn drop(&mut self) {
        // In real implementation, would remove subscriber
    }
}

/// Storage service - handles persistence
mod Storage {
    use super::*;
    use crate::l1_domain::serialization::{from_storage_format, to_storage_format};
    
    const STORAGE_KEY: &str = "layer9_haf_todos";
    
    pub fn save(list: &TodoList) {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsValue;
            
            if let Ok(Some(storage)) = web_sys::window()
                .and_then(|w| w.local_storage())
            {
                let data = to_storage_format(list);
                if let Ok(json) = serde_json::to_string(&data) {
                    let _ = storage.set_item(STORAGE_KEY, &json);
                }
            }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            println!("Saving todos: {:?}", list);
        }
    }
    
    pub fn load() -> Option<TodoList> {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
                .and_then(|storage| storage.get_item(STORAGE_KEY).ok())
                .flatten()
                .and_then(|json| serde_json::from_str(&json).ok())
                .map(from_storage_format)
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            None
        }
    }
}

/// Action to Command contract (L1 → L2)
pub struct ActionToCommandContract;

#[derive(Debug)]
pub enum Command {
    SaveToStorage(TodoList),
    NotifySubscribers(TodoList),
    LogAction(String),
}

impl L1ToL2Contract for ActionToCommandContract {
    type L1Type = TodoAction;
    type L2Type = Vec<Command>;
    
    fn translate(action: Self::L1Type) -> Self::L2Type {
        let description = match &action {
            TodoAction::Add { title } => format!("Adding todo: {}", title),
            TodoAction::Toggle { id } => format!("Toggling todo {}", id),
            TodoAction::Delete { id } => format!("Deleting todo {}", id),
            TodoAction::Edit { id, title } => format!("Editing todo {} to: {}", id, title),
            TodoAction::SetFilter { filter } => format!("Setting filter to: {:?}", filter),
            TodoAction::ClearCompleted => "Clearing completed todos".to_string(),
        };
        
        vec![
            Command::LogAction(description),
            // Other commands would be added based on the action
        ]
    }
}

/// Todo runtime service
layer9_core::haf_service!(L2, todo_runtime, {
    pub fn create_store() -> Rc<TodoStore> {
        TodoStore::new()
    }
    
    pub fn dispatch_action(store: &TodoStore, action: TodoAction) -> () {
        store.dispatch(action)
    }
    
    pub fn get_current_state(store: &TodoStore) -> TodoList {
        store.get_state()
    }
});

/// Analytics service - tracks todo usage
pub mod analytics {
    use super::*;
    
    #[derive(Default)]
    pub struct TodoAnalytics {
        actions_count: RefCell<usize>,
        todos_created: RefCell<usize>,
        todos_completed: RefCell<usize>,
    }
    
    impl TodoAnalytics {
        pub fn track_action(&self, action: &TodoAction) {
            *self.actions_count.borrow_mut() += 1;
            
            match action {
                TodoAction::Add { .. } => {
                    *self.todos_created.borrow_mut() += 1;
                }
                TodoAction::Toggle { .. } => {
                    *self.todos_completed.borrow_mut() += 1;
                }
                _ => {}
            }
        }
        
        pub fn get_stats(&self) -> Stats {
            Stats {
                total_actions: *self.actions_count.borrow(),
                todos_created: *self.todos_created.borrow(),
                todos_completed: *self.todos_completed.borrow(),
            }
        }
    }
    
    pub struct Stats {
        pub total_actions: usize,
        pub todos_created: usize,
        pub todos_completed: usize,
    }
}

/// Effects that can be triggered by todo actions
pub mod effects {
    use super::*;
    
    /// Show notification when todo is completed
    pub fn completion_effect(todo: &Todo) {
        if todo.completed {
            #[cfg(target_arch = "wasm32")]
            {
                web_sys::console::log_1(&format!("✅ Completed: {}", todo.title).into());
            }
            
            #[cfg(not(target_arch = "wasm32"))]
            {
                println!("✅ Completed: {}", todo.title);
            }
        }
    }
    
    /// Log all actions for debugging
    pub fn logging_effect(action: &TodoAction) {
        let msg = format!("[TodoAction] {:?}", action);
        
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::log_1(&msg.into());
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            println!("{}", msg);
        }
    }
}