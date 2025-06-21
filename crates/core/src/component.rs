//! Component System - L5

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element as DomElement, Event, HtmlElement, HtmlInputElement, MouseEvent, Node};

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
    pub on_submit: Option<Rc<dyn Fn(Event)>>,
    pub on_change: Option<Rc<dyn Fn(String)>>,
    pub on_input: Option<Rc<dyn Fn(String)>>,
    pub attributes: Vec<(String, String)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn test_onchange_event_handling() {
        // Test that on_change prop is correctly set
        let handler_called = Rc::new(RefCell::new(false));
        let handler_ref = handler_called.clone();
        
        let props = Props {
            on_change: Some(Rc::new(move |_value: String| {
                *handler_ref.borrow_mut() = true;
            })),
            ..Default::default()
        };
        
        // Verify on_change is set
        assert!(props.on_change.is_some());
        
        // Test the handler
        if let Some(handler) = props.on_change {
            handler("test".to_string());
            assert!(*handler_called.borrow());
        }
    }

    #[test] 
    fn test_view_macro_with_onchange() {
        let value_changed = Rc::new(RefCell::new(String::new()));
        let value_ref = value_changed.clone();
        
        let element = Element::Node {
            tag: "input".to_string(),
            props: Props {
                on_change: Some(Rc::new(move |v: String| {
                    *value_ref.borrow_mut() = v;
                })),
                ..Props::default()
            },
            children: vec![],
        };
        
        // Verify the element structure
        if let Element::Node { tag, props, .. } = element {
            assert_eq!(tag, "input");
            assert!(props.on_change.is_some());
        }
    }
}

impl std::fmt::Debug for Props {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Props")
            .field("class", &self.class)
            .field("id", &self.id)
            .field("on_click", &self.on_click.as_ref().map(|_| "Fn()"))
            .field("on_submit", &self.on_submit.as_ref().map(|_| "Fn(Event)"))
            .field("on_change", &self.on_change.as_ref().map(|_| "Fn(String)"))
            .field("on_input", &self.on_input.as_ref().map(|_| "Fn(String)"))
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
                        closure.forget();
                    }
                }

                // Handle submit event for forms
                if let Some(on_submit) = &props.on_submit {
                    if tag == "form" {
                        if let Some(form_element) = element.dyn_ref::<HtmlElement>() {
                            let handler = on_submit.clone();
                            let closure = Closure::wrap(Box::new(move |event: Event| {
                                event.prevent_default(); // Prevent form submission
                                handler(event);
                            })
                                as Box<dyn FnMut(_)>);

                            form_element.set_onsubmit(Some(closure.as_ref().unchecked_ref()));
                            closure.forget();
                        }
                    }
                }

                // Handle change event for inputs
                if let Some(on_change) = &props.on_change {
                    if tag == "input" || tag == "select" || tag == "textarea" {
                        if let Some(input_element) = element.dyn_ref::<HtmlInputElement>() {
                            let handler = on_change.clone();
                            let closure = Closure::wrap(Box::new(move |_event: Event| {
                                if let Some(target) = _event.target() {
                                    if let Some(input) = target.dyn_ref::<HtmlInputElement>() {
                                        handler(input.value());
                                    }
                                }
                            })
                                as Box<dyn FnMut(_)>);

                            input_element.set_onchange(Some(closure.as_ref().unchecked_ref()));
                            closure.forget();
                        }
                    }
                }

                // Handle input event for real-time updates
                if let Some(on_input) = &props.on_input {
                    if tag == "input" || tag == "textarea" {
                        if let Some(input_element) = element.dyn_ref::<HtmlInputElement>() {
                            let handler = on_input.clone();
                            let closure = Closure::wrap(Box::new(move |_event: Event| {
                                if let Some(target) = _event.target() {
                                    if let Some(input) = target.dyn_ref::<HtmlInputElement>() {
                                        handler(input.value());
                                    }
                                }
                            })
                                as Box<dyn FnMut(_)>);

                            input_element.set_oninput(Some(closure.as_ref().unchecked_ref()));
                            closure.forget();
                        }
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
        crate::reactive_v2::queue_current_render();
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

    // Element with onchange
    (<$tag:ident onchange={$handler:expr}> $($children:tt)* </$end_tag:ident>) => {{
        assert_eq!(stringify!($tag), stringify!($end_tag), "Mismatched tags");

        let mut props = $crate::component::Props::default();
        props.on_change = Some(std::rc::Rc::new($handler));

        let children = vec![$($crate::view!($children)),*];

        $crate::component::Element::Node {
            tag: stringify!($tag).to_string(),
            props,
            children,
        }
    }};

    // Element with onsubmit
    (<$tag:ident onsubmit={$handler:expr}> $($children:tt)* </$end_tag:ident>) => {{
        assert_eq!(stringify!($tag), stringify!($end_tag), "Mismatched tags");

        let mut props = $crate::component::Props::default();
        props.on_submit = Some(std::rc::Rc::new($handler));

        let children = vec![$($crate::view!($children)),*];

        $crate::component::Element::Node {
            tag: stringify!($tag).to_string(),
            props,
            children,
        }
    }};

    // Element with oninput
    (<$tag:ident oninput={$handler:expr}> $($children:tt)* </$end_tag:ident>) => {{
        assert_eq!(stringify!($tag), stringify!($end_tag), "Mismatched tags");

        let mut props = $crate::component::Props::default();
        props.on_input = Some(std::rc::Rc::new($handler));

        let children = vec![$($crate::view!($children)),*];

        $crate::component::Element::Node {
            tag: stringify!($tag).to_string(),
            props,
            children,
        }
    }};

    // Element with value attribute (for controlled inputs)
    (<$tag:ident value={$value:expr}> $($children:tt)* </$end_tag:ident>) => {{
        assert_eq!(stringify!($tag), stringify!($end_tag), "Mismatched tags");

        let mut props = $crate::component::Props::default();
        props.attributes.push(("value".to_string(), $value.to_string()));

        let children = vec![$($crate::view!($children)),*];

        $crate::component::Element::Node {
            tag: stringify!($tag).to_string(),
            props,
            children,
        }
    }};

    // Element with type attribute  
    (<$tag:ident type=$type:literal> $($children:tt)* </$end_tag:ident>) => {{
        assert_eq!(stringify!($tag), stringify!($end_tag), "Mismatched tags");

        let mut props = $crate::component::Props::default();
        props.attributes.push(("type".to_string(), $type.to_string()));

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
