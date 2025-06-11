//! Minimal WARP Application

use warp_framework::prelude::*;
use wasm_bindgen::prelude::*;

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
                        .title("WARP App")
                        .component(HomePage)
                }),
            },
        ]
    }
    
    fn initialize(&self) {
        inject_global_styles();
        web_sys::console::log_1(&"WARP App initialized!".into());
    }
}

impl L8::Architecture for App {
    type App = MinimalApp;
    
    fn design() -> L8::ArchitectureDesign {
        L8::ArchitectureDesign {
            layers: vec![
                Layer::L1Infrastructure,
                Layer::L5Components,
                Layer::L7Application,
            ],
            boundaries: vec![],
        }
    }
}

// Home page component
struct HomePage;

impl Component for HomePage {
    fn render(&self) -> Element {
        view! {
            <div class="container">
                <h1>"Welcome to WARP"</h1>
                <p>"Web Architecture Rust Platform"</p>
                {Button::new("Get Started")
                    .variant(ButtonVariant::Primary)
                    .render()}
            </div>
        }
    }
}

// Minimal app type
struct MinimalApp;
impl L7::Application for MinimalApp {
    type State = ();
    type Action = ();
    
    fn reduce(_: &Self::State, _: Self::Action) -> Self::State {
        ()
    }
}

// Entry point
#[wasm_bindgen(start)]
pub fn main() {
    run_app(App);
}