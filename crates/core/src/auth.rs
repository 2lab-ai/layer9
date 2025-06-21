//! Authentication support for Layer9

use crate::config::Config;
use crate::jwt::{Jwt, JwtClaims};
use sha2::{Digest, Sha256};
use base64::{engine::general_purpose::STANDARD, Engine};
use web_sys::Storage;

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user: Option<User>,
    pub token: Option<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
}

impl AuthContext {
    pub fn new() -> Self {
        Self {
            user: None,
            token: None,
            permissions: Vec::new(),
        }
    }
    
    pub fn is_authenticated(&self) -> bool {
        self.user.is_some()
    }
    
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }
    
    pub fn login(&mut self, user: User, token: String) {
        self.user = Some(user);
        self.token = Some(token);
    }
    
    pub fn logout(&mut self) {
        self.user = None;
        self.token = None;
        self.permissions.clear();
    }
}

pub trait AuthProvider: AuthProviderClone {
    fn authenticate(&self, username: &str, password: &str) -> Result<(User, String), String>;
    fn validate_token(&self, token: &str) -> Result<User, String>;
    fn refresh_token(&self, token: &str) -> Result<String, String>;
}

// Helper trait to make AuthProvider object-safe and cloneable
pub trait AuthProviderClone {
    fn clone_box(&self) -> Box<dyn AuthProvider>;
}

impl<T> AuthProviderClone for T
where
    T: AuthProvider + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn AuthProvider> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn AuthProvider> {
    fn clone(&self) -> Box<dyn AuthProvider> {
        self.clone_box()
    }
}

/// Mock auth provider for testing
#[derive(Clone)]
pub struct MockAuthProvider;

impl AuthProvider for MockAuthProvider {
    fn authenticate(&self, username: &str, _password: &str) -> Result<(User, String), String> {
        let user = User {
            id: "123".to_string(),
            username: username.to_string(),
            email: format!("{}@example.com", username),
            roles: vec!["user".to_string()],
        };
        let token = "mock-token-123".to_string();
        Ok((user, token))
    }
    
    fn validate_token(&self, _token: &str) -> Result<User, String> {
        Ok(User {
            id: "123".to_string(),
            username: "testuser".to_string(),
            email: "testuser@example.com".to_string(),
            roles: vec!["user".to_string()],
        })
    }
    
    fn refresh_token(&self, _token: &str) -> Result<String, String> {
        Ok("refreshed-token-456".to_string())
    }
}

/// JWT-based authentication provider
#[derive(Clone)]
pub struct JwtAuthProvider {
    jwt: Jwt,
    users: Vec<(String, String, User)>, // (username, password_hash, user)
}

impl JwtAuthProvider {
    pub fn new(secret: String) -> Self {
        Self {
            jwt: Jwt::new(secret),
            users: Vec::new(),
        }
    }

    pub fn add_user(&mut self, username: String, password: String, email: String, roles: Vec<String>) -> String {
        let id = format!("user-{}", self.users.len() + 1);
        let password_hash = self.hash_password(&password);
        let user = User {
            id: id.clone(),
            username: username.clone(),
            email,
            roles,
        };
        self.users.push((username, password_hash, user));
        id
    }

    fn hash_password(&self, password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(b"layer9-salt"); // In production, use a random salt per user
        let result = hasher.finalize();
        STANDARD.encode(result)
    }

    fn verify_password(&self, password: &str, hash: &str) -> bool {
        let computed_hash = self.hash_password(password);
        computed_hash == hash
    }

    fn get_local_storage() -> Option<Storage> {
        web_sys::window()?.local_storage().ok()?
    }

    pub fn store_token(token: &str) -> Result<(), String> {
        let storage = Self::get_local_storage()
            .ok_or_else(|| "Local storage not available".to_string())?;
        storage.set_item("auth_token", token)
            .map_err(|_| "Failed to store token".to_string())
    }

    pub fn get_stored_token() -> Option<String> {
        let storage = Self::get_local_storage()?;
        storage.get_item("auth_token").ok()?
    }

    pub fn clear_stored_token() -> Result<(), String> {
        let storage = Self::get_local_storage()
            .ok_or_else(|| "Local storage not available".to_string())?;
        storage.remove_item("auth_token")
            .map_err(|_| "Failed to clear token".to_string())
    }
}

impl AuthProvider for JwtAuthProvider {
    fn authenticate(&self, username: &str, password: &str) -> Result<(User, String), String> {
        // Find user
        let (_, stored_hash, user) = self.users.iter()
            .find(|(u, _, _)| u == username)
            .ok_or_else(|| "Invalid username or password".to_string())?;

        // Verify password
        if !self.verify_password(password, stored_hash) {
            return Err("Invalid username or password".to_string());
        }

        // Create JWT token
        let now = js_sys::Date::now() as u64 / 1000;
        let claims = JwtClaims {
            sub: user.id.clone(),
            username: user.username.clone(),
            email: user.email.clone(),
            roles: user.roles.clone(),
            exp: now + 86400, // 24 hours
            iat: now,
            permissions: self.get_permissions_for_roles(&user.roles),
        };

        let token = self.jwt.create_token(&claims)?;
        
        // Store token in local storage
        let _ = Self::store_token(&token);

        Ok((user.clone(), token))
    }

    fn validate_token(&self, token: &str) -> Result<User, String> {
        let claims = self.jwt.verify_token(token)?;
        
        Ok(User {
            id: claims.sub,
            username: claims.username,
            email: claims.email,
            roles: claims.roles,
        })
    }

    fn refresh_token(&self, token: &str) -> Result<String, String> {
        let claims = self.jwt.verify_token(token)?;
        
        // Create new token with extended expiration
        let now = js_sys::Date::now() as u64 / 1000;
        let new_claims = JwtClaims {
            sub: claims.sub,
            username: claims.username,
            email: claims.email,
            roles: claims.roles,
            exp: now + 86400, // Another 24 hours
            iat: now,
            permissions: claims.permissions,
        };

        let new_token = self.jwt.create_token(&new_claims)?;
        
        // Store new token
        let _ = Self::store_token(&new_token);

        Ok(new_token)
    }
}

impl JwtAuthProvider {
    pub fn get_permissions_for_roles(&self, roles: &[String]) -> Vec<String> {
        let mut permissions = Vec::new();
        
        for role in roles {
            match role.as_str() {
                "admin" => {
                    permissions.extend_from_slice(&[
                        "read".to_string(),
                        "write".to_string(),
                        "delete".to_string(),
                        "admin".to_string(),
                    ]);
                }
                "user" => {
                    permissions.extend_from_slice(&[
                        "read".to_string(),
                        "write".to_string(),
                    ]);
                }
                "guest" => {
                    permissions.push("read".to_string());
                }
                _ => {}
            }
        }
        
        permissions.sort();
        permissions.dedup();
        permissions
    }
}

// Hook function for authentication
pub fn use_auth() -> AuthContext {
    AuthContext::new()
}

// Auth service for managing authentication state
#[derive(Clone)]
pub struct AuthService {
    context: AuthContext,
    provider: Box<dyn AuthProvider>,
}

impl AuthService {
    pub fn new(provider: Box<dyn AuthProvider>) -> Self {
        let mut service = Self {
            context: AuthContext::new(),
            provider,
        };
        
        // Try to restore session from stored token
        service.restore_session();
        service
    }

    pub fn with_jwt_provider(secret: String) -> Self {
        Self::new(Box::new(JwtAuthProvider::new(secret)))
    }

    pub fn with_mock_provider() -> Self {
        Self::new(Box::new(MockAuthProvider))
    }
    
    /// Create an AuthService with configuration
    pub fn with_config(config: &Config) -> Self {
        if config.should_use_mock_auth() {
            Self::with_mock_provider()
        } else {
            let secret = config.get_jwt_secret();
            Self::with_jwt_provider(secret)
        }
    }

    pub async fn login(&mut self, username: &str, password: &str) -> Result<(), String> {
        let (user, token) = self.provider.authenticate(username, password)?;
        
        // Update permissions based on user roles
        let permissions = self.get_permissions_for_user(&user);
        
        self.context.user = Some(user);
        self.context.token = Some(token);
        self.context.permissions = permissions;
        
        Ok(())
    }

    pub fn logout(&mut self) {
        self.context.logout();
        // Clear stored token
        let _ = JwtAuthProvider::clear_stored_token();
    }

    pub fn is_authenticated(&self) -> bool {
        self.context.is_authenticated()
    }

    pub fn get_context(&self) -> &AuthContext {
        &self.context
    }

    pub fn get_current_user(&self) -> Option<&User> {
        self.context.user.as_ref()
    }

    pub async fn refresh_session(&mut self) -> Result<(), String> {
        if let Some(token) = &self.context.token {
            let new_token = self.provider.refresh_token(token)?;
            self.context.token = Some(new_token);
            Ok(())
        } else {
            Err("No active session to refresh".to_string())
        }
    }

    pub fn validate_token(&self, token: &str) -> Result<User, String> {
        self.provider.validate_token(token)
    }

    fn restore_session(&mut self) {
        if let Some(token) = JwtAuthProvider::get_stored_token() {
            if let Ok(user) = self.provider.validate_token(&token) {
                let permissions = self.get_permissions_for_user(&user);
                self.context.user = Some(user);
                self.context.token = Some(token);
                self.context.permissions = permissions;
            }
        }
    }

    fn get_permissions_for_user(&self, user: &User) -> Vec<String> {
        let mut permissions = Vec::new();
        
        for role in &user.roles {
            match role.as_str() {
                "admin" => {
                    permissions.extend_from_slice(&[
                        "read".to_string(),
                        "write".to_string(),
                        "delete".to_string(),
                        "admin".to_string(),
                    ]);
                }
                "user" => {
                    permissions.extend_from_slice(&[
                        "read".to_string(),
                        "write".to_string(),
                    ]);
                }
                "guest" => {
                    permissions.push("read".to_string());
                }
                _ => {}
            }
        }
        
        permissions.sort();
        permissions.dedup();
        permissions
    }
}

// Implement Debug manually since AuthProvider doesn't implement Debug
impl std::fmt::Debug for AuthService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthService")
            .field("context", &self.context)
            .field("provider", &"<AuthProvider>")
            .finish()
    }
}

impl Default for AuthService {
    fn default() -> Self {
        let config = Config::get_or_init();
        
        if config.should_use_mock_auth() {
            // Use mock provider for testing
            Self::with_mock_provider()
        } else {
            // Use JWT provider with configured or default secret
            let secret = config.get_jwt_secret();
            Self::with_jwt_provider(secret)
        }
    }
}

// Protected component wrapper
#[derive(Debug, Clone)]
pub struct Protected {
    pub required_permission: Option<String>,
    pub redirect_to: String,
}

impl Protected {
    pub fn new() -> Self {
        Self {
            required_permission: None,
            redirect_to: "/login".to_string(),
        }
    }

    pub fn with_permission(mut self, permission: String) -> Self {
        self.required_permission = Some(permission);
        self
    }

    pub fn with_redirect(mut self, redirect: String) -> Self {
        self.redirect_to = redirect;
        self
    }

    pub fn check_access(&self, auth_context: &AuthContext) -> bool {
        if !auth_context.is_authenticated() {
            return false;
        }

        if let Some(permission) = &self.required_permission {
            return auth_context.has_permission(permission);
        }

        true
    }
}

impl Default for Protected {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AuthContext {
    fn default() -> Self {
        Self::new()
    }
}
