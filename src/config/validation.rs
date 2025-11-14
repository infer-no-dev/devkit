//! Configuration validation utilities.

use crate::config::{Config, ConfigError};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Validation function to check OpenAI provider configuration
fn validate_openai_provider_config(config: &Config) -> Result<(), ConfigError> {
    if config.codegen.ai_model_settings.default_provider == "openai"
        && config.codegen.ai_model_settings.openai.is_none()
    {
        return Err(ConfigError::ValidationError(
            "OpenAI is set as the default provider but no OpenAI configuration is provided"
                .to_string(),
        ));
    }
    Ok(())
}

/// Validation function to check Anthropic provider configuration
fn validate_anthropic_provider_config(config: &Config) -> Result<(), ConfigError> {
    if config.codegen.ai_model_settings.default_provider == "anthropic"
        && config.codegen.ai_model_settings.anthropic.is_none()
    {
        return Err(ConfigError::ValidationError(
            "Anthropic is set as the default provider but no Anthropic configuration is provided"
                .to_string(),
        ));
    }
    Ok(())
}

/// Validation function to check default model consistency
fn validate_default_model_consistency(config: &Config) -> Result<(), ConfigError> {
    match config.codegen.ai_model_settings.default_provider.as_str() {
        "openai" => {
            if let Some(openai_config) = &config.codegen.ai_model_settings.openai {
                if config.codegen.ai_model_settings.default_model != openai_config.default_model {
                    return Err(ConfigError::ValidationError(format!(
                        "The default_model '{}' doesn't match the OpenAI default_model '{}'",
                        config.codegen.ai_model_settings.default_model, openai_config.default_model
                    )));
                }
            }
        }
        "anthropic" => {
            if let Some(anthropic_config) = &config.codegen.ai_model_settings.anthropic {
                if config.codegen.ai_model_settings.default_model != anthropic_config.default_model
                {
                    return Err(ConfigError::ValidationError(format!(
                        "The default_model '{}' doesn't match the Anthropic default_model '{}'",
                        config.codegen.ai_model_settings.default_model,
                        anthropic_config.default_model
                    )));
                }
            }
        }
        "ollama" => {
            if let Some(ollama_model) = &config.codegen.ai_model_settings.ollama.default_model {
                if config.codegen.ai_model_settings.default_model != *ollama_model {
                    return Err(ConfigError::ValidationError(format!(
                        "The default_model '{}' doesn't match the Ollama default_model '{}'",
                        config.codegen.ai_model_settings.default_model, ollama_model
                    )));
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// Configuration validator with comprehensive validation rules
pub struct ConfigValidator {
    valid_log_levels: HashSet<String>,
    valid_themes: HashSet<String>,
    valid_color_schemes: HashSet<String>,
    valid_naming_conventions: HashSet<String>,
    valid_indentation_types: HashSet<String>,
    valid_ai_providers: HashSet<String>,
    valid_agent_priorities: HashSet<String>,
    valid_shells: HashSet<String>,
    required_keybindings: HashSet<String>,
    interdependent_validations: Vec<fn(&Config) -> Result<(), ConfigError>>,
}

impl ConfigValidator {
    /// Creates a new configuration validator with all validation rules
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

        let mut valid_agent_priorities = HashSet::new();
        valid_agent_priorities.insert("low".to_string());
        valid_agent_priorities.insert("normal".to_string());
        valid_agent_priorities.insert("high".to_string());
        valid_agent_priorities.insert("critical".to_string());

        let mut valid_shells = HashSet::new();
        valid_shells.insert("bash".to_string());
        valid_shells.insert("zsh".to_string());
        valid_shells.insert("fish".to_string());
        valid_shells.insert("powershell".to_string());
        valid_shells.insert("cmd".to_string());

        let mut required_keybindings = HashSet::new();
        required_keybindings.insert("quit".to_string());
        required_keybindings.insert("input_mode".to_string());
        required_keybindings.insert("command_mode".to_string());
        required_keybindings.insert("escape".to_string());
        required_keybindings.insert("enter".to_string());

        // Create interdependent validations that work across configuration sections
        let interdependent_validations: Vec<fn(&Config) -> Result<(), ConfigError>> = vec![
            validate_openai_provider_config,
            validate_anthropic_provider_config,
            validate_default_model_consistency,
        ];

        Self {
            valid_log_levels,
            valid_themes,
            valid_color_schemes,
            valid_naming_conventions,
            valid_indentation_types,
            valid_ai_providers,
            valid_agent_priorities,
            valid_shells,
            required_keybindings,
            interdependent_validations,
        }
    }

    /// Comprehensive configuration validation
    pub fn validate(&self, config: &Config) -> Result<(), ConfigError> {
        // First validate each section individually
        self.validate_general(&config.general)?;
        self.validate_agents(&config.agents)?;
        self.validate_codegen(&config.codegen)?;
        self.validate_shell(&config.shell)?;
        self.validate_ui(&config.ui)?;
        self.validate_keybindings(&config.keybindings)?;
        self.validate_chat(&config.chat)?;

        // Then run interdependent validations that work across sections
        for validation in &self.interdependent_validations {
            validation(config)?;
        }

        Ok(())
    }

    /// Validate general configuration
    /// Validates general configuration section
    fn validate_general(&self, general: &crate::config::GeneralConfig) -> Result<(), ConfigError> {
        if general.log_level.is_empty() {
            return Err(ConfigError::ValidationError(
                "log_level cannot be empty".to_string(),
            ));
        }

        if !self.valid_log_levels.contains(&general.log_level) {
            return Err(ConfigError::ValidationError(format!(
                "Invalid log_level '{}'. Valid options are: {:?}",
                general.log_level, self.valid_log_levels
            )));
        }

        if let Some(workspace_path) = &general.workspace_path {
            self.validate_directory_path(workspace_path, "Workspace path")?;
        }

        Ok(())
    }

    /// Validate agent configuration
    /// Validates agent configuration section
    fn validate_agents(&self, agents: &crate::config::AgentConfig) -> Result<(), ConfigError> {
        if agents.max_concurrent_agents == 0 {
            return Err(ConfigError::ValidationError(
                "max_concurrent_agents must be greater than 0".to_string(),
            ));
        }

        if agents.max_concurrent_agents > 100 {
            return Err(ConfigError::ValidationError(
                "max_concurrent_agents cannot exceed 100".to_string(),
            ));
        }

        if agents.agent_timeout_seconds == 0 {
            return Err(ConfigError::ValidationError(
                "agent_timeout_seconds must be greater than 0".to_string(),
            ));
        }

        if agents.agent_timeout_seconds > 3600 {
            return Err(ConfigError::ValidationError(
                "agent_timeout_seconds cannot exceed 3600 (1 hour)".to_string(),
            ));
        }

        if !self
            .valid_agent_priorities
            .contains(&agents.default_agent_priority)
        {
            return Err(ConfigError::ValidationError(format!(
                "Invalid default_agent_priority '{}'. Valid options are: {:?}",
                agents.default_agent_priority, self.valid_agent_priorities
            )));
        }

        // Validate custom agents
        for custom_agent in &agents.custom_agents {
            if custom_agent.name.is_empty() {
                return Err(ConfigError::ValidationError(
                    "Custom agent name cannot be empty".to_string(),
                ));
            }

            if custom_agent.capabilities.is_empty() {
                return Err(ConfigError::ValidationError(format!(
                    "Custom agent '{}' must have at least one capability",
                    custom_agent.name
                )));
            }

            if !self.valid_agent_priorities.contains(&custom_agent.priority) {
                return Err(ConfigError::ValidationError(format!(
                    "Invalid priority '{}' for custom agent '{}'. Valid options are: {:?}",
                    custom_agent.priority, custom_agent.name, self.valid_agent_priorities
                )));
            }

            // Validate settings structure if needed
            if custom_agent.settings.is_empty() {
                // Just a warning could be logged here, but not failing validation
            }
        }

        // Validate notification settings
        if agents.notification_settings.auto_dismiss_timeout > 300000 {
            return Err(ConfigError::ValidationError(
                "auto_dismiss_timeout cannot exceed 300000ms (5 minutes)".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate code generation configuration
    /// Validates code generation configuration section
    fn validate_codegen(&self, codegen: &crate::config::CodegenConfig) -> Result<(), ConfigError> {
        self.validate_style_config(&codegen.default_style)?;

        // Validate language preferences
        for (language, lang_config) in &codegen.language_preferences {
            if language.is_empty() {
                return Err(ConfigError::ValidationError(
                    "Language name cannot be empty".to_string(),
                ));
            }

            self.validate_style_config(&lang_config.style)?;
        }

        // Validate template directories
        for template_dir in &codegen.template_directories {
            self.validate_directory_path(template_dir, "Template directory")?;
        }

        // Validate AI model settings
        self.validate_ai_model_config(&codegen.ai_model_settings)?;

        Ok(())
    }

    /// Validate style configuration
    fn validate_style_config(&self, style: &crate::config::StyleConfig) -> Result<(), ConfigError> {
        if !self.valid_indentation_types.contains(&style.indentation) {
            return Err(ConfigError::ValidationError(format!(
                "Invalid indentation type '{}'. Valid options are: {:?}",
                style.indentation, self.valid_indentation_types
            )));
        }

        if style.indent_size == 0 || style.indent_size > 16 {
            return Err(ConfigError::ValidationError(
                "indent_size must be between 1 and 16".to_string(),
            ));
        }

        if style.line_length < 40 || style.line_length > 500 {
            return Err(ConfigError::ValidationError(
                "line_length must be between 40 and 500".to_string(),
            ));
        }

        if !self
            .valid_naming_conventions
            .contains(&style.naming_convention)
        {
            return Err(ConfigError::ValidationError(format!(
                "Invalid naming_convention '{}'. Valid options are: {:?}",
                style.naming_convention, self.valid_naming_conventions
            )));
        }

        Ok(())
    }

    /// Validate AI model configuration
    /// Validates AI model configuration section
    fn validate_ai_model_config(
        &self,
        ai_config: &crate::config::AIModelConfig,
    ) -> Result<(), ConfigError> {
        if !self
            .valid_ai_providers
            .contains(&ai_config.default_provider)
        {
            return Err(ConfigError::ValidationError(format!(
                "Invalid AI provider '{}'. Valid options are: {:?}",
                ai_config.default_provider, self.valid_ai_providers
            )));
        }

        if ai_config.default_model.is_empty() {
            return Err(ConfigError::ValidationError(
                "default_model cannot be empty".to_string(),
            ));
        }

        if ai_config.context_window_size == 0 || ai_config.context_window_size > 1000000 {
            return Err(ConfigError::ValidationError(
                "context_window_size must be between 1 and 1000000".to_string(),
            ));
        }

        if ai_config.temperature < 0.0 || ai_config.temperature > 2.0 {
            return Err(ConfigError::ValidationError(
                "temperature must be between 0.0 and 2.0".to_string(),
            ));
        }

        if ai_config.max_tokens == 0 || ai_config.max_tokens > 100000 {
            return Err(ConfigError::ValidationError(
                "max_tokens must be between 1 and 100000".to_string(),
            ));
        }

        // Validate Ollama config
        if ai_config.ollama.endpoint.is_empty() {
            return Err(ConfigError::ValidationError(
                "Ollama endpoint cannot be empty".to_string(),
            ));
        }

        if !ai_config.ollama.endpoint.starts_with("http://")
            && !ai_config.ollama.endpoint.starts_with("https://")
        {
            return Err(ConfigError::ValidationError(
                "Ollama endpoint must start with http:// or https://".to_string(),
            ));
        }

        if ai_config.ollama.timeout_seconds == 0 || ai_config.ollama.timeout_seconds > 3600 {
            return Err(ConfigError::ValidationError(
                "Ollama timeout_seconds must be between 1 and 3600".to_string(),
            ));
        }

        if ai_config.ollama.max_retries > 10 {
            return Err(ConfigError::ValidationError(
                "Ollama max_retries cannot exceed 10".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate shell configuration
    /// Validates shell configuration section
    fn validate_shell(&self, shell: &crate::config::ShellConfig) -> Result<(), ConfigError> {
        if shell.command_timeout == 0 || shell.command_timeout > 3600 {
            return Err(ConfigError::ValidationError(
                "command_timeout must be between 1 and 3600 seconds".to_string(),
            ));
        }

        // Validate preferred shell if specified
        if let Some(shell_name) = &shell.preferred_shell {
            if !self.valid_shells.contains(shell_name) {
                return Err(ConfigError::ValidationError(format!(
                    "Invalid shell '{}'. Valid options are: {:?}",
                    shell_name, self.valid_shells
                )));
            }
        }

        // Validate custom commands
        for (command_name, command_template) in &shell.custom_commands {
            if command_name.is_empty() {
                return Err(ConfigError::ValidationError(
                    "Custom command name cannot be empty".to_string(),
                ));
            }

            if command_template.is_empty() {
                return Err(ConfigError::ValidationError(format!(
                    "Custom command '{}' template cannot be empty",
                    command_name
                )));
            }
        }

        Ok(())
    }

    /// Validate chat configuration
    fn validate_chat(&self, chat: &crate::config::ChatConfig) -> Result<(), ConfigError> {
        if chat.min_code_score < 0.0 || chat.min_code_score > 100.0 {
            return Err(ConfigError::ValidationError(
                "chat.min_code_score must be between 0.0 and 100.0".to_string(),
            ));
        }
        if chat.min_margin < 0.0 || chat.min_margin > 100.0 {
            return Err(ConfigError::ValidationError(
                "chat.min_margin must be between 0.0 and 100.0".to_string(),
            ));
        }
        if chat.entity_weight < 0.0 || chat.entity_weight > 10.0 {
            return Err(ConfigError::ValidationError(
                "chat.entity_weight must be between 0.0 and 10.0".to_string(),
            ));
        }
        if chat.language_hint_weight < 0.0 || chat.language_hint_weight > 10.0 {
            return Err(ConfigError::ValidationError(
                "chat.language_hint_weight must be between 0.0 and 10.0".to_string(),
            ));
        }
        Ok(())
    }

    /// Validate UI configuration
    /// Validates UI configuration section
    fn validate_ui(&self, ui: &crate::config::UIConfig) -> Result<(), ConfigError> {
        if !self.valid_themes.contains(&ui.theme) {
            return Err(ConfigError::ValidationError(format!(
                "Invalid theme '{}'. Valid options are: {:?}",
                ui.theme, self.valid_themes
            )));
        }

        if !self.valid_color_schemes.contains(&ui.color_scheme) {
            return Err(ConfigError::ValidationError(format!(
                "Invalid color_scheme '{}'. Valid options are: {:?}",
                ui.color_scheme, self.valid_color_schemes
            )));
        }

        if ui.font_size < 8 || ui.font_size > 48 {
            return Err(ConfigError::ValidationError(
                "font_size must be between 8 and 48".to_string(),
            ));
        }

        // Validate panel layout
        let layout = &ui.panel_layout;
        if layout.output_panel_percentage + layout.agent_panel_percentage != 100 {
            return Err(ConfigError::ValidationError(
                "output_panel_percentage + agent_panel_percentage must equal 100".to_string(),
            ));
        }

        if layout.output_panel_percentage < 20 || layout.output_panel_percentage > 80 {
            return Err(ConfigError::ValidationError(
                "output_panel_percentage must be between 20 and 80".to_string(),
            ));
        }

        if layout.notification_panel_height > 20 {
            return Err(ConfigError::ValidationError(
                "notification_panel_height cannot exceed 20".to_string(),
            ));
        }

        if layout.input_panel_height == 0 || layout.input_panel_height > 10 {
            return Err(ConfigError::ValidationError(
                "input_panel_height must be between 1 and 10".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate keybindings
    /// Validates keybindings configuration section
    fn validate_keybindings(
        &self,
        keybindings: &std::collections::HashMap<String, String>,
    ) -> Result<(), ConfigError> {
        for required in &self.required_keybindings {
            if !keybindings.contains_key(required) {
                return Err(ConfigError::ValidationError(format!(
                    "Required keybinding '{}' is missing",
                    required
                )));
            }
        }

        // Check for empty keybindings
        for (action, key) in keybindings {
            if key.is_empty() {
                return Err(ConfigError::ValidationError(format!(
                    "Keybinding for action '{}' cannot be empty",
                    action
                )));
            }
        }

        // Check for duplicate key assignments
        let mut seen_keys = HashMap::new();
        for (action, key) in keybindings {
            if let Some(existing_action) = seen_keys.get(key) {
                return Err(ConfigError::ValidationError(format!(
                    "Duplicate key '{}' assigned to both '{}' and '{}'",
                    key, existing_action, action
                )));
            }
            seen_keys.insert(key, action);
        }

        Ok(())
    }

    // Helper methods for commonly used validation patterns

    /// Validates a directory path exists and is actually a directory
    fn validate_directory_path(&self, path: &Path, description: &str) -> Result<(), ConfigError> {
        if !path.exists() {
            return Err(ConfigError::ValidationError(format!(
                "{} does not exist: {:?}",
                description, path
            )));
        }

        if !path.is_dir() {
            return Err(ConfigError::ValidationError(format!(
                "{} is not a directory: {:?}",
                description, path
            )));
        }

        Ok(())
    }

    /// Validates a numeric value is within an acceptable range
    fn validate_range<T>(
        &self,
        value: T,
        min: T,
        max: T,
        field_name: &str,
    ) -> Result<(), ConfigError>
    where
        T: std::cmp::PartialOrd + std::fmt::Display,
    {
        if value < min || value > max {
            return Err(ConfigError::ValidationError(format!(
                "{}={} must be between {} and {}",
                field_name, value, min, max
            )));
        }
        Ok(())
    }

    /// Validates a string field is not empty
    fn validate_non_empty(&self, value: &str, field_name: &str) -> Result<(), ConfigError> {
        if value.is_empty() {
            return Err(ConfigError::ValidationError(format!(
                "{} cannot be empty",
                field_name
            )));
        }
        Ok(())
    }

    /// Validates that a string value is in a set of allowed values
    fn validate_allowed_value(
        &self,
        value: &str,
        allowed: &HashSet<String>,
        field_name: &str,
    ) -> Result<(), ConfigError> {
        if !allowed.contains(value) {
            return Err(ConfigError::ValidationError(format!(
                "Invalid {} '{}'. Valid options are: {:?}",
                field_name, value, allowed
            )));
        }
        Ok(())
    }
}

// Manually implement Send and Sync for ConfigValidator
unsafe impl Send for ConfigValidator {}
unsafe impl Sync for ConfigValidator {}

// Manually implement Debug since we can't derive it due to the function pointers
impl std::fmt::Debug for ConfigValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigValidator")
            .field("valid_log_levels", &self.valid_log_levels)
            .field("valid_themes", &self.valid_themes)
            .field("valid_color_schemes", &self.valid_color_schemes)
            .field("valid_naming_conventions", &self.valid_naming_conventions)
            .field("valid_indentation_types", &self.valid_indentation_types)
            .field("valid_ai_providers", &self.valid_ai_providers)
            .field("valid_agent_priorities", &self.valid_agent_priorities)
            .field("valid_shells", &self.valid_shells)
            .field("required_keybindings", &self.required_keybindings)
            .field(
                "interdependent_validations",
                &format!(
                    "[{} validation functions]",
                    self.interdependent_validations.len()
                ),
            )
            .finish()
    }
}
