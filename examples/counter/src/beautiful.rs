//! Beautiful Counter Example - Showcasing Layer9's Reactive Features
//!
//! A stunning counter with animations, gradients, and modern UI design

use layer9_core::prelude::*;
use layer9_core::hooks::use_state_hook as use_state;
use layer9_core::reactive_v2::mount;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct BeautifulCounter;

impl Component for BeautifulCounter {
    fn render(&self) -> Element {
        let (count, set_count) = use_state(0i32);
        let (animation_class, set_animation_class) = use_state(String::new());
        
        // Helper to trigger animation
        let trigger_animation = {
            let set_animation_class = set_animation_class.clone();
            move |direction: &str| {
                set_animation_class(format!("animate-{}", direction));
                // Reset animation class after animation completes
                let set_animation_class_clone = set_animation_class.clone();
                web_sys::window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(
                    &wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                        set_animation_class_clone(String::new());
                    }) as Box<dyn FnMut()>).as_ref().unchecked_ref(),
                    300
                ).unwrap();
            }
        };
        
        let increment = {
            let count = count;
            let set_count = set_count.clone();
            let trigger_animation = trigger_animation.clone();
            move || {
                set_count(count + 1);
                trigger_animation("up");
            }
        };
        
        let decrement = {
            let count = count;
            let set_count = set_count.clone();
            let trigger_animation = trigger_animation.clone();
            move || {
                set_count(count - 1);
                trigger_animation("down");
            }
        };
        
        let increment_by = |amount: i32| {
            let count = count;
            let set_count = set_count.clone();
            let trigger_animation = trigger_animation.clone();
            move || {
                set_count(count + amount);
                trigger_animation("up");
            }
        };
        
        let reset = {
            let set_count = set_count.clone();
            let trigger_animation = trigger_animation.clone();
            move || {
                set_count(0);
                trigger_animation("reset");
            }
        };
        
        // Determine counter color based on value
        let counter_class = if count > 0 {
            "positive"
        } else if count < 0 {
            "negative"
        } else {
            "zero"
        };

        Element::Node {
            tag: "div".to_string(),
            props: Props {
                class: Some("beautiful-counter".to_string()),
                ..Default::default()
            },
            children: vec![
                // Inline styles
                Element::Node {
                    tag: "style".to_string(),
                    props: Props::default(),
                    children: vec![Element::Text(BEAUTIFUL_STYLES.to_string())],
                },
                
                // Background decoration
                Element::Node {
                    tag: "div".to_string(),
                    props: Props {
                        class: Some("bg-decoration".to_string()),
                        ..Default::default()
                    },
                    children: vec![
                        Element::Node {
                            tag: "div".to_string(),
                            props: Props {
                                class: Some("circle circle-1".to_string()),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                        Element::Node {
                            tag: "div".to_string(),
                            props: Props {
                                class: Some("circle circle-2".to_string()),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                        Element::Node {
                            tag: "div".to_string(),
                            props: Props {
                                class: Some("circle circle-3".to_string()),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                    ],
                },
                
                // Main content
                Element::Node {
                    tag: "div".to_string(),
                    props: Props {
                        class: Some("counter-content".to_string()),
                        ..Default::default()
                    },
                    children: vec![
                        // Header
                        Element::Node {
                            tag: "header".to_string(),
                            props: Props::default(),
                            children: vec![
                                Element::Node {
                                    tag: "h1".to_string(),
                                    props: Props::default(),
                                    children: vec![
                                        Element::Node {
                                            tag: "span".to_string(),
                                            props: Props {
                                                class: Some("gradient-text".to_string()),
                                                ..Default::default()
                                            },
                                            children: vec![Element::Text("Layer9".to_string())],
                                        },
                                        Element::Text(" Counter".to_string()),
                                    ],
                                },
                                Element::Node {
                                    tag: "p".to_string(),
                                    props: Props {
                                        class: Some("subtitle".to_string()),
                                        ..Default::default()
                                    },
                                    children: vec![Element::Text("Beautiful reactive state management".to_string())],
                                },
                            ],
                        },
                        
                        // Counter display
                        Element::Node {
                            tag: "div".to_string(),
                            props: Props {
                                class: Some(format!("counter-display {} {}", counter_class, animation_class)),
                                ..Default::default()
                            },
                            children: vec![
                                Element::Node {
                                    tag: "div".to_string(),
                                    props: Props {
                                        class: Some("counter-value".to_string()),
                                        ..Default::default()
                                    },
                                    children: vec![Element::Text(count.to_string())],
                                },
                                Element::Node {
                                    tag: "div".to_string(),
                                    props: Props {
                                        class: Some("counter-label".to_string()),
                                        ..Default::default()
                                    },
                                    children: vec![Element::Text(if count.abs() == 1 { "click" } else { "clicks" }.to_string())],
                                },
                            ],
                        },
                        
                        // Quick actions
                        Element::Node {
                            tag: "div".to_string(),
                            props: Props {
                                class: Some("quick-actions".to_string()),
                                ..Default::default()
                            },
                            children: vec![
                                create_quick_button("-10", increment_by(-10)),
                                create_quick_button("-5", increment_by(-5)),
                                create_quick_button("+5", increment_by(5)),
                                create_quick_button("+10", increment_by(10)),
                            ],
                        },
                        
                        // Main controls
                        Element::Node {
                            tag: "div".to_string(),
                            props: Props {
                                class: Some("main-controls".to_string()),
                                ..Default::default()
                            },
                            children: vec![
                                Element::Node {
                                    tag: "button".to_string(),
                                    props: Props {
                                        class: Some("btn btn-decrement".to_string()),
                                        on_click: Some(Rc::new(decrement)),
                                        ..Default::default()
                                    },
                                    children: vec![
                                        Element::Node {
                                            tag: "span".to_string(),
                                            props: Props {
                                                class: Some("btn-icon".to_string()),
                                                ..Default::default()
                                            },
                                            children: vec![Element::Text("−".to_string())],
                                        },
                                        Element::Text("Decrement".to_string()),
                                    ],
                                },
                                Element::Node {
                                    tag: "button".to_string(),
                                    props: Props {
                                        class: Some("btn btn-reset".to_string()),
                                        on_click: Some(Rc::new(reset)),
                                        ..Default::default()
                                    },
                                    children: vec![
                                        Element::Node {
                                            tag: "span".to_string(),
                                            props: Props {
                                                class: Some("btn-icon".to_string()),
                                                ..Default::default()
                                            },
                                            children: vec![Element::Text("↺".to_string())],
                                        },
                                        Element::Text("Reset".to_string()),
                                    ],
                                },
                                Element::Node {
                                    tag: "button".to_string(),
                                    props: Props {
                                        class: Some("btn btn-increment".to_string()),
                                        on_click: Some(Rc::new(increment)),
                                        ..Default::default()
                                    },
                                    children: vec![
                                        Element::Node {
                                            tag: "span".to_string(),
                                            props: Props {
                                                class: Some("btn-icon".to_string()),
                                                ..Default::default()
                                            },
                                            children: vec![Element::Text("+".to_string())],
                                        },
                                        Element::Text("Increment".to_string()),
                                    ],
                                },
                            ],
                        },
                        
                        // Statistics
                        Element::Node {
                            tag: "div".to_string(),
                            props: Props {
                                class: Some("stats".to_string()),
                                ..Default::default()
                            },
                            children: vec![
                                create_stat("Status", if count > 0 { "Positive" } else if count < 0 { "Negative" } else { "Zero" }),
                                create_stat("Distance from zero", &count.abs().to_string()),
                                create_stat("Square", &(count * count).to_string()),
                            ],
                        },
                        
                        // Footer
                        Element::Node {
                            tag: "footer".to_string(),
                            props: Props::default(),
                            children: vec![
                                Element::Text("Built with ".to_string()),
                                Element::Node {
                                    tag: "a".to_string(),
                                    props: Props {
                                        attributes: vec![
                                            ("href".to_string(), "https://github.com/anthropics/layer9".to_string()),
                                            ("target".to_string(), "_blank".to_string()),
                                        ],
                                        ..Default::default()
                                    },
                                    children: vec![Element::Text("Layer9".to_string())],
                                },
                                Element::Text(" • Reactive Rust Web Framework".to_string()),
                            ],
                        },
                    ],
                },
            ],
        }
    }
}

fn create_quick_button(label: &str, handler: impl Fn() + 'static) -> Element {
    Element::Node {
        tag: "button".to_string(),
        props: Props {
            class: Some("quick-btn".to_string()),
            on_click: Some(Rc::new(handler)),
            ..Default::default()
        },
        children: vec![Element::Text(label.to_string())],
    }
}

fn create_stat(label: &str, value: &str) -> Element {
    Element::Node {
        tag: "div".to_string(),
        props: Props {
            class: Some("stat".to_string()),
            ..Default::default()
        },
        children: vec![
            Element::Node {
                tag: "div".to_string(),
                props: Props {
                    class: Some("stat-label".to_string()),
                    ..Default::default()
                },
                children: vec![Element::Text(label.to_string())],
            },
            Element::Node {
                tag: "div".to_string(),
                props: Props {
                    class: Some("stat-value".to_string()),
                    ..Default::default()
                },
                children: vec![Element::Text(value.to_string())],
            },
        ],
    }
}

const BEAUTIFUL_STYLES: &str = r#"
    :root {
        --primary: #6366f1;
        --primary-light: #818cf8;
        --primary-dark: #4f46e5;
        --secondary: #ec4899;
        --secondary-light: #f472b6;
        --success: #10b981;
        --danger: #ef4444;
        --warning: #f59e0b;
        --bg-gradient-1: #667eea;
        --bg-gradient-2: #764ba2;
        --bg-gradient-3: #f093fb;
    }
    
    * {
        box-sizing: border-box;
        margin: 0;
        padding: 0;
    }
    
    body {
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
        background: linear-gradient(135deg, var(--bg-gradient-1) 0%, var(--bg-gradient-2) 50%, var(--bg-gradient-3) 100%);
        min-height: 100vh;
        display: flex;
        align-items: center;
        justify-content: center;
        overflow: hidden;
    }
    
    .beautiful-counter {
        position: relative;
        width: 100%;
        max-width: 500px;
        margin: 0 20px;
    }
    
    .bg-decoration {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        pointer-events: none;
        overflow: hidden;
    }
    
    .circle {
        position: absolute;
        border-radius: 50%;
        background: rgba(255, 255, 255, 0.1);
        filter: blur(60px);
        animation: float 20s infinite ease-in-out;
    }
    
    .circle-1 {
        width: 400px;
        height: 400px;
        top: -200px;
        left: -100px;
        animation-delay: 0s;
    }
    
    .circle-2 {
        width: 300px;
        height: 300px;
        bottom: -150px;
        right: -100px;
        animation-delay: 5s;
    }
    
    .circle-3 {
        width: 250px;
        height: 250px;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        animation-delay: 10s;
    }
    
    @keyframes float {
        0%, 100% { transform: translate(0, 0) scale(1); }
        33% { transform: translate(30px, -30px) scale(1.1); }
        66% { transform: translate(-20px, 20px) scale(0.9); }
    }
    
    .counter-content {
        background: rgba(255, 255, 255, 0.95);
        backdrop-filter: blur(20px);
        border-radius: 30px;
        padding: 40px;
        box-shadow: 0 20px 60px rgba(0, 0, 0, 0.2);
        position: relative;
        z-index: 1;
    }
    
    header {
        text-align: center;
        margin-bottom: 40px;
    }
    
    h1 {
        font-size: 2.5rem;
        font-weight: 800;
        color: #1a202c;
        margin-bottom: 10px;
    }
    
    .gradient-text {
        background: linear-gradient(135deg, var(--primary), var(--secondary));
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        background-clip: text;
    }
    
    .subtitle {
        color: #718096;
        font-size: 1.1rem;
    }
    
    .counter-display {
        text-align: center;
        margin: 40px 0;
        transition: transform 0.3s ease;
    }
    
    .counter-display.animate-up {
        animation: bounceUp 0.3s ease;
    }
    
    .counter-display.animate-down {
        animation: bounceDown 0.3s ease;
    }
    
    .counter-display.animate-reset {
        animation: spin 0.5s ease;
    }
    
    @keyframes bounceUp {
        0% { transform: translateY(0); }
        50% { transform: translateY(-10px); }
        100% { transform: translateY(0); }
    }
    
    @keyframes bounceDown {
        0% { transform: translateY(0); }
        50% { transform: translateY(10px); }
        100% { transform: translateY(0); }
    }
    
    @keyframes spin {
        0% { transform: rotate(0deg) scale(1); }
        50% { transform: rotate(180deg) scale(0.8); }
        100% { transform: rotate(360deg) scale(1); }
    }
    
    .counter-value {
        font-size: 5rem;
        font-weight: 800;
        line-height: 1;
        transition: color 0.3s ease;
    }
    
    .counter-display.positive .counter-value {
        color: var(--success);
    }
    
    .counter-display.negative .counter-value {
        color: var(--danger);
    }
    
    .counter-display.zero .counter-value {
        color: var(--primary);
    }
    
    .counter-label {
        font-size: 1.2rem;
        color: #718096;
        margin-top: 10px;
        text-transform: uppercase;
        letter-spacing: 2px;
    }
    
    .quick-actions {
        display: flex;
        justify-content: center;
        gap: 10px;
        margin-bottom: 30px;
    }
    
    .quick-btn {
        background: #f7fafc;
        border: 2px solid #e2e8f0;
        border-radius: 10px;
        padding: 8px 16px;
        font-size: 0.9rem;
        font-weight: 600;
        color: #4a5568;
        cursor: pointer;
        transition: all 0.2s ease;
    }
    
    .quick-btn:hover {
        background: var(--primary);
        color: white;
        border-color: var(--primary);
        transform: translateY(-2px);
    }
    
    .main-controls {
        display: flex;
        gap: 15px;
        justify-content: center;
        margin-bottom: 40px;
    }
    
    .btn {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 12px 24px;
        border: none;
        border-radius: 15px;
        font-size: 1rem;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.3s ease;
        color: white;
        box-shadow: 0 4px 15px rgba(0, 0, 0, 0.1);
    }
    
    .btn-icon {
        font-size: 1.5rem;
        line-height: 1;
    }
    
    .btn-increment {
        background: linear-gradient(135deg, var(--success), #059669);
    }
    
    .btn-increment:hover {
        transform: translateY(-3px);
        box-shadow: 0 6px 20px rgba(16, 185, 129, 0.4);
    }
    
    .btn-decrement {
        background: linear-gradient(135deg, var(--danger), #dc2626);
    }
    
    .btn-decrement:hover {
        transform: translateY(-3px);
        box-shadow: 0 6px 20px rgba(239, 68, 68, 0.4);
    }
    
    .btn-reset {
        background: linear-gradient(135deg, var(--warning), #d97706);
    }
    
    .btn-reset:hover {
        transform: translateY(-3px);
        box-shadow: 0 6px 20px rgba(245, 158, 11, 0.4);
    }
    
    .stats {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 20px;
        margin-bottom: 30px;
    }
    
    .stat {
        text-align: center;
        padding: 15px;
        background: #f7fafc;
        border-radius: 10px;
    }
    
    .stat-label {
        font-size: 0.8rem;
        color: #718096;
        text-transform: uppercase;
        letter-spacing: 1px;
        margin-bottom: 5px;
    }
    
    .stat-value {
        font-size: 1.5rem;
        font-weight: 700;
        color: var(--primary);
    }
    
    footer {
        text-align: center;
        color: #718096;
        font-size: 0.9rem;
    }
    
    footer a {
        color: var(--primary);
        text-decoration: none;
        font-weight: 600;
        transition: color 0.2s ease;
    }
    
    footer a:hover {
        color: var(--primary-dark);
    }
    
    @media (max-width: 600px) {
        h1 {
            font-size: 2rem;
        }
        
        .counter-value {
            font-size: 4rem;
        }
        
        .main-controls {
            flex-direction: column;
        }
        
        .btn {
            width: 100%;
            justify-content: center;
        }
        
        .stats {
            grid-template-columns: 1fr;
        }
    }
"#;

#[wasm_bindgen(start)]
pub fn main() {
    web_sys::console::log_1(&"Beautiful Layer9 Counter starting...".into());
    mount(Box::new(BeautifulCounter), "root");
    web_sys::console::log_1(&"Beautiful Layer9 Counter mounted successfully!".into());
}