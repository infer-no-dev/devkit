# devkit

> **From Infer No Dev** - Just describe what you want, no manual coding needed.

An AI-powered development toolkit that turns natural language into code. Built in Rust for developers who are too lazy to write code manually (and that's a good thing).

## 🎯 What It Does

Instead of writing code yourself, just tell `devkit` what you want:

```bash
devkit generate "create a REST API endpoint for user authentication"
devkit test "generate unit tests for the payment processing module"  
devkit optimize "analyze this codebase for performance bottlenecks"
devkit analyze "what does this legacy code actually do?"
```

## ✨ Features

### 🤖 **Multi-Agent Intelligence**
- **Analysis Agent**: Understands your existing codebase
- **Code Generation Agent**: Creates new code from descriptions  
- **Test Generation Agent**: Writes comprehensive test suites
- **Optimization Agent**: Finds performance improvements
- **Debugging Agent**: Helps track down issues

### 🧠 **Smart Context Awareness**
- Analyzes your entire codebase for context
- Understands project structure and dependencies
- Maintains consistency with existing code patterns
- Git repository awareness and integration

### 🔥 **Multi-Language Support**
Generates high-quality code in:
- Rust, Python, JavaScript, TypeScript
- Go, Java, and more
- Automatically detects target language from context

### ⚡ **Built for Speed**
- Written in Rust for maximum performance
- Concurrent agent processing
- Optimized for large codebases
- Fast analysis and generation

## 🚀 Quick Start

### Installation

#### 🚀 Quick Install (Recommended)

**Linux & macOS:**
```bash
curl -sSL https://raw.githubusercontent.com/infer-no-dev/devkit/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
# Download and run the install script (coming soon)
# For now, use manual installation below
```

#### 📦 Package Managers

**Cargo (Rust):**
```bash
cargo install devkit
```

**Homebrew (macOS/Linux):**
```bash
# Coming soon
brew install devkit
```

#### 💾 Manual Installation

1. **Download pre-built binary:**
   - Visit the [releases page](https://github.com/infer-no-dev/devkit/releases)
   - Download the appropriate binary for your platform:
     - `devkit-x86_64-unknown-linux-gnu.tar.gz` (Linux x64)
     - `devkit-aarch64-unknown-linux-gnu.tar.gz` (Linux ARM64)
     - `devkit-x86_64-apple-darwin.tar.gz` (macOS Intel)
     - `devkit-aarch64-apple-darwin.tar.gz` (macOS Apple Silicon)
     - `devkit-x86_64-pc-windows-msvc.zip` (Windows x64)

2. **Extract and install:**
   ```bash
   # Linux/macOS
   tar -xzf devkit-*.tar.gz
   chmod +x devkit
   mv devkit ~/.local/bin/  # or /usr/local/bin/ for system-wide
   
   # Windows
   # Extract ZIP and add to PATH
   ```

#### 🔨 Build from Source

```bash
# Clone the repository
git clone https://github.com/infer-no-dev/devkit.git
cd devkit

# Build release binary
cargo build --release

# The binary will be at ./target/release/devkit
# Move to PATH
mv target/release/devkit ~/.local/bin/
```

#### ✅ Verify Installation

```bash
devkit --version
devkit --help
```

> **Note:** Make sure `~/.local/bin` is in your `$PATH`. Add this to your shell profile:
> ```bash
> export PATH="$HOME/.local/bin:$PATH"
> ```

### Basic Usage

```bash
# Initialize in your project
devkit init

# Generate code from natural language
devkit generate "create a function to calculate compound interest"

# Analyze existing code
devkit analyze ./src/main.rs

# Generate tests
devkit test "create tests for the user authentication module"

# Get optimization suggestions  
devkit optimize "analyze performance of the database queries"

# Interactive mode
devkit interactive
```

## 🛠️ Commands

### Core Commands
- `devkit generate <description>` - Generate code from natural language
- `devkit analyze <path>` - Analyze and understand existing code
- `devkit test <description>` - Generate comprehensive test suites
- `devkit optimize <description>` - Get performance optimization suggestions
- `devkit interactive` - Start conversational development mode

### Setup Commands
- `devkit init` - Initialize devkit in your project
- `devkit config` - Configure settings and preferences
- `devkit version` - Show version and system information

## 🏗️ How It Works

1. **Understand**: Analyzes your codebase structure, dependencies, and patterns
2. **Infer**: Uses AI agents to understand what you actually want
3. **Generate**: Creates code that fits seamlessly into your existing project
4. **Optimize**: Suggests improvements and catches potential issues

## 💡 Philosophy

We believe developers shouldn't waste time on repetitive coding tasks. Instead of manually writing boilerplate, tests, or debugging code, just describe what you want and let `devkit` handle the implementation.

**Too lazy to code manually?** Perfect. That's exactly who this is for.

## 🔧 Configuration

`devkit` can be configured via:
- `devkit.toml` - Project-specific settings
- `~/.devkit/config.toml` - Global user preferences  
- Command-line flags
- Environment variables

## 🤝 Contributing

We welcome contributions! Whether it's:
- New agent types
- Language support
- Bug fixes
- Documentation improvements

See `CONTRIBUTING.md` for guidelines.

## 📄 License

Licensed under either of:
- Apache License, Version 2.0
- MIT License

at your option.

---

**devkit** - Making developers productively lazy since 2024  
Built with ❤️ by [Infer No Dev](https://github.com/infer-no-dev)
