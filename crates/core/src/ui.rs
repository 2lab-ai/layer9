//! UI Component Library - L5 (shadcn/ui in Rust)

use crate::component::{Component, Element, Props};
use crate::styles::{style, StyleBuilder};
use crate::view;

/// Button component
pub struct Button {
    text: String,
    variant: ButtonVariant,
    on_click: Option<Box<dyn Fn()>>,
}

#[derive(Clone, Copy)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Outline,
    Ghost,
    Destructive,
}

impl Button {
    pub fn new(text: impl Into<String>) -> Self {
        Button {
            text: text.into(),
            variant: ButtonVariant::Primary,
            on_click: None,
        }
    }
    
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }
    
    pub fn on_click(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl Component for Button {
    fn render(&self) -> Element {
        let base_style = style![
            px(4),
            py(2),
            rounded,
            font_bold,
            transition,
        ];
        
        let variant_style = match self.variant {
            ButtonVariant::Primary => style![bg_black, text_white, hover_bg_gray_100],
            ButtonVariant::Secondary => style![bg_white, text_gray_500, border, border_gray_200],
            ButtonVariant::Outline => style![border, border_gray_200, hover_bg_gray_100],
            ButtonVariant::Ghost => style![hover_bg_gray_100],
            ButtonVariant::Destructive => style![text_white], // bg_red_500
        };
        
        let style_str = format!("{};{}", base_style.build(), variant_style.build());
        
        Element::Node {
            tag: "button".to_string(),
            props: Props {
                attributes: vec![("style".to_string(), style_str)],
                on_click: self.on_click.as_ref().map(|f| f.clone()),
                ..Default::default()
            },
            children: vec![Element::Text(self.text.clone())],
        }
    }
}

/// Card component
pub struct Card {
    children: Vec<Element>,
    class: Option<String>,
}

impl Card {
    pub fn new() -> Self {
        Card {
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

impl Component for Card {
    fn render(&self) -> Element {
        let style = style![
            bg_white,
            dark_bg_gray_800,
            rounded_lg,
            shadow,
            p(6),
            border,
            border_gray_200,
        ];
        
        Element::Node {
            tag: "div".to_string(),
            props: Props {
                class: self.class.clone(),
                attributes: vec![("style".to_string(), style.build())],
                ..Default::default()
            },
            children: self.children.clone(),
        }
    }
}

/// Input component
pub struct Input {
    placeholder: Option<String>,
    value: String,
    on_change: Option<Box<dyn Fn(String)>>,
}

impl Input {
    pub fn new() -> Self {
        Input {
            placeholder: None,
            value: String::new(),
            on_change: None,
        }
    }
    
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }
    
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }
    
    pub fn on_change(mut self, handler: impl Fn(String) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }
}

impl Component for Input {
    fn render(&self) -> Element {
        let style = style![
            px(3),
            py(2),
            border,
            border_gray_200,
            rounded,
            bg_white,
            dark_bg_gray_800,
            text_gray_500,
            dark_text_gray_100,
        ];
        
        let mut attrs = vec![
            ("style".to_string(), style.build()),
            ("value".to_string(), self.value.clone()),
        ];
        
        if let Some(placeholder) = &self.placeholder {
            attrs.push(("placeholder".to_string(), placeholder.clone()));
        }
        
        Element::Node {
            tag: "input".to_string(),
            props: Props {
                attributes: attrs,
                ..Default::default()
            },
            children: vec![],
        }
    }
}

/// Badge component
pub struct Badge {
    text: String,
    variant: BadgeVariant,
}

#[derive(Clone, Copy)]
pub enum BadgeVariant {
    Default,
    Secondary,
    Outline,
    Destructive,
}

impl Badge {
    pub fn new(text: impl Into<String>) -> Self {
        Badge {
            text: text.into(),
            variant: BadgeVariant::Default,
        }
    }
    
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }
}

impl Component for Badge {
    fn render(&self) -> Element {
        let base_style = style![
            px(2),
            py(1),
            text_sm,
            font_bold,
            rounded,
        ];
        
        let variant_style = match self.variant {
            BadgeVariant::Default => style![bg_black, text_white],
            BadgeVariant::Secondary => style![bg_white, text_gray_500],
            BadgeVariant::Outline => style![border, border_gray_200],
            BadgeVariant::Destructive => style![text_white], // bg_red_500
        };
        
        let style_str = format!("{};{}", base_style.build(), variant_style.build());
        
        Element::Node {
            tag: "span".to_string(),
            props: Props {
                attributes: vec![("style".to_string(), style_str)],
                ..Default::default()
            },
            children: vec![Element::Text(self.text.clone())],
        }
    }
}

/// Progress component
pub struct Progress {
    value: f32, // 0.0 to 100.0
    class: Option<String>,
}

impl Progress {
    pub fn new(value: f32) -> Self {
        Progress {
            value: value.clamp(0.0, 100.0),
            class: None,
        }
    }
    
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = Some(class.into());
        self
    }
}

impl Component for Progress {
    fn render(&self) -> Element {
        let container_style = style![
            bg_white,
            dark_bg_gray_800,
            rounded,
            shadow,
        ];
        
        let bar_style = format!(
            "width: {}%; height: 8px; background-color: #667eea; border-radius: 0.25rem; transition: width 0.3s ease",
            self.value
        );
        
        Element::Node {
            tag: "div".to_string(),
            props: Props {
                class: self.class.clone(),
                attributes: vec![
                    ("style".to_string(), container_style.build()),
                    ("role".to_string(), "progressbar".to_string()),
                    ("aria-valuenow".to_string(), self.value.to_string()),
                    ("aria-valuemin".to_string(), "0".to_string()),
                    ("aria-valuemax".to_string(), "100".to_string()),
                ],
                ..Default::default()
            },
            children: vec![
                Element::Node {
                    tag: "div".to_string(),
                    props: Props {
                        attributes: vec![("style".to_string(), bar_style)],
                        ..Default::default()
                    },
                    children: vec![],
                }
            ],
        }
    }
}

/// Avatar component
pub struct Avatar {
    src: Option<String>,
    alt: String,
    fallback: String,
}

impl Avatar {
    pub fn new() -> Self {
        Avatar {
            src: None,
            alt: String::new(),
            fallback: String::new(),
        }
    }
    
    pub fn src(mut self, src: impl Into<String>) -> Self {
        self.src = Some(src.into());
        self
    }
    
    pub fn alt(mut self, alt: impl Into<String>) -> Self {
        self.alt = alt.into();
        self
    }
    
    pub fn fallback(mut self, fallback: impl Into<String>) -> Self {
        self.fallback = fallback.into();
        self
    }
}

impl Component for Avatar {
    fn render(&self) -> Element {
        let style = style![
            rounded,
            shadow,
        ];
        
        if let Some(src) = &self.src {
            Element::Node {
                tag: "img".to_string(),
                props: Props {
                    attributes: vec![
                        ("src".to_string(), src.clone()),
                        ("alt".to_string(), self.alt.clone()),
                        ("style".to_string(), format!("{};width:40px;height:40px;border-radius:50%", style.build())),
                    ],
                    ..Default::default()
                },
                children: vec![],
            }
        } else {
            // Fallback to initials
            let fallback_style = style![
                flex,
                items_center,
                justify_center,
                bg_black,
                text_white,
                font_bold,
            ];
            
            Element::Node {
                tag: "div".to_string(),
                props: Props {
                    attributes: vec![
                        ("style".to_string(), format!("{};{};width:40px;height:40px;border-radius:50%", 
                            style.build(), fallback_style.build())),
                    ],
                    ..Default::default()
                },
                children: vec![Element::Text(self.fallback.clone())],
            }
        }
    }
}

/// Tabs component
pub struct Tabs {
    tabs: Vec<Tab>,
    active: usize,
}

pub struct Tab {
    label: String,
    content: Element,
}

impl Tabs {
    pub fn new() -> Self {
        Tabs {
            tabs: vec![],
            active: 0,
        }
    }
    
    pub fn add_tab(mut self, label: impl Into<String>, content: Element) -> Self {
        self.tabs.push(Tab {
            label: label.into(),
            content,
        });
        self
    }
    
    pub fn active(mut self, index: usize) -> Self {
        self.active = index;
        self
    }
}

impl Component for Tabs {
    fn render(&self) -> Element {
        let tab_list_style = style![
            flex,
            gap(2),
            border,
            border_gray_200,
            p(1),
            rounded_lg,
        ];
        
        let mut tab_buttons = vec![];
        for (i, tab) in self.tabs.iter().enumerate() {
            let is_active = i == self.active;
            let tab_style = if is_active {
                style![px(4), py(2), bg_black, text_white, rounded]
            } else {
                style![px(4), py(2), hover_bg_gray_100, rounded]
            };
            
            tab_buttons.push(Element::Node {
                tag: "button".to_string(),
                props: Props {
                    attributes: vec![("style".to_string(), tab_style.build())],
                    ..Default::default()
                },
                children: vec![Element::Text(tab.label.clone())],
            });
        }
        
        let content = if self.active < self.tabs.len() {
            self.tabs[self.active].content.clone()
        } else {
            Element::Text("No content".to_string())
        };
        
        view! {
            <div>
                <div style={tab_list_style.build()}>
                    {Element::Node {
                        tag: "div".to_string(),
                        props: Props::default(),
                        children: tab_buttons,
                    }}
                </div>
                <div class="tab-content">
                    {content}
                </div>
            </div>
        }
    }
}