//! Plugin Hot Reload System
//!
//! Provides file watching and automatic plugin reloading during development.
//! Monitors plugin files, manifests, and dependencies for changes and triggers
//! intelligent reloading with minimal disruption.

use crate::plugins::{PluginError, PluginManager, PluginManifestParser, ManifestValidationConfig};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};

/// Plugin hot reload manager
#[derive(Debug)]
pub struct PluginHotReloader {
    /// Hot reload configuration
    config: HotReloadConfig,
    /// File watcher
    watcher: Option<RecommendedWatcher>,
    /// Watch event receiver
    event_receiver: Option<mpsc::UnboundedReceiver<notify::Result<Event>>>,
    /// Plugin manager reference
    plugin_manager: Arc<RwLock<PluginManager>>,
    /// Manifest parser
    manifest_parser: PluginManifestParser,
    /// Watched plugins with their file dependencies
    watched_plugins: Arc<RwLock<HashMap<String, WatchedPlugin>>>,
    /// Debounce timers for file changes
    debounce_timers: Arc<RwLock<HashMap<PathBuf, Instant>>>,
    /// Reload queue
    reload_queue: Arc<RwLock<HashSet<String>>>,
    /// Active reload tasks
    active_reloads: Arc<RwLock<HashSet<String>>>,
}

/// Hot reload configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotReloadConfig {
    /// Enable hot reload
    pub enabled: bool,
    /// Debounce delay in milliseconds
    pub debounce_delay_ms: u64,
    /// Maximum reload attempts
    pub max_reload_attempts: u32,
    /// Reload timeout in seconds
    pub reload_timeout_seconds: u64,
    /// Watch recursively in plugin directories
    pub recursive_watch: bool,
    /// File patterns to watch
    pub watch_patterns: Vec<String>,
    /// File patterns to ignore
    pub ignore_patterns: Vec<String>,
    /// Enable manifest validation on reload
    pub validate_manifests: bool,
    /// Auto-restart failed plugins
    pub auto_restart_failed: bool,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            debounce_delay_ms: 300,
            max_reload_attempts: 3,
            reload_timeout_seconds: 30,
            recursive_watch: true,
            watch_patterns: vec![
                "*.toml".to_string(),
                "*.rs".to_string(),
                "*.wasm".to_string(),
                "*.so".to_string(),
                "*.dll".to_string(),
                "*.dylib".to_string(),
            ],
            ignore_patterns: vec![
                "target/**".to_string(),
                "*.tmp".to_string(),
                "*.swp".to_string(),
                ".git/**".to_string(),
                "node_modules/**".to_string(),
            ],
            validate_manifests: true,
            auto_restart_failed: true,
        }
    }
}

/// Information about a watched plugin
#[derive(Debug, Clone)]
pub struct WatchedPlugin {
    /// Plugin ID
    pub plugin_id: String,
    /// Plugin directory
    pub plugin_dir: PathBuf,
    /// Manifest path
    pub manifest_path: PathBuf,
    /// Entry point path
    pub entry_point: PathBuf,
    /// Additional watched paths
    pub watch_paths: Vec<PathBuf>,
    /// Last reload attempt
    pub last_reload: Option<Instant>,
    /// Reload attempt count
    pub reload_attempts: u32,
    /// Plugin status before reload
    pub previous_status: Option<crate::plugins::PluginStatus>,
}

/// Hot reload event types
#[derive(Debug, Clone)]
pub enum HotReloadEvent {
    /// Plugin file changed
    FileChanged { plugin_id: String, file_path: PathBuf },
    /// Plugin manifest changed
    ManifestChanged { plugin_id: String },
    /// Plugin reloading started
    ReloadStarted { plugin_id: String },
    /// Plugin reloaded successfully
    ReloadSucceeded { plugin_id: String },
    /// Plugin reload failed
    ReloadFailed { plugin_id: String, error: String },
    /// Plugin watch started
    WatchStarted { plugin_id: String },
    /// Plugin watch stopped
    WatchStopped { plugin_id: String },
}

impl PluginHotReloader {
    /// Create a new hot reload manager
    pub async fn new(
        config: HotReloadConfig,
        plugin_manager: Arc<RwLock<PluginManager>>,
    ) -> Result<Self, PluginError> {
        info!("Initializing plugin hot reload system");

        let manifest_parser = PluginManifestParser::new(ManifestValidationConfig::default());

        let mut reloader = Self {
            config,
            watcher: None,
            event_receiver: None,
            plugin_manager,
            manifest_parser,
            watched_plugins: Arc::new(RwLock::new(HashMap::new())),
            debounce_timers: Arc::new(RwLock::new(HashMap::new())),
            reload_queue: Arc::new(RwLock::new(HashSet::new())),
            active_reloads: Arc::new(RwLock::new(HashSet::new())),
        };

        if reloader.config.enabled {
            reloader.setup_file_watcher().await?;
        }

        Ok(reloader)
    }

    /// Setup file watcher
    async fn setup_file_watcher(&mut self) -> Result<(), PluginError> {
        let (tx, rx) = mpsc::unbounded_channel();

        let watcher = RecommendedWatcher::new(
            move |res| {
                if let Err(_) = tx.send(res) {
                    error!("Failed to send file watch event");
                }
            },
            Config::default(),
        ).map_err(|e| PluginError::IoError(format!("Failed to create file watcher: {}", e)))?;

        self.watcher = Some(watcher);
        self.event_receiver = Some(rx);

        info!("File watcher setup completed");
        Ok(())
    }

    /// Start hot reload system
    pub async fn start(&mut self) -> Result<(), PluginError> {
        if !self.config.enabled {
            info!("Hot reload is disabled");
            return Ok(());
        }

        info!("Starting plugin hot reload system");

        // Start the file watching loop
        self.start_watch_loop().await?;

        // Start the reload processing loop
        self.start_reload_loop().await?;

        info!("Plugin hot reload system started");
        Ok(())
    }

    /// Start file watching loop
    async fn start_watch_loop(&mut self) -> Result<(), PluginError> {
        let event_receiver = self.event_receiver.take()
            .ok_or_else(|| PluginError::InitializationFailed("Event receiver not available".to_string()))?;

        let watched_plugins = Arc::clone(&self.watched_plugins);
        let debounce_timers = Arc::clone(&self.debounce_timers);
        let reload_queue = Arc::clone(&self.reload_queue);
        let debounce_delay = Duration::from_millis(self.config.debounce_delay_ms);
        let ignore_patterns = self.config.ignore_patterns.clone();

        tokio::spawn(async move {
            Self::file_watch_loop(
                event_receiver,
                watched_plugins,
                debounce_timers,
                reload_queue,
                debounce_delay,
                ignore_patterns,
            ).await;
        });

        Ok(())
    }

    /// File watching loop
    async fn file_watch_loop(
        mut event_receiver: mpsc::UnboundedReceiver<notify::Result<Event>>,
        watched_plugins: Arc<RwLock<HashMap<String, WatchedPlugin>>>,
        debounce_timers: Arc<RwLock<HashMap<PathBuf, Instant>>>,
        reload_queue: Arc<RwLock<HashSet<String>>>,
        debounce_delay: Duration,
        ignore_patterns: Vec<String>,
    ) {
        while let Some(event_result) = event_receiver.recv().await {
            match event_result {
                Ok(event) => {
                    if let Err(e) = Self::handle_file_event(
                        &event,
                        &watched_plugins,
                        &debounce_timers,
                        &reload_queue,
                        debounce_delay,
                        &ignore_patterns,
                    ).await {
                        error!("Error handling file event: {}", e);
                    }
                }
                Err(e) => {
                    error!("File watch error: {}", e);
                }
            }
        }
    }

    /// Handle file system event
    async fn handle_file_event(
        event: &Event,
        watched_plugins: &Arc<RwLock<HashMap<String, WatchedPlugin>>>,
        debounce_timers: &Arc<RwLock<HashMap<PathBuf, Instant>>>,
        reload_queue: &Arc<RwLock<HashSet<String>>>,
        debounce_delay: Duration,
        ignore_patterns: &[String],
    ) -> Result<(), PluginError> {
        // Only handle modify and create events
        match &event.kind {
            EventKind::Modify(_) | EventKind::Create(_) => {}
            _ => return Ok(()),
        }

        for path in &event.paths {
            // Check ignore patterns
            if Self::should_ignore_path(path, ignore_patterns) {
                continue;
            }

            // Find plugin that owns this file
            let plugin_id = {
                let plugins = watched_plugins.read().await;
                plugins.iter()
                    .find(|(_, plugin)| {
                        path.starts_with(&plugin.plugin_dir) ||
                        path == &plugin.manifest_path ||
                        path == &plugin.entry_point ||
                        plugin.watch_paths.iter().any(|watch_path| path.starts_with(watch_path))
                    })
                    .map(|(id, _)| id.clone())
            };

            if let Some(plugin_id) = plugin_id {
                // Apply debouncing
                let now = Instant::now();
                let should_reload = {
                    let mut timers = debounce_timers.write().await;
                    match timers.get(path) {
                        Some(last_time) if now.duration_since(*last_time) < debounce_delay => false,
                        _ => {
                            timers.insert(path.clone(), now);
                            true
                        }
                    }
                };

                if should_reload {
                    debug!("Queuing plugin {} for reload due to file change: {:?}", plugin_id, path);
                    let mut queue = reload_queue.write().await;
                    queue.insert(plugin_id);
                }
            }
        }

        Ok(())
    }

    /// Check if path should be ignored
    fn should_ignore_path(path: &Path, ignore_patterns: &[String]) -> bool {
        let path_str = path.to_string_lossy();
        
        for pattern in ignore_patterns {
            // Simple glob-like matching
            if pattern.contains("**") {
                let prefix = pattern.split("**").next().unwrap_or("");
                if path_str.contains(prefix) {
                    return true;
                }
            } else if pattern.starts_with("*.") {
                let extension = pattern.strip_prefix("*.").unwrap_or("");
                if let Some(file_ext) = path.extension() {
                    if file_ext.to_string_lossy() == extension {
                        continue; // Don't ignore, this is a file we want to watch
                    }
                }
                return true; // Different extension, ignore
            } else if path_str.contains(pattern) {
                return true;
            }
        }
        
        false
    }

    /// Start reload processing loop
    async fn start_reload_loop(&self) -> Result<(), PluginError> {
        let plugin_manager = Arc::clone(&self.plugin_manager);
        let manifest_parser = self.manifest_parser.clone();
        let watched_plugins = Arc::clone(&self.watched_plugins);
        let reload_queue = Arc::clone(&self.reload_queue);
        let active_reloads = Arc::clone(&self.active_reloads);
        let config = self.config.clone();

        tokio::spawn(async move {
            Self::reload_processing_loop(
                plugin_manager,
                manifest_parser,
                watched_plugins,
                reload_queue,
                active_reloads,
                config,
            ).await;
        });

        Ok(())
    }

    /// Reload processing loop
    async fn reload_processing_loop(
        plugin_manager: Arc<RwLock<PluginManager>>,
        manifest_parser: PluginManifestParser,
        watched_plugins: Arc<RwLock<HashMap<String, WatchedPlugin>>>,
        reload_queue: Arc<RwLock<HashSet<String>>>,
        active_reloads: Arc<RwLock<HashSet<String>>>,
        config: HotReloadConfig,
    ) {
        let mut interval = tokio::time::interval(Duration::from_millis(100));
        
        loop {
            interval.tick().await;

            // Get plugins to reload
            let plugins_to_reload: Vec<String> = {
                let mut queue = reload_queue.write().await;
                let active = active_reloads.read().await;
                
                let ready_plugins: Vec<String> = queue
                    .iter()
                    .filter(|plugin_id| !active.contains(*plugin_id))
                    .cloned()
                    .collect();
                
                for plugin_id in &ready_plugins {
                    queue.remove(plugin_id);
                }
                
                ready_plugins
            };

            // Process reloads
            for plugin_id in plugins_to_reload {
                let plugin_manager = Arc::clone(&plugin_manager);
                let manifest_parser = manifest_parser.clone();
                let watched_plugins = Arc::clone(&watched_plugins);
                let active_reloads = Arc::clone(&active_reloads);
                let config = config.clone();
                
                // Mark as active
                {
                    let mut active = active_reloads.write().await;
                    active.insert(plugin_id.clone());
                }

                tokio::spawn(async move {
                    let result = Self::reload_plugin(
                        &plugin_id,
                        &plugin_manager,
                        &manifest_parser,
                        &watched_plugins,
                        &config,
                    ).await;

                    if let Err(e) = result {
                        error!("Failed to reload plugin {}: {}", plugin_id, e);
                    }

                    // Remove from active
                    let mut active = active_reloads.write().await;
                    active.remove(&plugin_id);
                });
            }
        }
    }

    /// Reload a specific plugin
    async fn reload_plugin(
        plugin_id: &str,
        plugin_manager: &Arc<RwLock<PluginManager>>,
        manifest_parser: &PluginManifestParser,
        watched_plugins: &Arc<RwLock<HashMap<String, WatchedPlugin>>>,
        config: &HotReloadConfig,
    ) -> Result<(), PluginError> {
        info!("Reloading plugin: {}", plugin_id);

        // Get plugin info
        let watched_plugin = {
            let plugins = watched_plugins.read().await;
            plugins.get(plugin_id).cloned()
        };

        let watched_plugin = watched_plugin
            .ok_or_else(|| PluginError::NotFound(format!("Plugin {} not watched", plugin_id)))?;

        // Update reload attempt count
        {
            let mut plugins = watched_plugins.write().await;
            if let Some(plugin) = plugins.get_mut(plugin_id) {
                plugin.reload_attempts += 1;
                plugin.last_reload = Some(Instant::now());
            }
        }

        // Check max attempts
        if watched_plugin.reload_attempts >= config.max_reload_attempts {
            return Err(PluginError::ExecutionFailed(
                format!("Max reload attempts ({}) exceeded for plugin {}", 
                        config.max_reload_attempts, plugin_id)
            ));
        }

        // Validate manifest if enabled
        if config.validate_manifests && watched_plugin.manifest_path.exists() {
            match manifest_parser.parse_manifest(&watched_plugin.manifest_path).await {
                Ok(_) => debug!("Manifest validation passed for plugin: {}", plugin_id),
                Err(e) => {
                    warn!("Manifest validation failed for plugin {}: {}", plugin_id, e);
                    // Continue with reload despite validation warning
                }
            }
        }

        // Perform the actual reload
        let manager = plugin_manager.write().await;
        
        // Unload the plugin first
        if let Err(e) = manager.unload_plugin(plugin_id).await {
            warn!("Failed to unload plugin {} during reload: {}", plugin_id, e);
        }

        // Load the plugin again
        // For now, we'll use the scan_and_load_plugins method as a placeholder
        // In a real implementation, we'd need a dedicated reload method
        match manager.scan_and_load_plugins().await {
            Ok(_) => {
                info!("Successfully reloaded plugin: {}", plugin_id);
                
                // Reset reload attempts on success
                let mut plugins = watched_plugins.write().await;
                if let Some(plugin) = plugins.get_mut(plugin_id) {
                    plugin.reload_attempts = 0;
                }
                
                Ok(())
            }
            Err(e) => {
                error!("Failed to reload plugin {}: {}", plugin_id, e);
                Err(e)
            }
        }
    }

    /// Add a plugin to hot reload watching
    pub async fn watch_plugin(
        &mut self,
        plugin_id: &str,
        plugin_dir: PathBuf,
        manifest_path: PathBuf,
        entry_point: PathBuf,
        additional_paths: Vec<PathBuf>,
    ) -> Result<(), PluginError> {
        if !self.config.enabled {
            return Ok(());
        }

        info!("Adding plugin {} to hot reload watch", plugin_id);

        let watched_plugin = WatchedPlugin {
            plugin_id: plugin_id.to_string(),
            plugin_dir: plugin_dir.clone(),
            manifest_path: manifest_path.clone(),
            entry_point: entry_point.clone(),
            watch_paths: additional_paths.clone(),
            last_reload: None,
            reload_attempts: 0,
            previous_status: None,
        };

        // Add to watched plugins
        {
            let mut plugins = self.watched_plugins.write().await;
            plugins.insert(plugin_id.to_string(), watched_plugin);
        }

        // Add file watches
        if let Some(watcher) = &mut self.watcher {
            let watch_mode = if self.config.recursive_watch {
                RecursiveMode::Recursive
            } else {
                RecursiveMode::NonRecursive
            };

            // Watch plugin directory
            if let Err(e) = watcher.watch(&plugin_dir, watch_mode) {
                error!("Failed to watch plugin directory {:?}: {}", plugin_dir, e);
            }

            // Watch manifest file
            if manifest_path.exists() {
                if let Err(e) = watcher.watch(&manifest_path, RecursiveMode::NonRecursive) {
                    error!("Failed to watch manifest {:?}: {}", manifest_path, e);
                }
            }

            // Watch entry point
            if entry_point.exists() {
                if let Err(e) = watcher.watch(&entry_point, RecursiveMode::NonRecursive) {
                    error!("Failed to watch entry point {:?}: {}", entry_point, e);
                }
            }

            // Watch additional paths
            for path in &additional_paths {
                if path.exists() {
                    let mode = if path.is_dir() && self.config.recursive_watch {
                        RecursiveMode::Recursive
                    } else {
                        RecursiveMode::NonRecursive
                    };
                    
                    if let Err(e) = watcher.watch(path, mode) {
                        error!("Failed to watch path {:?}: {}", path, e);
                    }
                }
            }
        }

        info!("Plugin {} added to hot reload watch", plugin_id);
        Ok(())
    }

    /// Remove a plugin from hot reload watching
    pub async fn unwatch_plugin(&mut self, plugin_id: &str) -> Result<(), PluginError> {
        info!("Removing plugin {} from hot reload watch", plugin_id);

        let watched_plugin = {
            let mut plugins = self.watched_plugins.write().await;
            plugins.remove(plugin_id)
        };

        if let Some(plugin) = watched_plugin {
            // Remove file watches
            if let Some(watcher) = &mut self.watcher {
                let _ = watcher.unwatch(&plugin.plugin_dir);
                let _ = watcher.unwatch(&plugin.manifest_path);
                let _ = watcher.unwatch(&plugin.entry_point);
                
                for path in &plugin.watch_paths {
                    let _ = watcher.unwatch(path);
                }
            }

            info!("Plugin {} removed from hot reload watch", plugin_id);
        }

        Ok(())
    }

    /// Get hot reload statistics
    pub async fn get_stats(&self) -> HotReloadStats {
        let plugins = self.watched_plugins.read().await;
        let reload_queue = self.reload_queue.read().await;
        let active_reloads = self.active_reloads.read().await;

        let total_reload_attempts = plugins.values()
            .map(|p| p.reload_attempts)
            .sum();

        HotReloadStats {
            enabled: self.config.enabled,
            watched_plugins_count: plugins.len(),
            queued_reloads_count: reload_queue.len(),
            active_reloads_count: active_reloads.len(),
            total_reload_attempts,
            debounce_delay_ms: self.config.debounce_delay_ms,
        }
    }

    /// Manually trigger a plugin reload
    pub async fn trigger_reload(&self, plugin_id: &str) -> Result<(), PluginError> {
        if !self.config.enabled {
            return Err(PluginError::ExecutionFailed("Hot reload is disabled".to_string()));
        }

        info!("Manually triggering reload for plugin: {}", plugin_id);
        
        let mut queue = self.reload_queue.write().await;
        queue.insert(plugin_id.to_string());
        
        Ok(())
    }

    /// Stop hot reload system
    pub async fn stop(&mut self) -> Result<(), PluginError> {
        info!("Stopping plugin hot reload system");

        // Clear all watches
        let plugin_ids: Vec<String> = {
            let plugins = self.watched_plugins.read().await;
            plugins.keys().cloned().collect()
        };

        for plugin_id in plugin_ids {
            self.unwatch_plugin(&plugin_id).await?;
        }

        // Drop watcher
        self.watcher = None;
        self.event_receiver = None;

        info!("Plugin hot reload system stopped");
        Ok(())
    }
}

/// Hot reload statistics
#[derive(Debug, Clone, Serialize)]
pub struct HotReloadStats {
    /// Whether hot reload is enabled
    pub enabled: bool,
    /// Number of watched plugins
    pub watched_plugins_count: usize,
    /// Number of queued reloads
    pub queued_reloads_count: usize,
    /// Number of active reloads
    pub active_reloads_count: usize,
    /// Total reload attempts across all plugins
    pub total_reload_attempts: u32,
    /// Debounce delay in milliseconds
    pub debounce_delay_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::time::sleep;

    async fn create_test_plugin_manager() -> Arc<RwLock<PluginManager>> {
        // This would need to be implemented when PluginManager is complete
        // For now, return a mock
        Arc::new(RwLock::new(
            PluginManager::new(crate::plugins::PluginSystemConfig::default())
                .await
                .unwrap()
        ))
    }

    #[tokio::test]
    async fn test_hot_reload_creation() {
        let config = HotReloadConfig::default();
        let plugin_manager = create_test_plugin_manager().await;
        
        let reloader = PluginHotReloader::new(config, plugin_manager).await;
        assert!(reloader.is_ok());
    }

    #[tokio::test]
    async fn test_ignore_patterns() {
        assert!(PluginHotReloader::should_ignore_path(
            Path::new("/test/target/debug/plugin.so"),
            &["target/**".to_string()]
        ));

        assert!(!PluginHotReloader::should_ignore_path(
            Path::new("/test/src/main.rs"),
            &["target/**".to_string()]
        ));

        assert!(PluginHotReloader::should_ignore_path(
            Path::new("/test/file.tmp"),
            &["*.tmp".to_string()]
        ));
    }

    #[tokio::test]
    async fn test_watch_plugin() {
        let temp_dir = TempDir::new().unwrap();
        let config = HotReloadConfig::default();
        let plugin_manager = create_test_plugin_manager().await;
        
        let mut reloader = PluginHotReloader::new(config, plugin_manager).await.unwrap();
        
        let plugin_dir = temp_dir.path().to_path_buf();
        let manifest_path = plugin_dir.join("plugin.toml");
        let entry_point = plugin_dir.join("main.rs");
        
        // Create test files
        std::fs::create_dir_all(&plugin_dir).unwrap();
        std::fs::write(&manifest_path, "# test manifest").unwrap();
        std::fs::write(&entry_point, "// test code").unwrap();

        let result = reloader.watch_plugin(
            "test-plugin",
            plugin_dir,
            manifest_path,
            entry_point,
            vec![]
        ).await;

        assert!(result.is_ok());

        let stats = reloader.get_stats().await;
        assert_eq!(stats.watched_plugins_count, 1);
    }

    #[test]
    fn test_hot_reload_config_default() {
        let config = HotReloadConfig::default();
        assert!(config.enabled);
        assert_eq!(config.debounce_delay_ms, 300);
        assert_eq!(config.max_reload_attempts, 3);
        assert!(config.validate_manifests);
    }

    #[tokio::test]
    async fn test_manual_reload_trigger() {
        let config = HotReloadConfig::default();
        let plugin_manager = create_test_plugin_manager().await;
        
        let reloader = PluginHotReloader::new(config, plugin_manager).await.unwrap();
        
        let result = reloader.trigger_reload("test-plugin").await;
        assert!(result.is_ok());

        let stats = reloader.get_stats().await;
        assert_eq!(stats.queued_reloads_count, 1);
    }

    #[tokio::test]
    async fn test_disabled_hot_reload() {
        let mut config = HotReloadConfig::default();
        config.enabled = false;
        
        let plugin_manager = create_test_plugin_manager().await;
        let reloader = PluginHotReloader::new(config, plugin_manager).await.unwrap();
        
        let result = reloader.trigger_reload("test-plugin").await;
        assert!(result.is_err());
    }
}