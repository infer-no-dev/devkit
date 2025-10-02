# Contributing to devkit

Thank you for your interest in contributing to devkit! This document provides guidelines and information for contributors.

## 🚀 Quick Start

### Development Environment Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/infer-no-dev/devkit.git
   cd devkit
   ```

2. **Install Rust (if not already installed):**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

3. **Install development dependencies:**
   ```bash
   # Required for cross-compilation (optional)
   cargo install cross
   
   # For code formatting and linting
   rustup component add rustfmt clippy
   ```

4. **Set up AI backend (for code generation):**
   ```bash
   # Install Ollama from https://ollama.ai
   # Pull the default model
   ollama pull llama3.2:latest
   ```

5. **Build and test:**
   ```bash
   cargo build --release  # Release mode recommended for performance
   cargo test
   cargo clippy -- -D warnings
   ```

6. **Try the working features:**
   ```bash
   # Check system status
   cargo run -- status
   
   # Install shell integration (aliases and completions)
   cargo run -- shell install
   
   # Test code generation
   cargo run -- generate "create a hello world function in rust"
   
   # Test codebase analysis
   cargo run -- analyze ./src --format json
   ```

## ✅ Current Working Features

As of the latest updates, these features are fully functional:

- ✅ **AI-powered code generation** using Ollama/local LLMs
- ✅ **Codebase analysis** with tree-sitter parsing and symbol indexing
- ✅ **Shell integration** with completions and aliases (bash, zsh, fish, PowerShell)
- ✅ **System status monitoring** with health checks
- ✅ **Multi-format output** (JSON, YAML, text) for analysis
- ✅ **Cross-platform support** (Linux, macOS, Windows)
- ✅ **Configuration management** with project-specific settings

### 🔧 Features In Development

- 🚧 **Interactive mode** for conversational development
- 🚧 **Multi-agent coordination** with task prioritization
- 🚧 **Advanced AI model switching** (OpenAI, Anthropic, etc.)
- 🚧 **Enhanced error handling** and logging systems
- 🚧 **Integration test suite** for end-to-end workflows

## 🎯 Types of Contributions

### 🐛 Bug Fixes
- Check existing issues first
- Create tests that reproduce the bug
- Fix the issue with minimal changes
- Ensure all tests pass

### ✨ New Features
- Discuss large features in issues first
- Follow the existing architecture patterns
- Add comprehensive tests
- Update documentation

### 🤖 New Agent Types
- Implement the `Agent` trait
- Add appropriate error handling
- Include integration tests
- Document agent capabilities

### 🌍 Language Support
- Add language detection patterns
- Create code generation templates
- Add syntax validation
- Include example outputs

### 📚 Documentation
- Keep README.md up to date
- Add inline code documentation
- Update WARP.md for architectural changes
- Write helpful examples

## 🏗️ Development Workflow

### 1. **Before You Start**
```bash
# Create a feature branch
git checkout -b feature/your-feature-name

# Make sure you're up to date
git pull origin main
```

### 2. **Development Process**
```bash
# Make your changes
# Test frequently
cargo test

# Check formatting
cargo fmt --check

# Run clippy for lints
cargo clippy -- -D warnings

# Test with release build
cargo test --release
```

### 3. **Before Submitting**
```bash
# Run the full test suite
cargo test --all-features

# Test core functionality manually
cargo run -- status
cargo run -- generate "test function" --language rust
cargo run -- analyze ./src --format json

# Test shell integration
cargo run -- shell status
cargo run -- shell install --dry-run

# Verify no warnings
cargo clippy -- -D warnings

# Check formatting
cargo fmt --check
```

## 🧪 Testing Guidelines

### Test Structure
- **Unit tests**: In the same file as the code (`#[cfg(test)]` modules)
- **Integration tests**: In the `tests/` directory
- **Example tests**: In the `examples/` directory

### Writing Good Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_feature_happy_path() {
        // Arrange
        let mut agent = MockAgent::new("test", "mock");
        
        // Act
        let result = agent.process_task(task).await;
        
        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().success, true);
    }
}
```

### Test Coverage
- Aim for high test coverage on core functionality
- Test both success and failure cases
- Include edge cases and boundary conditions
- Mock external dependencies (AI APIs, file system, etc.)

## 🏛️ Architecture Guidelines

### Core Principles
1. **Agent-based design**: Each major function is an agent
2. **Async-first**: Use tokio for all I/O operations
3. **Error handling**: Use `Result` types and proper error propagation
4. **Configuration**: Support both file-based and environment configuration
5. **Cross-platform**: Support Linux, macOS, and Windows

### Code Organization
```
src/
├── agents/          # Agent implementations
├── ai/             # AI provider integrations
├── cli/            # Command-line interface
├── codegen/        # Code generation logic
├── config/         # Configuration management
├── context/        # Codebase analysis and context
├── logging/        # Structured logging system
├── shell/          # Shell integration
├── ui/            # Terminal UI components
└── main.rs        # Application entry point
```

### Adding New CLI Commands

To add a new command to the CLI:

1. **Add to `src/cli.rs`:**
```rust
#[derive(Subcommand)]
pub enum Commands {
    // ... existing commands
    YourCommand {
        #[arg(help = "Description of the argument")]
        input: String,
        #[arg(long, help = "Optional flag")]
        flag: bool,
    },
}
```

2. **Handle in `src/main.rs`:**
```rust
Commands::YourCommand { input, flag } => {
    commands::your_command::execute(input, flag).await?;
}
```

3. **Create command implementation:**
```rust
// src/commands/your_command.rs
use anyhow::Result;

pub async fn execute(input: String, flag: bool) -> Result<()> {
    println!("Executing command with input: {}, flag: {}", input, flag);
    Ok(())
}
```

### Adding New Agents
```rust
use async_trait::async_trait;
use crate::agents::{Agent, AgentTask, AgentResult, AgentError, AgentStatus};

#[derive(Debug)]
pub struct YourNewAgent {
    // Agent state
}

#[async_trait]
impl Agent for YourNewAgent {
    fn id(&self) -> &str { "your_agent_id" }
    fn name(&self) -> &str { "Your Agent Name" }
    fn status(&self) -> AgentStatus { /* ... */ }
    fn can_handle(&self, task_type: &str) -> bool { /* ... */ }
    fn capabilities(&self) -> Vec<String> { /* ... */ }
    
    async fn process_task(&mut self, task: AgentTask) -> Result<AgentResult, AgentError> {
        // Your implementation
    }
    
    async fn shutdown(&mut self) -> Result<(), AgentError> {
        // Cleanup logic
    }
}
```

## 📝 Code Style

### Rust Style Guidelines
- Follow `rustfmt` formatting (run `cargo fmt`)
- Follow `clippy` lints (run `cargo clippy`)
- Use descriptive variable names
- Add documentation for public APIs
- Prefer explicit error handling over `.unwrap()`

### Commit Messages
```
type(scope): brief description

- More detailed explanation
- Why the change was made
- Any breaking changes

Fixes #123
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

## 🔍 Pull Request Process

### 1. **Create the PR**
- Use a descriptive title
- Fill out the PR template
- Link related issues
- Add screenshots/examples if relevant

### 2. **PR Requirements**
- [ ] All tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Documentation is updated
- [ ] Breaking changes are documented

### 3. **Review Process**
- Address all review feedback
- Keep the PR up to date with main
- Be responsive to questions
- Tests must pass before merging

## 🐛 Issue Reporting

### Bug Reports
Include:
- **Environment**: OS, Rust version, devkit version (`devkit --version`)
- **AI Backend**: Ollama status (`ollama list`), model availability
- **Shell**: Which shell you're using (bash, zsh, fish, PowerShell)
- **Steps to reproduce**: Exact commands/actions
- **Expected behavior**: What should happen
- **Actual behavior**: What actually happens
- **Error messages**: Full error output
- **System status**: Output of `devkit status`
- **Configuration**: Contents of `.devkit/config.toml` if relevant
- **Additional context**: Logs, configurations, etc.

### Testing Your Bug Fix
```bash
# Test the specific scenario
cargo run -- [your-failing-command]

# Verify system health
cargo run -- status

# Test with different configurations
RUST_LOG=debug cargo run -- [command] # for detailed logs
```

### Feature Requests
Include:
- **Problem description**: What problem does this solve?
- **Proposed solution**: How should it work?
- **Alternatives considered**: Other approaches
- **Additional context**: Use cases, examples

## 🚀 Release Process

### Version Numbering
We use [Semantic Versioning](https://semver.org/):
- `MAJOR.MINOR.PATCH`
- Major: Breaking changes
- Minor: New features (backwards compatible)
- Patch: Bug fixes (backwards compatible)

### Release Checklist
- [ ] Update version in `Cargo.toml`
- [ ] Update `CHANGELOG.md`
- [ ] Tag the release: `git tag v1.2.3`
- [ ] Push tags: `git push origin --tags`
- [ ] GitHub Actions will handle the rest

## 🤝 Community

### Getting Help
- **GitHub Issues**: Bug reports, feature requests
- **GitHub Discussions**: General questions, ideas
- **Documentation**: Check WARP.md for architecture details

### Code of Conduct
Be respectful, inclusive, and constructive. We're all here to make development easier and more productive.

## 📚 Additional Resources

- [Rust Documentation](https://doc.rust-lang.org/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Async Programming in Rust](https://rust-lang.github.io/async-book/)
- [Project Architecture (WARP.md)](./WARP.md)

---

Thank you for contributing to devkit! 🎉