//! Deployment status tracking

use chrono::{DateTime, Utc};
use colored::*;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Deployment status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeploymentStatus {
    /// Deployment is queued
    Queued,
    /// Build is in progress
    Building,
    /// Deployment is in progress
    Deploying,
    /// Deployment completed successfully
    Ready,
    /// Deployment failed
    Failed,
    /// Deployment was cancelled
    Cancelled,
}

impl DeploymentStatus {
    /// Check if deployment is complete (success or failure)
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Ready | Self::Failed | Self::Cancelled)
    }
    
    /// Check if deployment was successful
    #[allow(dead_code)]
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Ready)
    }
    
    /// Get color for status display
    pub fn color(&self) -> colored::Color {
        match self {
            Self::Queued => Color::Yellow,
            Self::Building => Color::Blue,
            Self::Deploying => Color::Cyan,
            Self::Ready => Color::Green,
            Self::Failed => Color::Red,
            Self::Cancelled => Color::BrightBlack,
        }
    }
}

impl fmt::Display for DeploymentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = match self {
            Self::Queued => "Queued",
            Self::Building => "Building",
            Self::Deploying => "Deploying",
            Self::Ready => "Ready",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
        };
        write!(f, "{}", status.color(self.color()))
    }
}

/// Deployment log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentLog {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Log level
    pub level: LogLevel,
    
    /// Log message
    pub message: String,
    
    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Log level
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl LogLevel {
    pub fn color(&self) -> Color {
        match self {
            Self::Debug => Color::BrightBlack,
            Self::Info => Color::White,
            Self::Warning => Color::Yellow,
            Self::Error => Color::Red,
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level = match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warning => "WARN",
            Self::Error => "ERROR",
        };
        write!(f, "{}", level.color(self.color()))
    }
}

impl DeploymentLog {
    /// Create a new log entry
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            message: message.into(),
            metadata: None,
        }
    }
    
    /// Create debug log
    #[allow(dead_code)]
    pub fn debug(message: impl Into<String>) -> Self {
        Self::new(LogLevel::Debug, message)
    }
    
    /// Create info log
    pub fn info(message: impl Into<String>) -> Self {
        Self::new(LogLevel::Info, message)
    }
    
    /// Create warning log
    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(LogLevel::Warning, message)
    }
    
    /// Create error log
    #[allow(dead_code)]
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(LogLevel::Error, message)
    }
    
    /// Add metadata to the log
    #[allow(dead_code)]
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
    
    /// Format log for display
    pub fn format(&self) -> String {
        format!(
            "{} {} {}",
            self.timestamp.format("%H:%M:%S").to_string().bright_black(),
            self.level,
            self.message
        )
    }
}

/// Deployment progress tracker
pub struct ProgressTracker {
    logs: Vec<DeploymentLog>,
    status: DeploymentStatus,
    #[allow(dead_code)]
    start_time: DateTime<Utc>,
}

impl ProgressTracker {
    /// Create new progress tracker
    pub fn new() -> Self {
        Self {
            logs: Vec::new(),
            status: DeploymentStatus::Queued,
            start_time: Utc::now(),
        }
    }
    
    /// Update status
    pub fn set_status(&mut self, status: DeploymentStatus) {
        let status_msg = format!("Status: {}", status);
        self.status = status;
        self.logs.push(DeploymentLog::info(status_msg));
    }
    
    /// Add log entry
    pub fn log(&mut self, log: DeploymentLog) {
        if log.level != LogLevel::Debug {
            println!("{}", log.format());
        }
        self.logs.push(log);
    }
    
    /// Get elapsed time
    #[allow(dead_code)]
    pub fn elapsed(&self) -> chrono::Duration {
        Utc::now() - self.start_time
    }
    
    /// Get all logs
    pub fn logs(&self) -> &[DeploymentLog] {
        &self.logs
    }
    
    /// Get current status
    pub fn status(&self) -> &DeploymentStatus {
        &self.status
    }
}