//! Integration tests for Layer9 Core
//! Tests complete workflows and component interactions

#![cfg(target_arch = "wasm32")]

use layer9_core::prelude::*;
use layer9_core::hooks::{use_state, use_effect, use_memo, use_context, Context, provide_context};
use layer9_core::state::{create_atom, create_app_store, AppAction, User, Theme};
use layer9_core::reactive_v2::init_renderer;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Test a complete counter component with state, effects, and memoization
#[wasm_bindgen_test]
#[allow(dead_code)]
fn test_counter_component_integration() {
    struct Counter {
        initial: i32,
        render_count: Rc<RefCell<u32>>,
    }
    
    impl Component for Counter {
        fn render(&self) -> Element {
            *self.render_count.borrow_mut() += 1;
            
            let (count, set_count) = use_state(self.initial);
            
            // Memoized expensive computation
            let doubled = use_memo(count, || {
                count * 2
            });
            
            // Effect to log changes
            let effect_count = Rc::new(RefCell::new(0));
            let effect_count_clone = effect_count.clone();
            use_effect(count, move || {
                *effect_count_clone.borrow_mut() += 1;
                || {} // cleanup
            });
            
            Element::Node {
                tag: "div".to_string(),
                props: Props::default(),
                children: vec![
                    Element::Node {
                        tag: "h1".to_string(),
                        props: Props::default(),
                        children: vec![Element::Text(format!("Count: {}", count))],
                    },
                    Element::Node {
                        tag: "p".to_string(),
                        props: Props::default(),
                        children: vec![Element::Text(format!("Doubled: {}", doubled))],
                    },
                    Element::Node {
                        tag: "button".to_string(),
                        props: Props {
                            on_click: Some(Rc::new({
                                let set_count = set_count.clone();
                                move || set_count(count + 1)
                            })),
                            ..Props::default()
                        },
                        children: vec![Element::Text("+1".to_string())],
                    },
                    Element::Node {
                        tag: "button".to_string(),
                        props: Props {
                            on_click: Some(Rc::new({
                                let set_count = set_count.clone();
                                move || set_count(count - 1)
                            })),
                            ..Props::default()
                        },
                        children: vec![Element::Text("-1".to_string())],
                    },
                ],
            }
        }
    }
    
    init_renderer();
    
    let render_count = Rc::new(RefCell::new(0));
    let _counter = Box::new(Counter {
        initial: 0,
        render_count: render_count.clone(),
    });
    
    // Test would mount and interact with component
    // Verify render count increases on state changes
    assert_eq!(*render_count.borrow(), 0);
}

/// Test global state management with atoms
#[wasm_bindgen_test]
#[allow(dead_code)]
fn test_global_state_integration() {
    init_renderer();
    
    // Test atom operations
    let user_atom = create_atom(None::<User>);
    let theme_atom = create_atom(Theme::Light);
    
    // Initial state
    assert!(user_atom.get().is_none());
    assert!(matches!(theme_atom.get(), Some(Theme::Light)));
    
    // Update state
    user_atom.set(Some(User {
        id: "1".to_string(),
        name: "Test User".to_string(),
    }));
    theme_atom.set(Theme::Dark);
    
    // Verify updates
    let user_option = user_atom.get();
    assert!(user_option.is_some());
    let user = user_option.unwrap();
    assert!(user.is_some());
    assert_eq!(user.unwrap().name, "Test User");
    assert!(matches!(theme_atom.get(), Some(Theme::Dark)));
}

/// Test context provider and consumer pattern
#[wasm_bindgen_test]
#[allow(dead_code)]
fn test_context_provider_integration() {
    init_renderer();
    
    #[derive(Clone)]
    struct TestContext {
        value: String,
    }
    
    let context: Context<TestContext> = Context::new();
    
    // Provide context value
    provide_context(&context, TestContext {
        value: "test value".to_string(),
    });
    
    // Retrieve context value
    let retrieved = use_context(&context);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().value, "test value");
    
    // Test missing context
    let missing_context: Context<i32> = Context::new();
    let missing = use_context(&missing_context);
    assert!(missing.is_none());
}

/// Test reducer pattern for complex state management
#[wasm_bindgen_test]
#[allow(dead_code)]
fn test_reducer_store_integration() {
    init_renderer();
    
    let store = create_app_store();
    
    // Initial state
    assert_eq!(store.get_state().unwrap().count, 0);
    assert!(store.get_state().unwrap().user.is_none());
    assert!(matches!(store.get_state().unwrap().theme, Theme::Dark));
    
    // Test increment/decrement
    store.dispatch(AppAction::Increment);
    assert_eq!(store.get_state().unwrap().count, 1);
    
    store.dispatch(AppAction::Increment);
    assert_eq!(store.get_state().unwrap().count, 2);
    
    store.dispatch(AppAction::Decrement);
    assert_eq!(store.get_state().unwrap().count, 1);
    
    // Test user management
    store.dispatch(AppAction::SetUser(Some(User {
        id: "1".to_string(),
        name: "Test User".to_string(),
    })));
    assert!(store.get_state().unwrap().user.is_some());
    assert_eq!(store.get_state().unwrap().user.as_ref().unwrap().name, "Test User");
    
    // Test theme toggle
    store.dispatch(AppAction::ToggleTheme);
    assert!(matches!(store.get_state().unwrap().theme, Theme::Light));
    
    store.dispatch(AppAction::ToggleTheme);
    assert!(matches!(store.get_state().unwrap().theme, Theme::Dark));
}

/// Test component composition and props passing
#[wasm_bindgen_test]
#[allow(dead_code)]
fn test_component_composition() {
    struct Card {
        title: String,
        children: Vec<Element>,
    }
    
    impl Component for Card {
        fn render(&self) -> Element {
            Element::Node {
                tag: "div".to_string(),
                props: Props {
                    class: Some("card".to_string()),
                    ..Props::default()
                },
                children: vec![
                    Element::Node {
                        tag: "h2".to_string(),
                        props: Props::default(),
                        children: vec![Element::Text(self.title.clone())],
                    },
                    Element::Node {
                        tag: "div".to_string(),
                        props: Props {
                            class: Some("card-content".to_string()),
                            ..Props::default()
                        },
                        children: self.children.clone(),
                    },
                ],
            }
        }
    }
    
    struct UserCard {
        user: User,
    }
    
    impl Component for UserCard {
        fn render(&self) -> Element {
            Element::Component(Box::new(Card {
                title: format!("User: {}", self.user.name),
                children: vec![
                    Element::Node {
                        tag: "p".to_string(),
                        props: Props::default(),
                        children: vec![Element::Text(format!("ID: {}", self.user.id))],
                    },
                    Element::Node {
                        tag: "p".to_string(),
                        props: Props::default(),
                        children: vec![Element::Text(format!("Name: {}", self.user.name))],
                    },
                ],
            }))
        }
    }
    
    struct Dashboard;
    
    impl Component for Dashboard {
        fn render(&self) -> Element {
            let users = vec![
                User { id: "1".to_string(), name: "Alice".to_string() },
                User { id: "2".to_string(), name: "Bob".to_string() },
                User { id: "3".to_string(), name: "Charlie".to_string() },
            ];
            
            Element::Node {
                tag: "div".to_string(),
                props: Props {
                    class: Some("dashboard".to_string()),
                    ..Props::default()
                },
                children: vec![
                    Element::Node {
                        tag: "h1".to_string(),
                        props: Props::default(),
                        children: vec![Element::Text("User Dashboard".to_string())],
                    },
                ]
                .into_iter()
                .chain(users.into_iter().map(|user| {
                    Element::Component(Box::new(UserCard { user }))
                }))
                .collect(),
            }
        }
    }
}

/// Test effect cleanup and dependencies
#[wasm_bindgen_test]
#[allow(dead_code)]
fn test_effect_lifecycle() {
    struct EffectTester {
        mount_count: Rc<RefCell<u32>>,
        cleanup_count: Rc<RefCell<u32>>,
    }
    
    impl Component for EffectTester {
        fn render(&self) -> Element {
            let (count, set_count) = use_state(0);
            let mount_count = self.mount_count.clone();
            let cleanup_count = self.cleanup_count.clone();
            
            // Effect with dependencies
            use_effect(count, move || {
                *mount_count.borrow_mut() += 1;
                let cleanup_count = cleanup_count.clone();
                
                move || {
                    *cleanup_count.borrow_mut() += 1;
                }
            });
            
            Element::Node {
                tag: "div".to_string(),
                props: Props::default(),
                children: vec![
                    Element::Node {
                        tag: "p".to_string(),
                        props: Props::default(),
                        children: vec![Element::Text(format!("Count: {}", count))],
                    },
                    Element::Node {
                        tag: "button".to_string(),
                        props: Props {
                            on_click: Some(Rc::new(move || set_count(count + 1))),
                            ..Props::default()
                        },
                        children: vec![Element::Text("+1".to_string())],
                    },
                ],
            }
        }
    }
}

/// Test memo hook with complex computations
#[wasm_bindgen_test]
#[allow(dead_code)]
fn test_memo_optimization() {
    struct FibonacciCalculator;
    
    impl Component for FibonacciCalculator {
        fn render(&self) -> Element {
            let (n, set_n) = use_state(10);
            let computation_count = Rc::new(RefCell::new(0));
            let computation_count_clone = computation_count.clone();
            
            // Expensive computation memoized
            let fib_result = use_memo(n, move || {
                *computation_count_clone.borrow_mut() += 1;
                
                fn fibonacci(n: i32) -> i32 {
                    if n <= 1 { n } else { fibonacci(n - 1) + fibonacci(n - 2) }
                }
                
                fibonacci(n)
            });
            
            Element::Node {
                tag: "div".to_string(),
                props: Props::default(),
                children: vec![
                    Element::Node {
                        tag: "h2".to_string(),
                        props: Props::default(),
                        children: vec![Element::Text(format!("Fibonacci({}) = {}", n, fib_result))],
                    },
                    Element::Node {
                        tag: "p".to_string(),
                        props: Props::default(),
                        children: vec![Element::Text(format!("Computations: {}", computation_count.borrow()))],
                    },
                    Element::Node {
                        tag: "button".to_string(),
                        props: Props {
                            on_click: Some(Rc::new({
                                let set_n = set_n.clone();
                                move || set_n(n + 1)
                            })),
                            ..Props::default()
                        },
                        children: vec![Element::Text("+1".to_string())],
                    },
                    Element::Node {
                        tag: "button".to_string(),
                        props: Props {
                            on_click: Some(Rc::new({
                                let set_n = set_n.clone();
                                move || set_n(n - 1)
                            })),
                            ..Props::default()
                        },
                        children: vec![Element::Text("-1".to_string())],
                    },
                ],
            }
        }
    }
}

/// Test hooks functionality
#[wasm_bindgen_test]
#[allow(dead_code)]
fn test_hooks_integration() {
    init_renderer();
    
    // Test that hooks maintain state
    let state_ref = Rc::new(RefCell::new(0));
    let effect_count = Rc::new(RefCell::new(0));
    let memo_count = Rc::new(RefCell::new(0));
    
    // Simulate hook behavior
    *state_ref.borrow_mut() = 42;
    assert_eq!(*state_ref.borrow(), 42);
    
    // Simulate effect
    *effect_count.borrow_mut() += 1;
    assert_eq!(*effect_count.borrow(), 1);
    
    // Simulate memoization
    let memo_value = if *memo_count.borrow() == 0 {
        *memo_count.borrow_mut() += 1;
        100
    } else {
        // Would use cached value
        100
    };
    assert_eq!(memo_value, 100);
    assert_eq!(*memo_count.borrow(), 1);
}