//! Environment Variables Support - L2/L3

use once_cell::sync::Lazy;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Environment variables store
static ENV_VARS: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut vars = HashMap::new();

    // In production, these would be injected at build time
    #[cfg(debug_assertions)]
    {
        // Development defaults
        vars.insert("APP_ENV".to_string(), "development".to_string());
        vars.insert("API_URL".to_string(), "http://localhost:3001".to_string());
        vars.insert("LOG_LEVEL".to_string(), "debug".to_string());
    }

    // Try to load from window.__ENV__ if available
    if let Some(window) = web_sys::window() {
        if let Ok(env_obj) = js_sys::Reflect::get(&window, &"__ENV__".into()) {
            let entries = js_sys::Object::entries(&env_obj.into());
            for i in 0..entries.length() {
                let entry = entries.get(i);
                if !entry.is_undefined() && !entry.is_null() && entry.is_array() {
                    let entry_array = js_sys::Array::from(&entry);
                    if entry_array.length() >= 2 {
                        if let (Some(key), Some(value)) = (
                            entry_array.get(0).as_string(),
                            entry_array.get(1).as_string(),
                        ) {
                            vars.insert(key, value);
                        }
                    }
                }
            }
        }
    }

    vars
});

/// Get environment variable
pub fn env(key: &str) -> Option<String> {
    ENV_VARS.get(key).cloned()
}

/// Get environment variable or default
pub fn env_or(key: &str, default: &str) -> String {
    ENV_VARS
        .get(key)
        .cloned()
        .unwrap_or_else(|| default.to_string())
}

/// Get required environment variable
pub fn env_required(key: &str) -> Result<String, String> {
    ENV_VARS
        .get(key)
        .cloned()
        .ok_or_else(|| format!("Required environment variable {} not found", key))
}

/// Check if running in production
pub fn is_production() -> bool {
    env("APP_ENV").map(|e| e == "production").unwrap_or(false)
}

/// Check if running in development
pub fn is_development() -> bool {
    env("APP_ENV").map(|e| e == "development").unwrap_or(true)
}

/// Environment configuration
#[derive(Clone)]
pub struct Config {
    pub app_env: String,
    pub api_url: String,
    pub log_level: String,
    pub features: Features,
    pub auth: AuthConfig,
    pub analytics: AnalyticsConfig,
}

#[derive(Clone)]
pub struct Features {
    pub maintenance_mode: bool,
    pub new_feature_flag: bool,
    pub experimental: bool,
}

#[derive(Clone)]
pub struct AuthConfig {
    pub oauth_providers: Vec<String>,
    pub session_timeout: u64,
    pub jwt_secret: Option<String>,
}

#[derive(Clone)]
pub struct AnalyticsConfig {
    pub enabled: bool,
    pub tracking_id: Option<String>,
    pub sample_rate: f32,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Ok(Config {
            app_env: env_or("APP_ENV", "development"),
            api_url: env_required("API_URL")?,
            log_level: env_or("LOG_LEVEL", "info"),
            features: Features {
                maintenance_mode: env("MAINTENANCE_MODE")
                    .map(|v| v == "true")
                    .unwrap_or(false),
                new_feature_flag: env("FEATURE_NEW_UI").map(|v| v == "true").unwrap_or(false),
                experimental: env("EXPERIMENTAL").map(|v| v == "true").unwrap_or(false),
            },
            auth: AuthConfig {
                oauth_providers: env("OAUTH_PROVIDERS")
                    .map(|v| v.split(',').map(|s| s.to_string()).collect())
                    .unwrap_or_else(|| vec!["github".to_string()]),
                session_timeout: env("SESSION_TIMEOUT")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(3600),
                jwt_secret: env("JWT_SECRET"),
            },
            analytics: AnalyticsConfig {
                enabled: env("ANALYTICS_ENABLED")
                    .map(|v| v == "true")
                    .unwrap_or(true),
                tracking_id: env("ANALYTICS_TRACKING_ID"),
                sample_rate: env("ANALYTICS_SAMPLE_RATE")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1.0),
            },
        })
    }
}

/// Global config instance
static CONFIG: Lazy<Result<Config, String>> = Lazy::new(|| Config::from_env());

/// Get global config
pub fn config() -> Result<&'static Config, &'static String> {
    CONFIG.as_ref()
}

/// Environment variable injection for build time
#[wasm_bindgen]
pub fn inject_env(key: String, value: String) {
    // This would be called during build to inject variables
    // In practice, we'd use a build script or webpack plugin
    web_sys::console::log_1(&format!("Injecting env var: {} = {}", key, value).into());
}

/// Feature flags
pub struct FeatureFlags;

impl FeatureFlags {
    pub fn is_enabled(feature: &str) -> bool {
        match feature {
            "maintenance_mode" => config()
                .map(|c| c.features.maintenance_mode)
                .unwrap_or(false),
            "new_ui" => config()
                .map(|c| c.features.new_feature_flag)
                .unwrap_or(false),
            "experimental" => config().map(|c| c.features.experimental).unwrap_or(false),
            _ => false,
        }
    }

    pub fn require(feature: &str) -> Result<(), String> {
        if Self::is_enabled(feature) {
            Ok(())
        } else {
            Err(format!("Feature {} is not enabled", feature))
        }
    }
}

/// Environment-specific behavior
pub fn with_env<T>(production: impl FnOnce() -> T, development: impl FnOnce() -> T) -> T {
    if is_production() {
        production()
    } else {
        development()
    }
}

/// Macro for compile-time environment variables
#[macro_export]
macro_rules! env_at_compile_time {
    ($key:expr) => {
        option_env!($key).unwrap_or("")
    };
    ($key:expr, $default:expr) => {
        option_env!($key).unwrap_or($default)
    };
}

/// Build information
pub struct BuildInfo;

impl BuildInfo {
    pub fn version() -> &'static str {
        env_at_compile_time!("CARGO_PKG_VERSION")
    }

    pub fn commit_hash() -> &'static str {
        env_at_compile_time!("GIT_COMMIT", "unknown")
    }

    pub fn build_date() -> &'static str {
        env_at_compile_time!("BUILD_DATE", "unknown")
    }

    pub fn target() -> &'static str {
        env_at_compile_time!("TARGET", "wasm32-unknown-unknown")
    }
}
