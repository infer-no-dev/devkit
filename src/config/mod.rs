//! Configuration management for the agentic development environment.
//!
//! This module handles user preferences, settings, and configuration files
//! for all aspects of the development environment.

pub mod defaults;
pub mod loader;
pub mod settings;
pub mod validation;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub agents: AgentConfig,
    pub codegen: CodegenConfig,
    pub shell: ShellConfig,
    pub ui: UIConfig,
    pub web: WebConfig,
    pub logging: crate::logging::LogConfig,
    pub keybindings: HashMap<String, String>,
}

/// General application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub workspace_path: Option<PathBuf>,
    pub log_level: String,
    pub auto_save: bool,
    pub backup_enabled: bool,
    pub telemetry_enabled: bool,
}

/// Agent system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub max_concurrent_agents: usize,
    pub agent_timeout_seconds: u64,
    pub default_agent_priority: String,
    pub notification_settings: NotificationConfig,
    pub custom_agents: Vec<CustomAgentConfig>,
}

/// Custom agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAgentConfig {
    pub name: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub priority: String,
    pub settings: HashMap<String, serde_json::Value>,
}

/// Notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub enabled: bool,
    pub sound_enabled: bool,
    pub desktop_notifications: bool,
    pub auto_dismiss_timeout: u64,
}

/// Code generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodegenConfig {
    pub default_style: StyleConfig,
    pub language_preferences: HashMap<String, LanguageConfig>,
    pub template_directories: Vec<PathBuf>,
    pub ai_model_settings: AIModelConfig,
}

/// Code style configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleConfig {
    pub indentation: String, // \"spaces\" or \"tabs\"
    pub indent_size: usize,
    pub line_length: usize,
    pub naming_convention: String,
    pub include_comments: bool,
    pub include_type_hints: bool,
}

/// Language-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    pub style: StyleConfig,
    pub formatter: Option<String>,
    pub linter: Option<String>,
    pub specific_settings: HashMap<String, serde_json::Value>,
}

/// AI model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIModelConfig {
    pub default_provider: String,
    pub default_model: String,
    pub ollama: OllamaConfig,
    pub openai: Option<OpenAIConfig>,
    pub anthropic: Option<AnthropicConfig>,
    pub context_window_size: usize,
    pub temperature: f64,
    pub max_tokens: usize,
}

/// Ollama-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub endpoint: String,
    pub timeout_seconds: u64,
    pub max_retries: usize,
    pub default_model: Option<String>,
}

/// OpenAI-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub organization: Option<String>,
    pub default_model: String,
}

/// Anthropic-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicConfig {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub default_model: String,
}

/// Shell integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    pub preferred_shell: Option<String>,
    pub environment_variables: HashMap<String, String>,
    pub command_timeout: u64,
    pub history_enabled: bool,
    pub custom_commands: HashMap<String, String>,
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    pub theme: String,
    pub color_scheme: String,
    pub font_size: u16,
    pub show_line_numbers: bool,
    pub show_timestamps: bool,
    pub auto_scroll: bool,
    pub panel_layout: PanelLayoutConfig,
}

/// Panel layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelLayoutConfig {
    pub output_panel_percentage: u16,
    pub agent_panel_percentage: u16,
    pub notification_panel_height: u16,
    pub input_panel_height: u16,
}

/// Web dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub cors_enabled: bool,
    pub static_files_path: Option<String>,
    pub auth_enabled: bool,
    pub auth_token: Option<String>,
    pub session_timeout_minutes: u32,
}

/// Configuration manager with enhanced features
#[derive(Debug)]
pub struct ConfigManager {
    config: Config,
    config_path: PathBuf,
    loader: loader::ConfigLoader,
    validator: validation::ConfigValidator,
    environment: String,
    watch_enabled: bool,
    config_cache: Option<Config>,
    last_modified: Option<std::time::SystemTime>,
}

/// Errors that can occur during configuration operations
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Failed to parse configuration: {0}")]
    ParseError(String),

    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] toml::ser::Error),

    #[error("Deserialization error: {0}")]
    DeserializationError(#[from] toml::de::Error),

    #[error("Hot reload failed: {0}")]
    HotReloadError(String),

    #[error("Environment configuration not found: {0}")]
    EnvironmentNotFound(String),

    #[error("Merge error: {0}")]
    MergeError(String),
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new(config_path: Option<PathBuf>) -> Result<Self, ConfigError> {
        Self::with_environment(config_path, "default")
    }

    /// Create a new configuration manager with smart defaults
    pub fn new_with_smart_defaults(config_path: Option<PathBuf>) -> Result<Self, ConfigError> {
        let system_defaults = defaults::SystemDefaults::detect();
        let config_path = config_path.unwrap_or_else(|| {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("devkit-env")
                .join("config.toml")
        });

        let loader = loader::ConfigLoader::new();
        let validator = validation::ConfigValidator::new();

        let mut manager = Self {
            config: system_defaults.generate_config(),
            config_path: config_path.clone(),
            loader,
            validator,
            environment: "default".to_string(),
            watch_enabled: false,
            config_cache: None,
            last_modified: None,
        };

        // Load existing config if it exists and merge with smart defaults
        if config_path.exists() {
            let existing_config = manager.loader.load_from_file(&config_path)?;
            manager.merge_configs(existing_config)?;
        }

        manager.validator.validate(&manager.config)?;

        if manager.config_path.exists() {
            manager.update_last_modified()?;
        }

        Ok(manager)
    }

    /// Create a new configuration manager for a specific environment
    pub fn with_environment(
        config_path: Option<PathBuf>,
        environment: &str,
    ) -> Result<Self, ConfigError> {
        let config_path = config_path.unwrap_or_else(|| {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("devkit-env")
                .join("config.toml")
        });

        let loader = loader::ConfigLoader::new();
        let validator = validation::ConfigValidator::new();

        let mut manager = Self {
            config: Config::default(),
            config_path: config_path.clone(),
            loader,
            validator,
            environment: environment.to_string(),
            watch_enabled: false,
            config_cache: None,
            last_modified: None,
        };

        // Load base configuration
        if config_path.exists() {
            manager.config = manager.loader.load_from_file(&config_path)?;
        }

        // Load environment-specific overrides
        let env_config_path = manager.get_environment_config_path();
        if env_config_path.exists() {
            let env_config = manager.loader.load_from_file(&env_config_path)?;
            manager.merge_configs(env_config)?;
        }

        manager.validator.validate(&manager.config)?;

        // Only update last modified if the config file exists
        if manager.config_path.exists() {
            manager.update_last_modified()?;
        }

        Ok(manager)
    }

    /// Load configuration from file
    pub fn load(&mut self) -> Result<(), ConfigError> {
        if self.config_path.exists() {
            self.config = self.loader.load_from_file(&self.config_path)?;
            self.validator.validate(&self.config)?;
        }
        Ok(())
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), ConfigError> {
        // Ensure the directory exists
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        self.loader.save_to_file(&self.config, &self.config_path)?;
        Ok(())
    }

    /// Get the current configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get the config file path
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    /// Update a configuration value
    pub fn update<F>(&mut self, updater: F) -> Result<(), ConfigError>
    where
        F: FnOnce(&mut Config),
    {
        updater(&mut self.config);
        self.validator.validate(&self.config)?;
        Ok(())
    }

    /// Reset to default configuration
    pub fn reset_to_default(&mut self) {
        self.config = Config::default();
    }

    /// Export configuration as JSON
    pub fn export_as_json(&self) -> Result<String, ConfigError> {
        serde_json::to_string_pretty(&self.config)
            .map_err(|e| ConfigError::ParseError(format!("JSON serialization failed: {}", e)))
    }

    /// Import configuration from JSON
    pub fn import_from_json(&mut self, json: &str) -> Result<(), ConfigError> {
        let config: Config = serde_json::from_str(json)
            .map_err(|e| ConfigError::ParseError(format!("JSON parsing failed: {}", e)))?;

        self.validator.validate(&config)?;
        self.config = config;
        Ok(())
    }

    /// Enable hot-reloading of configuration files
    pub fn enable_hot_reload(&mut self) {
        self.watch_enabled = true;
        self.config_cache = Some(self.config.clone());
    }

    /// Disable hot-reloading
    pub fn disable_hot_reload(&mut self) {
        self.watch_enabled = false;
        self.config_cache = None;
    }

    /// Check for configuration file changes and reload if necessary
    pub fn check_and_reload(&mut self) -> Result<bool, ConfigError> {
        if !self.watch_enabled {
            return Ok(false);
        }

        let current_modified = self.get_file_modified_time()?;

        if let Some(last_modified) = self.last_modified {
            if current_modified <= last_modified {
                return Ok(false); // No changes
            }
        }

        // File has been modified, reload configuration
        self.load()?;
        self.last_modified = Some(current_modified);

        Ok(true)
    }

    /// Get the current environment name
    pub fn environment(&self) -> &str {
        &self.environment
    }

    /// Switch to a different environment
    pub fn switch_environment(&mut self, environment: &str) -> Result<(), ConfigError> {
        self.environment = environment.to_string();

        // Reload configuration with new environment
        let base_config = if self.config_path.exists() {
            self.loader.load_from_file(&self.config_path)?
        } else {
            Config::default()
        };

        self.config = base_config;

        // Load environment-specific overrides
        let env_config_path = self.get_environment_config_path();
        if env_config_path.exists() {
            let env_config = self.loader.load_from_file(&env_config_path)?;
            self.merge_configs(env_config)?;
        }

        self.validator.validate(&self.config)?;
        Ok(())
    }

    /// Get available environments
    pub fn available_environments(&self) -> Result<Vec<String>, ConfigError> {
        let mut environments = vec!["default".to_string()];

        if let Some(parent) = self.config_path.parent() {
            if let Ok(entries) = std::fs::read_dir(parent) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with("config.")
                            && name.ends_with(".toml")
                            && name != "config.toml"
                        {
                            if let Some(env_name) = name
                                .strip_prefix("config.")
                                .and_then(|n| n.strip_suffix(".toml"))
                            {
                                environments.push(env_name.to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok(environments)
    }

    /// Create a new environment configuration with smart defaults
    pub fn create_environment_with_defaults(
        &self,
        environment: &str,
        base_on_current: bool,
    ) -> Result<(), ConfigError> {
        let env_config_path = self.get_environment_config_path_for(environment);

        if env_config_path.exists() {
            return Err(ConfigError::ValidationError(format!(
                "Environment '{}' already exists",
                environment
            )));
        }

        let config_to_save = if base_on_current {
            self.config.clone()
        } else {
            let system_defaults = defaults::SystemDefaults::detect();
            system_defaults.generate_environment_config(environment)
        };

        if let Some(parent) = env_config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        self.loader
            .save_to_file(&config_to_save, &env_config_path)?;
        Ok(())
    }

    /// Create a new environment configuration
    pub fn create_environment(
        &self,
        environment: &str,
        base_on_current: bool,
    ) -> Result<(), ConfigError> {
        let env_config_path = self.get_environment_config_path_for(environment);

        if env_config_path.exists() {
            return Err(ConfigError::ValidationError(format!(
                "Environment '{}' already exists",
                environment
            )));
        }

        let config_to_save = if base_on_current {
            self.config.clone()
        } else {
            Config::default()
        };

        if let Some(parent) = env_config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        self.loader
            .save_to_file(&config_to_save, &env_config_path)?;
        Ok(())
    }

    /// Delete an environment configuration
    pub fn delete_environment(&self, environment: &str) -> Result<(), ConfigError> {
        if environment == "default" {
            return Err(ConfigError::ValidationError(
                "Cannot delete the default environment".to_string(),
            ));
        }

        let env_config_path = self.get_environment_config_path_for(environment);

        if !env_config_path.exists() {
            return Err(ConfigError::EnvironmentNotFound(environment.to_string()));
        }

        std::fs::remove_file(env_config_path)?;
        Ok(())
    }

    /// Validate current configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.validator.validate(&self.config)
    }

    /// Apply a configuration profile preset
    pub fn apply_profile(&mut self, profile: &str) -> Result<(), ConfigError> {
        let new_config = match profile {
            "minimal" => defaults::ConfigProfiles::minimal(),
            "performance" => defaults::ConfigProfiles::performance(),
            "privacy" => defaults::ConfigProfiles::privacy(),
            "collaborative" => defaults::ConfigProfiles::collaborative(),
            _ => {
                return Err(ConfigError::ValidationError(
                    format!("Unknown profile '{}'. Available profiles: minimal, performance, privacy, collaborative", profile)
                ));
            }
        };

        self.validator.validate(&new_config)?;
        self.config = new_config;
        Ok(())
    }

    /// Get system capabilities information
    pub fn get_system_info(&self) -> defaults::SystemDefaults {
        defaults::SystemDefaults::detect()
    }

    /// Get configuration value by path (dot notation)
    pub fn get_value(&self, path: &str) -> Option<serde_json::Value> {
        let config_json = serde_json::to_value(&self.config).ok()?;
        self.get_nested_value(&config_json, path)
    }

    /// Set configuration value by path (dot notation)
    pub fn set_value(&mut self, path: &str, value: serde_json::Value) -> Result<(), ConfigError> {
        let mut config_json = serde_json::to_value(&self.config)
            .map_err(|e| ConfigError::ParseError(format!("Failed to serialize config: {}", e)))?;

        self.set_nested_value(&mut config_json, path, value)?;

        self.config = serde_json::from_value(config_json)
            .map_err(|e| ConfigError::ParseError(format!("Failed to deserialize config: {}", e)))?;

        self.validator.validate(&self.config)?;
        Ok(())
    }

    /// Get backup configuration
    pub fn get_backup(&self) -> Option<&Config> {
        self.config_cache.as_ref()
    }

    /// Restore from backup
    pub fn restore_from_backup(&mut self) -> Result<(), ConfigError> {
        if let Some(backup) = self.config_cache.take() {
            self.config = backup;
            self.validator.validate(&self.config)?;
            Ok(())
        } else {
            Err(ConfigError::ValidationError(
                "No backup available".to_string(),
            ))
        }
    }

    // Private helper methods

    fn get_environment_config_path(&self) -> PathBuf {
        self.get_environment_config_path_for(&self.environment)
    }

    fn get_environment_config_path_for(&self, environment: &str) -> PathBuf {
        if environment == "default" {
            self.config_path.clone()
        } else {
            let parent = self
                .config_path
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            parent.join(format!("config.{}.toml", environment))
        }
    }

    fn merge_configs(&mut self, override_config: Config) -> Result<(), ConfigError> {
        // Simple merge - in a real implementation this would be more sophisticated
        // For now, we'll just use the override config directly
        // TODO: Implement proper deep merging

        let base_json = serde_json::to_value(&self.config).map_err(|e| {
            ConfigError::MergeError(format!("Failed to serialize base config: {}", e))
        })?;

        let override_json = serde_json::to_value(&override_config).map_err(|e| {
            ConfigError::MergeError(format!("Failed to serialize override config: {}", e))
        })?;

        let merged = self.merge_json_values(base_json, override_json)?;

        self.config = serde_json::from_value(merged).map_err(|e| {
            ConfigError::MergeError(format!("Failed to deserialize merged config: {}", e))
        })?;

        Ok(())
    }

    fn merge_json_values(
        &self,
        base: serde_json::Value,
        override_val: serde_json::Value,
    ) -> Result<serde_json::Value, ConfigError> {
        match (base, override_val) {
            (serde_json::Value::Object(mut base_obj), serde_json::Value::Object(override_obj)) => {
                for (key, value) in override_obj {
                    base_obj.insert(key, value);
                }
                Ok(serde_json::Value::Object(base_obj))
            }
            (_, override_val) => Ok(override_val),
        }
    }

    fn get_file_modified_time(&self) -> Result<std::time::SystemTime, ConfigError> {
        let metadata = std::fs::metadata(&self.config_path)?;
        Ok(metadata.modified()?)
    }

    fn update_last_modified(&mut self) -> Result<(), ConfigError> {
        self.last_modified = Some(self.get_file_modified_time()?);
        Ok(())
    }

    fn get_nested_value(&self, json: &serde_json::Value, path: &str) -> Option<serde_json::Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = json;

        for part in parts {
            match current {
                serde_json::Value::Object(obj) => {
                    current = obj.get(part)?;
                }
                _ => return None,
            }
        }

        Some(current.clone())
    }

    fn set_nested_value(
        &self,
        json: &mut serde_json::Value,
        path: &str,
        value: serde_json::Value,
    ) -> Result<(), ConfigError> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return Err(ConfigError::ValidationError("Empty path".to_string()));
        }

        let mut current = json;

        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part, set the value
                match current {
                    serde_json::Value::Object(ref mut obj) => {
                        obj.insert(part.to_string(), value);
                        return Ok(());
                    }
                    _ => {
                        return Err(ConfigError::ValidationError(
                            "Cannot set value on non-object".to_string(),
                        ))
                    }
                }
            } else {
                // Navigate deeper
                match current {
                    serde_json::Value::Object(ref mut obj) => {
                        current = obj
                            .entry(part.to_string())
                            .or_insert_with(|| serde_json::json!({}));
                    }
                    _ => {
                        return Err(ConfigError::ValidationError(
                            "Cannot navigate through non-object".to_string(),
                        ))
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            agents: AgentConfig::default(),
            codegen: CodegenConfig::default(),
            shell: ShellConfig::default(),
            ui: UIConfig::default(),
            web: WebConfig::default(),
            logging: crate::logging::LogConfig::default(),
            keybindings: default_keybindings(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            workspace_path: None,
            log_level: "info".to_string(),
            auto_save: true,
            backup_enabled: true,
            telemetry_enabled: false,
        }
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_concurrent_agents: 5,
            agent_timeout_seconds: 300,
            default_agent_priority: "normal".to_string(),
            notification_settings: NotificationConfig::default(),
            custom_agents: Vec::new(),
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sound_enabled: false,
            desktop_notifications: true,
            auto_dismiss_timeout: 5000,
        }
    }
}

impl Default for CodegenConfig {
    fn default() -> Self {
        Self {
            default_style: StyleConfig::default(),
            language_preferences: HashMap::new(),
            template_directories: Vec::new(),
            ai_model_settings: AIModelConfig::default(),
        }
    }
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            indentation: "spaces".to_string(),
            indent_size: 4,
            line_length: 100,
            naming_convention: "auto".to_string(),
            include_comments: true,
            include_type_hints: true,
        }
    }
}

impl Default for AIModelConfig {
    fn default() -> Self {
        Self {
            default_provider: "ollama".to_string(),
            default_model: "llama3.2:latest".to_string(),
            ollama: OllamaConfig::default(),
            openai: None,
            anthropic: None,
            context_window_size: 8192,
            temperature: 0.7,
            max_tokens: 1000,
        }
    }
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:11434".to_string(),
            timeout_seconds: 300,
            max_retries: 3,
            default_model: Some("llama3.2:latest".to_string()),
        }
    }
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            preferred_shell: None,
            environment_variables: HashMap::new(),
            command_timeout: 30,
            history_enabled: true,
            custom_commands: HashMap::new(),
        }
    }
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            color_scheme: "dark".to_string(),
            font_size: 14,
            show_line_numbers: true,
            show_timestamps: true,
            auto_scroll: true,
            panel_layout: PanelLayoutConfig::default(),
        }
    }
}

impl Default for PanelLayoutConfig {
    fn default() -> Self {
        Self {
            output_panel_percentage: 70,
            agent_panel_percentage: 30,
            notification_panel_height: 5,
            input_panel_height: 3,
        }
    }
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: "127.0.0.1".to_string(),
            port: 8080,
            cors_enabled: true,
            static_files_path: None,
            auth_enabled: false,
            auth_token: None,
            session_timeout_minutes: 60,
        }
    }
}

/// Default keybindings
fn default_keybindings() -> HashMap<String, String> {
    let mut bindings = HashMap::new();

    // Navigation
    bindings.insert("quit".to_string(), "q".to_string());
    bindings.insert("input_mode".to_string(), "i".to_string());
    bindings.insert("command_mode".to_string(), ":".to_string());
    bindings.insert("agent_view".to_string(), "a".to_string());
    bindings.insert("settings".to_string(), "s".to_string());

    // Input handling
    bindings.insert("escape".to_string(), "Esc".to_string());
    bindings.insert("enter".to_string(), "Enter".to_string());
    bindings.insert("backspace".to_string(), "Backspace".to_string());

    // Scrolling
    bindings.insert("scroll_up".to_string(), "k".to_string());
    bindings.insert("scroll_down".to_string(), "j".to_string());
    bindings.insert("page_up".to_string(), "u".to_string());
    bindings.insert("page_down".to_string(), "d".to_string());

    bindings
}
