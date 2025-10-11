//! Comprehensive Error Handling Module
//!
//! This module provides standardized error handling patterns across the entire devkit codebase.
//! It includes unified error types, result wrappers, error context, and recovery strategies.

use std::error::Error;
use std::fmt;
use thiserror::Error;

/// Main error type that encompasses all possible errors in the system
#[derive(Debug, Error)]
pub enum DevKitError {
    // Core system errors
    #[error("Agent error: {0}")]
    Agent(#[from] crate::agents::AgentError),

    #[error("Configuration error: {0}")]
    Config(#[from] crate::config::ConfigError),

    #[error("AI service error: {0}")]
    AI(#[from] crate::ai::AIError),

    #[error("Context analysis error: {0}")]
    Context(#[from] crate::context::ContextError),

    #[error("Shell operation error: {0}")]
    Shell(#[from] crate::shell::ShellError),

    #[error("Code generation error: {0}")]
    Codegen(#[from] crate::codegen::CodeGenError),

    // Infrastructure errors
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML serialization error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Channel error: {0}")]
    Channel(String),

    // Business logic errors
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    #[error("Operation not supported: {operation} for {context}")]
    UnsupportedOperation { operation: String, context: String },

    #[error("Resource not found: {resource_type} '{name}'")]
    ResourceNotFound { resource_type: String, name: String },

    #[error("Permission denied: {action} on {resource}")]
    PermissionDenied { action: String, resource: String },

    #[error("Timeout occurred: {operation} after {timeout_ms}ms")]
    Timeout { operation: String, timeout_ms: u64 },

    #[error("Validation failed: {field} - {message}")]
    ValidationError { field: String, message: String },

    #[error("Dependency error: {dependency} - {reason}")]
    DependencyError { dependency: String, reason: String },

    // System state errors
    #[error("Invalid system state: {state} - {reason}")]
    InvalidState { state: String, reason: String },

    #[error("Resource exhausted: {resource} - {details}")]
    ResourceExhausted { resource: String, details: String },

    #[error("Concurrent access conflict: {resource}")]
    ConcurrencyError { resource: String },

    // User-facing errors
    #[error("User error: {message}")]
    UserError { message: String },

    #[error("Command failed: {command} - {reason}")]
    CommandFailed { command: String, reason: String },

    // Generic fallback
    #[error("Internal error: {message}")]
    Internal { message: String },
}

/// Result type alias for DevKit operations
pub type DevKitResult<T> = Result<T, DevKitError>;

/// Error context for providing additional information
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub component: String,
    pub details: Option<String>,
    pub timestamp: std::time::SystemTime,
    pub correlation_id: Option<String>,
}

impl ErrorContext {
    pub fn new(operation: &str, component: &str) -> Self {
        Self {
            operation: operation.to_string(),
            component: component.to_string(),
            details: None,
            timestamp: std::time::SystemTime::now(),
            correlation_id: None,
        }
    }

    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }

    pub fn with_correlation_id(mut self, id: &str) -> Self {
        self.correlation_id = Some(id.to_string());
        self
    }
}

/// Enhanced error with context information
#[derive(Debug)]
pub struct ContextualError {
    pub error: DevKitError,
    pub context: ErrorContext,
    pub cause_chain: Vec<String>,
}

impl ContextualError {
    pub fn new(error: DevKitError, context: ErrorContext) -> Self {
        let cause_chain = Self::build_cause_chain(&error);
        Self {
            error,
            context,
            cause_chain,
        }
    }

    fn build_cause_chain(error: &DevKitError) -> Vec<String> {
        let mut chain = vec![error.to_string()];
        let mut current = error.source();

        while let Some(err) = current {
            chain.push(err.to_string());
            current = err.source();
        }

        chain
    }

    pub fn root_cause(&self) -> &str {
        self.cause_chain.last().unwrap_or(&self.cause_chain[0])
    }
}

impl fmt::Display for ContextualError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Error in {} ({}): {}",
            self.context.component, self.context.operation, self.error
        )?;

        if let Some(details) = &self.context.details {
            writeln!(f, "Details: {}", details)?;
        }

        if let Some(correlation_id) = &self.context.correlation_id {
            writeln!(f, "Correlation ID: {}", correlation_id)?;
        }

        if self.cause_chain.len() > 1 {
            writeln!(f, "Cause chain:")?;
            for (i, cause) in self.cause_chain.iter().enumerate() {
                writeln!(f, "  {}: {}", i + 1, cause)?;
            }
        }

        Ok(())
    }
}

impl std::error::Error for ContextualError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Helper trait for adding context to errors
pub trait WithContext<T> {
    fn with_context(self, context: ErrorContext) -> Result<T, ContextualError>;
    fn with_operation(self, operation: &str, component: &str) -> Result<T, ContextualError>;
}

impl<T, E> WithContext<T> for Result<T, E>
where
    E: Into<DevKitError>,
{
    fn with_context(self, context: ErrorContext) -> Result<T, ContextualError> {
        self.map_err(|e| ContextualError::new(e.into(), context))
    }

    fn with_operation(self, operation: &str, component: &str) -> Result<T, ContextualError> {
        let context = ErrorContext::new(operation, component);
        self.with_context(context)
    }
}

/// Error recovery strategies
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry the operation with the same parameters
    Retry { max_attempts: usize, delay_ms: u64 },
    /// Retry with exponential backoff
    RetryWithBackoff {
        max_attempts: usize,
        base_delay_ms: u64,
        max_delay_ms: u64,
    },
    /// Use fallback value or operation
    Fallback,
    /// Fail fast - don't attempt recovery
    FailFast,
    /// Skip this operation and continue
    Skip,
    /// Prompt user for intervention
    UserIntervention,
}

/// Error handler for implementing recovery strategies
#[derive(Debug)]
pub struct ErrorHandler {
    pub strategies: std::collections::HashMap<String, RecoveryStrategy>,
    pub default_strategy: RecoveryStrategy,
}

impl Default for ErrorHandler {
    fn default() -> Self {
        let mut strategies = std::collections::HashMap::new();

        // Define default strategies for different error types
        strategies.insert(
            "network".to_string(),
            RecoveryStrategy::RetryWithBackoff {
                max_attempts: 3,
                base_delay_ms: 1000,
                max_delay_ms: 10000,
            },
        );
        strategies.insert(
            "io".to_string(),
            RecoveryStrategy::Retry {
                max_attempts: 2,
                delay_ms: 500,
            },
        );
        strategies.insert("validation".to_string(), RecoveryStrategy::UserIntervention);
        strategies.insert("permission".to_string(), RecoveryStrategy::UserIntervention);
        strategies.insert(
            "timeout".to_string(),
            RecoveryStrategy::Retry {
                max_attempts: 2,
                delay_ms: 1000,
            },
        );

        Self {
            strategies,
            default_strategy: RecoveryStrategy::FailFast,
        }
    }
}

impl ErrorHandler {
    pub fn get_strategy(&self, error_type: &str) -> &RecoveryStrategy {
        self.strategies
            .get(error_type)
            .unwrap_or(&self.default_strategy)
    }

    pub async fn handle_error(&self, error: &DevKitError) -> RecoveryStrategy {
        let error_type = match error {
            DevKitError::Http(_) => "network",
            DevKitError::IO(_) => "io",
            DevKitError::ValidationError { .. } => "validation",
            DevKitError::PermissionDenied { .. } => "permission",
            DevKitError::Timeout { .. } => "timeout",
            _ => "general",
        };

        self.get_strategy(error_type).clone()
    }
}

/// Utility functions for error handling
pub mod utils {
    use super::*;

    /// Check if an error is recoverable
    pub fn is_recoverable(error: &DevKitError) -> bool {
        matches!(
            error,
            DevKitError::Http(_)
                | DevKitError::IO(_)
                | DevKitError::Timeout { .. }
                | DevKitError::Channel(_)
                | DevKitError::ResourceExhausted { .. }
        )
    }

    /// Check if an error is user-facing (should be shown to user)
    pub fn is_user_facing(error: &DevKitError) -> bool {
        matches!(
            error,
            DevKitError::UserError { .. }
                | DevKitError::CommandFailed { .. }
                | DevKitError::ValidationError { .. }
                | DevKitError::PermissionDenied { .. }
                | DevKitError::ResourceNotFound { .. }
        )
    }

    /// Simplify error message for user display
    pub fn simplify_error_message(error: &DevKitError) -> String {
        match error {
            DevKitError::IO(e) => format!("File operation failed: {}", e),
            DevKitError::Http(e) => format!("Network request failed: {}", e),
            DevKitError::ValidationError { field, message } => {
                format!("Invalid {}: {}", field, message)
            }
            DevKitError::ResourceNotFound {
                resource_type,
                name,
            } => {
                format!("{} '{}' not found", resource_type, name)
            }
            DevKitError::PermissionDenied { action, resource } => {
                format!("Permission denied: cannot {} {}", action, resource)
            }
            _ => error.to_string(),
        }
    }

    /// Create a DevKitError from a string message
    pub fn internal_error(message: &str) -> DevKitError {
        DevKitError::Internal {
            message: message.to_string(),
        }
    }

    /// Create a validation error
    pub fn validation_error(field: &str, message: &str) -> DevKitError {
        DevKitError::ValidationError {
            field: field.to_string(),
            message: message.to_string(),
        }
    }

    /// Create a resource not found error
    pub fn not_found_error(resource_type: &str, name: &str) -> DevKitError {
        DevKitError::ResourceNotFound {
            resource_type: resource_type.to_string(),
            name: name.to_string(),
        }
    }

    /// Create a user error
    pub fn user_error(message: &str) -> DevKitError {
        DevKitError::UserError {
            message: message.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new("test_op", "test_component")
            .with_details("test details")
            .with_correlation_id("123");

        assert_eq!(context.operation, "test_op");
        assert_eq!(context.component, "test_component");
        assert_eq!(context.details, Some("test details".to_string()));
        assert_eq!(context.correlation_id, Some("123".to_string()));
    }

    #[test]
    fn test_with_context() {
        let result: Result<(), std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "test file not found",
        ));

        let contextual_result = result.with_operation("read_file", "file_manager");
        assert!(contextual_result.is_err());

        let err = contextual_result.unwrap_err();
        assert_eq!(err.context.operation, "read_file");
        assert_eq!(err.context.component, "file_manager");
    }

    #[test]
    fn test_error_utils() {
        let io_error = DevKitError::IO(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "access denied",
        ));

        assert!(utils::is_recoverable(&io_error));
        assert!(!utils::is_user_facing(&io_error));

        let user_error = utils::user_error("Invalid input provided");
        assert!(!utils::is_recoverable(&user_error));
        assert!(utils::is_user_facing(&user_error));
    }
}
