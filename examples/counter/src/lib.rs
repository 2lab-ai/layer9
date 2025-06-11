//! Counter Example - Demonstrates Layer9's hierarchical architecture

use layer9_framework::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::console;

// L9: Philosophy
struct CounterPhilosophy;

impl L9::Philosophy for CounterPhilosophy {
    fn vision(&self) -> &'static str {
        "Demonstrate that even simple interactions have deep structure"
    }
    
    fn purpose(&self) -> &'static str {
        "Show how counting connects to universal computation"
    }
}

// L8: Architecture
struct CounterArchitecture;

impl L8::Architecture for CounterArchitecture {
    type App = CounterApp;
    
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
            boundaries: vec![
                (Layer::L7Application, Layer::L5Components),
                (Layer::L5Components, Layer::L4Services),
            ],
        }
    }
}

// L7: Application
struct CounterApp;

impl L7::Application for CounterApp {
    type State = CounterState;
    type Action = CounterAction;
    
    fn reduce(state: &Self::State, action: Self::Action) -> Self::State {
        match action {
            CounterAction::Increment => CounterState {
                count: state.count + 1,
                ..state.clone()
            },
            CounterAction::Decrement => CounterState {
                count: state.count - 1,
                ..state.clone()
            },
            CounterAction::Reset => CounterState {
                count: 0,
                ..state.clone()
            },
        }
    }
}

#[derive(Clone)]
struct CounterState {
    count: i32,
    history: Vec<i32>,
}

enum CounterAction {
    Increment,
    Decrement,
    Reset,
}

// L5: Components
struct Counter {
    count: State<i32>,
}

impl Counter {
    fn new() -> Self {
        Counter {
            count: use_state(|| 0),
        }
    }
}

impl Component for Counter {
    fn render(&self) -> Element {
        let count = self.count.get();
        
        view! {
            <div class="counter-container">
                <h1>"Layer9 Counter Example"</h1>
                <p>"Count: " {count.to_string()}</p>
                <div class="buttons">
                    <button id="increment">"+"</button>
                    <button id="decrement">"-"</button>
                    <button id="reset">"Reset"</button>
                </div>
                <div class="philosophy">
                    <p>"This simple counter demonstrates Layer9's hierarchical architecture"</p>
                    <p>"From L9 (Philosophy) to L1 (Infrastructure), every layer has its purpose"</p>
                </div>
            </div>
        }
    }
}

// L4: Server Functions
server_function! {
    fn get_counter_stats() -> CounterStats {
        Ok(CounterStats {
            total_clicks: 42,
            average_value: 7,
            peak_value: 100,
        })
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CounterStats {
    total_clicks: u32,
    average_value: f64,
    peak_value: i32,
}

// Main app setup
#[wasm_bindgen]
pub struct App;

impl Layer9App for App {
    fn routes(&self) -> Vec<Route> {
        vec![
            Route {
                path: "/".to_string(),
                handler: RouteHandler::Page(|| {
                    Page::new()
                        .title("Layer9 Counter")
                        .component(Counter::new())
                }),
            },
        ]
    }
    
    fn initialize(&self) {
        console::log_1(&"Layer9 Counter App initialized!".into());
        console::log_1(&"Hierarchical layers active:".into());
        console::log_1(&"L9: Philosophy - Why we count".into());
        console::log_1(&"L8: Architecture - How we structure counting".into());
        console::log_1(&"L7: Application - Counter business logic".into());
        console::log_1(&"L5: Components - UI representation".into());
        console::log_1(&"L4: Services - Server communication".into());
        console::log_1(&"L1-L3: Infrastructure & Runtime".into());
    }
}

impl L8::Architecture for App {
    type App = CounterApp;
    
    fn design() -> L8::ArchitectureDesign {
        CounterArchitecture::design()
    }
}

// Entry point
#[wasm_bindgen(start)]
pub fn main() {
    run_app(App);
}