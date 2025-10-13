//! Plugin Configuration Management
//!
//! Manages plugin configuration, user preferences, and settings persistence.
//! Provides validation, type safety, and configuration hot-reloading.

use crate::plugins::PluginError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use std::sync::Arc;

/// Plugin configuration manager
#[derive(Debug)]
pub struct PluginConfigManager {
    /// Configuration storage
    configs: Arc<RwLock<HashMap<String, PluginConfig>>>,
    /// Configuration directory
    config_dir: PathBuf,
    /// Global configuration
    global_config: GlobalPluginConfig,
    /// Configuration schema registry
    schemas: HashMap<String, ConfigSchema>,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin ID
    pub plugin_id: String,
    /// Plugin version this config is for
    pub plugin_version: String,
    /// Configuration values
    pub values: HashMap<String, ConfigValue>,
    /// User preferences
    pub preferences: HashMap<String, ConfigValue>,
    /// Configuration metadata
    pub metadata: ConfigMetadata,
}

/// Configuration value with type information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
    Object(HashMap<String, ConfigValue>),
    Null,
}

/// Configuration metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    /// Configuration created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Configuration source
    pub source: ConfigSource,
    /// Configuration version
    pub version: String,
    /// Whether config is user-modified
    pub user_modified: bool,
}

/// Configuration source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigSource {
    /// Default configuration from plugin
    Default,
    /// User configuration file
    UserFile,
    /// Environment variables
    Environment,
    /// Command line arguments
    CommandLine,
    /// API/Runtime configuration
    Runtime,
}

/// Global plugin system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalPluginConfig {
    /// Plugin system enabled
    pub enabled: bool,
    /// Auto-update plugins
    pub auto_update: bool,
    /// Plugin execution timeout (seconds)
    pub execution_timeout: u64,
    /// Maximum plugin memory (MB)
    pub max_memory_mb: u64,
    /// Enable plugin sandboxing
    pub enable_sandbox: bool,
    /// Allowed plugin sources
    pub allowed_sources: Vec<String>,
    /// Plugin cache settings
    pub cache_settings: CacheSettings,
    /// Logging configuration
    pub logging: LoggingConfig,
}

impl Default for GlobalPluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_update: false,
            execution_timeout: 300,
            max_memory_mb: 512,
            enable_sandbox: true,
            allowed_sources: vec![
                "https://plugins.devkit.dev".to_string(),
                "local".to_string(),
            ],
            cache_settings: CacheSettings::default(),
            logging: LoggingConfig::default(),
        }
    }
}

/// Cache settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSettings {
    /// Cache directory
    pub cache_dir: PathBuf,
    /// Maximum cache size (MB)
    pub max_size_mb: u64,
    /// Cache TTL (hours)
    pub ttl_hours: u64,
    /// Auto cleanup enabled
    pub auto_cleanup: bool,
}

impl Default for CacheSettings {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            cache_dir: home.join(".devkit").join("cache"),
            max_size_mb: 1024,
            ttl_hours: 24,
            auto_cleanup: true,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Plugin log level
    pub level: String,
    /// Log to file
    pub log_to_file: bool,
    /// Log file path
    pub log_file: Option<PathBuf>,
    /// Include plugin ID in logs
    pub include_plugin_id: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            log_to_file: true,
            log_file: Some(dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
                .join(".devkit").join("logs").join("plugins.log")),
            include_plugin_id: true,
        }
    }
}

/// Configuration schema for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSchema {
    /// Schema version
    pub version: String,
    /// Configuration fields
    pub fields: HashMap<String, ConfigFieldSchema>,
    /// Required fields
    pub required: Vec<String>,
}

/// Configuration field schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFieldSchema {
    /// Field type
    pub field_type: ConfigFieldType,
    /// Field description
    pub description: String,
    /// Default value
    pub default: Option<ConfigValue>,
    /// Whether field is required
    pub required: bool,
    /// Validation rules
    pub validation: Option<ValidationRules>,
}

/// Configuration field types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigFieldType {
    String,
    Integer,
    Float,
    Boolean,
    Array(Box<ConfigFieldType>),
    Object(HashMap<String, ConfigFieldSchema>),
}

/// Validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    /// Minimum value (for numbers)
    pub min: Option<f64>,
    /// Maximum value (for numbers)
    pub max: Option<f64>,
    /// Minimum length (for strings/arrays)
    pub min_length: Option<usize>,
    /// Maximum length (for strings/arrays)
    pub max_length: Option<usize>,
    /// Pattern (for strings)
    pub pattern: Option<String>,
    /// Allowed values (enum)
    pub allowed_values: Option<Vec<ConfigValue>>,
}

impl PluginConfigManager {
    /// Create a new configuration manager
    pub async fn new(config_dir: PathBuf, global_config: GlobalPluginConfig) -> Result<Self, PluginError> {
        // Ensure configuration directory exists
        fs::create_dir_all(&config_dir)
            .await
            .map_err(|e| PluginError::IoError(format!("Failed to create config directory: {}", e)))?;

        let manager = Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
            config_dir,
            global_config,
            schemas: HashMap::new(),
        };

        info!("Plugin configuration manager initialized");
        Ok(manager)
    }

    /// Load plugin configuration
    pub async fn load_config(&self, plugin_id: &str) -> Result<PluginConfig, PluginError> {
        debug!("Loading configuration for plugin: {}", plugin_id);

        // Check in-memory cache first
        {
            let configs = self.configs.read().await;
            if let Some(config) = configs.get(plugin_id) {
                return Ok(config.clone());
            }
        }

        // Load from file
        let config_path = self.config_dir.join(format!("{}.toml", plugin_id));
        
        let config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .await
                .map_err(|e| PluginError::IoError(format!("Failed to read config file: {}", e)))?;

            toml::from_str::<PluginConfig>(&content)
                .map_err(|e| PluginError::ConfigurationError(format!("Invalid config format: {}", e)))?
        } else {
            // Create default configuration
            self.create_default_config(plugin_id).await?
        };

        // Validate configuration
        self.validate_config(&config).await?;

        // Cache configuration
        {
            let mut configs = self.configs.write().await;
            configs.insert(plugin_id.to_string(), config.clone());
        }

        info!("Configuration loaded for plugin: {}", plugin_id);
        Ok(config)
    }

    /// Save plugin configuration
    pub async fn save_config(&self, config: &PluginConfig) -> Result<(), PluginError> {
        debug!("Saving configuration for plugin: {}", config.plugin_id);

        // Validate configuration
        self.validate_config(config).await?;

        // Update metadata
        let mut updated_config = config.clone();
        updated_config.metadata.updated_at = chrono::Utc::now();
        updated_config.metadata.user_modified = true;

        // Save to file
        let config_path = self.config_dir.join(format!("{}.toml", config.plugin_id));
        let content = toml::to_string_pretty(&updated_config)
            .map_err(|e| PluginError::ConfigurationError(format!("Failed to serialize config: {}", e)))?;

        fs::write(&config_path, content)
            .await
            .map_err(|e| PluginError::IoError(format!("Failed to write config file: {}", e)))?;

        // Update cache
        {
            let mut configs = self.configs.write().await;
            configs.insert(config.plugin_id.clone(), updated_config);
        }

        info!("Configuration saved for plugin: {}", config.plugin_id);
        Ok(())
    }

    /// Get configuration value
    pub async fn get_value<T>(&self, plugin_id: &str, key: &str) -> Result<Option<T>, PluginError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let config = self.load_config(plugin_id).await?;
        
        if let Some(value) = config.values.get(key) {
            let json_value = serde_json::to_value(value)
                .map_err(|e| PluginError::ConfigurationError(format!("Value serialization failed: {}", e)))?;
            
            let typed_value = serde_json::from_value(json_value)
                .map_err(|e| PluginError::ConfigurationError(format!("Value deserialization failed: {}", e)))?;
            
            Ok(Some(typed_value))
        } else {
            Ok(None)
        }
    }

    /// Set configuration value
    pub async fn set_value(&self, plugin_id: &str, key: &str, value: ConfigValue) -> Result<(), PluginError> {
        debug!("Setting configuration value: {}:{} = {:?}", plugin_id, key, value);

        let mut config = self.load_config(plugin_id).await?;
        
        // Validate the new value if schema exists
        if let Some(schema) = self.schemas.get(plugin_id) {
            self.validate_field_value(&schema, key, &value)?;
        }

        config.values.insert(key.to_string(), value);
        self.save_config(&config).await?;

        Ok(())
    }

    /// Get user preference
    pub async fn get_preference<T>(&self, plugin_id: &str, key: &str) -> Result<Option<T>, PluginError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let config = self.load_config(plugin_id).await?;
        
        if let Some(value) = config.preferences.get(key) {
            let json_value = serde_json::to_value(value)
                .map_err(|e| PluginError::ConfigurationError(format!("Preference serialization failed: {}", e)))?;
            
            let typed_value = serde_json::from_value(json_value)
                .map_err(|e| PluginError::ConfigurationError(format!("Preference deserialization failed: {}", e)))?;
            
            Ok(Some(typed_value))
        } else {
            Ok(None)
        }
    }

    /// Set user preference
    pub async fn set_preference(&self, plugin_id: &str, key: &str, value: ConfigValue) -> Result<(), PluginError> {
        debug!("Setting user preference: {}:{} = {:?}", plugin_id, key, value);

        let mut config = self.load_config(plugin_id).await?;
        config.preferences.insert(key.to_string(), value);
        self.save_config(&config).await?;

        Ok(())
    }

    /// Register configuration schema
    pub async fn register_schema(&mut self, plugin_id: &str, schema: ConfigSchema) -> Result<(), PluginError> {
        debug!("Registering configuration schema for plugin: {}", plugin_id);
        
        self.schemas.insert(plugin_id.to_string(), schema);
        
        // Validate existing configuration against new schema
        if let Ok(config) = self.load_config(plugin_id).await {
            if let Err(e) = self.validate_config(&config).await {
                warn!("Existing configuration for plugin {} is invalid: {}", plugin_id, e);
            }
        }

        Ok(())
    }

    /// Reset plugin configuration to defaults
    pub async fn reset_config(&self, plugin_id: &str) -> Result<(), PluginError> {
        debug!("Resetting configuration for plugin: {}", plugin_id);

        let default_config = self.create_default_config(plugin_id).await?;
        self.save_config(&default_config).await?;

        info!("Configuration reset to defaults for plugin: {}", plugin_id);
        Ok(())
    }

    /// Get global configuration
    pub fn get_global_config(&self) -> &GlobalPluginConfig {
        &self.global_config
    }

    /// Update global configuration
    pub async fn update_global_config(&mut self, config: GlobalPluginConfig) -> Result<(), PluginError> {
        debug!("Updating global plugin configuration");

        self.global_config = config.clone();

        // Save global configuration
        let global_config_path = self.config_dir.join("global.toml");
        let content = toml::to_string_pretty(&config)
            .map_err(|e| PluginError::ConfigurationError(format!("Failed to serialize global config: {}", e)))?;

        fs::write(&global_config_path, content)
            .await
            .map_err(|e| PluginError::IoError(format!("Failed to write global config: {}", e)))?;

        info!("Global plugin configuration updated");
        Ok(())
    }

    /// List all plugin configurations
    pub async fn list_configs(&self) -> Result<Vec<String>, PluginError> {
        let mut plugin_ids = Vec::new();
        
        let mut entries = fs::read_dir(&self.config_dir)
            .await
            .map_err(|e| PluginError::IoError(format!("Failed to read config directory: {}", e)))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| PluginError::IoError(format!("Failed to read directory entry: {}", e)))? 
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                    if filename != "global" {
                        plugin_ids.push(filename.to_string());
                    }
                }
            }
        }

        Ok(plugin_ids)
    }

    // Private helper methods

    async fn create_default_config(&self, plugin_id: &str) -> Result<PluginConfig, PluginError> {
        let now = chrono::Utc::now();
        
        let config = PluginConfig {
            plugin_id: plugin_id.to_string(),
            plugin_version: "1.0.0".to_string(), // Default version
            values: self.get_default_values(plugin_id),
            preferences: HashMap::new(),
            metadata: ConfigMetadata {
                created_at: now,
                updated_at: now,
                source: ConfigSource::Default,
                version: "1.0.0".to_string(),
                user_modified: false,
            },
        };

        Ok(config)
    }

    fn get_default_values(&self, plugin_id: &str) -> HashMap<String, ConfigValue> {
        let mut defaults = HashMap::new();
        
        // Add common default values
        defaults.insert("enabled".to_string(), ConfigValue::Boolean(true));
        defaults.insert("timeout".to_string(), ConfigValue::Integer(30));
        defaults.insert("max_retries".to_string(), ConfigValue::Integer(3));
        
        // Add schema-based defaults if available
        if let Some(schema) = self.schemas.get(plugin_id) {
            for (key, field_schema) in &schema.fields {
                if let Some(default_value) = &field_schema.default {
                    defaults.insert(key.clone(), default_value.clone());
                }
            }
        }

        defaults
    }

    async fn validate_config(&self, config: &PluginConfig) -> Result<(), PluginError> {
        // Check if schema exists for this plugin
        if let Some(schema) = self.schemas.get(&config.plugin_id) {
            self.validate_against_schema(config, schema)?;
        }

        // Basic validation
        if config.plugin_id.is_empty() {
            return Err(PluginError::ConfigurationError("Plugin ID cannot be empty".to_string()));
        }

        if config.plugin_version.is_empty() {
            return Err(PluginError::ConfigurationError("Plugin version cannot be empty".to_string()));
        }

        Ok(())
    }

    fn validate_against_schema(&self, config: &PluginConfig, schema: &ConfigSchema) -> Result<(), PluginError> {
        // Check required fields
        for required_field in &schema.required {
            if !config.values.contains_key(required_field) {
                return Err(PluginError::ConfigurationError(
                    format!("Required configuration field missing: {}", required_field)
                ));
            }
        }

        // Validate each configured value
        for (key, value) in &config.values {
            if let Some(field_schema) = schema.fields.get(key) {
                self.validate_field_value_against_schema(field_schema, value)?;
            }
        }

        Ok(())
    }

    fn validate_field_value(&self, schema: &ConfigSchema, key: &str, value: &ConfigValue) -> Result<(), PluginError> {
        if let Some(field_schema) = schema.fields.get(key) {
            self.validate_field_value_against_schema(field_schema, value)?;
        }
        Ok(())
    }

    fn validate_field_value_against_schema(&self, schema: &ConfigFieldSchema, value: &ConfigValue) -> Result<(), PluginError> {
        // Type validation
        match (&schema.field_type, value) {
            (ConfigFieldType::String, ConfigValue::String(_)) => {}
            (ConfigFieldType::Integer, ConfigValue::Integer(_)) => {}
            (ConfigFieldType::Float, ConfigValue::Float(_)) => {}
            (ConfigFieldType::Boolean, ConfigValue::Boolean(_)) => {}
            (ConfigFieldType::Array(_), ConfigValue::Array(_)) => {}
            (ConfigFieldType::Object(_), ConfigValue::Object(_)) => {}
            _ => {
                return Err(PluginError::ConfigurationError(
                    format!("Type mismatch: expected {:?}, got {:?}", schema.field_type, value)
                ));
            }
        }

        // Additional validation rules
        if let Some(validation) = &schema.validation {
            self.validate_rules(value, validation)?;
        }

        Ok(())
    }

    fn validate_rules(&self, value: &ConfigValue, rules: &ValidationRules) -> Result<(), PluginError> {
        match value {
            ConfigValue::String(s) => {
                if let Some(min_len) = rules.min_length {
                    if s.len() < min_len {
                        return Err(PluginError::ConfigurationError(
                            format!("String too short: {} < {}", s.len(), min_len)
                        ));
                    }
                }
                if let Some(max_len) = rules.max_length {
                    if s.len() > max_len {
                        return Err(PluginError::ConfigurationError(
                            format!("String too long: {} > {}", s.len(), max_len)
                        ));
                    }
                }
                if let Some(pattern) = &rules.pattern {
                    // TODO: Implement regex validation
                    debug!("Pattern validation not implemented: {}", pattern);
                }
            }
            ConfigValue::Integer(i) => {
                if let Some(min) = rules.min {
                    if (*i as f64) < min {
                        return Err(PluginError::ConfigurationError(
                            format!("Value too small: {} < {}", i, min)
                        ));
                    }
                }
                if let Some(max) = rules.max {
                    if (*i as f64) > max {
                        return Err(PluginError::ConfigurationError(
                            format!("Value too large: {} > {}", i, max)
                        ));
                    }
                }
            }
            ConfigValue::Float(f) => {
                if let Some(min) = rules.min {
                    if *f < min {
                        return Err(PluginError::ConfigurationError(
                            format!("Value too small: {} < {}", f, min)
                        ));
                    }
                }
                if let Some(max) = rules.max {
                    if *f > max {
                        return Err(PluginError::ConfigurationError(
                            format!("Value too large: {} > {}", f, max)
                        ));
                    }
                }
            }
            ConfigValue::Array(arr) => {
                if let Some(min_len) = rules.min_length {
                    if arr.len() < min_len {
                        return Err(PluginError::ConfigurationError(
                            format!("Array too short: {} < {}", arr.len(), min_len)
                        ));
                    }
                }
                if let Some(max_len) = rules.max_length {
                    if arr.len() > max_len {
                        return Err(PluginError::ConfigurationError(
                            format!("Array too long: {} > {}", arr.len(), max_len)
                        ));
                    }
                }
            }
            _ => {}
        }

        // Check allowed values
        if let Some(allowed_values) = &rules.allowed_values {
            if !allowed_values.contains(value) {
                return Err(PluginError::ConfigurationError(
                    "Value not in allowed values list".to_string()
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_config_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let global_config = GlobalPluginConfig::default();
        
        let manager = PluginConfigManager::new(temp_dir.path().to_path_buf(), global_config).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_default_config_creation() {
        let temp_dir = TempDir::new().unwrap();
        let global_config = GlobalPluginConfig::default();
        let manager = PluginConfigManager::new(temp_dir.path().to_path_buf(), global_config).await.unwrap();

        let config = manager.load_config("test-plugin").await.unwrap();
        assert_eq!(config.plugin_id, "test-plugin");
        assert!(!config.values.is_empty());
    }

    #[tokio::test]
    async fn test_config_value_operations() {
        let temp_dir = TempDir::new().unwrap();
        let global_config = GlobalPluginConfig::default();
        let manager = PluginConfigManager::new(temp_dir.path().to_path_buf(), global_config).await.unwrap();

        // Set a configuration value
        manager.set_value("test-plugin", "test_key", ConfigValue::String("test_value".to_string())).await.unwrap();

        // Get the configuration value
        let value: Option<String> = manager.get_value("test-plugin", "test_key").await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));
    }

    #[tokio::test]
    async fn test_user_preferences() {
        let temp_dir = TempDir::new().unwrap();
        let global_config = GlobalPluginConfig::default();
        let manager = PluginConfigManager::new(temp_dir.path().to_path_buf(), global_config).await.unwrap();

        // Set a user preference
        manager.set_preference("test-plugin", "theme", ConfigValue::String("dark".to_string())).await.unwrap();

        // Get the user preference
        let preference: Option<String> = manager.get_preference("test-plugin", "theme").await.unwrap();
        assert_eq!(preference, Some("dark".to_string()));
    }
}