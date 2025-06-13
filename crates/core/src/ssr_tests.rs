//! SSR Tests

#[cfg(test)]
mod tests {
    use crate::ssr::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use async_trait::async_trait;
    
    // Test component
    struct TestComponent {
        content: String,
    }

    #[async_trait]
    impl SSRComponent for TestComponent {
        fn render_to_string(&self, ctx: &SSRContext) -> String {
            format!(
                "<div class='test' data-route='{}'>{}</div>",
                ctx.route, self.content
            )
        }
        
        async fn get_server_props(&self, _ctx: &SSRContext) -> Result<serde_json::Value, String> {
            Ok(serde_json::json!({
                "content": self.content,
                "server_rendered": true,
            }))
        }
    }
    
    #[tokio::test]
    async fn test_ssr_context_creation() {
        let ctx = SSRContext::new()
            .with_route("/test".to_string())
            .with_state(r#"{"count": 42}"#.to_string())
            .add_meta_tag("<meta name='test' content='value'>".to_string());
            
        assert_eq!(ctx.route, "/test");
        assert_eq!(ctx.initial_state.unwrap(), r#"{"count": 42}"#);
        assert_eq!(ctx.meta_tags.len(), 1);
    }
    
    #[tokio::test]
    async fn test_ssr_context_query_params() {
        let mut params = HashMap::new();
        params.insert("page".to_string(), "2".to_string());
        params.insert("sort".to_string(), "date".to_string());
        
        let ctx = SSRContext::new()
            .with_query_params(params);
            
        assert_eq!(ctx.query_params.get("page").unwrap(), "2");
        assert_eq!(ctx.query_params.get("sort").unwrap(), "date");
    }
    
    #[tokio::test]
    async fn test_ssr_renderer_basic() {
        let ctx = SSRContext::new();
        let mut renderer = SSRRenderer::new();
        
        renderer.add_component(Box::new(TestComponent {
            content: "Hello SSR".to_string(),
        }));
        
        let html = renderer.render(&ctx).await;
        
        assert!(html.contains("Hello SSR"));
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("</html>"));
    }
    
    #[tokio::test]
    async fn test_ssr_renderer_with_state() {
        let ctx = SSRContext::new()
            .with_state(r#"{"user": "test"}"#.to_string());
            
        let renderer = SSRRenderer::new();
        let html = renderer.render(&ctx).await;
        
        assert!(html.contains(r#"window.__INITIAL_STATE__ = {"user": "test"};"#));
    }
    
    #[tokio::test]
    async fn test_ssr_renderer_with_meta_tags() {
        let ctx = SSRContext::new()
            .add_meta_tag(r#"<meta name="description" content="Test page">"#.to_string())
            .add_meta_tag(r#"<meta property="og:title" content="Test">"#.to_string());
            
        let renderer = SSRRenderer::new();
        let html = renderer.render(&ctx).await;
        
        assert!(html.contains(r#"<meta name="description" content="Test page">"#));
        assert!(html.contains(r#"<meta property="og:title" content="Test">"#));
    }
    
    #[tokio::test]
    async fn test_ssr_renderer_hydration() {
        let ctx = SSRContext::new()
            .with_route("/test".to_string());
            
        let renderer = SSRRenderer::new()
            .enable_hydration(true);
            
        let html = renderer.render(&ctx).await;
        
        assert!(html.contains("window.__SSR_CONTEXT__"));
        assert!(html.contains("hydrate_app()"));
    }
    
    #[tokio::test]
    async fn test_ssr_renderer_no_hydration() {
        let ctx = SSRContext::new();
        let renderer = SSRRenderer::new()
            .enable_hydration(false);
            
        let html = renderer.render(&ctx).await;
        
        // Should have neither SSR context nor hydration script when hydration is disabled
        assert!(!html.contains("window.__SSR_CONTEXT__"));
        assert!(!html.contains("hydrate_app()"));
    }
    
    #[tokio::test]
    async fn test_ssr_component_server_props() {
        let component = TestComponent {
            content: "Server content".to_string(),
        };
        
        let ctx = SSRContext::new();
        let props = component.get_server_props(&ctx).await.unwrap();
        
        assert_eq!(props["content"], "Server content");
        assert_eq!(props["server_rendered"], true);
    }
    
    #[tokio::test]
    async fn test_ssr_data_states() {
        // Test loading state
        let loading: SSRData<String> = SSRData::loading();
        assert!(loading.is_loading);
        assert!(loading.data.is_none());
        assert!(loading.error.is_none());
        
        // Test success state
        let success = SSRData::success("Hello".to_string());
        assert!(!success.is_loading);
        assert_eq!(success.data.unwrap(), "Hello");
        assert!(success.error.is_none());
        
        // Test error state
        let error: SSRData<String> = SSRData::error("Failed".to_string());
        assert!(!error.is_loading);
        assert!(error.data.is_none());
        assert_eq!(error.error.unwrap(), "Failed");
    }
    
    #[tokio::test]
    async fn test_ssr_context_serialization() {
        let mut headers = HashMap::new();
        headers.insert("user-agent".to_string(), "test-agent".to_string());
        
        let ctx = SSRContext::new()
            .with_route("/api/test".to_string())
            .with_headers(headers);
            
        // Serialize and deserialize
        let json = serde_json::to_string(&ctx).unwrap();
        let deserialized: SSRContext = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.route, "/api/test");
        // Headers are skipped in serialization
        assert!(deserialized.request_headers.is_empty());
    }
    
    #[tokio::test]
    async fn test_multiple_components() {
        let mut renderer = SSRRenderer::new();
        
        renderer.add_component(Box::new(TestComponent {
            content: "First".to_string(),
        }));
        
        renderer.add_component(Box::new(TestComponent {
            content: "Second".to_string(),
        }));
        
        let ctx = SSRContext::new();
        let html = renderer.render(&ctx).await;
        
        assert!(html.contains("First"));
        assert!(html.contains("Second"));
    }
    
    // Test SSR App implementation
    struct TestSSRApp;
    
    #[cfg(not(target_arch = "wasm32"))]
    #[async_trait]
    impl SSRApp for TestSSRApp {
        fn routes(&self) -> Vec<SSRRoute> {
            vec![
                SSRRoute {
                    path: "/".to_string(),
                    handler: Arc::new(TestHandler),
                },
            ]
        }
    }
    
    struct TestHandler;
    
    #[cfg(not(target_arch = "wasm32"))]
    #[async_trait]
    impl SSRRouteHandler for TestHandler {
        async fn handle(&self, ctx: SSRContext) -> Result<String, String> {
            let mut renderer = SSRRenderer::new();
            renderer.add_component(Box::new(TestComponent {
                content: "Route handler test".to_string(),
            }));
            Ok(renderer.render(&ctx).await)
        }
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_ssr_app_routes() {
        let app = TestSSRApp;
        let routes = app.routes();
        
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].path, "/");
    }
    
    #[tokio::test]
    async fn test_example_ssr_component() {
        let component = ExampleSSRComponent {
            title: "Test Title".to_string(),
        };
        
        let mut params = HashMap::new();
        params.insert("id".to_string(), "123".to_string());
        
        let ctx = SSRContext::new()
            .with_route("/test".to_string())
            .with_query_params(params);
            
        let html = component.render_to_string(&ctx);
        
        assert!(html.contains("Test Title"));
        assert!(html.contains("Route: /test"));
        assert!(html.contains("123"));
    }
}