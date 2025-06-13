//! JWT implementation for Layer9 authentication

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtHeader {
    pub alg: String,
    pub typ: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,        // Subject (user ID)
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
    pub exp: u64,          // Expiration time
    pub iat: u64,          // Issued at
    pub permissions: Vec<String>,
}

#[derive(Clone)]
pub struct Jwt {
    secret: String,
}

impl Jwt {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn create_token(&self, claims: &JwtClaims) -> Result<String, String> {
        let header = JwtHeader {
            alg: "HS256".to_string(),
            typ: "JWT".to_string(),
        };

        let header_json = serde_json::to_string(&header)
            .map_err(|e| format!("Failed to serialize header: {}", e))?;
        let claims_json = serde_json::to_string(&claims)
            .map_err(|e| format!("Failed to serialize claims: {}", e))?;

        let header_b64 = URL_SAFE_NO_PAD.encode(header_json.as_bytes());
        let claims_b64 = URL_SAFE_NO_PAD.encode(claims_json.as_bytes());

        let payload = format!("{}.{}", header_b64, claims_b64);
        let signature = self.sign(&payload);

        Ok(format!("{}.{}", payload, signature))
    }

    pub fn verify_token(&self, token: &str) -> Result<JwtClaims, String> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err("Invalid token format".to_string());
        }

        let payload = format!("{}.{}", parts[0], parts[1]);
        let signature = parts[2];

        // Verify signature
        let expected_signature = self.sign(&payload);
        if signature != expected_signature {
            return Err("Invalid signature".to_string());
        }

        // Decode claims
        let claims_bytes = URL_SAFE_NO_PAD
            .decode(parts[1])
            .map_err(|e| format!("Failed to decode claims: {}", e))?;
        let claims: JwtClaims = serde_json::from_slice(&claims_bytes)
            .map_err(|e| format!("Failed to parse claims: {}", e))?;

        // Check expiration
        let now = js_sys::Date::now() as u64 / 1000;
        if claims.exp < now {
            return Err("Token expired".to_string());
        }

        Ok(claims)
    }

    fn sign(&self, payload: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        hasher.update(self.secret.as_bytes());
        let result = hasher.finalize();
        URL_SAFE_NO_PAD.encode(result)
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_jwt_create_and_verify() {
        let jwt = Jwt::new("test-secret".to_string());
        let now = js_sys::Date::now() as u64 / 1000;
        
        let claims = JwtClaims {
            sub: "123".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            roles: vec!["user".to_string()],
            exp: now + 3600, // 1 hour from now
            iat: now,
            permissions: vec!["read".to_string()],
        };

        let token = jwt.create_token(&claims).unwrap();
        let verified_claims = jwt.verify_token(&token).unwrap();

        assert_eq!(verified_claims.sub, claims.sub);
        assert_eq!(verified_claims.username, claims.username);
        assert_eq!(verified_claims.email, claims.email);
    }

    #[wasm_bindgen_test]
    fn test_jwt_expired_token() {
        let jwt = Jwt::new("test-secret".to_string());
        let now = js_sys::Date::now() as u64 / 1000;
        
        let claims = JwtClaims {
            sub: "123".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            roles: vec!["user".to_string()],
            exp: now - 3600, // 1 hour ago (expired)
            iat: now - 7200,
            permissions: vec!["read".to_string()],
        };

        let token = jwt.create_token(&claims).unwrap();
        let result = jwt.verify_token(&token);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Token expired");
    }

    #[wasm_bindgen_test]
    fn test_jwt_invalid_signature() {
        let jwt1 = Jwt::new("secret1".to_string());
        let jwt2 = Jwt::new("secret2".to_string());
        let now = js_sys::Date::now() as u64 / 1000;
        
        let claims = JwtClaims {
            sub: "123".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            roles: vec!["user".to_string()],
            exp: now + 3600,
            iat: now,
            permissions: vec!["read".to_string()],
        };

        let token = jwt1.create_token(&claims).unwrap();
        let result = jwt2.verify_token(&token);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid signature");
    }
}