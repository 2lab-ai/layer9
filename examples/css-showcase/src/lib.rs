//! CSS Showcase - Demonstrates all CSS-in-Rust features
//!
//! This example showcases:
//! - Runtime CSS generation
//! - Hover, focus, and active states
//! - Media queries and responsive design
//! - CSS variables and theming
//! - Animations and transitions
//! - Scoped styles

use layer9_core::prelude::*;
use wasm_bindgen::prelude::*;
use std::rc::Rc;

/// Theme switcher component
struct ThemeSwitcher {
    is_dark: State<bool>,
}

impl Component for ThemeSwitcher {
    fn render(&self) -> Element {
        let is_dark = self.is_dark.get();
        let toggle_theme = {
            let state = self.is_dark.clone();
            move || {
                let new_theme = !state.get();
                state.set(new_theme);
                
                // Update CSS variables
                let mut vars = CssVariables::new();
                if new_theme {
                    vars.set("background", "#1a202c");
                    vars.set("text", "#e2e8f0");
                } else {
                    vars.set("background", "#ffffff");
                    vars.set("text", "#2d3748");
                }
            }
        };

        // Create a styled button with hover effects
        let button_style = CssBuilder::new()
            .properties(css_props! {
                "position" => "fixed",
                "top" => "1rem",
                "right" => "1rem",
                "padding" => "0.75rem 1.5rem",
                "background-color" => if is_dark { "#4a5568" } else { "#e2e8f0" },
                "color" => if is_dark { "#e2e8f0" } else { "#2d3748" },
                "border" => "none",
                "border-radius" => "9999px",
                "cursor" => "pointer",
                "transition" => "all 0.3s ease",
                "font-weight" => "600",
                "z-index" => "1000"
            })
            .hover(css_props! {
                "transform" => "scale(1.05)",
                "box-shadow" => "0 4px 6px rgba(0, 0, 0, 0.1)"
            })
            .active(css_props! {
                "transform" => "scale(0.95)"
            });

        Element::Node {
            tag: "button".to_string(),
            props: Props {
                class: Some(button_style.build()),
                on_click: Some(Rc::new(toggle_theme)),
                ..Default::default()
            },
            children: vec![Element::Text(
                if is_dark { "☀️ Light Mode" } else { "🌙 Dark Mode" }.to_string()
            )],
        }
    }
}

/// Hero section with animated gradient
struct HeroSection;

impl Component for HeroSection {
    fn render(&self) -> Element {
        // Create gradient animation
        let gradient_animation = Animation::new("gradient-shift".to_string())
            .keyframe("0%", css_props! {
                "background-position" => "0% 50%"
            })
            .keyframe("50%", css_props! {
                "background-position" => "100% 50%"
            })
            .keyframe("100%", css_props! {
                "background-position" => "0% 50%"
            });

        let hero_style = CssBuilder::new()
            .properties(css_props! {
                "min-height" => "60vh",
                "display" => "flex",
                "flex-direction" => "column",
                "align-items" => "center",
                "justify-content" => "center",
                "background" => "linear-gradient(135deg, #667eea 0%, #764ba2 50%, #f093fb 100%)",
                "background-size" => "200% 200%",
                "animation" => "gradient-shift 8s ease infinite",
                "color" => "white",
                "text-align" => "center",
                "padding" => "2rem"
            })
            .animation(gradient_animation);

        let title_style = CssBuilder::new()
            .properties(css_props! {
                "font-size" => "3.5rem",
                "font-weight" => "800",
                "margin-bottom" => "1rem",
                "text-shadow" => "2px 2px 4px rgba(0, 0, 0, 0.3)"
            })
            .breakpoint(Breakpoint::Md, css_props! {
                "font-size" => "2.5rem"
            })
            .breakpoint(Breakpoint::Sm, css_props! {
                "font-size" => "2rem"
            });

        let subtitle_style = CssBuilder::new()
            .properties(css_props! {
                "font-size" => "1.5rem",
                "opacity" => "0.9",
                "max-width" => "600px"
            });

        Element::Node {
            tag: "section".to_string(),
            props: Props {
                class: Some(hero_style.build()),
                ..Default::default()
            },
            children: vec![
                Element::Node {
                    tag: "h1".to_string(),
                    props: Props {
                        class: Some(title_style.build()),
                        ..Default::default()
                    },
                    children: vec![Element::Text("CSS-in-Rust Showcase".to_string())],
                },
                Element::Node {
                    tag: "p".to_string(),
                    props: Props {
                        class: Some(subtitle_style.build()),
                        ..Default::default()
                    },
                    children: vec![Element::Text(
                        "Experience the power of runtime CSS generation with hover states, \
                         media queries, animations, and more!".to_string()
                    )],
                },
            ],
        }
    }
}

/// Interactive card grid
struct CardGrid;

impl Component for CardGrid {
    fn render(&self) -> Element {
        let grid_style = CssBuilder::new()
            .properties(css_props! {
                "display" => "grid",
                "grid-template-columns" => "repeat(auto-fit, minmax(300px, 1fr))",
                "gap" => "2rem",
                "padding" => "3rem",
                "background-color" => "var(--background)"
            });

        let cards = vec![
            ("🎨", "Dynamic Styling", "Runtime CSS generation with type-safe builders"),
            ("🖱️", "Interactive States", "Hover, focus, and active pseudo-classes"),
            ("📱", "Responsive Design", "Media queries and breakpoint utilities"),
            ("🎭", "Theming", "CSS variables for dynamic theme switching"),
            ("✨", "Animations", "Smooth transitions and keyframe animations"),
            ("🔒", "Scoped Styles", "Automatically generated unique class names"),
        ];

        Element::Node {
            tag: "section".to_string(),
            props: Props {
                class: Some(grid_style.build()),
                ..Default::default()
            },
            children: cards.into_iter().map(|(icon, title, desc)| {
                Card { icon, title, description: desc }.render()
            }).collect(),
        }
    }
}

/// Individual card component
struct Card {
    icon: &'static str,
    title: &'static str,
    description: &'static str,
}

impl Component for Card {
    fn render(&self) -> Element {
        let card_style = CssBuilder::new()
            .prefix("card")
            .properties(css_props! {
                "background-color" => "white",
                "border-radius" => "1rem",
                "padding" => "2rem",
                "box-shadow" => "0 4px 6px rgba(0, 0, 0, 0.1)",
                "transition" => "all 0.3s ease",
                "cursor" => "pointer",
                "position" => "relative",
                "overflow" => "hidden"
            })
            .hover(css_props! {
                "transform" => "translateY(-8px) scale(1.02)",
                "box-shadow" => "0 12px 20px rgba(0, 0, 0, 0.15)"
            })
            .pseudo("before", css_props! {
                "content" => "''",
                "position" => "absolute",
                "top" => "0",
                "left" => "0",
                "width" => "100%",
                "height" => "4px",
                "background" => "linear-gradient(90deg, #667eea, #764ba2)",
                "transform" => "scaleX(0)",
                "transform-origin" => "left",
                "transition" => "transform 0.3s ease"
            })
            .pseudo("hover:before", css_props! {
                "transform" => "scaleX(1)"
            })
            .media("(prefers-color-scheme: dark)", css_props! {
                "background-color" => "#2d3748",
                "color" => "#e2e8f0"
            });

        let icon_style = CssBuilder::new()
            .properties(css_props! {
                "font-size" => "3rem",
                "margin-bottom" => "1rem",
                "display" => "block"
            });

        let title_style = CssBuilder::new()
            .properties(css_props! {
                "font-size" => "1.5rem",
                "font-weight" => "700",
                "margin-bottom" => "0.5rem",
                "color" => "var(--text)"
            });

        let desc_style = CssBuilder::new()
            .properties(css_props! {
                "color" => "#718096",
                "line-height" => "1.6"
            })
            .media("(prefers-color-scheme: dark)", css_props! {
                "color" => "#a0aec0"
            });

        Element::Node {
            tag: "div".to_string(),
            props: Props {
                class: Some(card_style.build()),
                ..Default::default()
            },
            children: vec![
                Element::Node {
                    tag: "span".to_string(),
                    props: Props {
                        class: Some(icon_style.build()),
                        ..Default::default()
                    },
                    children: vec![Element::Text(self.icon.to_string())],
                },
                Element::Node {
                    tag: "h3".to_string(),
                    props: Props {
                        class: Some(title_style.build()),
                        ..Default::default()
                    },
                    children: vec![Element::Text(self.title.to_string())],
                },
                Element::Node {
                    tag: "p".to_string(),
                    props: Props {
                        class: Some(desc_style.build()),
                        ..Default::default()
                    },
                    children: vec![Element::Text(self.description.to_string())],
                },
            ],
        }
    }
}

/// Button showcase section
struct ButtonShowcase;

impl Component for ButtonShowcase {
    fn render(&self) -> Element {
        let section_style = CssBuilder::new()
            .properties(css_props! {
                "padding" => "3rem",
                "background-color" => "#f7fafc",
                "text-align" => "center"
            })
            .media("(prefers-color-scheme: dark)", css_props! {
                "background-color" => "#1a202c"
            });

        let button_container = CssBuilder::new()
            .properties(css_props! {
                "display" => "flex",
                "gap" => "1rem",
                "justify-content" => "center",
                "flex-wrap" => "wrap",
                "margin-top" => "2rem"
            });

        Element::Node {
            tag: "section".to_string(),
            props: Props {
                class: Some(section_style.build()),
                ..Default::default()
            },
            children: vec![
                Element::Node {
                    tag: "h2".to_string(),
                    props: Props {
                        class: Some(styles::heading().build()),
                        ..Default::default()
                    },
                    children: vec![Element::Text("Interactive Buttons".to_string())],
                },
                Element::Node {
                    tag: "div".to_string(),
                    props: Props {
                        class: Some(button_container.build()),
                        ..Default::default()
                    },
                    children: vec![
                        Element::Node {
                            tag: "button".to_string(),
                            props: Props {
                                class: Some(styles::button_primary().build()),
                                ..Default::default()
                            },
                            children: vec![Element::Text("Primary Button".to_string())],
                        },
                        SecondaryButton.render(),
                        GhostButton.render(),
                        GradientButton.render(),
                    ],
                },
            ],
        }
    }
}

/// Secondary button style
struct SecondaryButton;

impl Component for SecondaryButton {
    fn render(&self) -> Element {
        let style = CssBuilder::new()
            .properties(css_props! {
                "padding" => "0.5rem 1rem",
                "background-color" => "transparent",
                "color" => "var(--primary)",
                "border" => "2px solid var(--primary)",
                "border-radius" => "0.375rem",
                "font-size" => "1rem",
                "font-weight" => "500",
                "cursor" => "pointer",
                "transition" => "all 0.2s ease"
            })
            .hover(css_props! {
                "background-color" => "var(--primary)",
                "color" => "white",
                "transform" => "translateY(-2px)"
            })
            .focus(css_props! {
                "outline" => "none",
                "box-shadow" => "0 0 0 3px rgba(102, 126, 234, 0.3)"
            });

        Element::Node {
            tag: "button".to_string(),
            props: Props {
                class: Some(style.build()),
                ..Default::default()
            },
            children: vec![Element::Text("Secondary Button".to_string())],
        }
    }
}

/// Ghost button style
struct GhostButton;

impl Component for GhostButton {
    fn render(&self) -> Element {
        let style = CssBuilder::new()
            .properties(css_props! {
                "padding" => "0.5rem 1rem",
                "background-color" => "transparent",
                "color" => "#718096",
                "border" => "none",
                "font-size" => "1rem",
                "font-weight" => "500",
                "cursor" => "pointer",
                "position" => "relative",
                "transition" => "color 0.2s ease"
            })
            .hover(css_props! {
                "color" => "var(--primary)"
            })
            .pseudo("after", css_props! {
                "content" => "''",
                "position" => "absolute",
                "bottom" => "0",
                "left" => "50%",
                "width" => "0",
                "height" => "2px",
                "background-color" => "var(--primary)",
                "transform" => "translateX(-50%)",
                "transition" => "width 0.3s ease"
            })
            .pseudo("hover:after", css_props! {
                "width" => "100%"
            });

        Element::Node {
            tag: "button".to_string(),
            props: Props {
                class: Some(style.build()),
                ..Default::default()
            },
            children: vec![Element::Text("Ghost Button".to_string())],
        }
    }
}

/// Gradient button with animation
struct GradientButton;

impl Component for GradientButton {
    fn render(&self) -> Element {
        let pulse_animation = Animation::new("pulse".to_string())
            .keyframe("0%", css_props! {
                "box-shadow" => "0 0 0 0 rgba(102, 126, 234, 0.7)"
            })
            .keyframe("70%", css_props! {
                "box-shadow" => "0 0 0 10px rgba(102, 126, 234, 0)"
            })
            .keyframe("100%", css_props! {
                "box-shadow" => "0 0 0 0 rgba(102, 126, 234, 0)"
            });

        let style = CssBuilder::new()
            .properties(css_props! {
                "padding" => "0.5rem 1rem",
                "background" => "linear-gradient(135deg, #667eea, #764ba2)",
                "color" => "white",
                "border" => "none",
                "border-radius" => "0.375rem",
                "font-size" => "1rem",
                "font-weight" => "500",
                "cursor" => "pointer",
                "position" => "relative",
                "overflow" => "hidden",
                "transition" => "transform 0.2s ease"
            })
            .hover(css_props! {
                "transform" => "translateY(-2px)",
                "animation" => "pulse 1.5s infinite"
            })
            .animation(pulse_animation);

        Element::Node {
            tag: "button".to_string(),
            props: Props {
                class: Some(style.build()),
                ..Default::default()
            },
            children: vec![Element::Text("Gradient Button".to_string())],
        }
    }
}

/// Main app component
struct CssShowcaseApp {
    theme_switcher: ThemeSwitcher,
}

impl CssShowcaseApp {
    fn new() -> Self {
        Self {
            theme_switcher: ThemeSwitcher {
                is_dark: use_state(|| false),
            },
        }
    }
}

impl Component for CssShowcaseApp {
    fn render(&self) -> Element {
        let app_style = CssBuilder::new()
            .properties(css_props! {
                "min-height" => "100vh",
                "background-color" => "var(--background)",
                "color" => "var(--text)",
                "transition" => "background-color 0.3s ease, color 0.3s ease"
            });

        Element::Node {
            tag: "div".to_string(),
            props: Props {
                class: Some(app_style.build()),
                ..Default::default()
            },
            children: vec![
                self.theme_switcher.render(),
                HeroSection.render(),
                CardGrid.render(),
                ButtonShowcase.render(),
                ResponsiveGrid.render(),
            ],
        }
    }
}

/// Responsive grid showcase
struct ResponsiveGrid;

impl Component for ResponsiveGrid {
    fn render(&self) -> Element {
        let section_style = CssBuilder::new()
            .properties(css_props! {
                "padding" => "3rem",
                "background-color" => "var(--background)"
            });

        let grid_style = CssBuilder::new()
            .properties(css_props! {
                "display" => "grid",
                "gap" => "1rem",
                "grid-template-columns" => "1fr"
            })
            .breakpoint(Breakpoint::Sm, css_props! {
                "grid-template-columns" => "repeat(2, 1fr)"
            })
            .breakpoint(Breakpoint::Md, css_props! {
                "grid-template-columns" => "repeat(3, 1fr)"
            })
            .breakpoint(Breakpoint::Lg, css_props! {
                "grid-template-columns" => "repeat(4, 1fr)"
            });

        let item_style_class = CssBuilder::new()
            .properties(css_props! {
                "background" => "linear-gradient(135deg, #f093fb, #f5576c)",
                "padding" => "3rem 1rem",
                "border-radius" => "0.5rem",
                "text-align" => "center",
                "color" => "white",
                "font-weight" => "600",
                "transition" => "transform 0.2s ease"
            })
            .hover(css_props! {
                "transform" => "scale(1.05)"
            })
            .build();

        Element::Node {
            tag: "section".to_string(),
            props: Props {
                class: Some(section_style.build()),
                ..Default::default()
            },
            children: vec![
                Element::Node {
                    tag: "h2".to_string(),
                    props: Props {
                        class: Some(styles::heading().build()),
                        ..Default::default()
                    },
                    children: vec![Element::Text("Responsive Grid".to_string())],
                },
                Element::Node {
                    tag: "p".to_string(),
                    props: Props {
                        attributes: vec![("style".to_string(), "text-align: center; margin-bottom: 2rem; color: #718096;".to_string())],
                        ..Default::default()
                    },
                    children: vec![Element::Text("Resize your window to see the grid adapt!".to_string())],
                },
                Element::Node {
                    tag: "div".to_string(),
                    props: Props {
                        class: Some(grid_style.build()),
                        ..Default::default()
                    },
                    children: (1..=8).map(|i| {
                        Element::Node {
                            tag: "div".to_string(),
                            props: Props {
                                class: Some(item_style_class.clone()),
                                ..Default::default()
                            },
                            children: vec![Element::Text(format!("Item {}", i))],
                        }
                    }).collect(),
                },
            ],
        }
    }
}

/// Entry point
#[wasm_bindgen(start)]
pub fn main() {
    // Initialize CSS runtime and inject global styles
    inject_css_runtime();
    
    // Create and mount the app
    let app = CssShowcaseApp::new();
    mount(Box::new(app), "app");
}