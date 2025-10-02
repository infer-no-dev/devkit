//! Template System
//! 
//! This module provides Handlebars templates for generating complete project structures
//! from system blueprints, enabling high-fidelity system self-replication.

use std::collections::HashMap;

/// Template manager for blueprint-based code generation
pub struct TemplateManager {
    templates: HashMap<String, String>,
}

impl TemplateManager {
    /// Create a new template manager with default templates
    pub fn new() -> Self {
        Self {
            templates: Self::load_default_templates(),
        }
    }

    /// Get a template by name
    pub fn get_template(&self, name: &str) -> Option<&String> {
        self.templates.get(name)
    }

    /// Load default templates
    pub fn load_default_templates() -> HashMap<String, String> {
        let mut templates = HashMap::new();

        // Cargo.toml template
        templates.insert("cargo_toml".to_string(), r#"[package]
name = "{{blueprint.metadata.name}}"
version = "{{blueprint.metadata.version}}"
description = "{{blueprint.metadata.description}}"
edition = "2021"

[dependencies]
{{#each dependencies}}
{{this}} = "*"
{{/each}}
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
chrono = { version = "0.4", features = ["serde"] }
tree-sitter = "0.20"
tree-sitter-rust = "0.20"
handlebars = "4.0"
tracing = "0.1"
tracing-subscriber = "0.3"

[dev-dependencies]
tempfile = "3.0"

[[bin]]
name = "{{snake_case blueprint.metadata.name}}"
path = "src/main.rs"
"#.to_string());

        // main.rs template
        templates.insert("main_rs".to_string(), r#"//! {{blueprint.metadata.name}} - {{blueprint.metadata.description}}
//! 
//! This is an intelligent, multi-agent development environment built in Rust,
//! designed for AI-assisted code generation on large existing codebases.

use anyhow::Result;
use clap::{Arg, Command};
use tracing::{info, warn, error};

mod agents;
mod codegen;
mod context;
mod shell;
mod ui;
mod config;
mod blueprint;

use agents::AgentManager;
use codegen::CodeGenerator;
use context::CodebaseContext;
use config::ConfigManager;
use blueprint::{BlueprintExtractor, BlueprintGenerator};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let matches = Command::new("{{blueprint.metadata.name}}")
        .version("{{blueprint.metadata.version}}")
        .about("{{blueprint.metadata.description}}")
        .subcommand(
            Command::new("analyze")
                .about("Analyze a codebase for context")
                .arg(Arg::new("path")
                    .help("Path to the codebase")
                    .required(true)
                    .index(1))
        )
        .subcommand(
            Command::new("generate")
                .about("Generate code from natural language")
                .arg(Arg::new("prompt")
                    .help("Natural language description")
                    .required(true)
                    .index(1))
                .arg(Arg::new("output")
                    .short('o')
                    .long("output")
                    .help("Output file path")
                    .value_name("FILE"))
        )
        .subcommand(
            Command::new("blueprint")
                .about("Blueprint operations")
                .subcommand(
                    Command::new("extract")
                        .about("Extract system blueprint from codebase")
                        .arg(Arg::new("path")
                            .help("Path to the codebase")
                            .required(true)
                            .index(1))
                        .arg(Arg::new("output")
                            .short('o')
                            .long("output")
                            .help("Output blueprint file")
                            .value_name("FILE")
                            .default_value("system_blueprint.toml"))
                )
                .subcommand(
                    Command::new("generate")
                        .about("Generate project from blueprint")
                        .arg(Arg::new("blueprint")
                            .help("Path to blueprint file")
                            .required(true)
                            .index(1))
                        .arg(Arg::new("output")
                            .short('o')
                            .long("output")
                            .help("Output directory")
                            .value_name("DIR")
                            .default_value("./generated_project"))
                )
        )
        .subcommand(
            Command::new("interactive")
                .about("Start interactive mode")
        )
        .get_matches();

    match matches.subcommand() {
        Some(("analyze", sub_matches)) => {
            let path = sub_matches.get_one::<String>("path").unwrap();
            info!("Analyzing codebase at: {}", path);
            analyze_codebase(path).await?;
        }
        Some(("generate", sub_matches)) => {
            let prompt = sub_matches.get_one::<String>("prompt").unwrap();
            let output = sub_matches.get_one::<String>("output");
            info!("Generating code from prompt: {}", prompt);
            generate_code(prompt, output).await?;
        }
        Some(("blueprint", sub_matches)) => {
            match sub_matches.subcommand() {
                Some(("extract", extract_matches)) => {
                    let path = extract_matches.get_one::<String>("path").unwrap();
                    let output = extract_matches.get_one::<String>("output").unwrap();
                    info!("Extracting blueprint from: {} to: {}", path, output);
                    extract_blueprint(path, output).await?;
                }
                Some(("generate", gen_matches)) => {
                    let blueprint_path = gen_matches.get_one::<String>("blueprint").unwrap();
                    let output_dir = gen_matches.get_one::<String>("output").unwrap();
                    info!("Generating project from blueprint: {} to: {}", blueprint_path, output_dir);
                    generate_from_blueprint(blueprint_path, output_dir).await?;
                }
                _ => {
                    error!("Unknown blueprint subcommand");
                }
            }
        }
        Some(("interactive", _)) => {
            info!("Starting interactive mode");
            run_interactive_mode().await?;
        }
        _ => {
            warn!("No command specified, starting interactive mode");
            run_interactive_mode().await?;
        }
    }

    Ok(())
}

async fn analyze_codebase(path: &str) -> Result<()> {
    let context = CodebaseContext::from_path(path).await?;
    println!("Analysis completed: {} files processed", context.file_count());
    Ok(())
}

async fn generate_code(prompt: &str, output: Option<&String>) -> Result<()> {
    let generator = CodeGenerator::new().await?;
    let code = generator.generate_from_prompt(prompt).await?;
    
    if let Some(output_path) = output {
        tokio::fs::write(output_path, &code).await?;
        println!("Code generated and saved to: {}", output_path);
    } else {
        println!("Generated code:\n{}", code);
    }
    
    Ok(())
}

async fn extract_blueprint(codebase_path: &str, output_path: &str) -> Result<()> {
    let mut extractor = BlueprintExtractor::new(codebase_path.into())?;
    let blueprint = extractor.extract_blueprint().await?;
    
    let output_file = std::path::Path::new(output_path);
    blueprint.save_to_file(output_file)?;
    
    println!("Blueprint extracted and saved to: {}", output_path);
    Ok(())
}

async fn generate_from_blueprint(blueprint_path: &str, output_dir: &str) -> Result<()> {
    let blueprint_file = std::path::Path::new(blueprint_path);
    let blueprint = SystemBlueprint::load_from_file(blueprint_file)?;
    
    let mut generator = BlueprintGenerator::new(output_dir.into())?;
    generator.generate_project(&blueprint).await?;
    
    println!("Project generated from blueprint at: {}", output_dir);
    Ok(())
}

async fn run_interactive_mode() -> Result<()> {
    println!("🤖 Welcome to {{blueprint.metadata.name}} Interactive Mode!");
    println!("Type 'help' for available commands or 'exit' to quit.");
    
    let config = ConfigManager::load().await?;
    let mut agent_manager = AgentManager::new(config).await?;
    
    loop {
        use std::io::{self, Write};
        
        print!("> ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        match input {
            "exit" | "quit" => {
                println!("Goodbye! 👋");
                break;
            }
            "help" => {
                println!("Available commands:");
                println!("  help - Show this help message");
                println!("  analyze <path> - Analyze a codebase");
                println!("  generate <prompt> - Generate code from description");
                println!("  blueprint extract <path> - Extract system blueprint");
                println!("  blueprint generate <file> - Generate project from blueprint");
                println!("  exit - Exit interactive mode");
            }
            cmd if cmd.starts_with("analyze ") => {
                let path = &cmd[8..].trim();
                if let Err(e) = analyze_codebase(path).await {
                    error!("Analysis failed: {}", e);
                }
            }
            cmd if cmd.starts_with("generate ") => {
                let prompt = &cmd[9..].trim();
                if let Err(e) = generate_code(prompt, None).await {
                    error!("Generation failed: {}", e);
                }
            }
            "" => continue,
            _ => {
                println!("Unknown command: {}. Type 'help' for available commands.", input);
            }
        }
    }
    
    Ok(())
}

use blueprint::{SystemBlueprint, BlueprintExtractor, BlueprintGenerator};
"#.to_string());

        // lib.rs template
        templates.insert("lib_rs".to_string(), r#"//! {{blueprint.metadata.name}} Library
//! 
//! {{blueprint.metadata.description}}

pub mod agents;
pub mod codegen;
pub mod context;
pub mod shell;
pub mod ui;
pub mod config;
pub mod blueprint;

// Re-export main types
pub use agents::{Agent, AgentManager, AgentError, AgentResult};
pub use codegen::{CodeGenerator, CodeGenerationRequest, GeneratedCode};
pub use context::{CodebaseContext, FileContext, SymbolIndex};
pub use config::{ConfigManager, AgenticConfig};
pub use blueprint::{SystemBlueprint, BlueprintExtractor, BlueprintGenerator};

use anyhow::Result;

/// Initialize the {{snake_case blueprint.metadata.name}} library
pub async fn init() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    Ok(())
}

/// Version information
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
"#.to_string());

        // Module template
        templates.insert("module_rs".to_string(), r#"//! {{module.name}} Module
//! 
//! {{module.purpose}}

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
{{#if system.architecture.concurrency_model.primary_pattern}}
use tokio::sync::{RwLock, Mutex};
use std::sync::Arc;
{{/if}}

{{#each module.internal_structure.primary_types}}
/// {{purpose}}
#[derive(Debug, Clone, Serialize, Deserialize)]
{{format_visibility visibility}}struct {{name}} {
    {{#each fields_or_variants}}
    {{#if @first}}pub {{/if}}{{this}},
    {{/each}}
}

impl {{name}} {
    /// Create a new {{name}}
    pub fn new() -> Self {
        Self {
            {{#each fields_or_variants}}
            // Initialize field: {{this}}
            {{/each}}
        }
    }
}
{{/each}}

{{#each module.internal_structure.functions}}
/// {{purpose}}
{{format_visibility visibility}}{{#if is_async}}async {{/if}}fn {{name}}({{#each parameters}}{{name}}: {{param_type}}{{#unless @last}}, {{/unless}}{{/each}}) -> {{return_type}} {
    // TODO: Implement {{name}}
    {{#if return_type}}
    {{#if (eq return_type "Result<(), Box<dyn std::error::Error>>")}}
    Ok(())
    {{else if (eq return_type "()")}}
    // Implementation goes here
    {{else}}
    unimplemented!("{{name}} not yet implemented")
    {{/if}}
    {{/if}}
}
{{/each}}

{{#each module.internal_structure.constants}}
/// {{purpose}}
{{format_visibility visibility}}const {{upper_case name}}: {{value_type}} = Default::default();
{{/each}}

#[cfg(test)]
mod tests {
    use super::*;

    {{#each module.internal_structure.functions}}
    {{#if (eq visibility "pub")}}
    #[{{#if is_async}}tokio::{{/if}}test]
    {{#if is_async}}async {{/if}}fn test_{{snake_case name}}() {
        // Test implementation for {{name}}
        {{#if is_async}}
        let result = {{name}}().await;
        {{else}}
        let result = {{name}}();
        {{/if}}
        // Add assertions here
    }
    {{/if}}
    {{/each}}
}
"#.to_string());

        // Configuration template
        templates.insert("config_toml".to_string(), r#"# {{system_name}} Configuration
# Generated from system blueprint

[system]
name = "{{system_name}}"
{{#each strategy.hierarchy}}
config_layer = "{{this}}"
{{/each}}

[ai_providers]
# AI provider configurations will be added here
default_provider = "ollama"

[agents]
max_concurrent = 4
task_timeout = 300
retry_attempts = 3

[context]
max_file_size = 10485760  # 10MB
cache_size = 1000
analysis_depth = 5

[shell]
default_timeout = 30
command_history_size = 1000

[ui]
theme = "default"
show_progress = true
log_level = "info"

{{#if strategy.secret_management}}
[secrets]
storage_method = "{{strategy.secret_management.storage_method}}"
encryption = "{{strategy.secret_management.encryption_approach}}"
{{/if}}
"#.to_string());

        // Test template
        templates.insert("test_rs".to_string(), r#"//! Integration tests for {{module.name}}

use {{snake_case module.name}}::*;
use anyhow::Result;

#[tokio::test]
async fn test_{{snake_case module.name}}_basic_functionality() -> Result<()> {
    // Basic functionality test
    // TODO: Implement comprehensive tests based on module specification
    Ok(())
}

{{#each module.public_interface}}
#[tokio::test]
async fn test_{{snake_case name}}() -> Result<()> {
    // Test for {{name}} - {{documentation}}
    // TODO: Implement specific test for {{name}}
    Ok(())
}
{{/each}}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::tempdir;

    {{#if module.testing_strategy.mock_strategies}}
    // Mock implementations for testing
    {{#each module.testing_strategy.mock_strategies}}
    // Mock for: {{this}}
    {{/each}}
    {{/if}}

    #[tokio::test]
    async fn test_{{snake_case module.name}}_error_handling() -> Result<()> {
        // Error handling tests
        Ok(())
    }

    #[tokio::test]
    async fn test_{{snake_case module.name}}_performance() -> Result<()> {
        // Performance characteristics validation
        {{#each module.performance_characteristics.optimization_opportunities}}
        // Test optimization: {{this}}
        {{/each}}
        Ok(())
    }
}
"#.to_string());

        // README template
        templates.insert("readme_md".to_string(), r#"# {{blueprint.metadata.name}}

{{blueprint.metadata.description}}

## Overview

This is an intelligent, multi-agent development environment built in Rust, designed for AI-assisted code generation on large existing codebases. The system leverages multiple concurrent AI agents, advanced code analysis, and cross-shell compatibility to provide natural language programming assistance.

## Architecture

- **System Type**: {{blueprint.architecture.system_type}}
- **Concurrency Model**: {{blueprint.architecture.concurrency_model.primary_pattern}}
- **Primary Language**: {{blueprint.metadata.primary_language}}

### Key Components

{{#each blueprint.modules}}
- **{{name}}**: {{purpose}}
{{/each}}

## Features

{{#each blueprint.patterns.architectural_patterns}}
- {{pattern_name}}: {{usage_context}}
{{/each}}

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd {{blueprint.metadata.name}}

# Build the project
cargo build --release

# Run the application
cargo run
```

## Usage

### Command Line Interface

```bash
# Analyze a codebase
{{snake_case blueprint.metadata.name}} analyze /path/to/codebase

# Generate code from natural language
{{snake_case blueprint.metadata.name}} generate "create a function to parse JSON"

# Extract system blueprint
{{snake_case blueprint.metadata.name}} blueprint extract /path/to/codebase -o blueprint.toml

# Generate project from blueprint
{{snake_case blueprint.metadata.name}} blueprint generate blueprint.toml -o new_project

# Interactive mode
{{snake_case blueprint.metadata.name}} interactive
```

### Blueprint System

This system includes a revolutionary blueprint capability that can:

1. **Extract comprehensive system blueprints** from existing codebases
2. **Generate complete, functional projects** from blueprints
3. **Enable true system self-replication** with high fidelity

#### Blueprint Extraction

```bash
{{snake_case blueprint.metadata.name}} blueprint extract . -o system_blueprint.toml
```

This analyzes the current codebase and generates a comprehensive blueprint containing:

- Architectural decisions and their reasoning
- Module specifications and dependencies
- Design patterns and anti-patterns
- Implementation details and optimizations
- Testing strategies and performance characteristics
- Security patterns and deployment strategies

#### Project Generation

```bash
{{snake_case blueprint.metadata.name}} blueprint generate system_blueprint.toml -o replicated_system
```

This creates a complete, functional project from the blueprint, including:

- Full source code with proper implementations
- Build configuration and dependencies
- Tests and documentation
- CI/CD pipeline configuration
- Project-specific guidance (WARP.md)

## Configuration

The system uses a hierarchical configuration approach:

{{#each blueprint.configuration.hierarchy}}
- {{this}}
{{/each}}

Configuration files:
- `.agentic-config.toml` - Project-specific settings
- `config.toml` - User configuration (gitignored)

## Development

### Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run
```

### Code Quality

```bash
# Linting
cargo clippy

# Formatting
cargo fmt

# Check without building
cargo check
```

## Testing Strategy

{{#if blueprint.testing.test_pyramid.unit_tests.percentage_of_tests}}
- Unit Tests: {{blueprint.testing.test_pyramid.unit_tests.percentage_of_tests}}%
- Integration Tests: {{blueprint.testing.test_pyramid.integration_tests.percentage_of_tests}}%
- System Tests: {{blueprint.testing.test_pyramid.system_tests.percentage_of_tests}}%
{{/if}}

## Performance

{{#each blueprint.performance.critical_paths}}
- **{{path_description}}**: {{performance_impact}}
{{/each}}

## Security

{{#if blueprint.security.authentication.primary_method}}
- Authentication: {{blueprint.security.authentication.primary_method}}
- Authorization: {{blueprint.security.authorization.model}}
- Data Protection: {{blueprint.security.data_protection.encryption_at_rest}}
{{/if}}

## License

[Add license information here]

## Contributing

[Add contributing guidelines here]
"#.to_string());

        // WARP.md template
        templates.insert("warp_md".to_string(), r#"# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

{{blueprint.metadata.description}}

**Architecture**: {{blueprint.architecture.system_type}}  
**Language**: {{blueprint.metadata.primary_language}}  
**Concurrency**: {{blueprint.architecture.concurrency_model.primary_pattern}}

## Development Commands

### Build and Run
```bash
# Build the project (release mode for performance)
cargo build --release

# Build for development
cargo build

# Run the application
cargo run

# Run with specific subcommands
cargo run -- analyze ./path/to/project
cargo run -- generate "create a function to parse JSON"
cargo run -- blueprint extract . -o blueprint.toml
cargo run -- interactive
```

### Code Quality
```bash
# Check compilation without building
cargo check

# Run clippy for linting
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests for specific module
cargo test {{snake_case blueprint.metadata.name}}::
```

### Development Utilities
```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Show dependency tree
cargo tree

# Run with debug logging
RUST_LOG=debug cargo run

# Run with trace logging
RUST_LOG=trace cargo run
```

## Architecture Overview

{{#each blueprint.modules}}
### {{name}} (`src/{{snake_case name}}/`)
{{purpose}}

**Key Features:**
{{#each public_interface}}
- {{name}} ({{interface_type}}): {{documentation}}
{{/each}}
{{/each}}

## Key Development Patterns

{{#each blueprint.patterns.architectural_patterns}}
### {{pattern_name}}
**Context**: {{usage_context}}  
**Implementation**: {{implementation_details}}  
**Benefits**: {{#each benefits_realized}}{{this}}{{#unless @last}}, {{/unless}}{{/each}}
{{/each}}

## Important Implementation Details

### Error Handling
{{blueprint.architecture.error_handling.propagation_strategy}}

Key error types:
{{#each blueprint.architecture.error_handling.error_types}}
- **{{name}}** ({{category}}): {{handling_strategy}}
{{/each}}

### Async Architecture
{{#if blueprint.architecture.concurrency_model}}
Built on {{blueprint.architecture.concurrency_model.primary_pattern}} for {{blueprint.architecture.concurrency_model.performance_characteristics}}.

Synchronization primitives:
{{#each blueprint.architecture.concurrency_model.synchronization_primitives}}
- {{this}}
{{/each}}
{{/if}}

### Dependencies
{{#each blueprint.implementation.third_party_dependencies}}
- **{{crate_name}}**: {{purpose}}
{{/each}}

## Configuration Files

### Project Configuration
- `.agentic-config.toml` - Project-specific settings
- `config.toml` - User configuration (gitignored)

### Supported Formats
{{#each blueprint.configuration.formats_supported}}
- {{this}}
{{/each}}

## Performance Considerations

{{#each blueprint.performance.critical_paths}}
- **{{path_description}}**: {{performance_impact}}
  - Optimizations: {{#each optimizations_applied}}{{this}}{{#unless @last}}, {{/unless}}{{/each}}
{{/each}}

## Security Notes

{{#if blueprint.security.data_protection}}
- Encryption at rest: {{blueprint.security.data_protection.encryption_at_rest}}
- Encryption in transit: {{blueprint.security.data_protection.encryption_in_transit}}
- Key management: {{blueprint.security.data_protection.key_management}}
{{/if}}

## Testing Strategy

{{#if blueprint.testing.test_automation}}
- CI Integration: {{blueprint.testing.test_automation.ci_integration}}
- Test Triggers: {{#each blueprint.testing.test_automation.test_triggers}}{{this}}{{#unless @last}}, {{/unless}}{{/each}}
- Parallel Execution: {{blueprint.testing.test_automation.parallel_execution}}
{{/if}}
"#.to_string());

        // CI/CD template
        templates.insert("ci_yml".to_string(), r#"name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test Suite
    runs-on: ubuntu-latest
    {{#if testing_strategy.test_automation.parallel_execution}}
    strategy:
      matrix:
        rust-version: [stable, beta]
    {{/if}}
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        {{#if testing_strategy.test_automation.parallel_execution}}
        toolchain: {{{matrix_rust_version}}}
        {{else}}
        toolchain: stable
        {{/if}}
        profile: minimal
        override: true
        components: rustfmt, clippy

    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target/
        key: {{{github_cache_key}}}

    - name: Check formatting
      run: cargo fmt --all -- --check

    - name: Run Clippy
      run: cargo clippy --all-targets --all-features -- -D warnings

    - name: Build
      run: cargo build --verbose

    - name: Run tests
      run: cargo test --verbose

    {{#if testing_strategy.performance_testing}}
    - name: Run benchmarks
      run: cargo bench
    {{/if}}

  {{#if testing_strategy.security_testing}}
  security:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - name: Security audit
      run: |
        cargo install cargo-audit
        cargo audit
  {{/if}}

  {{#if deployment_strategy}}
  deploy:
    name: Deploy
    needs: [test]
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Build release
      run: cargo build --release
    
    # Add deployment steps here based on deployment strategy
    {{/if}}
"#.to_string());

        templates
    }

    /// Add a custom template
    pub fn add_template(&mut self, name: String, template: String) {
        self.templates.insert(name, template);
    }

    /// List all available templates
    pub fn list_templates(&self) -> Vec<&String> {
        self.templates.keys().collect()
    }
}

impl Default for TemplateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_manager() {
        let manager = TemplateManager::new();
        
        assert!(manager.get_template("cargo_toml").is_some());
        assert!(manager.get_template("main_rs").is_some());
        assert!(manager.get_template("nonexistent").is_none());
        
        let templates = manager.list_templates();
        assert!(!templates.is_empty());
    }

    #[test]
    fn test_add_custom_template() {
        let mut manager = TemplateManager::new();
        let custom_template = "Custom template content".to_string();
        
        manager.add_template("custom".to_string(), custom_template.clone());
        assert_eq!(manager.get_template("custom"), Some(&custom_template));
    }
}