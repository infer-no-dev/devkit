// Agentic Development Environment Library
//
// This library provides an intelligent, multi-agent development environment
// built in Rust for AI-assisted code generation on large existing codebases.

pub mod agents;
pub mod ai;
pub mod blueprint;
pub mod cli;
pub mod codegen;
pub mod config;
pub mod context;
pub mod error;
pub mod integrations;
pub mod interactive;
pub mod logging;
pub mod plugins;
pub mod services;
pub mod shell;
pub mod system_bus;
pub mod ui;
pub mod web;

#[cfg(test)]
pub mod testing;

// Comprehensive test modules
#[cfg(test)]
pub mod tests;

// Re-export commonly used types
pub use agents::{Agent, AgentError, AgentResult, AgentStatus, AgentTask, TaskPriority};
pub use codegen::{CodeGenError as CodegenError, CodeGenerator, GenerationConfig};
pub use config::{Config, ConfigError, ConfigManager};
pub use context::{CodebaseContext, ContextError, FileContext};
pub use error::{
    ContextualError, DevKitError, DevKitResult, ErrorContext, ErrorHandler, WithContext,
};
pub use logging::{
    LogConfig, LogContext, LogEntry, LogFormat, LogLevel, LoggingError, LoggingSystem,
};
pub use shell::{ShellConfig, ShellError};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
