//! Layer 3: UI - Todo user interface
//! 
//! This layer provides the user-facing components and handles external interactions.
//! It depends on L2 and L1 but nothing depends on it.

use crate::l1_domain::{Filter, Todo, TodoAction, TodoList};
use crate::l2_runtime::{TodoStore, todo_runtime};
use layer9_core::haf::{
    Component, VNode, Props,
    layers::{L1, L2, L3},
    haf_component,
};
use layer9_core::prelude::*;
use std::rc::Rc;

/// Main Todo App component
pub struct TodoApp {
    store: Rc<TodoStore>,
}

impl TodoApp {
    pub fn new() -> Self {
        TodoApp {
            store: todo_runtime::create_store(),
        }
    }
}

// Define components using HAF macro
haf_component!(L3, TodoHeader, HeaderProps, {
    VNode::Element {
        tag: "header".to_string(),
        props: Props {
            attributes: vec![("class".to_string(), "header".to_string())],
        },
        children: vec![
            VNode::Element {
                tag: "h1".to_string(),
                props: Props::default(),
                children: vec![VNode::Text("todos".to_string())],
            },
            VNode::Element {
                tag: "input".to_string(),
                props: Props {
                    attributes: vec![
                        ("class".to_string(), "new-todo".to_string()),
                        ("placeholder".to_string(), "What needs to be done?".to_string()),
                        ("autofocus".to_string(), "true".to_string()),
                    ],
                },
                children: vec![],
            },
        ],
    }
});

#[derive(Clone)]
pub struct HeaderProps {
    on_add: Rc<dyn Fn(String)>,
}

haf_component!(L3, TodoItem, ItemProps, {
    let todo = &props.todo;
    let class = if todo.completed {
        "completed"
    } else {
        ""
    };
    
    VNode::Element {
        tag: "li".to_string(),
        props: Props {
            attributes: vec![("class".to_string(), class.to_string())],
        },
        children: vec![
            VNode::Element {
                tag: "div".to_string(),
                props: Props {
                    attributes: vec![("class".to_string(), "view".to_string())],
                },
                children: vec![
                    VNode::Element {
                        tag: "input".to_string(),
                        props: Props {
                            attributes: vec![
                                ("class".to_string(), "toggle".to_string()),
                                ("type".to_string(), "checkbox".to_string()),
                                ("checked".to_string(), todo.completed.to_string()),
                            ],
                        },
                        children: vec![],
                    },
                    VNode::Element {
                        tag: "label".to_string(),
                        props: Props::default(),
                        children: vec![VNode::Text(todo.title.clone())],
                    },
                    VNode::Element {
                        tag: "button".to_string(),
                        props: Props {
                            attributes: vec![("class".to_string(), "destroy".to_string())],
                        },
                        children: vec![],
                    },
                ],
            },
        ],
    }
});

#[derive(Clone)]
pub struct ItemProps {
    todo: Todo,
    on_toggle: Rc<dyn Fn(usize)>,
    on_delete: Rc<dyn Fn(usize)>,
}

haf_component!(L3, TodoFooter, FooterProps, {
    let active_count = props.active_count;
    let has_completed = props.has_completed;
    let current_filter = props.current_filter;
    
    VNode::Element {
        tag: "footer".to_string(),
        props: Props {
            attributes: vec![("class".to_string(), "footer".to_string())],
        },
        children: vec![
            // Item count
            VNode::Element {
                tag: "span".to_string(),
                props: Props {
                    attributes: vec![("class".to_string(), "todo-count".to_string())],
                },
                children: vec![
                    VNode::Element {
                        tag: "strong".to_string(),
                        props: Props::default(),
                        children: vec![VNode::Text(active_count.to_string())],
                    },
                    VNode::Text(format!(" {} left", if active_count == 1 { "item" } else { "items" })),
                ],
            },
            // Filters
            VNode::Element {
                tag: "ul".to_string(),
                props: Props {
                    attributes: vec![("class".to_string(), "filters".to_string())],
                },
                children: vec![
                    filter_link(Filter::All, current_filter),
                    filter_link(Filter::Active, current_filter),
                    filter_link(Filter::Completed, current_filter),
                ],
            },
            // Clear completed button
            if has_completed {
                VNode::Element {
                    tag: "button".to_string(),
                    props: Props {
                        attributes: vec![("class".to_string(), "clear-completed".to_string())],
                    },
                    children: vec![VNode::Text("Clear completed".to_string())],
                }
            } else {
                VNode::Text("".to_string())
            },
        ],
    }
});

#[derive(Clone)]
pub struct FooterProps {
    active_count: usize,
    has_completed: bool,
    current_filter: Filter,
    on_filter_change: Rc<dyn Fn(Filter)>,
    on_clear_completed: Rc<dyn Fn()>,
}

fn filter_link(filter: Filter, current: Filter) -> VNode {
    let (text, href) = match filter {
        Filter::All => ("All", "#/"),
        Filter::Active => ("Active", "#/active"),
        Filter::Completed => ("Completed", "#/completed"),
    };
    
    let class = if filter == current {
        "selected"
    } else {
        ""
    };
    
    VNode::Element {
        tag: "li".to_string(),
        props: Props::default(),
        children: vec![
            VNode::Element {
                tag: "a".to_string(),
                props: Props {
                    attributes: vec![
                        ("class".to_string(), class.to_string()),
                        ("href".to_string(), href.to_string()),
                    ],
                },
                children: vec![VNode::Text(text.to_string())],
            },
        ],
    }
}

/// Main app render function
pub fn render_app(store: &TodoStore) -> VNode {
    let state = store.get_state();
    let visible_todos = state.visible_todos();
    
    VNode::Element {
        tag: "section".to_string(),
        props: Props {
            attributes: vec![("class".to_string(), "todoapp".to_string())],
        },
        children: vec![
            // Header
            TodoHeader(HeaderProps {
                on_add: Rc::new(move |title| {
                    todo_runtime::dispatch_action(
                        store,
                        TodoAction::Add { title }
                    );
                }),
            }).inner.render(),
            
            // Main section
            if !state.todos.is_empty() {
                VNode::Element {
                    tag: "section".to_string(),
                    props: Props {
                        attributes: vec![("class".to_string(), "main".to_string())],
                    },
                    children: vec![
                        // Toggle all
                        VNode::Element {
                            tag: "input".to_string(),
                            props: Props {
                                attributes: vec![
                                    ("id".to_string(), "toggle-all".to_string()),
                                    ("class".to_string(), "toggle-all".to_string()),
                                    ("type".to_string(), "checkbox".to_string()),
                                ],
                            },
                            children: vec![],
                        },
                        VNode::Element {
                            tag: "label".to_string(),
                            props: Props {
                                attributes: vec![("for".to_string(), "toggle-all".to_string())],
                            },
                            children: vec![VNode::Text("Mark all as complete".to_string())],
                        },
                        // Todo list
                        VNode::Element {
                            tag: "ul".to_string(),
                            props: Props {
                                attributes: vec![("class".to_string(), "todo-list".to_string())],
                            },
                            children: visible_todos.into_iter().map(|todo| {
                                TodoItem(ItemProps {
                                    todo: todo.clone(),
                                    on_toggle: Rc::new(move |id| {
                                        todo_runtime::dispatch_action(
                                            store,
                                            TodoAction::Toggle { id }
                                        );
                                    }),
                                    on_delete: Rc::new(move |id| {
                                        todo_runtime::dispatch_action(
                                            store,
                                            TodoAction::Delete { id }
                                        );
                                    }),
                                }).inner.render()
                            }).collect(),
                        },
                    ],
                }
            } else {
                VNode::Text("".to_string())
            },
            
            // Footer
            if !state.todos.is_empty() {
                TodoFooter(FooterProps {
                    active_count: state.active_count(),
                    has_completed: state.has_completed(),
                    current_filter: state.filter,
                    on_filter_change: Rc::new(move |filter| {
                        todo_runtime::dispatch_action(
                            store,
                            TodoAction::SetFilter { filter }
                        );
                    }),
                    on_clear_completed: Rc::new(move || {
                        todo_runtime::dispatch_action(
                            store,
                            TodoAction::ClearCompleted
                        );
                    }),
                }).inner.render()
            } else {
                VNode::Text("".to_string())
            },
        ],
    }
}

/// Style definitions (would normally be in a separate CSS file)
pub fn get_styles() -> &'static str {
    r#"
    body {
        font: 14px 'Helvetica Neue', Helvetica, Arial, sans-serif;
        background: #f5f5f5;
        color: #111111;
        margin: 0;
        padding: 0;
    }

    .todoapp {
        background: #fff;
        margin: 130px 0 40px 0;
        position: relative;
        box-shadow: 0 2px 4px 0 rgba(0, 0, 0, 0.2),
                    0 25px 50px 0 rgba(0, 0, 0, 0.1);
    }

    .todoapp h1 {
        position: absolute;
        top: -140px;
        width: 100%;
        font-size: 80px;
        font-weight: 200;
        text-align: center;
        color: #b83f45;
        text-rendering: optimizeLegibility;
    }

    .new-todo {
        position: relative;
        margin: 0;
        width: 100%;
        font-size: 24px;
        font-family: inherit;
        font-weight: inherit;
        line-height: 1.4em;
        color: inherit;
        padding: 6px;
        border: 1px solid #999;
        box-shadow: inset 0 -1px 5px 0 rgba(0, 0, 0, 0.2);
        box-sizing: border-box;
    }

    .todo-list {
        margin: 0;
        padding: 0;
        list-style: none;
    }

    .todo-list li {
        position: relative;
        font-size: 24px;
        border-bottom: 1px solid #ededed;
    }

    .todo-list li.completed label {
        color: #a9a9a9;
        text-decoration: line-through;
    }

    .footer {
        padding: 10px 15px;
        height: 20px;
        text-align: center;
        font-size: 15px;
        border-top: 1px solid #e6e6e6;
    }

    .filters {
        margin: 0;
        padding: 0;
        list-style: none;
        position: absolute;
        right: 0;
        left: 0;
    }

    .filters li {
        display: inline;
    }

    .filters li a {
        color: inherit;
        margin: 3px;
        padding: 3px 7px;
        text-decoration: none;
        border: 1px solid transparent;
        border-radius: 3px;
    }

    .filters li a.selected {
        border-color: #b83f45;
    }
    "#
}