//! Layer 1: Domain - Pure Todo business logic
//! 
//! This layer contains only pure functions and data structures.
//! No I/O, no side effects, no framework dependencies.

use serde::{Deserialize, Serialize};

/// Pure Todo data structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Todo {
    pub id: usize,
    pub title: String,
    pub completed: bool,
}

/// Pure TodoList state
#[derive(Debug, Clone, PartialEq)]
pub struct TodoList {
    pub todos: Vec<Todo>,
    pub next_id: usize,
    pub filter: Filter,
}

/// Filter options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Filter {
    All,
    Active,
    Completed,
}

/// Pure actions that can be performed on todos
#[derive(Debug, Clone)]
pub enum TodoAction {
    Add { title: String },
    Toggle { id: usize },
    Delete { id: usize },
    Edit { id: usize, title: String },
    SetFilter { filter: Filter },
    ClearCompleted,
}

impl TodoList {
    /// Create a new empty todo list
    pub fn new() -> Self {
        TodoList {
            todos: Vec::new(),
            next_id: 1,
            filter: Filter::All,
        }
    }
    
    /// Pure reducer function - takes state and action, returns new state
    pub fn reduce(self, action: TodoAction) -> TodoList {
        match action {
            TodoAction::Add { title } => self.add_todo(title),
            TodoAction::Toggle { id } => self.toggle_todo(id),
            TodoAction::Delete { id } => self.delete_todo(id),
            TodoAction::Edit { id, title } => self.edit_todo(id, title),
            TodoAction::SetFilter { filter } => self.set_filter(filter),
            TodoAction::ClearCompleted => self.clear_completed(),
        }
    }
    
    /// Add a new todo
    fn add_todo(mut self, title: String) -> TodoList {
        if !title.trim().is_empty() {
            self.todos.push(Todo {
                id: self.next_id,
                title: title.trim().to_string(),
                completed: false,
            });
            self.next_id += 1;
        }
        self
    }
    
    /// Toggle todo completion status
    fn toggle_todo(mut self, id: usize) -> TodoList {
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            todo.completed = !todo.completed;
        }
        self
    }
    
    /// Delete a todo
    fn delete_todo(mut self, id: usize) -> TodoList {
        self.todos.retain(|t| t.id != id);
        self
    }
    
    /// Edit todo title
    fn edit_todo(mut self, id: usize, title: String) -> TodoList {
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            todo.title = title.trim().to_string();
        }
        self
    }
    
    /// Set current filter
    fn set_filter(mut self, filter: Filter) -> TodoList {
        self.filter = filter;
        self
    }
    
    /// Clear all completed todos
    fn clear_completed(mut self) -> TodoList {
        self.todos.retain(|t| !t.completed);
        self
    }
    
    /// Get filtered todos (pure computation)
    pub fn visible_todos(&self) -> Vec<&Todo> {
        self.todos
            .iter()
            .filter(|todo| match self.filter {
                Filter::All => true,
                Filter::Active => !todo.completed,
                Filter::Completed => todo.completed,
            })
            .collect()
    }
    
    /// Count active todos
    pub fn active_count(&self) -> usize {
        self.todos.iter().filter(|t| !t.completed).count()
    }
    
    /// Check if any todos are completed
    pub fn has_completed(&self) -> bool {
        self.todos.iter().any(|t| t.completed)
    }
}

/// Pure validation functions
pub mod validation {
    /// Validate todo title
    pub fn validate_title(title: &str) -> Result<(), &'static str> {
        let trimmed = title.trim();
        if trimmed.is_empty() {
            Err("Todo title cannot be empty")
        } else if trimmed.len() > 200 {
            Err("Todo title is too long (max 200 characters)")
        } else {
            Ok(())
        }
    }
}

/// Pure serialization helpers
pub mod serialization {
    use super::*;
    
    /// Convert TodoList to JSON-serializable format
    pub fn to_storage_format(list: &TodoList) -> StorageFormat {
        StorageFormat {
            todos: list.todos.clone(),
            next_id: list.next_id,
        }
    }
    
    /// Restore TodoList from storage format
    pub fn from_storage_format(storage: StorageFormat) -> TodoList {
        TodoList {
            todos: storage.todos,
            next_id: storage.next_id,
            filter: Filter::All,
        }
    }
    
    #[derive(Serialize, Deserialize)]
    pub struct StorageFormat {
        pub todos: Vec<Todo>,
        pub next_id: usize,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_todo() {
        let list = TodoList::new();
        let list = list.reduce(TodoAction::Add {
            title: "Test todo".to_string(),
        });
        
        assert_eq!(list.todos.len(), 1);
        assert_eq!(list.todos[0].title, "Test todo");
        assert!(!list.todos[0].completed);
    }
    
    #[test]
    fn test_toggle_todo() {
        let mut list = TodoList::new();
        list = list.reduce(TodoAction::Add {
            title: "Test".to_string(),
        });
        
        let id = list.todos[0].id;
        list = list.reduce(TodoAction::Toggle { id });
        
        assert!(list.todos[0].completed);
    }
    
    #[test]
    fn test_filter_todos() {
        let mut list = TodoList::new();
        list = list.reduce(TodoAction::Add {
            title: "Active".to_string(),
        });
        list = list.reduce(TodoAction::Add {
            title: "Completed".to_string(),
        });
        
        let completed_id = list.todos[1].id;
        list = list.reduce(TodoAction::Toggle { id: completed_id });
        
        list = list.reduce(TodoAction::SetFilter {
            filter: Filter::Active,
        });
        
        let visible = list.visible_todos();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].title, "Active");
    }
}