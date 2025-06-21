//! Full HAF Layer9 Application Template
//! 
//! Complete example with routing, state management, and services following HAF principles.

use layer9::prelude::*;
use layer9::haf::{layers::*, component::*, vdom::*, Contract, Service};
use std::collections::HashMap;

// ==================== L1: Pure Business Logic ====================

/// Application routes - pure data
#[derive(Clone, Debug, PartialEq)]
pub enum Route {
    Home,
    About,
    Todo,
    NotFound,
}

impl Route {
    /// Parse route from path string
    pub fn from_path(path: &str) -> Self {
        match path {
            "/" | "/home" => Route::Home,
            "/about" => Route::About,
            "/todo" => Route::Todo,
            _ => Route::NotFound,
        }
    }
    
    /// Get path string for route
    pub fn to_path(&self) -> &'static str {
        match self {
            Route::Home => "/",
            Route::About => "/about",
            Route::Todo => "/todo",
            Route::NotFound => "/404",
        }
    }
}

/// Todo item - pure data
#[derive(Clone, Debug, PartialEq)]
pub struct Todo {
    pub id: u32,
    pub text: String,
    pub completed: bool,
}

/// Application state - pure data
#[derive(Clone, Debug)]
pub struct AppState {
    pub route: Route,
    pub todos: Vec<Todo>,
    pub next_todo_id: u32,
    pub theme: Theme,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Theme {
    Light,
    Dark,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            route: Route::Home,
            todos: vec![
                Todo { id: 1, text: "Learn HAF architecture".to_string(), completed: true },
                Todo { id: 2, text: "Build with Layer9".to_string(), completed: false },
            ],
            next_todo_id: 3,
            theme: Theme::Dark,
        }
    }
}

/// Pure state transitions
pub mod transitions {
    use super::*;
    
    pub fn navigate(state: &AppState, route: Route) -> AppState {
        AppState {
            route,
            ..state.clone()
        }
    }
    
    pub fn add_todo(state: &AppState, text: String) -> AppState {
        let mut todos = state.todos.clone();
        todos.push(Todo {
            id: state.next_todo_id,
            text,
            completed: false,
        });
        
        AppState {
            todos,
            next_todo_id: state.next_todo_id + 1,
            ..state.clone()
        }
    }
    
    pub fn toggle_todo(state: &AppState, id: u32) -> AppState {
        let todos = state.todos.iter().map(|todo| {
            if todo.id == id {
                Todo {
                    completed: !todo.completed,
                    ..todo.clone()
                }
            } else {
                todo.clone()
            }
        }).collect();
        
        AppState {
            todos,
            ..state.clone()
        }
    }
    
    pub fn delete_todo(state: &AppState, id: u32) -> AppState {
        let todos = state.todos.iter()
            .filter(|todo| todo.id != id)
            .cloned()
            .collect();
        
        AppState {
            todos,
            ..state.clone()
        }
    }
    
    pub fn toggle_theme(state: &AppState) -> AppState {
        AppState {
            theme: match state.theme {
                Theme::Light => Theme::Dark,
                Theme::Dark => Theme::Light,
            },
            ..state.clone()
        }
    }
}

/// Pure components
pub mod components {
    use super::*;
    
    /// Navigation component
    pub struct Nav;
    
    impl Default for Nav {
        fn default() -> Self { Self }
    }
    
    impl PureComponent<L1> for Nav {
        type Props = Route;
        
        fn render(&self, current_route: &Self::Props) -> VNode<L1> {
            VNode::Element {
                tag: "nav".to_string(),
                props: VProps {
                    class: Some("nav".to_string()),
                    ..Default::default()
                },
                children: vec![
                    nav_link("Home", Route::Home, current_route),
                    nav_link("About", Route::About, current_route),
                    nav_link("Todo", Route::Todo, current_route),
                ],
            }
        }
    }
    
    fn nav_link(text: &str, route: Route, current: &Route) -> VNode<L1> {
        VNode::Element {
            tag: "a".to_string(),
            props: VProps {
                class: Some(if &route == current { "active" } else { "" }.to_string()),
                attributes: vec![("href".to_string(), route.to_path().to_string())],
                events: vec![("click".to_string(), EventId(100 + route as u64))],
                ..Default::default()
            },
            children: vec![VNode::Text(text.to_string())],
        }
    }
    
    /// Home page component
    pub struct HomePage;
    
    impl Default for HomePage {
        fn default() -> Self { Self }
    }
    
    impl PureComponent<L1> for HomePage {
        type Props = Theme;
        
        fn render(&self, theme: &Self::Props) -> VNode<L1> {
            VNode::Element {
                tag: "div".to_string(),
                props: VProps {
                    class: Some("page home".to_string()),
                    ..Default::default()
                },
                children: vec![
                    VNode::Element {
                        tag: "h1".to_string(),
                        props: VProps::default(),
                        children: vec![VNode::Text("Welcome to HAF Layer9".to_string())],
                    },
                    VNode::Element {
                        tag: "p".to_string(),
                        props: VProps::default(),
                        children: vec![VNode::Text(
                            "A hierarchical architecture framework for building robust web applications.".to_string()
                        )],
                    },
                    VNode::Element {
                        tag: "button".to_string(),
                        props: VProps {
                            events: vec![("click".to_string(), EventId(200))],
                            ..Default::default()
                        },
                        children: vec![VNode::Text(
                            format!("Switch to {} theme", 
                                match theme {
                                    Theme::Light => "dark",
                                    Theme::Dark => "light",
                                }
                            )
                        )],
                    },
                ],
            }
        }
    }
    
    /// Todo page component
    pub struct TodoPage;
    
    impl Default for TodoPage {
        fn default() -> Self { Self }
    }
    
    impl PureComponent<L1> for TodoPage {
        type Props = Vec<Todo>;
        
        fn render(&self, todos: &Self::Props) -> VNode<L1> {
            VNode::Element {
                tag: "div".to_string(),
                props: VProps {
                    class: Some("page todo".to_string()),
                    ..Default::default()
                },
                children: vec![
                    VNode::Element {
                        tag: "h1".to_string(),
                        props: VProps::default(),
                        children: vec![VNode::Text("Todo List".to_string())],
                    },
                    VNode::Element {
                        tag: "form".to_string(),
                        props: VProps {
                            events: vec![("submit".to_string(), EventId(300))],
                            ..Default::default()
                        },
                        children: vec![
                            VNode::Element {
                                tag: "input".to_string(),
                                props: VProps {
                                    attributes: vec![
                                        ("type".to_string(), "text".to_string()),
                                        ("placeholder".to_string(), "Add a new todo...".to_string()),
                                    ],
                                    ..Default::default()
                                },
                                children: vec![],
                            },
                            VNode::Element {
                                tag: "button".to_string(),
                                props: VProps {
                                    attributes: vec![("type".to_string(), "submit".to_string())],
                                    ..Default::default()
                                },
                                children: vec![VNode::Text("Add".to_string())],
                            },
                        ],
                    },
                    VNode::Element {
                        tag: "ul".to_string(),
                        props: VProps {
                            class: Some("todo-list".to_string()),
                            ..Default::default()
                        },
                        children: todos.iter().map(|todo| {
                            VNode::Element {
                                tag: "li".to_string(),
                                props: VProps {
                                    class: Some(if todo.completed { "completed" } else { "" }.to_string()),
                                    ..Default::default()
                                },
                                children: vec![
                                    VNode::Element {
                                        tag: "input".to_string(),
                                        props: VProps {
                                            attributes: vec![
                                                ("type".to_string(), "checkbox".to_string()),
                                                ("checked".to_string(), todo.completed.to_string()),
                                            ],
                                            events: vec![("change".to_string(), EventId(400 + todo.id as u64))],
                                            ..Default::default()
                                        },
                                        children: vec![],
                                    },
                                    VNode::Element {
                                        tag: "span".to_string(),
                                        props: VProps::default(),
                                        children: vec![VNode::Text(todo.text.clone())],
                                    },
                                    VNode::Element {
                                        tag: "button".to_string(),
                                        props: VProps {
                                            events: vec![("click".to_string(), EventId(500 + todo.id as u64))],
                                            ..Default::default()
                                        },
                                        children: vec![VNode::Text("Delete".to_string())],
                                    },
                                ],
                            }
                        }).collect(),
                    },
                ],
            }
        }
    }
}

// ==================== L2: Runtime Layer ====================

/// Application runtime
pub struct AppRuntime {
    state: AppState,
    component_runtime: ComponentRuntime<L2>,
    patch_runtime: PatchRuntime<L2>,
    event_handlers: HashMap<EventId, Box<dyn Fn() -> Action>>,
}

/// Actions that can modify state
#[derive(Clone, Debug)]
pub enum Action {
    Navigate(Route),
    AddTodo(String),
    ToggleTodo(u32),
    DeleteTodo(u32),
    ToggleTheme,
}

impl AppRuntime {
    pub fn new() -> Self {
        let mut runtime = Self {
            state: AppState::default(),
            component_runtime: ComponentRuntime::new(),
            patch_runtime: PatchRuntime::new(),
            event_handlers: HashMap::new(),
        };
        
        runtime.setup_event_handlers();
        runtime
    }
    
    /// Handle actions
    pub fn dispatch(&mut self, action: Action) -> Vec<Contract<Patch<L1>, DomOperation>> {
        use transitions::*;
        
        // Calculate new state
        let new_state = match action {
            Action::Navigate(route) => navigate(&self.state, route),
            Action::AddTodo(text) => add_todo(&self.state, text),
            Action::ToggleTodo(id) => toggle_todo(&self.state, id),
            Action::DeleteTodo(id) => delete_todo(&self.state, id),
            Action::ToggleTheme => toggle_theme(&self.state),
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
        use components::*;
        
        VNode::Element {
            tag: "div".to_string(),
            props: VProps {
                class: Some(format!("app theme-{}", 
                    match state.theme {
                        Theme::Light => "light",
                        Theme::Dark => "dark",
                    }
                )),
                ..Default::default()
            },
            children: vec![
                Nav::default().render(&state.route),
                match &state.route {
                    Route::Home => HomePage::default().render(&state.theme),
                    Route::About => VNode::Element {
                        tag: "div".to_string(),
                        props: VProps {
                            class: Some("page about".to_string()),
                            ..Default::default()
                        },
                        children: vec![
                            VNode::Element {
                                tag: "h1".to_string(),
                                props: VProps::default(),
                                children: vec![VNode::Text("About HAF".to_string())],
                            },
                            VNode::Element {
                                tag: "p".to_string(),
                                props: VProps::default(),
                                children: vec![VNode::Text(
                                    "HAF enforces clean architecture through compile-time guarantees.".to_string()
                                )],
                            },
                        ],
                    },
                    Route::Todo => TodoPage::default().render(&state.todos),
                    Route::NotFound => VNode::Element {
                        tag: "div".to_string(),
                        props: VProps {
                            class: Some("page not-found".to_string()),
                            ..Default::default()
                        },
                        children: vec![
                            VNode::Element {
                                tag: "h1".to_string(),
                                props: VProps::default(),
                                children: vec![VNode::Text("404 - Page Not Found".to_string())],
                            },
                        ],
                    },
                },
            ],
        }
    }
    
    fn setup_event_handlers(&mut self) {
        // Navigation handlers
        self.event_handlers.insert(EventId(100), Box::new(|| Action::Navigate(Route::Home)));
        self.event_handlers.insert(EventId(101), Box::new(|| Action::Navigate(Route::About)));
        self.event_handlers.insert(EventId(102), Box::new(|| Action::Navigate(Route::Todo)));
        
        // Theme toggle
        self.event_handlers.insert(EventId(200), Box::new(|| Action::ToggleTheme));
        
        // Todo handlers would be dynamic based on todo IDs
    }
}

// ==================== L3: Framework Layer ====================

/// Browser router service
crate::haf_service!(L3, router_service, {
    pub fn init() -> () {
        // Set up popstate listener
        // In real implementation, would integrate with runtime
    }
    
    pub fn navigate(path: &str) -> Result<(), String> {
        // Use browser history API
        Ok(())
    }
});

/// Local storage service
crate::haf_service!(L3, storage_service, {
    pub fn save_state(state: &AppState) -> Result<(), String> {
        // Serialize and save to localStorage
        Ok(())
    }
    
    pub fn load_state() -> Result<Option<AppState>, String> {
        // Load and deserialize from localStorage
        Ok(None)
    }
});

/// Application entry point
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    
    // Initialize services
    router_service::init();
    
    // Load persisted state
    let initial_state = storage_service::load_state()
        .ok()
        .flatten()
        .unwrap_or_default();
    
    // Create and mount app
    let mut app = ComponentApp::<L3>::new();
    
    // Render initial state
    let runtime = AppRuntime::new();
    let initial_vnode = runtime.render_app(&initial_state);
    
    app.mount("#app", initial_vnode);
}