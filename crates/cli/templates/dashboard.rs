//! Dashboard WARP Application

use warp_framework::prelude::*;
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

// Global state
static APP_STORE: Lazy<ReducerStore<AppState, AppAction>> = Lazy::new(|| {
    create_app_store()
});

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
                        .title("Dashboard")
                        .component(DashboardPage)
                }),
            },
            Route {
                path: "/settings".to_string(),
                handler: RouteHandler::Page(|| {
                    Page::new()
                        .title("Settings")
                        .component(SettingsPage)
                }),
            },
        ]
    }
    
    fn initialize(&self) {
        inject_global_styles();
        
        // Initialize router
        let config = RouterConfig {
            routes: vec![
                route("/", |_| Box::new(DashboardPage)),
                route("/settings", |_| Box::new(SettingsPage)),
                route("/profile/:id", |params| Box::new(ProfilePage { 
                    id: params.params.get("id").cloned().unwrap_or_default() 
                })),
            ],
            not_found: Box::new(NotFoundPage),
        };
        
        init_router(config).expect("Failed to initialize router");
    }
}

impl L8::Architecture for App {
    type App = DashboardApp;
    
    fn design() -> L8::ArchitectureDesign {
        L8::ArchitectureDesign {
            layers: vec![
                Layer::L1Infrastructure,
                Layer::L4Services,
                Layer::L5Components,
                Layer::L6Features,
                Layer::L7Application,
            ],
            boundaries: vec![],
        }
    }
}

// Dashboard page
struct DashboardPage;

impl Component for DashboardPage {
    fn render(&self) -> Element {
        let (state, dispatch) = use_reducer(&APP_STORE);
        let stats = use_swr::<DashboardStats>("/api/stats");
        
        view! {
            <div class="dashboard">
                <Header />
                
                <div class="stats-grid">
                    {if stats.is_loading() {
                        view! { <div>"Loading stats..."</div> }
                    } else if let Some(data) = stats.data() {
                        view! {
                            <div>
                                {StatCard::new("Total Users", &data.users.to_string()).render()}
                                {StatCard::new("Revenue", &format!("${}", data.revenue)).render()}
                                {StatCard::new("Active Sessions", &data.sessions.to_string()).render()}
                            </div>
                        }
                    } else {
                        view! { <div>"Failed to load stats"</div> }
                    }}
                </div>
                
                <div class="content">
                    <Card::new()
                        .children(vec![
                            view! { <h2>"Recent Activity"</h2> },
                            view! { <ActivityList /> },
                        ])
                        .render()
                </div>
            </div>
        }
    }
}

// Header component
struct Header;

impl Component for Header {
    fn render(&self) -> Element {
        let auth = use_auth();
        
        let header_style = style![
            flex,
            justify_between,
            items_center,
            p(4),
            border,
            border_gray_200,
            mb(6),
        ];
        
        view! {
            <header style={header_style.build()}>
                <h1>"WARP Dashboard"</h1>
                
                <nav>
                    {Link::new("/")
                        .children(vec![view! { "Dashboard" }])
                        .render()}
                    {Link::new("/settings")
                        .children(vec![view! { "Settings" }])
                        .render()}
                </nav>
                
                <div class="user-menu">
                    {if let Some(user) = auth.user {
                        view! {
                            <div>
                                {Avatar::new()
                                    .src(user.image.as_deref().unwrap_or_default())
                                    .fallback(&user.name[0..1])
                                    .render()}
                                <span>{user.name}</span>
                            </div>
                        }
                    } else {
                        Button::new("Login")
                            .variant(ButtonVariant::Primary)
                            .render()
                    }}
                </div>
            </header>
        }
    }
}

// Settings page
struct SettingsPage;

impl Component for SettingsPage {
    fn render(&self) -> Element {
        let (state, dispatch) = use_reducer(&APP_STORE);
        
        view! {
            <div class="settings">
                <Header />
                
                <Card::new()
                    .children(vec![
                        view! { <h2>"Settings"</h2> },
                        
                        view! {
                            <div class="setting-item">
                                <label>"Theme"</label>
                                {Button::new("Toggle Theme")
                                    .on_click(move || dispatch(AppAction::ToggleTheme))
                                    .render()}
                            </div>
                        },
                    ])
                    .render()
            </div>
        }
    }
}

// Profile page with params
struct ProfilePage {
    id: String,
}

impl Component for ProfilePage {
    fn render(&self) -> Element {
        view! {
            <div>
                <Header />
                <h1>"Profile: "{&self.id}</h1>
            </div>
        }
    }
}

// Not found page
struct NotFoundPage;

impl Component for NotFoundPage {
    fn render(&self) -> Element {
        view! {
            <div class="not-found">
                <h1>"404"</h1>
                <p>"Page not found"</p>
                {Link::new("/")
                    .children(vec![view! { "Go home" }])
                    .render()}
            </div>
        }
    }
}

// Activity list component
struct ActivityList;

impl Component for ActivityList {
    fn render(&self) -> Element {
        view! {
            <ul class="activity-list">
                <li>"User signed up"</li>
                <li>"Payment received"</li>
                <li>"New feature deployed"</li>
            </ul>
        }
    }
}

// Stat card component
struct StatCard {
    title: String,
    value: String,
}

impl StatCard {
    fn new(title: impl Into<String>, value: impl Into<String>) -> Self {
        StatCard {
            title: title.into(),
            value: value.into(),
        }
    }
}

impl Component for StatCard {
    fn render(&self) -> Element {
        Card::new()
            .children(vec![
                view! { <h3>{&self.title}</h3> },
                view! { <div class="stat-value">{&self.value}</div> },
            ])
            .render()
    }
}

// Data structures
#[derive(Serialize, Deserialize)]
struct DashboardStats {
    users: u32,
    revenue: u32,
    sessions: u32,
}

// Dashboard app type
struct DashboardApp;
impl L7::Application for DashboardApp {
    type State = AppState;
    type Action = AppAction;
    
    fn reduce(state: &Self::State, action: Self::Action) -> Self::State {
        APP_STORE.dispatch(action);
        state.clone()
    }
}

// Hooks
fn use_swr<T: Clone + for<'de> Deserialize<'de> + 'static>(url: &str) -> SWR<T> {
    SWR::new(url)
}

// Re-exports
use once_cell::sync::Lazy;

// Entry point
#[wasm_bindgen(start)]
pub fn main() {
    run_app(App);
}