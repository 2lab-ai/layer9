//! Layer9 Counter Example - Reactive Rendering
//!
//! This example demonstrates the new reactive rendering system with automatic
//! DOM updates when state changes.

use layer9_core::prelude::*;
use wasm_bindgen::prelude::*;

// Use `wee_alloc` as the global allocator for smaller bundle size
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Counter component with reactive state
struct CounterApp {
    count: State<i32>,
}

impl CounterApp {
    fn new() -> Self {
        Self {
            count: State::new(0),
        }
    }
}

impl Component for CounterApp {
    fn render(&self) -> Element {
        let count_value = self.count.get();

        // Create the root element with reactive event handlers
        Element::Node {
            tag: "div".to_string(),
            props: Props {
                class: Some("layer9-app".to_string()),
                ..Default::default()
            },
            children: vec![
                // Inline styles
                Element::Node {
                    tag: "style".to_string(),
                    props: Default::default(),
                    children: vec![Element::Text(
                        if cfg!(debug_assertions) {
                            include_str!("../styles.css")
                        } else {
                            include_str!("../styles.min.css")
                        }.to_string()
                    )],
                },
                // Title
                Element::Node {
                    tag: "h1".to_string(),
                    props: Default::default(),
                    children: vec![Element::Text("Layer9 Counter".to_string())],
                },
                // Counter display
                Element::Node {
                    tag: "p".to_string(),
                    props: Props {
                        class: Some("counter-value".to_string()),
                        ..Default::default()
                    },
                    children: vec![Element::Text(format!("Count: {}", count_value))],
                },
                // Button container
                Element::Node {
                    tag: "div".to_string(),
                    props: Props {
                        class: Some("button-container".to_string()),
                        ..Default::default()
                    },
                    children: vec![
                        // Increment button
                        Element::Node {
                            tag: "button".to_string(),
                            props: Props {
                                class: Some("btn btn-primary".to_string()),
                                on_click: Some(std::rc::Rc::new({
                                    let count = self.count.clone();
                                    move || count.set(count.get() + 1)
                                })),
                                ..Default::default()
                            },
                            children: vec![Element::Text("Increment".to_string())],
                        },
                        // Decrement button
                        Element::Node {
                            tag: "button".to_string(),
                            props: Props {
                                class: Some("btn btn-secondary".to_string()),
                                on_click: Some(std::rc::Rc::new({
                                    let count = self.count.clone();
                                    move || count.set(count.get() - 1)
                                })),
                                ..Default::default()
                            },
                            children: vec![Element::Text("Decrement".to_string())],
                        },
                        // Reset button
                        Element::Node {
                            tag: "button".to_string(),
                            props: Props {
                                class: Some("btn btn-warning".to_string()),
                                on_click: Some(std::rc::Rc::new({
                                    let count = self.count.clone();
                                    move || count.set(0)
                                })),
                                ..Default::default()
                            },
                            children: vec![Element::Text("Reset".to_string())],
                        },
                    ],
                },
                // Info text
                Element::Node {
                    tag: "p".to_string(),
                    props: Props {
                        class: Some("info".to_string()),
                        ..Default::default()
                    },
                    children: vec![
                        Element::Text("Built with ".to_string()),
                        Element::Node {
                            tag: "strong".to_string(),
                            props: Default::default(),
                            children: vec![Element::Text("Layer9".to_string())],
                        },
                        Element::Text(" - A Reactive Rust Web Framework".to_string()),
                    ],
                },
            ],
        }
    }
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Set panic hook for better error messages
    console_error_panic_hook::set_once();

    // Create and mount the app using the reactive renderer
    let app = CounterApp::new();
    mount(Box::new(app), "root");

    // Log success
    web_sys::console::log_1(&"Layer9 Counter App initialized with reactive rendering!".into());

    Ok(())
}