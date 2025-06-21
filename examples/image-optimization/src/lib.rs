//! Image Optimization Example
//! Demonstrates all image optimization features in Layer9

use layer9_core::prelude::*;
use layer9_core::image::{ImagePlaceholder, BackgroundImage, ImageLoading};
use wasm_bindgen::prelude::*;

// Demo styles
#[allow(dead_code)]
const DEMO_STYLES: &str = r#"
* {
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    margin: 0;
    padding: 0;
    background: #f5f5f5;
    color: #333;
}

.image-gallery {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
}

h1 {
    text-align: center;
    margin-bottom: 3rem;
    font-size: 2.5rem;
    color: #000;
}

h2 {
    margin: 3rem 0 1.5rem;
    font-size: 1.8rem;
    color: #333;
}

section {
    background: white;
    padding: 2rem;
    margin-bottom: 2rem;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 1.5rem;
}

.gallery-image {
    width: 100%;
    height: auto;
    border-radius: 4px;
    transition: opacity 0.3s ease;
}

.gallery-image.loading {
    opacity: 0;
}

.gallery-image.loaded {
    opacity: 1;
}

.format-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 2rem;
}

.format-grid > div {
    text-align: center;
}

.format-grid h3 {
    margin-bottom: 1rem;
    font-size: 1.2rem;
}

.quality-demo-container {
    max-width: 800px;
    margin: 0 auto;
}

.controls {
    margin-bottom: 2rem;
}

.controls label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 600;
}

.controls input[type="range"] {
    width: 100%;
    height: 8px;
    border-radius: 4px;
    background: #ddd;
    outline: none;
}

.quality-comparison img {
    width: 100%;
    height: auto;
    border-radius: 4px;
}

.background-demo {
    position: relative;
    min-height: 400px;
    display: flex;
    align-items: center;
    justify-content: center;
}

.background-image {
    position: absolute;
    inset: 0;
    background-size: cover;
    background-position: center;
    transition: opacity 0.5s ease;
}

.content-overlay {
    position: relative;
    z-index: 1;
    background: rgba(255, 255, 255, 0.9);
    padding: 2rem;
    border-radius: 8px;
    text-align: center;
}

/* Loading shimmer effect */
.shimmer {
    background: linear-gradient(
        90deg,
        #f0f0f0 0%,
        #f8f8f8 50%,
        #f0f0f0 100%
    );
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
}

@keyframes shimmer {
    0% {
        background-position: -200% 0;
    }
    100% {
        background-position: 200% 0;
    }
}

/* Responsive adjustments */
@media (max-width: 768px) {
    .image-gallery {
        padding: 1rem;
    }
    
    section {
        padding: 1rem;
    }
    
    h1 {
        font-size: 2rem;
    }
    
    h2 {
        font-size: 1.5rem;
    }
    
    .grid {
        grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
        gap: 1rem;
    }
}
"#;

/// Image gallery component demonstrating various optimization techniques
pub struct ImageGallery;

impl Component for ImageGallery {
    fn render(&self) -> Element {
        Element::Node {
            tag: "div".to_string(),
            props: Props {
                class: Some("image-gallery".to_string()),
                ..Default::default()
            },
            children: vec![
                // Title
                Element::Node {
                    tag: "h1".to_string(),
                    props: Props::default(),
                    children: vec![Element::Text("Layer9 Image Optimization Demo".to_string())],
                },
                
                // Hero section
                render_hero_section(),
                
                // Lazy gallery section
                render_lazy_gallery(),
                
                // Art direction section
                render_art_direction(),
                
                // Blur placeholder section
                render_blur_placeholder(),
                
                // Format demo section
                render_format_demo(),
                
                // Quality demo section
                QualityDemo.render(),
                
                // Background image section
                render_background_demo(),
            ],
        }
    }
}

fn render_hero_section() -> Element {
    Element::Node {
        tag: "section".to_string(),
        props: Props {
            class: Some("hero-section".to_string()),
            ..Default::default()
        },
        children: vec![
            Element::Node {
                tag: "h2".to_string(),
                props: Props::default(),
                children: vec![Element::Text("Hero Image with Responsive Loading".to_string())],
            },
            Image::new("/images/hero.jpg")
                .alt("Hero image demonstrating responsive loading")
                .width(1920)
                .height(1080)
                .sizes("(max-width: 640px) 100vw, (max-width: 1024px) 90vw, 1200px")
                .quality(90)
                .priority()
                .render(),
        ],
    }
}

fn render_lazy_gallery() -> Element {
    Element::Node {
        tag: "section".to_string(),
        props: Props {
            class: Some("lazy-gallery".to_string()),
            ..Default::default()
        },
        children: vec![
            Element::Node {
                tag: "h2".to_string(),
                props: Props::default(),
                children: vec![Element::Text("Lazy Loaded Image Gallery".to_string())],
            },
            Element::Node {
                tag: "div".to_string(),
                props: Props {
                    class: Some("grid".to_string()),
                    ..Default::default()
                },
                children: (1..=12).map(|i| {
                    LazyImage::new(format!("/images/gallery/photo-{}.jpg", i))
                        .alt(format!("Gallery photo {}", i))
                        .width(400)
                        .height(300)
                        .placeholder(format!("/images/gallery/photo-{}-placeholder.jpg", i))
                        .class("gallery-image")
                        .srcset(format!(
                            "/images/gallery/photo-{}-400w.jpg 400w, /images/gallery/photo-{}-800w.jpg 800w",
                            i, i
                        ))
                        .sizes("(max-width: 640px) 100vw, (max-width: 1024px) 50vw, 400px")
                        .render()
                }).collect(),
            },
        ],
    }
}

fn render_art_direction() -> Element {
    Element::Node {
        tag: "section".to_string(),
        props: Props {
            class: Some("art-direction".to_string()),
            ..Default::default()
        },
        children: vec![
            Element::Node {
                tag: "h2".to_string(),
                props: Props::default(),
                children: vec![Element::Text("Art Direction with Picture Element".to_string())],
            },
            Picture::new(
                Image::new("/images/product.jpg")
                    .alt("Product image with art direction")
            )
            .source_with_type(
                "(max-width: 640px)",
                "/images/product-mobile.webp",
                "image/webp"
            )
            .source_with_type(
                "(max-width: 640px)",
                "/images/product-mobile.jpg",
                "image/jpeg"
            )
            .source_with_type(
                "(min-width: 641px)",
                "/images/product-desktop.webp",
                "image/webp"
            )
            .render(),
        ],
    }
}

fn render_blur_placeholder() -> Element {
    Element::Node {
        tag: "section".to_string(),
        props: Props {
            class: Some("blur-placeholder".to_string()),
            ..Default::default()
        },
        children: vec![
            Element::Node {
                tag: "h2".to_string(),
                props: Props::default(),
                children: vec![Element::Text("Image with Blur Placeholder".to_string())],
            },
            Image::new("/images/landscape.jpg")
                .alt("Beautiful landscape with blur placeholder")
                .width(1200)
                .height(800)
                .placeholder(ImagePlaceholder::Blur(
                    "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQAAAQ...".to_string()
                ))
                .render(),
        ],
    }
}

fn render_format_demo() -> Element {
    Element::Node {
        tag: "section".to_string(),
        props: Props {
            class: Some("format-demo".to_string()),
            ..Default::default()
        },
        children: vec![
            Element::Node {
                tag: "h2".to_string(),
                props: Props::default(),
                children: vec![Element::Text("Format Optimization Demo".to_string())],
            },
            Element::Node {
                tag: "div".to_string(),
                props: Props {
                    class: Some("format-grid".to_string()),
                    ..Default::default()
                },
                children: vec![
                    render_format_item("Original JPEG", "/images/sample.jpg", "Original JPEG image"),
                    render_format_item("Optimized WebP", "/_layer9/image?src=/images/sample.jpg&f=webp", "WebP optimized image"),
                    render_format_item("AVIF Format", "/_layer9/image?src=/images/sample.jpg&f=avif", "AVIF optimized image"),
                ],
            },
        ],
    }
}

fn render_format_item(title: &str, src: &str, alt: &str) -> Element {
    Element::Node {
        tag: "div".to_string(),
        props: Props::default(),
        children: vec![
            Element::Node {
                tag: "h3".to_string(),
                props: Props::default(),
                children: vec![Element::Text(title.to_string())],
            },
            Image::new(src)
                .alt(alt)
                .width(600)
                .height(400)
                .render(),
        ],
    }
}

fn render_background_demo() -> Element {
    Element::Node {
        tag: "section".to_string(),
        props: Props {
            class: Some("background-demo".to_string()),
            ..Default::default()
        },
        children: vec![
            Element::Node {
                tag: "h2".to_string(),
                props: Props::default(),
                children: vec![Element::Text("Background Image Optimization".to_string())],
            },
            BackgroundImage::new("/images/background.jpg")
                .loading(ImageLoading::Lazy)
                .children(vec![
                    Element::Node {
                        tag: "div".to_string(),
                        props: Props {
                            class: Some("content-overlay".to_string()),
                            ..Default::default()
                        },
                        children: vec![
                            Element::Node {
                                tag: "h3".to_string(),
                                props: Props::default(),
                                children: vec![Element::Text("Content over optimized background".to_string())],
                            },
                            Element::Node {
                                tag: "p".to_string(),
                                props: Props::default(),
                                children: vec![Element::Text("Background images are also lazily loaded and optimized".to_string())],
                            },
                        ],
                    }
                ])
                .render(),
        ],
    }
}

/// Interactive quality demo component
struct QualityDemo;

impl Component for QualityDemo {
    fn render(&self) -> Element {
        let quality = use_state(|| 85u8);
        let quality_value = quality.get();
        
        Element::Node {
            tag: "section".to_string(),
            props: Props {
                class: Some("quality-demo".to_string()),
                ..Default::default()
            },
            children: vec![
                Element::Node {
                    tag: "h2".to_string(),
                    props: Props::default(),
                    children: vec![Element::Text("Quality Settings Demo".to_string())],
                },
                Element::Node {
                    tag: "div".to_string(),
                    props: Props {
                        class: Some("quality-demo-container".to_string()),
                        ..Default::default()
                    },
                    children: vec![
                        Element::Node {
                            tag: "div".to_string(),
                            props: Props {
                                class: Some("controls".to_string()),
                                ..Default::default()
                            },
                            children: vec![
                                Element::Node {
                                    tag: "label".to_string(),
                                    props: Props {
                                        attributes: vec![("for".to_string(), "quality-slider".to_string())],
                                        ..Default::default()
                                    },
                                    children: vec![
                                        Element::Text(format!("Quality: {}%", quality_value)),
                                    ],
                                },
                                Element::Node {
                                    tag: "input".to_string(),
                                    props: Props {
                                        attributes: vec![
                                            ("type".to_string(), "range".to_string()),
                                            ("id".to_string(), "quality-slider".to_string()),
                                            ("min".to_string(), "1".to_string()),
                                            ("max".to_string(), "100".to_string()),
                                            ("value".to_string(), quality_value.to_string()),
                                        ],
                                        ..Default::default()
                                    },
                                    children: vec![],
                                },
                            ],
                        },
                        Element::Node {
                            tag: "div".to_string(),
                            props: Props {
                                class: Some("quality-comparison".to_string()),
                                ..Default::default()
                            },
                            children: vec![
                                Image::new("/images/quality-test.jpg")
                                    .alt(format!("Image at {}% quality", quality_value))
                                    .width(800)
                                    .height(600)
                                    .quality(quality_value)
                                    .render(),
                            ],
                        },
                    ],
                },
            ],
        }
    }
}

/// Image optimization service worker registration
fn register_service_worker() {
    if let Some(window) = web_sys::window() {
        let navigator = window.navigator();
        let service_worker = navigator.service_worker();
        wasm_bindgen_futures::spawn_local(async move {
            let promise = service_worker.register("/sw.js");
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(_) => web_sys::console::log_1(&"Service Worker registered".into()),
                Err(e) => web_sys::console::error_1(&format!("SW registration failed: {:?}", e).into()),
            }
        });
    }
}

/// Initialize the demo app
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
    
    // Initialize lazy loading manager
    LazyLoadManager::init(Default::default());
    
    // Register service worker for offline image caching
    register_service_worker();
    
    // Inject styles
    inject_global_styles();
    
    // Mount the app
    mount(Box::new(ImageGallery), "app");
}