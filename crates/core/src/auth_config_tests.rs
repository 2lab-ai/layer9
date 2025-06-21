//! Tests for authentication configuration

#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod tests {
    use crate::auth::*;
    use crate::config::{Config, ConfigBuilder};
    use wasm_bindgen_test::*;
    
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_auth_service_default_uses_jwt() {
        // Clear any existing config
        let _ = Config::get_or_init();
        
        // Create default auth service
        let auth_service = AuthService::default();
        
        // The service should be created successfully
        assert!(!auth_service.is_authenticated());
    }

    #[wasm_bindgen_test]
    async fn test_auth_service_with_config_mock() {
        let config = ConfigBuilder::new()
            .use_mock_auth(true)
            .build();
        
        let auth_service = AuthService::with_config(&config);
        
        // Mock provider should allow easy authentication
        let mut auth_service_mut = auth_service;
        let result = auth_service_mut.login("testuser", "anypassword").await;
        assert!(result.is_ok());
        assert!(auth_service_mut.is_authenticated());
    }

    #[wasm_bindgen_test]
    async fn test_auth_service_with_config_jwt() {
        let config = ConfigBuilder::new()
            .jwt_secret("test-secret-for-testing-purposes-only".to_string())
            .use_mock_auth(false)
            .build();
        
        let auth_service = AuthService::with_config(&config);
        
        // JWT provider requires valid credentials
        let mut auth_service_mut = auth_service;
        let result = auth_service_mut.login("nonexistent", "wrongpassword").await;
        assert!(result.is_err());
        assert!(!auth_service_mut.is_authenticated());
    }

    #[wasm_bindgen_test]
    async fn test_jwt_provider_with_custom_secret() {
        let custom_secret = "my-custom-jwt-secret-for-testing";
        let mut auth_service = AuthService::with_jwt_provider(custom_secret.to_string());
        
        // Service should be created but not authenticated
        assert!(!auth_service.is_authenticated());
        
        // Try to login with non-existent user
        let result = auth_service.login("testuser", "password").await;
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    fn test_config_jwt_secret_fallback() {
        let config = Config::new();
        
        // Should return default secret with warning
        let secret = config.get_jwt_secret();
        assert!(!secret.is_empty());
        assert_eq!(secret, config.default_jwt_secret);
    }

    #[wasm_bindgen_test]
    fn test_config_with_jwt_secret() {
        let config = ConfigBuilder::new()
            .jwt_secret("my-production-secret".to_string())
            .build();
        
        let secret = config.get_jwt_secret();
        assert_eq!(secret, "my-production-secret");
    }
}