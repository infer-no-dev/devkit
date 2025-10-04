use devkit::{
    agents::{AgentSystem, AgentTask, TaskPriority},
    ai::AIManager,
    codegen::{CodeGenerator, GenerationRequest},
    config::ConfigManager,
    context::{AnalysisConfig, ContextManager},
    error::DevKitError,
    logging::LogLevel,
};
use std::collections::HashMap;
use tokio;

/// Basic example demonstrating core DevKit functionality
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 DevKit Basic Usage Example");
    println!("===============================");

    // 1. Initialize Configuration
    println!("\n📋 1. Setting up configuration...");
    let config_manager = ConfigManager::new(None)?;
    let config = config_manager.config().clone();

    println!("✅ Configuration loaded");
    println!("   📝 Log Level: {:?}", config.general.log_level);
    println!("   🏠 Environment: {}", config.general.environment);

    // 2. Initialize AI Manager
    println!("\n🤖 2. Initializing AI Manager...");
    let ai_manager = match AIManager::new(config.clone()).await {
        Ok(manager) => {
            println!("✅ AI Manager initialized successfully");
            manager
        }
        Err(e) => {
            println!("⚠️  AI Manager initialization failed: {}", e);
            println!("   This is expected if no AI providers are configured");
            return demonstrate_without_ai().await;
        }
    };

    // 3. Initialize Context Manager
    println!("\n📁 3. Setting up context analysis...");
    let context_manager = ContextManager::new()?;
    println!("✅ Context Manager ready");

    // 4. Initialize Code Generator
    println!("\n⚙️  4. Setting up code generation...");
    let code_generator = CodeGenerator::new()?;
    println!("✅ Code Generator ready");

    // 5. Initialize Agent System
    println!("\n🔸️  5. Starting agent system...");
    let agent_system = AgentSystem::new();
    agent_system.start().await?;
    println!("✅ Agent System running");

    // 6. Demonstrate Context Analysis
    println!("\n🔍 6. Analyzing codebase context...");
    let current_dir = std::env::current_dir()?;
    let analysis_config = AnalysisConfig::default();

    match context_manager
        .analyze_codebase(current_dir.clone(), analysis_config)
        .await
    {
        Ok(context) => {
            println!("✅ Context analysis complete:");
            println!("   📄 Files analyzed: {}", context.files.len());
            println!(
                "   🔗 Symbols indexed: {}",
                context.metadata.indexed_symbols
            );
            println!("   📊 Total lines: {}", context.metadata.total_lines);

            // Show some sample files
            if !context.files.is_empty() {
                println!("   📋 Sample files:");
                for (i, file) in context.files.iter().enumerate().take(3) {
                    println!("      - {}", file.path.display());
                    if i == 2 && context.files.len() > 3 {
                        println!("      ... and {} more", context.files.len() - 3);
                        break;
                    }
                }
            }
        }
        Err(e) => {
            println!("⚠️  Context analysis failed: {}", e);
        }
    }

    // 7. Demonstrate Code Generation
    println!("\n💻 7. Generating code...");
    let generation_request = GenerationRequest {
        prompt: "Create a simple Rust function that calculates the factorial of a number"
            .to_string(),
        context: None,
        file_path: None,
        constraints: Vec::new(),
    };

    match code_generator.generate_code(&generation_request).await {
        Ok(result) => {
            println!("✅ Code generated successfully:");
            println!("   🔧 Generated code:");
            println!("   {}", "-".repeat(40));
            // Show first few lines
            let lines: Vec<&str> = result.lines().collect();
            for line in lines.iter().take(10) {
                println!("   {}", line);
            }
            if lines.len() > 10 {
                println!("   ... ({} more lines)", lines.len() - 10);
            }
            println!("   {}", "-".repeat(40));
        }
        Err(e) => {
            println!("⚠️  Code generation failed: {}", e);
        }
    }

    // 8. Demonstrate Agent System
    println!("\n👥 8. Using agent system...");

    // Create a code generation task
    let mut task_metadata = HashMap::new();
    task_metadata.insert("language".to_string(), serde_json::json!("rust"));
    task_metadata.insert("complexity".to_string(), serde_json::json!("simple"));

    let agent_task = AgentTask {
        id: "example_task_001".to_string(),
        task_type: "code_generation".to_string(),
        description: "Generate a HashMap utility function".to_string(),
        context: serde_json::json!({
            "prompt": "Create a utility function that merges two HashMaps",
            "language": "rust",
            "style": "functional"
        }),
        priority: TaskPriority::Normal,
        deadline: None,
        metadata: task_metadata,
    };

    match agent_system.submit_task(agent_task).await {
        Ok(result) => {
            println!("✅ Agent task completed:");
            println!("   🎯 Task ID: {}", result.task_id);
            println!("   🤖 Agent: {}", result.agent_id);
            println!(
                "   ⏱️  Processing time: {}ms",
                result.processing_duration_ms
            );
            println!(
                "   📝 Output: {}",
                result.output.chars().take(100).collect::<String>()
            );
            if result.output.len() > 100 {
                println!("      ... (truncated)");
            }
        }
        Err(e) => {
            println!("⚠️  Agent task failed: {}", e);
        }
    }

    // 9. Show System Status
    println!("\n📊 9. System status:");
    let agent_statuses = agent_system.get_agent_statuses().await;
    println!("   🤖 Active agents: {}", agent_statuses.len());

    for (agent_name, status) in agent_statuses.iter().take(5) {
        println!("      - {}: {:?}", agent_name, status);
    }

    // 10. Cleanup
    println!("\n🔄 10. Shutting down systems...");
    agent_system.stop().await?;
    println!("✅ All systems shut down cleanly");

    println!("\n🎉 DevKit example completed successfully!");
    println!("   For more examples, check the examples/ directory");
    println!("   Visit https://github.com/infer-no-dev/devkit for documentation");

    Ok(())
}

/// Fallback demonstration when AI is not available
async fn demonstrate_without_ai() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔧 Running limited demo without AI providers...");

    // Initialize minimal configuration
    let config_manager = ConfigManager::new(None)?;
    let config = config_manager.config();

    // Context analysis still works without AI
    println!("\n📁 Demonstrating context analysis...");
    let context_manager = ContextManager::new()?;
    let current_dir = std::env::current_dir()?;
    let analysis_config = AnalysisConfig::default();

    match context_manager
        .analyze_codebase(current_dir, analysis_config)
        .await
    {
        Ok(context) => {
            println!("✅ Context analysis complete:");
            println!("   📄 Files found: {}", context.files.len());
            println!("   📊 Total size: {} bytes", context.metadata.total_size);
        }
        Err(e) => {
            println!("⚠️  Context analysis failed: {}", e);
        }
    }

    // Configuration demonstration
    println!("\n⚙️  Configuration capabilities:");
    println!("   🏠 Environment: {}", config.general.environment);
    println!("   📝 Log Level: {:?}", config.general.log_level);
    println!("   🔧 Max agents: {}", config.agents.max_concurrent);

    println!("\n💡 To enable full functionality:");
    println!("   1. Configure an AI provider (Ollama, OpenAI, or Anthropic)");
    println!("   2. Set API keys in environment variables or config files");
    println!("   3. Run this example again");

    Ok(())
}

/// Helper function to demonstrate error handling
fn handle_devkit_error(error: DevKitError) {
    match error {
        DevKitError::Context(e) => {
            println!("📁 Context Error: {}", e);
        }
        DevKitError::Agent(e) => {
            println!("👥 Agent System Error: {}", e);
        }
        DevKitError::Shell(e) => {
            println!("💻 Shell Error: {}", e);
        }
        DevKitError::IO(e) => {
            println!("💾 IO Error: {}", e);
        }
    }
}
