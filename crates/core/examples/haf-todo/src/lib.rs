//! L1: Core - Pure business logic for TODO app

use layer9_core::haf::{
    layers::L1,
    component::{VNode, VProps, PureComponent},
};

#[derive(Clone, Debug, PartialEq)]
pub struct Todo {
    pub id: u32,
    pub text: String,
    pub completed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TodoListState {
    pub todos: Vec<Todo>,
    pub filter: Filter,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Filter {
    All,
    Active,
    Completed,
}

impl TodoListState {
    pub fn new() -> Self {
        Self {
            todos: vec![],
            filter: Filter::All,
        }
    }
    
    pub fn add_todo(&self, text: String) -> Self {
        let mut todos = self.todos.clone();
        todos.push(Todo {
            id: self.todos.len() as u32 + 1,
            text,
            completed: false,
        });
        
        Self {
            todos,
            filter: self.filter.clone(),
        }
    }
    
    pub fn toggle_todo(&self, id: u32) -> Self {
        let todos = self.todos.iter()
            .map(|todo| {
                if todo.id == id {
                    Todo {
                        id: todo.id,
                        text: todo.text.clone(),
                        completed: !todo.completed,
                    }
                } else {
                    todo.clone()
                }
            })
            .collect();
            
        Self {
            todos,
            filter: self.filter.clone(),
        }
    }
    
    pub fn visible_todos(&self) -> Vec<&Todo> {
        self.todos.iter()
            .filter(|todo| match self.filter {
                Filter::All => true,
                Filter::Active => !todo.completed,
                Filter::Completed => todo.completed,
            })
            .collect()
    }
}
