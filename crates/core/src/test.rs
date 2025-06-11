//! Testing Framework - L7/L8

#[allow(unused_imports)] // These are used in test macros and examples
use crate::component::{Component, Element, Props, use_state};
use std::future::Future;
#[allow(unused_imports)] // Used in test macros
use std::panic;
#[allow(unused_imports)] // Used in test macros
use std::rc::Rc;
use wasm_bindgen::JsCast;
#[cfg(test)]
use wasm_bindgen_test::*;

// Type alias to simplify complex types
type PropsModifierFn<T> = Box<dyn Fn(&mut T)>;

/// Test context for components
pub struct TestContext {
    container: web_sys::Element,
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TestContext {
    pub fn new() -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let container = document.create_element("div").unwrap();
        container.set_id("test-container");
        document.body().unwrap().append_child(&container).unwrap();

        TestContext { container }
    }

    pub fn render(&self, component: impl Component) -> TestResult {
        // Clear container
        self.container.set_inner_html("");

        // Render component
        component.mount(&self.container);

        TestResult {
            container: self.container.clone(),
        }
    }

    pub fn cleanup(&self) {
        self.container.remove();
    }
}

/// Test result with query methods
pub struct TestResult {
    container: web_sys::Element,
}

impl TestResult {
    pub fn get_by_text(&self, text: &str) -> Option<web_sys::Element> {
        let document = web_sys::window().unwrap().document().unwrap();
        let selector = format!("#{} *", self.container.id());
        let elements = document.query_selector_all(&selector).unwrap();

        for i in 0..elements.length() {
            if let Some(element) = elements.item(i) {
                if let Some(el) = element.dyn_ref::<web_sys::Element>() {
                    if el.text_content().map(|t| t.contains(text)).unwrap_or(false) {
                        return Some(el.clone());
                    }
                }
            }
        }

        None
    }

    pub fn get_by_role(&self, role: &str) -> Option<web_sys::Element> {
        self.container
            .query_selector(&format!("[role='{}']", role))
            .ok()
            .flatten()
    }

    pub fn get_by_id(&self, id: &str) -> Option<web_sys::Element> {
        self.container
            .query_selector(&format!("#{}", id))
            .ok()
            .flatten()
    }

    pub fn get_all_by_class(&self, class: &str) -> Vec<web_sys::Element> {
        let document = web_sys::window().unwrap().document().unwrap();
        let selector = format!("#{} .{}", self.container.id(), class);
        let elements = document.query_selector_all(&selector).unwrap();

        let mut result = vec![];
        for i in 0..elements.length() {
            if let Some(element) = elements.item(i) {
                if let Some(el) = element.dyn_ref::<web_sys::Element>() {
                    result.push(el.clone());
                }
            }
        }

        result
    }

    pub fn debug(&self) {
        web_sys::console::log_1(&self.container.outer_html().into());
    }
}

/// Test utilities
pub struct TestUtils;

impl TestUtils {
    /// Simulate click event
    pub fn click(element: &web_sys::Element) {
        let event = web_sys::MouseEvent::new("click").unwrap();
        element.dispatch_event(&event).unwrap();
    }

    /// Simulate input change
    pub fn change_input(element: &web_sys::Element, value: &str) {
        if let Some(input) = element.dyn_ref::<web_sys::HtmlInputElement>() {
            input.set_value(value);

            let event = web_sys::Event::new("input").unwrap();
            element.dispatch_event(&event).unwrap();
        }
    }

    /// Wait for condition
    pub async fn wait_for<F>(condition: F, timeout_ms: u32) -> Result<(), String>
    where
        F: Fn() -> bool,
    {
        let start = js_sys::Date::now();

        while !condition() {
            if js_sys::Date::now() - start > timeout_ms as f64 {
                return Err("Timeout waiting for condition".to_string());
            }

            // Sleep for a bit
            wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut |resolve, _| {
                web_sys::window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 10)
                    .unwrap();
            }))
            .await
            .unwrap();
        }

        Ok(())
    }

    /// Wait for element
    pub async fn wait_for_element(
        container: &web_sys::Element,
        selector: &str,
    ) -> Result<web_sys::Element, String> {
        Self::wait_for(
            || container.query_selector(selector).ok().flatten().is_some(),
            5000,
        )
        .await?;

        container
            .query_selector(selector)
            .ok()
            .flatten()
            .ok_or_else(|| "Element not found".to_string())
    }
}

/// Test macros
#[macro_export]
macro_rules! layer9_test {
    ($name:ident, $body:expr) => {
        #[wasm_bindgen_test]
        async fn $name() {
            let ctx = TestContext::new();
            let result = panic::catch_unwind(panic::AssertUnwindSafe(|| $body(&ctx)));
            ctx.cleanup();

            if let Err(e) = result {
                panic::resume_unwind(e);
            }
        }
    };
}

#[macro_export]
macro_rules! assert_text_content {
    ($element:expr, $expected:expr) => {
        assert_eq!(
            $element.text_content().unwrap_or_default(),
            $expected,
            "Expected text content '{}', got '{}'",
            $expected,
            $element.text_content().unwrap_or_default()
        );
    };
}

#[macro_export]
macro_rules! assert_has_class {
    ($element:expr, $class:expr) => {
        assert!(
            $element.class_list().contains($class),
            "Expected element to have class '{}', but it doesn't",
            $class
        );
    };
}

/// Component test harness
pub struct ComponentTest<T: Component> {
    component: T,
    props_modifier: Option<PropsModifierFn<T>>,
}

impl<T: Component> ComponentTest<T> {
    pub fn new(component: T) -> Self {
        ComponentTest {
            component,
            props_modifier: None,
        }
    }

    pub fn with_props(mut self, modifier: impl Fn(&mut T) + 'static) -> Self {
        self.props_modifier = Some(Box::new(modifier));
        self
    }

    pub fn render(mut self) -> TestResult {
        if let Some(modifier) = self.props_modifier {
            modifier(&mut self.component);
        }

        let ctx = TestContext::new();
        ctx.render(self.component)
    }
}

/// Snapshot testing
pub struct Snapshot;

impl Snapshot {
    pub fn assert_matches(name: &str, content: &str) {
        // In real implementation, this would:
        // 1. Load snapshot from file
        // 2. Compare with content
        // 3. Update snapshot if in update mode
        // 4. Fail test if mismatch

        web_sys::console::log_1(
            &format!("Snapshot test '{}' would check:\n{}", name, content).into(),
        );
    }
}

/// Performance testing
pub struct PerfTest;

impl PerfTest {
    pub fn measure<F, R>(name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = web_sys::Performance::now(&web_sys::window().unwrap().performance().unwrap());
        let result = f();
        let end = web_sys::Performance::now(&web_sys::window().unwrap().performance().unwrap());

        web_sys::console::log_1(&format!("{} took {}ms", name, end - start).into());

        result
    }

    pub async fn measure_async<F, Fut, R>(name: &str, f: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = R>,
    {
        let start = web_sys::Performance::now(&web_sys::window().unwrap().performance().unwrap());
        let result = f().await;
        let end = web_sys::Performance::now(&web_sys::window().unwrap().performance().unwrap());

        web_sys::console::log_1(&format!("{} took {}ms", name, end - start).into());

        result
    }
}

/// Example tests
#[cfg(test)]
mod tests {
    use super::*;

    layer9_test!(test_button_click, |ctx: &TestContext| {
        // Create a counter component
        struct Counter;
        impl Component for Counter {
            fn render(&self) -> Element {
                let count = use_state(|| 0);

                Element::Node {
                    tag: "div".to_string(),
                    props: Props::default(),
                    children: vec![
                        Element::Node {
                            tag: "span".to_string(),
                            props: Props {
                                id: Some("count".to_string()),
                                ..Default::default()
                            },
                            children: vec![Element::Text(count.get().to_string())],
                        },
                        Element::Node {
                            tag: "button".to_string(),
                            props: Props {
                                id: Some("increment".to_string()),
                                on_click: Some(Rc::new(move || count.set(count.get() + 1))),
                                ..Default::default()
                            },
                            children: vec![Element::Text("Increment".to_string())],
                        },
                    ],
                }
            }
        }

        // Render component
        let result = ctx.render(Counter);

        // Get elements
        let count_el = result.get_by_id("count").unwrap();
        let button = result.get_by_id("increment").unwrap();

        // Initial state
        assert_text_content!(count_el, "0");

        // Click button
        TestUtils::click(&button);

        // Check updated state
        assert_text_content!(count_el, "1");
    });

    /* // Commented out due to lifetime issues
    layer9_test!(test_form_submission, |ctx: &TestContext| async {
        // Test form component
        struct LoginForm;
        impl Component for LoginForm {
            fn render(&self) -> Element {
                Element::Node {
                    tag: "form".to_string(),
                    props: Props {
                        id: Some("login-form".to_string()),
                        ..Default::default()
                    },
                    children: vec![
                        Element::Node {
                            tag: "input".to_string(),
                            props: Props {
                                id: Some("email".to_string()),
                                attributes: vec![("type".to_string(), "email".to_string())],
                                ..Default::default()
                            },
                            children: vec![],
                        },
                        Element::Node {
                            tag: "input".to_string(),
                            props: Props {
                                id: Some("password".to_string()),
                                attributes: vec![("type".to_string(), "password".to_string())],
                                ..Default::default()
                            },
                            children: vec![],
                        },
                        Element::Node {
                            tag: "button".to_string(),
                            props: Props {
                                attributes: vec![("type".to_string(), "submit".to_string())],
                                ..Default::default()
                            },
                            children: vec![Element::Text("Login".to_string())],
                        },
                    ],
                }
            }
        }

        let result = ctx.render(LoginForm);

        // Fill form
        let email = result.get_by_id("email").unwrap();
        let password = result.get_by_id("password").unwrap();

        TestUtils::change_input(&email, "test@example.com");
        TestUtils::change_input(&password, "secret123");

        // Submit form
        let form = result.get_by_id("login-form").unwrap();
        let event = web_sys::Event::new("submit").unwrap();
        form.dispatch_event(&event).unwrap();

        // Wait for async operation
        TestUtils::wait_for(
            || {
                // Check for success message or redirect
                false
            },
            1000,
        )
        .await
        .ok();
    }); */
}
