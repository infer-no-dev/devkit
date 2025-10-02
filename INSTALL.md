# DevKit Installation Guide 🚀

Quick reference for installing and setting up DevKit.

## 🎯 Prerequisites

**Choose an AI Backend:**

### Option 1: Ollama (Recommended - Free & Local)
```bash
# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Pull a code-focused model
ollama pull llama3.2:latest
# OR for larger/better model:
ollama pull codellama:latest
```

### Option 2: OpenAI (Cloud Service)
```bash
export OPENAI_API_KEY="your-api-key-here"
```

### Option 3: Anthropic Claude (Cloud Service)
```bash
export ANTHROPIC_API_KEY="your-api-key-here"
```

## 🛠️ Install DevKit

### Build from Source
```bash
# Clone repository
git clone https://github.com/infer-no-dev/devkit.git
cd devkit

# Build release binary (optimized)
cargo build --release

# Install to PATH
cp target/release/devkit ~/.local/bin/
```

### Verify Installation
```bash
# Check version
devkit --version

# Check system health
devkit status

# View all commands
devkit --help
```

## 🐚 Shell Integration (Highly Recommended)

```bash
# Auto-install completion & aliases for your shell
devkit shell install

# Check integration status  
devkit shell status

# Restart your shell or source config
source ~/.bashrc  # or ~/.zshrc, ~/.config/fish/config.fish
```

**After shell integration, you get:**
- Tab completion for all commands
- `dk` alias (short for `devkit`)  
- Helper functions: `dk-analyze`, `dk-generate`, `dk-status`

## 🚀 Quick Test

```bash
# Test AI code generation
devkit generate "create a hello world function in rust"

# Test codebase analysis
devkit analyze . --format text

# Check system status
devkit status
```

## 🔧 Configuration

Create `config.toml` in your project directory:

```toml
[codegen.ai_model_settings]
default_provider = "ollama"        # or "openai", "anthropic"
default_model = "llama3.2:latest"  # or "gpt-4", "claude-3-sonnet"

[codegen.ai_model_settings.ollama]
endpoint = "http://localhost:11434"

# Optional: API keys for cloud providers
# [codegen.ai_model_settings.openai]
# api_key = "${OPENAI_API_KEY}"
# model = "gpt-4"

# [codegen.ai_model_settings.anthropic] 
# api_key = "${ANTHROPIC_API_KEY}"
# model = "claude-3-sonnet"
```

## ❓ Troubleshooting

### "AI generation failed"
1. **Ollama users**: Check if running: `curl http://localhost:11434/api/tags`
2. **API users**: Verify environment variables: `echo $OPENAI_API_KEY`
3. **Check status**: `devkit status` should show ✅ for all components

### "Command not found: devkit"
1. **Check PATH**: `echo $PATH | grep .local/bin`
2. **Add to PATH**: Add to shell profile: `export PATH="$HOME/.local/bin:$PATH"`
3. **Restart shell**: Open new terminal or `source ~/.bashrc`

### Shell completion not working
1. **Check status**: `devkit shell status`
2. **Reinstall**: `devkit shell install`
3. **Restart shell**: Open new terminal

## 🎉 You're Ready!

**Basic workflow:**
```bash
# Initialize project
devkit init ./my-project

# Generate code
devkit generate "create a function that processes JSON data"

# Analyze codebase
devkit analyze ./src

# Check system health
devkit status
```

**With aliases (after shell integration):**
```bash
dk status              # Quick status
dk-analyze ./src       # Quick analysis  
dk-generate "prompt"   # Quick generation
```

## 📚 Next Steps

- Read the [full README](README.md) for detailed usage
- Check out [examples and tutorials](examples/)
- Join the [community discussions](https://github.com/infer-no-dev/devkit/discussions)

---

**Need help?** Open an issue on [GitHub](https://github.com/infer-no-dev/devkit/issues) 🆘