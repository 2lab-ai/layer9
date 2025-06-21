//! Deployment configuration

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt, path::{Path, PathBuf}};

/// Supported deployment platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Vercel,
    Netlify,
    #[serde(rename = "aws")]
    Aws,
    Cloudflare,
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platform::Vercel => write!(f, "Vercel"),
            Platform::Netlify => write!(f, "Netlify"),
            Platform::Aws => write!(f, "AWS S3/CloudFront"),
            Platform::Cloudflare => write!(f, "Cloudflare Pages"),
        }
    }
}

impl std::str::FromStr for Platform {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "vercel" => Ok(Platform::Vercel),
            "netlify" => Ok(Platform::Netlify),
            "aws" => Ok(Platform::Aws),
            "cloudflare" => Ok(Platform::Cloudflare),
            _ => Err(anyhow::anyhow!("Unknown platform: {}", s)),
        }
    }
}

/// Deployment configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct DeployConfig {
    /// Default deployment settings
    pub deploy: DeploySettings,
    
    /// Platform-specific configurations
    #[serde(default)]
    pub platforms: HashMap<String, PlatformConfig>,
    
    /// Environment-specific settings
    #[serde(default)]
    pub environments: HashMap<String, EnvironmentConfig>,
}

/// General deployment settings
#[derive(Debug, Deserialize, Serialize)]
pub struct DeploySettings {
    /// Default platform
    pub platform: Platform,
    
    /// Build output directory
    #[serde(default = "default_build_dir")]
    pub build_dir: String,
    
    /// Default environment
    #[serde(default = "default_environment")]
    pub environment: String,
    
    /// Project name (used for deployment naming)
    pub project_name: Option<String>,
    
    /// Custom domain (if configured)
    pub domain: Option<String>,
    
    /// Enable deployment notifications
    #[serde(default)]
    pub notifications: bool,
}

/// Platform-specific configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct PlatformConfig {
    /// API token/key (usually from environment variable)
    pub token: Option<String>,
    
    /// Team/Organization ID
    pub team_id: Option<String>,
    
    /// Project ID (for existing projects)
    pub project_id: Option<String>,
    
    /// Region (for AWS/Cloudflare)
    pub region: Option<String>,
    
    /// Custom build command
    pub build_command: Option<String>,
    
    /// Output directory override
    pub output_directory: Option<String>,
    
    /// Framework preset
    pub framework: Option<String>,
    
    /// Node.js version
    pub node_version: Option<String>,
    
    /// Additional platform-specific settings
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Environment-specific configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct EnvironmentConfig {
    /// Environment variables
    #[serde(default)]
    pub variables: HashMap<String, String>,
    
    /// Secret references (actual values from .env or external)
    #[serde(default)]
    pub secrets: Vec<String>,
    
    /// Custom domain for this environment
    pub domain: Option<String>,
    
    /// Build command override
    pub build_command: Option<String>,
    
    /// Deployment hooks
    #[serde(default)]
    pub hooks: DeploymentHooks,
}

/// Deployment lifecycle hooks
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct DeploymentHooks {
    /// Command to run before build
    pub pre_build: Option<String>,
    
    /// Command to run after build
    pub post_build: Option<String>,
    
    /// Command to run after successful deployment
    pub post_deploy: Option<String>,
}

impl DeployConfig {
    /// Load configuration from file
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;
        
        let config: Self = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))?;
        
        Ok(config)
    }
    
    /// Load configuration from project root
    pub fn from_project_root() -> Result<Self> {
        let paths = [
            PathBuf::from("layer9.deploy.toml"),
            PathBuf::from(".layer9/deploy.toml"),
            PathBuf::from("deploy.toml"),
        ];
        
        for path in &paths {
            if path.exists() {
                return Self::from_file(path);
            }
        }
        
        // Return default configuration if no file found
        Ok(Self::default())
    }
    
    /// Get platform-specific configuration
    pub fn platform_config(&self, platform: Platform) -> Option<&PlatformConfig> {
        let key = match platform {
            Platform::Vercel => "vercel",
            Platform::Netlify => "netlify",
            Platform::Aws => "aws",
            Platform::Cloudflare => "cloudflare",
        };
        self.platforms.get(key)
    }
    
    /// Get environment-specific configuration
    pub fn environment_config(&self, env: &str) -> Option<&EnvironmentConfig> {
        self.environments.get(env)
    }
}

impl Default for DeployConfig {
    fn default() -> Self {
        Self {
            deploy: DeploySettings {
                platform: Platform::Vercel,
                build_dir: default_build_dir(),
                environment: default_environment(),
                project_name: None,
                domain: None,
                notifications: false,
            },
            platforms: HashMap::new(),
            environments: HashMap::new(),
        }
    }
}

fn default_build_dir() -> String {
    "dist".to_string()
}

fn default_environment() -> String {
    "production".to_string()
}

/// Example configuration file content
pub const EXAMPLE_CONFIG: &str = r#"# Layer9 Deployment Configuration

[deploy]
platform = "vercel"          # Default platform: vercel, netlify, aws, cloudflare
build_dir = "dist"           # Build output directory
environment = "production"   # Default environment
project_name = "my-layer9-app"
domain = "example.com"

# Platform configurations
[platforms.vercel]
token = "$VERCEL_TOKEN"      # API token from environment variable
team_id = "team_xxx"         # Optional: Vercel team ID
framework = "vanilla"        # Framework preset
node_version = "18.x"

[platforms.netlify]
token = "$NETLIFY_TOKEN"
team_id = "netlify-team"

[platforms.aws]
token = "$AWS_ACCESS_KEY_ID"
region = "us-east-1"
# S3 bucket will be created as: {project_name}-{environment}

[platforms.cloudflare]
token = "$CLOUDFLARE_API_TOKEN"
account_id = "your-account-id"

# Environment configurations
[environments.production]
domain = "app.example.com"

[environments.production.variables]
API_URL = "https://api.example.com"
PUBLIC_URL = "https://app.example.com"

[environments.production.secrets]
secrets = ["DATABASE_URL", "JWT_SECRET", "API_KEY"]

[environments.staging]
domain = "staging.example.com"
build_command = "layer9 build --mode staging"

[environments.staging.variables]
API_URL = "https://staging-api.example.com"
PUBLIC_URL = "https://staging.example.com"
DEBUG = "true"

[environments.preview]
# Preview deployments get automatic URLs
[environments.preview.variables]
API_URL = "https://preview-api.example.com"
DEBUG = "true"

# Deployment hooks
[environments.production.hooks]
pre_build = "npm test"
post_deploy = "curl -X POST https://api.example.com/deploy-webhook"
"#;