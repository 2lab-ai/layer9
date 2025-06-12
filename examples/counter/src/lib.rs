//! Layer9 Counter Example - Pure Framework Version
//!
//! This example demonstrates using Layer9's component system and state management
//! without any direct DOM manipulation.

use layer9_core::{
    component::{Component, Element, Props, State},
    ui::Card,
};
use std::rc::Rc;
use wasm_bindgen::prelude::*;

// Use `wee_alloc` as the global allocator for smaller bundle size
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Counter component that uses Layer9 framework
struct CounterComponent {
    count: State<i32>,
}

impl CounterComponent {
    fn new() -> Self {
        Self {
            count: State::new(0),
        }
    }
}

impl Component for CounterComponent {
    fn render(&self) -> Element {
        let count_value = self.count.get();

        // Create styled container using Layer9's UI components
        let card = Card::new()
            .class("layer9-app")
            .children(vec![
                // Title
                Element::Node {
                    tag: "h1".to_string(),
                    props: Props::default(),
                    children: vec![Element::Text("Layer9 Counter".to_string())],
                },
                // Counter display
                Element::Node {
                    tag: "p".to_string(),
                    props: Props {
                        class: Some("counter-value".to_string()),
                        id: Some("counter-display".to_string()),
                        on_click: None,
                        attributes: vec![],
                    },
                    children: vec![Element::Text(format!("Count: {}", count_value))],
                },
                // Button container
                Element::Node {
                    tag: "div".to_string(),
                    props: Props {
                        class: Some("button-container".to_string()),
                        id: None,
                        on_click: None,
                        attributes: vec![],
                    },
                    children: vec![
                        // Increment button
                        Element::Node {
                            tag: "button".to_string(),
                            props: Props {
                                class: Some("btn btn-primary".to_string()),
                                id: None,
                                on_click: Some(Rc::new({
                                    let count = self.count.clone();
                                    move || {
                                        let current = count.get();
                                        count.set(current + 1);
                                    }
                                })),
                                attributes: vec![],
                            },
                            children: vec![Element::Text("Increment".to_string())],
                        },
                        // Decrement button
                        Element::Node {
                            tag: "button".to_string(),
                            props: Props {
                                class: Some("btn btn-secondary".to_string()),
                                id: None,
                                on_click: Some(Rc::new({
                                    let count = self.count.clone();
                                    move || {
                                        let current = count.get();
                                        count.set(current - 1);
                                    }
                                })),
                                attributes: vec![],
                            },
                            children: vec![Element::Text("Decrement".to_string())],
                        },
                        // Reset button
                        Element::Node {
                            tag: "button".to_string(),
                            props: Props {
                                class: Some("btn btn-warning".to_string()),
                                id: None,
                                on_click: Some(Rc::new({
                                    let count = self.count.clone();
                                    move || {
                                        count.set(0);
                                    }
                                })),
                                attributes: vec![],
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
                        id: None,
                        on_click: None,
                        attributes: vec![],
                    },
                    children: vec![
                        Element::Text("Built with ".to_string()),
                        Element::Node {
                            tag: "strong".to_string(),
                            props: Props::default(),
                            children: vec![Element::Text("Layer9".to_string())],
                        },
                        Element::Text(" - A Rust Web Framework".to_string()),
                    ],
                },
            ]);
        
        // Return the Card component wrapped in an Element
        Element::Component(Box::new(card))
    }
}

// Wrapper component that includes styles and manages updates
struct AppComponent {
    counter: CounterComponent,
    container: std::cell::RefCell<Option<web_sys::Element>>,
}

impl AppComponent {
    fn new() -> std::rc::Rc<Self> {
        std::rc::Rc::new(Self {
            counter: CounterComponent::new(),
            container: std::cell::RefCell::new(None),
        })
    }
    
    fn setup_reactive_updates(self: &std::rc::Rc<Self>) {
        let app_clone = self.clone();
        self.counter.count.set_update_callback(Box::new(move || {
            if let Some(container) = app_clone.container.borrow().as_ref() {
                // Re-render the component
                container.set_inner_html("");
                let rendered = app_clone.render();
                let dom_node = rendered.to_dom();
                let _ = container.append_child(&dom_node);
            }
        }));
    }
}

impl Component for AppComponent {
    fn render(&self) -> Element {
        Element::Node {
            tag: "div".to_string(),
            props: Props::default(),
            children: vec![
                // Add styles
                Element::Node {
                    tag: "style".to_string(),
                    props: Props::default(),
                    children: vec![Element::Text(
                        if cfg!(debug_assertions) {
                            include_str!("../styles.css")
                        } else {
                            include_str!("../styles.min.css")
                        }.to_string()
                    )],
                },
                // Render counter
                self.counter.render(),
            ],
        }
    }
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Set panic hook for better error messages
    console_error_panic_hook::set_once();

    // Create the app component
    let app = AppComponent::new();

    // Get the document body
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    // Clear existing content
    body.set_inner_html("");
    
    // Create a container
    let container = document.create_element("div")?;
    body.append_child(&container)?;
    
    // Store container reference
    *app.container.borrow_mut() = Some(container.clone());
    
    // Setup reactive updates
    app.setup_reactive_updates();
    
    // Initial render
    let rendered = app.render();
    let dom_node = rendered.to_dom();
    container.append_child(&dom_node)?;

    // Log success
    web_sys::console::log_1(&"Layer9 Counter App initialized!".into());

    Ok(())
}
