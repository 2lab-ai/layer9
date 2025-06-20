//! Basic HAF Tests

use crate::haf::{
    layers::{L1, L2, L3},
    Layer, Contract,
    component::{VNode, VProps},
};

#[test]
fn test_layer_types_exist() {
    // Verify layer types are zero-sized
    assert_eq!(std::mem::size_of::<L1>(), 0);
    assert_eq!(std::mem::size_of::<L2>(), 0);
    assert_eq!(std::mem::size_of::<L3>(), 0);
}

#[test]
fn test_contract_creation() {
    let input = "L1 data";
    let output = "L2 result";
    let contract = Contract::new(input, output);
    
    assert_eq!(contract.input, "L1 data");
    assert_eq!(contract.output, "L2 result");
}

#[test]
fn test_vnode_creation() {
    // Test text node
    let text_node: VNode<L1> = VNode::Text("Hello".to_string());
    assert!(matches!(text_node, VNode::Text(_)));
    
    // Test element node
    let element_node: VNode<L1> = VNode::Element {
        tag: "div".to_string(),
        props: VProps::default(),
        children: vec![],
    };
    assert!(matches!(element_node, VNode::Element { .. }));
}

#[test]
fn test_layer_depth() {
    assert_eq!(L1::DEPTH, 1);
    assert_eq!(L2::DEPTH, 2);
    assert_eq!(L3::DEPTH, 3);
}

#[test]
fn test_layer_names() {
    assert_eq!(L1::NAME, "Core");
    assert_eq!(L2::NAME, "Runtime");
    assert_eq!(L3::NAME, "Framework");
}