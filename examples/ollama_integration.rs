//! Example demonstrating Ollama integration for code generation
//!
//! This example shows how to set up and use the Ollama Local LLM integration
//! for AI-powered code generation in the agentic development environment.

use devkit_env::ai::AIManager;
use devkit_env::codegen::{CodeGenerator, GenerationConfig, GenerationRequest};
use devkit_env::config::{AIModelConfig, OllamaConfig};
use devkit_env::context::CodebaseContext;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦙 Ollama Integration Example");
    println!("============================\n");

    // Create Ollama configuration
    let ollama_config = OllamaConfig {
        endpoint: "http://localhost:11434".to_string(),
        timeout_seconds: 300,
        max_retries: 3,
        default_model: Some("llama3.2".to_string()),
    };

    // Create AI model configuration with Ollama as primary provider
    let ai_config = AIModelConfig {
        default_provider: "ollama".to_string(),
        default_model: "llama3.2".to_string(),
        ollama: ollama_config,
        openai: None,
        anthropic: None,
        context_window_size: 8192,
        temperature: 0.3,
        max_tokens: 2000,
    };

    // Initialize AI manager
    println!("🔧 Initializing AI Manager...");
    let ai_manager = match AIManager::new(ai_config).await {
        Ok(manager) => {
            println!("✅ AI Manager initialized successfully");
            manager
        }
        Err(e) => {
            println!("❌ Failed to initialize AI Manager: {}", e);
            return Err(Box::new(e) as Box<dyn std::error::Error>);
        }
    };

    // Check Ollama health
    println!("\n🏥 Checking Ollama health...");
    let health_results = ai_manager.health_check_all().await;
    for (provider, is_healthy) in health_results {
        let status = if is_healthy {
            "✅ Healthy"
        } else {
            "❌ Unhealthy"
        };
        println!("Provider {:?}: {}", provider, status);
    }

    // List available models
    println!("\n📋 Listing available models...");
    match ai_manager.list_all_models().await {
        Ok(models) => {
            if models.is_empty() {
                println!("⚠️  No models found. Please ensure Ollama is running and has models installed.");
                println!("   You can install a model with: ollama pull llama3.2");
            } else {
                println!("Found {} model(s):", models.len());
                for model in models.iter().take(3) {
                    // Show first 3 models
                    println!("  • {}", model.name);
                    if let Some(desc) = &model.description {
                        println!("    {}", desc);
                    }
                    println!("    Context window: {} tokens", model.context_window);
                    println!("    Capabilities: {:?}\n", model.capabilities);
                }
            }
        }
        Err(e) => {
            println!("⚠️  Could not list models: {}", e);
            println!("   This might be because Ollama is not running or accessible.");
        }
    }

    // Set up code generator with AI integration
    println!("🔧 Setting up Code Generator with AI integration...");
    let mut code_generator =
        CodeGenerator::new().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    code_generator.set_ai_manager(Arc::new(ai_manager));
    println!(
        "✅ Code Generator initialized with AI support: {}",
        code_generator.has_ai()
    );

    // Example 1: Generate a simple Rust function
    println!("\n🚀 Example 1: Generate a Rust function");
    println!("=====================================\n");

    let rust_prompt = "Create a function that calculates the factorial of a number using recursion";
    let generation_config = GenerationConfig {
        target_language: Some("rust".to_string()),
        temperature: Some(0.3),
        max_tokens: Some(500),
        use_ai: true,
        ..GenerationConfig::default()
    };

    let context = CodebaseContext {
        files: Vec::new(),
        dependencies: Vec::new(),
        root_path: std::env::current_dir().unwrap_or_default(),
        metadata: Default::default(),
        repository_info: Default::default(),
        symbols: Default::default(),
    };

    let request = GenerationRequest {
        prompt: rust_prompt.to_string(),
        file_path: Some("factorial.rs".to_string()),
        context: context.clone(),
        config: generation_config.clone(),
        constraints: vec!["Use proper error handling".to_string()],
    };

    match code_generator.generate_from_prompt(request).await {
        Ok(result) => {
            println!("✅ Code generated successfully!");
            println!("Language: {}", result.language);
            println!("Confidence: {:.2}", result.confidence_score);
            println!("Generation time: {}ms", result.metadata.generation_time_ms);
            println!("Tokens used: {}", result.metadata.tokens_used);
            println!("\nGenerated code:");
            println!("{}", "=".repeat(50));
            println!("{}", result.generated_code);
            println!("{}", "=".repeat(50));

            if !result.suggestions.is_empty() {
                println!("\n💡 Suggestions:");
                for suggestion in result.suggestions {
                    println!("  • {}", suggestion);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to generate code: {}", e);
        }
    }

    println!("\n🎉 Example completed!");
    println!("\n📝 Note: To use this example:");
    println!("   1. Ensure Ollama is installed and running (ollama serve)");
    println!("   2. Install a model (e.g., ollama pull llama3.2)");
    println!("   3. Run this example: cargo run --example ollama_integration");

    Ok(())
}
