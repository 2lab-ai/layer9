//! Beautiful Async Counter Example
//! Demonstrates async component loading with stunning UI

use layer9_core::prelude::*;
use layer9_core::hooks::use_state;
use layer9_core::reactive_v2::mount;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct AsyncCounter;

impl Component for AsyncCounter {
    fn render(&self) -> Element {
        let (count, set_count) = use_state(0i32);
        let (loading, set_loading) = use_state(true);
        let (message, set_message) = use_state("Loading...".to_string());
        let (error, set_error) = use_state(false);
        
        // Simulate async data loading on mount
        use_effect((), {
            let set_count = set_count.clone();
            let set_loading = set_loading.clone();
            let set_message = set_message.clone();
            move || {
                spawn_local(async move {
                    // Simulate API delay
                    let window = web_sys::window().unwrap();
                    let promise = js_sys::Promise::new(&mut |resolve, _| {
                        window.set_timeout_with_callback_and_timeout_and_arguments_0(
                            &resolve,
                            1500
                        ).unwrap();
                    });
                    wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
                    
                    // Set initial count from "server"
                    set_count(42);
                    set_loading(false);
                    set_message("Initial count loaded from server!".to_string());
                });
                || {}
            }
        });
        
        // Update message based on count
        use_effect(count, {
            let count = count;
            let set_message = set_message.clone();
            move || {
                if !loading {
                    let msg = match count {
                        0 => "Zero - The beginning of everything! 🌟",
                        n if n < 0 => "Venturing into negative territory! ❄️",
                        n if n > 100 => "Triple digits! You're on fire! 🔥",
                        n if n == 42 => "The answer to everything! 🌌",
                        n if n % 10 == 0 => "Perfect ten! Nice round number! ✨",
                        _ => "Keep counting, you're doing great! 🚀",
                    };
                    set_message(msg.to_string());
                }
                || {}
            }
        });
        
        let increment = {
            let count = count;
            let set_count = set_count.clone();
            move || set_count(count + 1)
        };
        
        let decrement = {
            let count = count;
            let set_count = set_count.clone();
            move || set_count(count - 1)
        };
        
        let async_reset = {
            let set_count = set_count.clone();
            let set_loading = set_loading.clone();
            let set_message = set_message.clone();
            move || {
                let set_count = set_count.clone();
                let set_loading = set_loading.clone();
                let set_message = set_message.clone();
                
                set_loading(true);
                set_message("Resetting...".to_string());
                
                spawn_local(async move {
                    // Simulate async reset
                    let window = web_sys::window().unwrap();
                    let promise = js_sys::Promise::new(&mut |resolve, _| {
                        window.set_timeout_with_callback_and_timeout_and_arguments_0(
                            &resolve,
                            800
                        ).unwrap();
                    });
                    wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
                    
                    set_count(0);
                    set_loading(false);
                    set_message("Reset complete! Fresh start! 🎯".to_string());
                });
            }
        };
        
        let async_random = {
            let set_count = set_count.clone();
            let set_loading = set_loading.clone();
            let set_message = set_message.clone();
            move || {
                let set_count = set_count.clone();
                let set_loading = set_loading.clone();
                let set_message = set_message.clone();
                
                set_loading(true);
                set_message("Fetching random number...".to_string());
                
                spawn_local(async move {
                    // Simulate API call for random number
                    let window = web_sys::window().unwrap();
                    let promise = js_sys::Promise::new(&mut |resolve, _| {
                        window.set_timeout_with_callback_and_timeout_and_arguments_0(
                            &resolve,
                            1000
                        ).unwrap();
                    });
                    wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
                    
                    let random = (js_sys::Math::random() * 100.0) as i32;
                    set_count(random);
                    set_loading(false);
                    set_message(format!("Random number {} loaded! 🎲", random));
                });
            }
        };

        Element::Node {
            tag: "div".to_string(),
            props: Props {
                class: Some("async-counter-app".to_string()),
                ..Default::default()
            },
            children: vec![
                // Inline styles
                Element::Node {
                    tag: "style".to_string(),
                    props: Props::default(),
                    children: vec![Element::Text(ASYNC_STYLES.to_string())],
                },
                
                // Background decorations
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
                                class: Some("pulse pulse-1".to_string()),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                        Element::Node {
                            tag: "div".to_string(),
                            props: Props {
                                class: Some("pulse pulse-2".to_string()),
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
                        class: Some("content-card".to_string()),
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
                                            children: vec![Element::Text("Async".to_string())],
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
                                    children: vec![Element::Text("Experience asynchronous state management".to_string())],
                                },
                            ],
                        },
                        
                        // Counter display
                        Element::Node {
                            tag: "div".to_string(),
                            props: Props {
                                class: Some(if loading { "counter-display loading" } else { "counter-display" }.to_string()),
                                ..Default::default()
                            },
                            children: vec![
                                if loading {
                                    Element::Node {
                                        tag: "div".to_string(),
                                        props: Props {
                                            class: Some("loading-spinner".to_string()),
                                            ..Default::default()
                                        },
                                        children: vec![],
                                    }
                                } else {
                                    Element::Node {
                                        tag: "div".to_string(),
                                        props: Props {
                                            class: Some("counter-value".to_string()),
                                            ..Default::default()
                                        },
                                        children: vec![Element::Text(count.to_string())],
                                    }
                                },
                            ],
                        },
                        
                        // Message display
                        Element::Node {
                            tag: "div".to_string(),
                            props: Props {
                                class: Some("message-display".to_string()),
                                ..Default::default()
                            },
                            children: vec![
                                Element::Node {
                                    tag: "p".to_string(),
                                    props: Props {
                                        class: Some(if loading { "message loading" } else { "message" }.to_string()),
                                        ..Default::default()
                                    },
                                    children: vec![Element::Text(message.clone())],
                                },
                            ],
                        },
                        
                        // Control buttons
                        Element::Node {
                            tag: "div".to_string(),
                            props: Props {
                                class: Some("controls".to_string()),
                                ..Default::default()
                            },
                            children: vec![
                                // Sync buttons
                                Element::Node {
                                    tag: "div".to_string(),
                                    props: Props {
                                        class: Some("sync-controls".to_string()),
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
                                            children: vec![Element::Text("− Decrement".to_string())],
                                        },
                                        Element::Node {
                                            tag: "button".to_string(),
                                            props: Props {
                                                class: Some("btn btn-increment".to_string()),
                                                on_click: Some(Rc::new(increment)),
                                                ..Default::default()
                                            },
                                            children: vec![Element::Text("+ Increment".to_string())],
                                        },
                                    ],
                                },
                                
                                // Async buttons
                                Element::Node {
                                    tag: "div".to_string(),
                                    props: Props {
                                        class: Some("async-controls".to_string()),
                                        ..Default::default()
                                    },
                                    children: vec![
                                        Element::Node {
                                            tag: "button".to_string(),
                                            props: Props {
                                                class: Some(if loading { "btn btn-async disabled" } else { "btn btn-async" }.to_string()),
                                                on_click: if loading { None } else { Some(Rc::new(async_reset)) },
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
                                                Element::Text(" Async Reset".to_string()),
                                            ],
                                        },
                                        Element::Node {
                                            tag: "button".to_string(),
                                            props: Props {
                                                class: Some(if loading { "btn btn-async disabled" } else { "btn btn-async" }.to_string()),
                                                on_click: if loading { None } else { Some(Rc::new(async_random)) },
                                                ..Default::default()
                                            },
                                            children: vec![
                                                Element::Node {
                                                    tag: "span".to_string(),
                                                    props: Props {
                                                        class: Some("btn-icon".to_string()),
                                                        ..Default::default()
                                                    },
                                                    children: vec![Element::Text("🎲".to_string())],
                                                },
                                                Element::Text(" Random".to_string()),
                                            ],
                                        },
                                    ],
                                },
                            ],
                        },
                        
                        // Footer
                        Element::Node {
                            tag: "footer".to_string(),
                            props: Props::default(),
                            children: vec![
                                Element::Text("Powered by ".to_string()),
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
                                Element::Text(" • Async Rust Web Framework".to_string()),
                            ],
                        },
                    ],
                },
            ],
        }
    }
}

const ASYNC_STYLES: &str = r#"
    :root {
        --async-primary: #8b5cf6;
        --async-secondary: #ec4899;
        --async-success: #10b981;
        --async-warning: #f59e0b;
        --async-gradient-1: #8b5cf6;
        --async-gradient-2: #ec4899;
        --async-gradient-3: #06b6d4;
    }
    
    * {
        box-sizing: border-box;
        margin: 0;
        padding: 0;
    }
    
    body {
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        background: linear-gradient(135deg, var(--async-gradient-1), var(--async-gradient-2), var(--async-gradient-3));
        min-height: 100vh;
        display: flex;
        align-items: center;
        justify-content: center;
        overflow: hidden;
    }
    
    .async-counter-app {
        position: relative;
        width: 100%;
        max-width: 600px;
        margin: 0 20px;
    }
    
    .bg-decoration {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        pointer-events: none;
    }
    
    .pulse {
        position: absolute;
        border-radius: 50%;
        background: rgba(255, 255, 255, 0.1);
        animation: pulse 4s ease-in-out infinite;
    }
    
    .pulse-1 {
        width: 600px;
        height: 600px;
        top: -300px;
        right: -300px;
    }
    
    .pulse-2 {
        width: 400px;
        height: 400px;
        bottom: -200px;
        left: -200px;
        animation-delay: 2s;
    }
    
    @keyframes pulse {
        0%, 100% {
            transform: scale(1);
            opacity: 0.1;
        }
        50% {
            transform: scale(1.2);
            opacity: 0.3;
        }
    }
    
    .content-card {
        background: rgba(255, 255, 255, 0.95);
        backdrop-filter: blur(20px);
        border-radius: 30px;
        padding: 50px;
        box-shadow: 0 25px 50px rgba(0, 0, 0, 0.15);
        position: relative;
        z-index: 1;
    }
    
    header {
        text-align: center;
        margin-bottom: 40px;
    }
    
    h1 {
        font-size: 3rem;
        font-weight: 800;
        color: #1a202c;
        margin-bottom: 10px;
    }
    
    .gradient-text {
        background: linear-gradient(135deg, var(--async-primary), var(--async-secondary));
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        background-clip: text;
    }
    
    .subtitle {
        color: #64748b;
        font-size: 1.2rem;
    }
    
    .counter-display {
        text-align: center;
        margin: 50px 0;
        min-height: 120px;
        display: flex;
        align-items: center;
        justify-content: center;
    }
    
    .counter-display.loading {
        opacity: 0.7;
    }
    
    .counter-value {
        font-size: 6rem;
        font-weight: 800;
        background: linear-gradient(135deg, var(--async-primary), var(--async-secondary));
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        background-clip: text;
        animation: fadeIn 0.5s ease;
    }
    
    @keyframes fadeIn {
        from {
            opacity: 0;
            transform: scale(0.8);
        }
        to {
            opacity: 1;
            transform: scale(1);
        }
    }
    
    .loading-spinner {
        width: 80px;
        height: 80px;
        border: 4px solid #f3f4f6;
        border-top: 4px solid var(--async-primary);
        border-radius: 50%;
        animation: spin 1s linear infinite;
    }
    
    @keyframes spin {
        to { transform: rotate(360deg); }
    }
    
    .message-display {
        text-align: center;
        margin-bottom: 40px;
        min-height: 30px;
    }
    
    .message {
        font-size: 1.2rem;
        color: #64748b;
        transition: all 0.3s ease;
    }
    
    .message.loading {
        color: var(--async-primary);
        font-style: italic;
    }
    
    .controls {
        display: flex;
        flex-direction: column;
        gap: 20px;
    }
    
    .sync-controls, .async-controls {
        display: flex;
        gap: 15px;
        justify-content: center;
    }
    
    .btn {
        padding: 15px 30px;
        border: none;
        border-radius: 15px;
        font-size: 1.1rem;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.3s ease;
        display: flex;
        align-items: center;
        gap: 8px;
        box-shadow: 0 4px 15px rgba(0, 0, 0, 0.1);
    }
    
    .btn-icon {
        font-size: 1.4rem;
    }
    
    .btn-increment {
        background: linear-gradient(135deg, var(--async-success), #059669);
        color: white;
    }
    
    .btn-increment:hover {
        transform: translateY(-2px);
        box-shadow: 0 6px 20px rgba(16, 185, 129, 0.4);
    }
    
    .btn-decrement {
        background: linear-gradient(135deg, var(--async-warning), #d97706);
        color: white;
    }
    
    .btn-decrement:hover {
        transform: translateY(-2px);
        box-shadow: 0 6px 20px rgba(245, 158, 11, 0.4);
    }
    
    .btn-async {
        background: linear-gradient(135deg, var(--async-primary), var(--async-secondary));
        color: white;
    }
    
    .btn-async:hover:not(.disabled) {
        transform: translateY(-2px);
        box-shadow: 0 6px 20px rgba(139, 92, 246, 0.4);
    }
    
    .btn.disabled {
        opacity: 0.5;
        cursor: not-allowed;
        transform: none !important;
    }
    
    footer {
        text-align: center;
        margin-top: 40px;
        color: #64748b;
        font-size: 0.9rem;
    }
    
    footer a {
        color: var(--async-primary);
        text-decoration: none;
        font-weight: 600;
    }
    
    footer a:hover {
        text-decoration: underline;
    }
    
    @media (max-width: 600px) {
        h1 {
            font-size: 2rem;
        }
        
        .counter-value {
            font-size: 4rem;
        }
        
        .content-card {
            padding: 30px;
        }
        
        .sync-controls, .async-controls {
            flex-direction: column;
        }
        
        .btn {
            width: 100%;
            justify-content: center;
        }
    }
"#;

#[wasm_bindgen(start)]
pub fn main() {
    web_sys::console::log_1(&"Beautiful Async Counter starting...".into());
    mount(Box::new(AsyncCounter), "root");
    web_sys::console::log_1(&"Beautiful Async Counter mounted successfully!".into());
}