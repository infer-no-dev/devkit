//! Comprehensive structured logging system for the agentic development environment.
//!
//! This module provides structured logging with multiple output formats, configurable
//! destinations, contextual information, and performance-aware logging capabilities.

pub mod config;
pub mod context;
pub mod formatter;
pub mod metrics;
pub mod output;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;

pub use config::{LogConfig, LogFormat, LogLevel, LogOutput};
pub use context::LogContext;
pub use formatter::{JsonFormatter, LogFormatter, StructuredFormatter, TextFormatter};
pub use metrics::LogMetrics;
pub use output::{ConsoleOutput, FileOutput, LogOutput as OutputDestination, SyslogOutput};

/// Main logging system that coordinates all logging activities
#[derive(Debug)]
pub struct LoggingSystem {
    config: LogConfig,
    formatters: HashMap<LogFormat, Arc<dyn LogFormatter + Send + Sync>>,
    outputs: Vec<Arc<dyn OutputDestination + Send + Sync>>,
    context: Arc<RwLock<LogContext>>,
    metrics: Arc<RwLock<LogMetrics>>,
    filters: Vec<Box<dyn LogFilter + Send + Sync>>,
}

/// Structured log entry with rich metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp when the log entry was created
    pub timestamp: DateTime<Utc>,
    /// Log level (error, warn, info, debug, trace)
    pub level: LogLevel,
    /// The main log message
    pub message: String,
    /// Component that generated the log
    pub component: String,
    /// Optional module path
    pub module: Option<String>,
    /// Source file and line number
    pub location: Option<LogLocation>,
    /// Thread ID where the log originated
    pub thread_id: Option<String>,
    /// Request/task ID for correlation
    pub correlation_id: Option<String>,
    /// User ID if available
    pub user_id: Option<String>,
    /// Session ID for user sessions
    pub session_id: Option<String>,
    /// Structured data fields
    pub fields: HashMap<String, serde_json::Value>,
    /// Performance metrics
    pub metrics: Option<LogEntryMetrics>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Environment information
    pub environment: Option<String>,
    /// Error information if this is an error log
    pub error: Option<LogError>,
}

/// Source location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLocation {
    pub file: String,
    pub line: u32,
    pub column: Option<u32>,
}

/// Performance metrics for a log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryMetrics {
    pub duration_ms: Option<f64>,
    pub memory_usage_kb: Option<u64>,
    pub cpu_usage_percent: Option<f64>,
    pub custom_metrics: HashMap<String, f64>,
}

/// Error information in log entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogError {
    pub error_type: String,
    pub error_message: String,
    pub error_code: Option<String>,
    pub stack_trace: Option<String>,
    pub caused_by: Option<Box<LogError>>,
}

/// Log filtering interface
pub trait LogFilter: fmt::Debug {
    fn should_log(&self, entry: &LogEntry) -> bool;
    fn name(&self) -> &str;
}

/// Component-based log filter
#[derive(Debug)]
pub struct ComponentFilter {
    allowed_components: Vec<String>,
    blocked_components: Vec<String>,
}

/// Level-based log filter
#[derive(Debug)]
pub struct LevelFilter {
    min_level: LogLevel,
}

/// Rate limiting filter
#[derive(Debug)]
pub struct RateLimitFilter {
    max_entries_per_minute: usize,
    current_count: Arc<RwLock<usize>>,
    last_reset: Arc<RwLock<DateTime<Utc>>>,
}

/// Sampling filter for high-volume logs
#[derive(Debug)]
pub struct SamplingFilter {
    sample_rate: f64, // 0.0 to 1.0
}

impl LoggingSystem {
    /// Create a new logging system with the given configuration
    pub fn new(config: LogConfig) -> Result<Self, LoggingError> {
        let mut formatters: HashMap<LogFormat, Arc<dyn LogFormatter + Send + Sync>> =
            HashMap::new();
        formatters.insert(LogFormat::Json, Arc::new(JsonFormatter::new()));
        formatters.insert(LogFormat::Text, Arc::new(TextFormatter::new()));
        formatters.insert(LogFormat::Structured, Arc::new(StructuredFormatter::new()));

        let outputs = Self::create_outputs(&config)?;
        let filters = Self::create_filters(&config);

        Ok(Self {
            config,
            formatters,
            outputs,
            context: Arc::new(RwLock::new(LogContext::new())),
            metrics: Arc::new(RwLock::new(LogMetrics::new())),
            filters,
        })
    }

    /// Log a message with the specified level
    pub async fn log(
        &self,
        level: LogLevel,
        message: &str,
        component: &str,
    ) -> Result<(), LoggingError> {
        self.log_with_fields(level, message, component, HashMap::new(), None)
            .await
    }

    /// Log a message with structured fields
    pub async fn log_with_fields(
        &self,
        level: LogLevel,
        message: &str,
        component: &str,
        fields: HashMap<String, serde_json::Value>,
        location: Option<LogLocation>,
    ) -> Result<(), LoggingError> {
        // Check if we should log at this level
        if !self.should_log_level(level) {
            return Ok(());
        }

        let context = self.context.read().await;

        let entry = LogEntry {
            timestamp: Utc::now(),
            level,
            message: message.to_string(),
            component: component.to_string(),
            module: None,
            location,
            thread_id: Some(format!("{:?}", std::thread::current().id())),
            correlation_id: context.correlation_id.clone(),
            user_id: context.user_id.clone(),
            session_id: context.session_id.clone(),
            fields,
            metrics: None,
            tags: context.tags.clone(),
            environment: Some(context.environment.clone()),
            error: None,
        };

        drop(context);
        self.process_log_entry(entry).await
    }

    /// Log an error with full error information
    pub async fn log_error(
        &self,
        message: &str,
        component: &str,
        error: &dyn std::error::Error,
        location: Option<LogLocation>,
    ) -> Result<(), LoggingError> {
        let log_error = LogError {
            error_type: std::any::type_name_of_val(error).to_string(),
            error_message: error.to_string(),
            error_code: None,
            stack_trace: Some(format!("{:?}", error)),
            caused_by: error.source().map(|source| {
                Box::new(LogError {
                    error_type: std::any::type_name_of_val(source).to_string(),
                    error_message: source.to_string(),
                    error_code: None,
                    stack_trace: None,
                    caused_by: None,
                })
            }),
        };

        let context = self.context.read().await;

        let entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Error,
            message: message.to_string(),
            component: component.to_string(),
            module: None,
            location,
            thread_id: Some(format!("{:?}", std::thread::current().id())),
            correlation_id: context.correlation_id.clone(),
            user_id: context.user_id.clone(),
            session_id: context.session_id.clone(),
            fields: HashMap::new(),
            metrics: None,
            tags: context.tags.clone(),
            environment: Some(context.environment.clone()),
            error: Some(log_error),
        };

        drop(context);
        self.process_log_entry(entry).await
    }

    /// Log with performance metrics
    pub async fn log_with_metrics(
        &self,
        level: LogLevel,
        message: &str,
        component: &str,
        metrics: LogEntryMetrics,
        location: Option<LogLocation>,
    ) -> Result<(), LoggingError> {
        let context = self.context.read().await;

        let entry = LogEntry {
            timestamp: Utc::now(),
            level,
            message: message.to_string(),
            component: component.to_string(),
            module: None,
            location,
            thread_id: Some(format!("{:?}", std::thread::current().id())),
            correlation_id: context.correlation_id.clone(),
            user_id: context.user_id.clone(),
            session_id: context.session_id.clone(),
            fields: HashMap::new(),
            metrics: Some(metrics),
            tags: context.tags.clone(),
            environment: Some(context.environment.clone()),
            error: None,
        };

        drop(context);
        self.process_log_entry(entry).await
    }

    /// Update the logging context
    pub async fn update_context<F>(&self, updater: F) -> Result<(), LoggingError>
    where
        F: FnOnce(&mut LogContext),
    {
        let mut context = self.context.write().await;
        updater(&mut *context);
        Ok(())
    }

    /// Get current logging metrics
    pub async fn get_metrics(&self) -> LogMetrics {
        self.metrics.read().await.clone()
    }

    /// Flush all output destinations
    pub async fn flush(&self) -> Result<(), LoggingError> {
        for output in &self.outputs {
            output.flush().await?;
        }
        Ok(())
    }

    /// Shutdown the logging system gracefully
    pub async fn shutdown(&self) -> Result<(), LoggingError> {
        self.flush().await?;
        for output in &self.outputs {
            output.close().await?;
        }
        Ok(())
    }

    // Private methods

    fn should_log_level(&self, level: LogLevel) -> bool {
        level >= self.config.min_level
    }

    async fn process_log_entry(&self, entry: LogEntry) -> Result<(), LoggingError> {
        // Apply filters
        for filter in &self.filters {
            if !filter.should_log(&entry) {
                return Ok(());
            }
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.record_entry(&entry);
        }

        // Format and output to all destinations
        for output in &self.outputs {
            let format = output.preferred_format();
            if let Some(formatter) = self.formatters.get(&format) {
                let formatted = formatter.format(&entry)?;
                output.write(&formatted).await?;
            }
        }

        Ok(())
    }

    fn create_outputs(
        config: &LogConfig,
    ) -> Result<Vec<Arc<dyn OutputDestination + Send + Sync>>, LoggingError> {
        let mut outputs: Vec<Arc<dyn OutputDestination + Send + Sync>> = Vec::new();

        for output_config in &config.outputs {
            match output_config {
                LogOutput::Console { format, colored } => {
                    outputs.push(Arc::new(ConsoleOutput::new(*format, *colored)));
                }
                LogOutput::File {
                    path,
                    format,
                    rotation,
                } => {
                    let file_rotation =
                        rotation
                            .as_ref()
                            .map(|r| crate::logging::output::FileRotation {
                                max_size_bytes: r.max_size_bytes,
                                max_files: r.max_files,
                                compress: r.compress,
                            });
                    outputs.push(Arc::new(FileOutput::new(
                        path.clone(),
                        *format,
                        file_rotation,
                    )?));
                }
                LogOutput::Syslog { facility, format } => {
                    let output_facility = match facility {
                        crate::logging::config::SyslogFacility::User => {
                            crate::logging::output::SyslogFacility::User
                        }
                        crate::logging::config::SyslogFacility::Mail => {
                            crate::logging::output::SyslogFacility::Mail
                        }
                        crate::logging::config::SyslogFacility::Daemon => {
                            crate::logging::output::SyslogFacility::Daemon
                        }
                        crate::logging::config::SyslogFacility::Auth => {
                            crate::logging::output::SyslogFacility::Auth
                        }
                        crate::logging::config::SyslogFacility::Syslog => {
                            crate::logging::output::SyslogFacility::Syslog
                        }
                        crate::logging::config::SyslogFacility::Lpr => {
                            crate::logging::output::SyslogFacility::Lpr
                        }
                        crate::logging::config::SyslogFacility::News => {
                            crate::logging::output::SyslogFacility::News
                        }
                        crate::logging::config::SyslogFacility::Uucp => {
                            crate::logging::output::SyslogFacility::Uucp
                        }
                        crate::logging::config::SyslogFacility::Cron => {
                            crate::logging::output::SyslogFacility::Cron
                        }
                        crate::logging::config::SyslogFacility::Authpriv => {
                            crate::logging::output::SyslogFacility::Authpriv
                        }
                        crate::logging::config::SyslogFacility::Ftp => {
                            crate::logging::output::SyslogFacility::Ftp
                        }
                        crate::logging::config::SyslogFacility::Local0 => {
                            crate::logging::output::SyslogFacility::Local0
                        }
                        crate::logging::config::SyslogFacility::Local1 => {
                            crate::logging::output::SyslogFacility::Local1
                        }
                        crate::logging::config::SyslogFacility::Local2 => {
                            crate::logging::output::SyslogFacility::Local2
                        }
                        crate::logging::config::SyslogFacility::Local3 => {
                            crate::logging::output::SyslogFacility::Local3
                        }
                        crate::logging::config::SyslogFacility::Local4 => {
                            crate::logging::output::SyslogFacility::Local4
                        }
                        crate::logging::config::SyslogFacility::Local5 => {
                            crate::logging::output::SyslogFacility::Local5
                        }
                        crate::logging::config::SyslogFacility::Local6 => {
                            crate::logging::output::SyslogFacility::Local6
                        }
                        crate::logging::config::SyslogFacility::Local7 => {
                            crate::logging::output::SyslogFacility::Local7
                        }
                    };
                    outputs.push(Arc::new(SyslogOutput::new(output_facility, *format)?));
                }
            }
        }

        if outputs.is_empty() {
            // Default to console output
            outputs.push(Arc::new(ConsoleOutput::new(LogFormat::Text, true)));
        }

        Ok(outputs)
    }

    fn create_filters(config: &LogConfig) -> Vec<Box<dyn LogFilter + Send + Sync>> {
        let mut filters: Vec<Box<dyn LogFilter + Send + Sync>> = Vec::new();

        // Level filter
        filters.push(Box::new(LevelFilter {
            min_level: config.min_level,
        }));

        // Component filters
        if !config.allowed_components.is_empty() || !config.blocked_components.is_empty() {
            filters.push(Box::new(ComponentFilter {
                allowed_components: config.allowed_components.clone(),
                blocked_components: config.blocked_components.clone(),
            }));
        }

        // Rate limiting
        if let Some(rate_limit) = config.rate_limit_per_minute {
            filters.push(Box::new(RateLimitFilter {
                max_entries_per_minute: rate_limit,
                current_count: Arc::new(RwLock::new(0)),
                last_reset: Arc::new(RwLock::new(Utc::now())),
            }));
        }

        // Sampling
        if let Some(sample_rate) = config.sample_rate {
            filters.push(Box::new(SamplingFilter { sample_rate }));
        }

        filters
    }
}

// Filter implementations
impl LogFilter for ComponentFilter {
    fn should_log(&self, entry: &LogEntry) -> bool {
        if !self.allowed_components.is_empty() {
            return self.allowed_components.contains(&entry.component);
        }

        if !self.blocked_components.is_empty() {
            return !self.blocked_components.contains(&entry.component);
        }

        true
    }

    fn name(&self) -> &str {
        "component"
    }
}

impl LogFilter for LevelFilter {
    fn should_log(&self, entry: &LogEntry) -> bool {
        entry.level >= self.min_level
    }

    fn name(&self) -> &str {
        "level"
    }
}

impl LogFilter for RateLimitFilter {
    fn should_log(&self, _entry: &LogEntry) -> bool {
        // This is a simplified rate limiter - in production you'd want something more sophisticated
        let now = Utc::now();

        // Reset counter if a minute has passed (simplified check)
        if let (Ok(mut count), Ok(mut last_reset)) =
            (self.current_count.try_write(), self.last_reset.try_write())
        {
            if now.signed_duration_since(*last_reset).num_minutes() >= 1 {
                *count = 0;
                *last_reset = now;
            }

            if *count < self.max_entries_per_minute {
                *count += 1;
                true
            } else {
                false
            }
        } else {
            // If we can't acquire locks, allow the log (fail open)
            true
        }
    }

    fn name(&self) -> &str {
        "rate_limit"
    }
}

impl LogFilter for SamplingFilter {
    fn should_log(&self, _entry: &LogEntry) -> bool {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen::<f64>() < self.sample_rate
    }

    fn name(&self) -> &str {
        "sampling"
    }
}

/// Logging system errors
#[derive(Debug, thiserror::Error)]
pub enum LoggingError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Formatter error: {0}")]
    FormatterError(String),

    #[error("Output error: {0}")]
    OutputError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("System error: {0}")]
    SystemError(String),
}

/// Convenience macros for logging
#[macro_export]
macro_rules! log_error {
    ($system:expr, $component:expr, $($arg:tt)*) => {
        $system.log(crate::logging::LogLevel::Error, &format!($($arg)*), $component).await
    };
}

#[macro_export]
macro_rules! log_warn {
    ($system:expr, $component:expr, $($arg:tt)*) => {
        $system.log(crate::logging::LogLevel::Warn, &format!($($arg)*), $component).await
    };
}

#[macro_export]
macro_rules! log_info {
    ($system:expr, $component:expr, $($arg:tt)*) => {
        $system.log(crate::logging::LogLevel::Info, &format!($($arg)*), $component).await
    };
}

#[macro_export]
macro_rules! log_debug {
    ($system:expr, $component:expr, $($arg:tt)*) => {
        $system.log(crate::logging::LogLevel::Debug, &format!($($arg)*), $component).await
    };
}

#[macro_export]
macro_rules! log_trace {
    ($system:expr, $component:expr, $($arg:tt)*) => {
        $system.log(crate::logging::LogLevel::Trace, &format!($($arg)*), $component).await
    };
}

/// Global logging system instance
static GLOBAL_LOGGER: once_cell::sync::OnceCell<Arc<LoggingSystem>> =
    once_cell::sync::OnceCell::new();

/// Initialize the global logging system
pub fn init_global_logger(config: LogConfig) -> Result<(), LoggingError> {
    let logger = Arc::new(LoggingSystem::new(config)?);
    GLOBAL_LOGGER
        .set(logger)
        .map_err(|_| LoggingError::SystemError("Global logger already initialized".to_string()))?;
    Ok(())
}

/// Get the global logging system
pub fn global_logger() -> Option<Arc<LoggingSystem>> {
    GLOBAL_LOGGER.get().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_logging_system_creation() {
        let config = LogConfig::default();
        let system = LoggingSystem::new(config).unwrap();

        assert!(!system.outputs.is_empty());
        assert!(!system.formatters.is_empty());
    }

    #[tokio::test]
    async fn test_basic_logging() {
        let config = LogConfig::default();
        let system = LoggingSystem::new(config).unwrap();

        system
            .log(LogLevel::Info, "Test message", "test_component")
            .await
            .unwrap();
        system.flush().await.unwrap();
    }

    #[tokio::test]
    async fn test_context_updates() {
        let config = LogConfig::default();
        let system = LoggingSystem::new(config).unwrap();

        system
            .update_context(|ctx| {
                ctx.correlation_id = Some("test-123".to_string());
                ctx.user_id = Some("user-456".to_string());
            })
            .await
            .unwrap();

        let context = system.context.read().await;
        assert_eq!(context.correlation_id, Some("test-123".to_string()));
        assert_eq!(context.user_id, Some("user-456".to_string()));
    }
}
