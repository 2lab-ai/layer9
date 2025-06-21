//! Configuration module for Layer9
//! 
//! Provides centralized configuration management with support for
//! environment variables and defaults.

use std::sync::OnceLock;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

/// Global configuration instance
static CONFIG: OnceLock<Config> = OnceLock::new();

/// Main configuration structure
#[derive(Debug, Clone)]
pub struct Config {
    /// JWT secret for authentication
    /// Can be set via LAYER9_JWT_SECRET environment variable
    pub jwt_secret: Option<String>,
    
    /// Whether to use mock authentication (for testing)
    /// Can be set via LAYER9_USE_MOCK_AUTH environment variable
    pub use_mock_auth: bool,
    
    /// Default JWT secret if none is provided
    /// WARNING: This should only be used in development
    pub default_jwt_secret: String,
}

impl Config {
    /// Create a new configuration instance
    pub fn new() -> Self {
        Self {
            jwt_secret: None,
            use_mock_auth: false,
            default_jwt_secret: "layer9-development-secret-change-in-production".to_string(),
        }
    }
    
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::new();
        
        // In WASM environment, we can't access process env vars directly
        // Instead, we'll use a different approach for browser environments
        #[cfg(target_arch = "wasm32")]
        {
            // In browser, we might get config from:
            // 1. Global window object
            // 2. Meta tags
            // 3. Config endpoint
            // For now, we'll use defaults or injected values
            if let Some(window) = web_sys::window() {
                if let Ok(layer9_config) = js_sys::Reflect::get(&window, &"LAYER9_CONFIG".into()) {
                    if let Some(config_obj) = layer9_config.dyn_ref::<js_sys::Object>() {
                        // Try to get JWT secret
                        if let Ok(jwt_secret) = js_sys::Reflect::get(config_obj, &"jwtSecret".into()) {
                            if let Some(secret) = jwt_secret.as_string() {
                                if !secret.is_empty() {
                                    config.jwt_secret = Some(secret);
                                }
                            }
                        }
                        
                        // Try to get mock auth flag
                        if let Ok(use_mock) = js_sys::Reflect::get(config_obj, &"useMockAuth".into()) {
                            if let Some(mock_bool) = use_mock.as_bool() {
                                config.use_mock_auth = mock_bool;
                            }
                        }
                    }
                }
            }
        }
        
        // For non-WASM targets (like server-side), we could read from env vars
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(secret) = std::env::var("LAYER9_JWT_SECRET") {
                if !secret.is_empty() {
                    config.jwt_secret = Some(secret);
                }
            }
            
            if let Ok(use_mock) = std::env::var("LAYER9_USE_MOCK_AUTH") {
                config.use_mock_auth = use_mock.to_lowercase() == "true" || use_mock == "1";
            }
        }
        
        config
    }
    
    /// Get the JWT secret to use
    /// Returns the configured secret or the default if none is set
    pub fn get_jwt_secret(&self) -> String {
        self.jwt_secret.as_ref()
            .cloned()
            .unwrap_or_else(|| {
                #[cfg(target_arch = "wasm32")]
                web_sys::console::warn_1(&"Using default JWT secret. This should only be used in development!".into());
                
                #[cfg(not(target_arch = "wasm32"))]
                eprintln!("WARNING: Using default JWT secret. This should only be used in development!");
                
                self.default_jwt_secret.clone()
            })
    }
    
    /// Check if we should use mock authentication
    pub fn should_use_mock_auth(&self) -> bool {
        self.use_mock_auth
    }
    
    /// Initialize the global configuration
    /// This should be called once at application startup
    pub fn init() -> &'static Config {
        CONFIG.get_or_init(Self::from_env)
    }
    
    /// Get the global configuration instance
    /// Panics if init() hasn't been called
    pub fn get() -> &'static Config {
        CONFIG.get().expect("Config not initialized. Call Config::init() first.")
    }
    
    /// Get the global configuration instance, initializing if needed
    pub fn get_or_init() -> &'static Config {
        CONFIG.get_or_init(Self::from_env)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder pattern for configuration
pub struct ConfigBuilder {
    config: Config,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self {
            config: Config::new(),
        }
    }
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn jwt_secret(mut self, secret: String) -> Self {
        self.config.jwt_secret = Some(secret);
        self
    }
    
    pub fn use_mock_auth(mut self, use_mock: bool) -> Self {
        self.config.use_mock_auth = use_mock;
        self
    }
    
    pub fn build(self) -> Config {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_defaults() {
        let config = Config::new();
        assert!(config.jwt_secret.is_none());
        assert!(!config.use_mock_auth);
        assert!(!config.default_jwt_secret.is_empty());
    }
    
    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .jwt_secret("test-secret".to_string())
            .use_mock_auth(true)
            .build();
            
        assert_eq!(config.jwt_secret, Some("test-secret".to_string()));
        assert!(config.use_mock_auth);
    }
    
    #[test]
    fn test_get_jwt_secret() {
        let config = Config::new();
        assert_eq!(config.get_jwt_secret(), config.default_jwt_secret);
        
        let config_with_secret = ConfigBuilder::new()
            .jwt_secret("custom-secret".to_string())
            .build();
        assert_eq!(config_with_secret.get_jwt_secret(), "custom-secret");
    }
}