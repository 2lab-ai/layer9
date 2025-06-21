//! Vercel deployment implementation

use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};
use tokio::fs;

use super::{
    config::{DeployConfig, PlatformConfig},
    environment::Environment,
    status::{DeploymentLog, DeploymentStatus, ProgressTracker},
    DeployOptions, DeployResult,
};

const VERCEL_API_URL: &str = "https://api.vercel.com";
#[allow(dead_code)]
const VERCEL_API_VERSION: &str = "v13";

/// Vercel deployment configuration
struct VercelDeployment<'a> {
    client: Client,
    token: String,
    options: &'a DeployOptions,
    config: &'a DeployConfig,
    #[allow(dead_code)]
    platform_config: Option<&'a PlatformConfig>,
    environment: &'a Environment,
    tracker: ProgressTracker,
}

/// Deploy to Vercel
pub async fn deploy(
    options: &DeployOptions,
    config: &DeployConfig,
    environment: &Environment,
) -> Result<DeployResult> {
    let mut deployment = VercelDeployment::new(options, config, environment)?;
    deployment.execute().await
}

/// Get deployment status
pub async fn get_status(deployment_id: &str) -> Result<DeploymentStatus> {
    let token = std::env::var("VERCEL_TOKEN")
        .context("VERCEL_TOKEN environment variable not set")?;
    
    let client = Client::new();
    let url = format!("{}/v13/deployments/{}", VERCEL_API_URL, deployment_id);
    
    let response = client
        .get(&url)
        .bearer_auth(&token)
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to get deployment status"));
    }
    
    let deployment: VercelDeploymentResponse = response.json().await?;
    Ok(deployment.ready_state.into())
}

/// List recent deployments
pub async fn list_deployments(limit: usize) -> Result<Vec<DeployResult>> {
    let token = std::env::var("VERCEL_TOKEN")
        .context("VERCEL_TOKEN environment variable not set")?;
    
    let client = Client::new();
    let url = format!("{}/v6/deployments?limit={}", VERCEL_API_URL, limit);
    
    let response = client
        .get(&url)
        .bearer_auth(&token)
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to list deployments"));
    }
    
    let list: VercelDeploymentList = response.json().await?;
    
    Ok(list.deployments.into_iter().map(|d| DeployResult {
        deployment_id: d.uid,
        url: d.url.unwrap_or_default(),
        preview_url: None,
        status: d.ready_state.into(),
        logs: vec![],
    }).collect())
}

/// Rollback deployment (promote previous deployment)
pub async fn rollback(deployment_id: &str) -> Result<()> {
    // Vercel doesn't have direct rollback, but we can promote a previous deployment
    // This would typically be done through the Vercel dashboard
    Err(anyhow::anyhow!(
        "Vercel rollback must be done through the dashboard. Visit: https://vercel.com/deployments/{}",
        deployment_id
    ))
}

impl<'a> VercelDeployment<'a> {
    fn new(
        options: &'a DeployOptions,
        config: &'a DeployConfig,
        environment: &'a Environment,
    ) -> Result<Self> {
        // Get API token
        let platform_config = config.platform_config(super::Platform::Vercel);
        let token = if let Some(pc) = platform_config {
            if let Some(token) = &pc.token {
                if let Some(stripped) = token.strip_prefix('$') {
                    std::env::var(stripped)
                        .with_context(|| format!("Environment variable {} not set", stripped))?
                } else {
                    token.clone()
                }
            } else {
                std::env::var("VERCEL_TOKEN")
                    .context("VERCEL_TOKEN environment variable not set")?
            }
        } else {
            std::env::var("VERCEL_TOKEN")
                .context("VERCEL_TOKEN environment variable not set")?
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
        
        // Create deployment
        let deployment = self.create_deployment().await?;
        
        // Upload files
        self.tracker.log(DeploymentLog::info("Uploading files..."));
        self.upload_files(&deployment).await?;
        
        // Wait for deployment to be ready
        self.tracker.set_status(DeploymentStatus::Deploying);
        let final_deployment = self.wait_for_ready(&deployment.id).await?;
        
        Ok(DeployResult {
            deployment_id: final_deployment.uid,
            url: final_deployment.url.unwrap_or_default(),
            preview_url: final_deployment.alias.first().cloned(),
            status: final_deployment.ready_state.into(),
            logs: self.tracker.logs().to_vec(),
        })
    }
    
    async fn create_deployment(&mut self) -> Result<VercelDeploymentResponse> {
        self.tracker.log(DeploymentLog::info("Creating Vercel deployment..."));
        
        // Build deployment request
        let mut request = VercelDeploymentRequest {
            name: self.config.deploy.project_name.clone()
                .unwrap_or_else(|| "layer9-app".to_string()),
            project_id: self.platform_config.and_then(|c| c.project_id.clone()),
            target: Some(match self.environment.name.as_str() {
                "production" => "production",
                _ => "preview",
            }.to_string()),
            git_source: None,
            functions: None,
            routes: None,
            regions: vec!["sfo1".to_string()], // Default to San Francisco
            env: self.build_env_vars(),
            build_env: self.build_env_vars(),
            framework: self.platform_config
                .and_then(|c| c.framework.clone())
                .or_else(|| Some("vanilla".to_string())),
        };
        
        // Set custom domain if configured
        if let Some(domain) = &self.config.deploy.domain {
            request.env.insert("CUSTOM_DOMAIN".to_string(), domain.clone());
        }
        
        let url = format!("{}/v13/deployments", VERCEL_API_URL);
        let response = self.client
            .post(&url)
            .bearer_auth(&self.token)
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Failed to create deployment: {}", error_text));
        }
        
        let deployment: VercelDeploymentResponse = response.json().await?;
        self.tracker.log(DeploymentLog::info(format!(
            "Created deployment: {}",
            deployment.id
        )));
        
        Ok(deployment)
    }
    
    async fn upload_files(&mut self, deployment: &VercelDeploymentResponse) -> Result<()> {
        // Get list of files to upload
        let files = self.collect_files().await?;
        self.tracker.log(DeploymentLog::info(format!(
            "Found {} files to upload",
            files.len()
        )));
        
        // Create file tree
        let file_tree = self.create_file_tree(&files).await?;
        
        // Upload file tree
        let url = format!("{}/v2/deployments/{}/files", VERCEL_API_URL, deployment.id);
        let response = self.client
            .post(&url)
            .bearer_auth(&self.token)
            .json(&file_tree)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Failed to upload files: {}", error_text));
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
                    mode: if is_executable(&metadata) { 0o755 } else { 0o644 },
                });
            } else if metadata.is_dir() {
                Box::pin(self.collect_files_recursive(base_dir, &path, files)).await?;
            }
        }
        
        Ok(())
    }
    
    async fn create_file_tree(&self, files: &[FileInfo]) -> Result<Vec<VercelFile>> {
        let mut tree = Vec::new();
        
        for file in files {
            // Calculate SHA-1 hash
            let hash = sha1_hash(&file.content);
            
            // Upload file content
            let upload_url = format!("{}/v2/files/{}", VERCEL_API_URL, hash);
            let response = self.client
                .post(&upload_url)
                .bearer_auth(&self.token)
                .header("x-vercel-digest", &hash)
                .body(file.content.clone())
                .send()
                .await?;
            
            if !response.status().is_success() && response.status() != StatusCode::CONFLICT {
                return Err(anyhow::anyhow!("Failed to upload file: {}", file.path));
            }
            
            tree.push(VercelFile {
                file: file.path.clone(),
                sha: hash,
                size: file.content.len(),
            });
        }
        
        Ok(tree)
    }
    
    async fn wait_for_ready(&mut self, deployment_id: &str) -> Result<VercelDeploymentResponse> {
        let mut attempt = 0;
        let max_attempts = 300; // 5 minutes with 1 second intervals
        
        loop {
            attempt += 1;
            if attempt > max_attempts {
                return Err(anyhow::anyhow!("Deployment timeout"));
            }
            
            // Get deployment status
            let url = format!("{}/v13/deployments/{}", VERCEL_API_URL, deployment_id);
            let response = self.client
                .get(&url)
                .bearer_auth(&self.token)
                .send()
                .await?;
            
            if !response.status().is_success() {
                return Err(anyhow::anyhow!("Failed to get deployment status"));
            }
            
            let deployment: VercelDeploymentResponse = response.json().await?;
            let status: DeploymentStatus = deployment.ready_state.clone().into();
            
            if status != self.tracker.status().clone() {
                self.tracker.set_status(status.clone());
            }
            
            if status.is_complete() {
                return Ok(deployment);
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
    #[allow(dead_code)]
    mode: u32,
}

/// Vercel API types
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct VercelDeploymentRequest {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    git_source: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    functions: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    routes: Option<Vec<serde_json::Value>>,
    regions: Vec<String>,
    env: HashMap<String, String>,
    build_env: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    framework: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VercelDeploymentResponse {
    id: String,
    uid: String,
    #[allow(dead_code)]
    name: String,
    url: Option<String>,
    #[allow(dead_code)]
    created: i64,
    ready_state: VercelReadyState,
    #[serde(default)]
    alias: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum VercelReadyState {
    Queued,
    Building,
    Initializing,
    Deploying,
    Ready,
    Error,
    Canceled,
}

impl From<VercelReadyState> for DeploymentStatus {
    fn from(state: VercelReadyState) -> Self {
        match state {
            VercelReadyState::Queued => DeploymentStatus::Queued,
            VercelReadyState::Building | VercelReadyState::Initializing => DeploymentStatus::Building,
            VercelReadyState::Deploying => DeploymentStatus::Deploying,
            VercelReadyState::Ready => DeploymentStatus::Ready,
            VercelReadyState::Error => DeploymentStatus::Failed,
            VercelReadyState::Canceled => DeploymentStatus::Cancelled,
        }
    }
}

#[derive(Debug, Serialize)]
struct VercelFile {
    file: String,
    sha: String,
    size: usize,
}

#[derive(Debug, Deserialize)]
struct VercelDeploymentList {
    deployments: Vec<VercelDeploymentResponse>,
}

/// Calculate SHA-256 hash of content
fn sha1_hash(content: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

/// Check if file is executable
fn is_executable(metadata: &std::fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        metadata.mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        false
    }
}