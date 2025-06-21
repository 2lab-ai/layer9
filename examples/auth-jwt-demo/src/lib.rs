//! JWT Authentication Demo
//! 
//! This example demonstrates the new JWT-based authentication system

use layer9_core::auth::{AuthService, JwtAuthProvider};
use layer9_core::config::ConfigBuilder;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen(start)]
pub async fn main() {
    console_error_panic_hook::set_once();
    
    console::log_1(&"=== Layer9 JWT Authentication Demo ===".into());
    
    // Example 1: Using default configuration (JWT with default secret)
    console::log_1(&"\n1. Default Configuration:".into());
    {
        let _auth_service = AuthService::default();
        console::log_1(&"Created AuthService with default configuration".into());
        console::log_1(&"This uses JwtAuthProvider with configured or default secret".into());
    }
    
    // Example 2: Using custom JWT secret
    console::log_1(&"\n2. Custom JWT Secret:".into());
    {
        
        // Create a JWT provider and add a test user
        let mut jwt_provider = JwtAuthProvider::new("my-custom-secret-key".to_string());
        jwt_provider.add_user(
            "testuser".to_string(),
            "password123".to_string(),
            "test@example.com".to_string(),
            vec!["user".to_string()]
        );
        
        // Now create auth service with this provider
        let mut auth_service = AuthService::new(Box::new(jwt_provider));
        
        // Try to login
        match auth_service.login("testuser", "password123").await {
            Ok(_) => {
                console::log_1(&"✓ Login successful!".into());
                if let Some(user) = auth_service.get_current_user() {
                    console::log_1(&format!("  User: {} ({})", user.username, user.email).into());
                    console::log_1(&format!("  Roles: {:?}", user.roles).into());
                }
            }
            Err(e) => {
                console::log_1(&format!("✗ Login failed: {}", e).into());
            }
        }
        
        // Try wrong password
        auth_service.logout();
        match auth_service.login("testuser", "wrongpassword").await {
            Ok(_) => {
                console::log_1(&"Unexpected: Login should have failed!".into());
            }
            Err(e) => {
                console::log_1(&format!("✓ Login correctly failed with wrong password: {}", e).into());
            }
        }
    }
    
    // Example 3: Using configuration builder
    console::log_1(&"\n3. Configuration Builder:".into());
    {
        let config = ConfigBuilder::new()
            .jwt_secret("config-builder-secret".to_string())
            .use_mock_auth(false)
            .build();
        
        let _auth_service = AuthService::with_config(&config);
        console::log_1(&"Created AuthService with custom configuration".into());
        console::log_1(&format!("  JWT Secret: {}", config.get_jwt_secret()).into());
        console::log_1(&format!("  Use Mock Auth: {}", config.should_use_mock_auth()).into());
    }
    
    // Example 4: Using mock authentication for testing
    console::log_1(&"\n4. Mock Authentication (Testing):".into());
    {
        let config = ConfigBuilder::new()
            .use_mock_auth(true)
            .build();
        
        let mut auth_service = AuthService::with_config(&config);
        
        // Mock auth accepts any credentials
        match auth_service.login("anyuser", "anypassword").await {
            Ok(_) => {
                console::log_1(&"✓ Mock login successful (accepts any credentials)".into());
                if let Some(user) = auth_service.get_current_user() {
                    console::log_1(&format!("  Mock User: {} ({})", user.username, user.email).into());
                }
            }
            Err(e) => {
                console::log_1(&format!("✗ Mock login failed: {}", e).into());
            }
        }
    }
    
    // Example 5: Token validation and refresh
    console::log_1(&"\n5. Token Validation and Refresh:".into());
    {
        let mut jwt_provider = JwtAuthProvider::new("token-test-secret".to_string());
        jwt_provider.add_user(
            "tokenuser".to_string(),
            "tokenpass".to_string(),
            "token@example.com".to_string(),
            vec!["admin".to_string()]
        );
        
        let mut auth_service = AuthService::new(Box::new(jwt_provider));
        
        // Login to get a token
        match auth_service.login("tokenuser", "tokenpass").await {
            Ok(_) => {
                console::log_1(&"✓ Login successful, got token".into());
                
                // Get the token
                if let Some(token) = &auth_service.get_context().token {
                    console::log_1(&format!("  Token (truncated): {}...", &token[..20]).into());
                    
                    // Validate token
                    match auth_service.validate_token(token) {
                        Ok(user) => {
                            console::log_1(&format!("✓ Token valid for user: {}", user.username).into());
                        }
                        Err(e) => {
                            console::log_1(&format!("✗ Token validation failed: {}", e).into());
                        }
                    }
                    
                    // Refresh token
                    match auth_service.refresh_session().await {
                        Ok(_) => {
                            console::log_1(&"✓ Token refreshed successfully".into());
                        }
                        Err(e) => {
                            console::log_1(&format!("✗ Token refresh failed: {}", e).into());
                        }
                    }
                }
            }
            Err(e) => {
                console::log_1(&format!("✗ Login failed: {}", e).into());
            }
        }
    }
    
    console::log_1(&"\n=== Demo Complete ===".into());
}