//! Virtual DOM - L3

use crate::component::Element;
use web_sys::Element as DomElement;

/// Virtual DOM diffing and patching
pub struct VDom {
    root: Option<Element>,
    dom_root: Option<DomElement>,
}

impl Default for VDom {
    fn default() -> Self {
        Self::new()
    }
}

impl VDom {
    pub fn new() -> Self {
        VDom {
            root: None,
            dom_root: None,
        }
    }

    pub fn mount(&mut self, root_id: &str) {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let dom_root = document
            .get_element_by_id(root_id)
            .unwrap_or_else(|| panic!("Element with id '{}' not found", root_id));

        self.dom_root = Some(dom_root);
    }

    pub fn render(&mut self, element: Element) {
        if let Some(dom_root) = &self.dom_root {
            // Simple replace for MVP - no diffing yet
            dom_root.set_inner_html("");
            let dom_node = element.to_dom();
            dom_root.append_child(&dom_node).unwrap();
            self.root = Some(element);
        }
    }

    // TODO: Implement proper diffing algorithm
    pub fn diff(&self, _old: &Element, _new: &Element) -> Vec<Patch> {
        vec![]
    }

    pub fn patch(&mut self, _patches: Vec<Patch>) {
        // TODO: Apply patches to DOM
    }
}

/// Patch operations for efficient updates
pub enum Patch {
    Replace {
        path: Vec<usize>,
        element: Element,
    },
    UpdateText {
        path: Vec<usize>,
        text: String,
    },
    SetAttribute {
        path: Vec<usize>,
        name: String,
        value: String,
    },
    RemoveAttribute {
        path: Vec<usize>,
        name: String,
    },
    InsertChild {
        path: Vec<usize>,
        index: usize,
        element: Element,
    },
    RemoveChild {
        path: Vec<usize>,
        index: usize,
    },
}
