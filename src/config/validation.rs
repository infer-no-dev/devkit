//! Configuration validation utilities.

use crate::config::{Config, ConfigError};
use std::collections::HashSet;

/// Configuration validator with comprehensive validation rules
#[derive(Debug)]
pub struct ConfigValidator {
    valid_log_levels: HashSet<String>,
    valid_themes: HashSet<String>,
    valid_color_schemes: HashSet<String>,
    valid_naming_conventions: HashSet<String>,
    valid_indentation_types: HashSet<String>,
    valid_ai_providers: HashSet<String>,
}

impl ConfigValidator {
    pub fn new() -> Self {
        let mut valid_log_levels = HashSet::new();
        valid_log_levels.insert("error".to_string());
        valid_log_levels.insert("warn".to_string());
        valid_log_levels.insert("info".to_string());
        valid_log_levels.insert("debug".to_string());
        valid_log_levels.insert("trace".to_string());
        
        let mut valid_themes = HashSet::new();
        valid_themes.insert("default".to_string());
        valid_themes.insert("dark".to_string());
        valid_themes.insert("light".to_string());
        valid_themes.insert("high-contrast".to_string());
        
        let mut valid_color_schemes = HashSet::new();
        valid_color_schemes.insert("dark".to_string());
        valid_color_schemes.insert("light".to_string());
        valid_color_schemes.insert("auto".to_string());
        
        let mut valid_naming_conventions = HashSet::new();
        valid_naming_conventions.insert("auto".to_string());
        valid_naming_conventions.insert("camelCase".to_string());
        valid_naming_conventions.insert("PascalCase".to_string());
        valid_naming_conventions.insert("snake_case".to_string());
        valid_naming_conventions.insert("kebab-case".to_string());
        valid_naming_conventions.insert("SCREAMING_SNAKE_CASE".to_string());
        
        let mut valid_indentation_types = HashSet::new();
        valid_indentation_types.insert("spaces".to_string());
        valid_indentation_types.insert("tabs".to_string());
        
        let mut valid_ai_providers = HashSet::new();
        valid_ai_providers.insert("ollama".to_string());
        valid_ai_providers.insert("openai".to_string());
        valid_ai_providers.insert("anthropic".to_string());
        
        Self {
            valid_log_levels,
            valid_themes,
            valid_color_schemes,
            valid_naming_conventions,
            valid_indentation_types,
            valid_ai_providers,
        }
    }
    
    /// Comprehensive configuration validation
    pub fn validate(&self, config: &Config) -> Result<(), ConfigError> {
        self.validate_general(&config.general)?;
        self.validate_agents(&config.agents)?;
        self.validate_codegen(&config.codegen)?;
        self.validate_shell(&config.shell)?;
        self.validate_ui(&config.ui)?;
        self.validate_keybindings(&config.keybindings)?;
        Ok(())
    }
    
    /// Validate general configuration
    fn validate_general(&self, general: &crate::config::GeneralConfig) -> Result<(), ConfigError> {
        if general.log_level.is_empty() {
            return Err(ConfigError::ValidationError("log_level cannot be empty".to_string()));
        }
        
        if !self.valid_log_levels.contains(&general.log_level) {
            return Err(ConfigError::ValidationError(
                format!("Invalid log_level '{}'. Valid options are: {:?}", 
                    general.log_level, self.valid_log_levels)
            ));
        }
        
        if let Some(workspace_path) = &general.workspace_path {
            if !workspace_path.exists() {
                return Err(ConfigError::ValidationError(
                    format!("Workspace path does not exist: {:?}", workspace_path)
                ));
            }
            
            if !workspace_path.is_dir() {
                return Err(ConfigError::ValidationError(
                    format!("Workspace path is not a directory: {:?}", workspace_path)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validate agent configuration
    fn validate_agents(&self, agents: &crate::config::AgentConfig) -> Result<(), ConfigError> {
        if agents.max_concurrent_agents == 0 {
            return Err(ConfigError::ValidationError(
                "max_concurrent_agents must be greater than 0".to_string()
            ));
        }
        
        if agents.max_concurrent_agents > 100 {
            return Err(ConfigError::ValidationError(
                "max_concurrent_agents cannot exceed 100".to_string()
            ));
        }
        
        if agents.agent_timeout_seconds == 0 {
            return Err(ConfigError::ValidationError(
                "agent_timeout_seconds must be greater than 0".to_string()
            ));
        }
        
        if agents.agent_timeout_seconds > 3600 {
            return Err(ConfigError::ValidationError(
                "agent_timeout_seconds cannot exceed 3600 (1 hour)".to_string()
            ));
        }
        
        let valid_priorities = ["low", "normal", "high", "critical"];
        if !valid_priorities.contains(&agents.default_agent_priority.as_str()) {
            return Err(ConfigError::ValidationError(
                format!("Invalid default_agent_priority '{}'. Valid options are: {:?}", 
                    agents.default_agent_priority, valid_priorities)
            ));
        }
        
        // Validate custom agents
        for custom_agent in &agents.custom_agents {
            if custom_agent.name.is_empty() {
                return Err(ConfigError::ValidationError(
                    "Custom agent name cannot be empty".to_string()
                ));
            }
            
            if custom_agent.capabilities.is_empty() {
                return Err(ConfigError::ValidationError(
                    format!("Custom agent '{}' must have at least one capability", custom_agent.name)
                ));
            }
            
            if !valid_priorities.contains(&custom_agent.priority.as_str()) {
                return Err(ConfigError::ValidationError(
                    format!("Invalid priority '{}' for custom agent '{}'", 
                        custom_agent.priority, custom_agent.name)
                ));
            }
        }
        
        // Validate notification settings
        if agents.notification_settings.auto_dismiss_timeout > 300000 {
            return Err(ConfigError::ValidationError(
                "auto_dismiss_timeout cannot exceed 300000ms (5 minutes)".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Validate code generation configuration
    fn validate_codegen(&self, codegen: &crate::config::CodegenConfig) -> Result<(), ConfigError> {
        self.validate_style_config(&codegen.default_style)?;
        
        // Validate language preferences
        for (language, lang_config) in &codegen.language_preferences {
            if language.is_empty() {
                return Err(ConfigError::ValidationError(
                    "Language name cannot be empty".to_string()
                ));
            }
            
            self.validate_style_config(&lang_config.style)?;
        }
        
        // Validate template directories
        for template_dir in &codegen.template_directories {
            if !template_dir.exists() {
                return Err(ConfigError::ValidationError(
                    format!("Template directory does not exist: {:?}", template_dir)
                ));
            }
            
            if !template_dir.is_dir() {
                return Err(ConfigError::ValidationError(
                    format!("Template path is not a directory: {:?}", template_dir)
                ));
            }
        }
        
        // Validate AI model settings
        self.validate_ai_model_config(&codegen.ai_model_settings)?;
        
        Ok(())
    }
    
    /// Validate style configuration
    fn validate_style_config(&self, style: &crate::config::StyleConfig) -> Result<(), ConfigError> {
        if !self.valid_indentation_types.contains(&style.indentation) {
            return Err(ConfigError::ValidationError(
                format!("Invalid indentation type '{}'. Valid options are: {:?}", 
                    style.indentation, self.valid_indentation_types)
            ));
        }
        
        if style.indent_size == 0 || style.indent_size > 16 {
            return Err(ConfigError::ValidationError(
                "indent_size must be between 1 and 16".to_string()
            ));
        }
        
        if style.line_length < 40 || style.line_length > 500 {
            return Err(ConfigError::ValidationError(
                "line_length must be between 40 and 500".to_string()
            ));
        }
        
        if !self.valid_naming_conventions.contains(&style.naming_convention) {
            return Err(ConfigError::ValidationError(
                format!("Invalid naming_convention '{}'. Valid options are: {:?}", 
                    style.naming_convention, self.valid_naming_conventions)
            ));
        }
        
        Ok(())
    }
    
    /// Validate AI model configuration
    fn validate_ai_model_config(&self, ai_config: &crate::config::AIModelConfig) -> Result<(), ConfigError> {
        if !self.valid_ai_providers.contains(&ai_config.default_provider) {
            return Err(ConfigError::ValidationError(
                format!("Invalid AI provider '{}'. Valid options are: {:?}", 
                    ai_config.default_provider, self.valid_ai_providers)
            ));
        }
        
        if ai_config.default_model.is_empty() {
            return Err(ConfigError::ValidationError(
                "default_model cannot be empty".to_string()
            ));
        }
        
        if ai_config.context_window_size == 0 || ai_config.context_window_size > 1000000 {
            return Err(ConfigError::ValidationError(
                "context_window_size must be between 1 and 1000000".to_string()
            ));
        }
        
        if ai_config.temperature < 0.0 || ai_config.temperature > 2.0 {
            return Err(ConfigError::ValidationError(
                "temperature must be between 0.0 and 2.0".to_string()
            ));
        }
        
        if ai_config.max_tokens == 0 || ai_config.max_tokens > 100000 {
            return Err(ConfigError::ValidationError(
                "max_tokens must be between 1 and 100000".to_string()
            ));
        }
        
        // Validate Ollama config
        if ai_config.ollama.endpoint.is_empty() {
            return Err(ConfigError::ValidationError(
                "Ollama endpoint cannot be empty".to_string()
            ));
        }
        
        if !ai_config.ollama.endpoint.starts_with("http://") && !ai_config.ollama.endpoint.starts_with("https://") {
            return Err(ConfigError::ValidationError(
                "Ollama endpoint must start with http:// or https://".to_string()
            ));
        }
        
        if ai_config.ollama.timeout_seconds == 0 || ai_config.ollama.timeout_seconds > 3600 {
            return Err(ConfigError::ValidationError(
                "Ollama timeout_seconds must be between 1 and 3600".to_string()
            ));
        }
        
        if ai_config.ollama.max_retries > 10 {
            return Err(ConfigError::ValidationError(
                "Ollama max_retries cannot exceed 10".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Validate shell configuration
    fn validate_shell(&self, shell: &crate::config::ShellConfig) -> Result<(), ConfigError> {
        if shell.command_timeout == 0 || shell.command_timeout > 3600 {
            return Err(ConfigError::ValidationError(
                "command_timeout must be between 1 and 3600 seconds".to_string()
            ));
        }
        
        // Validate custom commands
        for (command_name, command_template) in &shell.custom_commands {
            if command_name.is_empty() {
                return Err(ConfigError::ValidationError(
                    "Custom command name cannot be empty".to_string()
                ));
            }
            
            if command_template.is_empty() {
                return Err(ConfigError::ValidationError(
                    format!("Custom command '{}' template cannot be empty", command_name)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validate UI configuration
    fn validate_ui(&self, ui: &crate::config::UIConfig) -> Result<(), ConfigError> {
        if !self.valid_themes.contains(&ui.theme) {
            return Err(ConfigError::ValidationError(
                format!("Invalid theme '{}'. Valid options are: {:?}", 
                    ui.theme, self.valid_themes)
            ));
        }
        
        if !self.valid_color_schemes.contains(&ui.color_scheme) {
            return Err(ConfigError::ValidationError(
                format!("Invalid color_scheme '{}'. Valid options are: {:?}", 
                    ui.color_scheme, self.valid_color_schemes)
            ));
        }
        
        if ui.font_size < 8 || ui.font_size > 48 {
            return Err(ConfigError::ValidationError(
                "font_size must be between 8 and 48".to_string()
            ));
        }
        
        // Validate panel layout
        let layout = &ui.panel_layout;
        if layout.output_panel_percentage + layout.agent_panel_percentage != 100 {
            return Err(ConfigError::ValidationError(
                "output_panel_percentage + agent_panel_percentage must equal 100".to_string()
            ));
        }
        
        if layout.output_panel_percentage < 20 || layout.output_panel_percentage > 80 {
            return Err(ConfigError::ValidationError(
                "output_panel_percentage must be between 20 and 80".to_string()
            ));
        }
        
        if layout.notification_panel_height > 20 {
            return Err(ConfigError::ValidationError(
                "notification_panel_height cannot exceed 20".to_string()
            ));
        }
        
        if layout.input_panel_height == 0 || layout.input_panel_height > 10 {
            return Err(ConfigError::ValidationError(
                "input_panel_height must be between 1 and 10".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Validate keybindings
    fn validate_keybindings(&self, keybindings: &std::collections::HashMap<String, String>) -> Result<(), ConfigError> {
        let required_bindings = [
            "quit", "input_mode", "escape", "enter"
        ];
        
        for required in &required_bindings {
            if !keybindings.contains_key(*required) {
                return Err(ConfigError::ValidationError(
                    format!("Required keybinding '{}' is missing", required)
                ));
            }
        }
        
        // Check for empty keybindings
        for (action, key) in keybindings {
            if key.is_empty() {
                return Err(ConfigError::ValidationError(
                    format!("Keybinding for action '{}' cannot be empty", action)
                ));
            }
        }
        
        Ok(())
    }
}
