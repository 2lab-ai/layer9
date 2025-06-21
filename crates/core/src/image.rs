//! Image Optimization - L5

use crate::component::{use_state, Component};
use crate::prelude::*;
use crate::hooks::use_effect;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{IntersectionObserver, IntersectionObserverEntry};

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
    on_load: Option<Rc<dyn Fn()>>,
    on_error: Option<Rc<dyn Fn()>>,
}

#[derive(Clone, Copy)]
pub enum ImageLoading {
    Lazy,
    Eager,
}

#[derive(Clone)]
pub enum ImagePlaceholder {
    Blur(String),  // base64 blurred image
    Color(String), // solid color
    Shimmer,       // animated shimmer effect
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
        self.on_load = Some(Rc::new(handler));
        self
    }

    pub fn on_error(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_error = Some(Rc::new(handler));
        self
    }

    fn generate_srcset(&self) -> String {
        if let Some(srcset) = &self.srcset {
            return srcset.clone();
        }

        // Generate responsive srcset with Layer9 optimization
        let widths = [640, 750, 828, 1080, 1200, 1920, 2048, 3840];

        widths
            .iter()
            .map(|&w| format!("{} {}w", self.optimize_url_with_width(&self.src, w), w))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn optimize_url(&self, url: &str) -> String {
        // Use Layer9 image optimization endpoint
        if self.width.is_some() || self.height.is_some() || self.quality != 75 {
            let mut params = vec![];
            
            if let Some(w) = self.width {
                params.push(format!("w={}", w));
            }
            if let Some(h) = self.height {
                params.push(format!("h={}", h));
            }
            params.push(format!("q={}", self.quality));
            
            format!(
                "/_layer9/image?src={}&{}",
                urlencoding::encode(url),
                params.join("&")
            )
        } else {
            url.to_string()
        }
    }

    fn optimize_url_with_width(&self, url: &str, width: u32) -> String {
        format!(
            "/_layer9/image?src={}&w={}&q={}",
            urlencoding::encode(url),
            width,
            self.quality
        )
    }
}

impl Component for Image {
    fn render(&self) -> Element {
        let is_lazy = matches!(self.loading, ImageLoading::Lazy);
        let loaded = use_state(|| !is_lazy);
        let _error = use_state(|| false);

        // Image attributes
        let mut attrs = vec![("alt".to_string(), self.alt.clone())];

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
        attrs.push((
            "loading".to_string(),
            match self.loading {
                ImageLoading::Lazy => "lazy",
                ImageLoading::Eager => "eager",
            }
            .to_string(),
        ));

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
        // Note: position and overflow are not available in StyleBuilder
        // We'll add them directly to the style string
        let container_style = "position: relative; overflow: hidden";

        let placeholder_element = if let Some(placeholder) = &self.placeholder {
            match placeholder {
                ImagePlaceholder::Blur(blur_url) => {
                    Element::Node {
                        tag: "img".to_string(),
                        props: Props {
                            attributes: vec![
                                ("src".to_string(), blur_url.clone()),
                                ("alt".to_string(), "".to_string()),
                                ("aria-hidden".to_string(), "true".to_string()),
                                ("style".to_string(), "position: absolute; inset: 0; filter: blur(20px); transform: scale(1.05);".to_string()),
                            ],
                            ..Default::default()
                        },
                        children: vec![],
                    }
                }
                ImagePlaceholder::Color(color) => {
                    Element::Node {
                        tag: "div".to_string(),
                        props: Props {
                            attributes: vec![
                                ("style".to_string(), format!("position: absolute; inset: 0; background-color: {}", color)),
                                ("aria-hidden".to_string(), "true".to_string()),
                            ],
                            ..Default::default()
                        },
                        children: vec![],
                    }
                }
                ImagePlaceholder::Shimmer => {
                    Element::Node {
                        tag: "div".to_string(),
                        props: Props {
                            class: Some("shimmer".to_string()),
                            attributes: vec![
                                ("style".to_string(), "position: absolute; inset: 0;".to_string()),
                                ("aria-hidden".to_string(), "true".to_string()),
                            ],
                            ..Default::default()
                        },
                        children: vec![],
                    }
                }
            }
        } else {
            Element::Node {
                tag: "div".to_string(),
                props: Props::default(),
                children: vec![],
            }
        };

        let _on_load = self.on_load.clone();
        let _on_error = self.on_error.clone();

        let img_props = Props {
            class: Some(if loaded.get() { "loaded" } else { "loading" }.to_string()),
            attributes: attrs,
            ..Default::default()
        };

        // Note: In a real implementation, we'd need to handle onload and onerror properly
        // For now, we'll just add them as attributes

        Element::Node {
            tag: "div".to_string(),
            props: Props {
                class: Some("image-container".to_string()),
                attributes: vec![("style".to_string(), container_style.to_string())],
                ..Default::default()
            },
            children: vec![
                placeholder_element,
                Element::Node {
                    tag: "img".to_string(),
                    props: img_props,
                    children: vec![],
                },
            ],
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
        type_: impl Into<String>,
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
        Element::Node {
            tag: "picture".to_string(),
            props: Props::default(),
            children: {
                let mut children: Vec<Element> = self
                    .sources
                    .iter()
                    .map(|source| {
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
                    })
                    .collect();

                children.push(self.img.render());
                children
            },
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
            let loaded_clone = loaded.clone();
            let container_id_clone = container_id.clone();
            use_effect((), move || {
                let observer = IntersectionObserver::new(
                    Closure::<dyn FnMut(Vec<IntersectionObserverEntry>)>::new(
                        move |entries: Vec<IntersectionObserverEntry>| {
                            for entry in entries {
                                if entry.is_intersecting() {
                                    loaded_clone.set(true);
                                    // Disconnect observer
                                }
                            }
                        },
                    )
                    .into_js_value()
                    .unchecked_ref(),
                )
                .unwrap();

                if let Some(element) = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.get_element_by_id(&container_id_clone))
                {
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

// Re-exports
