//! Authentication System - L3/L4

use wasm_bindgen::prelude::*;
use web_sys::{window, Storage};
use serde::{Deserialize, Serialize};
use crate::layers::*;

/// Authentication state
#[derive(Clone, Serialize, Deserialize)]
pub struct AuthState {
    pub user: Option<User>,
    pub token: Option<String>,
    pub expires_at: Option<u64>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub image: Option<String>,
    pub provider: AuthProvider,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AuthProvider {
    GitHub,
    Google,
    Email,
}

/// L4: Authentication Service
pub struct AuthService {
    storage: Storage,
}

impl L4::Service for AuthService {
    type Request = AuthRequest;
    type Response = AuthResponse;
    
    async fn handle(&self, req: Self::Request) -> Self::Response {
        match req {
            AuthRequest::Login(provider) => self.login(provider).await,
            AuthRequest::Logout => self.logout().await,
            AuthRequest::Refresh => self.refresh_token().await,
            AuthRequest::GetUser => self.get_current_user(),
        }
    }
}

pub enum AuthRequest {
    Login(AuthProvider),
    Logout,
    Refresh,
    GetUser,
}

pub enum AuthResponse {
    Success(AuthState),
    Error(String),
    Redirect(String),
}

impl AuthService {
    pub fn new() -> Result<Self, JsValue> {
        let storage = window()
            .ok_or("No window")?
            .local_storage()?
            .ok_or("No local storage")?;
        
        Ok(AuthService { storage })
    }
    
    async fn login(&self, provider: AuthProvider) -> AuthResponse {
        // OAuth flow
        let redirect_url = match provider {
            AuthProvider::GitHub => {
                let client_id = self.get_env("GITHUB_CLIENT_ID");
                let redirect_uri = self.get_env("REDIRECT_URI");
                format!(
                    "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=user:email",
                    client_id, redirect_uri
                )
            }
            AuthProvider::Google => {
                let client_id = self.get_env("GOOGLE_CLIENT_ID");
                let redirect_uri = self.get_env("REDIRECT_URI");
                format!(
                    "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=email%20profile",
                    client_id, redirect_uri
                )
            }
            AuthProvider::Email => {
                return AuthResponse::Error("Email auth not implemented".to_string());
            }
        };
        
        AuthResponse::Redirect(redirect_url)
    }
    
    async fn logout(&self) -> AuthResponse {
        // Clear storage
        let _ = self.storage.remove_item("warp_auth_token");
        let _ = self.storage.remove_item("warp_auth_user");
        
        AuthResponse::Success(AuthState {
            user: None,
            token: None,
            expires_at: None,
        })
    }
    
    async fn refresh_token(&self) -> AuthResponse {
        // Check if token is expired
        if let Ok(Some(token)) = self.storage.get_item("warp_auth_token") {
            // TODO: Verify JWT and refresh if needed
            AuthResponse::Success(self.get_state())
        } else {
            AuthResponse::Error("No token to refresh".to_string())
        }
    }
    
    fn get_current_user(&self) -> AuthResponse {
        AuthResponse::Success(self.get_state())
    }
    
    fn get_state(&self) -> AuthState {
        let user = self.storage
            .get_item("warp_auth_user")
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str(&s).ok());
        
        let token = self.storage
            .get_item("warp_auth_token")
            .ok()
            .flatten();
        
        let expires_at = self.storage
            .get_item("warp_auth_expires")
            .ok()
            .flatten()
            .and_then(|s| s.parse().ok());
        
        AuthState { user, token, expires_at }
    }
    
    fn get_env(&self, key: &str) -> String {
        // In real implementation, these would be injected at build time
        match key {
            "GITHUB_CLIENT_ID" => "your_github_client_id",
            "GOOGLE_CLIENT_ID" => "your_google_client_id", 
            "REDIRECT_URI" => "http://localhost:8080/auth/callback",
            _ => "missing_env_var",
        }.to_string()
    }
}

/// OAuth callback handler
#[wasm_bindgen]
pub async fn handle_oauth_callback(code: String, provider: String) -> Result<JsValue, JsValue> {
    let auth_service = AuthService::new()?;
    
    // Exchange code for token
    let token_url = match provider.as_str() {
        "github" => "https://github.com/login/oauth/access_token",
        "google" => "https://oauth2.googleapis.com/token",
        _ => return Err("Unknown provider".into()),
    };
    
    // TODO: Implement token exchange
    // This would typically be done server-side for security
    
    Ok(JsValue::from_str("Success"))
}

/// Hook for using auth in components
pub fn use_auth() -> AuthState {
    let auth_service = AuthService::new().expect("Failed to create auth service");
    auth_service.get_state()
}

/// Protected route wrapper
pub struct Protected<T: Component> {
    component: T,
    fallback: Option<Box<dyn Component>>,
}

impl<T: Component> Protected<T> {
    pub fn new(component: T) -> Self {
        Protected {
            component,
            fallback: None,
        }
    }
    
    pub fn fallback(mut self, fallback: impl Component + 'static) -> Self {
        self.fallback = Some(Box::new(fallback));
        self
    }
}

impl<T: Component> Component for Protected<T> {
    fn render(&self) -> Element {
        let auth = use_auth();
        
        if auth.user.is_some() {
            self.component.render()
        } else if let Some(fallback) = &self.fallback {
            fallback.render()
        } else {
            view! {
                <div class="auth-required">
                    <h2>"Authentication Required"</h2>
                    <p>"Please log in to view this content"</p>
                    <button id="login-github">"Login with GitHub"</button>
                </div>
            }
        }
    }
}