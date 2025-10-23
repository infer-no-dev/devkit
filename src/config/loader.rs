//! Configuration loading and saving utilities with fallback support.

use crate::config::{Config, ConfigError};
use std::path::PathBuf;
use tracing::{info, warn, debug};

/// Configuration loader/saver with hierarchical fallback support
#[derive(Debug)]
pub struct ConfigLoader {
    /// Search paths for configuration files, in priority order
    search_paths: Vec<PathBuf>,
}

/// Result of configuration loading attempt
#[derive(Debug)]
pub struct ConfigLoadResult {
    pub config: Config,
    pub loaded_from: PathBuf,
    pub fallback_used: bool,
    pub errors: Vec<(PathBuf, ConfigError)>,
}

impl ConfigLoader {
    pub fn new() -> Self {
        Self {
            search_paths: Vec::new(),
        }
    }

    /// Create a new ConfigLoader with standard search paths
    pub fn new_with_search_paths() -> Self {
        let mut search_paths = Vec::new();
        
        // 1. Explicit config file path (if specified via CLI)
        // This will be added dynamically when known
        
        // 2. User-specific config directory
        if let Some(config_dir) = dirs::config_dir() {
            search_paths.push(config_dir.join("devkit-env").join("config.toml"));
        }
        
        // 3. Current working directory
        search_paths.push(PathBuf::from("config.toml"));
        
        // 4. Project root (look for .git and check parent directories)
        if let Ok(current_dir) = std::env::current_dir() {
            if let Some(project_root) = find_project_root(&current_dir) {
                search_paths.push(project_root.join("config.toml"));
            }
        }
        
        // 5. System-wide config (if exists)
        search_paths.push(PathBuf::from("/etc/devkit/config.toml"));
        
        Self { search_paths }
    }
    
    /// Add a search path with highest priority
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.insert(0, path);
    }
    
    /// Load configuration with fallback support
    pub fn load_with_fallback(&self) -> Result<ConfigLoadResult, ConfigError> {
        let mut errors = Vec::new();
        let mut attempted_paths = Vec::new();
        
        // Try each search path in order
        for path in &self.search_paths {
            attempted_paths.push(path.clone());
            
            if !path.exists() {
                debug!("Config file does not exist: {}", path.display());
                continue;
            }
            
            match self.load_from_file(path) {
                Ok(config) => {
                    info!("Successfully loaded config from: {}", path.display());
                    return Ok(ConfigLoadResult {
                        config,
                        loaded_from: path.clone(),
                        fallback_used: !errors.is_empty(),
                        errors,
                    });
                }
                Err(err) => {
                    warn!("Failed to load config from {}: {}", path.display(), err);
                    errors.push((path.clone(), err));
                    continue;
                }
            }
        }
        
        // If all configs failed, try to create a default config
        warn!("All configuration files failed to load, using defaults");
        
        // Try to save default config to the first writable location
        let default_config = Config::default();
        let mut save_path = None;
        
        for path in &self.search_paths {
            if let Some(parent) = path.parent() {
                if parent.exists() || std::fs::create_dir_all(parent).is_ok() {
                    if self.save_to_file(&default_config, path).is_ok() {
                        info!("Created default config at: {}", path.display());
                        save_path = Some(path.clone());
                        break;
                    }
                }
            }
        }
        
        Ok(ConfigLoadResult {
            config: default_config,
            loaded_from: save_path.unwrap_or_else(|| PathBuf::from("<default>")),
            fallback_used: true,
            errors,
        })
    }
    
    /// Load configuration from a specific file (original method)
    pub fn load_from_file(&self, path: &PathBuf) -> Result<Config, ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::IOError(e))?;
        
        let config: Config = toml::from_str(&content)
            .map_err(|e| ConfigError::DeserializationError(e))?;
        
        Ok(config)
    }
    
    /// Load and validate configuration with fallback
    pub fn load_and_validate_with_fallback(&self, validator: &crate::config::validation::ConfigValidator) -> Result<ConfigLoadResult, ConfigError> {
        let mut result = self.load_with_fallback()?;
        
        // Try to validate the loaded config
        match validator.validate(&result.config) {
            Ok(()) => Ok(result),
            Err(validation_err) => {
                warn!("Config validation failed for {}: {}", result.loaded_from.display(), validation_err);
                
                // If validation fails, try to merge with defaults to fill missing fields
                let mut default_config = Config::default();
                if let Ok(merged) = self.merge_with_defaults(&result.config, &default_config) {
                    if validator.validate(&merged).is_ok() {
                        info!("Successfully merged config with defaults");
                        result.config = merged;
                        result.fallback_used = true;
                        return Ok(result);
                    }
                }
                
                // If merging fails, use pure defaults
                warn!("Using pure default configuration due to validation failures");
                result.config = default_config;
                result.fallback_used = true;
                result.errors.push((result.loaded_from.clone(), validation_err));
                Ok(result)
            }
        }
    }
    
    /// Merge a config with defaults, filling in missing fields
    fn merge_with_defaults(&self, config: &Config, defaults: &Config) -> Result<Config, ConfigError> {
        // This is a simplified merge - in a real implementation, you'd want
        // to do a deep merge of the configuration structures
        let config_json = serde_json::to_value(config)
            .map_err(|e| ConfigError::ParseError(format!("Failed to serialize config: {}", e)))?;
        let defaults_json = serde_json::to_value(defaults)
            .map_err(|e| ConfigError::ParseError(format!("Failed to serialize defaults: {}", e)))?;
        
        let merged = merge_json_objects(defaults_json, config_json);
        let merged_config: Config = serde_json::from_value(merged)
            .map_err(|e| ConfigError::ParseError(format!("Merge failed: {}", e)))?;
        
        Ok(merged_config)
    }

    pub fn save_to_file(&self, config: &Config, path: &PathBuf) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(config)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Find the project root by looking for .git directory
fn find_project_root(start_dir: &PathBuf) -> Option<PathBuf> {
    let mut current = start_dir.clone();
    
    loop {
        if current.join(".git").exists() {
            return Some(current);
        }
        
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }
    
    None
}

/// Deep merge two JSON objects, with the second taking priority
fn merge_json_objects(mut base: serde_json::Value, overlay: serde_json::Value) -> serde_json::Value {
    match (base, overlay) {
        (serde_json::Value::Object(ref mut base_map), serde_json::Value::Object(overlay_map)) => {
            for (key, value) in overlay_map {
                base_map.entry(key).and_modify(|base_value| {
                    *base_value = merge_json_objects(base_value.clone(), value.clone());
                }).or_insert(value);
            }
            serde_json::Value::Object(base_map.clone())
        }
        (_, overlay) => overlay, // Non-object values: overlay wins
    }
}
