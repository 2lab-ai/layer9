use layer9_framework::prelude::*;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct SimpleTodoApp;

impl Component for SimpleTodoApp {
    fn render(&self) -> Element {
        let (todos, set_todos) = use_state(vec![
            "Learn Rust".to_string(),
            "Build with Layer9".to_string(),
            "Create awesome apps".to_string(),
        ]);
        
        let (new_todo, set_new_todo) = use_state(String::new());
        
        let add_todo = {
            let todos = todos.clone();
            let set_todos = set_todos.clone();
            let new_todo = new_todo.clone();
            let set_new_todo = set_new_todo.clone();
            move || {
                if !new_todo.trim().is_empty() {
                    let mut updated_todos = todos.clone();
                    updated_todos.push(new_todo.trim().to_string());
                    set_todos(updated_todos);
                    set_new_todo(String::new());
                }
            }
        };
        
        let todo_items: Vec<Element> = todos.iter().enumerate().map(|(index, todo)| {
            let todos_clone = todos.clone();
            let set_todos_clone = set_todos.clone();
            Element::Node {
                tag: "li".to_string(),
                props: Props {
                    class: Some("todo-item".to_string()),
                    ..Default::default()
                },
                children: vec![
                    Element::Text(todo.clone()),
                    Element::Node {
                        tag: "button".to_string(),
                        props: Props {
                            class: Some("delete-btn".to_string()),
                            on_click: Some(Rc::new(move || {
                                let mut updated_todos = todos_clone.clone();
                                updated_todos.remove(index);
                                set_todos_clone(updated_todos);
                            })),
                            ..Default::default()
                        },
                        children: vec![Element::Text("×".to_string())],
                    },
                ],
            }
        }).collect();
        
        Element::Node {
            tag: "div".to_string(),
            props: Props {
                class: Some("simple-todo-app".to_string()),
                ..Default::default()
            },
            children: vec![
                Element::Node {
                    tag: "style".to_string(),
                    props: Props::default(),
                    children: vec![Element::Text(SIMPLE_STYLES.to_string())],
                },
                Element::Node {
                    tag: "h1".to_string(),
                    props: Props::default(),
                    children: vec![Element::Text("Simple Layer9 Todo".to_string())],
                },
                Element::Node {
                    tag: "div".to_string(),
                    props: Props {
                        class: Some("input-container".to_string()),
                        ..Default::default()
                    },
                    children: vec![
                        Element::Node {
                            tag: "input".to_string(),
                            props: Props {
                                class: Some("todo-input".to_string()),
                                id: Some("todo-input".to_string()),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                        Element::Node {
                            tag: "button".to_string(),
                            props: Props {
                                class: Some("add-btn".to_string()),
                                on_click: Some(Rc::new(add_todo)),
                                ..Default::default()
                            },
                            children: vec![Element::Text("Add Todo".to_string())],
                        },
                    ],
                },
                Element::Node {
                    tag: "ul".to_string(),
                    props: Props {
                        class: Some("todo-list".to_string()),
                        ..Default::default()
                    },
                    children: todo_items,
                },
                Element::Node {
                    tag: "p".to_string(),
                    props: Props {
                        class: Some("count".to_string()),
                        ..Default::default()
                    },
                    children: vec![Element::Text(format!("Total todos: {}", todos.len()))],
                },
            ],
        }
    }
}

const SIMPLE_STYLES: &str = r#"
    body {
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
        background: #f5f5f5;
        padding: 20px;
    }
    
    .simple-todo-app {
        max-width: 500px;
        margin: 0 auto;
        background: white;
        padding: 30px;
        border-radius: 10px;
        box-shadow: 0 2px 10px rgba(0,0,0,0.1);
    }
    
    h1 {
        color: #333;
        margin-bottom: 20px;
    }
    
    .input-container {
        display: flex;
        margin-bottom: 20px;
        gap: 10px;
    }
    
    .todo-input {
        flex: 1;
        padding: 10px;
        border: 2px solid #e0e0e0;
        border-radius: 5px;
        font-size: 16px;
    }
    
    .add-btn {
        padding: 10px 20px;
        background: #4CAF50;
        color: white;
        border: none;
        border-radius: 5px;
        cursor: pointer;
        font-size: 16px;
    }
    
    .add-btn:hover {
        background: #45a049;
    }
    
    .todo-list {
        list-style: none;
        padding: 0;
    }
    
    .todo-item {
        padding: 10px;
        border-bottom: 1px solid #eee;
        display: flex;
        justify-content: space-between;
        align-items: center;
    }
    
    .delete-btn {
        background: #f44336;
        color: white;
        border: none;
        padding: 5px 10px;
        border-radius: 3px;
        cursor: pointer;
    }
    
    .delete-btn:hover {
        background: #d32f2f;
    }
    
    .count {
        margin-top: 20px;
        color: #666;
        text-align: center;
    }
"#;

#[wasm_bindgen(start)]
pub fn main() {
    web_sys::console::log_1(&"Simple Layer9 Todo starting...".into());
    mount(Box::new(SimpleTodoApp), "root");
    web_sys::console::log_1(&"Simple Layer9 Todo mounted!".into());
}