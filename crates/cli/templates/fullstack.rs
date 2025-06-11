//! Full-Stack WARP Application with SSR

use warp_framework::prelude::*;
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

// Define your app
#[wasm_bindgen]
pub struct App;

impl WarpApp for App {
    fn routes(&self) -> Vec<Route> {
        vec![
            Route {
                path: "/".to_string(),
                handler: RouteHandler::Page(|| {
                    Page::new()
                        .title("Full-Stack WARP")
                        .component(HomePage)
                }),
            },
            Route {
                path: "/api/todos".to_string(),
                handler: RouteHandler::Api(|| {
                    // This would be handled server-side
                    JsValue::from_str(r#"{"todos": []}"#)
                }),
            },
        ]
    }
    
    fn initialize(&self) {
        inject_global_styles();
        
        // Initialize router with SSR support
        let config = RouterConfig {
            routes: vec![
                route("/", |_| Box::new(HomePage)),
                route("/todos", |_| Box::new(TodosPage)),
                route("/about", |_| Box::new(AboutPage)),
            ],
            not_found: Box::new(NotFoundPage),
        };
        
        init_router(config).expect("Failed to initialize router");
    }
}

impl L8::Architecture for App {
    type App = FullStackApp;
    
    fn design() -> L8::ArchitectureDesign {
        L8::ArchitectureDesign {
            layers: vec![
                Layer::L1Infrastructure,
                Layer::L2Platform,
                Layer::L3Runtime,
                Layer::L4Services,
                Layer::L5Components,
                Layer::L6Features,
                Layer::L7Application,
                Layer::L8Architecture,
            ],
            boundaries: vec![],
        }
    }
}

// SSR support
impl SSRApp for App {
    fn html_template(&self) -> &'static str {
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <meta name="description" content="Full-Stack WARP Application">
    <style>{styles}</style>
    <script>window.__WARP_PROPS__ = {props};</script>
</head>
<body>
    <div id="warp-root">{content}</div>
    <script type="module">
        import init, { hydrate_app } from '/warp_bundle.js';
        init().then(() => {
            hydrate_app();
        });
    </script>
</body>
</html>"#
    }
}

// Home page with server data
struct HomePage;

impl Component for HomePage {
    fn render(&self) -> Element {
        view! {
            <div class="home">
                <Navigation />
                
                <main>
                    <Hero />
                    <Features />
                    <TodoSection />
                </main>
                
                <Footer />
            </div>
        }
    }
}

impl SSRComponent for HomePage {
    async fn get_server_props(ctx: &SSRContext) -> Result<serde_json::Value, String> {
        // Fetch data on server
        Ok(serde_json::json!({
            "featured_todos": [
                {"id": 1, "text": "Build with WARP", "done": false},
                {"id": 2, "text": "Deploy to production", "done": false},
            ]
        }))
    }
}

// Navigation component
struct Navigation;

impl Component for Navigation {
    fn render(&self) -> Element {
        let nav_style = style![
            flex,
            justify_between,
            items_center,
            px(6),
            py(4),
            bg_black,
            text_white,
        ];
        
        view! {
            <nav style={nav_style.build()}>
                <div class="logo">
                    <h1>"WARP"</h1>
                </div>
                
                <div class="nav-links">
                    {Link::new("/").children(vec![view! { "Home" }]).render()}
                    {Link::new("/todos").children(vec![view! { "Todos" }]).render()}
                    {Link::new("/about").children(vec![view! { "About" }]).render()}
                </div>
                
                <div class="auth">
                    {Protected::new(UserMenu)
                        .fallback(LoginButton)
                        .render()}
                </div>
            </nav>
        }
    }
}

// Hero section
struct Hero;

impl Component for Hero {
    fn render(&self) -> Element {
        let hero_style = style![
            text_center,
            py(20),
            px(4),
        ];
        
        view! {
            <section style={hero_style.build()}>
                <h1 class="hero-title">"Build Full-Stack Apps with Rust"</h1>
                <p class="hero-subtitle">
                    "Server-side rendering, authentication, and real-time updates"
                </p>
                {Button::new("Get Started")
                    .variant(ButtonVariant::Primary)
                    .on_click(|| navigate("/todos").unwrap())
                    .render()}
            </section>
        }
    }
}

// Features section
struct Features;

impl Component for Features {
    fn render(&self) -> Element {
        let features = vec![
            ("🚀", "Blazing Fast", "WASM performance with Rust safety"),
            ("🔒", "Type Safe", "End-to-end type safety from server to client"),
            ("📦", "SSR Ready", "Server-side rendering out of the box"),
            ("🎨", "Styled", "CSS-in-Rust with zero runtime"),
        ];
        
        let grid_style = style![
            grid,
            lg_grid_cols(4),
            gap(6),
            px(6),
            py(12),
        ];
        
        view! {
            <section style={grid_style.build()}>
                {features.into_iter().map(|(icon, title, desc)| {
                    Card::new()
                        .children(vec![
                            view! { <div class="feature-icon">{icon}</div> },
                            view! { <h3>{title}</h3> },
                            view! { <p>{desc}</p> },
                        ])
                        .render()
                }).collect::<Vec<_>>()}
            </section>
        }
    }
}

// Todo section with API integration
struct TodoSection;

impl Component for TodoSection {
    fn render(&self) -> Element {
        let todos = use_swr::<TodosResponse>("/api/todos");
        
        view! {
            <section class="todos">
                <h2>"Recent Todos"</h2>
                
                {if todos.is_loading() {
                    view! { <div>"Loading todos..."</div> }
                } else if let Some(data) = todos.data() {
                    view! {
                        <ul>
                            {data.todos.iter().map(|todo| {
                                view! {
                                    <li>
                                        <input type="checkbox" checked={todo.done} />
                                        <span>{&todo.text}</span>
                                    </li>
                                }
                            }).collect::<Vec<_>>()}
                        </ul>
                    }
                } else {
                    view! { <div>"Failed to load todos"</div> }
                }}
            </section>
        }
    }
}

// Todos page
struct TodosPage;

impl Component for TodosPage {
    fn render(&self) -> Element {
        let new_todo = use_state(|| String::new());
        let todos = use_atom(&TODOS_ATOM);
        
        view! {
            <div class="todos-page">
                <Navigation />
                
                <main>
                    <h1>"Todo List"</h1>
                    
                    <div class="todo-input">
                        {Input::new()
                            .placeholder("What needs to be done?")
                            .value(new_todo.get())
                            .on_change(move |v| new_todo.set(v))
                            .render()}
                        {Button::new("Add")
                            .on_click(move || {
                                if !new_todo.get().is_empty() {
                                    add_todo(new_todo.get());
                                    new_todo.set(String::new());
                                }
                            })
                            .render()}
                    </div>
                    
                    <TodoList />
                </main>
            </div>
        }
    }
}

// Todo list component
struct TodoList;

impl Component for TodoList {
    fn render(&self) -> Element {
        let todos = use_atom(&TODOS_ATOM);
        
        view! {
            <ul class="todo-list">
                {todos.get().unwrap_or_default().iter().map(|todo| {
                    view! {
                        <li>
                            <input 
                                type="checkbox" 
                                checked={todo.done}
                                onchange={move || toggle_todo(todo.id)}
                            />
                            <span class={if todo.done { "done" } else { "" }}>
                                {&todo.text}
                            </span>
                            {Button::new("Delete")
                                .variant(ButtonVariant::Destructive)
                                .on_click(move || delete_todo(todo.id))
                                .render()}
                        </li>
                    }
                }).collect::<Vec<_>>()}
            </ul>
        }
    }
}

// About page
struct AboutPage;

impl Component for AboutPage {
    fn render(&self) -> Element {
        view! {
            <div>
                <Navigation />
                <main>
                    <h1>"About WARP"</h1>
                    <p>"Web Architecture Rust Platform"</p>
                </main>
            </div>
        }
    }
}

// User menu
struct UserMenu;

impl Component for UserMenu {
    fn render(&self) -> Element {
        let auth = use_auth();
        
        view! {
            <div class="user-menu">
                {if let Some(user) = auth.user {
                    view! {
                        <div>
                            <span>{user.name}</span>
                            {Button::new("Logout")
                                .variant(ButtonVariant::Ghost)
                                .render()}
                        </div>
                    }
                } else {
                    view! { <div /> }
                }}
            </div>
        }
    }
}

// Login button
struct LoginButton;

impl Component for LoginButton {
    fn render(&self) -> Element {
        Button::new("Login")
            .variant(ButtonVariant::Primary)
            .render()
    }
}

// Footer
struct Footer;

impl Component for Footer {
    fn render(&self) -> Element {
        let footer_style = style![
            text_center,
            py(8),
            text_gray_500,
        ];
        
        view! {
            <footer style={footer_style.build()}>
                <p>"Built with WARP - Web Architecture Rust Platform"</p>
            </footer>
        }
    }
}

// Not found page
struct NotFoundPage;

impl Component for NotFoundPage {
    fn render(&self) -> Element {
        view! {
            <div>
                <Navigation />
                <main>
                    <h1>"404 - Page Not Found"</h1>
                    {Link::new("/").children(vec![view! { "Go Home" }]).render()}
                </main>
            </div>
        }
    }
}

// Data structures
#[derive(Clone, Serialize, Deserialize)]
struct Todo {
    id: u32,
    text: String,
    done: bool,
}

#[derive(Serialize, Deserialize)]
struct TodosResponse {
    todos: Vec<Todo>,
}

// Global state
static TODOS_ATOM: Lazy<Atom<Vec<Todo>>> = Lazy::new(|| {
    create_atom(vec![
        Todo { id: 1, text: "Learn WARP".to_string(), done: false },
        Todo { id: 2, text: "Build something awesome".to_string(), done: false },
    ])
});

// Todo actions
fn add_todo(text: String) {
    TODOS_ATOM.update(|todos| {
        let id = todos.iter().map(|t| t.id).max().unwrap_or(0) + 1;
        todos.push(Todo { id, text, done: false });
    });
}

fn toggle_todo(id: u32) {
    TODOS_ATOM.update(|todos| {
        if let Some(todo) = todos.iter_mut().find(|t| t.id == id) {
            todo.done = !todo.done;
        }
    });
}

fn delete_todo(id: u32) {
    TODOS_ATOM.update(|todos| {
        todos.retain(|t| t.id != id);
    });
}

// App type
struct FullStackApp;
impl L7::Application for FullStackApp {
    type State = ();
    type Action = ();
    
    fn reduce(_: &Self::State, _: Self::Action) -> Self::State {
        ()
    }
}

// Hooks
fn use_swr<T: Clone + for<'de> Deserialize<'de> + 'static>(url: &str) -> SWR<T> {
    SWR::new(url)
}

// Re-exports
use once_cell::sync::Lazy;

// Entry point for client
#[wasm_bindgen(start)]
pub fn main() {
    // Check if we're hydrating SSR content
    if let Some(window) = web_sys::window() {
        if js_sys::Reflect::has(&window, &"__WARP_PROPS__".into()).unwrap_or(false) {
            hydrate_app(App);
        } else {
            run_app(App);
        }
    }
}

// Entry point for server (would be in separate file)
#[cfg(not(target_arch = "wasm32"))]
pub fn create_server() -> axum::Router {
    create_ssr_server(App)
}