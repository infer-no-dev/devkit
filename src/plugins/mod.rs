//! Plugin System
//!
//! This module provides a comprehensive plugin architecture for extending DevKit functionality.
//! Plugins are dynamically loaded agents with enhanced capabilities, metadata, and lifecycle management.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Plugin System                            │
//! ├─────────────────┬─────────────────┬─────────────────────────┤
//! │  Plugin Manager │  Plugin Registry│   Plugin Loader         │
//! │                 │                 │                         │
//! │ • Lifecycle     │ • Discovery     │ • Dynamic Loading       │
//! │ • Dependencies  │ • Versioning    │ • Safety Checks         │
//! │ • Sandboxing    │ • Metadata      │ • Hot Reload            │
//! └─────────────────┴─────────────────┴─────────────────────────┘
//! ```

// pub mod agent;
pub mod loader;
pub mod manager;
// pub mod manifest;
pub mod marketplace;
// pub mod registry;
// pub mod sandbox;
pub mod types;

// Re-export core types
// pub use agent::{PluginAgent, PluginWrapper};
pub use loader::{PluginLoader, PluginLoaderFactory, PluginType};
pub use manager::PluginManager;
// pub use manifest::{PluginManifest, PluginMetadata};
pub use marketplace::{MarketplaceClient, MarketplaceConfig, MarketplacePlugin};
// pub use registry::PluginRegistry;
pub use types::{
    Plugin, PluginError, PluginEvent, PluginHandle, PluginInfo, PluginState, PluginStatus,
    PluginMetadata, PluginCapability, PluginHealth, PluginDependency,
};

use crate::agents::{Agent, AgentError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Plugin system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSystemConfig {
    /// Directory where plugins are installed
    pub plugin_dir: PathBuf,
    /// Whether to enable hot reloading
    pub hot_reload: bool,
    /// Maximum number of plugins that can be loaded simultaneously
    pub max_plugins: usize,
    /// Plugin execution timeout in seconds
    pub execution_timeout: u64,
    /// Enable sandboxing for plugins
    pub enable_sandbox: bool,
    /// Allowed plugin permissions
    pub default_permissions: Vec<String>,
    /// Plugin registry URLs
    pub registry_urls: Vec<String>,
}

impl Default for PluginSystemConfig {
    fn default() -> Self {
        Self {
            plugin_dir: PathBuf::from("plugins"),
            hot_reload: true,
            max_plugins: 50,
            execution_timeout: 300,
            enable_sandbox: true,
            default_permissions: vec![
                "filesystem:read".to_string(),
                "network:http".to_string(),
            ],
            registry_urls: vec!["https://plugins.devkit.dev".to_string()],
        }
    }
}

/// Plugin API version for compatibility checking
pub const PLUGIN_API_VERSION: &str = "1.0.0";

/// Initialize the plugin system
pub async fn init_plugin_system(_config: PluginSystemConfig) -> Result<(), PluginError> {
    // TODO: Implement when PluginManager is available
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_system_config_default() {
        let config = PluginSystemConfig::default();
        assert_eq!(config.plugin_dir, PathBuf::from("plugins"));
        assert!(config.hot_reload);
        assert_eq!(config.max_plugins, 50);
    }

    #[tokio::test]
    async fn test_plugin_system_init() {
        let config = PluginSystemConfig::default();
        let result = init_plugin_system(config).await;
        // For now, just check that init succeeds
        assert!(result.is_ok());
    }
}