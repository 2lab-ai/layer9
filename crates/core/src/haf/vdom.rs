//! HAF-compliant Virtual DOM System
//! 
//! This module separates VDOM concerns across layers:
//! - L1: Pure diff algorithm and patch generation
//! - L2: Patch application runtime
//! - L3: DOM API bindings

use crate::haf::{layers::{L1, L2, L3}, Layer, Contract};
use crate::haf::component::{VNode, VProps, EventId};
use std::marker::PhantomData;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

// ==================== L1: Pure Diff Algorithm ====================

/// Pure patch representation (L1)
#[derive(Debug, PartialEq)]
pub enum Patch<L: Layer> {
    Replace(VNode<L>),
    UpdateText(String),
    UpdateElement(Vec<PropPatch>),
    InsertChild(usize, VNode<L>),
    RemoveChild(usize),
    MoveChild(usize, usize),
    _Layer(PhantomData<L>),
}

impl<L: Layer> Clone for Patch<L> {
    fn clone(&self) -> Self {
        match self {
            Patch::Replace(vnode) => Patch::Replace(vnode.clone()),
            Patch::UpdateText(text) => Patch::UpdateText(text.clone()),
            Patch::UpdateElement(patches) => Patch::UpdateElement(patches.clone()),
            Patch::InsertChild(index, vnode) => Patch::InsertChild(*index, vnode.clone()),
            Patch::RemoveChild(index) => Patch::RemoveChild(*index),
            Patch::MoveChild(from, to) => Patch::MoveChild(*from, *to),
            Patch::_Layer(p) => Patch::_Layer(*p),
        }
    }
}

/// Property patch (L1)
#[derive(Debug, Clone, PartialEq)]
pub enum PropPatch {
    SetClass(Option<String>),
    SetId(Option<String>),
    SetStyle(Option<String>),
    SetAttribute(String, Option<String>),
    AddEvent(String, EventId),
    RemoveEvent(String),
}

/// Pure VDOM diff algorithm (L1)
pub struct VDomDiff<L: Layer> {
    _layer: PhantomData<L>,
}

impl Default for VDomDiff<L1> {
    fn default() -> Self {
        Self::new()
    }
}

impl VDomDiff<L1> {
    pub fn new() -> Self {
        Self { _layer: PhantomData }
    }
    
    /// Diff two VNodes and generate patches
    pub fn diff(&self, old: &VNode<L1>, new: &VNode<L1>) -> Vec<Patch<L1>> {
        let mut patches = Vec::new();
        self.diff_nodes(old, new, &mut patches);
        patches
    }
    
    fn diff_nodes(&self, old: &VNode<L1>, new: &VNode<L1>, patches: &mut Vec<Patch<L1>>) {
        match (old, new) {
            (VNode::Text(old_text), VNode::Text(new_text)) => {
                if old_text != new_text {
                    patches.push(Patch::UpdateText(new_text.clone()));
                }
            }
            
            (VNode::Element { tag: old_tag, props: old_props, children: old_children },
             VNode::Element { tag: new_tag, props: new_props, children: new_children }) => {
                if old_tag != new_tag {
                    patches.push(Patch::Replace(new.clone()));
                } else {
                    // Diff properties
                    let prop_patches = self.diff_props(old_props, new_props);
                    if !prop_patches.is_empty() {
                        patches.push(Patch::UpdateElement(prop_patches));
                    }
                    
                    // Diff children
                    self.diff_children(old_children, new_children, patches);
                }
            }
            
            _ => {
                // Different node types - replace
                patches.push(Patch::Replace(new.clone()));
            }
        }
    }
    
    fn diff_props(&self, old: &VProps, new: &VProps) -> Vec<PropPatch> {
        let mut patches = Vec::new();
        
        if old.class != new.class {
            patches.push(PropPatch::SetClass(new.class.clone()));
        }
        
        if old.id != new.id {
            patches.push(PropPatch::SetId(new.id.clone()));
        }
        
        if old.style != new.style {
            patches.push(PropPatch::SetStyle(new.style.clone()));
        }
        
        // Diff attributes
        for (key, new_value) in &new.attributes {
            if old.attributes.iter().find(|(k, _)| k == key).map(|(_, v)| v) != Some(new_value) {
                patches.push(PropPatch::SetAttribute(key.clone(), Some(new_value.clone())));
            }
        }
        
        // Remove old attributes not in new
        for (key, _) in &old.attributes {
            if !new.attributes.iter().any(|(k, _)| k == key) {
                patches.push(PropPatch::SetAttribute(key.clone(), None));
            }
        }
        
        // Diff events
        for (event, id) in &new.events {
            if !old.events.iter().any(|(e, _)| e == event) {
                patches.push(PropPatch::AddEvent(event.clone(), *id));
            }
        }
        
        for (event, _) in &old.events {
            if !new.events.iter().any(|(e, _)| e == event) {
                patches.push(PropPatch::RemoveEvent(event.clone()));
            }
        }
        
        patches
    }
    
    fn diff_children(&self, old: &[VNode<L1>], new: &[VNode<L1>], patches: &mut Vec<Patch<L1>>) {
        // Simple algorithm - can be optimized with keyed diffing
        let mut i = 0;
        
        while i < old.len() && i < new.len() {
            self.diff_nodes(&old[i], &new[i], patches);
            i += 1;
        }
        
        // Remove extra old children
        while i < old.len() {
            patches.push(Patch::RemoveChild(i));
            i += 1;
        }
        
        // Add new children
        while i < new.len() {
            patches.push(Patch::InsertChild(i, new[i].clone()));
            i += 1;
        }
    }
}

// ==================== L2: Patch Runtime ====================

/// Patch application runtime (L2)
pub struct PatchRuntime<L: Layer> {
    patches: Vec<Patch<L1>>,
    operations: Vec<DomOperation>,
    _layer: PhantomData<L>,
}

impl Default for PatchRuntime<L2> {
    fn default() -> Self {
        Self::new()
    }
}

impl PatchRuntime<L2> {
    pub fn new() -> Self {
        Self {
            patches: Vec::new(),
            operations: Vec::new(),
            _layer: PhantomData,
        }
    }
    
    /// Apply patches and generate DOM operations
    pub fn apply_patches(&mut self, patches: Vec<Patch<L1>>) -> Vec<Contract<Patch<L1>, DomOperation>> {
        self.patches = patches;
        let mut contracts = Vec::new();
        
        for patch in self.patches.clone() {
            let operation = self.patch_to_operation(&patch);
            contracts.push(Contract::new(patch, operation));
        }
        
        contracts
    }
    
    fn patch_to_operation(&self, patch: &Patch<L1>) -> DomOperation {
        match patch {
            Patch::Replace(vnode) => DomOperation::ReplaceNode {
                handle: DomHandle { node_id: 0 }, // TODO: Proper handle management
                new_vnode: vnode.clone(),
            },
            
            Patch::UpdateText(text) => DomOperation::SetTextContent {
                handle: DomHandle { node_id: 0 },
                text: text.clone(),
            },
            
            Patch::UpdateElement(prop_patches) => DomOperation::UpdateProperties {
                handle: DomHandle { node_id: 0 },
                patches: prop_patches.clone(),
            },
            
            Patch::InsertChild(index, vnode) => DomOperation::InsertChild {
                parent: DomHandle { node_id: 0 },
                index: *index,
                child: vnode.clone(),
            },
            
            Patch::RemoveChild(index) => DomOperation::RemoveChild {
                parent: DomHandle { node_id: 0 },
                index: *index,
            },
            
            Patch::MoveChild(from, to) => DomOperation::MoveChild {
                parent: DomHandle { node_id: 0 },
                from: *from,
                to: *to,
            },
            
            _ => DomOperation::NoOp,
        }
    }
    
    /// Handle events (L2)
    pub fn handle_event(&self, event_id: EventId) -> Option<Box<dyn Fn()>> {
        // Look up event handler
        // In real implementation, would maintain event handler registry
        self.operations
            .iter()
            .find_map(|op| match op {
                DomOperation::UpdateProperties { patches, .. } => {
                    patches.iter().find_map(|patch| match patch {
                        PropPatch::AddEvent(_, id) if *id == event_id => {
                            Some(Box::new(|| println!("Event handled!")) as Box<dyn Fn()>)
                        }
                        _ => None,
                    })
                }
                _ => None,
            })
            .or_else(|| None)
    }
}

/// DOM operation descriptor (L2/L3 boundary)
pub enum DomOperation {
    ReplaceNode { handle: DomHandle, new_vnode: VNode<L1> },
    SetTextContent { handle: DomHandle, text: String },
    UpdateProperties { handle: DomHandle, patches: Vec<PropPatch> },
    InsertChild { parent: DomHandle, index: usize, child: VNode<L1> },
    RemoveChild { parent: DomHandle, index: usize },
    MoveChild { parent: DomHandle, from: usize, to: usize },
    NoOp,
}

/// DOM node handle (L2/L3 boundary)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DomHandle {
    node_id: u64,
}

// ==================== L3: DOM Bindings ====================

/// DOM renderer (L3)
#[cfg(target_arch = "wasm32")]
pub struct DomRenderer<L: Layer> {
    document: web_sys::Document,
    nodes: Vec<(u64, web_sys::Node)>,
    _layer: PhantomData<L>,
}

#[cfg(not(target_arch = "wasm32"))]
pub struct DomRenderer<L: Layer> {
    _layer: PhantomData<L>,
}

impl Default for DomRenderer<L3> {
    fn default() -> Self {
        Self::new()
    }
}

impl DomRenderer<L3> {
    pub fn new() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self {
                document: web_sys::window().unwrap().document().unwrap(),
                nodes: Vec::new(),
                _layer: PhantomData,
            }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                _layer: PhantomData,
            }
        }
    }
    
    /// Execute DOM operations
    pub fn execute_operations(&mut self, operations: Vec<Contract<Patch<L1>, DomOperation>>) {
        for contract in operations {
            self.execute_operation(&contract.output);
        }
    }
    
    #[cfg(target_arch = "wasm32")]
    fn execute_operation(&mut self, operation: &DomOperation) {
        match operation {
            DomOperation::ReplaceNode { handle, new_vnode } => {
                if let Some(node) = self.get_node(handle) {
                    let new_node = self.create_dom_node(new_vnode);
                    if let Some(parent) = node.parent_node() {
                        parent.replace_child(&new_node, &node).unwrap();
                        self.update_node(handle, new_node);
                    }
                }
            }
            
            DomOperation::SetTextContent { handle, text } => {
                if let Some(node) = self.get_node(handle) {
                    node.set_text_content(Some(text));
                }
            }
            
            DomOperation::UpdateProperties { handle, patches } => {
                if let Some(node) = self.get_node(handle) {
                    if let Some(element) = node.dyn_ref::<web_sys::Element>() {
                        for patch in patches {
                            self.apply_prop_patch(element, patch);
                        }
                    }
                }
            }
            
            DomOperation::InsertChild { parent, index, child } => {
                if let Some(parent_node) = self.get_node(parent) {
                    let child_node = self.create_dom_node(child);
                    let children = parent_node.child_nodes();
                    if (*index as u32) < children.length() {
                        let before = children.get(*index as u32).unwrap();
                        parent_node.insert_before(&child_node, Some(&before)).unwrap();
                    } else {
                        parent_node.append_child(&child_node).unwrap();
                    }
                }
            }
            
            DomOperation::RemoveChild { parent, index } => {
                if let Some(parent_node) = self.get_node(parent) {
                    let children = parent_node.child_nodes();
                    if let Some(child) = children.get(*index as u32) {
                        parent_node.remove_child(&child).unwrap();
                    }
                }
            }
            
            DomOperation::MoveChild { parent, from, to } => {
                if let Some(parent_node) = self.get_node(parent) {
                    let children = parent_node.child_nodes();
                    if let Some(child) = children.get(*from as u32) {
                        parent_node.remove_child(&child).unwrap();
                        
                        let children = parent_node.child_nodes();
                        if *to < children.length() as usize {
                            let before = children.get(*to as u32).unwrap();
                            parent_node.insert_before(&child, Some(&before)).unwrap();
                        } else {
                            parent_node.append_child(&child).unwrap();
                        }
                    }
                }
            }
            
            DomOperation::NoOp => {}
        }
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn execute_operation(&mut self, _operation: &DomOperation) {
        // No-op for non-wasm targets
    }
    
    #[cfg(target_arch = "wasm32")]
    fn get_node(&self, handle: &DomHandle) -> Option<&web_sys::Node> {
        self.nodes
            .iter()
            .find(|(id, _)| id == &handle.node_id)
            .map(|(_, node)| node)
    }
    
    #[cfg(target_arch = "wasm32")]
    fn update_node(&mut self, handle: &DomHandle, node: web_sys::Node) {
        if let Some((_, old_node)) = self.nodes.iter_mut().find(|(id, _)| id == &handle.node_id) {
            *old_node = node;
        } else {
            self.nodes.push((handle.node_id, node));
        }
    }
    
    #[cfg(target_arch = "wasm32")]
    fn create_dom_node(&self, vnode: &VNode<L1>) -> web_sys::Node {
        match vnode {
            VNode::Text(text) => {
                self.document.create_text_node(text).into()
            }
            
            VNode::Element { tag, props, children } => {
                let element = self.document.create_element(tag).unwrap();
                
                // Apply properties
                if let Some(class) = &props.class {
                    element.set_class_name(class);
                }
                if let Some(id) = &props.id {
                    element.set_id(id);
                }
                if let Some(style) = &props.style {
                    element.set_attribute("style", style).unwrap();
                }
                
                for (key, value) in &props.attributes {
                    element.set_attribute(key, value).unwrap();
                }
                
                // Add children
                for child in children {
                    let child_node = self.create_dom_node(child);
                    element.append_child(&child_node).unwrap();
                }
                
                element.into()
            }
            
            _ => {
                // For other node types, create a placeholder
                self.document.create_text_node("").into()
            }
        }
    }
    
    #[cfg(target_arch = "wasm32")]
    fn apply_prop_patch(&self, element: &web_sys::Element, patch: &PropPatch) {
        match patch {
            PropPatch::SetClass(class) => {
                if let Some(class) = class {
                    element.set_class_name(class);
                } else {
                    element.remove_attribute("class").unwrap();
                }
            }
            
            PropPatch::SetId(id) => {
                if let Some(id) = id {
                    element.set_id(id);
                } else {
                    element.remove_attribute("id").unwrap();
                }
            }
            
            PropPatch::SetStyle(style) => {
                if let Some(style) = style {
                    element.set_attribute("style", style).unwrap();
                } else {
                    element.remove_attribute("style").unwrap();
                }
            }
            
            PropPatch::SetAttribute(key, value) => {
                if let Some(value) = value {
                    element.set_attribute(key, value).unwrap();
                } else {
                    element.remove_attribute(key).unwrap();
                }
            }
            
            PropPatch::AddEvent(event, _id) => {
                // TODO: Implement event handling
                println!("Adding event: {}", event);
            }
            
            PropPatch::RemoveEvent(event) => {
                // TODO: Implement event removal
                println!("Removing event: {}", event);
            }
        }
    }
}

/// L1 to L2 contract for VDOM operations
pub struct VDomContract;

impl L1ToL2Contract for VDomContract {
    type L1Type = Vec<Patch<L1>>;
    type L2Type = Vec<DomOperation>;
    
    fn translate(patches: Self::L1Type) -> Self::L2Type {
        let mut runtime = PatchRuntime::<L2>::new();
        runtime.apply_patches(patches)
            .into_iter()
            .map(|contract| contract.output)
            .collect()
    }
}

/// Trait for L1 to L2 contracts
pub trait L1ToL2Contract {
    type L1Type;
    type L2Type;
    
    fn translate(input: Self::L1Type) -> Self::L2Type;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_diff() {
        let diff = VDomDiff::<L1>::new();
        
        let old = VNode::Text("Hello".to_string());
        let new = VNode::Text("World".to_string());
        
        let patches = diff.diff(&old, &new);
        assert_eq!(patches.len(), 1);
        assert!(matches!(patches[0], Patch::UpdateText(ref s) if s == "World"));
    }
    
    #[test]
    fn test_element_diff() {
        let diff = VDomDiff::<L1>::new();
        
        let old = VNode::Element {
            tag: "div".to_string(),
            props: VProps {
                class: Some("old".to_string()),
                ..Default::default()
            },
            children: vec![],
        };
        
        let new = VNode::Element {
            tag: "div".to_string(),
            props: VProps {
                class: Some("new".to_string()),
                ..Default::default()
            },
            children: vec![],
        };
        
        let patches = diff.diff(&old, &new);
        assert_eq!(patches.len(), 1);
        assert!(matches!(patches[0], Patch::UpdateElement(ref props) if props.len() == 1));
    }
}