use layer9_framework::prelude::*;
use layer9_framework::hooks::use_state;
use layer9_framework::reactive_v2::mount;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use uuid::Uuid;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct Todo {
    id: String,
    text: String,
    completed: bool,
    created_at: String,
}

impl Todo {
    fn new(text: String) -> Self {
        Todo {
            id: Uuid::new_v4().to_string(),
            text,
            completed: false,
            created_at: {
                // Use JavaScript Date for WASM compatibility
                let date = js_sys::Date::new_0();
                format!("{}-{:02}-{:02} {:02}:{:02}",
                    date.get_full_year(),
                    date.get_month() + 1,
                    date.get_date(),
                    date.get_hours(),
                    date.get_minutes()
                )
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Filter {
    All,
    Active,
    Completed,
}

struct TodoApp;

impl Component for TodoApp {
    fn render(&self) -> Element {
        let (todos, set_todos) = use_state(Vec::<Todo>::new());
        let (filter, set_filter) = use_state(Filter::All);

        // Calculate stats
        let active_count = todos.iter().filter(|t| !t.completed).count();
        let completed_count = todos.iter().filter(|t| t.completed).count();

        // Filter todos
        let filtered_todos: Vec<Todo> = todos
            .iter()
            .filter(|todo| match &filter {
                Filter::All => true,
                Filter::Active => !todo.completed,
                Filter::Completed => todo.completed,
            })
            .cloned()
            .collect();

        // Helper function to add todo from input
        let add_todo_handler = {
            let todos = todos.clone();
            let set_todos = set_todos.clone();
            Rc::new(move || {
                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();
                if let Some(input) = document.get_element_by_id("todo-input") {
                    if let Ok(input) = input.dyn_into::<web_sys::HtmlInputElement>() {
                        let text = input.value();
                        if !text.trim().is_empty() {
                            let mut new_todos = todos.clone();
                            new_todos.push(Todo::new(text.trim().to_string()));
                            set_todos(new_todos);
                            input.set_value("");
                        }
                    }
                }
            })
        };

        // Build UI elements
        let style_element = Element::Node {
            tag: "style".to_string(),
            props: Props::default(),
            children: vec![Element::Text(STYLES.to_string())],
        };

        let header = Element::Node {
            tag: "div".to_string(),
            props: Props {
                class: Some("header".to_string()),
                ..Default::default()
            },
            children: vec![
                Element::Node {
                    tag: "h1".to_string(),
                    props: Props::default(),
                    children: vec![Element::Text("Layer9 Todo".to_string())],
                },
                Element::Node {
                    tag: "p".to_string(),
                    props: Props {
                        class: Some("subtitle".to_string()),
                        ..Default::default()
                    },
                    children: vec![Element::Text("A beautiful todo app built with Rust and WASM".to_string())],
                },
            ],
        };

        let input_section = Element::Node {
            tag: "div".to_string(),
            props: Props {
                class: Some("input-section".to_string()),
                ..Default::default()
            },
            children: vec![
                Element::Node {
                    tag: "input".to_string(),
                    props: Props {
                        class: Some("todo-input".to_string()),
                        id: Some("todo-input".to_string()),
                        attributes: vec![
                            ("type".to_string(), "text".to_string()),
                            ("placeholder".to_string(), "What needs to be done?".to_string()),
                        ],
                        ..Default::default()
                    },
                    children: vec![],
                },
                Element::Node {
                    tag: "button".to_string(),
                    props: Props {
                        class: Some("add-btn".to_string()),
                        on_click: Some(add_todo_handler),
                        ..Default::default()
                    },
                    children: vec![Element::Text("Add".to_string())],
                },
            ],
        };

        let todos_container = if filtered_todos.is_empty() && todos.is_empty() {
            Element::Node {
                tag: "div".to_string(),
                props: Props {
                    class: Some("todos-container".to_string()),
                    ..Default::default()
                },
                children: vec![
                    Element::Node {
                        tag: "div".to_string(),
                        props: Props {
                            class: Some("empty-state".to_string()),
                            ..Default::default()
                        },
                        children: vec![
                            Element::Node {
                                tag: "p".to_string(),
                                props: Props::default(),
                                children: vec![Element::Text("No todos yet!".to_string())],
                            },
                            Element::Node {
                                tag: "p".to_string(),
                                props: Props {
                                    class: Some("hint".to_string()),
                                    ..Default::default()
                                },
                                children: vec![Element::Text("Add one above to get started".to_string())],
                            },
                        ],
                    },
                ],
            }
        } else if filtered_todos.is_empty() {
            Element::Node {
                tag: "div".to_string(),
                props: Props {
                    class: Some("todos-container".to_string()),
                    ..Default::default()
                },
                children: vec![
                    Element::Node {
                        tag: "div".to_string(),
                        props: Props {
                            class: Some("empty-state".to_string()),
                            ..Default::default()
                        },
                        children: vec![
                            Element::Node {
                                tag: "p".to_string(),
                                props: Props::default(),
                                children: vec![Element::Text(format!("No {} todos", filter_text(&filter)))],
                            },
                        ],
                    },
                ],
            }
        } else {
            let todo_items: Vec<Element> = filtered_todos.iter().map(|todo| {
                let todo_id = todo.id.clone();
                let is_completed = todo.completed;
                
                let set_todos_clone = set_todos.clone();
                let todos_clone = todos.clone();
                let todo_id_toggle = todo_id.clone();
                let todo_id_delete = todo_id.clone();
                
                // Clone for the second closure
                let set_todos_clone_delete = set_todos.clone();
                let todos_clone_delete = todos.clone();
                
                Element::Node {
                    tag: "li".to_string(),
                    props: Props {
                        class: Some(if is_completed { "todo-item completed".to_string() } else { "todo-item".to_string() }),
                        ..Default::default()
                    },
                    children: vec![
                        Element::Node {
                            tag: "div".to_string(),
                            props: Props {
                                class: Some("todo-content".to_string()),
                                ..Default::default()
                            },
                            children: vec![
                                Element::Node {
                                    tag: "input".to_string(),
                                    props: Props {
                                        class: Some("todo-checkbox".to_string()),
                                        attributes: vec![
                                            ("type".to_string(), "checkbox".to_string()),
                                            if is_completed { ("checked".to_string(), "checked".to_string()) } else { ("".to_string(), "".to_string()) },
                                        ].into_iter().filter(|(k, _)| !k.is_empty()).collect(),
                                        on_click: Some(Rc::new(move || {
                                            let mut new_todos = todos_clone.clone();
                                            if let Some(todo) = new_todos.iter_mut().find(|t| t.id == todo_id_toggle) {
                                                todo.completed = !todo.completed;
                                            }
                                            set_todos_clone(new_todos);
                                        })),
                                        ..Default::default()
                                    },
                                    children: vec![],
                                },
                                Element::Node {
                                    tag: "span".to_string(),
                                    props: Props {
                                        class: Some("todo-text".to_string()),
                                        ..Default::default()
                                    },
                                    children: vec![Element::Text(todo.text.clone())],
                                },
                                Element::Node {
                                    tag: "button".to_string(),
                                    props: Props {
                                        class: Some("delete-btn".to_string()),
                                        on_click: Some(Rc::new(move || {
                                            let new_todos = todos_clone_delete.iter()
                                                .filter(|t| t.id != todo_id_delete)
                                                .cloned()
                                                .collect();
                                            set_todos_clone_delete(new_todos);
                                        })),
                                        ..Default::default()
                                    },
                                    children: vec![Element::Text("×".to_string())],
                                },
                            ],
                        },
                        Element::Node {
                            tag: "div".to_string(),
                            props: Props {
                                class: Some("todo-meta".to_string()),
                                ..Default::default()
                            },
                            children: vec![Element::Text(todo.created_at.clone())],
                        },
                    ],
                }
            }).collect();

            Element::Node {
                tag: "div".to_string(),
                props: Props {
                    class: Some("todos-container".to_string()),
                    ..Default::default()
                },
                children: vec![
                    Element::Node {
                        tag: "ul".to_string(),
                        props: Props {
                            class: Some("todo-list".to_string()),
                            ..Default::default()
                        },
                        children: todo_items,
                    },
                ],
            }
        };

        let footer = if !todos.is_empty() {
            let set_filter_clone = set_filter.clone();
            let set_todos_clone = set_todos.clone();
            let todos_clone = todos.clone();
            
            Element::Node {
                tag: "div".to_string(),
                props: Props {
                    class: Some("footer".to_string()),
                    ..Default::default()
                },
                children: vec![
                    Element::Node {
                        tag: "div".to_string(),
                        props: Props {
                            class: Some("stats".to_string()),
                            ..Default::default()
                        },
                        children: vec![
                            Element::Node {
                                tag: "span".to_string(),
                                props: Props::default(),
                                children: vec![Element::Text(format!("{} active", active_count))],
                            },
                            Element::Node {
                                tag: "span".to_string(),
                                props: Props {
                                    class: Some("dot".to_string()),
                                    ..Default::default()
                                },
                                children: vec![Element::Text("•".to_string())],
                            },
                            Element::Node {
                                tag: "span".to_string(),
                                props: Props::default(),
                                children: vec![Element::Text(format!("{} completed", completed_count))],
                            },
                        ],
                    },
                    Element::Node {
                        tag: "div".to_string(),
                        props: Props {
                            class: Some("filters".to_string()),
                            ..Default::default()
                        },
                        children: vec![
                            create_filter_button(&filter, &set_filter_clone, Filter::All),
                            create_filter_button(&filter, &set_filter_clone, Filter::Active),
                            create_filter_button(&filter, &set_filter_clone, Filter::Completed),
                        ],
                    },
                    if completed_count > 0 {
                        Element::Node {
                            tag: "button".to_string(),
                            props: Props {
                                class: Some("clear-btn".to_string()),
                                on_click: Some(Rc::new(move || {
                                    let new_todos = todos_clone.iter()
                                        .filter(|t| !t.completed)
                                        .cloned()
                                        .collect();
                                    set_todos_clone(new_todos);
                                })),
                                ..Default::default()
                            },
                            children: vec![Element::Text("Clear completed".to_string())],
                        }
                    } else {
                        Element::Node {
                            tag: "div".to_string(),
                            props: Props::default(),
                            children: vec![],
                        }
                    },
                ],
            }
        } else {
            Element::Node {
                tag: "div".to_string(),
                props: Props::default(),
                children: vec![],
            }
        };

        let info = Element::Node {
            tag: "div".to_string(),
            props: Props {
                class: Some("info".to_string()),
                ..Default::default()
            },
            children: vec![
                Element::Node {
                    tag: "p".to_string(),
                    props: Props::default(),
                    children: vec![
                        Element::Text("Built with ".to_string()),
                        Element::Node {
                            tag: "a".to_string(),
                            props: Props {
                                attributes: vec![
                                    ("href".to_string(), "https://github.com/anthropics/layer9".to_string()),
                                    ("target".to_string(), "_blank".to_string()),
                                ],
                                ..Default::default()
                            },
                            children: vec![Element::Text("Layer9".to_string())],
                        },
                        Element::Text(" - A Reactive Rust Web Framework".to_string()),
                    ],
                },
            ],
        };

        // Main app container
        Element::Node {
            tag: "div".to_string(),
            props: Props {
                class: Some("todo-app".to_string()),
                ..Default::default()
            },
            children: vec![
                style_element,
                header,
                input_section,
                todos_container,
                footer,
                info,
            ],
        }
    }
}

fn create_filter_button(current_filter: &Filter, set_filter: &(impl Fn(Filter) + Clone + 'static), target_filter: Filter) -> Element {
    let is_active = current_filter == &target_filter;
    let set_filter_clone = set_filter.clone();
    let target_filter_clone = target_filter.clone();
    
    Element::Node {
        tag: "button".to_string(),
        props: Props {
            class: Some(if is_active { "filter-btn active".to_string() } else { "filter-btn".to_string() }),
            on_click: Some(Rc::new(move || {
                set_filter_clone(target_filter_clone.clone());
            })),
            ..Default::default()
        },
        children: vec![Element::Text(filter_text(&target_filter).to_string())],
    }
}

fn filter_text(filter: &Filter) -> &str {
    match filter {
        Filter::All => "all",
        Filter::Active => "active",
        Filter::Completed => "completed",
    }
}

const STYLES: &str = r#"
    :root {
        --primary: #6366f1;
        --primary-dark: #4f46e5;
        --danger: #ef4444;
        --success: #10b981;
        --gray-50: #fafafa;
        --gray-100: #f3f4f6;
        --gray-200: #e5e7eb;
        --gray-300: #d1d5db;
        --gray-400: #9ca3af;
        --gray-500: #6b7280;
        --gray-600: #4b5563;
        --gray-700: #374151;
        --gray-800: #1f2937;
        --gray-900: #111827;
    }
    
    * {
        box-sizing: border-box;
        margin: 0;
        padding: 0;
    }
    
    body {
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        min-height: 100vh;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 20px;
    }
    
    .todo-app {
        background: white;
        border-radius: 20px;
        box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
        width: 100%;
        max-width: 600px;
        overflow: hidden;
    }
    
    .header {
        background: linear-gradient(135deg, var(--primary) 0%, var(--primary-dark) 100%);
        color: white;
        padding: 30px;
        text-align: center;
    }
    
    .header h1 {
        font-size: 2.5rem;
        font-weight: 700;
        margin-bottom: 8px;
    }
    
    .subtitle {
        opacity: 0.9;
        font-size: 1rem;
    }
    
    .input-section {
        display: flex;
        padding: 20px;
        background: var(--gray-100);
        border-bottom: 1px solid var(--gray-200);
        gap: 10px;
    }
    
    .todo-input {
        flex: 1;
        padding: 15px;
        font-size: 1.1rem;
        border: 2px solid transparent;
        border-radius: 10px;
        outline: none;
        transition: border-color 0.3s;
    }
    
    .todo-input:focus {
        border-color: var(--primary);
    }
    
    .add-btn {
        padding: 15px 25px;
        background: var(--primary);
        color: white;
        border: none;
        border-radius: 10px;
        font-size: 1rem;
        font-weight: 600;
        cursor: pointer;
        transition: background-color 0.3s;
    }
    
    .add-btn:hover {
        background: var(--primary-dark);
    }
    
    .todos-container {
        min-height: 200px;
        max-height: 400px;
        overflow-y: auto;
    }
    
    .empty-state {
        padding: 60px 20px;
        text-align: center;
        color: var(--gray-400);
    }
    
    .empty-state p {
        font-size: 1.2rem;
        margin-bottom: 8px;
    }
    
    .hint {
        font-size: 0.9rem;
        opacity: 0.8;
    }
    
    .todo-list {
        list-style: none;
    }
    
    .todo-item {
        border-bottom: 1px solid var(--gray-100);
        transition: background-color 0.3s;
    }
    
    .todo-item:hover {
        background-color: var(--gray-50);
    }
    
    .todo-content {
        display: flex;
        align-items: center;
        padding: 15px 20px;
    }
    
    .todo-checkbox {
        width: 20px;
        height: 20px;
        margin-right: 15px;
        cursor: pointer;
        accent-color: var(--primary);
    }
    
    .todo-text {
        flex: 1;
        font-size: 1.1rem;
        color: var(--gray-800);
        word-break: break-word;
    }
    
    .todo-item.completed .todo-text {
        text-decoration: line-through;
        color: var(--gray-400);
    }
    
    .delete-btn {
        background: none;
        border: none;
        font-size: 1.5rem;
        color: var(--gray-400);
        cursor: pointer;
        padding: 0 10px;
        opacity: 0;
        transition: opacity 0.3s, color 0.3s;
    }
    
    .todo-item:hover .delete-btn {
        opacity: 1;
    }
    
    .delete-btn:hover {
        color: var(--danger);
    }
    
    .todo-meta {
        padding: 0 20px 10px 50px;
        font-size: 0.8rem;
        color: var(--gray-400);
    }
    
    .footer {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 20px;
        background: var(--gray-100);
        border-top: 1px solid var(--gray-200);
        flex-wrap: wrap;
        gap: 10px;
    }
    
    .stats {
        color: var(--gray-600);
        font-size: 0.9rem;
    }
    
    .dot {
        margin: 0 8px;
        opacity: 0.5;
    }
    
    .filters {
        display: flex;
        gap: 5px;
    }
    
    .filter-btn {
        background: none;
        border: 2px solid transparent;
        padding: 5px 12px;
        border-radius: 5px;
        cursor: pointer;
        font-size: 0.9rem;
        color: var(--gray-600);
        transition: all 0.3s;
    }
    
    .filter-btn:hover {
        border-color: var(--gray-300);
    }
    
    .filter-btn.active {
        border-color: var(--primary);
        color: var(--primary);
    }
    
    .clear-btn {
        background: none;
        border: none;
        color: var(--danger);
        cursor: pointer;
        font-size: 0.9rem;
        transition: opacity 0.3s;
    }
    
    .clear-btn:hover {
        opacity: 0.8;
    }
    
    .info {
        padding: 20px;
        text-align: center;
        font-size: 0.8rem;
        color: var(--gray-400);
    }
    
    .info p {
        margin: 5px 0;
    }
    
    .info a {
        color: var(--primary);
        text-decoration: none;
    }
    
    .info a:hover {
        text-decoration: underline;
    }
    
    /* Scrollbar styling */
    .todos-container::-webkit-scrollbar {
        width: 6px;
    }
    
    .todos-container::-webkit-scrollbar-track {
        background: var(--gray-100);
    }
    
    .todos-container::-webkit-scrollbar-thumb {
        background: var(--gray-300);
        border-radius: 3px;
    }
    
    .todos-container::-webkit-scrollbar-thumb:hover {
        background: var(--gray-400);
    }
"#;

#[wasm_bindgen(start)]
pub fn main() {
    web_sys::console::log_1(&"Layer9 Todo App starting...".into());
    mount(Box::new(TodoApp), "root");
    web_sys::console::log_1(&"Layer9 Todo App mounted successfully!".into());
}