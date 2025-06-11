//! Router System - L7

use crate::component::{Component, Element};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Page definition
pub struct Page {
    pub title: String,
    pub meta: HashMap<String, String>,
    pub component: Box<dyn Component>,
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
pub struct Route {
    pub path: String,
    pub handler: RouteHandler,
}

pub enum RouteHandler {
    Page(fn() -> Page),
    Api(fn() -> JsValue),
    Redirect(String),
}

/// Router
pub struct Router {
    routes: HashMap<String, Route>,
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
            .get_element_by_id("warp-root")
            .expect("No #warp-root element found");
        
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
        view! { <div>"Empty Page"</div> }
    }
}

/// 404 component
struct NotFoundComponent;

impl Component for NotFoundComponent {
    fn render(&self) -> Element {
        view! {
            <div class="not-found">
                <h1>"404"</h1>
                <p>"Page not found"</p>
            </div>
        }
    }
}