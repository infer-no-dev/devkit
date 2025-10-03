# Ollama Integration for Agentic Development Environment

This document describes the implementation of Ollama Local LLM integration as the primary AI provider for the Agentic Development Environment.

## Overview

The integration enables the use of local Large Language Models (LLMs) via Ollama for:

- ✅ **AI-powered code generation** from natural language prompts
- ✅ **Intelligent code completion and suggestions**
- ✅ **Context-aware code analysis and refactoring**
- ✅ **Multi-provider AI architecture** (supports Ollama, OpenAI, Anthropic)
- ✅ **Streaming and non-streaming completions**
- ✅ **Configurable model parameters** (temperature, tokens, etc.)
- ✅ **Health monitoring and model management**

## Architecture

### Core Components

1. **AI Module** (`src/ai/`)
   - **Abstract AI Client trait** - Unified interface for different providers
   - **Ollama Client** - HTTP client for Ollama API integration
   - **AI Manager** - Coordinates multiple AI providers with caching
   - **Model Management** - Handles model discovery and configuration

2. **Configuration System** (`src/config/`)
   - **Ollama-specific settings** - Endpoint, timeout, retry configuration
   - **Multi-provider support** - OpenAI, Anthropic future integration
   - **Hierarchical configuration** - Project and global settings

3. **Code Generation Integration** (`src/codegen/`)
   - **AI-enhanced generation** - Uses Ollama for intelligent code creation
   - **Fallback mechanisms** - Template-based generation when AI unavailable
   - **Context-aware prompts** - Incorporates codebase context

### Key Features

#### 🔧 **Flexible Provider Architecture**

```rust
// AI providers are abstracted through the AIClient trait
#[async_trait]
pub trait AIClient {
    async fn list_models(&self) -> Result<Vec<ModelInfo>, AIError>;
    async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse, AIError>;
    async fn chat_completion_stream(&self, request: ChatRequest) -> Result<Receiver<ChatStreamChunk>, AIError>;
    async fn health_check(&self) -> Result<bool, AIError>;
}
```

#### 🦙 **Ollama Client Implementation**

```rust
// Ollama client with configurable settings
let ollama_config = OllamaConfig {
    endpoint: "http://localhost:11434".to_string(),
    timeout_seconds: 300,
    max_retries: 3,
    default_model: Some("llama3.2".to_string()),
};

let client = OllamaClient::new(ollama_config);
```

#### 🤖 **AI Manager for Provider Coordination**

```rust
// Initialize AI manager with Ollama as primary provider
let ai_manager = AIManager::new(ai_model_config).await?;

// Check health across all providers
let health = ai_manager.health_check_all().await;

// List available models
let models = ai_manager.list_all_models().await?;

// Generate responses with automatic provider selection
let response = ai_manager.chat_completion_default(request).await?;
```

#### 📝 **Context-Aware Code Generation**

```rust
// AI-enhanced code generation with codebase context
let result = code_generator.generate_with_ai(
    prompt,
    language,
    &codebase_context,  // Includes file context, symbols, imports
    &generation_config
).await?;
```

## Configuration

### Default Configuration

The system defaults to using Ollama as the primary AI provider:

```toml
# .agentic-config.toml
[codegen.ai_model_settings]
default_provider = "ollama"
default_model = "llama3.2"
context_window_size = 8192
temperature = 0.7
max_tokens = 1000

[codegen.ai_model_settings.ollama]
endpoint = "http://localhost:11434"
timeout_seconds = 300
max_retries = 3
default_model = "llama3.2"
```

### Supported Models

The integration automatically detects and configures common Ollama models:

| Model Family | Context Window | Capabilities |
|--------------|----------------|--------------|
| Llama 3.2 | 8,192 tokens | Code generation, analysis, conversation |
| Llama 2 | 4,096 tokens | Code generation, conversation |
| Code Llama | 16,384 tokens | Code generation, analysis, completion |
| Mistral/Mixtral | 32,768 tokens | Code generation, analysis, reasoning |
| Qwen | 32,768 tokens | Code generation, analysis, multilingual |
| Phi | 2,048 tokens | Code generation, lightweight inference |

## Usage Examples

### Basic Code Generation

```rust
use devkit_env::ai::AIManager;
use devkit_env::codegen::{CodeGenerator, GenerationRequest};

// Initialize with Ollama
let ai_manager = AIManager::from_config(&config).await?;
let mut code_generator = CodeGenerator::new()?;
code_generator.set_ai_manager(ai_manager);

// Generate code from natural language
let request = GenerationRequest {
    prompt: "Create a function that calculates Fibonacci numbers".to_string(),
    config: GenerationConfig {
        target_language: Some("rust".to_string()),
        use_ai: true,
        temperature: Some(0.3),
        max_tokens: Some(500),
        ..Default::default()
    },
    ..Default::default()
};

let result = code_generator.generate_from_prompt(request).await?;
println!("Generated code: {}", result.generated_code);
```

### Direct AI Chat

```rust
use devkit_env::ai::{ChatMessage, ChatRequest, ModelParameters};

let messages = vec![
    ChatMessage::system("You are a helpful programming assistant."),
    ChatMessage::user("Explain Rust ownership in simple terms."),
];

let request = ChatRequest {
    model: String::new(), // Use default
    messages,
    parameters: Some(ModelParameters {
        temperature: Some(0.7),
        max_tokens: Some(300),
        ..Default::default()
    }),
    stream: false,
};

let response = ai_manager.chat_completion_default(request).await?;
println!("AI Response: {}", response.message.content);
```

### Streaming Responses

```rust
let mut stream = ai_manager.chat_completion_stream_default(request).await?;

while let Some(chunk_result) = stream.recv().await {
    match chunk_result {
        Ok(chunk) => {
            print!("{}", chunk.delta);
            if chunk.finish_reason.is_some() {
                break;
            }
        }
        Err(e) => eprintln!("Stream error: {}", e),
    }
}
```

## Installation and Setup

### Prerequisites

1. **Install Ollama**
   ```bash
   # macOS
   curl -fsSL https://ollama.com/install.sh | sh
   
   # Linux
   curl -fsSL https://ollama.com/install.sh | sh
   
   # Windows: Download from https://ollama.com/download
   ```

2. **Start Ollama Service**
   ```bash
   ollama serve
   ```

3. **Install Models**
   ```bash
   # Install Llama 3.2 (recommended)
   ollama pull llama3.2
   
   # Install Code Llama for enhanced coding
   ollama pull codellama
   
   # Install Mistral for larger context
   ollama pull mistral
   ```

4. **Verify Installation**
   ```bash
   ollama list
   ollama run llama3.2 "Hello, world!"
   ```

### Project Integration

1. **Add to Cargo.toml** (dependencies already included):
   ```toml
   [dependencies]
   reqwest = { version = "0.11", features = ["json", "stream"] }
   tokio = { version = "1.0", features = ["full"] }
   tokio-stream = "0.1"
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   thiserror = "1.0"
   async-trait = "0.1"
   ```

2. **Initialize AI Manager**
   ```rust
   use devkit_env::ai::AIManager;
   use devkit_env::config::Config;

   let config = Config::default();
   let ai_manager = AIManager::from_config(&config).await?;
   ```

3. **Run Example**
   ```bash
   cargo run --example ollama_integration
   ```

## Performance Considerations

### Model Selection

- **Small tasks**: Use `llama3.2` (8B parameters) for fast responses
- **Code generation**: Use `codellama` (7B-34B) for specialized code tasks  
- **Complex reasoning**: Use `mistral` or `mixtral` for advanced analysis
- **Resource constrained**: Use `phi` (3B parameters) for lightweight inference

### Optimization Tips

1. **Adjust temperature** based on task:
   - Code generation: 0.2-0.4 (more deterministic)
   - Creative tasks: 0.7-0.9 (more creative)
   - Factual queries: 0.1-0.3 (more focused)

2. **Limit context window usage**:
   - Include only relevant codebase context
   - Truncate long files to essential parts
   - Use context_depth setting to control inclusion

3. **Cache responses** for repeated queries:
   - AI Manager includes built-in model info caching
   - Consider implementing response caching for frequent patterns

4. **Use streaming** for long responses:
   - Provides immediate feedback to users
   - Allows early termination of poor responses
   - Better user experience for code generation

## Troubleshooting

### Common Issues

1. **"Connection refused" errors**
   ```bash
   # Ensure Ollama is running
   ollama serve
   
   # Check if service is accessible
   curl http://localhost:11434/api/tags
   ```

2. **"Model not found" errors**
   ```bash
   # List installed models
   ollama list
   
   # Pull required model
   ollama pull llama3.2
   ```

3. **Slow responses**
   - Reduce `max_tokens` in requests
   - Use smaller models for simpler tasks
   - Check system resources (RAM, GPU if available)

4. **Empty or poor quality responses**
   - Adjust temperature (try 0.3-0.7 range)
   - Improve prompt clarity and specificity
   - Ensure model has sufficient context

### Debugging

Enable debug logging to troubleshoot issues:

```bash
RUST_LOG=debug cargo run --example ollama_integration
```

Check AI Manager health status:

```rust
let health = ai_manager.health_check_all().await;
for (provider, status) in health {
    println!("Provider {:?}: {}", provider, 
        if status { "Healthy" } else { "Unhealthy" });
}
```

## Development and Extension

### Adding New Providers

To add support for additional AI providers:

1. **Implement the AIClient trait**:
   ```rust
   pub struct CustomProviderClient {
       config: CustomConfig,
       client: HttpClient,
   }

   #[async_trait]
   impl AIClient for CustomProviderClient {
       // Implement required methods...
   }
   ```

2. **Add provider configuration**:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct CustomConfig {
       pub api_endpoint: String,
       pub api_key: Option<String>,
       // ... other settings
   }
   ```

3. **Register in AIManager**:
   ```rust
   // Add to AIProvider enum
   pub enum AIProvider {
       Ollama,
       OpenAI,
       Anthropic,
       Custom(String),
   }
   ```

### Contributing

The AI integration is designed to be extensible. Areas for contribution:

- **Additional model providers** (OpenAI, Anthropic, local inference engines)
- **Advanced context management** (smarter context selection, RAG integration)
- **Model fine-tuning support** (LoRA adapters, custom model training)
- **Performance optimizations** (response caching, batching)
- **UI enhancements** (streaming progress, model switching)

## Security Considerations

### Local Privacy

Ollama integration provides significant privacy benefits:

- ✅ **All data stays local** - No external API calls for model inference
- ✅ **No usage tracking** - Unlike cloud providers, no request logging
- ✅ **Customizable models** - Can use domain-specific or privacy-focused models
- ✅ **Air-gapped operation** - Works completely offline after initial setup

### Best Practices

1. **Model Selection**: Choose appropriate models for sensitivity levels
2. **Context Filtering**: Avoid including sensitive data in prompts
3. **Access Control**: Secure Ollama endpoint if exposed to network
4. **Model Updates**: Keep models updated for security and performance

## Conclusion

The Ollama integration provides a robust, privacy-focused AI foundation for the Agentic Development Environment. It combines the power of modern Large Language Models with local control and customization, making it ideal for enterprise and privacy-conscious development workflows.

The architecture is designed for extensibility, allowing easy addition of new providers and capabilities as the AI landscape evolves. The integration is production-ready and provides a solid foundation for AI-enhanced development workflows.

---

**Status**: ✅ **Complete** - Core integration implemented and tested
**Next Steps**: Advanced context management, agent AI capabilities, model management features
**Documentation**: [API Reference](api-reference.md) | [Configuration Guide](configuration.md) | [Examples](../examples/)
