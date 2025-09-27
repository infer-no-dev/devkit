//! Demo workflow command for showcasing the agentic development environment

use crate::agents::{AgentSystem, AgentTask, TaskPriority};
use crate::cli::{CliRunner, DemoArgs};
use crate::codegen::templates::{Template, TemplateManager, TemplateVariable};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

pub async fn run(runner: &mut CliRunner, args: DemoArgs) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info("🚀 Welcome to Agentic Development Environment Demo");
    runner.print_info("This demo showcases the analyze → generate → interactive workflow");

    // Initialize systems
    let agent_system = Arc::new(AgentSystem::new());
    let _ = agent_system.initialize().await;

    let template_manager = TemplateManager::new()?;

    if args.step.is_none() || args.step.as_deref() == Some("all") {
        run_full_demo(runner, agent_system, template_manager).await
    } else {
        match args.step.as_deref().unwrap() {
            "analyze" => run_analyze_demo(runner, agent_system).await,
            "generate" => run_generate_demo(runner, agent_system, template_manager).await,
            "interactive" => run_interactive_demo(runner).await,
            step => {
                runner.print_error(&format!("Unknown demo step: {}", step));
                runner.print_info("Available steps: analyze, generate, interactive, all");
                Ok(())
            }
        }
    }
}

async fn run_full_demo(
    runner: &mut CliRunner,
    agent_system: Arc<AgentSystem>,
    template_manager: TemplateManager,
) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info("\n🔍 Phase 1: Code Analysis");
    run_analyze_demo(runner, agent_system.clone()).await?;

    sleep(Duration::from_secs(2)).await;

    runner.print_info("\n🛠️ Phase 2: Code Generation");
    run_generate_demo(runner, agent_system.clone(), template_manager).await?;

    sleep(Duration::from_secs(2)).await;

    runner.print_info("\n💬 Phase 3: Interactive Mode");
    run_interactive_demo(runner).await?;

    runner.print_success("✅ Demo completed successfully!");
    runner.print_info("The agentic development environment is ready for use.");

    Ok(())
}

async fn run_analyze_demo(
    runner: &mut CliRunner,
    agent_system: Arc<AgentSystem>,
) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info("Creating sample project for analysis...");

    // Create a sample project structure
    let demo_dir = create_demo_project()?;
    runner.print_info(&format!("Demo project created at: {}", demo_dir.display()));

    // Create analysis task
    let analysis_task = AgentTask {
        id: "demo_analysis".to_string(),
        task_type: "code_analysis".to_string(),
        description: "Analyze the demo Rust project structure and identify patterns".to_string(),
        context: serde_json::json!({
            "project_path": demo_dir.to_string_lossy(),
            "include_patterns": ["*.rs", "*.toml"],
            "analysis_type": "comprehensive"
        }),
        priority: TaskPriority::High,
        deadline: None,
        metadata: std::collections::HashMap::new(),
    };

    runner.print_info("Submitting analysis task to agent system...");

    match agent_system.submit_task(analysis_task).await {
        Ok(result) => {
            runner.print_success("✅ Analysis completed!");
            runner.print_info(&format!("Analysis result: {}", result.output));

            if !result.artifacts.is_empty() {
                runner.print_info(&format!("Generated {} artifacts:", result.artifacts.len()));
                for artifact in &result.artifacts {
                    runner.print_info(&format!(
                        "  - {} ({})",
                        artifact.name, artifact.artifact_type
                    ));
                }
            }
        }
        Err(e) => {
            runner.print_warning(&format!("Analysis failed: {}", e));
            runner
                .print_info("This is expected in demo mode - agents are not fully implemented yet");
        }
    }

    Ok(())
}

async fn run_generate_demo(
    runner: &mut CliRunner,
    agent_system: Arc<AgentSystem>,
    mut template_manager: TemplateManager,
) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info("Setting up code generation templates...");

    // Create sample templates
    create_demo_templates(&mut template_manager)?;

    // Create code generation task
    let generation_task = AgentTask {
        id: "demo_generation".to_string(),
        task_type: "code_generation".to_string(),
        description: "Generate a complete REST API handler with error handling".to_string(),
        context: serde_json::json!({
            "template": "rust_api_handler",
            "function_name": "handle_user_request",
            "parameters": "user_id: u64, request: UserRequest",
            "return_type": "Result<ApiResponse, ApiError>",
            "include_validation": true,
            "include_tests": true
        }),
        priority: TaskPriority::Normal,
        deadline: None,
        metadata: std::collections::HashMap::new(),
    };

    runner.print_info("Submitting code generation task...");

    match agent_system.submit_task(generation_task).await {
        Ok(result) => {
            runner.print_success("✅ Code generation completed!");
            runner.print_info(&result.output);

            // Display generated code
            if !result.artifacts.is_empty() {
                for artifact in &result.artifacts {
                    if artifact.artifact_type == "code" {
                        runner.print_info(&format!("\n📄 Generated {}:", artifact.name));
                        runner.print_code(&artifact.content);
                    }
                }
            }
        }
        Err(e) => {
            runner.print_warning(&format!("Code generation failed: {}", e));
            runner.print_info("Showing template-based example instead:");

            // Show example of what would be generated
            let example_code = generate_example_code(&template_manager)?;
            runner.print_code(&example_code);
        }
    }

    Ok(())
}

async fn run_interactive_demo(runner: &mut CliRunner) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info("Interactive mode allows natural language interaction with AI agents.");
    runner.print_info("Example commands you can use:");
    runner.print_info("  • 'Generate a function to validate email addresses'");
    runner.print_info("  • 'Explain how this algorithm works'");
    runner.print_info("  • 'Add unit tests for the user service'");
    runner.print_info("  • 'Debug this memory leak issue'");
    runner.print_info("  • 'Optimize this database query'");

    runner.print_info("\nSystem commands (start with /):");
    runner.print_info("  • /help - Show available commands");
    runner.print_info("  • /agents - List active agents");
    runner.print_info("  • /status - Show system status");
    runner.print_info("  • /save - Save session to file");

    if runner.is_interactive() {
        runner.print_info("\n🎯 Would you like to start interactive mode now? (y/N)");

        // In a real implementation, we'd read user input here
        runner.print_info("To start interactive mode, run:");
        runner.print_command("agentic-dev interactive");
    } else {
        runner.print_info("\n🎯 To start interactive mode, run:");
        runner.print_command("agentic-dev interactive");
    }

    Ok(())
}

fn create_demo_project() -> Result<PathBuf, Box<dyn std::error::Error>> {
    use std::fs;

    let project_path = std::env::current_dir()?.join("demo_project");
    if project_path.exists() {
        fs::remove_dir_all(&project_path)?;
    }
    fs::create_dir_all(&project_path)?;

    // Create Cargo.toml
    let cargo_toml = r#"[package]
name = "demo-api-server"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
warp = "0.3"
anyhow = "1.0"
uuid = { version = "1.0", features = ["v4"] }
"#;
    fs::write(project_path.join("Cargo.toml"), cargo_toml)?;

    // Create src directory and main.rs
    let src_dir = project_path.join("src");
    fs::create_dir_all(&src_dir)?;

    let main_rs = r#"use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub success: bool,
    pub data: serde_json::Value,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("User not found: {0}")]
    UserNotFound(u64),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Internal server error: {0}")]
    InternalError(String),
}

// TODO: This function needs to be implemented by the code generation agent
pub async fn handle_user_request(
    user_id: u64, 
    request: UserRequest
) -> Result<ApiResponse, ApiError> {
    // Placeholder implementation
    todo!("Implement user request handling logic")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Demo API Server starting...");
    
    // This is where the agent-generated code would be integrated
    let response = handle_user_request(1, UserRequest {
        name: Some("Alice".to_string()),
        email: Some("alice@example.com".to_string()),
    }).await?;
    
    println!("Response: {:?}", response);
    
    Ok(())
}
"#;
    fs::write(src_dir.join("main.rs"), main_rs)?;

    // Create lib.rs with additional modules
    let lib_rs = r#"//! Demo API Server Library
//!
//! This demonstrates a typical Rust web service structure
//! that benefits from AI-assisted development.

pub mod handlers;
pub mod models;
pub mod utils;

pub use handlers::*;
pub use models::*;
"#;
    fs::write(src_dir.join("lib.rs"), lib_rs)?;

    // Create placeholder modules
    fs::write(
        src_dir.join("handlers.rs"),
        "// API handlers will be generated here\n",
    )?;
    fs::write(
        src_dir.join("models.rs"),
        "// Data models will be generated here\n",
    )?;
    fs::write(
        src_dir.join("utils.rs"),
        "// Utility functions will be generated here\n",
    )?;

    // Create tests directory
    let tests_dir = project_path.join("tests");
    fs::create_dir_all(&tests_dir)?;
    fs::write(
        tests_dir.join("integration_tests.rs"),
        "// Integration tests will be generated here\n",
    )?;

    Ok(project_path)
}

fn create_demo_templates(
    template_manager: &mut TemplateManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let rust_api_handler = Template {
        name: "rust_api_handler".to_string(),
        language: "rust".to_string(),
        description: "REST API handler with error handling".to_string(),
        content: r#"/// Handle {{description}}
pub async fn {{function_name}}(
    {{parameters}}
) -> {{return_type}} {
    // Validate input parameters
    {{#if include_validation}}
    if let Err(validation_error) = validate_request(&request) {
        return Err(ApiError::ValidationError(validation_error));
    }
    {{/if}}
    
    // Process the request
    match process_{{function_name}}({{parameter_names}}).await {
        Ok(result) => {
            Ok(ApiResponse {
                success: true,
                data: serde_json::to_value(result)?,
                message: "Request processed successfully".to_string(),
            })
        }
        Err(e) => {
            tracing::error!("Error processing request: {}", e);
            Err(ApiError::InternalError(e.to_string()))
        }
    }
}

{{#if include_validation}}
fn validate_request(request: &UserRequest) -> Result<(), String> {
    if let Some(ref email) = request.email {
        if !is_valid_email(email) {
            return Err("Invalid email format".to_string());
        }
    }
    
    if let Some(ref name) = request.name {
        if name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }
    }
    
    Ok(())
}

fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}
{{/if}}

async fn process_{{function_name}}(
    {{parameters}}
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // TODO: Implement actual processing logic
    Ok(serde_json::json!({
        "processed": true,
        "user_id": user_id,
        "timestamp": chrono::Utc::now()
    }))
}

{{#if include_tests}}
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_{{function_name}}_success() {
        let request = UserRequest {
            name: Some("Test User".to_string()),
            email: Some("test@example.com".to_string()),
        };
        
        let result = {{function_name}}(1, request).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(response.success);
    }
    
    #[tokio::test]
    async fn test_{{function_name}}_validation_error() {
        let request = UserRequest {
            name: Some("".to_string()), // Empty name should fail validation
            email: Some("invalid-email".to_string()),
        };
        
        let result = {{function_name}}(1, request).await;
        assert!(result.is_err());
    }
}
{{/if}}
"#
        .to_string(),
        variables: vec![
            TemplateVariable::required("function_name", "Name of the function"),
            TemplateVariable::required("parameters", "Function parameters"),
            TemplateVariable::required("return_type", "Function return type"),
            TemplateVariable::optional("description", "Function description", "Generated function"),
            TemplateVariable::optional("parameter_names", "Parameter names for function calls", ""),
            TemplateVariable::optional("include_validation", "Include validation code", "false"),
            TemplateVariable::optional("include_tests", "Include test code", "false"),
        ],
    };

    template_manager.add_template(rust_api_handler);

    Ok(())
}

fn generate_example_code(
    _template_manager: &TemplateManager,
) -> Result<String, Box<dyn std::error::Error>> {
    // This would normally use the template engine, but for demo purposes we'll return static code
    Ok(
        r#"/// Handle user request with validation and error handling
pub async fn handle_user_request(
    user_id: u64,
    request: UserRequest,
) -> Result<ApiResponse, ApiError> {
    // Validate input parameters
    if let Err(validation_error) = validate_request(&request) {
        return Err(ApiError::ValidationError(validation_error));
    }
    
    // Process the request
    match process_user_request(user_id, request).await {
        Ok(result) => {
            Ok(ApiResponse {
                success: true,
                data: serde_json::to_value(result)?,
                message: "Request processed successfully".to_string(),
            })
        }
        Err(e) => {
            tracing::error!("Error processing request: {}", e);
            Err(ApiError::InternalError(e.to_string()))
        }
    }
}

fn validate_request(request: &UserRequest) -> Result<(), String> {
    if let Some(ref email) = request.email {
        if !is_valid_email(email) {
            return Err("Invalid email format".to_string());
        }
    }
    
    if let Some(ref name) = request.name {
        if name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }
    }
    
    Ok(())
}

fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}

async fn process_user_request(
    user_id: u64,
    request: UserRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    Ok(serde_json::json!({
        "processed": true,
        "user_id": user_id,
        "timestamp": chrono::Utc::now(),
        "updated_fields": {
            "name": request.name,
            "email": request.email
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_handle_user_request_success() {
        let request = UserRequest {
            name: Some("Alice".to_string()),
            email: Some("alice@example.com".to_string()),
        };
        
        let result = handle_user_request(1, request).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(response.success);
        assert_eq!(response.message, "Request processed successfully");
    }
    
    #[tokio::test]
    async fn test_handle_user_request_validation_error() {
        let request = UserRequest {
            name: Some("".to_string()),
            email: Some("invalid-email".to_string()),
        };
        
        let result = handle_user_request(1, request).await;
        assert!(result.is_err());
        
        if let Err(ApiError::ValidationError(msg)) = result {
            assert!(msg.contains("email") || msg.contains("name"));
        } else {
            panic!("Expected ValidationError");
        }
    }
}
"#
        .to_string(),
    )
}
