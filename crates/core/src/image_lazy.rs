//! Lazy loading implementation for images using Intersection Observer

use crate::component::{use_state, Component};
use crate::prelude::*;
use crate::hooks::{use_effect, use_ref};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    Element as DomElement, 
    HtmlImageElement, 
    IntersectionObserver, 
    IntersectionObserverEntry,
    IntersectionObserverInit,
};

/// Lazy loading configuration
#[derive(Clone)]
pub struct LazyConfig {
    /// Root margin for intersection observer (e.g., "50px")
    pub root_margin: String,
    /// Threshold for intersection (0.0 to 1.0)
    pub threshold: f64,
    /// Whether to unobserve after loading
    pub unobserve_on_load: bool,
}

impl Default for LazyConfig {
    fn default() -> Self {
        Self {
            root_margin: "50px".to_string(),
            threshold: 0.01,
            unobserve_on_load: true,
        }
    }
}

/// Global lazy loading manager
pub struct LazyLoadManager {
    observer: Option<IntersectionObserver>,
    #[allow(dead_code)]
    config: LazyConfig,
}

thread_local! {
    static LAZY_MANAGER: RefCell<Option<LazyLoadManager>> = const { RefCell::new(None) };
}

impl LazyLoadManager {
    /// Initialize the global lazy load manager
    pub fn init(config: LazyConfig) {
        LAZY_MANAGER.with(|manager| {
            let mut manager = manager.borrow_mut();
            if manager.is_none() {
                *manager = Some(LazyLoadManager::new(config));
            }
        });
    }

    fn new(config: LazyConfig) -> Self {
        let observer = create_intersection_observer(&config);
        Self {
            observer,
            config,
        }
    }

    /// Observe an element for lazy loading
    pub fn observe(element: &DomElement) {
        LAZY_MANAGER.with(|manager| {
            if let Some(ref manager) = *manager.borrow() {
                if let Some(ref observer) = manager.observer {
                    observer.observe(element);
                }
            }
        });
    }

    /// Unobserve an element
    pub fn unobserve(element: &DomElement) {
        LAZY_MANAGER.with(|manager| {
            if let Some(ref manager) = *manager.borrow() {
                if let Some(ref observer) = manager.observer {
                    observer.unobserve(element);
                }
            }
        });
    }
}

/// Create intersection observer for lazy loading
fn create_intersection_observer(config: &LazyConfig) -> Option<IntersectionObserver> {
    let unobserve_on_load = config.unobserve_on_load;
    let callback = Closure::wrap(Box::new(move |entries: Vec<IntersectionObserverEntry>, observer: IntersectionObserver| {
        for entry in entries {
            if entry.is_intersecting() {
                let target = entry.target();
                
                // Load image
                if let Ok(img) = target.dyn_into::<HtmlImageElement>() {
                    // Get data-src attribute
                    if let Some(src) = img.get_attribute("data-src") {
                        img.set_src(&src);
                        img.remove_attribute("data-src").ok();
                        
                        // Add loaded class
                        let class_list = img.class_list();
                        class_list.remove_1("loading").ok();
                        class_list.add_1("loaded").ok();
                    }
                    
                    // Get data-srcset attribute
                    if let Some(srcset) = img.get_attribute("data-srcset") {
                        img.set_attribute("srcset", &srcset).ok();
                        img.remove_attribute("data-srcset").ok();
                    }
                    
                    // Unobserve if configured
                    if unobserve_on_load {
                        observer.unobserve(&img);
                    }
                }
            }
        }
    }) as Box<dyn FnMut(Vec<IntersectionObserverEntry>, IntersectionObserver)>);

    let options = IntersectionObserverInit::new();
    options.set_root_margin(&config.root_margin);
    
    let threshold_array = js_sys::Array::new();
    threshold_array.push(&JsValue::from(config.threshold));
    options.set_threshold(&threshold_array);

    let observer = IntersectionObserver::new_with_options(
        callback.as_ref().unchecked_ref(),
        &options,
    ).ok();

    callback.forget(); // Prevent closure from being dropped
    observer
}

/// Hook for lazy loading a single image
pub fn use_lazy_image(src: &str, placeholder: Option<&str>) -> (String, bool) {
    let loaded = use_state(|| false);
    let element_ref = use_ref::<Option<DomElement>>(None);
    
    let _src_clone = src.to_string();
    let placeholder_clone = placeholder.map(|p| p.to_string());
    let loaded_clone = loaded.clone();
    
    use_effect((), move || {
        // Initialize lazy manager if not already done
        LazyLoadManager::init(LazyConfig::default());
        
        // Get element and observe
        if let Some(element) = element_ref.borrow().as_ref() {
                // Set up load handler
                let load_handler = Closure::wrap(Box::new(move || {
                    loaded_clone.set(true);
                }) as Box<dyn Fn()>);
                
                if let Some(img) = element.dyn_ref::<HtmlImageElement>() {
                    img.set_onload(Some(load_handler.as_ref().unchecked_ref()));
                    load_handler.forget();
                }
                
                LazyLoadManager::observe(element);
        }
        
        move || {
            // Cleanup: unobserve
            if let Some(element) = element_ref.borrow().as_ref() {
                LazyLoadManager::unobserve(element);
            }
        }
    });
    
    let current_src = if loaded.get() {
        src.to_string()
    } else {
        placeholder_clone.unwrap_or_else(|| "data:image/svg+xml,%3Csvg%20xmlns='http://www.w3.org/2000/svg'%20width='1'%20height='1'%3E%3C/svg%3E".to_string())
    };
    
    (current_src, loaded.get())
}

/// Lazy loaded image component
pub struct LazyImage {
    src: String,
    alt: String,
    width: Option<u32>,
    height: Option<u32>,
    placeholder: Option<String>,
    class: Option<String>,
    srcset: Option<String>,
    sizes: Option<String>,
}

impl LazyImage {
    pub fn new(src: impl Into<String>) -> Self {
        Self {
            src: src.into(),
            alt: String::new(),
            width: None,
            height: None,
            placeholder: None,
            class: None,
            srcset: None,
            sizes: None,
        }
    }

    pub fn alt(mut self, alt: impl Into<String>) -> Self {
        self.alt = alt.into();
        self
    }

    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = Some(class.into());
        self
    }

    pub fn srcset(mut self, srcset: impl Into<String>) -> Self {
        self.srcset = Some(srcset.into());
        self
    }

    pub fn sizes(mut self, sizes: impl Into<String>) -> Self {
        self.sizes = Some(sizes.into());
        self
    }
}

impl Component for LazyImage {
    fn render(&self) -> Element {
        let loaded = use_state(|| false);
        let img_ref = use_ref::<Option<DomElement>>(None);
        
        // Set up lazy loading
        let loaded_clone = loaded.clone();
        let img_ref_clone = img_ref.clone();
        let _src_clone = self.src.clone();
        
        // Note: Since we can't directly return different closure types,
        // we'll handle cleanup differently
        let cleanup_ref = img_ref.clone();
        
        use_effect((), move || {
            // Initialize lazy manager
            LazyLoadManager::init(LazyConfig::default());
            
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    // Use a unique ID to find our element
                    let id = format!("lazy-img-{}", js_sys::Math::random());
                    
                    // Set up a timeout to find and observe the element
                    let closure = Closure::wrap(Box::new(move || {
                        if let Some(element) = document.get_element_by_id(&id) {
                            *img_ref_clone.borrow_mut() = Some(element.clone());
                            
                            // Set up load handler
                            let loaded_inner = loaded_clone.clone();
                            let load_handler = Closure::wrap(Box::new(move || {
                                loaded_inner.set(true);
                            }) as Box<dyn Fn()>);
                            
                            if let Some(img) = element.dyn_ref::<HtmlImageElement>() {
                                img.set_onload(Some(load_handler.as_ref().unchecked_ref()));
                                load_handler.forget();
                                
                                // Observe for lazy loading
                                LazyLoadManager::observe(&element.clone());
                            }
                        }
                    }) as Box<dyn Fn()>);
                    
                    window.set_timeout_with_callback_and_timeout_and_arguments_0(
                        closure.as_ref().unchecked_ref(),
                        0,
                    ).ok();
                    
                    closure.forget();
                }
            }
            
            // Cleanup function
            move || {
                if let Some(element) = cleanup_ref.borrow().as_ref() {
                    LazyLoadManager::unobserve(element);
                }
            }
        });
        
        // Build attributes
        let mut attrs = vec![
            ("alt".to_string(), self.alt.clone()),
            ("id".to_string(), format!("lazy-img-{}", js_sys::Math::random())),
        ];
        
        // Add dimensions
        if let Some(width) = self.width {
            attrs.push(("width".to_string(), width.to_string()));
        }
        if let Some(height) = self.height {
            attrs.push(("height".to_string(), height.to_string()));
        }
        
        // Handle src/data-src based on loaded state
        if loaded.get() {
            attrs.push(("src".to_string(), self.src.clone()));
            if let Some(ref srcset) = self.srcset {
                attrs.push(("srcset".to_string(), srcset.clone()));
            }
        } else {
            attrs.push(("data-src".to_string(), self.src.clone()));
            if let Some(ref placeholder) = self.placeholder {
                attrs.push(("src".to_string(), placeholder.clone()));
            } else {
                attrs.push(("src".to_string(), "data:image/svg+xml,%3Csvg%20xmlns='http://www.w3.org/2000/svg'%20width='1'%20height='1'%3E%3C/svg%3E".to_string()));
            }
            if let Some(ref srcset) = self.srcset {
                attrs.push(("data-srcset".to_string(), srcset.clone()));
            }
        }
        
        // Add sizes
        if let Some(ref sizes) = self.sizes {
            attrs.push(("sizes".to_string(), sizes.clone()));
        }
        
        // Add loading attribute
        attrs.push(("loading".to_string(), "lazy".to_string()));
        
        // Build class
        let mut classes = vec![];
        if let Some(ref class) = self.class {
            classes.push(class.clone());
        }
        classes.push(if loaded.get() { "loaded" } else { "loading" }.to_string());
        
        Element::Node {
            tag: "img".to_string(),
            props: Props {
                class: Some(classes.join(" ")),
                attributes: attrs,
                ..Default::default()
            },
            children: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_config_default() {
        let config = LazyConfig::default();
        assert_eq!(config.root_margin, "50px");
        assert_eq!(config.threshold, 0.01);
        assert!(config.unobserve_on_load);
    }
}