//! Router System - L7

use crate::component::{Component, Element, Props};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Page definition
pub struct Page {
    pub title: String,
    pub meta: HashMap<String, String>,
    pub component: Box<dyn Component>,
}

impl Default for Page {
    fn default() -> Self {
        Self::new()
    }
}

impl Page {
    pub fn new() -> Self {
        Page {
            title: String::new(),
            meta: HashMap::new(),
            component: Box::new(EmptyComponent),
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn component(mut self, component: impl Component + 'static) -> Self {
        self.component = Box::new(component);
        self
    }

    pub fn meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.meta.insert(key.into(), value.into());
        self
    }
}

/// Route definition
#[derive(Clone)]
pub struct Route {
    pub path: String,
    pub handler: RouteHandler,
}

#[derive(Clone)]
pub enum RouteHandler {
    Page(fn() -> Page),
    Api(fn() -> JsValue),
    Redirect(String),
}

/// Router
pub struct Router {
    routes: HashMap<String, Route>,
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, route: Route) {
        self.routes.insert(route.path.clone(), route);
    }

    pub fn navigate(&self, path: &str) {
        if let Some(route) = self.routes.get(path) {
            match &route.handler {
                RouteHandler::Page(handler) => {
                    let page = handler();
                    self.render_page(page);
                }
                RouteHandler::Api(_) => {
                    web_sys::console::error_1(&"Cannot navigate to API route".into());
                }
                RouteHandler::Redirect(to) => {
                    self.navigate(to);
                }
            }
        } else {
            self.render_404();
        }
    }

    fn render_page(&self, page: Page) {
        // Update document title
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .set_title(&page.title);

        // Render component
        let root = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("layer9-root")
            .expect("No #layer9-root element found");

        // Clear existing content
        root.set_inner_html("");

        // Mount new component
        page.component.mount(&root);
    }

    fn render_404(&self) {
        let page = Page::new()
            .title("404 - Not Found")
            .component(NotFoundComponent);
        self.render_page(page);
    }
}

/// Empty component for default
struct EmptyComponent;

impl Component for EmptyComponent {
    fn render(&self) -> Element {
        Element::Node {
            tag: "div".to_string(),
            props: Props::default(),
            children: vec![Element::Text("Empty Page".to_string())],
        }
    }
}

/// 404 component
struct NotFoundComponent;

impl Component for NotFoundComponent {
    fn render(&self) -> Element {
        Element::Node {
            tag: "div".to_string(),
            props: Props {
                class: Some("not-found".to_string()),
                ..Default::default()
            },
            children: vec![
                Element::Node {
                    tag: "h1".to_string(),
                    props: Props::default(),
                    children: vec![Element::Text("404".to_string())],
                },
                Element::Node {
                    tag: "p".to_string(),
                    props: Props::default(),
                    children: vec![Element::Text("Page not found".to_string())],
                },
            ],
        }
    }
}
