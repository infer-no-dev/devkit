//! Configuration structures and enums for the logging system.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Log level enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Trace => write!(f, "TRACE"),
        }
    }
}

impl std::str::FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "error" => Ok(LogLevel::Error),
            "warn" | "warning" => Ok(LogLevel::Warn),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            "trace" => Ok(LogLevel::Trace),
            _ => Err(format!("Invalid log level: {}", s)),
        }
    }
}

/// Log format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogFormat {
    /// JSON format for structured logging
    Json,
    /// Human-readable text format
    Text,
    /// Key-value structured format
    Structured,
}

impl Default for LogFormat {
    fn default() -> Self {
        LogFormat::Text
    }
}

impl std::fmt::Display for LogFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogFormat::Json => write!(f, "json"),
            LogFormat::Text => write!(f, "text"),
            LogFormat::Structured => write!(f, "structured"),
        }
    }
}

/// Log rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotation {
    /// Maximum file size before rotation (in bytes)
    pub max_size_bytes: u64,
    /// Maximum number of backup files to keep
    pub max_files: usize,
    /// Whether to compress rotated files
    pub compress: bool,
}

impl Default for LogRotation {
    fn default() -> Self {
        Self {
            max_size_bytes: 100 * 1024 * 1024, // 100 MB
            max_files: 5,
            compress: true,
        }
    }
}

/// Syslog facility enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyslogFacility {
    User,
    Mail,
    Daemon,
    Auth,
    Syslog,
    Lpr,
    News,
    Uucp,
    Cron,
    Authpriv,
    Ftp,
    Local0,
    Local1,
    Local2,
    Local3,
    Local4,
    Local5,
    Local6,
    Local7,
}

impl Default for SyslogFacility {
    fn default() -> Self {
        SyslogFacility::User
    }
}

/// Log output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LogOutput {
    Console {
        format: LogFormat,
        colored: bool,
    },
    File {
        path: PathBuf,
        format: LogFormat,
        rotation: Option<LogRotation>,
    },
    Syslog {
        facility: SyslogFacility,
        format: LogFormat,
    },
}

impl Default for LogOutput {
    fn default() -> Self {
        LogOutput::Console {
            format: LogFormat::Text,
            colored: true,
        }
    }
}

/// Main logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// Minimum log level to process
    pub min_level: LogLevel,

    /// Output destinations
    pub outputs: Vec<LogOutput>,

    /// Components to allow logging from (empty = allow all)
    pub allowed_components: Vec<String>,

    /// Components to block logging from
    pub blocked_components: Vec<String>,

    /// Maximum log entries per minute (rate limiting)
    pub rate_limit_per_minute: Option<usize>,

    /// Sample rate for high-volume logs (0.0 to 1.0)
    pub sample_rate: Option<f64>,

    /// Environment name for context
    pub environment: String,

    /// Whether to include source location information
    pub include_location: bool,

    /// Whether to include thread information
    pub include_thread_info: bool,

    /// Whether to capture performance metrics
    pub capture_metrics: bool,

    /// Buffer size for async logging
    pub buffer_size: usize,

    /// Timeout for flushing logs on shutdown (in milliseconds)
    pub flush_timeout_ms: u64,

    /// Custom log fields to include in all entries
    pub global_fields: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            min_level: LogLevel::Info,
            outputs: vec![LogOutput::default()],
            allowed_components: Vec::new(),
            blocked_components: Vec::new(),
            rate_limit_per_minute: None,
            sample_rate: None,
            environment: "development".to_string(),
            include_location: false,
            include_thread_info: true,
            capture_metrics: false,
            buffer_size: 1000,
            flush_timeout_ms: 5000,
            global_fields: std::collections::HashMap::new(),
        }
    }
}

impl LogConfig {
    /// Create a development-friendly configuration
    pub fn development() -> Self {
        Self {
            min_level: LogLevel::Debug,
            outputs: vec![LogOutput::Console {
                format: LogFormat::Text,
                colored: true,
            }],
            environment: "development".to_string(),
            include_location: true,
            include_thread_info: true,
            capture_metrics: true,
            ..Default::default()
        }
    }

    /// Create a production-ready configuration
    pub fn production(log_dir: PathBuf) -> Self {
        Self {
            min_level: LogLevel::Info,
            outputs: vec![
                LogOutput::File {
                    path: log_dir.join("application.log"),
                    format: LogFormat::Json,
                    rotation: Some(LogRotation::default()),
                },
                LogOutput::Console {
                    format: LogFormat::Text,
                    colored: false,
                },
            ],
            environment: "production".to_string(),
            include_location: false,
            include_thread_info: true,
            capture_metrics: true,
            rate_limit_per_minute: Some(10000),
            sample_rate: Some(1.0),
            ..Default::default()
        }
    }

    /// Create a testing configuration
    pub fn testing() -> Self {
        Self {
            min_level: LogLevel::Warn,
            outputs: vec![LogOutput::Console {
                format: LogFormat::Text,
                colored: false,
            }],
            environment: "testing".to_string(),
            include_location: false,
            include_thread_info: false,
            capture_metrics: false,
            ..Default::default()
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.outputs.is_empty() {
            return Err("At least one output must be configured".to_string());
        }

        if let Some(rate_limit) = self.rate_limit_per_minute {
            if rate_limit == 0 {
                return Err("Rate limit must be greater than 0".to_string());
            }
        }

        if let Some(sample_rate) = self.sample_rate {
            if sample_rate < 0.0 || sample_rate > 1.0 {
                return Err("Sample rate must be between 0.0 and 1.0".to_string());
            }
        }

        if self.buffer_size == 0 {
            return Err("Buffer size must be greater than 0".to_string());
        }

        if self.flush_timeout_ms == 0 {
            return Err("Flush timeout must be greater than 0".to_string());
        }

        // Validate file paths
        for output in &self.outputs {
            match output {
                LogOutput::File { path, .. } => {
                    if let Some(parent) = path.parent() {
                        if !parent.exists() {
                            return Err(format!(
                                "Log directory does not exist: {}",
                                parent.display()
                            ));
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Load configuration from a TOML file
    pub fn from_toml_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: LogConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Load configuration from a JSON file
    pub fn from_json_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: LogConfig = serde_json::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Save configuration to a TOML file
    pub fn to_toml_file(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Save configuration to a JSON file
    pub fn to_json_file(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Merge another configuration into this one
    pub fn merge(&mut self, other: LogConfig) {
        // Take the less restrictive log level (higher verbosity)
        if other.min_level > self.min_level {
            self.min_level = other.min_level;
        }

        // Merge outputs (avoiding duplicates)
        for output in other.outputs {
            if !self
                .outputs
                .iter()
                .any(|existing| std::mem::discriminant(existing) == std::mem::discriminant(&output))
            {
                self.outputs.push(output);
            }
        }

        // Merge component filters
        for component in other.allowed_components {
            if !self.allowed_components.contains(&component) {
                self.allowed_components.push(component);
            }
        }

        for component in other.blocked_components {
            if !self.blocked_components.contains(&component) {
                self.blocked_components.push(component);
            }
        }

        // Take more restrictive rate limiting
        match (self.rate_limit_per_minute, other.rate_limit_per_minute) {
            (Some(current), Some(other_limit)) => {
                self.rate_limit_per_minute = Some(current.min(other_limit));
            }
            (None, Some(limit)) => {
                self.rate_limit_per_minute = Some(limit);
            }
            _ => {}
        }

        // Take more restrictive sampling
        match (self.sample_rate, other.sample_rate) {
            (Some(current), Some(other_rate)) => {
                self.sample_rate = Some(current.min(other_rate));
            }
            (None, Some(rate)) => {
                self.sample_rate = Some(rate);
            }
            _ => {}
        }

        // Merge global fields
        for (key, value) in other.global_fields {
            self.global_fields.insert(key, value);
        }

        // Take other boolean flags if they're more restrictive or more informative
        if other.include_location {
            self.include_location = true;
        }
        if other.include_thread_info {
            self.include_thread_info = true;
        }
        if other.capture_metrics {
            self.capture_metrics = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Error < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Trace);
    }

    #[test]
    fn test_log_level_from_str() {
        assert_eq!("error".parse::<LogLevel>().unwrap(), LogLevel::Error);
        assert_eq!("warn".parse::<LogLevel>().unwrap(), LogLevel::Warn);
        assert_eq!("info".parse::<LogLevel>().unwrap(), LogLevel::Info);
        assert_eq!("debug".parse::<LogLevel>().unwrap(), LogLevel::Debug);
        assert_eq!("trace".parse::<LogLevel>().unwrap(), LogLevel::Trace);

        assert!("invalid".parse::<LogLevel>().is_err());
    }

    #[test]
    fn test_config_validation() {
        let mut config = LogConfig::default();
        assert!(config.validate().is_ok());

        config.outputs.clear();
        assert!(config.validate().is_err());

        config.outputs = vec![LogOutput::default()];
        config.rate_limit_per_minute = Some(0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_merge() {
        let mut config1 = LogConfig::default();
        config1.min_level = LogLevel::Info;
        config1.allowed_components = vec!["component1".to_string()];

        let mut config2 = LogConfig::default();
        config2.min_level = LogLevel::Debug;
        config2.allowed_components = vec!["component2".to_string()];
        config2.capture_metrics = true;

        config1.merge(config2);

        assert_eq!(config1.min_level, LogLevel::Debug);
        assert_eq!(config1.allowed_components.len(), 2);
        assert!(config1.capture_metrics);
    }
}
