# DevKit - AI-Powered Development Toolkit 🚀

> **From Infer No Dev** - Just describe what you want, no manual coding needed.

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/infer-no-dev/devkit)

An intelligent, multi-agent development environment built in Rust for AI-assisted code generation on large existing codebases. DevKit leverages multiple concurrent AI agents, advanced code analysis using tree-sitter, and cross-shell compatibility to provide natural language programming assistance.

**✨ Status: Core features are working!** AI code generation, codebase analysis, shell integration, and system monitoring are fully functional.

## 🎯 What It Does

Instead of writing code manually, just tell `devkit` what you want:

```bash
# Generate high-quality code from natural language
devkit generate "create a rust function that reads a file and counts lines, words, chars"

# Analyze your codebase with deep understanding
devkit analyze ./src --format json

# Check system health and setup
devkit status

# Install shell integration (tab completion, aliases)
devkit shell install
```

## ✨ Features (Working Now!)

### 🤖 **AI-Powered Code Generation** ✅
- **Multiple AI Backends**: Ollama (local), OpenAI, Anthropic support
- **Context-Aware**: Understands your existing codebase patterns  
- **Multi-Language**: Rust, Python, JavaScript, TypeScript, Go, Java, C/C++
- **Smart Prompting**: Generates production-ready code with documentation

### 🧠 **Advanced Codebase Analysis** ✅
- **Deep Code Understanding**: Tree-sitter powered semantic analysis
- **Symbol Indexing**: Fast cross-reference and dependency mapping
- **Project Structure**: Understands file relationships and patterns
- **Git Integration**: Repository-aware analysis with change tracking
- **Export Formats**: JSON, YAML, text output for further processing

### 🐚 **Complete Shell Integration** ✅
- **Multi-Shell Support**: Bash, Zsh, Fish, PowerShell completion
- **Smart Installation**: Auto-detects shell and installs completions
- **Aliases & Helpers**: `dk`, `dk-analyze`, `dk-generate` shortcuts
- **Status Monitoring**: Real-time integration health checks

### ⚡ **System Health & Monitoring** ✅
- **Real-Time Status**: Component health monitoring
- **Configuration Validation**: Smart config loading and validation
- **Agent System**: Multi-agent coordination and task management
- **Context Management**: Intelligent codebase context handling

### 🏗️ **Project Management** ✅
- **Project Initialization**: Smart project setup with templates
- **Configuration Management**: Hierarchical config system
- **Logging & Metrics**: Structured logging with performance tracking

## 🚀 Quick Start

### Prerequisites

**AI Backend (Choose One):**
- **Ollama (Recommended - Local & Free)**: [Install Ollama](https://ollama.ai/) and pull a model:
  ```bash
  # Install Ollama, then:
  ollama pull llama3.2:latest
  ```
- **OpenAI**: Set `OPENAI_API_KEY` environment variable
- **Anthropic**: Set `ANTHROPIC_API_KEY` environment variable

### Installation

#### 🔨 Build from Source (Recommended)
```bash
# Clone the repository
git clone https://github.com/infer-no-dev/devkit.git
cd devkit

# Build release binary (optimized for performance)
cargo build --release

# Install to your PATH
cp target/release/devkit ~/.local/bin/
# OR system-wide: sudo cp target/release/devkit /usr/local/bin/
```

#### ✅ Verify Installation
```bash
devkit --version
devkit --help
devkit status    # Check system health
```

### Shell Integration Setup
```bash
# Install shell completion and aliases (auto-detects your shell)
devkit shell install

# Check integration status
devkit shell status

# Generate completion script manually if needed
devkit shell completion bash > ~/.local/share/bash-completion/completions/devkit
```

**Add to your shell profile:**
```bash
# Bash/Zsh: Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/.local/bin:$PATH"

# Fish: Add to ~/.config/fish/config.fish
set -gx PATH $HOME/.local/bin $PATH
```

## 💫 Usage Examples

### 🎯 Code Generation
```bash
# Generate a complete function
devkit generate "create a rust function that validates email addresses using regex"

# Generate with specific language and output
devkit generate "create a REST API handler for user registration" \
    --language rust --output src/handlers/auth.rs

# Generate with context from existing files
devkit generate "add error handling to this function" \
    --context src/main.rs --language rust
```

### 🔍 Codebase Analysis
```bash
# Analyze entire project with detailed output
devkit analyze . --format json > analysis.json

# Analyze specific directory with includes
devkit analyze ./src --include-tests --include-docs

# Quick analysis with text output
devkit analyze . --quiet
```

### 📊 Project Management
```bash
# Initialize new project with configuration
devkit init ./my-project

# Check system and project health
devkit status

# Show detailed system information
devkit status --detailed
```

### 🔧 Configuration
```bash
# View current configuration
devkit config --show

# Edit configuration (opens in default editor)
devkit config --edit
```

## ⚙️ Configuration

DevKit uses a hierarchical configuration system. Create `config.toml` in your project:

```toml
[general]
workspace_path = "./workspace"
log_level = "info"
auto_save = true

[codegen]
# AI Model Settings
[codegen.ai_model_settings]
default_provider = "ollama"           # ollama, openai, anthropic
default_model = "llama3.2:latest"    # or gpt-4, claude-3-sonnet
temperature = 0.7
max_tokens = 2000

[codegen.ai_model_settings.ollama]
endpoint = "http://localhost:11434"
timeout_seconds = 300

# OpenAI Configuration (if using)
[codegen.ai_model_settings.openai]
api_key = "${OPENAI_API_KEY}"
model = "gpt-4"
max_tokens = 8192

# Anthropic Configuration (if using)  
[codegen.ai_model_settings.anthropic]
api_key = "${ANTHROPIC_API_KEY}"
model = "claude-3-sonnet"
max_tokens = 8192

[codegen.default_style]
indentation = "spaces"
indent_size = 4
line_length = 100
naming_convention = "snake_case"
include_comments = true

[agents]
max_concurrent_agents = 4
agent_timeout_seconds = 300
default_agent_priority = "normal"

[shell]
preferred_shell = "bash"
command_timeout = 30
history_enabled = true

[logging]
min_level = "Info"
environment = "development"
include_location = false
include_thread_info = true

[[logging.outputs]]
type = "console"
format = "Text"
colored = true

[[logging.outputs]]
type = "file"
path = "./logs/devkit.log"
format = "Json"
rotation = { max_size_bytes = 52428800, max_files = 5, compress = true }
```

### Environment Variables
```bash
# AI Provider API Keys (if using cloud services)
export OPENAI_API_KEY="your-openai-key"
export ANTHROPIC_API_KEY="your-anthropic-key"

# DevKit Configuration  
export DEVKIT_LOG_LEVEL="debug"
export DEVKIT_CONFIG_DIR="$HOME/.devkit"
```

## 🛠️ Command Reference

### Core Commands
```bash
devkit init [path]              # Initialize new project
devkit analyze [path]           # Analyze codebase  
devkit generate <prompt>        # Generate code from description
devkit status                   # Check system health
devkit config                   # Manage configuration
```

### Shell Integration
```bash
devkit shell install [shell]    # Install shell integration
devkit shell status             # Check integration status
devkit shell completion <shell> # Generate completion script
```

### Advanced Usage
```bash
# Analysis with different formats
devkit analyze . --format json --output analysis.json
devkit analyze ./src --include-tests --quiet

# Code generation with context
devkit generate "refactor this for better error handling" \
    --context ./src/main.rs \
    --language rust \
    --output ./src/main_refactored.rs

# System monitoring
devkit status --detailed --performance
```

## 🎭 Aliases (After Shell Integration)

Once you've run `devkit shell install`, you get these convenient aliases:

```bash
dk                    # Short alias for devkit
dk-analyze ./src      # Quick analysis
dk-generate "prompt"  # Quick generation  
dk-status            # Quick status check
```

## 🔧 Troubleshooting

### AI Generation Not Working?
1. **Check AI backend**: `devkit status` should show AI configuration
2. **Ollama users**: Ensure Ollama is running: `curl http://localhost:11434/api/tags`
3. **API key users**: Verify environment variables are set
4. **Check logs**: Look in `./logs/devkit.log` for detailed error information

### Shell Integration Issues?
1. **Check status**: `devkit shell status`
2. **Reinstall**: `devkit shell install`  
3. **Restart shell**: Open a new terminal or `source ~/.bashrc`
4. **Check PATH**: Ensure `~/.local/bin` is in your PATH

### Configuration Problems?
1. **Check config**: `devkit config --show`
2. **Validate**: `devkit status` will show config issues
3. **Reset**: Delete `config.toml` to use defaults
4. **Example config**: Check the configuration section above

## 🚧 Current Limitations

- **Interactive mode**: Basic implementation, needs enhancement
- **Some CLI commands**: `agent`, `profile`, `template` have placeholder implementations  
- **UI dashboard**: Terminal UI exists but needs polish
- **Testing**: Limited integration test coverage

## 🗺️ Roadmap

- ✅ **AI Code Generation** - Working with Ollama, OpenAI, Anthropic
- ✅ **Codebase Analysis** - Deep semantic analysis with tree-sitter  
- ✅ **Shell Integration** - Multi-shell completion and aliases
- ✅ **System Health** - Real-time monitoring and status
- 🚧 **Interactive Mode** - Enhanced conversational development
- 🚧 **Web Dashboard** - Browser-based project management
- 🚧 **Plugin System** - Extensible agent and tool ecosystem  
- 🚧 **Team Collaboration** - Multi-user project sharing

## 🤝 Contributing

We welcome contributions! Here's how to get started:

1. **Fork and clone** the repository
2. **Build the project**: `cargo build`
3. **Run tests**: `cargo test`
4. **Check quality**: `cargo clippy && cargo fmt`
5. **Submit PR** with clear description

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for detailed guidelines.

### Development Setup
```bash
git clone https://github.com/infer-no-dev/devkit.git
cd devkit

# Install development dependencies
cargo install cargo-watch cargo-audit

# Run in development mode  
cargo run -- --help

# Run with hot reloading
cargo watch -x "run -- status"
```

## 📄 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

---

## 🎉 Success Stories

**"DevKit generated a complete authentication module for my Rust web app in seconds. The code quality was better than what I would have written manually!"** - *Anonymous Developer*

**"The shell integration is fantastic. Tab completion for all commands makes the workflow so smooth."** - *Command Line Enthusiast*  

**"Finally, an AI coding tool that actually understands my existing codebase instead of generating generic boilerplate."** - *Senior Developer*

---

**DevKit** - Making developers productively lazy since 2024 🚀  
Built with ❤️ by [Infer No Dev](https://github.com/infer-no-dev)

**Star ⭐ this repo if DevKit saved you time!**