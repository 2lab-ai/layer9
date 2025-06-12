//! Component System - L5

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element as DomElement, HtmlElement, MouseEvent, Node};

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

impl std::fmt::Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Element::Text(text) => f.debug_tuple("Text").field(text).finish(),
            Element::Node { tag, props, children } => {
                f.debug_struct("Node")
                    .field("tag", tag)
                    .field("props", props)
                    .field("children", children)
                    .finish()
            }
            Element::Component(_) => f.debug_tuple("Component").field(&"dyn Component").finish(),
        }
    }
}

/// Component properties
#[derive(Default, Clone)]
pub struct Props {
    pub class: Option<String>,
    pub id: Option<String>,
    pub on_click: Option<Rc<dyn Fn()>>,
    pub attributes: Vec<(String, String)>,
}

impl std::fmt::Debug for Props {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Props")
            .field("class", &self.class)
            .field("id", &self.id)
            .field("on_click", &self.on_click.as_ref().map(|_| "Fn()"))
            .field("attributes", &self.attributes)
            .finish()
    }
}

// Make Component cloneable
impl Clone for Box<dyn Component> {
    fn clone(&self) -> Self {
        // This is a simplified clone - in production you'd want proper cloning
        Box::new(EmptyComponent)
    }
}

struct EmptyComponent;
impl Component for EmptyComponent {
    fn render(&self) -> Element {
        Element::Text(String::new())
    }
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
            Element::Text(text) => web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_text_node(text)
                .into(),
            Element::Node {
                tag,
                props,
                children,
            } => {
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

                // Handle click event
                if let Some(on_click) = &props.on_click {
                    if let Some(html_element) = element.dyn_ref::<HtmlElement>() {
                        let handler = on_click.clone();
                        let closure = Closure::wrap(Box::new(move |_event: MouseEvent| {
                            handler();
                        })
                            as Box<dyn FnMut(_)>);

                        html_element.set_onclick(Some(closure.as_ref().unchecked_ref()));

                        // Important: We need to forget the closure to prevent it from being dropped
                        // In a real application, you'd want to store these closures somewhere
                        // to properly clean them up later
                        closure.forget();
                    }
                }

                // Add children
                for child in children {
                    element.append_child(&child.to_dom()).unwrap();
                }

                element.into()
            }
            Element::Component(component) => component.render().to_dom(),
        }
    }
}

/// Reactive state hook
#[derive(Clone)]
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
        // Trigger automatic re-render through the reactive system
        crate::reactive::queue_current_render();
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
        $crate::component::Element::Text($text.to_string())
    };

    // Simple element without attributes
    (<$tag:ident> $($children:tt)* </$end_tag:ident>) => {{
        assert_eq!(stringify!($tag), stringify!($end_tag), "Mismatched tags");

        let children = vec![$($crate::view!($children)),*];

        $crate::component::Element::Node {
            tag: stringify!($tag).to_string(),
            props: $crate::component::Props::default(),
            children,
        }
    }};

    // Self-closing element
    (<$tag:ident />) => {{
        $crate::component::Element::Node {
            tag: stringify!($tag).to_string(),
            props: $crate::component::Props::default(),
            children: vec![],
        }
    }};

    // Element with class
    (<$tag:ident class=$class:literal> $($children:tt)* </$end_tag:ident>) => {{
        assert_eq!(stringify!($tag), stringify!($end_tag), "Mismatched tags");

        let mut props = $crate::component::Props::default();
        props.class = Some($class.to_string());

        let children = vec![$($crate::view!($children)),*];

        $crate::component::Element::Node {
            tag: stringify!($tag).to_string(),
            props,
            children,
        }
    }};

    // Element with onclick
    (<$tag:ident onclick={$handler:expr}> $($children:tt)* </$end_tag:ident>) => {{
        assert_eq!(stringify!($tag), stringify!($end_tag), "Mismatched tags");

        let mut props = $crate::component::Props::default();
        props.on_click = Some(std::rc::Rc::new($handler));

        let children = vec![$($crate::view!($children)),*];

        $crate::component::Element::Node {
            tag: stringify!($tag).to_string(),
            props,
            children,
        }
    }};

    // Element with class and onclick
    (<$tag:ident class=$class:literal onclick={$handler:expr}> $($children:tt)* </$end_tag:ident>) => {{
        assert_eq!(stringify!($tag), stringify!($end_tag), "Mismatched tags");

        let mut props = $crate::component::Props::default();
        props.class = Some($class.to_string());
        props.on_click = Some(std::rc::Rc::new($handler));

        let children = vec![$($crate::view!($children)),*];

        $crate::component::Element::Node {
            tag: stringify!($tag).to_string(),
            props,
            children,
        }
    }};

    // Block expression
    ({ $expr:expr }) => {
        $crate::component::Element::Text($expr.to_string())
    };
}

pub use view;
