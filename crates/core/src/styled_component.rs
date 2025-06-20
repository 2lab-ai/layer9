//! Styled Components - CSS-in-Rust component integration
//!
//! This module provides integration between the CSS runtime and the component system,
//! allowing components to use dynamic CSS with full support for hover states,
//! media queries, and more.

use crate::component::{Component, Element, Props};
use crate::css_runtime::{CssBuilder, CssRule};

/// A component wrapper that applies CSS styles dynamically
pub struct StyledComponent<C: Component> {
    inner: C,
    class_name: String,
}

impl<C: Component> StyledComponent<C> {
    /// Create a new styled component with a CSS rule
    pub fn new(component: C, css_rule: CssRule) -> Self {
        let mut manager = crate::css_runtime::STYLESHEET_MANAGER.lock();
        let class_name = manager.add_rule(&css_rule);
        
        Self {
            inner: component,
            class_name,
        }
    }
    
    /// Create a styled component using the CSS builder
    pub fn from_builder(component: C, builder: CssBuilder) -> Self {
        let class_name = builder.build();
        
        Self {
            inner: component,
            class_name,
        }
    }
}

impl<C: Component> Component for StyledComponent<C> {
    fn render(&self) -> Element {
        let mut element = self.inner.render();
        
        // Apply the generated class name to the root element
        match &mut element {
            Element::Node { props, .. } => {
                match &mut props.class {
                    Some(existing) => {
                        // Append to existing classes
                        *existing = format!("{} {}", existing, self.class_name);
                    }
                    None => {
                        props.class = Some(self.class_name.clone());
                    }
                }
            }
            _ => {
                // Wrap non-node elements in a div with the class
                element = Element::Node {
                    tag: "div".to_string(),
                    props: Props {
                        class: Some(self.class_name.clone()),
                        ..Default::default()
                    },
                    children: vec![element],
                };
            }
        }
        
        element
    }
}

/// Extension trait for components to add styling capabilities
pub trait ComponentStyling: Component + Sized {
    /// Apply CSS styles to this component
    fn styled(self, styles: CssBuilder) -> StyledComponent<Self> {
        StyledComponent::from_builder(self, styles)
    }
    
    /// Apply CSS styles with a custom class prefix
    fn styled_with_prefix(self, prefix: &str, styles: CssBuilder) -> StyledComponent<Self> {
        StyledComponent::from_builder(self, styles.prefix(prefix))
    }
}

// Implement the extension trait for all components
impl<T: Component> ComponentStyling for T {}

/// Helper macro for creating styled components with inline CSS
#[macro_export]
macro_rules! styled {
    ($component:expr, {
        $($prop:ident: $value:expr),* $(,)?
    }) => {{
        let mut builder = $crate::css_runtime::CssBuilder::new();
        
        // Add base properties
        $(
            builder = builder.property(stringify!($prop), &$value.to_string());
        )*
        
        $crate::styled_component::StyledComponent::from_builder($component, builder)
    }};
}

pub use styled;

/// Predefined style utilities
pub mod styles {
    use crate::css_runtime::{Breakpoint, CssBuilder, css_props};
    
    /// Button styles
    pub fn button_primary() -> CssBuilder {
        CssBuilder::new()
            .properties(css_props! {
                "display" => "inline-flex",
                "align-items" => "center",
                "justify-content" => "center",
                "padding" => "0.5rem 1rem",
                "background-color" => "var(--primary)",
                "color" => "white",
                "border" => "none",
                "border-radius" => "0.375rem",
                "font-size" => "1rem",
                "font-weight" => "500",
                "cursor" => "pointer",
                "transition" => "all var(--transition-fast)",
                "outline" => "none"
            })
            .hover(css_props! {
                "background-color" => "var(--primary-dark)",
                "transform" => "translateY(-1px)",
                "box-shadow" => "var(--shadow-md)"
            })
            .focus(css_props! {
                "box-shadow" => "0 0 0 3px rgba(102, 126, 234, 0.3)"
            })
            .active(css_props! {
                "transform" => "translateY(0)",
                "box-shadow" => "var(--shadow-sm)"
            })
            .disabled(css_props! {
                "opacity" => "0.5",
                "cursor" => "not-allowed"
            })
    }
    
    /// Card styles
    pub fn card() -> CssBuilder {
        CssBuilder::new()
            .properties(css_props! {
                "background-color" => "white",
                "border-radius" => "0.5rem",
                "box-shadow" => "var(--shadow-md)",
                "padding" => "1.5rem",
                "transition" => "all var(--transition-normal)"
            })
            .hover(css_props! {
                "box-shadow" => "var(--shadow-lg)",
                "transform" => "translateY(-2px)"
            })
            .media("(prefers-color-scheme: dark)", css_props! {
                "background-color" => "var(--background-dark)",
                "border" => "1px solid var(--border-dark)"
            })
    }
    
    /// Grid container
    pub fn grid(columns: u8) -> CssBuilder {
        CssBuilder::new()
            .properties(css_props! {
                "display" => "grid",
                "gap" => "1rem",
                "grid-template-columns" => format!("repeat({}, minmax(0, 1fr))", columns)
            })
            .breakpoint(Breakpoint::Md, css_props! {
                "grid-template-columns" => format!("repeat({}, minmax(0, 1fr))", columns.min(2))
            })
            .breakpoint(Breakpoint::Sm, css_props! {
                "grid-template-columns" => "1fr"
            })
    }
    
    /// Flex container
    pub fn flex_center() -> CssBuilder {
        CssBuilder::new()
            .properties(css_props! {
                "display" => "flex",
                "align-items" => "center",
                "justify-content" => "center"
            })
    }
    
    /// Text styles
    pub fn heading() -> CssBuilder {
        CssBuilder::new()
            .properties(css_props! {
                "font-size" => "2rem",
                "font-weight" => "700",
                "line-height" => "1.2",
                "margin-bottom" => "1rem",
                "color" => "var(--text)"
            })
            .breakpoint(Breakpoint::Md, css_props! {
                "font-size" => "1.75rem"
            })
            .breakpoint(Breakpoint::Sm, css_props! {
                "font-size" => "1.5rem"
            })
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use super::*;
    use crate::component::Component;
    
    struct TestComponent {
        text: String,
    }
    
    impl Component for TestComponent {
        fn render(&self) -> Element {
            Element::Text(self.text.clone())
        }
    }
    
    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_styled_component() {
        let component = TestComponent {
            text: "Hello".to_string(),
        };
        
        let styled = component.styled(
            CssBuilder::new()
                .property("color", "red")
                .property("font-size", "16px")
        );
        
        // The styled component should wrap the original in a div with classes
        match styled.render() {
            Element::Node { props, children, .. } => {
                assert!(props.class.is_some());
                assert!(props.class.unwrap().starts_with("l9-"));
                assert_eq!(children.len(), 1);
            }
            _ => panic!("Expected Node element"),
        }
    }
}