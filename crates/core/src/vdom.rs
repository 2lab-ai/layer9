//! Virtual DOM - L3

use crate::component::{Element, Props};
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::{Element as DomElement, Node};

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
            if let Some(old_root) = &self.root {
                // Diff and patch
                let patches = self.diff(old_root, &element, &[]);
                self.apply_patches(&patches, dom_root);
            } else {
                // Initial render
                dom_root.set_inner_html("");
                let dom_node = element.to_dom();
                dom_root.append_child(&dom_node).unwrap();
            }
            self.root = Some(element);
        }
    }

    /// Diff two virtual DOM trees and generate patches
    pub fn diff(&self, old: &Element, new: &Element, path: &[usize]) -> Vec<Patch> {
        let mut patches = vec![];

        match (old, new) {
            // Text nodes
            (Element::Text(old_text), Element::Text(new_text)) => {
                if old_text != new_text {
                    patches.push(Patch::UpdateText {
                        path: path.to_vec(),
                        text: new_text.clone(),
                    });
                }
            }
            
            // Different node types - replace entire node
            (Element::Text(_), Element::Node { .. }) 
            | (Element::Node { .. }, Element::Text(_))
            | (Element::Component(_), Element::Text(_))
            | (Element::Component(_), Element::Node { .. })
            | (Element::Text(_), Element::Component(_))
            | (Element::Node { .. }, Element::Component(_)) => {
                patches.push(Patch::Replace {
                    path: path.to_vec(),
                    element: new.clone(),
                });
            }
            
            // Element nodes with same tag
            (
                Element::Node { tag: old_tag, props: old_props, children: old_children },
                Element::Node { tag: new_tag, props: new_props, children: new_children }
            ) => {
                if old_tag != new_tag {
                    // Different tags - replace entire element
                    patches.push(Patch::Replace {
                        path: path.to_vec(),
                        element: new.clone(),
                    });
                } else {
                    // Same tag - diff props and children
                    patches.extend(self.diff_props(old_props, new_props, path));
                    patches.extend(self.diff_children(old_children, new_children, path));
                }
            }
            
            // Components - always re-render for now
            (Element::Component(_), Element::Component(_)) => {
                patches.push(Patch::Replace {
                    path: path.to_vec(),
                    element: new.clone(),
                });
            }
        }

        patches
    }

    /// Diff properties between two elements
    fn diff_props(&self, old_props: &Props, new_props: &Props, path: &[usize]) -> Vec<Patch> {
        let mut patches = vec![];

        // Check class changes
        match (&old_props.class, &new_props.class) {
            (Some(old), Some(new)) if old != new => {
                patches.push(Patch::SetAttribute {
                    path: path.to_vec(),
                    name: "class".to_string(),
                    value: new.clone(),
                });
            }
            (None, Some(new)) => {
                patches.push(Patch::SetAttribute {
                    path: path.to_vec(),
                    name: "class".to_string(),
                    value: new.clone(),
                });
            }
            (Some(_), None) => {
                patches.push(Patch::RemoveAttribute {
                    path: path.to_vec(),
                    name: "class".to_string(),
                });
            }
            _ => {}
        }

        // Check id changes
        match (&old_props.id, &new_props.id) {
            (Some(old), Some(new)) if old != new => {
                patches.push(Patch::SetAttribute {
                    path: path.to_vec(),
                    name: "id".to_string(),
                    value: new.clone(),
                });
            }
            (None, Some(new)) => {
                patches.push(Patch::SetAttribute {
                    path: path.to_vec(),
                    name: "id".to_string(),
                    value: new.clone(),
                });
            }
            (Some(_), None) => {
                patches.push(Patch::RemoveAttribute {
                    path: path.to_vec(),
                    name: "id".to_string(),
                });
            }
            _ => {}
        }

        // Diff attributes
        let old_attrs: HashMap<_, _> = old_props.attributes.iter().cloned().collect();
        let new_attrs: HashMap<_, _> = new_props.attributes.iter().cloned().collect();

        // Check for added or changed attributes
        for (key, value) in &new_attrs {
            if old_attrs.get(key) != Some(value) {
                patches.push(Patch::SetAttribute {
                    path: path.to_vec(),
                    name: key.clone(),
                    value: value.clone(),
                });
            }
        }

        // Check for removed attributes
        for key in old_attrs.keys() {
            if !new_attrs.contains_key(key) {
                patches.push(Patch::RemoveAttribute {
                    path: path.to_vec(),
                    name: key.clone(),
                });
            }
        }

        patches
    }

    /// Diff children using a simple algorithm
    fn diff_children(&self, old_children: &[Element], new_children: &[Element], path: &[usize]) -> Vec<Patch> {
        let mut patches = vec![];
        let old_len = old_children.len();
        let new_len = new_children.len();
        let min_len = old_len.min(new_len);

        // Diff common children
        for i in 0..min_len {
            let mut child_path = path.to_vec();
            child_path.push(i);
            patches.extend(self.diff(&old_children[i], &new_children[i], &child_path));
        }

        // Handle added children
        for (i, child) in new_children.iter().enumerate().skip(min_len) {
            patches.push(Patch::InsertChild {
                path: path.to_vec(),
                index: i,
                element: child.clone(),
            });
        }

        // Handle removed children
        for i in (min_len..old_len).rev() {
            patches.push(Patch::RemoveChild {
                path: path.to_vec(),
                index: i,
            });
        }

        patches
    }

    /// Apply patches to the DOM
    pub fn apply_patches(&self, patches: &[Patch], root: &DomElement) {
        for patch in patches {
            match patch {
                Patch::Replace { path, element } => {
                    if let Some(target) = self.find_node(root, path) {
                        let new_node = element.to_dom();
                        if let Some(parent) = target.parent_node() {
                            parent.replace_child(&new_node, &target).unwrap();
                        }
                    }
                }
                
                Patch::UpdateText { path, text } => {
                    if let Some(target) = self.find_node(root, path) {
                        target.set_text_content(Some(text));
                    }
                }
                
                Patch::SetAttribute { path, name, value } => {
                    if let Some(target) = self.find_node(root, path) {
                        if let Some(element) = target.dyn_ref::<DomElement>() {
                            element.set_attribute(name, value).unwrap();
                        }
                    }
                }
                
                Patch::RemoveAttribute { path, name } => {
                    if let Some(target) = self.find_node(root, path) {
                        if let Some(element) = target.dyn_ref::<DomElement>() {
                            element.remove_attribute(name).unwrap();
                        }
                    }
                }
                
                Patch::InsertChild { path, index, element } => {
                    if let Some(parent) = self.find_node(root, path) {
                        let new_child = element.to_dom();
                        let children = parent.child_nodes();
                        
                        if *index < children.length() as usize {
                            let before = children.item(*index as u32).unwrap();
                            parent.insert_before(&new_child, Some(&before)).unwrap();
                        } else {
                            parent.append_child(&new_child).unwrap();
                        }
                    }
                }
                
                Patch::RemoveChild { path, index } => {
                    if let Some(parent) = self.find_node(root, path) {
                        let children = parent.child_nodes();
                        if let Some(child) = children.item(*index as u32) {
                            parent.remove_child(&child).unwrap();
                        }
                    }
                }
            }
        }
    }

    /// Find a node in the DOM tree by path
    fn find_node(&self, root: &DomElement, path: &[usize]) -> Option<Node> {
        let mut current: Node = root.clone().into();
        
        for &index in path {
            let children = current.child_nodes();
            if let Some(child) = children.item(index as u32) {
                current = child;
            } else {
                return None;
            }
        }
        
        Some(current)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{Element, Props};
    
    #[test]
    fn test_diff_text_nodes() {
        let vdom = VDom::new();
        
        // Test identical text nodes
        let old = Element::Text("Hello".to_string());
        let new = Element::Text("Hello".to_string());
        let patches = vdom.diff(&old, &new, &[]);
        assert!(patches.is_empty());
        
        // Test different text nodes
        let old = Element::Text("Hello".to_string());
        let new = Element::Text("World".to_string());
        let patches = vdom.diff(&old, &new, &[]);
        assert_eq!(patches.len(), 1);
        match &patches[0] {
            Patch::UpdateText { text, .. } => assert_eq!(text, "World"),
            _ => panic!("Expected UpdateText patch"),
        }
    }
    
    #[test]
    fn test_diff_different_node_types() {
        let vdom = VDom::new();
        
        // Text to Element
        let old = Element::Text("Hello".to_string());
        let new = Element::Node {
            tag: "div".to_string(),
            props: Props::default(),
            children: vec![],
        };
        let patches = vdom.diff(&old, &new, &[]);
        assert_eq!(patches.len(), 1);
        assert!(matches!(&patches[0], Patch::Replace { .. }));
    }
    
    #[test]
    fn test_diff_element_props() {
        let vdom = VDom::new();
        
        // Test class change
        let old = Element::Node {
            tag: "div".to_string(),
            props: Props {
                class: Some("old".to_string()),
                ..Default::default()
            },
            children: vec![],
        };
        let new = Element::Node {
            tag: "div".to_string(),
            props: Props {
                class: Some("new".to_string()),
                ..Default::default()
            },
            children: vec![],
        };
        let patches = vdom.diff(&old, &new, &[]);
        assert_eq!(patches.len(), 1);
        match &patches[0] {
            Patch::SetAttribute { name, value, .. } => {
                assert_eq!(name, "class");
                assert_eq!(value, "new");
            }
            _ => panic!("Expected SetAttribute patch"),
        }
    }
    
    #[test]
    fn test_diff_children() {
        let vdom = VDom::new();
        
        // Test adding children
        let old = Element::Node {
            tag: "div".to_string(),
            props: Props::default(),
            children: vec![
                Element::Text("Child 1".to_string()),
            ],
        };
        let new = Element::Node {
            tag: "div".to_string(),
            props: Props::default(),
            children: vec![
                Element::Text("Child 1".to_string()),
                Element::Text("Child 2".to_string()),
            ],
        };
        let patches = vdom.diff(&old, &new, &[]);
        assert_eq!(patches.len(), 1);
        assert!(matches!(&patches[0], Patch::InsertChild { index: 1, .. }));
        
        // Test removing children
        let old = Element::Node {
            tag: "div".to_string(),
            props: Props::default(),
            children: vec![
                Element::Text("Child 1".to_string()),
                Element::Text("Child 2".to_string()),
            ],
        };
        let new = Element::Node {
            tag: "div".to_string(),
            props: Props::default(),
            children: vec![
                Element::Text("Child 1".to_string()),
            ],
        };
        let patches = vdom.diff(&old, &new, &[]);
        assert_eq!(patches.len(), 1);
        assert!(matches!(&patches[0], Patch::RemoveChild { index: 1, .. }));
    }
    
    #[test]
    fn test_diff_complex_tree() {
        let vdom = VDom::new();
        
        let old = Element::Node {
            tag: "div".to_string(),
            props: Props {
                class: Some("container".to_string()),
                ..Default::default()
            },
            children: vec![
                Element::Node {
                    tag: "h1".to_string(),
                    props: Props::default(),
                    children: vec![Element::Text("Title".to_string())],
                },
                Element::Node {
                    tag: "p".to_string(),
                    props: Props {
                        id: Some("para1".to_string()),
                        ..Default::default()
                    },
                    children: vec![Element::Text("Paragraph 1".to_string())],
                },
            ],
        };
        
        let new = Element::Node {
            tag: "div".to_string(),
            props: Props {
                class: Some("container updated".to_string()),
                ..Default::default()
            },
            children: vec![
                Element::Node {
                    tag: "h1".to_string(),
                    props: Props::default(),
                    children: vec![Element::Text("New Title".to_string())],
                },
                Element::Node {
                    tag: "p".to_string(),
                    props: Props {
                        id: Some("para1".to_string()),
                        attributes: vec![("data-test".to_string(), "value".to_string())],
                        ..Default::default()
                    },
                    children: vec![Element::Text("Paragraph 1".to_string())],
                },
                Element::Node {
                    tag: "p".to_string(),
                    props: Props::default(),
                    children: vec![Element::Text("Paragraph 2".to_string())],
                },
            ],
        };
        
        let patches = vdom.diff(&old, &new, &[]);
        
        // Should have patches for:
        // 1. Class attribute change on root
        // 2. Text update in h1
        // 3. New attribute on first p
        // 4. New p element
        assert!(patches.len() >= 4);
    }
}
