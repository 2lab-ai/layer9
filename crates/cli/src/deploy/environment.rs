//! Environment and secrets management

use anyhow::{Context, Result};
use std::{collections::HashMap, env, fs, path::Path};

use super::config::DeployConfig;

/// Environment configuration for deployment
#[derive(Debug, Clone)]
pub struct Environment {
    /// Environment name (production, staging, etc.)
    pub name: String,
    
    /// Environment variables
    pub variables: HashMap<String, String>,
    
    /// Secrets (sensitive values)
    pub secrets: Vec<Secret>,
}

/// Secret value
#[derive(Debug, Clone)]
pub struct Secret {
    /// Secret name/key
    pub name: String,
    
    /// Secret value (encrypted in memory)
    pub value: SecureString,
}

/// Secure string that doesn't expose value in debug output
#[derive(Clone)]
pub struct SecureString(String);

impl std::fmt::Debug for SecureString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecureString(***)")
    }
}

impl SecureString {
    pub fn new(value: String) -> Self {
        Self(value)
    }
    
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl Environment {
    /// Load environment configuration
    pub fn load(env_name: &str, config: &DeployConfig) -> Result<Self> {
        let mut variables = HashMap::new();
        let mut secrets = Vec::new();
        
        // Load from .env file if exists
        let env_files = [
            format!(".env.{}", env_name),
            format!(".env.{}.local", env_name),
            ".env".to_string(),
            ".env.local".to_string(),
        ];
        
        for env_file in &env_files {
            if Path::new(env_file).exists() {
                load_env_file(env_file, &mut variables)?;
            }
        }
        
        // Override with environment-specific config
        if let Some(env_config) = config.environment_config(env_name) {
            // Add configured variables
            for (key, value) in &env_config.variables {
                variables.insert(key.clone(), value.clone());
            }
            
            // Load secrets from environment or prompt
            for secret_name in &env_config.secrets {
                let value = load_secret(secret_name)?;
                secrets.push(Secret {
                    name: secret_name.clone(),
                    value: SecureString::new(value),
                });
            }
        }
        
        // Add Layer9-specific variables
        variables.insert("LAYER9_ENV".to_string(), env_name.to_string());
        variables.insert("NODE_ENV".to_string(), 
            if env_name == "production" { "production" } else { "development" }.to_string()
        );
        
        Ok(Self {
            name: env_name.to_string(),
            variables,
            secrets,
        })
    }
    
    /// Get all environment variables as a map
    pub fn to_env_map(&self) -> HashMap<String, String> {
        let mut map = self.variables.clone();
        
        // Add secrets to the map
        for secret in &self.secrets {
            map.insert(secret.name.clone(), secret.value.expose().to_string());
        }
        
        map
    }
    
    /// Validate that all required variables are set
    #[allow(dead_code)]
    pub fn validate(&self, required: &[&str]) -> Result<()> {
        let env_map = self.to_env_map();
        let mut missing = Vec::new();
        
        for var in required {
            if !env_map.contains_key(*var) {
                missing.push(*var);
            }
        }
        
        if !missing.is_empty() {
            return Err(anyhow::anyhow!(
                "Missing required environment variables: {}",
                missing.join(", ")
            ));
        }
        
        Ok(())
    }
}

/// Load environment variables from a file
fn load_env_file(path: &str, variables: &mut HashMap<String, String>) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read env file: {}", path))?;
    
    for line in content.lines() {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // Parse KEY=VALUE format
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            
            // Remove quotes if present
            let value = if (value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\'')) {
                &value[1..value.len()-1]
            } else {
                value
            };
            
            // Expand environment variables in value
            let expanded = expand_env_vars(value);
            variables.insert(key.to_string(), expanded);
        }
    }
    
    Ok(())
}

/// Load a secret value from environment or prompt user
fn load_secret(name: &str) -> Result<String> {
    // First try environment variable
    if let Ok(value) = env::var(name) {
        return Ok(value);
    }
    
    // Try with common prefixes
    for prefix in &["LAYER9_", "DEPLOY_", ""] {
        let var_name = format!("{}{}", prefix, name);
        if let Ok(value) = env::var(&var_name) {
            return Ok(value);
        }
    }
    
    // If not found, prompt user (only in interactive mode)
    if atty::is(atty::Stream::Stdin) {
        use dialoguer::Password;
        
        let value = Password::new()
            .with_prompt(format!("Enter value for {} (hidden)", name))
            .interact()?;
        
        Ok(value)
    } else {
        Err(anyhow::anyhow!(
            "Secret '{}' not found in environment. Set {} or run in interactive mode.",
            name, name
        ))
    }
}

/// Expand environment variables in a string
fn expand_env_vars(value: &str) -> String {
    let mut result = value.to_string();
    
    // Find all ${VAR} or $VAR patterns
    let re = regex::Regex::new(r"\$\{([^}]+)\}|\$([A-Za-z_][A-Za-z0-9_]*)").unwrap();
    
    for cap in re.captures_iter(value) {
        let var_name = cap.get(1).or(cap.get(2)).unwrap().as_str();
        if let Ok(var_value) = env::var(var_name) {
            let pattern = cap.get(0).unwrap().as_str();
            result = result.replace(pattern, &var_value);
        }
    }
    
    result
}

/// Generate .env.example file from current configuration
pub fn generate_env_example(config: &DeployConfig) -> Result<String> {
    let mut lines = vec![
        "# Layer9 Environment Variables".to_string(),
        "# Copy this file to .env and fill in the values".to_string(),
        "".to_string(),
    ];
    
    // Platform tokens
    lines.push("# Platform API Tokens".to_string());
    if config.platforms.contains_key("vercel") {
        lines.push("VERCEL_TOKEN=".to_string());
    }
    if config.platforms.contains_key("netlify") {
        lines.push("NETLIFY_TOKEN=".to_string());
    }
    if config.platforms.contains_key("aws") {
        lines.push("AWS_ACCESS_KEY_ID=".to_string());
        lines.push("AWS_SECRET_ACCESS_KEY=".to_string());
    }
    if config.platforms.contains_key("cloudflare") {
        lines.push("CLOUDFLARE_API_TOKEN=".to_string());
    }
    lines.push("".to_string());
    
    // Environment-specific variables
    for (env_name, env_config) in &config.environments {
        lines.push(format!("# {} Environment", env_name));
        
        // Variables
        for key in env_config.variables.keys() {
            lines.push(format!("{}=", key));
        }
        
        // Secrets
        for secret in &env_config.secrets {
            lines.push(format!("{}=", secret));
        }
        
        lines.push("".to_string());
    }
    
    Ok(lines.join("\n"))
}