//! Security Features - L2/L3
//! CSRF protection, CSP, rate limiting, and other security features

use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// CSRF token management
pub struct CsrfProtection {
    token_store: Rc<RefCell<HashMap<String, CsrfToken>>>,
    secret: String,
}

#[derive(Clone)]
struct CsrfToken {
    _value: String,
    _created_at: f64,
    expires_at: f64,
}

impl CsrfProtection {
    pub fn new(secret: impl Into<String>) -> Self {
        CsrfProtection {
            token_store: Rc::new(RefCell::new(HashMap::new())),
            secret: secret.into(),
        }
    }

    pub fn generate_token(&self) -> String {
        let random_bytes = self.generate_random_bytes(32);
        let timestamp = js_sys::Date::now();

        // Create token payload
        let payload = format!(
            "{}{}{}",
            general_purpose::STANDARD.encode(&random_bytes),
            timestamp,
            &self.secret
        );

        // Hash the payload
        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        let hash = hasher.finalize();

        let token_value = general_purpose::STANDARD.encode(hash);

        // Store token
        let token = CsrfToken {
            _value: token_value.clone(),
            _created_at: timestamp,
            expires_at: timestamp + 3600000.0, // 1 hour
        };

        self.token_store
            .borrow_mut()
            .insert(token_value.clone(), token);

        token_value
    }

    pub fn verify_token(&self, token: &str) -> bool {
        let mut store = self.token_store.borrow_mut();

        if let Some(csrf_token) = store.get(token) {
            let now = js_sys::Date::now();
            if now < csrf_token.expires_at {
                // Token is valid
                true
            } else {
                // Token expired, remove it
                store.remove(token);
                false
            }
        } else {
            false
        }
    }

    pub fn cleanup_expired(&self) {
        let now = js_sys::Date::now();
        self.token_store
            .borrow_mut()
            .retain(|_, token| token.expires_at > now);
    }

    fn generate_random_bytes(&self, len: usize) -> Vec<u8> {
        let crypto = web_sys::window()
            .unwrap()
            .crypto()
            .expect("Crypto API not available");

        let mut bytes = vec![0u8; len];
        crypto
            .get_random_values_with_u8_array(&mut bytes)
            .expect("Failed to generate random bytes");

        bytes
    }
}

/// Content Security Policy builder
pub struct ContentSecurityPolicy {
    directives: HashMap<String, Vec<String>>,
}

impl Default for ContentSecurityPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl ContentSecurityPolicy {
    pub fn new() -> Self {
        ContentSecurityPolicy {
            directives: HashMap::new(),
        }
    }

    pub fn default_src(mut self, sources: Vec<&str>) -> Self {
        self.directives.insert(
            "default-src".to_string(),
            sources.into_iter().map(|s| s.to_string()).collect(),
        );
        self
    }

    pub fn script_src(mut self, sources: Vec<&str>) -> Self {
        self.directives.insert(
            "script-src".to_string(),
            sources.into_iter().map(|s| s.to_string()).collect(),
        );
        self
    }

    pub fn style_src(mut self, sources: Vec<&str>) -> Self {
        self.directives.insert(
            "style-src".to_string(),
            sources.into_iter().map(|s| s.to_string()).collect(),
        );
        self
    }

    pub fn img_src(mut self, sources: Vec<&str>) -> Self {
        self.directives.insert(
            "img-src".to_string(),
            sources.into_iter().map(|s| s.to_string()).collect(),
        );
        self
    }

    pub fn connect_src(mut self, sources: Vec<&str>) -> Self {
        self.directives.insert(
            "connect-src".to_string(),
            sources.into_iter().map(|s| s.to_string()).collect(),
        );
        self
    }

    pub fn font_src(mut self, sources: Vec<&str>) -> Self {
        self.directives.insert(
            "font-src".to_string(),
            sources.into_iter().map(|s| s.to_string()).collect(),
        );
        self
    }

    pub fn frame_src(mut self, sources: Vec<&str>) -> Self {
        self.directives.insert(
            "frame-src".to_string(),
            sources.into_iter().map(|s| s.to_string()).collect(),
        );
        self
    }

    pub fn build(&self) -> String {
        self.directives
            .iter()
            .map(|(directive, sources)| format!("{} {}", directive, sources.join(" ")))
            .collect::<Vec<_>>()
            .join("; ")
    }
    
    pub fn to_header(&self) -> String {
        if self.directives.is_empty() {
            "default-src 'self'".to_string()
        } else {
            self.build()
        }
    }
    
    pub fn with_nonce(&self) -> (String, String) {
        use base64::Engine;
        // Generate a simple random nonce for the CSP
        let mut bytes = vec![0u8; 16];
        for byte in &mut bytes {
            *byte = (js_sys::Math::random() * 255.0) as u8;
        }
        let nonce = base64::engine::general_purpose::STANDARD.encode(bytes);
        let mut csp = self.clone();
        
        // Add nonce to script-src
        let script_sources = csp.directives.entry("script-src".to_string())
            .or_insert_with(|| vec!["'self'".to_string()]);
        script_sources.push(format!("'nonce-{}'", nonce));
        
        (nonce.clone(), csp.to_header())
    }
}

impl Clone for ContentSecurityPolicy {
    fn clone(&self) -> Self {
        ContentSecurityPolicy {
            directives: self.directives.clone(),
        }
    }
}

/// XSS protection
pub struct XssProtection;

impl XssProtection {
    /// Sanitize HTML input
    pub fn sanitize_html(input: &str) -> String {
        input
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
            .replace('/', "&#x2F;")
    }

    /// Sanitize JavaScript string
    pub fn sanitize_js(input: &str) -> String {
        input
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\'', "\\'")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }

    /// Sanitize URL
    pub fn sanitize_url(input: &str) -> String {
        if input.starts_with("javascript:")
            || input.starts_with("data:")
            || input.starts_with("vbscript:")
        {
            "#".to_string()
        } else {
            urlencoding::encode(input).to_string()
        }
    }
}

/// Input validation
pub struct InputValidator;

impl InputValidator {
    pub fn email(input: &str) -> Result<(), String> {
        let email_regex =
            regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();

        if email_regex.is_match(input) {
            Ok(())
        } else {
            Err("Invalid email address".to_string())
        }
    }

    pub fn url(input: &str) -> Result<(), String> {
        if let Ok(url) = web_sys::Url::new(input) {
            let protocol = url.protocol();
            if protocol == "http:" || protocol == "https:" {
                Ok(())
            } else {
                Err("URL must use http or https protocol".to_string())
            }
        } else {
            Err("Invalid URL".to_string())
        }
    }

    pub fn alphanumeric(input: &str) -> Result<(), String> {
        if input.chars().all(|c| c.is_alphanumeric()) {
            Ok(())
        } else {
            Err("Input must be alphanumeric".to_string())
        }
    }

    pub fn length(input: &str, min: usize, max: usize) -> Result<(), String> {
        let len = input.len();
        if len >= min && len <= max {
            Ok(())
        } else {
            Err(format!("Length must be between {} and {}", min, max))
        }
    }
}

/// Password strength checker
pub struct PasswordStrength;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordAnalysis {
    pub score: u8, // 0-100
    pub feedback: Vec<String>,
    pub is_strong: bool,
}

impl PasswordStrength {
    pub fn analyze(password: &str) -> PasswordAnalysis {
        let mut score: i32 = 0;
        let mut feedback = vec![];

        // Length check
        let len = password.len();
        if len >= 8 {
            score += 20;
            if len >= 12 {
                score += 10;
            }
        } else {
            feedback.push("Password should be at least 8 characters long".to_string());
        }

        // Character variety
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_digit = password.chars().any(|c| c.is_numeric());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        if has_lowercase {
            score += 15;
        } else {
            feedback.push("Add lowercase letters".to_string());
        }

        if has_uppercase {
            score += 15;
        } else {
            feedback.push("Add uppercase letters".to_string());
        }

        if has_digit {
            score += 15;
        } else {
            feedback.push("Add numbers".to_string());
        }

        if has_special {
            score += 25;
        } else {
            feedback.push("Add special characters (!@#$%^&*)".to_string());
        }

        // Common patterns check
        let common_patterns = vec!["123", "abc", "qwerty", "password"];
        for pattern in common_patterns {
            if password.to_lowercase().contains(pattern) {
                score = score.saturating_sub(20);
                feedback.push(format!("Avoid common patterns like '{}'", pattern));
            }
        }

        PasswordAnalysis {
            score: (score.min(100) as u8),
            feedback,
            is_strong: score >= 80,
        }
    }
}

/// Secure cookie handling
pub struct SecureCookie;

impl SecureCookie {
    pub fn set(name: &str, value: &str, options: CookieOptions) {
        let document = web_sys::window().unwrap().document().unwrap();

        let mut cookie_string = format!("{}={}", name, value);

        if let Some(max_age) = options.max_age {
            cookie_string.push_str(&format!("; Max-Age={}", max_age));
        }

        if let Some(path) = options.path {
            cookie_string.push_str(&format!("; Path={}", path));
        }

        if let Some(domain) = options.domain {
            cookie_string.push_str(&format!("; Domain={}", domain));
        }

        if options.secure {
            cookie_string.push_str("; Secure");
        }

        if options.http_only {
            cookie_string.push_str("; HttpOnly");
        }

        match options.same_site {
            SameSite::Strict => cookie_string.push_str("; SameSite=Strict"),
            SameSite::Lax => cookie_string.push_str("; SameSite=Lax"),
            SameSite::None => cookie_string.push_str("; SameSite=None"),
        }

        js_sys::Reflect::set(&document, &"cookie".into(), &cookie_string.into()).ok();
    }

    pub fn get(name: &str) -> Option<String> {
        let document = web_sys::window().unwrap().document().unwrap();
        let cookies = js_sys::Reflect::get(&document, &"cookie".into())
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();

        for cookie in cookies.split(';') {
            let cookie = cookie.trim();
            if let Some((cookie_name, cookie_value)) = cookie.split_once('=') {
                if cookie_name == name {
                    return Some(cookie_value.to_string());
                }
            }
        }

        None
    }

    pub fn delete(name: &str) {
        Self::set(
            name,
            "",
            CookieOptions {
                max_age: Some(0),
                ..Default::default()
            },
        );
    }
}

#[derive(Default)]
pub struct CookieOptions {
    pub max_age: Option<i32>,
    pub path: Option<String>,
    pub domain: Option<String>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: SameSite,
}

#[derive(Default)]
pub enum SameSite {
    Strict,
    #[default]
    Lax,
    None,
}

/// Subresource Integrity (SRI) generator
pub struct SubresourceIntegrity;

impl SubresourceIntegrity {
    pub fn generate(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let hash = hasher.finalize();

        format!("sha256-{}", general_purpose::STANDARD.encode(hash))
    }

    pub fn verify(content: &str, integrity: &str) -> bool {
        if let Some(_hash) = integrity.strip_prefix("sha256-") {
            let generated = Self::generate(content);
            generated == integrity
        } else {
            false
        }
    }
}

/// Frame options for X-Frame-Options header
pub enum FrameOptions {
    Deny,
    SameOrigin,
    AllowFrom(String),
}

/// Referrer policy options
pub enum ReferrerPolicy {
    NoReferrer,
    NoReferrerWhenDowngrade,
    Origin,
    OriginWhenCrossOrigin,
    SameOrigin,
    StrictOrigin,
    StrictOriginWhenCrossOrigin,
    UnsafeUrl,
}

/// Permissions policy configuration
pub struct PermissionsPolicy {
    directives: HashMap<String, Vec<String>>,
}

impl Default for PermissionsPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl PermissionsPolicy {
    pub fn new() -> Self {
        PermissionsPolicy {
            directives: HashMap::new(),
        }
    }
    
    pub fn camera(mut self, allowed: Vec<&str>) -> Self {
        self.directives.insert("camera".to_string(), allowed.into_iter().map(|s| s.to_string()).collect());
        self
    }
    
    pub fn microphone(mut self, allowed: Vec<&str>) -> Self {
        self.directives.insert("microphone".to_string(), allowed.into_iter().map(|s| s.to_string()).collect());
        self
    }
    
    pub fn geolocation(mut self, allowed: Vec<&str>) -> Self {
        self.directives.insert("geolocation".to_string(), allowed.into_iter().map(|s| s.to_string()).collect());
        self
    }
    
    pub fn to_header(&self) -> String {
        self.directives.iter()
            .map(|(feature, allowed)| {
                if allowed.is_empty() {
                    format!("{}=()", feature)
                } else {
                    format!("{}=({})", feature, allowed.join(" "))
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Input sanitizer
pub struct InputSanitizer;

impl Default for InputSanitizer {
    fn default() -> Self {
        Self::new()
    }
}

impl InputSanitizer {
    pub fn new() -> Self {
        InputSanitizer
    }
    
    pub fn sanitize(&self, input: &str) -> String {
        XssProtection::sanitize_html(input)
    }
    
    pub fn sanitize_html(&self, input: &str) -> String {
        XssProtection::sanitize_html(input)
    }
    
    pub fn sanitize_url(&self, input: &str) -> String {
        XssProtection::sanitize_url(input)
    }
    
    pub fn is_valid_email(&self, input: &str) -> bool {
        InputValidator::email(input).is_ok()
    }
}

/// Security headers configuration
pub struct SecurityHeaders {
    headers: HashMap<String, String>,
}

impl Default for SecurityHeaders {
    fn default() -> Self {
        Self::secure_defaults()
    }
}

impl SecurityHeaders {
    pub fn new() -> Self {
        SecurityHeaders {
            headers: HashMap::new(),
        }
    }
    pub fn secure_defaults() -> Self {
        let mut headers = HashMap::new();

        headers.insert("X-Content-Type-Options".to_string(), "nosniff".to_string());
        headers.insert("X-Frame-Options".to_string(), "DENY".to_string());
        headers.insert("X-XSS-Protection".to_string(), "1; mode=block".to_string());
        headers.insert(
            "Referrer-Policy".to_string(),
            "strict-origin-when-cross-origin".to_string(),
        );
        headers.insert(
            "Permissions-Policy".to_string(),
            "camera=(), microphone=(), geolocation=()".to_string(),
        );

        SecurityHeaders { headers }
    }

    pub fn add_hsts(mut self, max_age: u32, include_subdomains: bool, preload: bool) -> Self {
        let mut value = format!("max-age={}", max_age);
        if include_subdomains {
            value.push_str("; includeSubDomains");
        }
        if preload {
            value.push_str("; preload");
        }

        self.headers
            .insert("Strict-Transport-Security".to_string(), value);
        self
    }

    pub fn add_csp(mut self, csp: ContentSecurityPolicy) -> Self {
        self.headers
            .insert("Content-Security-Policy".to_string(), csp.build());
        self
    }
    
    pub fn frame_options(mut self, option: FrameOptions) -> Self {
        let value = match option {
            FrameOptions::Deny => "DENY",
            FrameOptions::SameOrigin => "SAMEORIGIN",
            FrameOptions::AllowFrom(url) => return {
                self.headers.insert("X-Frame-Options".to_string(), format!("ALLOW-FROM {}", url));
                self
            },
        };
        self.headers.insert("X-Frame-Options".to_string(), value.to_string());
        self
    }
    
    pub fn referrer_policy(mut self, policy: ReferrerPolicy) -> Self {
        let value = match policy {
            ReferrerPolicy::NoReferrer => "no-referrer",
            ReferrerPolicy::NoReferrerWhenDowngrade => "no-referrer-when-downgrade",
            ReferrerPolicy::Origin => "origin",
            ReferrerPolicy::OriginWhenCrossOrigin => "origin-when-cross-origin",
            ReferrerPolicy::SameOrigin => "same-origin",
            ReferrerPolicy::StrictOrigin => "strict-origin",
            ReferrerPolicy::StrictOriginWhenCrossOrigin => "strict-origin-when-cross-origin",
            ReferrerPolicy::UnsafeUrl => "unsafe-url",
        };
        self.headers.insert("Referrer-Policy".to_string(), value.to_string());
        self
    }
    
    pub fn to_headers(&self) -> HashMap<String, String> {
        self.headers.clone()
    }

    pub fn build(&self) -> HashMap<String, String> {
        self.headers.clone()
    }
}

/// Security context for components
pub struct SecurityContext {
    csrf: CsrfProtection,
    _csp: ContentSecurityPolicy,
}

impl SecurityContext {
    pub fn new(secret: impl Into<String>) -> Self {
        SecurityContext {
            csrf: CsrfProtection::new(secret),
            _csp: ContentSecurityPolicy::new()
                .default_src(vec!["'self'"])
                .script_src(vec!["'self'", "'unsafe-inline'"])
                .style_src(vec!["'self'", "'unsafe-inline'"]),
        }
    }

    pub fn csrf_token(&self) -> String {
        self.csrf.generate_token()
    }

    pub fn verify_csrf(&self, token: &str) -> bool {
        self.csrf.verify_token(token)
    }
    
    pub fn security_headers(&self) -> SecurityHeaders {
        SecurityHeaders::secure_defaults()
            .add_csp(self._csp.clone())
    }
}

/// Security hooks
pub fn use_security() -> SecurityContext {
    // In real app, this would get from context
    SecurityContext::new("layer9-secret-key")
}

pub fn use_csrf_token() -> String {
    use_security().csrf_token()
}

// Re-exports
use regex;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_csrf_token_generation() {
        let csrf = CsrfProtection::new("test-secret");
        let token1 = csrf.generate_token();
        let token2 = csrf.generate_token();
        
        // Tokens should be unique
        assert_ne!(token1, token2);
        
        // Tokens should be base64 encoded
        assert!(general_purpose::STANDARD.decode(&token1).is_ok());
        assert!(general_purpose::STANDARD.decode(&token2).is_ok());
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_csrf_token_verification() {
        let csrf = CsrfProtection::new("test-secret");
        let token = csrf.generate_token();
        
        // Valid token should verify
        assert!(csrf.verify_token(&token));
        
        // Invalid token should fail
        assert!(!csrf.verify_token("invalid-token"));
        
        // Empty token should fail
        assert!(!csrf.verify_token(""));
    }

    #[test]
    fn test_csp_header_generation() {
        let csp = ContentSecurityPolicy::new();
        let header = csp.to_header();
        
        assert!(header.contains("default-src 'self'"));
        
        // Test with custom directives
        let custom_csp = ContentSecurityPolicy::new()
            .default_src(vec!["'self'", "https:"])
            .script_src(vec!["'self'", "'unsafe-inline'"])
            .style_src(vec!["'self'", "'unsafe-inline'", "https://fonts.googleapis.com"])
            .connect_src(vec!["'self'", "wss:", "https:"]);
            
        let custom_header = custom_csp.to_header();
        assert!(custom_header.contains("default-src 'self' https:"));
        assert!(custom_header.contains("script-src 'self' 'unsafe-inline'"));
        assert!(custom_header.contains("style-src 'self' 'unsafe-inline' https://fonts.googleapis.com"));
        assert!(custom_header.contains("connect-src 'self' wss: https:"));
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_nonce_generation() {
        let csp = ContentSecurityPolicy::new();
        let (nonce1, header1) = csp.with_nonce();
        let (nonce2, header2) = csp.with_nonce();
        
        // Nonces should be unique
        assert_ne!(nonce1, nonce2);
        
        // Nonces should be in the header
        assert!(header1.contains(&format!("'nonce-{}'", nonce1)));
        assert!(header2.contains(&format!("'nonce-{}'", nonce2)));
    }


    #[test]
    fn test_security_headers() {
        let headers = SecurityHeaders::default();
        let header_map = headers.to_headers();
        
        // Check default headers
        assert_eq!(header_map.get("X-Frame-Options"), Some(&"DENY".to_string()));
        assert_eq!(header_map.get("X-Content-Type-Options"), Some(&"nosniff".to_string()));
        assert_eq!(header_map.get("X-XSS-Protection"), Some(&"1; mode=block".to_string()));
        assert_eq!(header_map.get("Referrer-Policy"), Some(&"strict-origin-when-cross-origin".to_string()));
        
        // Custom headers
        let custom_headers = SecurityHeaders::new()
            .frame_options(FrameOptions::SameOrigin)
            .referrer_policy(ReferrerPolicy::NoReferrer);
            
        let custom_map = custom_headers.to_headers();
        assert_eq!(custom_map.get("X-Frame-Options"), Some(&"SAMEORIGIN".to_string()));
        assert_eq!(custom_map.get("Referrer-Policy"), Some(&"no-referrer".to_string()));
    }

    #[test]
    fn test_permissions_policy() {
        let policy = PermissionsPolicy::new();
        let header = policy.to_header();
        
        // Default should be empty
        assert_eq!(header, "");
        
        // With features
        let custom_policy = PermissionsPolicy::new()
            .camera(vec!["'none'"])
            .microphone(vec!["'self'"])
            .geolocation(vec!["'self'", "https://maps.example.com"]);
            
        let custom_header = custom_policy.to_header();
        assert!(custom_header.contains("camera=('none')"));
        assert!(custom_header.contains("microphone=('self')"));
        assert!(custom_header.contains("geolocation=('self' https://maps.example.com)"));
    }

    #[test]
    fn test_input_sanitization() {
        let sanitizer = InputSanitizer::new();
        
        // HTML sanitization
        assert_eq!(
            sanitizer.sanitize_html("<script>alert('xss')</script>Hello"),
            "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;&#x2F;script&gt;Hello"
        );
        
        // URL sanitization
        assert_eq!(
            sanitizer.sanitize_url("javascript:alert('xss')"),
            "#"
        );
        assert_eq!(
            sanitizer.sanitize_url("https://example.com"),
            "https%3A%2F%2Fexample.com"
        );
        
        // Email validation
        assert!(sanitizer.is_valid_email("test@example.com"));
        assert!(!sanitizer.is_valid_email("invalid-email"));
        assert!(!sanitizer.is_valid_email("test@"));
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_security_context() {
        let ctx = SecurityContext::new("test-secret");
        
        // Headers should be set
        assert!(!ctx.security_headers().to_headers().is_empty());
        
        // CSRF token generation and verification
        let token = ctx.csrf_token();
        assert!(ctx.verify_csrf(&token));
        assert!(!ctx.verify_csrf("invalid-token"));
    }
}
