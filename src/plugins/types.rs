//! Plugin System Types
//!
//! Core types and traits for the plugin system, defining the plugin lifecycle,
//! capabilities, and communication interfaces.

use crate::agents::{Agent, AgentError, AgentResult, AgentTask};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::time::Instant;

/// Core plugin trait that all plugins must implement
#[async_trait::async_trait]
pub trait Plugin: Send + Sync {
    /// Get plugin unique identifier
    fn id(&self) -> &str;
    
    /// Get plugin name
    fn name(&self) -> &str;
    
    /// Get plugin version
    fn version(&self) -> &str;
    
    /// Initialize the plugin
    async fn initialize(&mut self) -> Result<(), PluginError>;
    
    /// Execute plugin with input data
    async fn execute(&mut self, input: &str) -> Result<String, PluginError>;
    
    /// Shutdown the plugin
    async fn shutdown(&mut self) -> Result<(), PluginError>;
    
    /// Get plugin capabilities
    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![]
    }
    
    /// Handle plugin configuration updates
    async fn configure(&mut self, _config: HashMap<String, serde_json::Value>) -> Result<(), PluginError> {
        Ok(())
    }
    
    /// Get plugin health status
    async fn health_check(&self) -> Result<PluginHealth, PluginError> {
        Ok(PluginHealth::Healthy)
    }
}

/// Plugin capabilities that can be advertised
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum PluginCapability {
    /// Can analyze code
    CodeAnalysis,
    /// Can generate code
    CodeGeneration,
    /// Can format code
    CodeFormatting,
    /// Can provide completions
    Completion,
    /// Can provide diagnostics
    Diagnostics,
    /// Can integrate with version control
    VersionControl,
    /// Can manage dependencies
    DependencyManagement,
    /// Can run tests
    Testing,
    /// Can provide documentation
    Documentation,
    /// Custom capability
    #[serde(untagged)]
    Custom(String),
}

/// Plugin health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginHealth {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

/// Plugin error types
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Plugin load failed: {0}")]
    LoadFailed(String),
    
    #[error("Plugin execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Plugin not found: {0}")]
    NotFound(String),
    
    #[error("Plugin dependency not found: {0}")]
    DependencyNotFound(String),
    
    #[error("Plugin permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Plugin configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("IO Error: {0}")]
    IoError(String),
    
    #[error("Invalid plugin manifest: {0}")]
    InvalidManifest(String),
    
    #[error("Plugin has dependents: {0:?}")]
    HasDependents(HashSet<String>),
    
    #[error("Plugin initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("Plugin communication error: {0}")]
    CommunicationError(String),
    
    #[error("Plugin security violation: {0}")]
    SecurityViolation(String),
    
    #[error("Plugin resource exhausted: {0}")]
    ResourceExhausted(String),
}

/// Plugin metadata and runtime information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub status: PluginStatus,
    pub capabilities: Vec<PluginCapability>,
    pub health: PluginHealth,
    pub load_time: Option<std::time::Duration>,
}

/// Plugin runtime status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginStatus {
    Loading,
    Active,
    Running,
    Stopped,
    Error,
    Disabled,
    Uninstalling,
}

/// Plugin state management
#[derive(Debug, Clone)]
pub enum PluginState {
    Unloaded,
    Loading,
    Loaded,
    Running,
    Stopped,
    Error(String),
    Disabled,
}

/// Plugin handle for management
pub struct PluginHandle {
    pub id: String,
    pub plugin: Box<dyn Plugin>,
    pub state: PluginState,
    pub metadata: PluginMetadata,
    pub load_time: Instant,
    pub config: HashMap<String, serde_json::Value>,
}

impl Clone for PluginHandle {
    fn clone(&self) -> Self {
        // Note: This is a simplified clone that doesn't actually clone the plugin
        // In a real implementation, you'd need a different approach for plugin cloning
        use crate::plugins::loader::NativePluginProxy;
        Self {
            id: self.id.clone(),
            plugin: Box::new(NativePluginProxy::new(
                self.metadata.clone(),
                std::path::PathBuf::from(&self.metadata.entry_point)
            )),
            state: self.state.clone(),
            metadata: self.metadata.clone(),
            load_time: self.load_time,
            config: self.config.clone(),
        }
    }
}

impl std::fmt::Debug for PluginHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginHandle")
            .field("id", &self.id)
            .field("plugin", &"<Plugin trait object>")
            .field("state", &self.state)
            .field("metadata", &self.metadata)
            .field("load_time", &self.load_time)
            .field("config", &self.config)
            .finish()
    }
}

/// Plugin system events
#[derive(Debug, Clone)]
pub enum PluginEvent {
    PluginLoaded { plugin_id: String, version: String },
    PluginUnloaded { plugin_id: String },
    PluginError { plugin_id: String, error: String },
    PluginStateChanged { plugin_id: String, new_state: PluginStatus },
    PluginConfigUpdated { plugin_id: String },
    PluginHealthChanged { plugin_id: String, health: PluginHealth },
    DependencyResolved { plugin_id: String, dependency_id: String },
    SystemEvent { event_type: String, data: serde_json::Value },
}

/// Plugin metadata (basic version here, more detailed in manager)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: String,
    pub tags: Vec<String>,
    pub dependencies: Vec<PluginDependency>,
    pub permissions: Vec<String>,
    pub entry_point: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub capabilities: Vec<PluginCapability>,
}

/// Plugin dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub id: String,
    pub version: String, // semver requirement
    pub optional: bool,
    pub reason: Option<String>,
}

/// Plugin execution context
#[derive(Debug, Clone)]
pub struct PluginContext {
    pub plugin_id: String,
    pub permissions: HashSet<String>,
    pub config: HashMap<String, serde_json::Value>,
    pub environment: HashMap<String, String>,
    pub working_directory: PathBuf,
    pub temp_directory: PathBuf,
}

/// Plugin execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub execution_time: std::time::Duration,
}

/// Plugin communication message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMessage {
    pub id: String,
    pub source: String,
    pub target: String,
    pub message_type: String,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Plugin registry entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRegistryEntry {
    pub metadata: PluginMetadata,
    pub installation_info: Option<InstallationInfo>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub checksum: Option<String>,
}

/// Plugin installation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationInfo {
    pub plugin_id: String,
    pub version: String,
    pub installed_at: chrono::DateTime<chrono::Utc>,
    pub install_path: PathBuf,
    pub checksum: String,
    pub auto_update: bool,
    pub enabled: bool,
    pub installation_source: InstallationSource,
}

/// Where the plugin was installed from
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallationSource {
    Marketplace { registry: String },
    LocalFile { path: PathBuf },
    Git { url: String, branch: Option<String> },
    Custom { source: String },
}

/// Plugin permission types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PluginPermission {
    /// Read file system
    FileSystemRead,
    /// Write file system
    FileSystemWrite,
    /// Execute commands
    ProcessExecution,
    /// Network access
    NetworkAccess,
    /// Environment variable access
    EnvironmentAccess,
    /// System information access
    SystemInfo,
    /// Custom permission
    Custom(String),
}