//! CSS Runtime System - Dynamic CSS generation and injection
//!
//! This module provides a comprehensive CSS-in-Rust system with:
//! - Runtime CSS generation and injection
//! - Pseudo-class support (:hover, :focus, :active, etc.)
//! - Media query support with responsive breakpoints
//! - CSS variables for dynamic theming
//! - Scoped styles and CSS modules
//! - Animation and transition utilities

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::HtmlStyleElement;

/// Global stylesheet manager
pub(crate) static STYLESHEET_MANAGER: Lazy<Mutex<StyleSheetManager>> = Lazy::new(|| {
    Mutex::new(StyleSheetManager::new())
});

/// Manages all CSS rules and their injection into the DOM
pub struct StyleSheetManager {
    /// Map of class names to their CSS rules
    rules: HashMap<String, String>,
    /// Whether the style element has been initialized
    initialized: bool,
    /// Counter for generating unique class names
    class_counter: u32,
}

impl StyleSheetManager {
    fn new() -> Self {
        Self {
            rules: HashMap::new(),
            initialized: false,
            class_counter: 0,
        }
    }

    /// Initialize the style element in the DOM
    pub fn init(&mut self) {
        if self.initialized {
            return;
        }

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let head = document.head().unwrap();

        let style = document.create_element("style").unwrap();
        let style_element = style.dyn_into::<HtmlStyleElement>().unwrap();
        style_element.set_type("text/css");
        style_element.set_id("layer9-runtime-styles");

        head.append_child(&style_element).unwrap();
        self.initialized = true;
    }

    /// Generate a unique class name
    fn generate_class_name(&mut self, prefix: Option<&str>) -> String {
        self.class_counter += 1;
        match prefix {
            Some(p) => format!("l9-{}-{}", p, self.class_counter),
            None => format!("l9-{}", self.class_counter),
        }
    }

    /// Add a CSS rule and return the generated class name
    pub fn add_rule(&mut self, css_rule: &CssRule) -> String {
        self.init();

        let class_name = self.generate_class_name(css_rule.prefix.as_deref());
        let css_text = css_rule.to_css(&class_name);

        self.rules.insert(class_name.clone(), css_text);
        self.update_styles();

        class_name
    }

    /// Update the style element with all current rules
    fn update_styles(&self) {
        if self.initialized {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            
            if let Some(element) = document.get_element_by_id("layer9-runtime-styles") {
                if let Ok(style_element) = element.dyn_into::<HtmlStyleElement>() {
                    let css_content = self
                        .rules
                        .values()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join("\n");

                    style_element.set_inner_html(&css_content);
                }
            }
        }
    }

    /// Remove a CSS rule by class name
    pub fn remove_rule(&mut self, class_name: &str) {
        self.rules.remove(class_name);
        self.update_styles();
    }

    /// Clear all rules
    pub fn clear(&mut self) {
        self.rules.clear();
        self.update_styles();
    }
}

/// Represents a CSS rule with all its properties
#[derive(Clone, Debug, Default)]
pub struct CssRule {
    /// Optional prefix for the generated class name
    pub prefix: Option<String>,
    /// Base CSS properties
    pub properties: HashMap<String, String>,
    /// Pseudo-class styles (e.g., :hover, :focus)
    pub pseudo_classes: HashMap<String, HashMap<String, String>>,
    /// Media query styles
    pub media_queries: HashMap<String, HashMap<String, String>>,
    /// CSS variables used in this rule
    pub variables: HashMap<String, String>,
    /// Keyframe animations
    pub animations: Vec<Animation>,
}

impl CssRule {
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert the rule to CSS text
    fn to_css(&self, class_name: &str) -> String {
        let mut css_parts = Vec::new();

        // Base rule
        if !self.properties.is_empty() {
            let props = self
                .properties
                .iter()
                .map(|(k, v)| format!("  {}: {};", k, v))
                .collect::<Vec<_>>()
                .join("\n");
            css_parts.push(format!(".{} {{\n{}\n}}", class_name, props));
        }

        // Pseudo-classes
        for (pseudo, props) in &self.pseudo_classes {
            if !props.is_empty() {
                let prop_text = props
                    .iter()
                    .map(|(k, v)| format!("  {}: {};", k, v))
                    .collect::<Vec<_>>()
                    .join("\n");
                css_parts.push(format!(".{}:{} {{\n{}\n}}", class_name, pseudo, prop_text));
            }
        }

        // Media queries
        for (query, props) in &self.media_queries {
            if !props.is_empty() {
                let prop_text = props
                    .iter()
                    .map(|(k, v)| format!("    {}: {};", k, v))
                    .collect::<Vec<_>>()
                    .join("\n");
                css_parts.push(format!(
                    "@media {} {{\n  .{} {{\n{}\n  }}\n}}",
                    query, class_name, prop_text
                ));
            }
        }

        // Animations
        for animation in &self.animations {
            css_parts.push(animation.to_css());
        }

        css_parts.join("\n\n")
    }
}

/// CSS animation definition
#[derive(Clone, Debug)]
pub struct Animation {
    pub name: String,
    pub keyframes: Vec<(String, HashMap<String, String>)>,
}

impl Animation {
    pub fn new(name: String) -> Self {
        Self {
            name,
            keyframes: Vec::new(),
        }
    }

    pub fn keyframe(mut self, percentage: &str, props: HashMap<String, String>) -> Self {
        self.keyframes.push((percentage.to_string(), props));
        self
    }

    fn to_css(&self) -> String {
        let keyframes = self
            .keyframes
            .iter()
            .map(|(pct, props)| {
                let prop_text = props
                    .iter()
                    .map(|(k, v)| format!("    {}: {};", k, v))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("  {} {{\n{}\n  }}", pct, prop_text)
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!("@keyframes {} {{\n{}\n}}", self.name, keyframes)
    }
}

/// Fluent API for building CSS rules
pub struct CssBuilder {
    rule: CssRule,
}

impl Default for CssBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CssBuilder {
    pub fn new() -> Self {
        Self {
            rule: CssRule::new(),
        }
    }

    /// Set a prefix for the generated class name
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.rule.prefix = Some(prefix.to_string());
        self
    }

    /// Add a CSS property
    pub fn property(mut self, key: &str, value: &str) -> Self {
        self.rule.properties.insert(key.to_string(), value.to_string());
        self
    }

    /// Add multiple CSS properties
    pub fn properties<I>(mut self, props: I) -> Self
    where
        I: IntoIterator<Item = (String, String)>,
    {
        self.rule.properties.extend(props);
        self
    }

    /// Add hover state styles
    pub fn hover(mut self, props: HashMap<String, String>) -> Self {
        self.rule.pseudo_classes.insert("hover".to_string(), props);
        self
    }

    /// Add focus state styles
    pub fn focus(mut self, props: HashMap<String, String>) -> Self {
        self.rule.pseudo_classes.insert("focus".to_string(), props);
        self
    }

    /// Add active state styles
    pub fn active(mut self, props: HashMap<String, String>) -> Self {
        self.rule.pseudo_classes.insert("active".to_string(), props);
        self
    }

    /// Add disabled state styles
    pub fn disabled(mut self, props: HashMap<String, String>) -> Self {
        self.rule.pseudo_classes.insert("disabled".to_string(), props);
        self
    }

    /// Add custom pseudo-class styles
    pub fn pseudo(mut self, pseudo: &str, props: HashMap<String, String>) -> Self {
        self.rule.pseudo_classes.insert(pseudo.to_string(), props);
        self
    }

    /// Add media query styles
    pub fn media(mut self, query: &str, props: HashMap<String, String>) -> Self {
        self.rule.media_queries.insert(query.to_string(), props);
        self
    }

    /// Add responsive breakpoint styles
    pub fn breakpoint(mut self, breakpoint: Breakpoint, props: HashMap<String, String>) -> Self {
        let query = breakpoint.to_media_query();
        self.rule.media_queries.insert(query, props);
        self
    }

    /// Add CSS variable
    pub fn variable(mut self, name: &str, value: &str) -> Self {
        self.rule.variables.insert(name.to_string(), value.to_string());
        self
    }

    /// Add animation
    pub fn animation(mut self, animation: Animation) -> Self {
        self.rule.animations.push(animation);
        self
    }

    /// Build and inject the CSS rule, returning the class name
    pub fn build(self) -> String {
        let mut manager = STYLESHEET_MANAGER.lock();
        manager.add_rule(&self.rule)
    }
}

/// Predefined responsive breakpoints
#[derive(Clone, Copy, Debug)]
pub enum Breakpoint {
    /// 640px
    Sm,
    /// 768px
    Md,
    /// 1024px
    Lg,
    /// 1280px
    Xl,
    /// 1536px
    Xxl,
    /// Custom breakpoint
    Custom(u32),
}

impl Breakpoint {
    fn to_media_query(self) -> String {
        match self {
            Self::Sm => "(min-width: 640px)".to_string(),
            Self::Md => "(min-width: 768px)".to_string(),
            Self::Lg => "(min-width: 1024px)".to_string(),
            Self::Xl => "(min-width: 1280px)".to_string(),
            Self::Xxl => "(min-width: 1536px)".to_string(),
            Self::Custom(px) => format!("(min-width: {}px)", px),
        }
    }
}

/// CSS variable system for dynamic theming
pub struct CssVariables {
    variables: HashMap<String, String>,
}

impl Default for CssVariables {
    fn default() -> Self {
        Self::new()
    }
}

impl CssVariables {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Set a CSS variable
    pub fn set(&mut self, name: &str, value: &str) {
        self.variables.insert(
            format!("--{}", name),
            value.to_string(),
        );
        self.apply();
    }

    /// Get a CSS variable value
    pub fn get(&self, name: &str) -> Option<&String> {
        self.variables.get(&format!("--{}", name))
    }

    /// Apply all variables to the document root
    fn apply(&self) {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        if let Some(root) = document.document_element() {
            if let Ok(html_element) = root.dyn_into::<web_sys::HtmlElement>() {
                let style = html_element.style();
                for (name, value) in &self.variables {
                    style.set_property(name, value).ok();
                }
            }
        }
    }

    /// Create a theme with predefined variables
    pub fn theme() -> Self {
        let mut vars = Self::new();
        
        // Colors
        vars.set("primary", "#667eea");
        vars.set("primary-dark", "#5a67d8");
        vars.set("secondary", "#48bb78");
        vars.set("background", "#ffffff");
        vars.set("background-dark", "#1a202c");
        vars.set("text", "#2d3748");
        vars.set("text-dark", "#e2e8f0");
        vars.set("border", "#e2e8f0");
        vars.set("border-dark", "#2d3748");
        
        // Spacing
        vars.set("spacing-xs", "0.25rem");
        vars.set("spacing-sm", "0.5rem");
        vars.set("spacing-md", "1rem");
        vars.set("spacing-lg", "1.5rem");
        vars.set("spacing-xl", "2rem");
        
        // Typography
        vars.set("font-sans", "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif");
        vars.set("font-mono", "Consolas, Monaco, 'Andale Mono', monospace");
        
        // Shadows
        vars.set("shadow-sm", "0 1px 2px 0 rgba(0, 0, 0, 0.05)");
        vars.set("shadow-md", "0 4px 6px -1px rgba(0, 0, 0, 0.1)");
        vars.set("shadow-lg", "0 10px 15px -3px rgba(0, 0, 0, 0.1)");
        
        // Transitions
        vars.set("transition-fast", "150ms ease-in-out");
        vars.set("transition-normal", "250ms ease-in-out");
        vars.set("transition-slow", "350ms ease-in-out");
        
        vars
    }
}

/// Inject global CSS variables and base styles
pub fn inject_global_styles() {
    // Initialize the stylesheet manager
    let mut manager = STYLESHEET_MANAGER.lock();
    manager.init();

    // Apply theme variables
    let theme = CssVariables::theme();
    drop(theme); // Variables are applied automatically

    // Add global reset and base styles
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let head = document.head().unwrap();

    let style = document.create_element("style").unwrap();
    style.set_id("layer9-global-styles");
    style.set_inner_html(
        r#"
        /* CSS Reset */
        *, *::before, *::after {
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }

        /* Base styles */
        body {
            font-family: var(--font-sans);
            background-color: var(--background);
            color: var(--text);
            line-height: 1.5;
            transition: background-color var(--transition-normal), color var(--transition-normal);
        }

        /* Dark mode support */
        @media (prefers-color-scheme: dark) {
            body {
                background-color: var(--background-dark);
                color: var(--text-dark);
            }
        }

        /* Focus visible for accessibility */
        :focus-visible {
            outline: 2px solid var(--primary);
            outline-offset: 2px;
        }

        /* Smooth scrolling */
        html {
            scroll-behavior: smooth;
        }

        /* Common utility classes */
        .l9-transition {
            transition: all var(--transition-normal);
        }

        .l9-shadow {
            box-shadow: var(--shadow-md);
        }

        .l9-rounded {
            border-radius: 0.375rem;
        }

        /* Animation utilities */
        @keyframes l9-fade-in {
            from { opacity: 0; }
            to { opacity: 1; }
        }

        @keyframes l9-slide-in {
            from { transform: translateY(1rem); opacity: 0; }
            to { transform: translateY(0); opacity: 1; }
        }

        @keyframes l9-scale-in {
            from { transform: scale(0.95); opacity: 0; }
            to { transform: scale(1); opacity: 1; }
        }

        .l9-animate-fade-in {
            animation: l9-fade-in var(--transition-normal) ease-out;
        }

        .l9-animate-slide-in {
            animation: l9-slide-in var(--transition-normal) ease-out;
        }

        .l9-animate-scale-in {
            animation: l9-scale-in var(--transition-normal) ease-out;
        }
        "#,
    );

    head.append_child(&style).unwrap();
}

/// Helper macro for creating CSS property maps
#[macro_export]
macro_rules! css_props {
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut map = std::collections::HashMap::new();
        $(
            map.insert($key.to_string(), $value.to_string());
        )*
        map
    }};
}

pub use css_props;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_rule_generation() {
        let rule = CssRule {
            prefix: Some("test".to_string()),
            properties: css_props! {
                "display" => "flex",
                "color" => "red"
            },
            pseudo_classes: HashMap::new(),
            media_queries: HashMap::new(),
            variables: HashMap::new(),
            animations: Vec::new(),
        };

        let css = rule.to_css("test-class");
        assert!(css.contains(".test-class"));
        assert!(css.contains("display: flex"));
        assert!(css.contains("color: red"));
    }

    #[test]
    fn test_css_builder() {
        let builder = CssBuilder::new()
            .property("display", "block")
            .property("padding", "1rem")
            .hover(css_props! {
                "background-color" => "#f0f0f0"
            });

        assert_eq!(builder.rule.properties.get("display").unwrap(), "block");
        assert_eq!(builder.rule.properties.get("padding").unwrap(), "1rem");
        assert!(builder.rule.pseudo_classes.contains_key("hover"));
    }

    #[test]
    fn test_breakpoint_media_queries() {
        assert_eq!(Breakpoint::Sm.to_media_query(), "(min-width: 640px)");
        assert_eq!(Breakpoint::Md.to_media_query(), "(min-width: 768px)");
        assert_eq!(Breakpoint::Custom(500).to_media_query(), "(min-width: 500px)");
    }

    #[test]
    fn test_animation_generation() {
        let animation = Animation::new("test-anim".to_string())
            .keyframe("0%", css_props! { "opacity" => "0" })
            .keyframe("100%", css_props! { "opacity" => "1" });

        let css = animation.to_css();
        assert!(css.contains("@keyframes test-anim"));
        assert!(css.contains("0%"));
        assert!(css.contains("100%"));
        assert!(css.contains("opacity: 0"));
        assert!(css.contains("opacity: 1"));
    }
}