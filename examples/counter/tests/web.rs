//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

use layer9_core::component::{Component, State};
use layer9_example_counter::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_counter_initial_state() {
    // Test that counter starts at 0
    let count_state = State::new(0);
    assert_eq!(count_state.get(), 0);
}

#[wasm_bindgen_test]
fn test_counter_increment() {
    // Test increment functionality
    let count_state = State::new(0);
    count_state.set(count_state.get() + 1);
    assert_eq!(count_state.get(), 1);
}

#[wasm_bindgen_test]
fn test_counter_decrement() {
    // Test decrement functionality
    let count_state = State::new(5);
    count_state.set(count_state.get() - 1);
    assert_eq!(count_state.get(), 4);
}

#[wasm_bindgen_test]
fn test_counter_reset() {
    // Test reset functionality
    let count_state = State::new(10);
    count_state.set(0);
    assert_eq!(count_state.get(), 0);
}

#[wasm_bindgen_test]
fn test_component_render() {
    // Test that component can render without panicking
    use layer9_core::component::Element;
    
    let element = Element::Node {
        tag: "div".to_string(),
        props: layer9_core::component::Props::default(),
        children: vec![
            Element::Text("Test Counter".to_string())
        ],
    };
    
    // Convert to DOM should not panic
    let _dom_node = element.to_dom();
}

#[wasm_bindgen_test]
async fn test_state_update_callback() {
    use std::rc::Rc;
    use std::cell::RefCell;
    
    // Test that state update callbacks are triggered
    let callback_called = Rc::new(RefCell::new(false));
    let callback_called_clone = callback_called.clone();
    
    let count_state = State::new(0);
    count_state.set_update_callback(Box::new(move || {
        *callback_called_clone.borrow_mut() = true;
    }));
    
    // Trigger state update
    count_state.set(1);
    
    // Callback should have been called
    assert!(*callback_called.borrow());
}