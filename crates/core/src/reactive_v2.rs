//! Reactive Rendering System V2 - L3
//! 
//! Fixed version that avoids borrowing issues by deferring effect execution

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use web_sys::{Element as DomElement, Node};
use wasm_bindgen::JsCast;

use crate::component::{Component, Element};
use crate::vdom::VDom;

type PendingEffect = (ComponentId, Box<dyn FnOnce() -> EffectCleanup>);

thread_local! {
    static RENDERER: RefCell<Option<Renderer>> = const { RefCell::new(None) };
    static PENDING_EFFECTS: RefCell<Vec<PendingEffect>> = RefCell::new(Vec::new());
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
    vdom: VDom,
}

impl Renderer {
    fn new() -> Self {
        Renderer {
            components: HashMap::new(),
            render_queue: HashSet::new(),
            next_id: 1,
            is_rendering: false,
            root_element: None,
            vdom: VDom::new(),
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
            let patches = self.vdom.diff(&old_vdom, &new_vdom, &[]);
            
            // Apply patches to the DOM node
            if let Some(instance) = self.components.get(&component_id) {
                if let Some(dom_node) = &instance.dom_node {
                    if let Some(element) = dom_node.dyn_ref::<DomElement>() {
                        self.vdom.apply_patches(&patches, element);
                    }
                }
            }
            
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








    /// Register a component for testing
    pub fn register_component(
        &mut self,
        component: Box<dyn Component>,
        parent_id: Option<ComponentId>,
    ) -> ComponentId {
        let component_id = self.create_component_instance(component, parent_id);
        self.render_component(component_id);
        self.run_pending_effects();
        component_id
    }
    
    /// Process the render queue
    pub fn process_queue(&mut self) {
        // Take ownership of the queue to avoid borrowing issues
        let queue = std::mem::take(&mut self.render_queue);
        self.is_rendering = true;
        
        for component_id in queue {
            self.render_component(component_id);
        }
        
        self.is_rendering = false;
        self.run_pending_effects();
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

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use super::*;
    use crate::component::{Component, Element, Props};
    use std::rc::Rc;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    // Test component that tracks render count
    #[cfg(test)]
    struct TestComponent {
        render_count: Rc<RefCell<u32>>,
    }

    impl Component for TestComponent {
        fn render(&self) -> Element {
            *self.render_count.borrow_mut() += 1;
            Element::Text(format!("Render count: {}", self.render_count.borrow()))
        }
    }

    #[wasm_bindgen_test]
    fn test_renderer_initialization() {
        init_renderer();
        
        RENDERER.with(|r| {
            assert!(r.borrow().is_some(), "Renderer should be initialized");
        });
    }

    #[wasm_bindgen_test]
    fn test_component_lifecycle() {
        init_renderer();
        
        let render_count = Rc::new(RefCell::new(0));
        let component = Box::new(TestComponent {
            render_count: render_count.clone(),
        });
        
        let component_id = RENDERER.with(|r| {
            let mut renderer = r.borrow_mut();
            let renderer = renderer.as_mut().unwrap();
            renderer.register_component(component, None)
        });
        
        // Component should render once on registration
        assert_eq!(*render_count.borrow(), 1);
        
        // Queue render
        queue_component_render(component_id);
        
        // Process render queue
        RENDERER.with(|r| {
            let mut renderer = r.borrow_mut();
            let renderer = renderer.as_mut().unwrap();
            renderer.process_queue();
        });
        
        // Should have rendered again
        assert_eq!(*render_count.borrow(), 2);
        
        // Unmount component
        RENDERER.with(|r| {
            let mut renderer = r.borrow_mut();
            let renderer = renderer.as_mut().unwrap();
            renderer.unmount_component(component_id);
        });
        
        // Component should be removed
        RENDERER.with(|r| {
            let renderer = r.borrow();
            let renderer = renderer.as_ref().unwrap();
            assert!(!renderer.components.contains_key(&component_id));
        });
    }

    #[wasm_bindgen_test]
    fn test_parent_child_relationships() {
        init_renderer();
        
        struct ParentComponent {
            child_id: RefCell<Option<ComponentId>>,
        }
        
        impl Component for ParentComponent {
            fn render(&self) -> Element {
                Element::Node {
                    tag: "div".to_string(),
                    props: Props::default(),
                    children: vec![Element::Component(Box::new(ChildComponent))],
                }
            }
        }
        
        struct ChildComponent;
        
        impl Component for ChildComponent {
            fn render(&self) -> Element {
                Element::Text("Child".to_string())
            }
        }
        
        let parent_id = RENDERER.with(|r| {
            let mut renderer = r.borrow_mut();
            let renderer = renderer.as_mut().unwrap();
            renderer.register_component(Box::new(ParentComponent {
                child_id: RefCell::new(None),
            }), None)
        });
        
        // Check parent has children
        RENDERER.with(|r| {
            let renderer = r.borrow();
            let renderer = renderer.as_ref().unwrap();
            let parent = renderer.components.get(&parent_id).unwrap();
            assert!(!parent.child_ids.is_empty(), "Parent should have children");
        });
    }

    #[wasm_bindgen_test]
    fn test_effect_execution() {
        init_renderer();
        
        let effect_ran = Rc::new(RefCell::new(false));
        let effect_ran_clone = effect_ran.clone();
        let cleanup_ran = Rc::new(RefCell::new(false));
        let cleanup_ran_clone = cleanup_ran.clone();
        
        struct EffectComponent {
            effect_ran: Rc<RefCell<bool>>,
            cleanup_ran: Rc<RefCell<bool>>,
        }
        
        impl Component for EffectComponent {
            fn render(&self) -> Element {
                let effect_ran = self.effect_ran.clone();
                let cleanup_ran = self.cleanup_ran.clone();
                
                queue_effect_for_current_component(move || {
                    *effect_ran.borrow_mut() = true;
                    Box::new(move || {
                        *cleanup_ran.borrow_mut() = true;
                    })
                });
                
                Element::Text("Effect component".to_string())
            }
        }
        
        let component_id = RENDERER.with(|r| {
            let mut renderer = r.borrow_mut();
            let renderer = renderer.as_mut().unwrap();
            renderer.register_component(Box::new(EffectComponent {
                effect_ran: effect_ran_clone.clone(),
                cleanup_ran: cleanup_ran_clone.clone(),
            }), None)
        });
        
        // Process pending effects
        RENDERER.with(|r| {
            let mut renderer = r.borrow_mut();
            let renderer = renderer.as_mut().unwrap();
            renderer.run_pending_effects();
        });
        
        assert!(*effect_ran.borrow(), "Effect should have run");
        
        // Unmount to trigger cleanup
        RENDERER.with(|r| {
            let mut renderer = r.borrow_mut();
            let renderer = renderer.as_mut().unwrap();
            renderer.unmount_component(component_id);
        });
        
        assert!(*cleanup_ran.borrow(), "Cleanup should have run");
    }

    #[wasm_bindgen_test]
    fn test_render_queue_deduplication() {
        init_renderer();
        
        let render_count = Rc::new(RefCell::new(0));
        let component = Box::new(TestComponent {
            render_count: render_count.clone(),
        });
        
        let component_id = RENDERER.with(|r| {
            let mut renderer = r.borrow_mut();
            let renderer = renderer.as_mut().unwrap();
            renderer.register_component(component, None)
        });
        
        // Reset render count
        *render_count.borrow_mut() = 0;
        
        // Queue multiple renders
        queue_component_render(component_id);
        queue_component_render(component_id);
        queue_component_render(component_id);
        
        // Process queue
        RENDERER.with(|r| {
            let mut renderer = r.borrow_mut();
            let renderer = renderer.as_mut().unwrap();
            renderer.process_queue();
        });
        
        // Should only render once despite multiple queues
        assert_eq!(*render_count.borrow(), 1, "Component should render only once");
    }
}