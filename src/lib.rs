// Agentic Development Environment Library
//
// This library provides an intelligent, multi-agent development environment
// built in Rust for AI-assisted code generation on large existing codebases.

// Allow common warnings during active development
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_doc_comments)]
#![allow(clippy::mismatched_lifetime_syntaxes)]
#![allow(mismatched_lifetime_syntaxes)]

pub mod agents;
pub mod ai;
pub mod analytics;
pub mod artifacts;
pub mod blueprint;
pub mod cli;
pub mod codegen;
pub mod config;
pub mod context;
pub mod evaluation;
pub mod error;
pub mod integration;
pub mod integrations;
pub mod interactive;
pub mod logging;
pub mod monitoring;
pub mod plugins;
pub mod sandbox;
pub mod services;
pub mod session;
pub mod telemetry;
pub mod tools;
pub mod secrets;
pub mod shell;
pub mod system_bus;
pub mod web;
pub mod ui;

#[cfg(test)]
pub mod testing;

// Comprehensive test modules
#[cfg(test)]
pub mod tests;

// Integration tests
#[cfg(test)]
mod integration_test;

// Re-export commonly used types
pub use agents::{Agent, AgentError, AgentResult, AgentStatus, AgentTask, TaskPriority};
pub use artifacts::{
    ArtifactManager, EnhancedArtifact, ArtifactDisplay, ArtifactViewerState, ViewMode,
};
pub use codegen::{CodeGenError as CodegenError, CodeGenerator, GenerationConfig, GeneratedCode};
pub use config::{Config, ConfigError, ConfigManager};
pub use context::{CodebaseContext, ContextError, FileContext};
pub use error::{
    ContextualError, DevKitError, DevKitResult, ErrorContext, ErrorHandler, WithContext,
};
pub use logging::{
    LogConfig, LogContext, LogEntry, LogFormat, LogLevel, LoggingError, LoggingSystem,
};
pub use shell::{ShellConfig, ShellError};
pub use evaluation::{
    EvaluationFramework, EvaluationConfig, EvaluationResult, EvaluationError,
    EvaluationContext, EvaluationEnvironment, BuildProfile,
};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
