// Agentic Development Environment Library
//
// This library provides an intelligent, multi-agent development environment
// built in Rust for AI-assisted code generation on large existing codebases.

pub mod agents;
pub mod ai;
pub mod codegen;
pub mod context;
pub mod shell;
pub mod ui;
pub mod config;
pub mod system_bus;
pub mod integrations;
pub mod interactive;
pub mod cli;

#[cfg(test)]
pub mod testing;

// Comprehensive test modules
#[cfg(test)]
pub mod tests;

// Re-export commonly used types
pub use agents::{Agent, AgentTask, AgentResult, AgentError, AgentStatus, TaskPriority};
pub use codegen::{CodeGenerator, GenerationConfig, CodeGenError as CodegenError};
pub use context::{FileContext, CodebaseContext, ContextError};
pub use config::{Config, ConfigManager, ConfigError};
pub use shell::{ShellConfig, ShellError};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
