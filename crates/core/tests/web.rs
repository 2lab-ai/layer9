//! WASM tests for layer9-core

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

use layer9_core::{
    component::{Component, Element, Props, State},
    router_v2::{route, Router},
    state::{create_atom, use_atom},
};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_element_creation() {
    let element = Element::Text("Hello World".to_string());
    match element {
        Element::Text(text) => assert_eq!(text, "Hello World"),
        _ => panic!("Expected text element"),
    }
}

#[wasm_bindgen_test]
fn test_node_element() {
    let node = Element::Node {
        tag: "div".to_string(),
        props: Props::default(),
        children: vec![Element::Text("Child".to_string())],
    };
    
    match node {
        Element::Node { tag, children, .. } => {
            assert_eq!(tag, "div");
            assert_eq!(children.len(), 1);
        }
        _ => panic!("Expected node element"),
    }
}

#[wasm_bindgen_test]
fn test_state_management() {
    let state = State::new(42);
    assert_eq!(state.get(), 42);
    
    state.set(100);
    assert_eq!(state.get(), 100);
}

#[wasm_bindgen_test]
fn test_atom_state() {
    let count_atom = create_atom(0);
    let count = use_atom(&count_atom);
    
    assert_eq!(count.get(), 0);
    count.set(5);
    assert_eq!(count.get(), 5);
}

#[wasm_bindgen_test]
fn test_router_pattern_matching() {
    let router = Router::new();
    
    // Test exact match
    if let Some(route) = router.match_route("/home") {
        assert_eq!(route.path, "/home");
    }
    
    // Test parameter extraction
    if let Some(route) = router.match_route("/user/123") {
        assert!(route.params.contains_key("id"));
    }
}

#[wasm_bindgen_test]
fn test_props_builder() {
    let mut props = Props::default();
    props.class = Some("test-class".to_string());
    props.id = Some("test-id".to_string());
    
    assert_eq!(props.class, Some("test-class".to_string()));
    assert_eq!(props.id, Some("test-id".to_string()));
}

#[wasm_bindgen_test]
async fn test_element_to_dom() {
    let element = Element::Node {
        tag: "button".to_string(),
        props: Props {
            class: Some("btn".to_string()),
            id: Some("test-btn".to_string()),
            on_click: None,
            attributes: vec![("data-test".to_string(), "true".to_string())],
        },
        children: vec![Element::Text("Click me".to_string())],
    };
    
    let dom_node = element.to_dom();
    
    // Verify DOM node was created correctly
    let element = dom_node.dyn_into::<web_sys::Element>().unwrap();
    assert_eq!(element.tag_name(), "BUTTON");
    assert_eq!(element.class_name(), "btn");
    assert_eq!(element.id(), "test-btn");
    assert_eq!(element.get_attribute("data-test"), Some("true".to_string()));
    assert_eq!(element.text_content(), Some("Click me".to_string()));
}