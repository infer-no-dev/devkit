//! Default configuration generation with system auto-detection.
//!
//! This module provides intelligent default configuration generation based on
//! the system environment, capabilities, and best practices.

use crate::config::*;
use crate::logging::config::LogRotation;
use crate::logging::{LogConfig, LogFormat, LogLevel, LogOutput};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// System capability detector for generating appropriate defaults
#[derive(Debug, Clone)]
pub struct SystemDefaults {
    pub detected_shell: Option<String>,
    pub detected_terminal: Option<String>,
    pub system_cores: usize,
    pub available_memory_gb: u64,
    pub detected_editors: Vec<String>,
    pub has_git: bool,
    pub has_docker: bool,
    pub workspace_suggestions: Vec<PathBuf>,
}

impl SystemDefaults {
    /// Detect system capabilities and create appropriate defaults
    pub fn detect() -> Self {
        Self {
            detected_shell: Self::detect_shell(),
            detected_terminal: Self::detect_terminal(),
            system_cores: Self::detect_cpu_cores(),
            available_memory_gb: Self::detect_available_memory(),
            detected_editors: Self::detect_editors(),
            has_git: Self::command_exists("git"),
            has_docker: Self::command_exists("docker"),
            workspace_suggestions: Self::suggest_workspace_paths(),
        }
    }

    /// Generate a complete default configuration based on detected system capabilities
    pub fn generate_config(&self) -> Config {
        Config {
            general: self.generate_general_config(),
            agents: self.generate_agent_config(),
            codegen: self.generate_codegen_config(),
            shell: self.generate_shell_config(),
            ui: self.generate_ui_config(),
            logging: self.generate_logging_config(),
            keybindings: self.generate_keybindings(),
        }
    }

    /// Generate environment-specific configuration variations
    pub fn generate_environment_config(&self, environment: &str) -> Config {
        let mut config = self.generate_config();

        match environment {
            "development" => {
                config.general.log_level = "debug".to_string();
                config.general.telemetry_enabled = false;
                config.agents.max_concurrent_agents = std::cmp::min(self.system_cores * 2, 8);
                config.codegen.ai_model_settings.temperature = 0.3; // More deterministic for dev
            }
            "staging" => {
                config.general.log_level = "info".to_string();
                config.general.telemetry_enabled = true;
                config.agents.max_concurrent_agents = std::cmp::min(self.system_cores, 5);
                config.codegen.ai_model_settings.temperature = 0.5;
            }
            "production" => {
                config.general.log_level = "warn".to_string();
                config.general.telemetry_enabled = true;
                config.general.auto_save = true;
                config.general.backup_enabled = true;
                config.agents.max_concurrent_agents = std::cmp::min(self.system_cores, 3);
                config.codegen.ai_model_settings.temperature = 0.7;
            }
            _ => {} // Use default config
        }

        // Also update logging config based on environment
        match environment {
            "development" => {
                config.logging = LogConfig::development();
            }
            "production" => {
                let log_dir = dirs::data_local_dir()
                    .unwrap_or_else(|| PathBuf::from("./logs"))
                    .join("devkit");
                config.logging = LogConfig::production(log_dir);
            }
            "testing" => {
                config.logging = LogConfig::testing();
            }
            _ => {} // Use default config
        }

        config
    }

    fn generate_logging_config(&self) -> LogConfig {
        // Types already imported at module level

        // Create log directory in user's data directory
        let log_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("./logs"))
            .join("devkit");

        // Choose logging configuration based on system capabilities
        let (log_level, buffer_size, outputs) = if self.available_memory_gb >= 8 {
            // High-resource system: comprehensive logging
            (
                LogLevel::Info,
                2000,
                vec![
                    LogOutput::Console {
                        format: LogFormat::Text,
                        colored: true,
                    },
                    LogOutput::File {
                        path: log_dir.join("devkit.log"),
                        format: LogFormat::Json,
                        rotation: Some(LogRotation {
                            max_size_bytes: 100 * 1024 * 1024, // 100MB
                            max_files: 5,
                            compress: true,
                        }),
                    },
                ],
            )
        } else {
            // Resource-constrained system: minimal logging
            (
                LogLevel::Warn,
                500,
                vec![
                    LogOutput::Console {
                        format: LogFormat::Text,
                        colored: true,
                    },
                    LogOutput::File {
                        path: log_dir.join("devkit.log"),
                        format: LogFormat::Text,
                        rotation: Some(LogRotation {
                            max_size_bytes: 10 * 1024 * 1024, // 10MB
                            max_files: 3,
                            compress: false,
                        }),
                    },
                ],
            )
        };

        LogConfig {
            min_level: log_level,
            outputs,
            allowed_components: Vec::new(),
            blocked_components: Vec::new(),
            rate_limit_per_minute: Some(1000),
            sample_rate: Some(1.0),
            environment: "development".to_string(),
            include_location: false,
            include_thread_info: true,
            capture_metrics: true,
            buffer_size,
            flush_timeout_ms: 5000,
            global_fields: {
                let mut fields = std::collections::HashMap::new();
                fields.insert(
                    "service".to_string(),
                    serde_json::Value::String("devkit".to_string()),
                );
                fields.insert(
                    "version".to_string(),
                    serde_json::Value::String(env!("CARGO_PKG_VERSION").to_string()),
                );
                if let Some(hostname) = gethostname::gethostname().to_str() {
                    fields.insert(
                        "hostname".to_string(),
                        serde_json::Value::String(hostname.to_string()),
                    );
                }
                fields
            },
        }
    }

    fn generate_general_config(&self) -> GeneralConfig {
        GeneralConfig {
            workspace_path: self.suggest_default_workspace(),
            log_level: "info".to_string(),
            auto_save: true,
            backup_enabled: self.available_memory_gb > 4, // Enable backups if we have enough memory
            telemetry_enabled: false,                     // Default to privacy-first
        }
    }

    fn generate_agent_config(&self) -> AgentConfig {
        // Scale concurrent agents based on system capabilities
        let max_concurrent = if self.available_memory_gb >= 16 {
            std::cmp::min(self.system_cores * 2, 10)
        } else if self.available_memory_gb >= 8 {
            std::cmp::min(self.system_cores, 5)
        } else {
            std::cmp::min(self.system_cores / 2, 3).max(1)
        };

        AgentConfig {
            max_concurrent_agents: max_concurrent,
            agent_timeout_seconds: if self.available_memory_gb >= 8 {
                300
            } else {
                180
            },
            default_agent_priority: "normal".to_string(),
            notification_settings: NotificationConfig {
                enabled: true,
                sound_enabled: false, // Default to quiet operation
                desktop_notifications: Self::supports_desktop_notifications(),
                auto_dismiss_timeout: 5000,
            },
            custom_agents: Vec::new(),
        }
    }

    fn generate_codegen_config(&self) -> CodegenConfig {
        let mut language_preferences = HashMap::new();

        // Set up common language preferences based on detected tools
        if Self::command_exists("rustc") {
            language_preferences.insert(
                "rust".to_string(),
                LanguageConfig {
                    style: StyleConfig {
                        indentation: "spaces".to_string(),
                        indent_size: 4,
                        line_length: 100,
                        naming_convention: "snake_case".to_string(),
                        include_comments: true,
                        include_type_hints: true,
                    },
                    formatter: Some("rustfmt".to_string()),
                    linter: Some("clippy".to_string()),
                    specific_settings: HashMap::new(),
                },
            );
        }

        if Self::command_exists("python") || Self::command_exists("python3") {
            language_preferences.insert(
                "python".to_string(),
                LanguageConfig {
                    style: StyleConfig {
                        indentation: "spaces".to_string(),
                        indent_size: 4,
                        line_length: 88,
                        naming_convention: "snake_case".to_string(),
                        include_comments: true,
                        include_type_hints: true,
                    },
                    formatter: Some("black".to_string()),
                    linter: Some("ruff".to_string()),
                    specific_settings: HashMap::new(),
                },
            );
        }

        if Self::command_exists("node") {
            language_preferences.insert(
                "javascript".to_string(),
                LanguageConfig {
                    style: StyleConfig {
                        indentation: "spaces".to_string(),
                        indent_size: 2,
                        line_length: 100,
                        naming_convention: "camelCase".to_string(),
                        include_comments: true,
                        include_type_hints: false,
                    },
                    formatter: Some("prettier".to_string()),
                    linter: Some("eslint".to_string()),
                    specific_settings: HashMap::new(),
                },
            );
        }

        CodegenConfig {
            default_style: StyleConfig::default(),
            language_preferences,
            template_directories: self.suggest_template_directories(),
            ai_model_settings: self.generate_ai_model_config(),
        }
    }

    fn generate_ai_model_config(&self) -> AIModelConfig {
        // Adjust context window and token limits based on available memory
        let (context_window, max_tokens) = if self.available_memory_gb >= 16 {
            (32768, 4000)
        } else if self.available_memory_gb >= 8 {
            (16384, 2000)
        } else {
            (8192, 1000)
        };

        AIModelConfig {
            default_provider: "ollama".to_string(),
            default_model: "llama3.2:latest".to_string(),
            ollama: OllamaConfig {
                endpoint: "http://localhost:11434".to_string(),
                timeout_seconds: if self.available_memory_gb >= 8 {
                    300
                } else {
                    180
                },
                max_retries: 3,
                default_model: Some("llama3.2:latest".to_string()),
            },
            openai: None, // User needs to configure API keys manually
            anthropic: None,
            context_window_size: context_window,
            temperature: 0.7,
            max_tokens,
        }
    }

    fn generate_shell_config(&self) -> ShellConfig {
        let mut environment_variables = HashMap::new();

        // Set up common development environment variables
        if let Ok(path) = std::env::var("PATH") {
            environment_variables.insert("PATH".to_string(), path);
        }

        // Add common development tool paths if they exist
        let dev_paths = [
            "/usr/local/bin",
            "/usr/bin",
            "/bin",
            "/usr/local/go/bin",
            "/opt/homebrew/bin", // macOS Homebrew
        ];

        for path in &dev_paths {
            if Path::new(path).exists() {
                environment_variables.insert(
                    format!("DEV_PATH_{}", path.replace(['/', '-'], "_").to_uppercase()),
                    path.to_string(),
                );
            }
        }

        ShellConfig {
            preferred_shell: self.detected_shell.clone(),
            environment_variables,
            command_timeout: 30,
            history_enabled: true,
            custom_commands: self.generate_custom_commands(),
        }
    }

    fn generate_ui_config(&self) -> UIConfig {
        // Adjust UI settings based on detected terminal capabilities
        let font_size = if self.detected_terminal.as_deref() == Some("alacritty")
            || self.detected_terminal.as_deref() == Some("kitty")
        {
            16 // These terminals typically handle larger fonts well
        } else {
            14
        };

        UIConfig {
            theme: "default".to_string(),
            color_scheme: "dark".to_string(), // Most developers prefer dark mode
            font_size,
            show_line_numbers: true,
            show_timestamps: true,
            auto_scroll: true,
            panel_layout: PanelLayoutConfig {
                output_panel_percentage: 70,
                agent_panel_percentage: 30,
                notification_panel_height: 5,
                input_panel_height: 3,
            },
        }
    }

    fn generate_keybindings(&self) -> HashMap<String, String> {
        let mut bindings = HashMap::new();

        // Use vim-style bindings if vim is detected, otherwise use standard bindings
        if self.detected_editors.contains(&"vim".to_string())
            || self.detected_editors.contains(&"nvim".to_string())
        {
            // Vim-style navigation
            bindings.insert("quit".to_string(), "q".to_string());
            bindings.insert("input_mode".to_string(), "i".to_string());
            bindings.insert("command_mode".to_string(), ":".to_string());
            bindings.insert("scroll_up".to_string(), "k".to_string());
            bindings.insert("scroll_down".to_string(), "j".to_string());
            bindings.insert("page_up".to_string(), "u".to_string());
            bindings.insert("page_down".to_string(), "d".to_string());
        } else {
            // Standard bindings
            bindings.insert("quit".to_string(), "q".to_string());
            bindings.insert("input_mode".to_string(), "i".to_string());
            bindings.insert("command_mode".to_string(), ":".to_string());
            bindings.insert("scroll_up".to_string(), "Up".to_string());
            bindings.insert("scroll_down".to_string(), "Down".to_string());
            bindings.insert("page_up".to_string(), "PageUp".to_string());
            bindings.insert("page_down".to_string(), "PageDown".to_string());
        }

        // Common bindings regardless of editor preference
        bindings.insert("agent_view".to_string(), "a".to_string());
        bindings.insert("settings".to_string(), "s".to_string());
        bindings.insert("escape".to_string(), "Esc".to_string());
        bindings.insert("enter".to_string(), "Enter".to_string());
        bindings.insert("backspace".to_string(), "Backspace".to_string());

        bindings
    }

    // System detection helper methods

    fn detect_shell() -> Option<String> {
        // Try to detect the current shell
        if let Ok(shell) = std::env::var("SHELL") {
            if let Some(name) = shell.split('/').last() {
                return Some(name.to_string());
            }
        }

        // Fallback to common shells
        let common_shells = ["bash", "zsh", "fish", "powershell"];
        for shell in &common_shells {
            if Self::command_exists(shell) {
                return Some(shell.to_string());
            }
        }

        None
    }

    fn detect_terminal() -> Option<String> {
        // Check common terminal environment variables
        if let Ok(term) = std::env::var("TERM_PROGRAM") {
            return Some(term.to_lowercase());
        }

        if let Ok(term) = std::env::var("TERMINAL_EMULATOR") {
            return Some(term.to_lowercase());
        }

        // Check for common terminals
        let terminals = ["alacritty", "kitty", "gnome-terminal", "konsole", "xterm"];
        for terminal in &terminals {
            if Self::command_exists(terminal) {
                return Some(terminal.to_string());
            }
        }

        None
    }

    fn detect_cpu_cores() -> usize {
        std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4) // Default to 4 cores if detection fails
    }

    fn detect_available_memory() -> u64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
                for line in meminfo.lines() {
                    if line.starts_with("MemAvailable:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<u64>() {
                                return kb / 1024 / 1024; // Convert to GB
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = std::process::Command::new("sysctl")
                .args(&["hw.memsize"])
                .output()
            {
                if let Ok(output_str) = std::str::from_utf8(&output.stdout) {
                    if let Some(bytes_str) = output_str.split_whitespace().nth(1) {
                        if let Ok(bytes) = bytes_str.parse::<u64>() {
                            return bytes / 1024 / 1024 / 1024; // Convert to GB
                        }
                    }
                }
            }
        }

        // Default to 8GB if detection fails
        8
    }

    fn detect_editors() -> Vec<String> {
        let editors = ["vim", "nvim", "nano", "emacs", "code", "subl", "atom"];
        editors
            .iter()
            .filter(|&editor| Self::command_exists(editor))
            .map(|&editor| editor.to_string())
            .collect()
    }

    fn suggest_workspace_paths() -> Vec<PathBuf> {
        let mut suggestions = Vec::new();

        // Common workspace locations
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

        let candidates = [
            home.join("Projects"),
            home.join("workspace"),
            home.join("code"),
            home.join("dev"),
            home.join("Development"),
            home.join("src"),
            PathBuf::from("/workspace"),
            PathBuf::from("/code"),
        ];

        for candidate in &candidates {
            if candidate.exists() && candidate.is_dir() {
                suggestions.push(candidate.clone());
            }
        }

        // If no common workspace found, suggest creating one
        if suggestions.is_empty() {
            suggestions.push(home.join("Projects"));
        }

        suggestions
    }

    fn suggest_default_workspace(&self) -> Option<PathBuf> {
        self.workspace_suggestions.first().cloned()
    }

    fn suggest_template_directories(&self) -> Vec<PathBuf> {
        let mut templates = Vec::new();

        // Look for common template locations
        if let Some(home) = dirs::home_dir() {
            let candidates = [
                home.join(".config")
                    .join("devkit-env")
                    .join("templates"),
                home.join(".agentic").join("templates"),
                home.join("Templates"),
            ];

            for candidate in &candidates {
                if candidate.exists() {
                    templates.push(candidate.clone());
                }
            }
        }

        templates
    }

    fn generate_custom_commands(&self) -> HashMap<String, String> {
        let mut commands = HashMap::new();

        // Add git shortcuts if git is available
        if self.has_git {
            commands.insert("gst".to_string(), "git status".to_string());
            commands.insert("gco".to_string(), "git checkout".to_string());
            commands.insert("gp".to_string(), "git push".to_string());
            commands.insert("gl".to_string(), "git log --oneline -10".to_string());
        }

        // Add docker shortcuts if docker is available
        if self.has_docker {
            commands.insert("dps".to_string(), "docker ps".to_string());
            commands.insert("di".to_string(), "docker images".to_string());
            commands.insert("dc".to_string(), "docker-compose".to_string());
        }

        // Add common development shortcuts
        commands.insert("ll".to_string(), "ls -la".to_string());
        commands.insert("la".to_string(), "ls -la".to_string());

        commands
    }

    fn command_exists(cmd: &str) -> bool {
        std::process::Command::new("which")
            .arg(cmd)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn supports_desktop_notifications() -> bool {
        // Check if we're in a desktop environment that supports notifications
        cfg!(target_os = "linux") && std::env::var("DISPLAY").is_ok()
            || cfg!(target_os = "macos")
            || cfg!(target_os = "windows")
    }
}

/// Configuration profile presets for different use cases
pub struct ConfigProfiles;

impl ConfigProfiles {
    /// Minimal configuration for resource-constrained environments
    pub fn minimal() -> Config {
        let system = SystemDefaults::detect();
        let mut config = system.generate_config();

        // Reduce resource usage
        config.agents.max_concurrent_agents = 1;
        config.codegen.ai_model_settings.context_window_size = 4096;
        config.codegen.ai_model_settings.max_tokens = 500;
        config.general.backup_enabled = false;
        config.agents.notification_settings.desktop_notifications = false;

        config
    }

    /// Performance-optimized configuration for powerful systems
    pub fn performance() -> Config {
        let system = SystemDefaults::detect();
        let mut config = system.generate_config();

        // Maximize performance
        config.agents.max_concurrent_agents = std::cmp::min(system.system_cores * 2, 12);
        config.codegen.ai_model_settings.context_window_size = 65536;
        config.codegen.ai_model_settings.max_tokens = 8000;
        config.general.backup_enabled = true;
        config.shell.command_timeout = 60;

        config
    }

    /// Privacy-focused configuration
    pub fn privacy() -> Config {
        let system = SystemDefaults::detect();
        let mut config = system.generate_config();

        // Minimize data collection and external connections
        config.general.telemetry_enabled = false;
        config.agents.notification_settings.desktop_notifications = false;
        config.codegen.ai_model_settings.default_provider = "ollama".to_string(); // Local AI only
        config.codegen.ai_model_settings.openai = None;
        config.codegen.ai_model_settings.anthropic = None;

        config
    }

    /// Team collaboration optimized configuration
    pub fn collaborative() -> Config {
        let system = SystemDefaults::detect();
        let mut config = system.generate_config();

        // Optimize for team work
        config.general.telemetry_enabled = true; // For team analytics
        config.agents.notification_settings.enabled = true;
        config.agents.notification_settings.desktop_notifications = true;
        config.general.backup_enabled = true;
        config.general.auto_save = true;

        config
    }
}
