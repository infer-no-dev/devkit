# DevKit API Reference

## Overview

DevKit is an AI-powered development toolkit that provides natural language programming assistance through a multi-agent system. This document covers the public API for integrating with DevKit programmatically.

## Core Components

### AI Manager

The AI Manager coordinates multiple AI providers and handles model selection.

```rust
use devkit::ai::AIManager;
use devkit::config::Config;

// Initialize with default configuration
let config = Config::default();
let ai_manager = AIManager::new(config).await?;

// Generate code
let result = ai_manager.generate_code(
    "Create a function to calculate fibonacci numbers",
    Some("rust".to_string()),
    None
).await?;
```

### Agent System

Multi-agent coordination for complex development tasks.

```rust
use devkit::agents::{AgentSystem, AgentTask, TaskPriority};

// Create agent system
let agent_system = AgentSystem::new(config).await?;

// Submit a task
let task = AgentTask::new(
    "code_generation",
    "Generate a REST API handler",
    TaskPriority::High,
    HashMap::new()
);

let result = agent_system.submit_task(task).await?;
```

### Context Manager

Analyzes codebases and provides intelligent context.

```rust
use devkit::context::{ContextManager, AnalysisConfig};
use std::path::PathBuf;

let context_manager = ContextManager::new(config)?;
let analysis_config = AnalysisConfig::default();

let context = context_manager.analyze_codebase(
    PathBuf::from("./my-project"),
    analysis_config
).await?;

println!("Found {} files with {} symbols", 
    context.files.len(), 
    context.metadata.indexed_symbols
);
```

### Code Generator

High-level code generation interface.

```rust
use devkit::codegen::CodeGenerator;

let code_gen = CodeGenerator::new(ai_manager);

let result = code_gen.generate_code(
    "Create a simple web server",
    Some("rust".to_string()),
    Some(context)  // Optional project context
).await?;

println!("Generated: {}", result.generated_code);
```

## Configuration

### Environment-Specific Configs

```toml
# development.toml
[general]
log_level = "debug"
environment = "development"

[agents]
max_concurrent = 2
timeout_seconds = 30

[codegen.ai_model_settings]
default_provider = "ollama"
default_model = "codellama:7b"
```

### Programmatic Configuration

```rust
use devkit::config::{Config, ConfigManager};

// Load configuration
let config_manager = ConfigManager::new()?;
let config = config_manager.load_environment("development").await?;

// Update configuration
config_manager.set("codegen.ai_model_settings.default_model", "codellama:13b").await?;
```

## CLI Integration

### Basic Commands

```bash
# Initialize a new project
devkit init ./new-project --template rust-web

# Analyze existing codebase
devkit analyze ./project --output analysis.json

# Interactive development
devkit interactive --project ./project

# Start dashboard
devkit start --project ./project
```

### Advanced Usage

```bash
# Generate code with specific context
devkit generate "Add authentication middleware" \
    --language rust \
    --context ./src/main.rs \
    --output ./src/auth.rs

# Review code quality
devkit review ./src \
    --severity medium \
    --format json \
    --output review_report.json
```

## Event System

### System Events

```rust
use devkit::system_bus::{SystemBus, SystemEvent, ComponentType};

let system_bus = SystemBus::new();
let handle = system_bus.get_handle();

// Subscribe to events
let mut receiver = handle.subscribe(|event| {
    matches!(event, SystemEvent::CodeGenerationCompleted { .. })
});

// Publish events
handle.publish(SystemEvent::CodeGenerationStarted {
    request_id: "req_123".to_string(),
    prompt: "Generate function".to_string(),
    language: Some("rust".to_string()),
}).await?;
```

## Error Handling

### Error Types

```rust
use devkit::error::{DevKitError, ContextualError, RecoveryStrategy};

match result {
    Ok(value) => println!("Success: {:?}", value),
    Err(DevKitError::AIProvider(e)) => {
        eprintln!("AI Provider error: {}", e);
        // Handle AI-specific errors
    },
    Err(DevKitError::Configuration(e)) => {
        eprintln!("Configuration error: {}", e);
        // Handle config errors
    },
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

### Recovery Strategies

```rust
use devkit::services::CodeAnalysisService;

let service = CodeAnalysisService::new(ai_manager, context_manager);

// Service handles retries and fallbacks automatically
let result = service.analyze_with_recovery(
    &file_path,
    RecoveryStrategy::RetryWithExponentialBackoff { max_attempts: 3 }
).await?;
```

## Testing

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use devkit::testing::{TestEnvironment, MockAIProvider};
    
    #[tokio::test]
    async fn test_code_generation() {
        let env = TestEnvironment::new().await;
        let mock_ai = MockAIProvider::new();
        
        // Configure mock responses
        mock_ai.expect_generate_code()
            .returning(|_| Ok("fn hello() { println!(\"Hello!\"); }".to_string()));
        
        let result = env.generate_code("Create hello function").await?;
        assert!(!result.generated_code.is_empty());
    }
}
```

### Integration Testing

```rust
use devkit::testing::{IntegrationTestRunner, TestProject};

#[tokio::test]
async fn test_full_workflow() {
    let runner = IntegrationTestRunner::new();
    let project = TestProject::rust_web_service();
    
    // Test complete workflow
    let result = runner
        .with_project(project)
        .analyze_codebase()
        .generate_feature("authentication")
        .run_tests()
        .execute()
        .await?;
    
    assert!(result.all_passed());
}
```

## Performance

### Benchmarking

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_code_generation(c: &mut Criterion) {
    c.bench_function("generate_simple_function", |b| {
        b.iter(|| {
            // Benchmark code generation
        });
    });
}

criterion_group!(benches, bench_code_generation);
criterion_main!(benches);
```

### Monitoring

```rust
use devkit::monitoring::{MonitoringSystem, MetricsCollector};

let monitoring = MonitoringSystem::new();
monitoring.start().await?;

// Record custom metrics
monitoring.record_metric("code_generation.latency_ms", 150.0, HashMap::new());

// Get system health
let health = monitoring.get_health_status().await;
println!("System health: {:?}", health);
```

## Best Practices

### Configuration Management

1. Use environment-specific configurations
2. Keep sensitive data in environment variables
3. Validate configuration at startup
4. Support hot-reloading for development

### Error Handling

1. Use structured error types
2. Provide contextual information
3. Implement recovery strategies
4. Log errors appropriately

### Performance

1. Use async/await for I/O operations
2. Cache analysis results when possible
3. Monitor resource usage
4. Profile critical paths

### Security

1. Sanitize all inputs
2. Use secure communication channels
3. Audit dependencies regularly
4. Follow least-privilege principle

## Examples

See the `/examples` directory for complete working examples:

- `examples/basic_usage.rs` - Basic API usage
- `examples/custom_agent.rs` - Creating custom agents
- `examples/batch_processing.rs` - Processing multiple files
- `examples/plugin_development.rs` - Developing plugins

## Support

- Documentation: https://docs.devkit.dev
- Issues: https://github.com/infer-no-dev/devkit/issues
- Discussions: https://github.com/infer-no-dev/devkit/discussions
- Discord: https://discord.gg/devkit