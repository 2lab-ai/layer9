use layer9_core::middleware_v2::{Context, Middleware, MiddlewareError, MiddlewareStack, Next, Request, Response, RouteParams};
use layer9_core::prelude::*;
use std::collections::HashMap;
use async_trait::async_trait;
use wasm_bindgen::prelude::*;
use web_sys::console;

// Test middleware 1: Adds a header
struct AddHeaderMiddleware {
    header_name: String,
    header_value: String,
}

#[async_trait(?Send)]
impl Middleware for AddHeaderMiddleware {
    async fn handle(&self, ctx: &mut Context, next: Next) -> Result<Response, MiddlewareError> {
        console::log_1(&format!("Middleware 1: Adding header {} = {}", self.header_name, self.header_value).into());
        
        // Add header to request
        ctx.request.headers.insert(self.header_name.clone(), self.header_value.clone());
        
        // Call next middleware
        let mut response = next().await?;
        
        // Also add to response
        response.headers.insert(self.header_name.clone(), self.header_value.clone());
        console::log_1(&format!("Middleware 1: Response status = {}", response.status).into());
        
        Ok(response)
    }
}

// Test middleware 2: Modifies the response body
struct ModifyBodyMiddleware {
    prefix: String,
}

#[async_trait(?Send)]
impl Middleware for ModifyBodyMiddleware {
    async fn handle(&self, _ctx: &mut Context, next: Next) -> Result<Response, MiddlewareError> {
        console::log_1(&format!("Middleware 2: Processing with prefix '{}'", self.prefix).into());
        
        // Call next middleware
        let mut response = next().await?;
        
        // Modify the body
        if let Some(body) = response.body {
            response.body = Some(format!("{}: {}", self.prefix, body));
            console::log_1(&format!("Middleware 2: Modified body to '{}'", response.body.as_ref().unwrap()).into());
        }
        
        Ok(response)
    }
}

// Test middleware 3: Sets the final response
struct FinalResponseMiddleware {
    message: String,
}

#[async_trait(?Send)]
impl Middleware for FinalResponseMiddleware {
    async fn handle(&self, ctx: &mut Context, next: Next) -> Result<Response, MiddlewareError> {
        console::log_1(&format!("Middleware 3: Setting final response to '{}'", self.message).into());
        
        // Set a response in context
        ctx.response = Response::new()
            .with_status(200)
            .with_body(self.message.clone());
        
        // Call next (which should just return the response)
        next().await
    }
}

// Test middleware 4: Error middleware
struct ErrorMiddleware;

#[async_trait(?Send)]
impl Middleware for ErrorMiddleware {
    async fn handle(&self, _ctx: &mut Context, _next: Next) -> Result<Response, MiddlewareError> {
        console::log_1(&"Middleware 4: Throwing error".into());
        Err(MiddlewareError {
            status: 500,
            message: "Test error from middleware".to_string(),
        })
    }
}

#[wasm_bindgen(start)]
pub async fn main() {
    console::log_1(&"Starting middleware chain test...".into());
    
    // Test 1: Basic middleware chaining
    console::log_1(&"\n=== Test 1: Basic Middleware Chaining ===".into());
    {
        let stack = MiddlewareStack::new()
            .use_middleware(AddHeaderMiddleware {
                header_name: "X-Test-Header".to_string(),
                header_value: "test-value".to_string(),
            })
            .use_middleware(ModifyBodyMiddleware {
                prefix: "Modified".to_string(),
            })
            .use_middleware(FinalResponseMiddleware {
                message: "Hello from final middleware".to_string(),
            });
        
        let ctx = Context {
            request: Request {
                method: Method::GET,
                url: "/test".to_string(),
                headers: HashMap::new(),
                body: None,
                user: None,
            },
            response: Response::new(),
            state: HashMap::new(),
            params: RouteParams {
                params: HashMap::new(),
                query: HashMap::new(),
            },
        };
        
        match stack.run(ctx).await {
            Ok(response) => {
                console::log_1(&format!("Success! Status: {}, Body: {:?}", response.status, response.body).into());
                console::log_1(&format!("Headers: {:?}", response.headers).into());
                
                // Verify the chain worked correctly
                assert_eq!(response.status, 200);
                assert_eq!(response.body, Some("Modified: Hello from final middleware".to_string()));
                assert!(response.headers.contains_key("X-Test-Header"));
            }
            Err(e) => {
                console::log_1(&format!("Error: {} (status: {})", e.message, e.status).into());
            }
        }
    }
    
    // Test 2: Error handling
    console::log_1(&"\n=== Test 2: Error Handling ===".into());
    {
        let stack = MiddlewareStack::new()
            .use_middleware(AddHeaderMiddleware {
                header_name: "X-Test-Header".to_string(),
                header_value: "test-value".to_string(),
            })
            .use_middleware(ErrorMiddleware)
            .use_middleware(ModifyBodyMiddleware {
                prefix: "Should not reach here".to_string(),
            });
        
        let ctx = Context {
            request: Request {
                method: Method::GET,
                url: "/test".to_string(),
                headers: HashMap::new(),
                body: None,
                user: None,
            },
            response: Response::new(),
            state: HashMap::new(),
            params: RouteParams {
                params: HashMap::new(),
                query: HashMap::new(),
            },
        };
        
        match stack.run(ctx).await {
            Ok(_) => {
                console::log_1(&"Unexpected success!".into());
            }
            Err(e) => {
                console::log_1(&format!("Expected error: {} (status: {})", e.message, e.status).into());
                assert_eq!(e.status, 500);
                assert_eq!(e.message, "Test error from middleware");
            }
        }
    }
    
    // Test 3: Empty middleware stack
    console::log_1(&"\n=== Test 3: Empty Middleware Stack ===".into());
    {
        let stack = MiddlewareStack::new();
        
        let ctx = Context {
            request: Request {
                method: Method::GET,
                url: "/test".to_string(),
                headers: HashMap::new(),
                body: None,
                user: None,
            },
            response: Response::new().with_status(204).with_body("Default response"),
            state: HashMap::new(),
            params: RouteParams {
                params: HashMap::new(),
                query: HashMap::new(),
            },
        };
        
        match stack.run(ctx).await {
            Ok(response) => {
                console::log_1(&format!("Success! Status: {}, Body: {:?}", response.status, response.body).into());
                // Should return the default response from context
                assert_eq!(response.status, 204);
                assert_eq!(response.body, Some("Default response".to_string()));
            }
            Err(e) => {
                console::log_1(&format!("Error: {} (status: {})", e.message, e.status).into());
            }
        }
    }
    
    // Test 4: Test middleware order
    console::log_1(&"\n=== Test 4: Middleware Order Test ===".into());
    {
        let stack = MiddlewareStack::new()
            .use_middleware(AddHeaderMiddleware {
                header_name: "X-First".to_string(),
                header_value: "1".to_string(),
            })
            .use_middleware(AddHeaderMiddleware {
                header_name: "X-Second".to_string(),
                header_value: "2".to_string(),
            })
            .use_middleware(AddHeaderMiddleware {
                header_name: "X-Third".to_string(),
                header_value: "3".to_string(),
            });
        
        let ctx = Context {
            request: Request {
                method: Method::GET,
                url: "/test".to_string(),
                headers: HashMap::new(),
                body: None,
                user: None,
            },
            response: Response::new().with_body("Test"),
            state: HashMap::new(),
            params: RouteParams {
                params: HashMap::new(),
                query: HashMap::new(),
            },
        };
        
        match stack.run(ctx).await {
            Ok(response) => {
                console::log_1(&format!("Success! Headers: {:?}", response.headers).into());
                // All headers should be added
                assert_eq!(response.headers.get("X-First"), Some(&"1".to_string()));
                assert_eq!(response.headers.get("X-Second"), Some(&"2".to_string()));
                assert_eq!(response.headers.get("X-Third"), Some(&"3".to_string()));
            }
            Err(e) => {
                console::log_1(&format!("Error: {} (status: {})", e.message, e.status).into());
            }
        }
    }
    
    console::log_1(&"\nAll tests completed!".into());
}