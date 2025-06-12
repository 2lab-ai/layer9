//! Application Structure - L8

use crate::layers::*;
use crate::router::{Route, Router};

/// Main application trait
pub trait Layer9App: L8::Architecture {
    fn routes(&self) -> Vec<Route>;
    fn initialize(&self);
}

/// Global app instance (type-erased)
/// Initialize and run the app
pub fn run_app<T: Layer9App + 'static>(app: T) {
    // Set panic hook for better error messages
    console_error_panic_hook::set_once();

    // Initialize app
    app.initialize();

    // Setup router
    let mut router = Router::new();
    for route in app.routes() {
        router.add_route(route);
    }

    // Navigate to current path
    let window = web_sys::window().unwrap();
    let location = window.location();
    let pathname = location.pathname().unwrap();
    router.navigate(&pathname);
}

/// App builder
pub struct AppBuilder {
    name: String,
    routes: Vec<Route>,
}

impl AppBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        AppBuilder {
            name: name.into(),
            routes: vec![],
        }
    }

    pub fn route(mut self, route: Route) -> Self {
        self.routes.push(route);
        self
    }

    pub fn build(self) -> impl Layer9App {
        struct BuiltApp {
            name: String,
            routes: Vec<Route>,
        }

        impl LayerBound for BuiltApp {
            const LAYER: Layer = Layer::L8Architecture;
        }

        impl L8::Architecture for BuiltApp {
            type App = DummyApp;

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
                        Layer::L9Philosophy,
                    ],
                    boundaries: vec![],
                }
            }
        }

        impl Layer9App for BuiltApp {
            fn routes(&self) -> Vec<Route> {
                self.routes.clone()
            }

            fn initialize(&self) {
                web_sys::console::log_1(&format!("Layer9 App '{}' initialized", self.name).into());
            }
        }

        // Dummy app type for now
        struct DummyApp;

        impl LayerBound for DummyApp {
            const LAYER: Layer = Layer::L7Application;
        }

        impl L7::Application for DummyApp {
            type State = ();
            type Action = ();

            fn reduce(_state: &Self::State, _action: Self::Action) -> Self::State {}
        }

        BuiltApp {
            name: self.name,
            routes: self.routes,
        }
    }
}
