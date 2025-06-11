//! Styling System - CSS-in-Rust (L3)

use crate::component::{Element, Props};
use crate::prelude::Component;
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Style class builder - Tailwind-like utility classes in Rust
#[derive(Default)]
pub struct StyleBuilder {
    classes: Vec<String>,
    custom: HashMap<String, String>,
}

impl StyleBuilder {
    pub fn new() -> Self {
        StyleBuilder::default()
    }

    // Layout
    pub fn flex(mut self) -> Self {
        self.classes.push("display: flex".to_string());
        self
    }

    pub fn grid(mut self) -> Self {
        self.classes.push("display: grid".to_string());
        self
    }

    pub fn hidden(mut self) -> Self {
        self.classes.push("display: none".to_string());
        self
    }

    // Flexbox
    pub fn items_center(mut self) -> Self {
        self.classes.push("align-items: center".to_string());
        self
    }

    pub fn justify_center(mut self) -> Self {
        self.classes.push("justify-content: center".to_string());
        self
    }

    pub fn justify_between(mut self) -> Self {
        self.classes.push("justify-content: space-between".to_string());
        self
    }

    pub fn gap(mut self, size: u8) -> Self {
        self.custom
            .insert("gap".to_string(), format!("{}rem", size as f32 * 0.25));
        self
    }

    // Spacing
    pub fn p(mut self, size: u8) -> Self {
        self.custom
            .insert("padding".to_string(), format!("{}rem", size as f32 * 0.25));
        self
    }

    pub fn px(mut self, size: u8) -> Self {
        self.custom.insert(
            "padding-left".to_string(),
            format!("{}rem", size as f32 * 0.25),
        );
        self.custom.insert(
            "padding-right".to_string(),
            format!("{}rem", size as f32 * 0.25),
        );
        self
    }

    pub fn py(mut self, size: u8) -> Self {
        self.custom.insert(
            "padding-top".to_string(),
            format!("{}rem", size as f32 * 0.25),
        );
        self.custom.insert(
            "padding-bottom".to_string(),
            format!("{}rem", size as f32 * 0.25),
        );
        self
    }

    pub fn m(mut self, size: u8) -> Self {
        self.custom
            .insert("margin".to_string(), format!("{}rem", size as f32 * 0.25));
        self
    }

    pub fn mx_auto(mut self) -> Self {
        self.custom
            .insert("margin-left".to_string(), "auto".to_string());
        self.custom
            .insert("margin-right".to_string(), "auto".to_string());
        self
    }

    // Typography
    pub fn text_sm(mut self) -> Self {
        self.classes.push("font-size: 0.875rem".to_string());
        self.classes.push("line-height: 1.25rem".to_string());
        self
    }

    pub fn text_lg(mut self) -> Self {
        self.classes.push("font-size: 1.125rem".to_string());
        self.classes.push("line-height: 1.75rem".to_string());
        self
    }

    pub fn text_xl(mut self) -> Self {
        self.classes.push("font-size: 1.25rem".to_string());
        self.classes.push("line-height: 1.75rem".to_string());
        self
    }

    pub fn font_bold(mut self) -> Self {
        self.classes.push("font-weight: 700".to_string());
        self
    }

    pub fn text_center(mut self) -> Self {
        self.classes.push("text-align: center".to_string());
        self
    }

    // Colors
    pub fn bg_black(mut self) -> Self {
        self.classes.push("background-color: #000000".to_string());
        self
    }

    pub fn bg_white(mut self) -> Self {
        self.classes.push("background-color: #ffffff".to_string());
        self
    }

    pub fn text_white(mut self) -> Self {
        self.classes.push("color: #ffffff".to_string());
        self
    }

    pub fn text_gray_500(mut self) -> Self {
        self.classes.push("color: #6b7280".to_string());
        self
    }

    // Borders
    pub fn border(mut self) -> Self {
        self.classes.push("border-width: 1px".to_string());
        self.classes.push("border-style: solid".to_string());
        self
    }

    pub fn border_gray_200(mut self) -> Self {
        self.classes.push("border-color: #e5e7eb".to_string());
        self
    }

    pub fn rounded(mut self) -> Self {
        self.classes.push("border-radius: 0.25rem".to_string());
        self
    }

    pub fn rounded_lg(mut self) -> Self {
        self.classes.push("border-radius: 0.5rem".to_string());
        self
    }

    // Effects
    pub fn shadow(mut self) -> Self {
        self.classes
            .push("box-shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.1)".to_string());
        self
    }

    pub fn transition(mut self) -> Self {
        self.classes.push("transition-property: all".to_string());
        self.classes.push("transition-duration: 150ms".to_string());
        self
    }

    pub fn hover_bg_gray_100(mut self) -> Self {
        // Note: Hover states need special handling in WASM
        self.classes.push("hover:background-color: #f3f4f6".to_string());
        self
    }

    // Dark mode
    pub fn dark_bg_gray_800(mut self) -> Self {
        self.classes
            .push("@media (prefers-color-scheme: dark) { background-color: #1f2937 }".to_string());
        self
    }

    pub fn dark_text_gray_100(mut self) -> Self {
        self.classes
            .push("@media (prefers-color-scheme: dark) { color: #f3f4f6 }".to_string());
        self
    }

    // Responsive
    pub fn md_flex(mut self) -> Self {
        self.classes
            .push("@media (min-width: 768px) { display: flex }".to_string());
        self
    }

    pub fn lg_grid_cols(mut self, cols: u8) -> Self {
        self.classes.push(format!(
            "@media (min-width: 1024px) {{ grid-template-columns: repeat({}, minmax(0, 1fr)) }}",
            cols
        ));
        self
    }

    // Build final style string
    pub fn build(self) -> String {
        let mut styles = vec![];

        // Add static classes
        for class in self.classes {
            styles.push(class.to_string());
        }

        // Add custom properties
        for (prop, value) in self.custom {
            styles.push(format!("{}: {}", prop, value));
        }

        styles.join("; ")
    }
}

/// Styled component wrapper
pub struct Styled<T: Component> {
    component: T,
    styles: String,
}

impl<T: Component> Styled<T> {
    pub fn new(component: T, styles: StyleBuilder) -> Self {
        Styled {
            component,
            styles: styles.build(),
        }
    }
}

impl<T: Component> Component for Styled<T> {
    fn render(&self) -> Element {
        let inner = self.component.render();

        // Wrap with styled div
        Element::Node {
            tag: "div".to_string(),
            props: Props {
                attributes: vec![("style".to_string(), self.styles.clone())],
                ..Default::default()
            },
            children: vec![inner],
        }
    }
}

/// Theme system
pub struct Theme {
    pub colors: ThemeColors,
    pub spacing: ThemeSpacing,
    pub typography: ThemeTypography,
}

pub struct ThemeColors {
    pub primary: &'static str,
    pub secondary: &'static str,
    pub background: &'static str,
    pub foreground: &'static str,
    pub muted: &'static str,
    pub border: &'static str,
}

pub struct ThemeSpacing {
    pub unit: f32, // 0.25rem
}

pub struct ThemeTypography {
    pub font_family: &'static str,
    pub font_size_base: &'static str,
}

#[allow(dead_code)]
static DEFAULT_THEME: Lazy<Theme> = Lazy::new(|| Theme {
    colors: ThemeColors {
        primary: "#667eea",
        secondary: "#764ba2",
        background: "#000000",
        foreground: "#ffffff",
        muted: "#6b7280",
        border: "#e5e7eb",
    },
    spacing: ThemeSpacing { unit: 0.25 },
    typography: ThemeTypography {
        font_family: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
        font_size_base: "16px",
    },
});

/// Global styles injection
pub fn inject_global_styles() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let head = document.head().unwrap();

    let style = document.create_element("style").unwrap();
    style.set_inner_html(
        r#"
        * {
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background-color: #000;
            color: #fff;
            line-height: 1.5;
        }
        
        /* Dark mode support */
        @media (prefers-color-scheme: dark) {
            :root {
                --background: #000;
                --foreground: #fff;
            }
        }
        
        /* Animations */
        @keyframes pulse {
            0%, 100% { opacity: 0.5; }
            50% { opacity: 1; }
        }
        
        .animate-pulse {
            animation: pulse 1.5s ease-in-out infinite;
        }
        
        /* Reality glitch effect */
        @keyframes glitch {
            0% { transform: translate(0); }
            20% { transform: translate(-2px, 2px); }
            40% { transform: translate(-2px, -2px); }
            60% { transform: translate(2px, 2px); }
            80% { transform: translate(2px, -2px); }
            100% { transform: translate(0); }
        }
        
        .reality-glitch {
            animation: glitch 0.3s ease-in-out;
        }
    "#,
    );

    head.append_child(&style).unwrap();
}

/// Style macro for inline styles
#[macro_export]
macro_rules! style {
    ($($method:ident $(($($arg:expr),*))?),* $(,)?) => {{
        let mut builder = $crate::styles::StyleBuilder::new();
        $(
            builder = builder.$method$(($($arg),*))?;
        )*
        builder
    }};
}

pub use style;

// Usage example:
// let button_style = style![
//     flex,
//     items_center,
//     gap(2),
//     px(4),
//     py(2),
//     bg_black,
//     text_white,
//     border,
//     rounded_lg,
//     transition,
//     hover_bg_gray_100,
// ];
