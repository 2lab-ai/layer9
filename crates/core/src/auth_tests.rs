//! Comprehensive tests for authentication module

#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod tests {
    use crate::auth::*;
    use wasm_bindgen_test::*;
    
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_auth_context_creation() {
        let context = AuthContext::new();
        assert!(!context.is_authenticated());
        assert!(context.user.is_none());
        assert!(context.token.is_none());
        assert!(context.permissions.is_empty());
    }

    #[wasm_bindgen_test]
    fn test_auth_context_login_logout() {
        let mut context = AuthContext::new();
        
        let user = User {
            id: "1".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            roles: vec!["user".to_string()],
        };
        
        context.login(user.clone(), "test-token".to_string());
        assert!(context.is_authenticated());
        assert_eq!(context.user.as_ref().unwrap().username, "testuser");
        assert_eq!(context.token.as_ref().unwrap(), "test-token");
        
        context.logout();
        assert!(!context.is_authenticated());
        assert!(context.user.is_none());
        assert!(context.token.is_none());
    }

    #[wasm_bindgen_test]
    fn test_auth_context_permissions() {
        let mut context = AuthContext::new();
        context.permissions = vec!["read".to_string(), "write".to_string()];
        
        assert!(context.has_permission("read"));
        assert!(context.has_permission("write"));
        assert!(!context.has_permission("delete"));
    }

    #[wasm_bindgen_test]
    fn test_mock_auth_provider() {
        let provider = MockAuthProvider;
        
        // Test authentication
        let result = provider.authenticate("testuser", "password");
        assert!(result.is_ok());
        let (user, token) = result.unwrap();
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "testuser@example.com");
        assert_eq!(token, "mock-token-123");
        
        // Test token validation
        let result = provider.validate_token("any-token");
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.username, "testuser");
        
        // Test token refresh
        let result = provider.refresh_token("old-token");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "refreshed-token-456");
    }

    #[wasm_bindgen_test]
    fn test_jwt_auth_provider_user_management() {
        let mut provider = JwtAuthProvider::new("test-secret".to_string());
        
        // Add users
        let id1 = provider.add_user(
            "alice".to_string(),
            "password123".to_string(),
            "alice@example.com".to_string(),
            vec!["user".to_string()],
        );
        
        let id2 = provider.add_user(
            "bob".to_string(),
            "secret456".to_string(),
            "bob@example.com".to_string(),
            vec!["admin".to_string()],
        );
        
        assert_eq!(id1, "user-1");
        assert_eq!(id2, "user-2");
        
        // Test authentication with correct password
        let result = provider.authenticate("alice", "password123");
        assert!(result.is_ok());
        let (user, token) = result.unwrap();
        assert_eq!(user.username, "alice");
        assert_eq!(user.email, "alice@example.com");
        assert!(!token.is_empty());
        
        // Test authentication with wrong password
        let result = provider.authenticate("alice", "wrongpassword");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid username or password");
        
        // Test authentication with non-existent user
        let result = provider.authenticate("charlie", "password");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid username or password");
    }

    #[wasm_bindgen_test]
    fn test_jwt_auth_provider_token_validation() {
        let mut provider = JwtAuthProvider::new("test-secret".to_string());
        
        provider.add_user(
            "testuser".to_string(),
            "password".to_string(),
            "test@example.com".to_string(),
            vec!["user".to_string()],
        );
        
        // Get a valid token
        let (_, token) = provider.authenticate("testuser", "password").unwrap();
        
        // Validate the token
        let result = provider.validate_token(&token);
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        
        // Test with invalid token
        let result = provider.validate_token("invalid-token");
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    fn test_jwt_auth_provider_token_refresh() {
        let mut provider = JwtAuthProvider::new("test-secret".to_string());
        
        provider.add_user(
            "testuser".to_string(),
            "password".to_string(),
            "test@example.com".to_string(),
            vec!["user".to_string()],
        );
        
        // Get a valid token
        let (_, token) = provider.authenticate("testuser", "password").unwrap();
        
        // Refresh the token
        let result = provider.refresh_token(&token);
        assert!(result.is_ok());
        let new_token = result.unwrap();
        assert!(!new_token.is_empty());
        assert_ne!(new_token, token); // Should be different
        
        // Validate the new token
        let result = provider.validate_token(&new_token);
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    fn test_jwt_auth_provider_permissions() {
        let provider = JwtAuthProvider::new("test-secret".to_string());
        
        // Test admin permissions
        let admin_perms = provider.get_permissions_for_roles(&["admin".to_string()]);
        assert!(admin_perms.contains(&"read".to_string()));
        assert!(admin_perms.contains(&"write".to_string()));
        assert!(admin_perms.contains(&"delete".to_string()));
        assert!(admin_perms.contains(&"admin".to_string()));
        
        // Test user permissions
        let user_perms = provider.get_permissions_for_roles(&["user".to_string()]);
        assert!(user_perms.contains(&"read".to_string()));
        assert!(user_perms.contains(&"write".to_string()));
        assert!(!user_perms.contains(&"delete".to_string()));
        assert!(!user_perms.contains(&"admin".to_string()));
        
        // Test guest permissions
        let guest_perms = provider.get_permissions_for_roles(&["guest".to_string()]);
        assert!(guest_perms.contains(&"read".to_string()));
        assert!(!guest_perms.contains(&"write".to_string()));
        
        // Test multiple roles
        let multi_perms = provider.get_permissions_for_roles(&["user".to_string(), "guest".to_string()]);
        assert!(multi_perms.contains(&"read".to_string()));
        assert!(multi_perms.contains(&"write".to_string()));
    }

    #[wasm_bindgen_test]
    async fn test_auth_service_with_mock_provider() {
        let mut service = AuthService::with_mock_provider();
        
        assert!(!service.is_authenticated());
        
        // Test login
        let result = service.login("testuser", "password").await;
        assert!(result.is_ok());
        assert!(service.is_authenticated());
        
        let user = service.get_current_user().unwrap();
        assert_eq!(user.username, "testuser");
        
        // Test logout
        service.logout();
        assert!(!service.is_authenticated());
        assert!(service.get_current_user().is_none());
    }

    #[wasm_bindgen_test]
    async fn test_auth_service_with_jwt_provider() {
        // Create provider with user first
        let mut provider = JwtAuthProvider::new("test-secret".to_string());
        provider.add_user(
            "alice".to_string(),
            "password123".to_string(),
            "alice@example.com".to_string(),
            vec!["user".to_string()],
        );
        
        let mut service = AuthService::new(Box::new(provider));
        
        // Test login with correct credentials
        let result = service.login("alice", "password123").await;
        assert!(result.is_ok());
        assert!(service.is_authenticated());
        
        let context = service.get_context();
        assert!(context.has_permission("read"));
        assert!(context.has_permission("write"));
        assert!(!context.has_permission("admin"));
        
        // Test session refresh
        let result = service.refresh_session().await;
        assert!(result.is_ok());
        
        // Test logout
        service.logout();
        assert!(!service.is_authenticated());
    }

    #[wasm_bindgen_test]
    fn test_protected_component() {
        let protected = Protected::new()
            .with_permission("admin".to_string())
            .with_redirect("/login".to_string());
        
        // Test with unauthenticated context
        let context = AuthContext::new();
        assert!(!protected.check_access(&context));
        
        // Test with authenticated user without permission
        let mut context = AuthContext::new();
        let user = User {
            id: "1".to_string(),
            username: "user".to_string(),
            email: "user@example.com".to_string(),
            roles: vec!["user".to_string()],
        };
        context.login(user, "token".to_string());
        context.permissions = vec!["read".to_string(), "write".to_string()];
        assert!(!protected.check_access(&context));
        
        // Test with authenticated user with permission
        context.permissions.push("admin".to_string());
        assert!(protected.check_access(&context));
        
        // Test without required permission (any authenticated user)
        let protected_any = Protected::new();
        assert!(protected_any.check_access(&context));
    }

    #[wasm_bindgen_test]
    fn test_use_auth_hook() {
        let context = use_auth();
        assert!(!context.is_authenticated());
        assert!(context.user.is_none());
        assert!(context.token.is_none());
    }
}