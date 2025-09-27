# DevKit - AI-Powered Development Toolkit 🚀

> Just describe what you want, no manual coding needed

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://rust-lang.org)
[![Security](https://img.shields.io/badge/security-audited-green.svg)](https://github.com/RustSec/advisory-db)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-passing-brightgreen.svg)](./scripts/dev.sh)

DevKit is an intelligent, multi-agent development environment built in Rust, designed for AI-assisted code generation on large existing codebases. The system leverages multiple concurrent AI agents, advanced code analysis using tree-sitter, and cross-shell compatibility to provide natural language programming assistance.

## ✨ Features

### 🤖 **Multi-Agent AI System**
- **Concurrent AI Agents**: Multiple specialized agents working in parallel
- **Smart Task Distribution**: Automatic workload balancing and prioritization
- **Multi-Provider Support**: Ollama, OpenAI, Anthropic, and more
- **Real-time Monitoring**: Live agent status and performance tracking

### 🧠 **Advanced Code Analysis**
- **Deep Codebase Understanding**: Tree-sitter powered semantic analysis
- **Symbol Indexing**: Fast cross-reference and dependency mapping
- **Intelligent Context**: Understanding of project structure and patterns
- **Repository Integration**: Git-aware analysis with change tracking

### 💻 **Cross-Platform Shell Integration**
- **Universal Compatibility**: Bash, Zsh, Fish, PowerShell support
- **Safe Command Execution**: Timeout protection and error handling
- **Environment Management**: Shell-aware variable and path handling

### 🎯 **Rich User Interface**
- **Terminal Dashboard**: Real-time multi-agent monitoring
- **Interactive Mode**: Conversational code generation
- **Progress Tracking**: Visual feedback for long-running operations
- **Customizable UI**: Configurable layouts and themes

### 🔧 **Enterprise-Ready Configuration**
- **Environment-Specific Configs**: Dev, staging, production profiles
- **Hot Reloading**: Live configuration updates
- **Validation & Defaults**: Smart configuration validation
- **Export/Import**: Portable configuration management

### 📊 **Monitoring & Observability**
- **Structured Logging**: JSON/text logging with context
- **Performance Metrics**: Real-time system health monitoring
- **Alert Management**: Proactive issue detection
- **Health Checks**: Automated system status monitoring

## 🚀 Quick Start

### Installation

#### From Source (Recommended)
```bash
git clone https://github.com/infer-no-dev/devkit.git
cd devkit
cargo build --release
cp target/release/devkit ~/.local/bin/
```

#### Using the Development Script
```bash
./scripts/dev.sh setup    # Install development tools
./scripts/dev.sh check    # Run quality checks
./scripts/dev.sh test     # Run comprehensive tests
```

### Basic Usage

#### 1. Initialize a New Project
```bash
devkit init ./my-project --template rust-web
cd my-project
```

#### 2. Analyze Existing Codebase
```bash
devkit analyze . --output analysis.json
```

#### 3. Generate Code with Natural Language
```bash
devkit generate "Add authentication middleware for JWT tokens" \
    --language rust \
    --context ./src/main.rs \
    --output ./src/auth.rs
```

#### 4. Interactive Development Session
```bash
devkit interactive --project .
```

#### 5. Launch AI Dashboard
```bash
devkit start --project .
```

## 📖 Configuration

### Environment Setup

Create `.devkit/config.toml`:

```toml
[general]
environment = "development"
log_level = "info"
max_context_files = 1000

[agents]
max_concurrent = 4
timeout_seconds = 120

[codegen]
default_language = "rust"
style_preferences = { indentation = "spaces", line_length = 100 }

[codegen.ai_model_settings]
default_provider = "ollama"
default_model = "codellama:7b"

# Configure AI Providers
[ai.ollama]
base_url = "http://localhost:11434"
timeout_seconds = 30

[ai.openai]
api_key = "${OPENAI_API_KEY}"
model = "gpt-4"
max_tokens = 8192

[ai.anthropic]
api_key = "${ANTHROPIC_API_KEY}"
model = "claude-3-sonnet"
max_tokens = 8192
```

### Environment Variables
```bash
export OPENAI_API_KEY="your-openai-key"
export ANTHROPIC_API_KEY="your-anthropic-key"
export DEVKIT_LOG_LEVEL="debug"
export DEVKIT_CONFIG_DIR="$HOME/.devkit"
```

## 🔧 Advanced Usage

### Programmatic API

```rust
use devkit::{
    ai::AIManager,
    agents::AgentSystem,
    codegen::CodeGenerator,
    config::ConfigManager,
    context::ContextManager,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize configuration
    let config_manager = ConfigManager::new()?;
    let config = config_manager.get();

    // Set up AI and agents
    let ai_manager = AIManager::new(config.clone()).await?;
    let agent_system = AgentSystem::new(config.clone()).await?;
    let code_generator = CodeGenerator::new(ai_manager);

    // Analyze codebase
    let context_manager = ContextManager::new(config.clone())?;
    let context = context_manager.analyze_codebase(
        std::env::current_dir()?,
        Default::default()
    ).await?;

    // Generate code with context
    let result = code_generator.generate_code(
        "Create a REST API handler for user management",
        Some("rust".to_string()),
        Some(context)
    ).await?;

    println!("Generated: {}", result.generated_code);
    Ok(())
}
```

### CLI Commands Reference

#### Core Commands
- `devkit init [path]` - Initialize new project
- `devkit analyze [path]` - Analyze codebase
- `devkit generate <prompt>` - Generate code
- `devkit review [path]` - Code quality review
- `devkit interactive` - Start interactive session
- `devkit start` - Launch AI dashboard

#### Management Commands
- `devkit config show` - Display configuration
- `devkit config set <key> <value>` - Update configuration
- `devkit agent list` - List available agents
- `devkit agent status` - Show agent status
- `devkit status` - System health check

#### Development Commands
- `devkit template list` - List project templates
- `devkit shell` - Enhanced shell integration
- `devkit demo` - Run demonstration

## 🏗️ Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────┐
│                    DevKit Core                          │
├─────────────────┬─────────────────┬─────────────────────┤
│   Agent System  │  AI Management  │  Context Analysis   │
│                 │                 │                     │
│ • Task Queue    │ • Multi-Provider│ • Tree-sitter       │
│ • Coordination  │ • Load Balancing│ • Symbol Index      │
│ • Monitoring    │ • Model Selection│ • Dependency Map    │
├─────────────────┼─────────────────┼─────────────────────┤
│  Code Generator │  Shell Manager  │  Configuration      │
│                 │                 │                     │
│ • Templates     │ • Cross-platform│ • Environment       │
│ • Language Det  │ • Safe Execution│ • Hot Reload        │
│ • Style Rules   │ • Error Handling│ • Validation        │
└─────────────────┴─────────────────┴─────────────────────┘
```

### Agent Types

- **🏗️ CodeGenerationAgent**: Creates code from natural language
- **🔍 AnalysisAgent**: Examines code quality and patterns
- **♻️ RefactoringAgent**: Improves and restructures code
- **🔎 CodeReviewAgent**: Performs comprehensive code reviews
- **📊 MonitoringAgent**: Tracks system performance

### Supported Languages

- **Primary**: Rust, Python, JavaScript/TypeScript, Go
- **Supported**: Java, C/C++, Ruby, PHP, C#, Swift
- **Planned**: Kotlin, Scala, Haskell, Elixir

## 🛡️ Security & Quality

### Security Features
- **Dependency Scanning**: Automated vulnerability detection
- **Input Sanitization**: Safe handling of user inputs
- **Secure Communication**: TLS encryption for API calls
- **Audit Logging**: Comprehensive security event logging

### Quality Assurance
- **Property-Based Testing**: Comprehensive test coverage
- **Performance Benchmarking**: Continuous performance monitoring
- **Code Quality Metrics**: Automated quality assessment
- **Security Audits**: Regular dependency security scanning

### Development Scripts
```bash
# Development environment setup
./scripts/dev.sh setup

# Run comprehensive quality checks
./scripts/dev.sh check

# Run all tests including property-based
./scripts/dev.sh test

# Start development watch mode
./scripts/dev.sh watch

# Performance profiling
./scripts/dev.sh profile

# Prepare release
./scripts/dev.sh release 0.1.1
```

## 📊 Monitoring

### Health Monitoring
The system includes comprehensive health monitoring:

```rust
use devkit::monitoring::MonitoringSystem;

let monitoring = MonitoringSystem::new();
monitoring.start().await?;

// Record custom metrics
monitoring.record_metric("code_generation.latency_ms", 150.0, tags);

// Get system health
let health = monitoring.get_health_status().await;
```

### Metrics Collected
- **System Performance**: Memory, CPU, disk usage
- **Agent Activity**: Task completion rates, response times
- **AI Provider**: API call success rates, latency
- **Code Generation**: Success rate, quality scores

## 🧪 Testing

### Running Tests
```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'

# Property-based tests
cargo test property_tests

# All tests
./scripts/dev.sh test
```

### Benchmarks
```bash
# Performance benchmarks
cargo bench

# With profiling
./scripts/dev.sh profile
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup
1. Clone the repository
2. Run `./scripts/dev.sh setup`
3. Run `./scripts/dev.sh check` to verify setup
4. Start development with `./scripts/dev.sh watch`

### Code Style
- Follow Rust standard formatting (`cargo fmt`)
- Ensure clippy passes (`cargo clippy`)
- Add comprehensive tests for new features
- Update documentation for public APIs

## 📄 License

This project is licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## 🆘 Support

- **Documentation**: [docs.devkit.dev](https://docs.devkit.dev)
- **Issues**: [GitHub Issues](https://github.com/infer-no-dev/devkit/issues)
- **Discussions**: [GitHub Discussions](https://github.com/infer-no-dev/devkit/discussions)
- **Discord**: [DevKit Community](https://discord.gg/devkit)

## 🗺️ Roadmap

### v0.2.0 - Enhanced Intelligence
- [ ] Advanced code understanding with semantic analysis
- [ ] Multi-language project support
- [ ] Plugin architecture for custom agents
- [ ] Web-based dashboard

### v0.3.0 - Enterprise Features
- [ ] Team collaboration features
- [ ] Role-based access control
- [ ] Audit logging and compliance
- [ ] Enterprise deployment guides

### v1.0.0 - Production Ready
- [ ] Stable API and configuration format
- [ ] Comprehensive documentation
- [ ] Performance optimizations
- [ ] Extended platform support

---

<div align="center">

**[⬆️ Back to top](#devkit---ai-powered-development-toolkit-)**

Made with ❤️ by the DevKit team

</div>