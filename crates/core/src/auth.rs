//! Authentication support for Layer9

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

impl Default for AuthContext {
    fn default() -> Self {
        Self::new()
    }
}

pub trait AuthProvider: Send + Sync {
    fn authenticate(&self, username: &str, password: &str) -> Result<(User, String), String>;
    fn validate_token(&self, token: &str) -> Result<User, String>;
    fn refresh_token(&self, token: &str) -> Result<String, String>;
}

/// Mock auth provider for testing
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

/// Hook for using authentication in components
pub fn use_auth() -> AuthContext {
    // In a real implementation, this would use a context provider
    AuthContext::new()
}

/// Authentication service for managing auth state
#[derive(Clone)]
pub struct AuthService {
    context: AuthContext,
}

impl AuthService {
    pub fn new() -> Self {
        Self {
            context: AuthContext::new(),
        }
    }
    
    pub fn with_provider<P: AuthProvider + 'static>(_provider: P) -> Self {
        Self {
            context: AuthContext::new(),
        }
    }
    
    pub async fn login(&mut self, username: &str, password: &str) -> Result<(), String> {
        let provider = MockAuthProvider;
        let (user, token) = provider.authenticate(username, password)?;
        self.context.login(user, token);
        Ok(())
    }
    
    pub fn logout(&mut self) {
        self.context.logout();
    }
    
    pub fn is_authenticated(&self) -> bool {
        self.context.is_authenticated()
    }
    
    pub fn current_user(&self) -> Option<&User> {
        self.context.user.as_ref()
    }
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new()
    }
}

/// Component wrapper for protected routes
#[derive(Clone)]
pub struct Protected {
    pub required_permission: Option<String>,
}

impl Protected {
    pub fn new() -> Self {
        Self {
            required_permission: None,
        }
    }
    
    pub fn with_permission(permission: String) -> Self {
        Self {
            required_permission: Some(permission),
        }
    }
    
    pub fn can_render(&self, auth_context: &AuthContext) -> bool {
        if !auth_context.is_authenticated() {
            return false;
        }
        
        if let Some(ref permission) = self.required_permission {
            auth_context.has_permission(permission)
        } else {
            true
        }
    }
}

impl Default for Protected {
    fn default() -> Self {
        Self::new()
    }
}