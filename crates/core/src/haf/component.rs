//! HAF-compliant Component System
//! 
//! This module provides a layered component architecture following HAF principles:
//! - L1: Pure component definitions and virtual DOM
//! - L2: Component runtime and lifecycle management
//! - L3: Framework API and DOM bindings

use crate::haf::{layers::{L1, L2, L3}, Layer, Contract};
use std::any::Any;
use std::marker::PhantomData;

// ==================== L1: Pure Component Layer ====================

/// Pure virtual DOM element (L1)
#[derive(Debug)]
pub enum VNode<L: Layer> {
    Text(String),
    Element {
        tag: String,
        props: VProps,
        children: Vec<VNode<L>>,
    },
    Component {
        key: Option<String>,
        props: Box<dyn Any>,
        render: fn(&dyn Any) -> VNode<L>,
    },
    Fragment(Vec<VNode<L>>),
    _Layer(PhantomData<L>),
}

impl<L: Layer> Clone for VNode<L> {
    fn clone(&self) -> Self {
        match self {
            VNode::Text(s) => VNode::Text(s.clone()),
            VNode::Element { tag, props, children } => VNode::Element {
                tag: tag.clone(),
                props: props.clone(),
                children: children.clone(),
            },
            VNode::Component { key, props: _, render } => {
                // Can't clone Box<dyn Any>, so we just create a placeholder
                // In real usage, components should be handled differently
                VNode::Component {
                    key: key.clone(),
                    props: Box::new(()),
                    render: *render,
                }
            }
            VNode::Fragment(children) => VNode::Fragment(children.clone()),
            VNode::_Layer(p) => VNode::_Layer(*p),
        }
    }
}

impl<L: Layer> PartialEq for VNode<L> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (VNode::Text(a), VNode::Text(b)) => a == b,
            (VNode::Element { tag: tag_a, props: props_a, children: children_a },
             VNode::Element { tag: tag_b, props: props_b, children: children_b }) => {
                tag_a == tag_b && props_a == props_b && children_a == children_b
            }
            (VNode::Component { key: key_a, .. }, VNode::Component { key: key_b, .. }) => {
                key_a == key_b
            }
            (VNode::Fragment(a), VNode::Fragment(b)) => a == b,
            (VNode::_Layer(_), VNode::_Layer(_)) => true,
            _ => false,
        }
    }
}

/// Pure component properties (L1)
#[derive(Debug, Clone, PartialEq, Default)]
pub struct VProps {
    pub key: Option<String>,
    pub class: Option<String>,
    pub id: Option<String>,
    pub style: Option<String>,
    pub attributes: Vec<(String, String)>,
    pub events: Vec<(String, EventId)>,
}

/// Event identifier for pure representation (L1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventId(pub u64);

/// Pure component trait (L1)
pub trait PureComponent<L: Layer>: 'static {
    type Props: Clone + 'static;
    
    fn render(&self, props: &Self::Props) -> VNode<L>;
}

/// Component definition (L1)
pub struct ComponentDef<L: Layer, C: PureComponent<L>> {
    _layer: PhantomData<L>,
    _component: PhantomData<C>,
}

impl<C: PureComponent<L1> + Default> Default for ComponentDef<L1, C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: PureComponent<L1> + Default> ComponentDef<L1, C> {
    pub fn new() -> Self {
        Self {
            _layer: PhantomData,
            _component: PhantomData,
        }
    }
    
    pub fn create(&self, props: C::Props) -> VNode<L1> {
        VNode::Component {
            key: None,
            props: Box::new(props.clone()),
            render: |any_props| {
                let props = any_props.downcast_ref::<C::Props>().unwrap();
                let component = C::default();
                component.render(props)
            },
        }
    }
}

// ==================== L2: Component Runtime Layer ====================

/// Component instance with runtime state (L2)
pub struct ComponentInstance<L: Layer> {
    pub vnode: VNode<L>,
    pub state: ComponentState,
    pub hooks: Vec<Box<dyn Any>>,
    _layer: PhantomData<L>,
}

/// Runtime component state (L2)
#[derive(Debug, Clone)]
pub struct ComponentState {
    pub mounted: bool,
    pub needs_update: bool,
    pub generation: u32,
}

/// Effect hook for side effects (L2)
pub struct Effect<L: Layer> {
    pub id: u64,
    pub deps: Vec<Box<dyn Any>>,
    pub cleanup: Option<Box<dyn FnOnce()>>,
    _layer: PhantomData<L>,
}

/// Component lifecycle trait (L2)
pub trait ComponentLifecycle<L: Layer> {
    fn mount(&mut self);
    fn update(&mut self);
    fn unmount(&mut self);
}

/// Runtime for managing components (L2)
pub struct ComponentRuntime<L: Layer> {
    instances: Vec<ComponentInstance<L>>,
    #[allow(dead_code)]
    effects: Vec<Effect<L>>,
    _layer: PhantomData<L>,
}

impl Default for ComponentRuntime<L2> {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentRuntime<L2> {
    pub fn new() -> Self {
        Self {
            instances: Vec::new(),
            effects: Vec::new(),
            _layer: PhantomData,
        }
    }
    
    pub fn mount_component(&mut self, vnode: VNode<L1>) -> Contract<VNode<L1>, usize> {
        let instance = ComponentInstance {
            vnode: self.transform_vnode(vnode.clone()),
            state: ComponentState {
                mounted: false,
                needs_update: false,
                generation: 0,
            },
            hooks: Vec::new(),
            _layer: PhantomData,
        };
        
        self.instances.push(instance);
        
        // Return the index instead of cloning the instance
        Contract::new(vnode, self.instances.len() - 1)
    }
    
    fn transform_vnode(&self, vnode: VNode<L1>) -> VNode<L2> {
        // Transform L1 VNode to L2 VNode with runtime capabilities
        // Since VNode is parametrized by Layer, we need a different approach
        // For now, we'll use a simple transformation
        match vnode {
            VNode::Text(text) => VNode::Text(text),
            VNode::Element { tag, props, children: _ } => VNode::Element {
                tag,
                props,
                children: vec![], // TODO: Implement proper child transformation
            },
            VNode::Component { key, props, .. } => VNode::Component {
                key,
                props,
                render: |_| VNode::Text("Component".to_string()), // Placeholder
            },
            VNode::Fragment(_) => VNode::Fragment(vec![]),
            VNode::_Layer(_) => VNode::_Layer(PhantomData),
        }
    }
}

// ==================== L3: Framework API Layer ====================

/// DOM operations contract (L3)
pub struct DomOps<L: Layer> {
    _layer: PhantomData<L>,
}

impl DomOps<L3> {
    pub fn create_element(tag: &str) -> web_sys::Element {
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element(tag)
            .unwrap()
    }
    
    pub fn set_text_content(element: &web_sys::Element, text: &str) {
        element.set_text_content(Some(text));
    }
    
    pub fn add_event_listener(
        element: &web_sys::Element,
        event: &str,
        handler: Box<dyn Fn(web_sys::Event)>,
    ) {
        use wasm_bindgen::closure::Closure;
        use wasm_bindgen::JsCast;
        
        let closure = Closure::wrap(handler as Box<dyn Fn(_)>);
        element
            .add_event_listener_with_callback(event, closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }
}

/// Component mount point (L3)
pub struct ComponentApp<L: Layer> {
    runtime: Contract<ComponentRuntime<L2>, Box<dyn Any>>,
    root: Option<web_sys::Element>,
    _layer: PhantomData<L>,
}

impl Default for ComponentApp<L3> {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentApp<L3> {
    pub fn new() -> Self {
        let runtime = ComponentRuntime::new();
        Self {
            runtime: Contract::new(runtime, Box::new(()) as Box<dyn Any>),
            root: None,
            _layer: PhantomData,
        }
    }
    
    pub fn mount(&mut self, selector: &str, vnode: VNode<L1>) {
        let document = web_sys::window().unwrap().document().unwrap();
        let root = document.query_selector(selector).unwrap().unwrap();
        
        // Mount component through runtime
        let contract = self.runtime.input.mount_component(vnode.clone());
        let _instance_index = contract.output;
        
        // Render to DOM
        self.render_to_dom(&root, &vnode);
        
        self.root = Some(root);
    }
    
    fn render_to_dom(&self, parent: &web_sys::Element, vnode: &VNode<L1>) {
        self.render_to_dom_impl(parent, vnode);
    }
    
    #[allow(clippy::only_used_in_recursion)]
    fn render_to_dom_impl(&self, parent: &web_sys::Element, vnode: &VNode<L1>) {
        match vnode {
            VNode::Text(text) => {
                let text_node = web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .create_text_node(text);
                parent.append_child(&text_node).unwrap();
            }
            VNode::Element { tag, props, children } => {
                let element = DomOps::<L3>::create_element(tag);
                
                // Apply properties
                if let Some(class) = &props.class {
                    element.set_class_name(class);
                }
                if let Some(id) = &props.id {
                    element.set_id(id);
                }
                
                // Apply attributes
                for (key, value) in &props.attributes {
                    element.set_attribute(key, value).unwrap();
                }
                
                // Render children
                for child in children {
                    self.render_to_dom_impl(&element, child);
                }
                
                parent.append_child(&element).unwrap();
            }
            VNode::Component { render, props, .. } => {
                let child_vnode = render(props.as_ref());
                self.render_to_dom_impl(parent, &child_vnode);
            }
            VNode::Fragment(children) => {
                for child in children {
                    self.render_to_dom_impl(parent, child);
                }
            }
            _ => {}
        }
    }
}

// ==================== HAF Component Macro ====================

// Macro moved to haf/mod.rs to avoid duplicate definition

// ==================== Example Usage ====================

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    
    #[derive(Clone)]
    struct ButtonProps {
        label: String,
        #[allow(dead_code)]
        onclick: Option<Rc<dyn Fn()>>,
    }
    
    struct Button;
    
    impl Default for Button {
        fn default() -> Self {
            Self
        }
    }
    
    impl PureComponent<L1> for Button {
        type Props = ButtonProps;
        
        fn render(&self, props: &Self::Props) -> VNode<L1> {
            VNode::Element {
                tag: "button".to_string(),
                props: VProps {
                    class: Some("btn".to_string()),
                    ..Default::default()
                },
                children: vec![
                    VNode::Text(props.label.clone())
                ],
            }
        }
    }
    
    #[test]
    fn test_component_creation() {
        let button_def = ComponentDef::<L1, Button>::new();
        let vnode = button_def.create(ButtonProps {
            label: "Click me".to_string(),
            onclick: None,
        });
        
        match vnode {
            VNode::Component { .. } => {}
            _ => panic!("Expected Component VNode"),
        }
    }
}