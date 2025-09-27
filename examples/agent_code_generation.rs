//! Agent-based Code Generation Example
//!
//! This example demonstrates the full agent system workflow:
//! 1. Initialize AI manager with Ollama
//! 2. Create and configure code generation agent
//! 3. Submit various code generation tasks
//! 4. Process results and display generated code

use agentic_dev_env::agents::{
    agent_types::CodeGenerationAgent, AgentSystem, AgentTask, TaskPriority,
};
use agentic_dev_env::ai::AIManager;
use agentic_dev_env::config::{AIModelConfig, OllamaConfig};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🤖 Agent-based Code Generation Example");
    println!("=====================================\n");

    // Step 1: Initialize AI Manager
    println!("🔧 Initializing AI Manager...");
    let ai_config = AIModelConfig {
        default_provider: "ollama".to_string(),
        default_model: "llama3.2".to_string(),
        ollama: OllamaConfig {
            endpoint: "http://localhost:11434".to_string(),
            timeout_seconds: 60,
            max_retries: 3,
            default_model: Some("llama3.2".to_string()),
        },
        openai: None,
        anthropic: None,
        context_window_size: 8192,
        temperature: 0.7,
        max_tokens: 2000,
    };

    let ai_manager = Arc::new(AIManager::new(ai_config).await?);
    println!("✅ AI Manager initialized\n");

    // Step 2: Health check
    println!("🏥 Checking AI service health...");
    let health_results = ai_manager.health_check_all().await;
    for (provider, is_healthy) in health_results {
        let status = if is_healthy {
            "✅ Healthy"
        } else {
            "❌ Unavailable"
        };
        println!("Provider {:?}: {}", provider, status);
    }
    println!();

    // Step 3: Initialize Agent System
    println!("🎯 Initializing Agent System...");
    let agent_system = AgentSystem::new();

    // Create and register code generation agent with AI manager
    let mut code_agent = CodeGenerationAgent::with_ai_manager(ai_manager.clone());
    agent_system.register_agent(Box::new(code_agent)).await;

    println!("✅ Agent system initialized with code generation agent\n");

    // Step 4: Test various code generation tasks
    let test_cases = vec![
        (
            "Generate a Rust function for calculating factorial",
            "generate_function",
            json!({
                "language": "rust",
                "requirements": [
                    "Handle edge cases for 0 and 1",
                    "Use proper error handling",
                    "Include documentation"
                ]
            }),
        ),
        (
            "Create a Python class for managing a simple database connection",
            "generate_struct",
            json!({
                "language": "python",
                "requirements": [
                    "Include connection pooling",
                    "Add proper error handling",
                    "Support context manager protocol"
                ]
            }),
        ),
        (
            "Generate a JavaScript function for debouncing user input",
            "generate_function",
            json!({
                "language": "javascript",
                "file_path": "src/utils/debounce.js",
                "requirements": [
                    "Support configurable delay",
                    "Handle multiple calls correctly",
                    "Return a promise"
                ]
            }),
        ),
        (
            "Refactor this Rust code to use better error handling",
            "refactor_code",
            json!({
                "language": "rust",
                "existing_code": "fn divide(a: i32, b: i32) -> i32 {\n    a / b\n}",
                "requirements": [
                    "Handle division by zero",
                    "Use Result type",
                    "Add proper documentation"
                ]
            }),
        ),
        (
            "Create a TypeScript interface for a user profile",
            "generate_struct",
            json!({
                "language": "typescript",
                "file_path": "types/user.ts",
                "requirements": [
                    "Include optional fields",
                    "Add validation constraints",
                    "Support nested objects"
                ]
            }),
        ),
    ];

    println!(
        "🚀 Running {} code generation test cases...\n",
        test_cases.len()
    );

    for (index, (description, task_type, context)) in test_cases.into_iter().enumerate() {
        println!("📋 Test Case {}: {}", index + 1, description);
        println!("{}", "=".repeat(60));

        // Create agent task
        let task = AgentTask {
            id: Uuid::new_v4().to_string(),
            task_type: task_type.to_string(),
            description: description.to_string(),
            context,
            priority: TaskPriority::Normal,
        };

        // Submit task to agent system
        let start_time = std::time::Instant::now();
        match agent_system.submit_task(task).await {
            Ok(result) => {
                let elapsed = start_time.elapsed();

                println!("✅ Task completed successfully!");
                println!("⏱️  Processing time: {:.2}s", elapsed.as_secs_f64());
                println!("📄 Output: {}", result.output);

                // Display generated artifacts
                for artifact in &result.artifacts {
                    println!(
                        "\n📁 Artifact: {} (type: {})",
                        artifact.name, artifact.artifact_type
                    );

                    // Extract metadata for display
                    if let Ok(metadata) =
                        serde_json::from_str::<serde_json::Value>(&artifact.metadata.to_string())
                    {
                        if let Some(language) = metadata.get("language") {
                            println!("🔤 Language: {}", language.as_str().unwrap_or("unknown"));
                        }
                        if let Some(confidence) = metadata.get("confidence") {
                            println!(
                                "📊 Confidence: {:.1}%",
                                confidence.as_f64().unwrap_or(0.0) * 100.0
                            );
                        }
                        if let Some(generation_time) = metadata.get("generation_time_ms") {
                            println!(
                                "⚡ AI Generation time: {}ms",
                                generation_time.as_i64().unwrap_or(0)
                            );
                        }
                    }

                    println!("\n🎯 Generated Code:");
                    println!("{}", "─".repeat(50));
                    println!("{}", artifact.content);
                    println!("{}", "─".repeat(50));
                }

                // Display suggested next actions
                if !result.next_actions.is_empty() {
                    println!("\n💡 Suggested next actions:");
                    for action in &result.next_actions {
                        println!("  • {}", action);
                    }
                }
            }
            Err(e) => {
                println!("❌ Task failed: {}", e);
            }
        }

        println!("\n{}\n", "═".repeat(80));
    }

    // Step 5: Display agent system status
    println!("📊 Agent System Status");
    println!("=====================");

    let agent_statuses = agent_system.get_agent_statuses().await;
    for (agent_name, status) in agent_statuses {
        println!("🤖 {}: {:?}", agent_name, status);
    }

    println!("\n🎉 All tests completed!");
    println!("\n💡 Tips for using the agent system:");
    println!("   • Provide clear, specific task descriptions");
    println!("   • Include context and requirements in task.context");
    println!("   • Use appropriate task types (generate_function, generate_struct, etc.)");
    println!("   • Specify target language and file path when known");
    println!("   • Review generated code and run suggested next actions");

    Ok(())
}
