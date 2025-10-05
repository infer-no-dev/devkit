//! Core plugin types and traits

use crate::agents::{Agent, AgentError, AgentResult, AgentTask};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Core plugin trait - extends Agent with plugin-specific functionality
#[async_trait::async_trait]
pub trait Plugin: Agent + Send + Sync {
    /// Plugin metadata
    fn metadata(&self) -> &PluginMetadata;
    
    /// Initialize the plugin
    async fn initialize(&mut self) -> Result<(), PluginError>;
    
    /// Activate the plugin
    async fn activate(&mut self) -> Result<(), PluginError>;
    
    /// Deactivate the plugin
    async fn deactivate(&mut self) -> Result<(), PluginError>;
    
    /// Handle plugin events
    async fn handle_event(&mut self, event: PluginEvent) -> Result<(), PluginError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub api_version: String,
    pub dependencies: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum PluginState {
    Unloaded,
    Loading,
    Loaded,
    Active,
    Inactive, 
    Error(String),
}

#[derive(Debug, Clone)]
pub enum PluginEvent {
    SystemStartup,
    SystemShutdown,
    TaskAssigned(AgentTask),
    ConfigChanged,
    Custom(String, serde_json::Value),
}

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Load failed: {0}")]
    LoadFailed(String),
    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Agent error: {0}")]
    AgentError(#[from] AgentError),
}

pub type PluginHandle = Arc<dyn Plugin>;
pub type PluginInfo = PluginMetadata;
pub type PluginStatus = PluginState;