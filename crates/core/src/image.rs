//! Image Optimization - L5

use crate::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{IntersectionObserver, IntersectionObserverEntry};
use std::rc::Rc;
use std::cell::RefCell;

/// Optimized image component
pub struct Image {
    src: String,
    alt: String,
    width: Option<u32>,
    height: Option<u32>,
    loading: ImageLoading,
    sizes: Option<String>,
    srcset: Option<String>,
    placeholder: Option<ImagePlaceholder>,
    priority: bool,
    quality: u8,
    on_load: Option<Box<dyn Fn()>>,
    on_error: Option<Box<dyn Fn()>>,
}

#[derive(Clone, Copy)]
pub enum ImageLoading {
    Lazy,
    Eager,
}

#[derive(Clone)]
pub enum ImagePlaceholder {
    Blur(String), // base64 blurred image
    Color(String), // solid color
    Shimmer, // animated shimmer effect
}

impl Image {
    pub fn new(src: impl Into<String>) -> Self {
        Image {
            src: src.into(),
            alt: String::new(),
            width: None,
            height: None,
            loading: ImageLoading::Lazy,
            sizes: None,
            srcset: None,
            placeholder: None,
            priority: false,
            quality: 75,
            on_load: None,
            on_error: None,
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
    
    pub fn loading(mut self, loading: ImageLoading) -> Self {
        self.loading = loading;
        self
    }
    
    pub fn sizes(mut self, sizes: impl Into<String>) -> Self {
        self.sizes = Some(sizes.into());
        self
    }
    
    pub fn srcset(mut self, srcset: impl Into<String>) -> Self {
        self.srcset = Some(srcset.into());
        self
    }
    
    pub fn placeholder(mut self, placeholder: ImagePlaceholder) -> Self {
        self.placeholder = Some(placeholder);
        self
    }
    
    pub fn priority(mut self) -> Self {
        self.priority = true;
        self.loading = ImageLoading::Eager;
        self
    }
    
    pub fn quality(mut self, quality: u8) -> Self {
        self.quality = quality.clamp(1, 100);
        self
    }
    
    pub fn on_load(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_load = Some(Box::new(handler));
        self
    }
    
    pub fn on_error(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_error = Some(Box::new(handler));
        self
    }
    
    fn generate_srcset(&self) -> String {
        if let Some(srcset) = &self.srcset {
            return srcset.clone();
        }
        
        // Generate responsive srcset
        let base_url = self.optimize_url(&self.src);
        let widths = [640, 750, 828, 1080, 1200, 1920, 2048, 3840];
        
        widths.iter()
            .map(|&w| format!("{} {}w", self.optimize_url_with_width(&base_url, w), w))
            .collect::<Vec<_>>()
            .join(", ")
    }
    
    fn optimize_url(&self, url: &str) -> String {
        // In production, this would use an image CDN
        if url.starts_with("http") {
            format!("/_next/image?url={}&q={}", urlencoding::encode(url), self.quality)
        } else {
            url.to_string()
        }
    }
    
    fn optimize_url_with_width(&self, url: &str, width: u32) -> String {
        format!("{}&w={}", url, width)
    }
}

impl Component for Image {
    fn render(&self) -> Element {
        let is_lazy = matches!(self.loading, ImageLoading::Lazy);
        let loaded = use_state(|| !is_lazy);
        let error = use_state(|| false);
        
        // Image attributes
        let mut attrs = vec![
            ("alt".to_string(), self.alt.clone()),
        ];
        
        if let Some(width) = self.width {
            attrs.push(("width".to_string(), width.to_string()));
        }
        
        if let Some(height) = self.height {
            attrs.push(("height".to_string(), height.to_string()));
        }
        
        if let Some(sizes) = &self.sizes {
            attrs.push(("sizes".to_string(), sizes.clone()));
        }
        
        // Generate srcset for responsive images
        let srcset = self.generate_srcset();
        attrs.push(("srcset".to_string(), srcset));
        
        // Lazy loading
        if is_lazy && !loaded.get() {
            attrs.push(("data-src".to_string(), self.optimize_url(&self.src)));
            attrs.push(("src".to_string(), "data:image/svg+xml,%3Csvg%20xmlns='http://www.w3.org/2000/svg'%20width='1'%20height='1'%3E%3C/svg%3E".to_string()));
        } else {
            attrs.push(("src".to_string(), self.optimize_url(&self.src)));
        }
        
        // Loading attribute
        attrs.push(("loading".to_string(), match self.loading {
            ImageLoading::Lazy => "lazy",
            ImageLoading::Eager => "eager",
        }.to_string()));
        
        // Decoding
        attrs.push(("decoding".to_string(), "async".to_string()));
        
        // Style for aspect ratio
        let style = if let (Some(width), Some(height)) = (self.width, self.height) {
            format!("aspect-ratio: {} / {}", width, height)
        } else {
            String::new()
        };
        
        if !style.is_empty() {
            attrs.push(("style".to_string(), style));
        }
        
        // Container for placeholder
        let container_style = style![
            position("relative"),
            overflow("hidden"),
        ];
        
        view! {
            <div style={container_style.build()} class="image-container">
                {if let Some(placeholder) = &self.placeholder {
                    match placeholder {
                        ImagePlaceholder::Blur(blur_url) => {
                            view! {
                                <img
                                    src={blur_url}
                                    alt=""
                                    aria-hidden="true"
                                    style="position: absolute; inset: 0; filter: blur(20px); transform: scale(1.05);"
                                />
                            }
                        }
                        ImagePlaceholder::Color(color) => {
                            view! {
                                <div
                                    style={format!("position: absolute; inset: 0; background-color: {}", color)}
                                    aria-hidden="true"
                                />
                            }
                        }
                        ImagePlaceholder::Shimmer => {
                            view! {
                                <div
                                    class="shimmer"
                                    style="position: absolute; inset: 0;"
                                    aria-hidden="true"
                                />
                            }
                        }
                    }
                } else {
                    view! { <div /> }
                }}
                
                <img
                    {attrs}
                    class={if loaded.get() { "loaded" } else { "loading" }}
                    onload={move || {
                        loaded.set(true);
                        if let Some(handler) = &self.on_load {
                            handler();
                        }
                    }}
                    onerror={move || {
                        error.set(true);
                        if let Some(handler) = &self.on_error {
                            handler();
                        }
                    }}
                />
            </div>
        }
    }
}

/// Picture component for art direction
pub struct Picture {
    sources: Vec<Source>,
    img: Image,
}

pub struct Source {
    media: String,
    srcset: String,
    type_: Option<String>,
}

impl Picture {
    pub fn new(img: Image) -> Self {
        Picture {
            sources: vec![],
            img,
        }
    }
    
    pub fn source(mut self, media: impl Into<String>, srcset: impl Into<String>) -> Self {
        self.sources.push(Source {
            media: media.into(),
            srcset: srcset.into(),
            type_: None,
        });
        self
    }
    
    pub fn source_with_type(
        mut self,
        media: impl Into<String>,
        srcset: impl Into<String>,
        type_: impl Into<String>
    ) -> Self {
        self.sources.push(Source {
            media: media.into(),
            srcset: srcset.into(),
            type_: Some(type_.into()),
        });
        self
    }
}

impl Component for Picture {
    fn render(&self) -> Element {
        view! {
            <picture>
                {self.sources.iter().map(|source| {
                    let mut attrs = vec![
                        ("media".to_string(), source.media.clone()),
                        ("srcset".to_string(), source.srcset.clone()),
                    ];
                    
                    if let Some(type_) = &source.type_ {
                        attrs.push(("type".to_string(), type_.clone()));
                    }
                    
                    Element::Node {
                        tag: "source".to_string(),
                        props: Props {
                            attributes: attrs,
                            ..Default::default()
                        },
                        children: vec![],
                    }
                }).collect::<Vec<_>>()}
                
                {self.img.render()}
            </picture>
        }
    }
}

/// Background image with lazy loading
pub struct BackgroundImage {
    src: String,
    children: Vec<Element>,
    loading: ImageLoading,
}

impl BackgroundImage {
    pub fn new(src: impl Into<String>) -> Self {
        BackgroundImage {
            src: src.into(),
            children: vec![],
            loading: ImageLoading::Lazy,
        }
    }
    
    pub fn children(mut self, children: Vec<Element>) -> Self {
        self.children = children;
        self
    }
    
    pub fn loading(mut self, loading: ImageLoading) -> Self {
        self.loading = loading;
        self
    }
}

impl Component for BackgroundImage {
    fn render(&self) -> Element {
        let loaded = use_state(|| false);
        let container_id = format!("bg-img-{}", js_sys::Math::random());
        
        // Set up intersection observer for lazy loading
        if matches!(self.loading, ImageLoading::Lazy) {
            use_effect(|| {
                let observer = IntersectionObserver::new(
                    &Closure::<dyn FnMut(Vec<IntersectionObserverEntry>)>::new(
                        move |entries: Vec<IntersectionObserverEntry>| {
                            for entry in entries {
                                if entry.is_intersecting() {
                                    loaded.set(true);
                                    // Disconnect observer
                                }
                            }
                        }
                    ).into_js_value().unchecked_ref()
                ).unwrap();
                
                if let Some(element) = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.get_element_by_id(&container_id)) {
                    observer.observe(&element);
                }
                
                move || {
                    observer.disconnect();
                }
            });
        }
        
        let style = if loaded.get() || matches!(self.loading, ImageLoading::Eager) {
            format!("background-image: url('{}')", self.src)
        } else {
            String::new()
        };
        
        Element::Node {
            tag: "div".to_string(),
            props: Props {
                id: Some(container_id),
                attributes: vec![
                    ("style".to_string(), style),
                    ("class".to_string(), "background-image".to_string()),
                ],
                ..Default::default()
            },
            children: self.children.clone(),
        }
    }
}

/// Helper to preload images
pub fn preload_image(src: &str) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let link = document.create_element("link").unwrap();
    
    link.set_attribute("rel", "preload").unwrap();
    link.set_attribute("as", "image").unwrap();
    link.set_attribute("href", src).unwrap();
    
    document.head().unwrap().append_child(&link).unwrap();
}

// Style helpers
fn position(value: &str) -> (&'static str, String) {
    ("position", value.to_string())
}

fn overflow(value: &str) -> (&'static str, String) {
    ("overflow", value.to_string())
}

// Re-exports
use crate::styles::style;