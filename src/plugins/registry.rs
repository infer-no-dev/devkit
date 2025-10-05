//! Plugin Registry System
//!
//! Manages plugin metadata, version tracking, local caching, and discovery.
//! Provides a centralized registry for all installed and available plugins.

use crate::plugins::{
    PluginError, PluginMetadata, PluginDependency, PluginCapability
};
use crate::plugins::types::{InstallationInfo, InstallationSource};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::fs;
use tokio::fs as async_fs;
use tracing::{debug, error, info, warn};
use chrono::{DateTime, Utc};

/// Plugin registry for managing plugin metadata and installations
#[derive(Debug, Default)]
pub struct PluginRegistry {
    /// Local plugin cache directory
    cache_dir: PathBuf,
    /// Plugin metadata indexed by ID
    metadata: HashMap<String, PluginRegistryEntry>,
    /// Installation information indexed by plugin ID
    installations: HashMap<String, InstallationInfo>,
    /// Plugin dependency graph
    dependency_graph: DependencyGraph,
    /// Registry configuration
    config: RegistryConfig,
}

/// Registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Registry cache directory
    pub cache_dir: PathBuf,
    /// Plugin installation directory
    pub plugin_dir: PathBuf,
    /// Automatic update interval in hours
    pub auto_update_interval: u64,
    /// Maximum cache size in MB
    pub max_cache_size: u64,
    /// Enable development mode (allows unsigned plugins)
    pub dev_mode: bool,
    /// Trusted plugin sources
    pub trusted_sources: Vec<String>,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let devkit_dir = home.join(".devkit");
        
        Self {
            cache_dir: devkit_dir.join("cache"),
            plugin_dir: devkit_dir.join("plugins"),
            auto_update_interval: 24, // 24 hours
            max_cache_size: 1024, // 1GB
            dev_mode: false,
            trusted_sources: vec![
                "https://plugins.devkit.dev".to_string(),
                "https://registry.npmjs.org".to_string(),
                "https://pypi.org".to_string(),
            ],
        }
    }
}

/// Plugin registry entry with metadata and caching info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRegistryEntry {
    /// Plugin metadata
    pub metadata: PluginMetadata,
    /// Installation information if installed
    pub installation: Option<InstallationInfo>,
    /// Cache timestamp
    pub cached_at: DateTime<Utc>,
    /// Source registry URL
    pub source: String,
    /// Verification status
    pub verified: bool,
    /// Available versions
    pub versions: Vec<VersionInfo>,
}

/// Version information for a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub released_at: DateTime<Utc>,
    pub download_url: String,
    pub checksum: String,
    pub size: u64,
    pub yanked: bool,
}

/// Plugin dependency graph for resolution
#[derive(Debug, Default)]
pub struct DependencyGraph {
    /// Direct dependencies: plugin_id -> set of dependency_ids
    dependencies: HashMap<String, HashSet<String>>,
    /// Reverse dependencies: plugin_id -> set of dependent_ids
    dependents: HashMap<String, HashSet<String>>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub async fn new(config: RegistryConfig) -> Result<Self, PluginError> {
        // Ensure cache and plugin directories exist
        async_fs::create_dir_all(&config.cache_dir)
            .await
            .map_err(|e| PluginError::IoError(format!("Failed to create cache directory: {}", e)))?;
            
        async_fs::create_dir_all(&config.plugin_dir)
            .await
            .map_err(|e| PluginError::IoError(format!("Failed to create plugin directory: {}", e)))?;

        let mut registry = Self {
            cache_dir: config.cache_dir.clone(),
            metadata: HashMap::new(),
            installations: HashMap::new(),
            dependency_graph: DependencyGraph::default(),
            config,
        };

        // Load existing cache
        registry.load_cache().await?;
        
        // Scan for installed plugins
        registry.scan_installed_plugins().await?;

        info!("Plugin registry initialized with {} cached entries", registry.metadata.len());
        Ok(registry)
    }

    /// Register a plugin in the registry
    pub async fn register_plugin(&mut self, metadata: PluginMetadata, source: String) -> Result<(), PluginError> {
        debug!("Registering plugin: {} v{}", metadata.id, metadata.version);

        let entry = PluginRegistryEntry {
            metadata: metadata.clone(),
            installation: self.installations.get(&metadata.id).cloned(),
            cached_at: Utc::now(),
            source,
            verified: self.is_trusted_source(&metadata),
            versions: vec![VersionInfo {
                version: metadata.version.clone(),
                released_at: metadata.updated_at,
                download_url: format!("https://plugins.devkit.dev/{}/download", metadata.id),
                checksum: "".to_string(), // TODO: Calculate checksum
                size: 0, // TODO: Get actual size
                yanked: false,
            }],
        };

        // Update dependency graph
        self.update_dependency_graph(&metadata);

        // Store entry
        self.metadata.insert(metadata.id.clone(), entry);

        // Save cache
        self.save_cache().await?;

        info!("Registered plugin: {} v{}", metadata.id, metadata.version);
        Ok(())
    }

    /// Get plugin metadata by ID
    pub fn get_plugin(&self, plugin_id: &str) -> Option<&PluginRegistryEntry> {
        self.metadata.get(plugin_id)
    }

    /// Search plugins by query
    pub fn search_plugins(&self, query: &str) -> Vec<&PluginRegistryEntry> {
        let query_lower = query.to_lowercase();
        
        self.metadata.values()
            .filter(|entry| {
                let metadata = &entry.metadata;
                metadata.name.to_lowercase().contains(&query_lower) ||
                metadata.description.to_lowercase().contains(&query_lower) ||
                metadata.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower)) ||
                metadata.id.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// List all plugins with optional filters
    pub fn list_plugins(&self, filter: Option<PluginFilter>) -> Vec<&PluginRegistryEntry> {
        let mut plugins: Vec<_> = self.metadata.values().collect();

        if let Some(filter) = filter {
            plugins = plugins.into_iter().filter(|entry| {
                match filter {
                    PluginFilter::Installed => entry.installation.is_some(),
                    PluginFilter::Available => entry.installation.is_none(),
                    PluginFilter::UpdateAvailable => {
                        if let Some(installation) = &entry.installation {
                            // Check if a newer version is available
                            entry.versions.iter().any(|v| {
                                v.version != installation.version && !v.yanked
                            })
                        } else {
                            false
                        }
                    }
                    PluginFilter::Verified => entry.verified,
                    PluginFilter::Capability(ref cap) => {
                        entry.metadata.capabilities.contains(cap)
                    }
                }
            }).collect();
        }

        // Sort by name
        plugins.sort_by(|a, b| a.metadata.name.cmp(&b.metadata.name));
        plugins
    }

    /// Mark plugin as installed
    pub async fn mark_installed(&mut self, plugin_id: &str, installation: InstallationInfo) -> Result<(), PluginError> {
        debug!("Marking plugin as installed: {}", plugin_id);

        self.installations.insert(plugin_id.to_string(), installation.clone());

        // Update registry entry if it exists
        if let Some(entry) = self.metadata.get_mut(plugin_id) {
            entry.installation = Some(installation);
        }

        self.save_cache().await?;
        Ok(())
    }

    /// Mark plugin as uninstalled
    pub async fn mark_uninstalled(&mut self, plugin_id: &str) -> Result<(), PluginError> {
        debug!("Marking plugin as uninstalled: {}", plugin_id);

        self.installations.remove(plugin_id);

        // Update registry entry if it exists
        if let Some(entry) = self.metadata.get_mut(plugin_id) {
            entry.installation = None;
        }

        self.save_cache().await?;
        Ok(())
    }

    /// Resolve plugin dependencies
    pub fn resolve_dependencies(&self, plugin_id: &str) -> Result<Vec<String>, PluginError> {
        let mut resolved = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = Vec::new();

        self.resolve_dependencies_recursive(plugin_id, &mut resolved, &mut visited, &mut stack)?;
        Ok(resolved)
    }

    /// Check for circular dependencies
    pub fn check_circular_dependencies(&self, plugin_id: &str) -> Result<(), PluginError> {
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();

        self.check_circular_dependencies_recursive(plugin_id, &mut visited, &mut stack)
    }

    /// Get plugins that depend on the given plugin
    pub fn get_dependents(&self, plugin_id: &str) -> Vec<&str> {
        self.dependency_graph.dependents
            .get(plugin_id)
            .map(|deps| deps.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Check if plugin can be safely uninstalled
    pub fn can_uninstall(&self, plugin_id: &str) -> Result<bool, PluginError> {
        let dependents = self.get_dependents(plugin_id);
        
        // Check if any dependents are installed
        for dependent in &dependents {
            if let Some(entry) = self.get_plugin(dependent) {
                if entry.installation.is_some() {
                    // Check if dependency is optional
                    if let Some(dep) = entry.metadata.dependencies.iter()
                        .find(|d| d.id == plugin_id) 
                    {
                        if !dep.optional {
                            return Ok(false);
                        }
                    }
                }
            }
        }

        Ok(true)
    }

    /// Clean up old cache entries
    pub async fn cleanup_cache(&mut self) -> Result<(), PluginError> {
        let cutoff = Utc::now() - chrono::Duration::days(30);
        let mut to_remove = Vec::new();

        for (plugin_id, entry) in &self.metadata {
            // Keep installed plugins and recently cached entries
            if entry.installation.is_none() && entry.cached_at < cutoff {
                to_remove.push(plugin_id.clone());
            }
        }

        for plugin_id in to_remove {
            self.metadata.remove(&plugin_id);
            info!("Removed stale cache entry: {}", plugin_id);
        }

        self.save_cache().await?;
        Ok(())
    }

    /// Get registry statistics
    pub fn get_stats(&self) -> RegistryStats {
        let total_plugins = self.metadata.len();
        let installed_plugins = self.installations.len();
        let verified_plugins = self.metadata.values().filter(|e| e.verified).count();

        let mut capabilities = HashMap::new();
        for entry in self.metadata.values() {
            for cap in &entry.metadata.capabilities {
                *capabilities.entry(cap.clone()).or_insert(0) += 1;
            }
        }

        RegistryStats {
            total_plugins,
            installed_plugins,
            available_plugins: total_plugins - installed_plugins,
            verified_plugins,
            cache_size_mb: self.calculate_cache_size(),
            capabilities,
        }
    }

    // Private helper methods

    async fn load_cache(&mut self) -> Result<(), PluginError> {
        let cache_file = self.cache_dir.join("registry.json");
        
        if !cache_file.exists() {
            debug!("No cache file found, starting with empty registry");
            return Ok(());
        }

        match async_fs::read_to_string(&cache_file).await {
            Ok(content) => {
                match serde_json::from_str::<HashMap<String, PluginRegistryEntry>>(&content) {
                    Ok(metadata) => {
                        self.metadata = metadata;
                        debug!("Loaded {} entries from cache", self.metadata.len());
                    }
                    Err(e) => {
                        warn!("Failed to parse cache file, starting fresh: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to read cache file: {}", e);
            }
        }

        Ok(())
    }

    async fn save_cache(&self) -> Result<(), PluginError> {
        let cache_file = self.cache_dir.join("registry.json");
        
        let content = serde_json::to_string_pretty(&self.metadata)
            .map_err(|e| PluginError::InvalidManifest(format!("Failed to serialize cache: {}", e)))?;

        async_fs::write(&cache_file, content)
            .await
            .map_err(|e| PluginError::IoError(format!("Failed to write cache: {}", e)))?;

        debug!("Saved registry cache with {} entries", self.metadata.len());
        Ok(())
    }

    async fn scan_installed_plugins(&mut self) -> Result<(), PluginError> {
        if !self.config.plugin_dir.exists() {
            return Ok(());
        }

        let mut entries = async_fs::read_dir(&self.config.plugin_dir)
            .await
            .map_err(|e| PluginError::IoError(format!("Failed to read plugin directory: {}", e)))?;

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_dir() {
                if let Err(e) = self.scan_plugin_directory(&path).await {
                    warn!("Failed to scan plugin directory {:?}: {}", path, e);
                }
            }
        }

        Ok(())
    }

    async fn scan_plugin_directory(&mut self, dir: &Path) -> Result<(), PluginError> {
        let manifest_path = dir.join("plugin.toml");
        if !manifest_path.exists() {
            return Ok(()); // Not a plugin directory
        }

        // Parse manifest
        let content = async_fs::read_to_string(&manifest_path)
            .await
            .map_err(|e| PluginError::IoError(e.to_string()))?;

        let manifest: PluginManifest = toml::from_str(&content)
            .map_err(|e| PluginError::InvalidManifest(e.to_string()))?;

        let metadata = manifest.into_metadata();

        // Create installation info
        let installation = InstallationInfo {
            plugin_id: metadata.id.clone(),
            version: metadata.version.clone(),
            installed_at: Utc::now(),
            install_path: dir.to_path_buf(),
            checksum: "".to_string(), // TODO: Calculate checksum
            auto_update: false,
            enabled: true,
            installation_source: InstallationSource::LocalFile { path: dir.to_path_buf() },
        };

        self.installations.insert(metadata.id.clone(), installation);

        // Register in metadata if not already present
        if !self.metadata.contains_key(&metadata.id) {
            self.register_plugin(metadata, "local".to_string()).await?;
        }

        Ok(())
    }

    fn update_dependency_graph(&mut self, metadata: &PluginMetadata) {
        let plugin_deps: HashSet<String> = metadata.dependencies
            .iter()
            .map(|d| d.id.clone())
            .collect();

        self.dependency_graph.dependencies.insert(metadata.id.clone(), plugin_deps);

        // Update reverse dependencies
        for dep in &metadata.dependencies {
            self.dependency_graph.dependents
                .entry(dep.id.clone())
                .or_insert_with(HashSet::new)
                .insert(metadata.id.clone());
        }
    }

    fn resolve_dependencies_recursive(
        &self,
        plugin_id: &str,
        resolved: &mut Vec<String>,
        visited: &mut HashSet<String>,
        stack: &mut Vec<String>,
    ) -> Result<(), PluginError> {
        if visited.contains(plugin_id) {
            return Ok(());
        }

        if stack.contains(&plugin_id.to_string()) {
            return Err(PluginError::InvalidManifest(format!("Circular dependency detected: {}", plugin_id)));
        }

        stack.push(plugin_id.to_string());

        if let Some(deps) = self.dependency_graph.dependencies.get(plugin_id) {
            for dep_id in deps {
                self.resolve_dependencies_recursive(dep_id, resolved, visited, stack)?;
            }
        }

        stack.pop();
        visited.insert(plugin_id.to_string());
        resolved.push(plugin_id.to_string());

        Ok(())
    }

    fn check_circular_dependencies_recursive(
        &self,
        plugin_id: &str,
        visited: &mut HashSet<String>,
        stack: &mut HashSet<String>,
    ) -> Result<(), PluginError> {
        if stack.contains(plugin_id) {
            return Err(PluginError::InvalidManifest(format!("Circular dependency detected: {}", plugin_id)));
        }

        if visited.contains(plugin_id) {
            return Ok(());
        }

        visited.insert(plugin_id.to_string());
        stack.insert(plugin_id.to_string());

        if let Some(deps) = self.dependency_graph.dependencies.get(plugin_id) {
            for dep_id in deps {
                self.check_circular_dependencies_recursive(dep_id, visited, stack)?;
            }
        }

        stack.remove(plugin_id);
        Ok(())
    }

    fn is_trusted_source(&self, metadata: &PluginMetadata) -> bool {
        // TODO: Implement proper verification logic
        if self.config.dev_mode {
            return true;
        }

        // Check if source is in trusted list
        if let Some(repo) = &metadata.repository {
            self.config.trusted_sources.iter()
                .any(|source| repo.starts_with(source))
        } else {
            false
        }
    }

    fn calculate_cache_size(&self) -> u64 {
        // TODO: Implement actual cache size calculation
        0
    }
}

/// Plugin filter options
#[derive(Debug, Clone)]
pub enum PluginFilter {
    Installed,
    Available,
    UpdateAvailable,
    Verified,
    Capability(PluginCapability),
}

/// Registry statistics
#[derive(Debug, Clone)]
pub struct RegistryStats {
    pub total_plugins: usize,
    pub installed_plugins: usize,
    pub available_plugins: usize,
    pub verified_plugins: usize,
    pub cache_size_mb: u64,
    pub capabilities: HashMap<PluginCapability, u32>,
}

/// Plugin manifest structure for TOML parsing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub metadata: PluginManifestMetadata,
    #[serde(default)]
    pub dependencies: Vec<PluginDependency>,
    #[serde(default)]
    pub permissions: PluginPermissions,
    #[serde(default)]
    pub capabilities: Vec<PluginCapabilityEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifestMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub entry_point: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginPermissions {
    #[serde(default)]
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCapabilityEntry {
    pub capability: String,
    pub name: Option<String>,
}

impl PluginManifest {
    /// Convert manifest to plugin metadata
    pub fn into_metadata(self) -> PluginMetadata {
        let created_at = DateTime::parse_from_rfc3339(&self.metadata.created_at)
            .unwrap_or_else(|_| Utc::now().into())
            .with_timezone(&Utc);
            
        let updated_at = DateTime::parse_from_rfc3339(&self.metadata.updated_at)
            .unwrap_or_else(|_| Utc::now().into())
            .with_timezone(&Utc);

        let capabilities = self.capabilities.into_iter().map(|cap| {
            match cap.capability.as_str() {
                "CodeAnalysis" => PluginCapability::CodeAnalysis,
                "CodeGeneration" => PluginCapability::CodeGeneration,
                "CodeFormatting" => PluginCapability::CodeFormatting,
                "Completion" => PluginCapability::Completion,
                "Diagnostics" => PluginCapability::Diagnostics,
                "VersionControl" => PluginCapability::VersionControl,
                "DependencyManagement" => PluginCapability::DependencyManagement,
                "Testing" => PluginCapability::Testing,
                "Documentation" => PluginCapability::Documentation,
                _ => PluginCapability::Custom(cap.name.unwrap_or(cap.capability)),
            }
        }).collect();

        PluginMetadata {
            id: self.metadata.id,
            name: self.metadata.name,
            version: self.metadata.version,
            description: self.metadata.description,
            author: self.metadata.author,
            homepage: self.metadata.homepage,
            repository: self.metadata.repository,
            license: self.metadata.license,
            tags: self.metadata.tags,
            dependencies: self.dependencies,
            permissions: self.permissions.permissions,
            entry_point: self.metadata.entry_point,
            created_at,
            updated_at,
            capabilities,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_registry_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = RegistryConfig {
            cache_dir: temp_dir.path().join("cache"),
            plugin_dir: temp_dir.path().join("plugins"),
            ..Default::default()
        };

        let registry = PluginRegistry::new(config).await;
        assert!(registry.is_ok());
    }

    #[tokio::test]
    async fn test_plugin_registration() {
        let temp_dir = TempDir::new().unwrap();
        let config = RegistryConfig {
            cache_dir: temp_dir.path().join("cache"),
            plugin_dir: temp_dir.path().join("plugins"),
            ..Default::default()
        };

        let mut registry = PluginRegistry::new(config).await.unwrap();

        let metadata = PluginMetadata {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            homepage: None,
            repository: None,
            license: "MIT".to_string(),
            tags: vec![],
            dependencies: vec![],
            permissions: vec![],
            entry_point: "plugin.py".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            capabilities: vec![PluginCapability::CodeAnalysis],
        };

        let result = registry.register_plugin(metadata, "test".to_string()).await;
        assert!(result.is_ok());

        let entry = registry.get_plugin("test-plugin");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().metadata.name, "Test Plugin");
    }

    #[tokio::test]
    async fn test_dependency_resolution() {
        let temp_dir = TempDir::new().unwrap();
        let config = RegistryConfig {
            cache_dir: temp_dir.path().join("cache"),
            plugin_dir: temp_dir.path().join("plugins"),
            ..Default::default()
        };

        let mut registry = PluginRegistry::new(config).await.unwrap();

        // Register plugin A that depends on B
        let plugin_a = PluginMetadata {
            id: "plugin-a".to_string(),
            name: "Plugin A".to_string(),
            version: "1.0.0".to_string(),
            description: "Plugin A".to_string(),
            author: "Test".to_string(),
            homepage: None,
            repository: None,
            license: "MIT".to_string(),
            tags: vec![],
            dependencies: vec![PluginDependency {
                id: "plugin-b".to_string(),
                version: "1.0.0".to_string(),
                optional: false,
                reason: None,
            }],
            permissions: vec![],
            entry_point: "plugin.py".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            capabilities: vec![],
        };

        let plugin_b = PluginMetadata {
            id: "plugin-b".to_string(),
            name: "Plugin B".to_string(),
            version: "1.0.0".to_string(),
            description: "Plugin B".to_string(),
            author: "Test".to_string(),
            homepage: None,
            repository: None,
            license: "MIT".to_string(),
            tags: vec![],
            dependencies: vec![],
            permissions: vec![],
            entry_point: "plugin.py".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            capabilities: vec![],
        };

        registry.register_plugin(plugin_b, "test".to_string()).await.unwrap();
        registry.register_plugin(plugin_a, "test".to_string()).await.unwrap();

        let resolved = registry.resolve_dependencies("plugin-a").unwrap();
        assert!(resolved.contains(&"plugin-b".to_string()));
        assert!(resolved.contains(&"plugin-a".to_string()));
    }
}