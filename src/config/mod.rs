//! Configuration management for the agentic development environment.
//!
//! This module handles user preferences, settings, and configuration files
//! for all aspects of the development environment.

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
    pub default_model: String,
    pub api_settings: HashMap<String, String>,
    pub context_window_size: usize,
    pub temperature: f64,
    pub max_tokens: usize,
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

/// Configuration manager
#[derive(Debug)]
pub struct ConfigManager {
    config: Config,
    config_path: PathBuf,
    loader: loader::ConfigLoader,
    validator: validation::ConfigValidator,
}

/// Errors that can occur during configuration operations
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error(\"Configuration file not found: {0}\")]
    FileNotFound(PathBuf),
    
    #[error(\"Failed to parse configuration: {0}\")]
    ParseError(String),
    
    #[error(\"Validation failed: {0}\")]
    ValidationError(String),
    
    #[error(\"IO error: {0}\")]
    IOError(#[from] std::io::Error),
    
    #[error(\"Serialization error: {0}\")]
    SerializationError(#[from] toml::ser::Error),
    
    #[error(\"Deserialization error: {0}\")]
    DeserializationError(#[from] toml::de::Error),
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new(config_path: Option<PathBuf>) -> Result<Self, ConfigError> {
        let config_path = config_path.unwrap_or_else(|| {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from(\".\"))
                .join(\"agentic-dev-env\")
                .join(\"config.toml\")
        });
        
        let loader = loader::ConfigLoader::new();
        let validator = validation::ConfigValidator::new();
        
        let config = if config_path.exists() {
            loader.load_from_file(&config_path)?
        } else {
            Config::default()
        };
        
        Ok(Self {
            config,
            config_path,
            loader,
            validator,
        })
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
            .map_err(|e| ConfigError::SerializationError(toml::ser::Error::custom(e.to_string())))
    }
    
    /// Import configuration from JSON
    pub fn import_from_json(&mut self, json: &str) -> Result<(), ConfigError> {
        let config: Config = serde_json::from_str(json)
            .map_err(|e| ConfigError::DeserializationError(toml::de::Error::custom(e.to_string())))?;
        
        self.validator.validate(&config)?;
        self.config = config;
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
            keybindings: default_keybindings(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            workspace_path: None,
            log_level: \"info\".to_string(),
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
            default_agent_priority: \"normal\".to_string(),
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
            indentation: \"spaces\".to_string(),
            indent_size: 4,
            line_length: 100,
            naming_convention: \"auto\".to_string(),
            include_comments: true,
            include_type_hints: true,
        }
    }
}

impl Default for AIModelConfig {
    fn default() -> Self {
        Self {
            default_model: \"gpt-3.5-turbo\".to_string(),
            api_settings: HashMap::new(),
            context_window_size: 4096,
            temperature: 0.7,
            max_tokens: 1000,
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
            theme: \"default\".to_string(),
            color_scheme: \"dark\".to_string(),
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

/// Default keybindings
fn default_keybindings() -> HashMap<String, String> {
    let mut bindings = HashMap::new();
    
    // Navigation
    bindings.insert(\"quit\".to_string(), \"q\".to_string());
    bindings.insert(\"input_mode\".to_string(), \"i\".to_string());
    bindings.insert(\"command_mode\".to_string(), \":\".to_string());
    bindings.insert(\"agent_view\".to_string(), \"a\".to_string());
    bindings.insert(\"settings\".to_string(), \"s\".to_string());
    
    // Input handling
    bindings.insert(\"escape\".to_string(), \"Esc\".to_string());
    bindings.insert(\"enter\".to_string(), \"Enter\".to_string());
    bindings.insert(\"backspace\".to_string(), \"Backspace\".to_string());
    
    // Scrolling
    bindings.insert(\"scroll_up\".to_string(), \"k\".to_string());
    bindings.insert(\"scroll_down\".to_string(), \"j\".to_string());
    bindings.insert(\"page_up\".to_string(), \"u\".to_string());
    bindings.insert(\"page_down\".to_string(), \"d\".to_string());
    
    bindings
}
