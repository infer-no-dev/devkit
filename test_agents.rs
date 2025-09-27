//! Test program for AI-integrated agents
//!
//! This program demonstrates the AI-powered agent functionality

use devkit::agents::{AgentSystem, AgentTask, TaskPriority};
use devkit::ai::AIManager;
use devkit::config::{Config, ConfigManager};
use serde_json::json;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🤖 Testing AI-Integrated Agents");
    println!("============================");

    // Load configuration
    let mut config_manager = ConfigManager::new(None)?;
    config_manager.load()?;

    // Initialize AI manager
    let ai_manager = match AIManager::from_config(config_manager.config()).await {
        Ok(manager) => {
            println!("✅ AI Manager initialized successfully");
            Some(Arc::new(manager))
        }
        Err(e) => {
            println!("⚠️ AI Manager failed to initialize: {}", e);
            println!("   Agents will use fallback behavior");
            None
        }
    };

    // Create agent system with AI manager if available
    let agent_system = match &ai_manager {
        Some(ai_mgr) => Arc::new(AgentSystem::with_ai_manager(ai_mgr.clone())),
        None => Arc::new(AgentSystem::new()),
    };

    // Initialize agents
    agent_system.initialize().await;
    println!("✅ Agent system initialized");

    // Test 1: Code Analysis Agent
    println!("\n🔍 Test 1: Code Analysis Agent");
    println!("------------------------------");
    let analysis_task = AgentTask {
        id: "test_analysis".to_string(),
        task_type: "analyze_code".to_string(),
        description: "Analyze this Rust function for potential improvements:\n\nfn fibonacci(n: u32) -> u32 {\n    if n <= 1 { n } else { fibonacci(n-1) + fibonacci(n-2) }\n}".to_string(),
        context: json!({
            "language": "rust",
            "function_name": "fibonacci",
            "analysis_focus": ["performance", "readability", "correctness"]
        }),
        priority: TaskPriority::High,
    };

    match agent_system.submit_task(analysis_task).await {
        Ok(result) => {
            println!("✅ Analysis completed!");
            println!("Result: {}", result.output);
            if !result.artifacts.is_empty() {
                println!("Generated {} artifacts", result.artifacts.len());
            }
        }
        Err(e) => {
            println!("❌ Analysis failed: {}", e);
        }
    }

    // Test 2: Code Generation Agent
    println!("\n🛠️ Test 2: Code Generation Agent");
    println!("--------------------------------");
    let generation_task = AgentTask {
        id: "test_generation".to_string(),
        task_type: "generate_code".to_string(),
        description:
            "Create a simple HTTP client in Rust that makes GET requests with error handling"
                .to_string(),
        context: json!({
            "language": "rust",
            "features": ["error_handling", "async"],
            "dependencies": ["tokio", "reqwest"]
        }),
        priority: TaskPriority::Normal,
    };

    match agent_system.submit_task(generation_task).await {
        Ok(result) => {
            println!("✅ Code generation completed!");
            println!("Result: {}", result.output);
            if !result.artifacts.is_empty() {
                println!("Generated code artifacts:");
                for artifact in &result.artifacts {
                    if artifact.artifact_type == "code" {
                        println!("\n--- Generated Code ---");
                        println!("{}", artifact.content);
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Code generation failed: {}", e);
        }
    }

    // Test 3: Debugging Agent
    println!("\n🐛 Test 3: Debugging Agent");
    println!("-------------------------");
    let debug_task = AgentTask {
        id: "test_debugging".to_string(),
        task_type: "debug_code".to_string(),
        description: "Help debug this Rust code that's causing a panic:\n\nfn main() {\n    let numbers = vec![1, 2, 3];\n    println!(\"{}\", numbers[5]);\n}".to_string(),
        context: json!({
            "error_type": "index_out_of_bounds",
            "language": "rust",
            "context": "accessing vector element"
        }),
        priority: TaskPriority::High,
    };

    match agent_system.submit_task(debug_task).await {
        Ok(result) => {
            println!("✅ Debugging analysis completed!");
            println!("Result: {}", result.output);
        }
        Err(e) => {
            println!("❌ Debugging failed: {}", e);
        }
    }

    // Test 4: Test Generation Agent
    println!("\n🧪 Test 4: Test Generation Agent");
    println!("-------------------------------");
    let test_task = AgentTask {
        id: "test_test_generation".to_string(),
        task_type: "generate_tests".to_string(),
        description: "Generate comprehensive tests for a string validation function that checks if an email is valid".to_string(),
        context: json!({
            "language": "rust",
            "function_signature": "fn is_valid_email(email: &str) -> bool",
            "test_cases": ["valid emails", "invalid emails", "edge cases"]
        }),
        priority: TaskPriority::Normal,
    };

    match agent_system.submit_task(test_task).await {
        Ok(result) => {
            println!("✅ Test generation completed!");
            println!("Result: {}", result.output);
            if !result.artifacts.is_empty() {
                for artifact in &result.artifacts {
                    if artifact.artifact_type == "test_code" {
                        println!("\n--- Generated Test Code ---");
                        println!("{}", artifact.content);
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Test generation failed: {}", e);
        }
    }

    println!("\n🎉 Agent testing completed!");
    Ok(())
}
