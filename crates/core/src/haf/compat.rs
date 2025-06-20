//! Compatibility layer for migrating from old component system to HAF
//! 
//! This module provides adapters and utilities to help migrate existing
//! Layer9 components to the HAF architecture gradually.

use crate::component as old;
use crate::haf::component::*;
use crate::haf::layers::L1;
use crate::haf::Layer;

/// Adapter to convert old Element to HAF VNode
pub fn element_to_vnode(element: &old::Element) -> VNode<L1> {
    match element {
        old::Element::Text(text) => VNode::Text(text.clone()),
        old::Element::Node { tag, props, children } => {
            VNode::Element {
                tag: tag.clone(),
                props: props_to_vprops(props),
                children: children.iter().map(element_to_vnode).collect(),
            }
        }
        old::Element::Component(_) => {
            // For now, wrap old components as text
            // In production, we'd need a proper adapter
            VNode::Text("[Legacy Component]".to_string())
        }
    }
}

/// Convert old Props to HAF VProps
fn props_to_vprops(props: &old::Props) -> VProps {
    let mut vprops = VProps {
        class: props.class.clone(),
        id: props.id.clone(),
        attributes: props.attributes.clone(),
        ..Default::default()
    };
    
    // Events need to be registered separately in HAF
    // This is just a placeholder
    if props.on_click.is_some() {
        vprops.events.push(("click".to_string(), EventId(1)));
    }
    if props.on_submit.is_some() {
        vprops.events.push(("submit".to_string(), EventId(2)));
    }
    if props.on_change.is_some() {
        vprops.events.push(("change".to_string(), EventId(3)));
    }
    if props.on_input.is_some() {
        vprops.events.push(("input".to_string(), EventId(4)));
    }
    
    vprops
}

/// Wrapper to use old components in HAF system
pub struct LegacyComponentWrapper<C: old::Component> {
    component: C,
}

impl<C: old::Component> PureComponent<L1> for LegacyComponentWrapper<C> {
    type Props = ();
    
    fn render(&self, _props: &Self::Props) -> VNode<L1> {
        let old_element = self.component.render();
        element_to_vnode(&old_element)
    }
}

/// Bridge for using HAF components in old system
pub struct HafComponentBridge<L: Layer> {
    vnode: VNode<L>,
}

impl<L: Layer> HafComponentBridge<L> {
    pub fn new(vnode: VNode<L>) -> Self {
        Self { vnode }
    }
}

impl<L: Layer> old::Component for HafComponentBridge<L> {
    fn render(&self) -> old::Element {
        vnode_to_element(&self.vnode)
    }
}

/// Convert HAF VNode to old Element
fn vnode_to_element<L: Layer>(vnode: &VNode<L>) -> old::Element {
    match vnode {
        VNode::Text(text) => old::Element::Text(text.clone()),
        VNode::Element { tag, props, children } => {
            old::Element::Node {
                tag: tag.clone(),
                props: vprops_to_props(props),
                children: children.iter().map(vnode_to_element).collect(),
            }
        }
        VNode::Component { .. } => {
            // Placeholder for component conversion
            old::Element::Text("[HAF Component]".to_string())
        }
        VNode::Fragment(children) => {
            // Old system doesn't have fragments, wrap in div
            old::Element::Node {
                tag: "div".to_string(),
                props: old::Props::default(),
                children: children.iter().map(vnode_to_element).collect(),
            }
        }
        _ => old::Element::Text("".to_string()),
    }
}

/// Convert HAF VProps to old Props
fn vprops_to_props(vprops: &VProps) -> old::Props {
    old::Props {
        class: vprops.class.clone(),
        id: vprops.id.clone(),
        attributes: vprops.attributes.clone(),
        // Events would need proper handling with closures
        on_click: None,
        on_submit: None,
        on_change: None,
        on_input: None,
    }
}

/// Migration helper macros
#[macro_export]
macro_rules! migrate_component {
    ($old_component:ty) => {
        impl $crate::haf::component::PureComponent<$crate::haf::L1> for $old_component {
            type Props = ();
            
            fn render(&self, _props: &Self::Props) -> $crate::haf::component::VNode<$crate::haf::L1> {
                let old_element = <Self as $crate::component::Component>::render(self);
                $crate::haf::compat::element_to_vnode(&old_element)
            }
        }
    };
}

/// Feature flag for HAF components
#[cfg(feature = "haf")]
pub use crate::haf::component as component;

#[cfg(not(feature = "haf"))]
pub use crate::component;

/// Example migration path
#[cfg(test)]
#[allow(dead_code)]
mod compat_tests {
    use super::*;
    use crate::component::Component;
    
    struct OldButton {
        label: String,
    }
    
    impl old::Component for OldButton {
        fn render(&self) -> old::Element {
            old::Element::Node {
                tag: "button".to_string(),
                props: old::Props {
                    class: Some("btn".to_string()),
                    ..Default::default()
                },
                children: vec![old::Element::Text(self.label.clone())],
            }
        }
    }
    
    #[test]
    fn test_old_to_haf_migration() {
        let old_button = OldButton {
            label: "Click me".to_string(),
        };
        
        let old_element = old_button.render();
        let haf_vnode = element_to_vnode(&old_element);
        
        match haf_vnode {
            VNode::Element { tag, .. } => {
                assert_eq!(tag, "button");
            }
            _ => panic!("Expected Element VNode"),
        }
    }
}

// Migration guide documentation
// 
// # Migrating from Old Component System to HAF
// 
// ## Step 1: Understand the Layers
// - L1 (Core): Pure components with no side effects
// - L2 (Runtime): Component lifecycle and state management
// - L3 (Framework): DOM operations and event handling
// 
// ## Step 2: Migrate Components
// 
// ### Option A: Quick Migration with Wrapper
// ```rust
// migrate_component!(MyOldComponent);
// ```
// 
// ### Option B: Full HAF Rewrite
// ```rust
// haf_component! {
//     pub struct MyComponent {
//         props: MyProps,
//     }
//     
//     impl render {
//         VNode::Element {
//             tag: "div".to_string(),
//             props: VProps::default(),
//             children: vec![],
//         }
//     }
// }
// ```
// 
// ## Step 3: Enable HAF Features
// Add to Cargo.toml:
// ```toml
// [features]
// haf-components = []
// ```
// 
// ## Step 4: Gradual Migration
// 1. Start with leaf components (no children)
// 2. Move up the component tree
// 3. Migrate state management last
// 4. Finally switch feature flag