//! Plugin Manager
//!
//! Manages the lifecycle of plugins including loading, unloading, dependency resolution,
//! and orchestration between different plugins.

use crate::plugins::{
    PluginError, PluginEvent, PluginHandle, PluginInfo, PluginState, PluginSystemConfig,
    PluginLoader, PluginMetadata, PluginStatus, PluginHealth,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::sync::{broadcast, watch};
use tokio::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Plugin Manager responsible for plugin lifecycle management
pub struct PluginManager {
    /// System configuration
    config: PluginSystemConfig,
    /// Loaded plugins indexed by ID
    plugins: Arc<RwLock<HashMap<String, PluginHandle>>>,
    /// Plugin dependency graph
    dependency_graph: Arc<RwLock<DependencyGraph>>,
    /// Event broadcaster for plugin events
    event_sender: broadcast::Sender<PluginEvent>,
    /// Shutdown signal receiver
    shutdown_receiver: watch::Receiver<bool>,
    /// Plugin loader instance
    loader: Arc<dyn PluginLoader + Send + Sync>,
    /// Plugin registry for metadata
    registry: Arc<RwLock<PluginRegistry>>,
    /// Runtime statistics
    stats: Arc<RwLock<PluginManagerStats>>,
}

/// Plugin dependency graph for resolution and ordering
#[derive(Debug, Default)]
struct DependencyGraph {
    /// Dependencies: plugin_id -> set of required plugin_ids
    dependencies: HashMap<String, HashSet<String>>,
    /// Reverse dependencies: plugin_id -> set of dependent plugin_ids
    dependents: HashMap<String, HashSet<String>>,
}

/// Plugin registry for local metadata management
#[derive(Debug, Default)]
pub struct PluginRegistry {
    /// Plugin metadata indexed by ID
    metadata: HashMap<String, PluginMetadata>,
    /// Version history for each plugin
    versions: HashMap<String, Vec<String>>,
    /// Installation information
    installations: HashMap<String, InstallationInfo>,
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
}

/// Plugin manager runtime statistics
#[derive(Debug, Default)]
pub struct PluginManagerStats {
    pub total_plugins: usize,
    pub loaded_plugins: usize,
    pub failed_plugins: usize,
    pub total_events: u64,
    pub startup_time: Option<Duration>,
    pub last_scan_time: Option<Instant>,
}


impl PluginManager {
    /// Create a new plugin manager
    pub async fn new(config: PluginSystemConfig) -> Result<Self, PluginError> {
        let (event_sender, _) = broadcast::channel(1000);
        let (shutdown_sender, shutdown_receiver) = watch::channel(false);

        // Create plugin directory if it doesn't exist
        if !config.plugin_dir.exists() {
            tokio::fs::create_dir_all(&config.plugin_dir)
                .await
                .map_err(|e| PluginError::IoError(e.to_string()))?;
        }

        // Initialize loader based on configuration
        let loader = Self::create_loader(&config)?;

        Ok(Self {
            config,
            plugins: Arc::new(RwLock::new(HashMap::new())),
            dependency_graph: Arc::new(RwLock::new(DependencyGraph::default())),
            event_sender,
            shutdown_receiver,
            loader,
            registry: Arc::new(RwLock::new(PluginRegistry::default())),
            stats: Arc::new(RwLock::new(PluginManagerStats::default())),
        })
    }

    /// Create appropriate plugin loader based on configuration
    fn create_loader(config: &PluginSystemConfig) -> Result<Arc<dyn PluginLoader + Send + Sync>, PluginError> {
        // Create a default native library loader
        // In a real implementation, this could be dynamic based on available plugins
        use crate::plugins::loader::NativePluginLoader;
        Ok(Arc::new(NativePluginLoader::new(config.clone())))
    }

    /// Scan for plugins and load them
    pub async fn scan_and_load_plugins(&self) -> Result<(), PluginError> {
        let start_time = Instant::now();
        info!("Scanning for plugins in: {:?}", self.config.plugin_dir);

        let mut plugins_found = 0;
        let mut plugins_loaded = 0;
        let mut plugins_failed = 0;

        // Scan plugin directory recursively
        let mut entries = tokio::fs::read_dir(&self.config.plugin_dir)
            .await
            .map_err(|e| PluginError::IoError(e.to_string()))?;

        while let Some(entry) = entries.next_entry()
            .await
            .map_err(|e| PluginError::IoError(e.to_string()))? 
        {
            let path = entry.path();
            if path.is_dir() {
                match self.scan_plugin_directory(&path).await {
                    Ok(count) => {
                        plugins_found += count;
                    }
                    Err(e) => {
                        warn!("Failed to scan plugin directory {:?}: {}", path, e);
                    }
                }
            }
        }

        // Load discovered plugins
        let plugin_paths = self.get_discovered_plugins().await?;
        for (path, metadata) in plugin_paths {
            match self.load_plugin_from_path(&path, &metadata).await {
                Ok(_) => {
                    plugins_loaded += 1;
                    info!("Successfully loaded plugin: {}", metadata.id);
                }
                Err(e) => {
                    plugins_failed += 1;
                    error!("Failed to load plugin {:?}: {}", path, e);
                }
            }
        }

        // Update statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_plugins = plugins_found;
            stats.loaded_plugins = plugins_loaded;
            stats.failed_plugins = plugins_failed;
            stats.startup_time = Some(start_time.elapsed());
            stats.last_scan_time = Some(Instant::now());
        }

        info!(
            "Plugin scan complete: {} found, {} loaded, {} failed in {:?}",
            plugins_found, plugins_loaded, plugins_failed, start_time.elapsed()
        );

        Ok(())
    }

    /// Scan a specific plugin directory
    async fn scan_plugin_directory(&self, dir: &Path) -> Result<usize, PluginError> {
        debug!("Scanning plugin directory: {:?}", dir);

        // Look for plugin manifest
        let manifest_path = dir.join("plugin.toml");
        if !manifest_path.exists() {
            return Ok(0);
        }

        // Parse plugin manifest
        let manifest_content = tokio::fs::read_to_string(&manifest_path)
            .await
            .map_err(|e| PluginError::IoError(e.to_string()))?;

        let metadata: PluginMetadata = toml::from_str(&manifest_content)
            .map_err(|e| PluginError::InvalidManifest(e.to_string()))?;

        // Register plugin metadata
        {
            let mut registry = self.registry.write().unwrap();
            registry.metadata.insert(metadata.id.clone(), metadata);
        }

        Ok(1)
    }

    /// Get list of discovered plugins ready for loading
    async fn get_discovered_plugins(&self) -> Result<Vec<(PathBuf, PluginMetadata)>, PluginError> {
        let registry = self.registry.read().unwrap();
        let mut plugins = Vec::new();

        for (plugin_id, metadata) in &registry.metadata {
            let plugin_dir = self.config.plugin_dir.join(plugin_id);
            if plugin_dir.exists() {
                plugins.push((plugin_dir, metadata.clone()));
            }
        }

        Ok(plugins)
    }

    /// Load a plugin from a specific path
    async fn load_plugin_from_path(&self, path: &Path, metadata: &PluginMetadata) -> Result<(), PluginError> {
        debug!("Loading plugin from: {:?}", path);

        // Check dependencies first
        self.resolve_dependencies(metadata).await?;

        // Load the plugin
        let plugin = self.loader.load_plugin(path, metadata).await?;

        // Create plugin handle
        let handle = PluginHandle {
            id: metadata.id.clone(),
            plugin,
            state: PluginState::Loaded,
            metadata: metadata.clone(),
            load_time: Instant::now(),
            config: HashMap::new(),
        };

        // Store plugin
        {
            let mut plugins = self.plugins.write().unwrap();
            plugins.insert(metadata.id.clone(), handle);
        }

        // Update dependency graph
        self.update_dependency_graph(metadata).await;

        // Broadcast plugin loaded event
        let _ = self.event_sender.send(PluginEvent::PluginLoaded {
            plugin_id: metadata.id.clone(),
            version: metadata.version.clone(),
        });

        Ok(())
    }

    /// Resolve plugin dependencies
    async fn resolve_dependencies(&self, metadata: &PluginMetadata) -> Result<(), PluginError> {
        for dep in &metadata.dependencies {
            if dep.optional {
                continue;
            }

            let plugins = self.plugins.read().unwrap();
            if !plugins.contains_key(&dep.id) {
                return Err(PluginError::DependencyNotFound(dep.id.clone()));
            }

            // TODO: Check version compatibility
        }

        Ok(())
    }

    /// Update dependency graph with new plugin
    async fn update_dependency_graph(&self, metadata: &PluginMetadata) {
        let mut graph = self.dependency_graph.write().unwrap();
        
        let plugin_deps: HashSet<String> = metadata.dependencies
            .iter()
            .map(|d| d.id.clone())
            .collect();
        
        graph.dependencies.insert(metadata.id.clone(), plugin_deps);

        // Update reverse dependencies
        for dep in &metadata.dependencies {
            graph.dependents
                .entry(dep.id.clone())
                .or_insert_with(HashSet::new)
                .insert(metadata.id.clone());
        }
    }

    /// Get loaded plugin by ID
    pub async fn get_plugin(&self, plugin_id: &str) -> Option<PluginHandle> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(plugin_id).cloned()
    }

    /// List all loaded plugins
    pub async fn list_plugins(&self) -> Vec<PluginInfo> {
        let plugins = self.plugins.read().unwrap();
        plugins.values()
            .map(|handle| PluginInfo {
                id: handle.id.clone(),
                name: handle.metadata.name.clone(),
                version: handle.metadata.version.clone(),
                status: match handle.state {
                    PluginState::Loaded => PluginStatus::Active,
                    PluginState::Running => PluginStatus::Running,
                    PluginState::Stopped => PluginStatus::Stopped,
                    PluginState::Error(_) => PluginStatus::Error,
                    _ => PluginStatus::Loading,
                },
                description: handle.metadata.description.clone(),
                author: handle.metadata.author.clone(),
                capabilities: handle.metadata.capabilities.clone(),
                health: PluginHealth::Healthy,
                load_time: Some(handle.load_time.elapsed()),
            })
            .collect()
    }

    /// Unload a plugin
    pub async fn unload_plugin(&self, plugin_id: &str) -> Result<(), PluginError> {
        // Check for dependents first
        {
            let graph = self.dependency_graph.read().unwrap();
            if let Some(dependents) = graph.dependents.get(plugin_id) {
                if !dependents.is_empty() {
                    return Err(PluginError::HasDependents(dependents.clone()));
                }
            }
        }

        // Remove from loaded plugins
        let handle = {
            let mut plugins = self.plugins.write().unwrap();
            plugins.remove(plugin_id)
        };

        if let Some(handle) = handle {
            // Unload through loader
            self.loader.unload_plugin(plugin_id).await?;

            // Broadcast plugin unloaded event
            let _ = self.event_sender.send(PluginEvent::PluginUnloaded {
                plugin_id: plugin_id.to_string(),
            });

            info!("Plugin '{}' unloaded successfully", plugin_id);
        }

        Ok(())
    }

    /// Get plugin manager statistics
    pub async fn get_stats(&self) -> PluginManagerStats {
        let stats = self.stats.read().unwrap();
        PluginManagerStats {
            total_plugins: stats.total_plugins,
            loaded_plugins: stats.loaded_plugins,
            failed_plugins: stats.failed_plugins,
            total_events: stats.total_events,
            startup_time: stats.startup_time,
            last_scan_time: stats.last_scan_time,
        }
    }

    /// Subscribe to plugin events
    pub fn subscribe_events(&self) -> broadcast::Receiver<PluginEvent> {
        self.event_sender.subscribe()
    }

    /// Shutdown the plugin manager
    pub async fn shutdown(&self) -> Result<(), PluginError> {
        info!("Shutting down plugin manager");

        // Unload all plugins
        let plugin_ids: Vec<String> = {
            let plugins = self.plugins.read().unwrap();
            plugins.keys().cloned().collect()
        };

        for plugin_id in plugin_ids {
            if let Err(e) = self.unload_plugin(&plugin_id).await {
                warn!("Failed to unload plugin '{}' during shutdown: {}", plugin_id, e);
            }
        }

        info!("Plugin manager shutdown complete");
        Ok(())
    }
}

impl std::fmt::Debug for PluginManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginManager")
            .field("config", &self.config)
            .field("plugins", &self.plugins)
            .field("dependency_graph", &self.dependency_graph)
            .field("loader", &"<PluginLoader trait object>")
            .field("registry", &self.registry)
            .field("stats", &self.stats)
            .finish()
    }
}

