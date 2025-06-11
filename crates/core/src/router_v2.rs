//! Advanced Router with Browser History - L7

use crate::component::{Component, Element};
use crate::state::{create_atom, use_atom, Atom};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{window, History, Location, PopStateEvent};

/// Router configuration
pub struct RouterConfig {
    pub routes: Vec<RouteDefinition>,
    pub not_found: Box<dyn Component>,
}

/// Route definition with params
pub struct RouteDefinition {
    pub path: String,
    pub component: Box<dyn Fn(RouteParams) -> Box<dyn Component>>,
    pub name: Option<String>,
}

/// Route parameters
#[derive(Clone, Debug)]
pub struct RouteParams {
    pub params: HashMap<String, String>,
    pub query: HashMap<String, String>,
}

/// Current route state
#[derive(Clone)]
pub struct RouteState {
    pub path: String,
    pub params: RouteParams,
}

impl Default for RouteState {
    fn default() -> Self {
        RouteState {
            path: "/".to_string(),
            params: RouteParams {
                params: HashMap::new(),
                query: HashMap::new(),
            },
        }
    }
}

/// Router instance
pub struct Router {
    config: Rc<RouterConfig>,
    state: Atom<RouteState>,
    history: History,
    _location: Location,
}

impl Router {
    pub fn new(config: RouterConfig) -> Result<Self, JsValue> {
        let window = window().ok_or("No window")?;
        let history = window.history()?;
        let location = window.location();

        let initial_state = RouteState {
            path: location.pathname()?,
            params: RouteParams {
                params: HashMap::new(),
                query: parse_query(&location.search()?),
            },
        };

        let state = create_atom(initial_state);

        let router = Router {
            config: Rc::new(config),
            state,
            history,
            _location: location,
        };

        // Setup popstate listener
        router.setup_listeners()?;

        Ok(router)
    }

    fn setup_listeners(&self) -> Result<(), JsValue> {
        let state = self.state.clone();
        let config = self.config.clone();

        let closure = Closure::<dyn FnMut(_)>::new(move |_event: PopStateEvent| {
            if let Some(window) = window() {
                if let Ok(pathname) = window.location().pathname() {
                    let new_state = RouteState {
                        path: pathname.clone(),
                        params: match_route(&pathname, &config.routes),
                    };
                    state.set(new_state);
                }
            }
        });

        window()
            .ok_or("No window")?
            .add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref())?;

        closure.forget(); // Keep closure alive

        Ok(())
    }

    pub fn navigate(&self, path: &str) -> Result<(), JsValue> {
        // Update browser history
        self.history
            .push_state_with_url(&JsValue::NULL, "", Some(path))?;

        // Update router state
        let new_state = RouteState {
            path: path.to_string(),
            params: match_route(path, &self.config.routes),
        };
        self.state.set(new_state);

        Ok(())
    }

    pub fn replace(&self, path: &str) -> Result<(), JsValue> {
        // Replace current history entry
        self.history
            .replace_state_with_url(&JsValue::NULL, "", Some(path))?;

        // Update router state
        let new_state = RouteState {
            path: path.to_string(),
            params: match_route(path, &self.config.routes),
        };
        self.state.set(new_state);

        Ok(())
    }

    pub fn back(&self) -> Result<(), JsValue> {
        self.history.back()
    }

    pub fn forward(&self) -> Result<(), JsValue> {
        self.history.forward()
    }
}

/// Match route and extract params
fn match_route(path: &str, routes: &[RouteDefinition]) -> RouteParams {
    for route_def in routes {
        if let Some(params) = match_path(&route_def.path, path) {
            return RouteParams {
                params,
                query: HashMap::new(), // TODO: Parse query params
            };
        }
    }

    RouteParams {
        params: HashMap::new(),
        query: HashMap::new(),
    }
}

/// Match path pattern against actual path
fn match_path(pattern: &str, path: &str) -> Option<HashMap<String, String>> {
    let pattern_parts: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if pattern_parts.len() != path_parts.len() {
        return None;
    }

    let mut params = HashMap::new();

    for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
        if let Some(param_name) = pattern_part.strip_prefix(':') {
            // Dynamic segment
            params.insert(param_name.to_string(), path_part.to_string());
        } else if pattern_part != path_part {
            // Static segment doesn't match
            return None;
        }
    }

    Some(params)
}

/// Parse query string
fn parse_query(query: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();

    if let Some(query) = query.strip_prefix('?') {
        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                params.insert(
                    urlencoding::decode(key).unwrap_or_default().into_owned(),
                    urlencoding::decode(value).unwrap_or_default().into_owned(),
                );
            }
        }
    }

    params
}

thread_local! {
    static ROUTER: RefCell<Option<Rc<Router>>> = const { RefCell::new(None) };
}

/// Initialize router
pub fn init_router(config: RouterConfig) -> Result<(), JsValue> {
    let router = Router::new(config)?;
    ROUTER.with(|r| {
        *r.borrow_mut() = Some(Rc::new(router));
    });
    Ok(())
}

/// Use router hook
pub fn use_router() -> Option<Rc<Router>> {
    ROUTER.with(|r| r.borrow().clone())
}

/// Use current route
pub fn use_route() -> Option<RouteState> {
    let router = use_router()?;
    let handle = use_atom(&router.state);
    handle.get().cloned()
}

/// Link component
pub struct Link {
    to: String,
    children: Vec<Element>,
    class: Option<String>,
}

impl Link {
    pub fn new(to: impl Into<String>) -> Self {
        Link {
            to: to.into(),
            children: vec![],
            class: None,
        }
    }

    pub fn children(mut self, children: Vec<Element>) -> Self {
        self.children = children;
        self
    }

    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = Some(class.into());
        self
    }
}

impl Component for Link {
    fn render(&self) -> Element {
        let to = self.to.clone();
        let on_click = Some(Rc::new(move || {
            if let Some(router) = use_router() {
                let _ = router.navigate(&to);
            }
        }) as Rc<dyn Fn()>);

        Element::Node {
            tag: "a".to_string(),
            props: crate::component::Props {
                class: self.class.clone(),
                on_click,
                attributes: vec![
                    ("href".to_string(), self.to.clone()),
                    ("onclick".to_string(), "event.preventDefault()".to_string()),
                ],
                ..Default::default()
            },
            children: self.children.clone(),
        }
    }
}

/// Route component - renders matching route
pub struct Route;

impl Component for Route {
    fn render(&self) -> Element {
        if let Some(router) = use_router() {
            if let Some(route_state) = use_route() {
                // Find matching route
                for route_def in &router.config.routes {
                    if match_path(&route_def.path, &route_state.path).is_some() {
                        let component = (route_def.component)(route_state.params.clone());
                        return component.render();
                    }
                }

                // No match - render 404
                return router.config.not_found.render();
            }
        }

        // Fallback
        Element::Text("Router not initialized".to_string())
    }
}

/// Navigate programmatically
pub fn navigate(path: &str) -> Result<(), JsValue> {
    if let Some(router) = use_router() {
        router.navigate(path)
    } else {
        Err("Router not initialized".into())
    }
}

/// Create route definition helper
pub fn route(
    path: impl Into<String>,
    component: impl Fn(RouteParams) -> Box<dyn Component> + 'static,
) -> RouteDefinition {
    RouteDefinition {
        path: path.into(),
        component: Box::new(component),
        name: None,
    }
}

/// Create route with name
pub fn named_route(
    name: impl Into<String>,
    path: impl Into<String>,
    component: impl Fn(RouteParams) -> Box<dyn Component> + 'static,
) -> RouteDefinition {
    RouteDefinition {
        path: path.into(),
        component: Box::new(component),
        name: Some(name.into()),
    }
}
