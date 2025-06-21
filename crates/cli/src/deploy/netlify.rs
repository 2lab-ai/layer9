//! Netlify deployment implementation

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};
use tokio::fs;

use super::{
    config::{DeployConfig, PlatformConfig},
    environment::Environment,
    status::{DeploymentLog, DeploymentStatus, ProgressTracker},
    DeployOptions, DeployResult,
};

const NETLIFY_API_URL: &str = "https://api.netlify.com/api/v1";

/// Netlify deployment configuration
struct NetlifyDeployment<'a> {
    client: Client,
    token: String,
    options: &'a DeployOptions,
    config: &'a DeployConfig,
    #[allow(dead_code)]
    platform_config: Option<&'a PlatformConfig>,
    environment: &'a Environment,
    tracker: ProgressTracker,
}

/// Deploy to Netlify
pub async fn deploy(
    options: &DeployOptions,
    config: &DeployConfig,
    environment: &Environment,
) -> Result<DeployResult> {
    let mut deployment = NetlifyDeployment::new(options, config, environment)?;
    deployment.execute().await
}

/// Get deployment status
pub async fn get_status(deployment_id: &str) -> Result<DeploymentStatus> {
    let token = std::env::var("NETLIFY_TOKEN")
        .context("NETLIFY_TOKEN environment variable not set")?;
    
    let client = Client::new();
    let url = format!("{}/deploys/{}", NETLIFY_API_URL, deployment_id);
    
    let response = client
        .get(&url)
        .bearer_auth(&token)
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to get deployment status"));
    }
    
    let deploy: NetlifyDeploy = response.json().await?;
    Ok(deploy.state.into())
}

/// List recent deployments
pub async fn list_deployments(limit: usize) -> Result<Vec<DeployResult>> {
    let token = std::env::var("NETLIFY_TOKEN")
        .context("NETLIFY_TOKEN environment variable not set")?;
    
    let client = Client::new();
    let url = format!("{}/sites?per_page={}", NETLIFY_API_URL, limit);
    
    let response = client
        .get(&url)
        .bearer_auth(&token)
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to list sites"));
    }
    
    let sites: Vec<NetlifySite> = response.json().await?;
    let mut results = Vec::new();
    
    // Get latest deploy for each site
    for site in sites {
        if let Some(deploy_url) = site.deploy_url {
            results.push(DeployResult {
                deployment_id: site.id,
                url: site.url,
                preview_url: Some(deploy_url),
                status: DeploymentStatus::Ready,
                logs: vec![],
            });
        }
    }
    
    Ok(results)
}

/// Rollback deployment
pub async fn rollback(deployment_id: &str) -> Result<()> {
    let token = std::env::var("NETLIFY_TOKEN")
        .context("NETLIFY_TOKEN environment variable not set")?;
    
    let client = Client::new();
    let url = format!("{}/deploys/{}/restore", NETLIFY_API_URL, deployment_id);
    
    let response = client
        .post(&url)
        .bearer_auth(&token)
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to rollback deployment"));
    }
    
    Ok(())
}

impl<'a> NetlifyDeployment<'a> {
    fn new(
        options: &'a DeployOptions,
        config: &'a DeployConfig,
        environment: &'a Environment,
    ) -> Result<Self> {
        // Get API token
        let platform_config = config.platform_config(super::Platform::Netlify);
        let token = if let Some(pc) = platform_config {
            if let Some(token) = &pc.token {
                if let Some(stripped) = token.strip_prefix('$') {
                    std::env::var(stripped)
                        .with_context(|| format!("Environment variable {} not set", stripped))?
                } else {
                    token.clone()
                }
            } else {
                std::env::var("NETLIFY_TOKEN")
                    .context("NETLIFY_TOKEN environment variable not set")?
            }
        } else {
            std::env::var("NETLIFY_TOKEN")
                .context("NETLIFY_TOKEN environment variable not set")?
        };
        
        let client = Client::new();
        
        Ok(Self {
            client,
            token,
            options,
            config,
            platform_config,
            environment,
            tracker: ProgressTracker::new(),
        })
    }
    
    async fn execute(&mut self) -> Result<DeployResult> {
        self.tracker.set_status(DeploymentStatus::Building);
        
        // Create or get site
        let site = self.get_or_create_site().await?;
        
        // Create deployment
        let deploy = self.create_deployment(&site).await?;
        
        // Upload files
        self.tracker.log(DeploymentLog::info("Uploading files..."));
        self.upload_files(&deploy).await?;
        
        // Finalize deployment
        self.tracker.set_status(DeploymentStatus::Deploying);
        let final_deploy = self.finalize_deployment(&deploy).await?;
        
        Ok(DeployResult {
            deployment_id: final_deploy.id,
            url: final_deploy.ssl_url.unwrap_or(final_deploy.url),
            preview_url: final_deploy.deploy_ssl_url.or(final_deploy.deploy_url),
            status: final_deploy.state.into(),
            logs: self.tracker.logs().to_vec(),
        })
    }
    
    async fn get_or_create_site(&mut self) -> Result<NetlifySite> {
        let site_name = self.config.deploy.project_name.clone()
            .unwrap_or_else(|| "layer9-app".to_string());
        
        // Try to get existing site
        let url = format!("{}/sites", NETLIFY_API_URL);
        let response = self.client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await?;
        
        if response.status().is_success() {
            let sites: Vec<NetlifySite> = response.json().await?;
            if let Some(site) = sites.into_iter().find(|s| s.name == site_name) {
                self.tracker.log(DeploymentLog::info(format!(
                    "Using existing site: {}",
                    site.name
                )));
                return Ok(site);
            }
        }
        
        // Create new site
        self.tracker.log(DeploymentLog::info("Creating new Netlify site..."));
        
        let create_request = NetlifySiteCreate {
            name: site_name.clone(),
            custom_domain: self.config.deploy.domain.clone(),
            env: self.build_env_vars(),
        };
        
        let response = self.client
            .post(&url)
            .bearer_auth(&self.token)
            .json(&create_request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Failed to create site: {}", error_text));
        }
        
        let site: NetlifySite = response.json().await?;
        self.tracker.log(DeploymentLog::info(format!(
            "Created site: {}",
            site.name
        )));
        
        Ok(site)
    }
    
    async fn create_deployment(&mut self, site: &NetlifySite) -> Result<NetlifyDeploy> {
        self.tracker.log(DeploymentLog::info("Creating deployment..."));
        
        let url = format!("{}/sites/{}/deploys", NETLIFY_API_URL, site.id);
        
        let create_request = NetlifyDeployCreate {
            draft: true, // We'll upload files then finalize
            title: format!("Layer9 deployment - {}", self.environment.name),
        };
        
        let response = self.client
            .post(&url)
            .bearer_auth(&self.token)
            .json(&create_request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Failed to create deployment: {}", error_text));
        }
        
        let deploy: NetlifyDeploy = response.json().await?;
        self.tracker.log(DeploymentLog::info(format!(
            "Created deployment: {}",
            deploy.id
        )));
        
        Ok(deploy)
    }
    
    async fn upload_files(&mut self, deploy: &NetlifyDeploy) -> Result<()> {
        // Get list of files to upload
        let files = self.collect_files().await?;
        self.tracker.log(DeploymentLog::info(format!(
            "Found {} files to upload",
            files.len()
        )));
        
        // Create file manifest
        let mut file_manifest = HashMap::new();
        for file in &files {
            let hash = sha256_hash(&file.content);
            file_manifest.insert(file.path.clone(), hash);
        }
        
        // Submit file manifest
        let url = format!("{}/deploys/{}/files", NETLIFY_API_URL, deploy.id);
        let response = self.client
            .put(&url)
            .bearer_auth(&self.token)
            .json(&NetlifyFileManifest {
                files: file_manifest.clone(),
            })
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Failed to submit file manifest: {}", error_text));
        }
        
        let required: NetlifyRequiredFiles = response.json().await?;
        
        // Upload required files
        for hash in required.required {
            if let Some(file) = files.iter().find(|f| sha256_hash(&f.content) == hash) {
                self.upload_file(&deploy.id, &file.path, &file.content).await?;
            }
        }
        
        Ok(())
    }
    
    async fn upload_file(&self, deploy_id: &str, path: &str, content: &[u8]) -> Result<()> {
        let url = format!("{}/deploys/{}/files/{}", NETLIFY_API_URL, deploy_id, path);
        
        let response = self.client
            .put(&url)
            .bearer_auth(&self.token)
            .header("Content-Type", "application/octet-stream")
            .body(content.to_vec())
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to upload file: {}", path));
        }
        
        Ok(())
    }
    
    async fn collect_files(&self) -> Result<Vec<FileInfo>> {
        let mut files = Vec::new();
        let build_dir = &self.options.build_dir;
        
        // Walk directory and collect files
        self.collect_files_recursive(build_dir, build_dir, &mut files).await?;
        
        Ok(files)
    }
    
    async fn collect_files_recursive(
        &self,
        base_dir: &Path,
        dir: &Path,
        files: &mut Vec<FileInfo>,
    ) -> Result<()> {
        let mut entries = fs::read_dir(dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let metadata = entry.metadata().await?;
            
            if metadata.is_file() {
                let relative_path = path.strip_prefix(base_dir)?;
                let content = fs::read(&path).await?;
                
                files.push(FileInfo {
                    path: relative_path.to_string_lossy().to_string(),
                    content,
                });
            } else if metadata.is_dir() {
                Box::pin(self.collect_files_recursive(base_dir, &path, files)).await?;
            }
        }
        
        Ok(())
    }
    
    async fn finalize_deployment(&mut self, deploy: &NetlifyDeploy) -> Result<NetlifyDeploy> {
        self.tracker.log(DeploymentLog::info("Finalizing deployment..."));
        
        // Update deploy to publish
        let url = format!("{}/sites/{}/deploys/{}", NETLIFY_API_URL, deploy.site_id, deploy.id);
        
        let response = self.client
            .post(&url)
            .bearer_auth(&self.token)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Failed to finalize deployment: {}", error_text));
        }
        
        // Wait for deployment to be ready
        self.wait_for_ready(&deploy.id).await
    }
    
    async fn wait_for_ready(&mut self, deploy_id: &str) -> Result<NetlifyDeploy> {
        let mut attempt = 0;
        let max_attempts = 300; // 5 minutes with 1 second intervals
        
        loop {
            attempt += 1;
            if attempt > max_attempts {
                return Err(anyhow::anyhow!("Deployment timeout"));
            }
            
            // Get deployment status
            let url = format!("{}/deploys/{}", NETLIFY_API_URL, deploy_id);
            let response = self.client
                .get(&url)
                .bearer_auth(&self.token)
                .send()
                .await?;
            
            if !response.status().is_success() {
                return Err(anyhow::anyhow!("Failed to get deployment status"));
            }
            
            let deploy: NetlifyDeploy = response.json().await?;
            let status: DeploymentStatus = deploy.state.clone().into();
            
            if status != self.tracker.status().clone() {
                self.tracker.set_status(status.clone());
            }
            
            if status.is_complete() {
                return Ok(deploy);
            }
            
            // Wait before next check
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
    
    fn build_env_vars(&self) -> HashMap<String, String> {
        self.environment.to_env_map()
    }
}

/// File information
struct FileInfo {
    path: String,
    content: Vec<u8>,
}

/// Netlify API types
#[derive(Debug, Serialize)]
struct NetlifySiteCreate {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_domain: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    env: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct NetlifySite {
    id: String,
    name: String,
    url: String,
    #[serde(default)]
    deploy_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct NetlifyDeployCreate {
    draft: bool,
    title: String,
}

#[derive(Debug, Deserialize)]
struct NetlifyDeploy {
    id: String,
    site_id: String,
    state: NetlifyDeployState,
    url: String,
    #[serde(default)]
    ssl_url: Option<String>,
    #[serde(default)]
    deploy_url: Option<String>,
    #[serde(default)]
    deploy_ssl_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
enum NetlifyDeployState {
    New,
    Building,
    Processing,
    Ready,
    Error,
}

impl From<NetlifyDeployState> for DeploymentStatus {
    fn from(state: NetlifyDeployState) -> Self {
        match state {
            NetlifyDeployState::New => DeploymentStatus::Queued,
            NetlifyDeployState::Building => DeploymentStatus::Building,
            NetlifyDeployState::Processing => DeploymentStatus::Deploying,
            NetlifyDeployState::Ready => DeploymentStatus::Ready,
            NetlifyDeployState::Error => DeploymentStatus::Failed,
        }
    }
}

#[derive(Debug, Serialize)]
struct NetlifyFileManifest {
    files: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct NetlifyRequiredFiles {
    required: Vec<String>,
}

/// Calculate SHA-256 hash of content
fn sha256_hash(content: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}