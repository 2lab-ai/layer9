//! Layer 1: Core - Pure business logic
//! 
//! This layer contains only pure functions and immutable data structures.
//! No I/O, no side effects, no external dependencies.

use super::{VNode, Props};

/// Pure diff algorithm - the heart of virtual DOM
#[derive(Clone, Debug, PartialEq)]
pub enum Patch {
    Replace { path: Vec<usize>, node: VNode },
    UpdateText { path: Vec<usize>, text: String },
    SetAttribute { path: Vec<usize>, name: String, value: String },
    RemoveAttribute { path: Vec<usize>, name: String },
    InsertChild { path: Vec<usize>, index: usize, node: VNode },
    RemoveChild { path: Vec<usize>, index: usize },
}

/// Pure function to diff two VNodes
pub fn diff(old: &VNode, new: &VNode) -> Vec<Patch> {
    diff_recursive(old, new, &mut vec![])
}

fn diff_recursive(old: &VNode, new: &VNode, path: &mut Vec<usize>) -> Vec<Patch> {
    match (old, new) {
        (VNode::Text(old_text), VNode::Text(new_text)) => {
            if old_text != new_text {
                vec![Patch::UpdateText {
                    path: path.to_vec(),
                    text: new_text.clone(),
                }]
            } else {
                vec![]
            }
        }
        (VNode::Element { tag: old_tag, props: old_props, children: old_children },
         VNode::Element { tag: new_tag, props: new_props, children: new_children }) => {
            if old_tag != new_tag {
                vec![Patch::Replace {
                    path: path.to_vec(),
                    node: new.clone(),
                }]
            } else {
                let mut patches = vec![];
                
                // Diff attributes
                patches.extend(diff_props(old_props, new_props, path));
                
                // Diff children
                let max_len = old_children.len().max(new_children.len());
                for i in 0..max_len {
                    path.push(i);
                    match (old_children.get(i), new_children.get(i)) {
                        (Some(old_child), Some(new_child)) => {
                            patches.extend(diff_recursive(old_child, new_child, path));
                        }
                        (Some(_), None) => {
                            patches.push(Patch::RemoveChild {
                                path: path[..path.len()-1].to_vec(),
                                index: i,
                            });
                        }
                        (None, Some(new_child)) => {
                            patches.push(Patch::InsertChild {
                                path: path[..path.len()-1].to_vec(),
                                index: i,
                                node: new_child.clone(),
                            });
                        }
                        (None, None) => unreachable!(),
                    }
                    path.pop();
                }
                
                patches
            }
        }
        _ => vec![Patch::Replace {
            path: path.clone(),
            node: new.clone(),
        }],
    }
}

fn diff_props(old: &Props, new: &Props, path: &[usize]) -> Vec<Patch> {
    let mut patches = vec![];
    
    // Check for changed/added attributes
    for (name, new_value) in &new.attributes {
        match old.attributes.iter().find(|(n, _)| n == name) {
            Some((_, old_value)) if old_value != new_value => {
                patches.push(Patch::SetAttribute {
                    path: path.to_vec(),
                    name: name.clone(),
                    value: new_value.clone(),
                });
            }
            None => {
                patches.push(Patch::SetAttribute {
                    path: path.to_vec(),
                    name: name.clone(),
                    value: new_value.clone(),
                });
            }
            _ => {}
        }
    }
    
    // Check for removed attributes
    for (name, _) in &old.attributes {
        if !new.attributes.iter().any(|(n, _)| n == name) {
            patches.push(Patch::RemoveAttribute {
                path: path.to_vec(),
                name: name.clone(),
            });
        }
    }
    
    patches
}

/// Pure Signal type - immutable value container
#[derive(Clone, Debug, PartialEq)]
pub struct Signal<T> {
    pub id: usize,
    pub value: T,
}

impl<T: Clone> Signal<T> {
    /// Pure function to create new signal with updated value
    pub fn with_value(&self, value: T) -> Signal<T> {
        Signal {
            id: self.id,
            value,
        }
    }
}

/// Pure Computed type - derived value
#[derive(Clone, Debug)]
pub struct Computed<T> {
    pub id: usize,
    pub dependencies: Vec<usize>,
    pub compute: fn(&[SignalValue]) -> T,
}

/// Type-erased signal value for computations
#[derive(Clone, Debug)]
pub enum SignalValue {
    String(String),
    Number(f64),
    Bool(bool),
}

/// Pure router matching
#[derive(Clone, Debug, PartialEq)]
pub struct RouteMatch {
    pub route: String,
    pub params: Vec<(String, String)>,
}

/// Pure function to match routes
pub fn match_route(pattern: &str, path: &str) -> Option<RouteMatch> {
    let pattern_parts: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    
    if pattern_parts.len() != path_parts.len() {
        return None;
    }
    
    let mut params = vec![];
    
    for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
        if let Some(stripped) = pattern_part.strip_prefix(':') {
            params.push((
                stripped.to_string(),
                path_part.to_string(),
            ));
        } else if pattern_part != path_part {
            return None;
        }
    }
    
    Some(RouteMatch {
        route: pattern.to_string(),
        params,
    })
}

/// Pure style computation
#[derive(Clone, Debug, PartialEq)]
pub struct Style {
    pub properties: Vec<(String, String)>,
}

impl Style {
    /// Pure function to merge styles
    pub fn merge(&self, other: &Style) -> Style {
        let mut properties = self.properties.clone();
        
        for (key, value) in &other.properties {
            if let Some(pos) = properties.iter().position(|(k, _)| k == key) {
                properties[pos] = (key.clone(), value.clone());
            } else {
                properties.push((key.clone(), value.clone()));
            }
        }
        
        Style { properties }
    }
    
    /// Pure function to compute CSS string
    pub fn to_css(&self) -> String {
        self.properties
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("; ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pure_diff() {
        let old = VNode::Element {
            tag: "div".to_string(),
            props: Props::default(),
            children: vec![VNode::Text("Hello".to_string())],
        };
        
        let new = VNode::Element {
            tag: "div".to_string(),
            props: Props::default(),
            children: vec![VNode::Text("World".to_string())],
        };
        
        let patches = diff(&old, &new);
        assert_eq!(patches.len(), 1);
        match &patches[0] {
            Patch::UpdateText { text, .. } => assert_eq!(text, "World"),
            _ => panic!("Expected UpdateText patch"),
        }
    }
    
    #[test]
    fn test_pure_route_matching() {
        let matched = match_route("/users/:id", "/users/123").unwrap();
        assert_eq!(matched.params, vec![("id".to_string(), "123".to_string())]);
        
        assert!(match_route("/users/:id", "/posts/123").is_none());
    }
    
    #[test]
    fn test_pure_style_merge() {
        let style1 = Style {
            properties: vec![
                ("color".to_string(), "red".to_string()),
                ("font-size".to_string(), "16px".to_string()),
            ],
        };
        
        let style2 = Style {
            properties: vec![
                ("color".to_string(), "blue".to_string()),
                ("margin".to_string(), "10px".to_string()),
            ],
        };
        
        let merged = style1.merge(&style2);
        assert_eq!(merged.properties.len(), 3);
        assert_eq!(merged.to_css(), "color: blue; font-size: 16px; margin: 10px");
    }
}