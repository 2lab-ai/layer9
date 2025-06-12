//! Async Counter Example with Suspense
//! Demonstrates async component loading with Suspense boundaries

use layer9_core::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

// Use `wee_alloc` as the global allocator for smaller bundle size
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Counter component that loads initial value asynchronously
struct AsyncCounter;

impl Component for AsyncCounter {
    fn render(&self) -> Element {
        // Use async data hook
        let initial_count = use_async_data::<i32>();
        
        // Local counter state
        let (count, set_count) = use_state(|| 0);
        
        // Load initial count on mount
        use_effect((), {
            let initial_count = initial_count.clone();
            let set_count = set_count.clone();
            move || {
                // Start loading
                initial_count.set_loading();
                
                // Spawn async task
                spawn_local(async move {
                    // Simulate API call delay
                    gloo_timers::future::TimeoutFuture::new(1000).await;
                    
                    // Set initial count
                    initial_count.set_success(42);
                    set_count(42);
                });
                
                || {}
            }
        });
        
        // Loading component
        let loading = view! {
            <div class="loading-container">
                <div class="spinner"></div>
                <p>"Loading initial count from server..."</p>
            </div>
        };
        
        // Error component
        let error_view = |err: &str| view! {
            <div class="error">
                <h3>"Failed to load initial count"</h3>
                <p>{err}</p>
            </div>
        };
        
        // Success view
        let counter_view = view! {
            <div class="counter">
                <h1>"Async Counter Example"</h1>
                <p class="count">"Count: "{count}</p>
                <div class="buttons">
                    <button onclick={move |_| set_count(count - 1)}>"-1"</button>
                    <button onclick={move |_| set_count(count + 1)}>"+1"</button>
                    <button onclick={move |_| set_count(0)}>"Reset"</button>
                </div>
                <AsyncDataDisplay count={count} />
            </div>
        };
        
        // Render based on async state
        match initial_count.get() {
            AsyncState::Loading => loading,
            AsyncState::Error(ref err) => error_view(err),
            AsyncState::Success(_) => counter_view,
        }
    }
}

/// Component that fetches data based on counter value
struct AsyncDataDisplay {
    count: i32,
}

impl Component for AsyncDataDisplay {
    fn render(&self) -> Element {
        let count = self.count;
        
        // Use async data hook
        let data = use_async_data::<String>();
        
        // Fetch data whenever count changes
        use_effect(count, {
            let data = data.clone();
            move || {
                data.set_loading();
                
                spawn_local(async move {
                    // Simulate API call
                    gloo_timers::future::TimeoutFuture::new(500).await;
                    
                    // Return different messages based on count
                    let message = match count {
                        0 => "Zero is where it all begins!",
                        n if n < 0 => "Going negative? That's bold!",
                        n if n > 100 => "Wow, that's a big number!",
                        n if n % 10 == 0 => "Nice round number!",
                        _ => "Keep counting!",
                    };
                    
                    data.set_success(message.to_string());
                });
                
                || {}
            }
        });
        
        view! {
            <div class="async-data">
                {match data.get() {
                    AsyncState::Loading => view! {
                        <span class="loading-text">"Fetching message..."</span>
                    },
                    AsyncState::Success(msg) => view! {
                        <p class="message">{msg}</p>
                    },
                    AsyncState::Error(ref err) => view! {
                        <p class="error-message">"Error: "{err}</p>
                    },
                }}
            </div>
        }
    }
}

/// App with Suspense and Error boundaries
struct App;

impl Component for App {
    fn render(&self) -> Element {
        // Custom loading component
        let loading = || view! {
            <div class="app-loading">
                <h1>"Async Counter"</h1>
                <div class="loader">
                    <div class="dot"></div>
                    <div class="dot"></div>
                    <div class="dot"></div>
                </div>
            </div>
        };
        
        // Error fallback
        let error_fallback = |error: &ErrorInfo| view! {
            <div class="error-boundary">
                <h2>"Something went wrong!"</h2>
                <p>{&error.message}</p>
                <button onclick={|_| web_sys::window().unwrap().location().reload().unwrap()}>
                    "Reload Page"
                </button>
            </div>
        };
        
        // Wrap app with error boundary
        with_error_boundary(
            AsyncCounter,
            error_fallback,
        ).render()
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    
    // Add styles
    inject_global_styles(r#"
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
        }
        
        .counter {
            background: white;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            text-align: center;
            min-width: 300px;
        }
        
        .count {
            font-size: 24px;
            margin: 20px 0;
            color: #333;
        }
        
        .buttons {
            display: flex;
            gap: 10px;
            justify-content: center;
            margin: 20px 0;
        }
        
        button {
            padding: 10px 20px;
            font-size: 16px;
            border: none;
            border-radius: 5px;
            background-color: #4CAF50;
            color: white;
            cursor: pointer;
            transition: background-color 0.3s;
        }
        
        button:hover {
            background-color: #45a049;
        }
        
        /* Loading styles */
        .loading-container, .app-loading {
            text-align: center;
            padding: 40px;
        }
        
        .spinner {
            border: 3px solid #f3f3f3;
            border-top: 3px solid #4CAF50;
            border-radius: 50%;
            width: 40px;
            height: 40px;
            animation: spin 1s linear infinite;
            margin: 0 auto 20px;
        }
        
        @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
        }
        
        .loader {
            display: flex;
            justify-content: center;
            gap: 10px;
            margin: 20px 0;
        }
        
        .dot {
            width: 10px;
            height: 10px;
            background-color: #4CAF50;
            border-radius: 50%;
            animation: bounce 1.4s infinite ease-in-out both;
        }
        
        .dot:nth-child(1) { animation-delay: -0.32s; }
        .dot:nth-child(2) { animation-delay: -0.16s; }
        
        @keyframes bounce {
            0%, 80%, 100% {
                transform: scale(0);
            }
            40% {
                transform: scale(1);
            }
        }
        
        /* Async data styles */
        .async-data {
            margin-top: 20px;
            padding: 15px;
            background-color: #f9f9f9;
            border-radius: 5px;
            min-height: 50px;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        
        .loading-text {
            color: #666;
            font-style: italic;
        }
        
        .message {
            color: #4CAF50;
            font-weight: bold;
            margin: 0;
        }
        
        /* Error styles */
        .error, .error-boundary {
            background-color: #fee;
            border: 1px solid #fcc;
            padding: 20px;
            border-radius: 5px;
            color: #c00;
            text-align: center;
        }
        
        .error-message {
            color: #c00;
            margin: 0;
        }
    "#);
    
    // Mount app
    mount(Box::new(App), "root");
}

// Required dependencies for async operations
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window)]
    fn setTimeout(closure: &Closure<dyn FnMut()>, millis: u32) -> u32;
}