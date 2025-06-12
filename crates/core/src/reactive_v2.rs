//! Reactive Rendering System V2 - L3
//! 
//! Fixed version that avoids borrowing issues by deferring effect execution

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use web_sys::{Element as DomElement, Node};

use crate::component::{Component, Element};

thread_local! {
    static RENDERER: RefCell<Option<Renderer>> = const { RefCell::new(None) };
    static PENDING_EFFECTS: RefCell<Vec<(ComponentId, Box<dyn FnOnce() -> EffectCleanup>)>> = RefCell::new(Vec::new());
}

/// Initialize the global renderer
pub fn init_renderer() {
    RENDERER.with(|r| {
        *r.borrow_mut() = Some(Renderer::new());
    });
}

/// Component instance with unique ID
pub struct ComponentInstance {
    #[allow(dead_code)]
    id: ComponentId,
    component: Box<dyn Component>,
    dom_node: Option<Node>,
    vdom: Option<Element>,
    parent_id: Option<ComponentId>,
    child_ids: Vec<ComponentId>,
    effects: Vec<EffectCleanup>,
}

type ComponentId = u32;
type EffectCleanup = Box<dyn FnOnce()>;

/// The main rendering engine
pub struct Renderer {
    components: HashMap<ComponentId, ComponentInstance>,
    render_queue: HashSet<ComponentId>,
    next_id: ComponentId,
    is_rendering: bool,
    root_element: Option<DomElement>,
}

impl Renderer {
    fn new() -> Self {
        Renderer {
            components: HashMap::new(),
            render_queue: HashSet::new(),
            next_id: 1,
            is_rendering: false,
            root_element: None,
        }
    }

    /// Mount a component to a DOM element
    pub fn mount_root(&mut self, component: Box<dyn Component>, root_id: &str) {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let root_element = document
            .get_element_by_id(root_id)
            .expect("Root element not found");

        self.root_element = Some(root_element.clone());

        // Create root component instance
        let component_id = self.create_component_instance(component, None);
        
        // Initial render
        self.render_component(component_id);
        
        // Mount to DOM
        if let Some(instance) = self.components.get(&component_id) {
            if let Some(dom_node) = &instance.dom_node {
                root_element.append_child(dom_node).unwrap();
            }
        }
        
        // Run any pending effects after initial render
        self.run_pending_effects();
    }

    /// Create a new component instance
    fn create_component_instance(
        &mut self,
        component: Box<dyn Component>,
        parent_id: Option<ComponentId>,
    ) -> ComponentId {
        let id = self.next_id;
        self.next_id += 1;

        let instance = ComponentInstance {
            id,
            component,
            dom_node: None,
            vdom: None,
            parent_id,
            child_ids: Vec::new(),
            effects: Vec::new(),
        };

        self.components.insert(id, instance);

        // Update parent's child list
        if let Some(parent_id) = parent_id {
            if let Some(parent) = self.components.get_mut(&parent_id) {
                parent.child_ids.push(id);
            }
        }

        id
    }

    /// Queue a component for re-rendering
    pub fn queue_render(&mut self, component_id: ComponentId) {
        self.render_queue.insert(component_id);
        
        if !self.is_rendering {
            self.flush_render_queue();
        }
    }

    /// Process all queued renders
    fn flush_render_queue(&mut self) {
        self.is_rendering = true;

        // Copy queue to avoid borrow issues
        let queue: Vec<ComponentId> = self.render_queue.drain().collect();

        for component_id in queue {
            self.render_component(component_id);
        }

        self.is_rendering = false;
        
        // Run any pending effects after all renders complete
        self.run_pending_effects();
    }

    /// Render a specific component
    fn render_component(&mut self, component_id: ComponentId) {
        // Reset hook index before rendering
        crate::hooks::reset_hook_index();
        
        // Get component and render new VDOM
        let (new_vdom, old_vdom) = {
            let instance = self.components.get(&component_id).unwrap();
            // Render within component context for hooks
            let new_vdom = with_current_component(component_id, || {
                instance.component.render()
            });
            let old_vdom = instance.vdom.clone();
            (new_vdom, old_vdom)
        };

        // Perform diffing and patching
        if let Some(old_vdom) = old_vdom {
            // Diff and patch existing DOM
            let patches = self.diff_elements(&old_vdom, &new_vdom);
            self.apply_patches(component_id, patches);
            
            // Update stored VDOM
            if let Some(instance) = self.components.get_mut(&component_id) {
                instance.vdom = Some(new_vdom);
            }
        } else {
            // Initial render - create DOM
            let dom_node = new_vdom.to_dom();
            
            if let Some(instance) = self.components.get_mut(&component_id) {
                instance.dom_node = Some(dom_node);
                instance.vdom = Some(new_vdom);
            }
        }
    }

    /// Run all pending effects
    fn run_pending_effects(&mut self) {
        let effects = PENDING_EFFECTS.with(|e| {
            std::mem::take(&mut *e.borrow_mut())
        });
        
        for (component_id, effect) in effects {
            let cleanup = effect();
            if let Some(instance) = self.components.get_mut(&component_id) {
                instance.effects.push(cleanup);
            }
        }
    }

    /// Diff two virtual DOM elements
    fn diff_elements(&self, old: &Element, new: &Element) -> Vec<Patch> {
        let mut patches = Vec::new();
        self.diff_recursive(old, new, &[], &mut patches);
        patches
    }

    /// Recursive diffing algorithm
    fn diff_recursive(
        &self,
        old: &Element,
        new: &Element,
        path: &[usize],
        patches: &mut Vec<Patch>,
    ) {
        match (old, new) {
            (Element::Text(old_text), Element::Text(new_text)) => {
                if old_text != new_text {
                    patches.push(Patch::UpdateText {
                        path: path.to_vec(),
                        text: new_text.clone(),
                    });
                }
            }
            (
                Element::Node { tag: old_tag, props: old_props, children: old_children },
                Element::Node { tag: new_tag, props: new_props, children: new_children },
            ) => {
                if old_tag != new_tag {
                    // Different tags - replace entire element
                    patches.push(Patch::Replace {
                        path: path.to_vec(),
                        element: new.clone(),
                    });
                } else {
                    // Same tag - check if props changed
                    let props_changed = self.diff_props(old_props, new_props, path, patches);
                    
                    if props_changed {
                        // Replace entire element if props changed (especially event handlers)
                        patches.push(Patch::Replace {
                            path: path.to_vec(),
                            element: new.clone(),
                        });
                    } else {
                        // Props unchanged, just diff children
                        self.diff_children(old_children, new_children, path, patches);
                    }
                }
            }
            _ => {
                // Different types - replace
                patches.push(Patch::Replace {
                    path: path.to_vec(),
                    element: new.clone(),
                });
            }
        }
    }

    /// Diff properties  
    fn diff_props(
        &self,
        old_props: &crate::component::Props,
        new_props: &crate::component::Props,
        _path: &[usize],
        _patches: &mut [Patch],
    ) -> bool {
        // Check if props have changed
        old_props.class != new_props.class
            || old_props.id != new_props.id
            || match (&old_props.on_click, &new_props.on_click) {
                (Some(old_click), Some(new_click)) => !Rc::ptr_eq(old_click, new_click),
                (None, None) => false,
                _ => true,
            }
    }

    /// Diff children
    fn diff_children(
        &self,
        old_children: &[Element],
        new_children: &[Element],
        path: &[usize],
        patches: &mut Vec<Patch>,
    ) {
        let max_len = old_children.len().max(new_children.len());

        for i in 0..max_len {
            let mut child_path = path.to_vec();
            child_path.push(i);

            match (old_children.get(i), new_children.get(i)) {
                (Some(old_child), Some(new_child)) => {
                    self.diff_recursive(old_child, new_child, &child_path, patches);
                }
                (Some(_), None) => {
                    patches.push(Patch::RemoveChild {
                        path: path.to_vec(),
                        index: i,
                    });
                }
                (None, Some(new_child)) => {
                    patches.push(Patch::InsertChild {
                        path: path.to_vec(),
                        index: i,
                        element: new_child.clone(),
                    });
                }
                (None, None) => unreachable!(),
            }
        }
    }

    /// Apply patches to the DOM
    fn apply_patches(&mut self, component_id: ComponentId, patches: Vec<Patch>) {
        for patch in patches {
            self.apply_patch(component_id, patch);
        }
    }

    /// Apply a single patch
    fn apply_patch(&mut self, component_id: ComponentId, patch: Patch) {
        // Get the DOM node first to avoid borrow issues
        let dom_node = self.components.get(&component_id)
            .and_then(|instance| instance.dom_node.clone());
        
        match patch {
            Patch::UpdateText { path, text } => {
                if let Some(node) = self.find_node_at_path(&dom_node, &path) {
                    node.set_text_content(Some(&text));
                }
            }
            Patch::Replace { path, element } => {
                if let Some(node) = self.find_node_at_path(&dom_node, &path) {
                    let new_node = element.to_dom();
                    if let Some(parent) = node.parent_node() {
                        parent.replace_child(&new_node, &node).unwrap();
                    }
                }
            }
            Patch::InsertChild { path, index: _, element } => {
                if let Some(node) = self.find_node_at_path(&dom_node, &path) {
                    let new_child = element.to_dom();
                    node.append_child(&new_child).unwrap();
                }
            }
            Patch::RemoveChild { path, index } => {
                if let Some(node) = self.find_node_at_path(&dom_node, &path) {
                    if let Some(child) = node.child_nodes().get(index as u32) {
                        node.remove_child(&child).unwrap();
                    }
                }
            }
        }
    }

    /// Find a DOM node at a specific path
    fn find_node_at_path(&self, root: &Option<Node>, path: &[usize]) -> Option<Node> {
        let mut current = root.clone()?;
        
        for &index in path {
            current = current.child_nodes().get(index as u32)?;
        }
        
        Some(current)
    }

    /// Clean up a component and its children
    pub fn unmount_component(&mut self, component_id: ComponentId) {
        if let Some(mut instance) = self.components.remove(&component_id) {
            // Run cleanup effects
            for cleanup in instance.effects.drain(..) {
                cleanup();
            }
            
            // Clean up component hooks
            crate::hooks::cleanup_component_hooks(component_id);

            // Remove from parent's child list
            if let Some(parent_id) = instance.parent_id {
                if let Some(parent) = self.components.get_mut(&parent_id) {
                    parent.child_ids.retain(|&id| id != component_id);
                }
            }

            // Unmount children recursively
            for child_id in instance.child_ids.clone() {
                self.unmount_component(child_id);
            }

            // Remove from DOM
            if let Some(dom_node) = instance.dom_node {
                if let Some(parent) = dom_node.parent_node() {
                    parent.remove_child(&dom_node).unwrap();
                }
            }
        }
    }
}

/// Patch operations for DOM updates
#[derive(Debug, Clone)]
enum Patch {
    UpdateText {
        path: Vec<usize>,
        text: String,
    },
    Replace {
        path: Vec<usize>,
        element: Element,
    },
    InsertChild {
        path: Vec<usize>,
        #[allow(dead_code)]
        index: usize,
        element: Element,
    },
    RemoveChild {
        path: Vec<usize>,
        index: usize,
    },
}

thread_local! {
    /// Current component ID (used by hooks)
    static CURRENT_COMPONENT: RefCell<Option<ComponentId>> = const { RefCell::new(None) };
}

pub fn with_current_component<T>(component_id: ComponentId, f: impl FnOnce() -> T) -> T {
    CURRENT_COMPONENT.with(|c| {
        *c.borrow_mut() = Some(component_id);
    });
    
    let result = f();
    
    CURRENT_COMPONENT.with(|c| {
        *c.borrow_mut() = None;
    });
    
    result
}

pub fn get_current_component() -> Option<ComponentId> {
    CURRENT_COMPONENT.with(|c| *c.borrow())
}

/// Get current component ID (for hooks)
fn get_current_component_id() -> Option<ComponentId> {
    get_current_component()
}

/// Queue a re-render for the current component
pub fn queue_current_render() {
    if let Some(component_id) = get_current_component_id() {
        RENDERER.with(|r| {
            if let Some(renderer) = r.borrow_mut().as_mut() {
                renderer.queue_render(component_id);
            }
        });
    }
}

/// Queue a specific component for re-rendering by ID
pub fn queue_component_render(component_id: ComponentId) {
    RENDERER.with(|r| {
        if let Some(renderer) = r.borrow_mut().as_mut() {
            renderer.queue_render(component_id);
        }
    });
}

/// Queue an effect to run after rendering completes
pub fn queue_effect_for_current_component(effect: impl FnOnce() -> EffectCleanup + 'static) {
    if let Some(component_id) = get_current_component() {
        PENDING_EFFECTS.with(|e| {
            e.borrow_mut().push((component_id, Box::new(effect)));
        });
    }
}

/// Run an effect for the current component (deferred)
pub fn run_current_effect(effect: impl FnOnce() -> EffectCleanup + 'static) {
    queue_effect_for_current_component(effect);
}

/// Mount a component to the DOM
pub fn mount(component: Box<dyn Component>, root_id: &str) {
    init_renderer();
    
    RENDERER.with(|r| {
        if let Some(renderer) = r.borrow_mut().as_mut() {
            renderer.mount_root(component, root_id);
        }
    });
}