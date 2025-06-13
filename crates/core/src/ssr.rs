//! Server-Side Rendering support for Layer9

use crate::prelude::*;
use std::collections::HashMap;

/// SSR Context for rendering components server-side
#[derive(Debug, Clone)]
pub struct SSRContext {
    pub props: HashMap<String, String>,
    pub initial_state: Option<String>,
    pub meta_tags: Vec<String>,
}

impl SSRContext {
    pub fn new() -> Self {
        Self {
            props: HashMap::new(),
            initial_state: None,
            meta_tags: Vec::new(),
        }
    }
    
    pub fn with_props(mut self, props: HashMap<String, String>) -> Self {
        self.props = props;
        self
    }
    
    pub fn with_state(mut self, state: String) -> Self {
        self.initial_state = Some(state);
        self
    }
    
    pub fn add_meta_tag(mut self, tag: String) -> Self {
        self.meta_tags.push(tag);
        self
    }
}

/// Trait for SSR-capable components
pub trait SSRComponent {
    fn render_to_string(&self, ctx: &SSRContext) -> String;
    fn get_data_requirements(&self) -> Vec<String> {
        Vec::new()
    }
}

/// SSR Renderer
pub struct SSRRenderer {
    components: Vec<Box<dyn SSRComponent>>,
    template: String,
}

impl SSRRenderer {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            template: Self::default_template(),
        }
    }
    
    pub fn with_template(mut self, template: String) -> Self {
        self.template = template;
        self
    }
    
    pub fn add_component(&mut self, component: Box<dyn SSRComponent>) {
        self.components.push(component);
    }
    
    pub fn render(&self, ctx: &SSRContext) -> String {
        let mut body = String::new();
        
        for component in &self.components {
            body.push_str(&component.render_to_string(ctx));
        }
        
        let mut html = self.template.clone();
        html = html.replace("{{content}}", &body);
        
        // Add meta tags
        let meta_tags = ctx.meta_tags.join("\n    ");
        html = html.replace("{{meta}}", &meta_tags);
        
        // Add initial state
        if let Some(state) = &ctx.initial_state {
            let state_script = format!(
                r#"<script>window.__INITIAL_STATE__ = {};</script>"#,
                state
            );
            html = html.replace("{{state}}", &state_script);
        } else {
            html = html.replace("{{state}}", "");
        }
        
        html
    }
    
    fn default_template() -> String {
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Layer9 SSR</title>
    {{meta}}
</head>
<body>
    <div id="app">{{content}}</div>
    {{state}}
    <script type="module" src="/pkg/layer9_app.js"></script>
</body>
</html>"#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ssr_context() {
        let ctx = SSRContext::new()
            .with_state(r#"{"counter": 0}"#.to_string())
            .add_meta_tag(r#"<meta name="description" content="Test">"#.to_string());
        
        assert!(ctx.initial_state.is_some());
        assert_eq!(ctx.meta_tags.len(), 1);
    }
}
