//! SSR Demo Application

use layer9_core::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use async_trait::async_trait;

/// Todo model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: u32,
    pub text: String,
    pub done: bool,
}

/// Home page component
pub struct HomePage;

impl HomePage {
    pub fn new() -> Self {
        HomePage
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl SSRComponent for HomePage {
    fn render_to_string(&self, ctx: &SSRContext) -> String {
        let todos_html = if let Some(todos_json) = ctx.props.get("todos") {
            if let Ok(todos) = serde_json::from_str::<Vec<Todo>>(todos_json) {
                todos.iter()
                    .map(|todo| {
                        format!(
                            r#"<li class="todo-item">
                                <input type="checkbox" {} />
                                <span class="{}">{}</span>
                            </li>"#,
                            if todo.done { "checked" } else { "" },
                            if todo.done { "done" } else { "" },
                            todo.text
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                "<li>Failed to load todos</li>".to_string()
            }
        } else {
            "<li>Loading todos...</li>".to_string()
        };

        format!(
            r#"
            <div class="container">
                <header>
                    <h1>Layer9 SSR Demo</h1>
                    <nav>
                        <a href="/">Home</a>
                        <a href="/todos">Todos</a>
                        <a href="/about">About</a>
                    </nav>
                </header>
                
                <main>
                    <section class="hero">
                        <h2>Welcome to Layer9 SSR</h2>
                        <p>This demo shows server-side rendering with hydration.</p>
                    </section>
                    
                    <section class="todos">
                        <h3>Recent Todos</h3>
                        <ul class="todo-list">
                            {}
                        </ul>
                    </section>
                </main>
                
                <footer>
                    <p>Built with Layer9 - Web Architecture Rust Platform</p>
                </footer>
                
                <style>
                    body {{
                        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                        margin: 0;
                        padding: 0;
                        background: #f5f5f5;
                    }}
                    .container {{
                        max-width: 1200px;
                        margin: 0 auto;
                        padding: 20px;
                    }}
                    header {{
                        background: #333;
                        color: white;
                        padding: 20px;
                        margin: -20px -20px 20px;
                    }}
                    nav a {{
                        color: white;
                        text-decoration: none;
                        margin-right: 20px;
                    }}
                    .hero {{
                        background: white;
                        padding: 40px;
                        text-align: center;
                        border-radius: 8px;
                        margin-bottom: 20px;
                    }}
                    .todos {{
                        background: white;
                        padding: 20px;
                        border-radius: 8px;
                    }}
                    .todo-list {{
                        list-style: none;
                        padding: 0;
                    }}
                    .todo-item {{
                        padding: 10px;
                        border-bottom: 1px solid #eee;
                    }}
                    .done {{
                        text-decoration: line-through;
                        opacity: 0.6;
                    }}
                    footer {{
                        text-align: center;
                        margin-top: 40px;
                        color: #666;
                    }}
                </style>
            </div>
            "#,
            todos_html
        )
    }

    async fn get_server_props(&self, _ctx: &SSRContext) -> Result<serde_json::Value, String> {
        // In a real app, this would fetch from database
        let todos = vec![
            Todo { id: 1, text: "Build SSR support".to_string(), done: true },
            Todo { id: 2, text: "Add hydration".to_string(), done: true },
            Todo { id: 3, text: "Test with real data".to_string(), done: false },
            Todo { id: 4, text: "Deploy to production".to_string(), done: false },
        ];

        Ok(serde_json::json!({
            "todos": todos,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }))
    }
}

/// Todos page component
pub struct TodosPage;

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl SSRComponent for TodosPage {
    fn render_to_string(&self, _ctx: &SSRContext) -> String {
        r#"
        <div class="container">
            <header>
                <h1>Layer9 SSR Demo</h1>
                <nav>
                    <a href="/">Home</a>
                    <a href="/todos">Todos</a>
                    <a href="/about">About</a>
                </nav>
            </header>
            
            <main>
                <h2>Todo List</h2>
                <div class="todo-app">
                    <input type="text" placeholder="What needs to be done?" />
                    <button>Add Todo</button>
                    <ul class="todo-list" id="todos"></ul>
                </div>
            </main>
        </div>
        "#.to_string()
    }
}

/// About page component
pub struct AboutPage;

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl SSRComponent for AboutPage {
    fn render_to_string(&self, _ctx: &SSRContext) -> String {
        r#"
        <div class="container">
            <header>
                <h1>Layer9 SSR Demo</h1>
                <nav>
                    <a href="/">Home</a>
                    <a href="/todos">Todos</a>
                    <a href="/about">About</a>
                </nav>
            </header>
            
            <main>
                <h2>About Layer9 SSR</h2>
                <p>This demo showcases Layer9's server-side rendering capabilities:</p>
                <ul>
                    <li>Server-side data fetching</li>
                    <li>HTML generation on the server</li>
                    <li>Client-side hydration</li>
                    <li>SEO-friendly pages</li>
                    <li>Fast initial page loads</li>
                </ul>
            </main>
        </div>
        "#.to_string()
    }
}

/// SSR App implementation
pub struct SSRDemoApp;

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl SSRApp for SSRDemoApp {
    fn routes(&self) -> Vec<SSRRoute> {
        vec![
            SSRRoute {
                path: "/".to_string(),
                handler: Arc::new(HomeHandler),
            },
            SSRRoute {
                path: "/todos".to_string(),
                handler: Arc::new(TodosHandler),
            },
            SSRRoute {
                path: "/about".to_string(),
                handler: Arc::new(AboutHandler),
            },
        ]
    }
}

/// Route handlers
struct HomeHandler;

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl SSRRouteHandler for HomeHandler {
    async fn handle(&self, mut ctx: SSRContext) -> Result<String, String> {
        let home = HomePage::new();
        
        // Get server props
        let props = home.get_server_props(&ctx).await?;
        
        // Add todos to context
        if let Some(todos) = props.get("todos") {
            ctx.props.insert("todos".to_string(), todos.to_string());
        }
        
        // Set initial state
        ctx.initial_state = Some(props.to_string());
        
        // Add meta tags
        ctx = ctx.add_meta_tag(r#"<meta name="description" content="Layer9 SSR Demo Application">"#.to_string());
        
        // Create renderer and render
        let mut renderer = SSRRenderer::new();
        renderer.add_component(Box::new(home));
        
        Ok(renderer.render(&ctx).await)
    }
}

struct TodosHandler;

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl SSRRouteHandler for TodosHandler {
    async fn handle(&self, ctx: SSRContext) -> Result<String, String> {
        let mut renderer = SSRRenderer::new();
        renderer.add_component(Box::new(TodosPage));
        Ok(renderer.render(&ctx).await)
    }
}

struct AboutHandler;

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl SSRRouteHandler for AboutHandler {
    async fn handle(&self, ctx: SSRContext) -> Result<String, String> {
        let mut renderer = SSRRenderer::new();
        renderer.add_component(Box::new(AboutPage));
        Ok(renderer.render(&ctx).await)
    }
}

// Client-side entry point
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    
    // Check if we're hydrating
    if let Some(window) = web_sys::window() {
        if js_sys::Reflect::has(&window, &"__SSR_CONTEXT__".into()).unwrap_or(false) {
            hydrate_app();
        }
    }
}