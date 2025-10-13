//! Integration tests for core DevKit functionality
//!
//! These tests verify that the basic components can work together
//! without requiring external AI services.

use crate::agents::{Agent, AgentTask, TaskPriority};
use crate::agents::agent_types::{CodeGenerationAgent, AnalysisAgent};
use crate::codegen::stubs::{generate_code_stub, infer_language_from_context, suggest_filename};
use serde_json::json;
use std::time::Duration;
use std::collections::HashMap;

/// Test that basic stub-based code generation works
#[tokio::test]
async fn test_stub_code_generation() {
    // Test the stub generation functions directly
    let rust_code = generate_code_stub("create a hello world function", Some("rust"));
    assert!(rust_code.contains("fn"));
    assert!(rust_code.contains("Hello"));
    
    // Test language inference
    let language = infer_language_from_context("create a rust function", None);
    assert_eq!(language, Some("rust".to_string()));
    
    let language2 = infer_language_from_context("write python code", None);
    assert_eq!(language2, Some("python".to_string()));
}

/// Test that agents can be created and can handle basic tasks
#[tokio::test]
async fn test_agent_creation_and_basic_functionality() {
    // Create a code generation agent
    let mut code_agent = CodeGenerationAgent::new();
    
    // Verify basic agent properties
    assert_eq!(code_agent.name(), "CodeGenerationAgent");
    assert!(code_agent.can_handle("generate_function"));
    assert!(code_agent.can_handle("generate_class"));
    assert!(!code_agent.can_handle("unknown_task"));
    
    // Create a basic task
    let task = AgentTask {
        id: "test_task_001".to_string(),
        task_type: "generate_function".to_string(),
        description: "Create a simple hello world function in Rust".to_string(),
        context: json!({
            "language": "rust",
            "requirements": ["should print hello world", "should be public"]
        }),
        priority: TaskPriority::Normal,
        deadline: None,
        metadata: HashMap::new(),
    };
    
    // Process the task
    let result = code_agent.process_task(task).await;
    
    // Verify the result
    assert!(result.is_ok());
    let agent_result = result.unwrap();
    assert!(agent_result.success);
    assert!(!agent_result.artifacts.is_empty());
    
    // Check that an artifact was created
    let artifact = &agent_result.artifacts[0];
    assert_eq!(artifact.artifact_type, "source_code");
    assert!(artifact.content.contains("fn"));
}

/// Test that analysis agents can be created
#[tokio::test]
async fn test_analysis_agent_creation() {
    let mut analysis_agent = AnalysisAgent::new();
    
    // Verify basic properties
    assert_eq!(analysis_agent.name(), "AnalysisAgent");
    assert!(analysis_agent.can_handle("analyze_code"));
    assert!(analysis_agent.can_handle("check_quality"));
    assert!(!analysis_agent.can_handle("generate_code"));
    
    // Test that it can process an analysis task
    let task = AgentTask {
        id: "analysis_task_001".to_string(),
        task_type: "analyze_code".to_string(),
        description: "Analyze the quality of this Rust function".to_string(),
        context: json!({
            "code": "fn hello() { println!(\"Hello, world!\"); }",
            "language": "rust"
        }),
        priority: TaskPriority::Normal,
        deadline: None,
        metadata: HashMap::new(),
    };
    
    let result = analysis_agent.process_task(task).await;
    assert!(result.is_ok());
    
    let agent_result = result.unwrap();
    assert!(agent_result.success);
}

/// Test that the codegen module can generate suggestions
#[tokio::test]
async fn test_codegen_suggestions() {
    // Test filename suggestions using the available suggest_filename function
    
    // Test filename suggestions
    let filename = suggest_filename("create a user authentication module", Some("rust"));
    assert!(filename.contains("auth") || filename.contains("user"));
    assert!(filename.ends_with(".rs"));
    
    let filename2 = suggest_filename("write a config parser", Some("rust"));
    assert!(filename2.ends_with(".rs"));
}

/// Test basic configuration and system setup
#[tokio::test]
async fn test_basic_system_functionality() {
    use crate::config::{Config, ConfigManager};
    
    // Test that we can create a basic config
    let config = Config::default();
    assert!(!config.codegen.ai_model_settings.default_model.is_empty());
    
    // Test that we can create a config manager
    let config_manager = ConfigManager::new(None);
    assert!(config_manager.is_ok());
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    /// Integration test that combines multiple components
    #[tokio::test]
    async fn test_full_code_generation_workflow() {
        // Create a code generation agent
        let mut agent = CodeGenerationAgent::new();
        
        // Create a realistic task
        let task = AgentTask {
            id: "integration_test_001".to_string(),
            task_type: "generate_function".to_string(),
            description: "Create a function to calculate the factorial of a number".to_string(),
            context: json!({
                "language": "rust",
                "requirements": [
                    "should handle edge cases",
                    "should return a Result type",
                    "should be well documented"
                ]
            }),
            priority: TaskPriority::High,
            deadline: Some(chrono::Utc::now() + chrono::Duration::minutes(30)),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("test_mode".to_string(), json!(true));
                meta
            },
        };
        
        // Process the task
        let start_time = std::time::Instant::now();
        let result = agent.process_task(task).await;
        let elapsed = start_time.elapsed();
        
        // Verify the result
        assert!(result.is_ok());
        let agent_result = result.unwrap();
        
        // Basic result validation
        assert!(agent_result.success);
        assert!(!agent_result.artifacts.is_empty());
        assert!(elapsed < Duration::from_secs(5)); // Should be fast for stub generation
        
        // Verify artifact content
        let artifact = &agent_result.artifacts[0];
        assert_eq!(artifact.artifact_type, "source_code");
        // Check that the artifact contains rust-like content
        assert!(artifact.content.contains("rust") || artifact.content.contains("fn") || artifact.name.ends_with(".rs"));
        
        // The stub should contain function-like structure
        assert!(artifact.content.contains("fn") || artifact.content.contains("TODO"));
        
        println!("✅ Generated code artifact:");
        println!("{}", artifact.content);
        println!("✅ Generation completed in {:?}", elapsed);
    }
}