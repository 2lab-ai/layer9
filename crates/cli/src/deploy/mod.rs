//! Layer9 Deployment System
//! 
//! Provides a unified deployment interface for multiple platforms without
//! requiring external CLI tools. Uses platform APIs directly for deployment.

pub mod config;
pub mod vercel;
pub mod netlify;
pub mod aws;
pub mod cloudflare;
pub mod environment;
pub mod status;

use anyhow::Result;
use colored::*;
use std::path::PathBuf;

pub use config::{DeployConfig, Platform};
pub use environment::Environment;
pub use status::{DeploymentStatus, DeploymentLog};

/// Deployment options
pub struct DeployOptions {
    /// Target platform
    pub platform: Platform,
    /// Build output directory
    pub build_dir: PathBuf,
    /// Environment (production, staging, preview)
    pub environment: String,
    /// Configuration file path
    pub config_path: Option<PathBuf>,
    /// Force deployment without confirmation
    pub force: bool,
    /// Show deployment logs
    #[allow(dead_code)]
    pub verbose: bool,
}

/// Deployment result
pub struct DeployResult {
    /// Deployment ID
    pub deployment_id: String,
    /// Deployment URL
    pub url: String,
    /// Preview URL (if available)
    pub preview_url: Option<String>,
    /// Deployment status
    pub status: DeploymentStatus,
    /// Build logs
    #[allow(dead_code)]
    pub logs: Vec<DeploymentLog>,
}

/// Deploy to the specified platform
pub async fn deploy(options: DeployOptions) -> Result<DeployResult> {
    println!("{}", "🚀 Starting deployment...".bright_blue().bold());
    
    // Load deployment configuration
    let config = if let Some(path) = &options.config_path {
        DeployConfig::from_file(path)?
    } else {
        DeployConfig::from_project_root()?
    };
    
    // Validate build directory
    if !options.build_dir.exists() {
        return Err(anyhow::anyhow!(
            "Build directory not found: {:?}. Run 'layer9 build' first.",
            options.build_dir
        ));
    }
    
    // Load environment variables and secrets
    let env = Environment::load(&options.environment, &config)?;
    
    // Confirm deployment (unless forced)
    if !options.force {
        println!("\n{}", "📋 Deployment Summary:".yellow().bold());
        println!("  Platform:    {}", options.platform);
        println!("  Environment: {}", options.environment);
        println!("  Build dir:   {:?}", options.build_dir);
        println!("  Variables:   {} defined", env.variables.len());
        println!("  Secrets:     {} configured", env.secrets.len());
        
        if !confirm_deployment()? {
            println!("{}", "❌ Deployment cancelled".red());
            return Err(anyhow::anyhow!("Deployment cancelled by user"));
        }
    }
    
    // Deploy to the selected platform
    let result = match options.platform {
        Platform::Vercel => vercel::deploy(&options, &config, &env).await?,
        Platform::Netlify => netlify::deploy(&options, &config, &env).await?,
        Platform::Aws => aws::deploy(&options, &config, &env).await?,
        Platform::Cloudflare => cloudflare::deploy(&options, &config, &env).await?,
    };
    
    // Show deployment result
    println!("\n{}", "✅ Deployment successful!".green().bold());
    println!("  URL:         {}", result.url.bright_cyan());
    if let Some(preview) = &result.preview_url {
        println!("  Preview:     {}", preview.bright_cyan());
    }
    println!("  ID:          {}", result.deployment_id.bright_black());
    
    Ok(result)
}

/// Get deployment status
pub async fn get_status(platform: Platform, deployment_id: &str) -> Result<DeploymentStatus> {
    match platform {
        Platform::Vercel => vercel::get_status(deployment_id).await,
        Platform::Netlify => netlify::get_status(deployment_id).await,
        Platform::Aws => aws::get_status(deployment_id).await,
        Platform::Cloudflare => cloudflare::get_status(deployment_id).await,
    }
}

/// List recent deployments
pub async fn list_deployments(platform: Platform, limit: usize) -> Result<Vec<DeployResult>> {
    match platform {
        Platform::Vercel => vercel::list_deployments(limit).await,
        Platform::Netlify => netlify::list_deployments(limit).await,
        Platform::Aws => aws::list_deployments(limit).await,
        Platform::Cloudflare => cloudflare::list_deployments(limit).await,
    }
}

/// Rollback to a previous deployment
pub async fn rollback(platform: Platform, deployment_id: &str) -> Result<()> {
    println!("{}", "⏪ Rolling back deployment...".yellow().bold());
    
    match platform {
        Platform::Vercel => vercel::rollback(deployment_id).await?,
        Platform::Netlify => netlify::rollback(deployment_id).await?,
        Platform::Aws => aws::rollback(deployment_id).await?,
        Platform::Cloudflare => cloudflare::rollback(deployment_id).await?,
    }
    
    println!("{}", "✅ Rollback complete!".green().bold());
    Ok(())
}

/// Confirm deployment with user
fn confirm_deployment() -> Result<bool> {
    use dialoguer::Confirm;
    
    Ok(Confirm::new()
        .with_prompt("Continue with deployment?")
        .default(true)
        .interact()?)
}