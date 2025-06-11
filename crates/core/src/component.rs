//! Component System - L5

use wasm_bindgen::prelude::*;
use web_sys::{Element as DomElement, Node};
use std::rc::Rc;
use std::cell::RefCell;

/// Virtual DOM Element
#[derive(Clone)]
pub enum Element {
    Text(String),
    Node {
        tag: String,
        props: Props,
        children: Vec<Element>,
    },
    Component(Box<dyn Component>),
}

/// Component properties
#[derive(Default, Clone)]
pub struct Props {
    pub class: Option<String>,
    pub id: Option<String>,
    pub on_click: Option<Rc<dyn Fn()>>,
    pub attributes: Vec<(String, String)>,
}

/// Base component trait
pub trait Component: 'static {
    fn render(&self) -> Element;
    
    fn mount(&self, parent: &DomElement) {
        let element = self.render();
        let dom_node = element.to_dom();
        parent.append_child(&dom_node).unwrap();
    }
}

impl Element {
    pub fn to_dom(&self) -> Node {
        match self {
            Element::Text(text) => {
                web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .create_text_node(text)
                    .into()
            }
            Element::Node { tag, props, children } => {
                let document = web_sys::window().unwrap().document().unwrap();
                let element = document.create_element(tag).unwrap();
                
                // Apply props
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
                
                // Add children
                for child in children {
                    element.append_child(&child.to_dom()).unwrap();
                }
                
                element.into()
            }
            Element::Component(component) => {
                component.render().to_dom()
            }
        }
    }
}

/// Reactive state hook
pub struct State<T> {
    value: Rc<RefCell<T>>,
}

impl<T: Clone> State<T> {
    pub fn new(initial: T) -> Self {
        State {
            value: Rc::new(RefCell::new(initial)),
        }
    }
    
    pub fn get(&self) -> T {
        self.value.borrow().clone()
    }
    
    pub fn set(&self, new_value: T) {
        *self.value.borrow_mut() = new_value;
        // Trigger re-render
        self.trigger_update();
    }
    
    fn trigger_update(&self) {
        // TODO: Implement efficient re-rendering
        web_sys::console::log_1(&"State updated, re-render needed".into());
    }
}

/// Hook for creating state
pub fn use_state<T: Clone + 'static>(initial: impl FnOnce() -> T) -> State<T> {
    State::new(initial())
}

/// View macro for JSX-like syntax
#[macro_export]
macro_rules! view {
    // Text node
    ($text:expr) => {
        Element::Text($text.to_string())
    };
    
    // Element with children
    (<$tag:ident $(class=$class:expr)? $(id=$id:expr)? > $($children:tt)* </$end_tag:ident>) => {{
        assert_eq!(stringify!($tag), stringify!($end_tag), "Mismatched tags");
        
        let mut props = Props::default();
        $(props.class = Some($class.to_string());)?
        $(props.id = Some($id.to_string());)?
        
        let children = vec![$($crate::view!($children)),*];
        
        Element::Node {
            tag: stringify!($tag).to_string(),
            props,
            children,
        }
    }};
    
    // Self-closing element
    (<$tag:ident $(class=$class:expr)? $(id=$id:expr)? />) => {{
        let mut props = Props::default();
        $(props.class = Some($class.to_string());)?
        $(props.id = Some($id.to_string());)?
        
        Element::Node {
            tag: stringify!($tag).to_string(),
            props,
            children: vec![],
        }
    }};
}

pub use view;