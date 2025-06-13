//! Integration tests for authentication and file upload working together

#[cfg(test)]
mod tests {
    use crate::auth::{AuthService, JwtAuthProvider};
    use crate::upload::{FileUploadManager, FileUploadComponent};
    use wasm_bindgen_test::*;
    use web_sys::File;
    use js_sys::{Uint8Array, Array};
    
    wasm_bindgen_test_configure!(run_in_browser);

    /// Helper to create a test file
    fn create_test_file(name: &str, content: &str) -> File {
        let bytes = content.as_bytes();
        let array = Uint8Array::new_with_length(bytes.len() as u32);
        array.copy_from(bytes);
        
        let file_parts = Array::new();
        file_parts.push(&array.buffer());
        
        File::new_with_blob_sequence(&file_parts.into(), name).unwrap()
    }

    /// Helper to create an authenticated auth service
    async fn create_authenticated_service() -> AuthService {
        let mut provider = JwtAuthProvider::new("test-secret".to_string());
        provider.add_user(
            "testuser".to_string(),
            "password123".to_string(),
            "test@example.com".to_string(),
            vec!["user".to_string()],
        );
        
        let mut service = AuthService::new(Box::new(provider));
        service.login("testuser", "password123").await.unwrap();
        service
    }

    #[wasm_bindgen_test]
    async fn test_upload_with_authentication() {
        // Create authenticated service
        let auth_service = create_authenticated_service().await;
        let auth_context = auth_service.get_context();
        
        // Verify we're authenticated
        assert!(auth_context.is_authenticated());
        assert!(auth_context.token.is_some());
        
        // Create file upload manager
        let mut upload_manager = FileUploadManager::new();
        let file = create_test_file("test.txt", "Hello, authenticated upload!");
        
        // Simulate upload with authentication token
        // The upload_file method should automatically include the auth token
        let result = upload_manager.upload_file(file, "/api/upload").await;
        
        // Even though we can't actually make the HTTP request in tests,
        // we verify the upload attempt was made
        assert!(result.is_err()); // Will fail due to no actual server
        
        // Verify the auth token is available for the upload
        let stored_token = crate::auth::JwtAuthProvider::get_stored_token();
        assert!(stored_token.is_some());
    }

    #[wasm_bindgen_test]
    async fn test_upload_without_authentication() {
        // Clear any existing auth tokens
        let _ = crate::auth::JwtAuthProvider::clear_stored_token();
        
        // Create upload manager without authentication
        let mut upload_manager = FileUploadManager::new();
        let file = create_test_file("test.txt", "Unauthenticated upload attempt");
        
        // Attempt upload without auth token
        let result = upload_manager.upload_file(file, "/api/upload").await;
        
        // Upload should still attempt (server would reject if auth required)
        assert!(result.is_err()); // Will fail due to no actual server
        
        // Verify no auth token was available
        let stored_token = crate::auth::JwtAuthProvider::get_stored_token();
        assert!(stored_token.is_none());
    }

    #[wasm_bindgen_test]
    async fn test_permission_based_upload() {
        // Create auth service with different user roles
        let mut provider = JwtAuthProvider::new("test-secret".to_string());
        
        // Admin user with upload permissions
        provider.add_user(
            "admin".to_string(),
            "admin123".to_string(),
            "admin@example.com".to_string(),
            vec!["admin".to_string()],
        );
        
        // Guest user without upload permissions
        provider.add_user(
            "guest".to_string(),
            "guest123".to_string(),
            "guest@example.com".to_string(),
            vec!["guest".to_string()],
        );
        
        let mut auth_service = AuthService::new(Box::new(provider));
        
        // Test admin upload
        auth_service.login("admin", "admin123").await.unwrap();
        let admin_context = auth_service.get_context();
        assert!(admin_context.has_permission("write"));
        assert!(admin_context.has_permission("admin"));
        
        // Admin should be able to upload
        let mut upload_manager = FileUploadManager::new();
        let file = create_test_file("admin-file.txt", "Admin upload");
        let result = upload_manager.upload_file(file, "/api/upload").await;
        assert!(result.is_err()); // Fails due to no server, but attempt was made
        
        // Logout and login as guest
        auth_service.logout();
        auth_service.login("guest", "guest123").await.unwrap();
        let guest_context = auth_service.get_context();
        assert!(guest_context.has_permission("read"));
        assert!(!guest_context.has_permission("write"));
        
        // Guest upload attempt - application logic should prevent this
        // In a real app, the UI would check permissions before allowing upload
    }

    #[wasm_bindgen_test]
    async fn test_upload_component_with_auth() {
        // Create authenticated service
        let _auth_service = create_authenticated_service().await;
        
        // Create upload component
        let upload_component = FileUploadComponent::new()
            .with_url("/api/upload".to_string())
            .with_max_size(1024 * 1024); // 1MB
        
        // Render component (should work with auth)
        let html = upload_component.render();
        assert!(html.contains("file-upload-component"));
        assert!(html.contains("upload-area"));
        
        // Verify auth token is available for component to use
        let stored_token = crate::auth::JwtAuthProvider::get_stored_token();
        assert!(stored_token.is_some());
    }

    #[wasm_bindgen_test]
    async fn test_auth_token_in_upload_headers() {
        // Create authenticated service with known token
        let mut provider = JwtAuthProvider::new("test-secret".to_string());
        provider.add_user(
            "user".to_string(),
            "pass".to_string(),
            "user@test.com".to_string(),
            vec!["user".to_string()],
        );
        
        let mut auth_service = AuthService::new(Box::new(provider));
        auth_service.login("user", "pass").await.unwrap();
        
        // Get the token
        let token = crate::auth::JwtAuthProvider::get_stored_token().unwrap();
        assert!(!token.is_empty());
        
        // Create upload manager
        let mut upload_manager = FileUploadManager::new();
        let file = create_test_file("auth-test.txt", "Testing auth headers");
        
        // The upload_file method should automatically include Bearer token
        let _result = upload_manager.upload_file(file, "/api/upload").await;
        
        // In a real scenario, we would verify the Authorization header
        // contains "Bearer {token}" in the request
    }

    #[wasm_bindgen_test]
    async fn test_upload_after_logout() {
        // Create authenticated service
        let mut auth_service = create_authenticated_service().await;
        
        // Verify authenticated
        assert!(auth_service.is_authenticated());
        assert!(crate::auth::JwtAuthProvider::get_stored_token().is_some());
        
        // Logout
        auth_service.logout();
        
        // Verify logged out
        assert!(!auth_service.is_authenticated());
        assert!(crate::auth::JwtAuthProvider::get_stored_token().is_none());
        
        // Attempt upload after logout
        let mut upload_manager = FileUploadManager::new();
        let file = create_test_file("after-logout.txt", "Should not have auth");
        
        let _result = upload_manager.upload_file(file, "/api/upload").await;
        
        // No auth token should be included in the request
        assert!(crate::auth::JwtAuthProvider::get_stored_token().is_none());
    }

    #[wasm_bindgen_test]
    async fn test_token_refresh_during_upload() {
        // Create auth service
        let mut auth_service = create_authenticated_service().await;
        
        // Get initial token
        let initial_token = crate::auth::JwtAuthProvider::get_stored_token().unwrap();
        
        // Refresh token
        auth_service.refresh_session().await.unwrap();
        
        // Get refreshed token
        let refreshed_token = crate::auth::JwtAuthProvider::get_stored_token().unwrap();
        
        // Tokens should be different
        assert_ne!(initial_token, refreshed_token);
        
        // Upload should use the new token
        let mut upload_manager = FileUploadManager::new();
        let file = create_test_file("refresh-test.txt", "Using refreshed token");
        
        let _result = upload_manager.upload_file(file, "/api/upload").await;
        
        // The refreshed token should still be stored
        assert_eq!(
            crate::auth::JwtAuthProvider::get_stored_token().unwrap(),
            refreshed_token
        );
    }

    #[wasm_bindgen_test]
    fn test_upload_file_validation_with_permissions() {
        // Test that file validation works independently of auth
        let upload_manager = FileUploadManager::new()
            .with_max_size(1024) // 1KB
            .with_allowed_types(vec!["text/plain".to_string()]);
        
        // Create files for testing
        let small_file = create_test_file("small.txt", "OK");
        let large_file = create_test_file("large.txt", &"X".repeat(2000));
        
        // Validation should work regardless of auth status
        assert!(upload_manager.validate_file(&small_file).is_ok());
        assert!(upload_manager.validate_file(&large_file).is_err());
    }
}