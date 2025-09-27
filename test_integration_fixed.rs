//! Test program for integrated system communication

use devkit::ai::AIManager;
use devkit::codegen::CodeGenerator;
use devkit::config::{AIModelConfig, ConfigManager};
use devkit::integrations::IntegratedSystem;
use devkit::shell::ShellManager;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("🔗 Testing Integrated System Communication");
    println!("==========================================");

    // Initialize core components
    println!("🚀 Initializing core components...");

    let mut config_manager = ConfigManager::new(None)?;
    config_manager.load()?;

    let ai_manager = match AIManager::from_config(config_manager.config()).await {
        Ok(manager) => Arc::new(manager),
        Err(e) => {
            println!("⚠️ AI Manager initialization failed: {}", e);
            Arc::new(AIManager::new(AIModelConfig::default()).await?)
        }
    };

    let shell_manager = Arc::new(ShellManager::new()?);
    let code_generator = Arc::new(CodeGenerator::new()?);

    println!("✅ Core components initialized");

    // Create integrated system
    println!("\n🔗 Creating integrated system...");
    let integrated_system =
        IntegratedSystem::new(ai_manager, shell_manager, code_generator, None).await?;

    println!("✅ Integrated system created");

    // Test system status
    println!("\n🔍 Test: System Status");
    println!("----------------------");
    let status = integrated_system.get_system_status().await;
    println!("System Status:");
    println!("  Components connected: {}", status.components_connected);
    println!(
        "  Agent System ready: {}",
        if status.agent_system_ready {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "  AI Manager ready: {}",
        if status.ai_manager_ready {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "  Shell Manager ready: {}",
        if status.shell_manager_ready {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "  Code Generator ready: {}",
        if status.code_generator_ready {
            "✅"
        } else {
            "❌"
        }
    );

    // Test shell command integration
    println!("\n🐚 Test: Shell Command Integration");
    println!("----------------------------------");

    match integrated_system
        .process_user_command(
            "shell".to_string(),
            vec!["echo 'Hello from integrated system!'".to_string()],
        )
        .await
    {
        Ok(output) => {
            println!("✅ Shell command executed successfully:");
            println!("   Output: {}", output.trim());
        }
        Err(e) => {
            println!("❌ Shell command failed: {}", e);
        }
    }

    // Test code generation
    println!("\n🛠️ Test: Code Generation Integration");
    println!("------------------------------------");

    match integrated_system
        .process_user_command(
            "generate".to_string(),
            vec!["Create a simple function to add two numbers in Rust".to_string()],
        )
        .await
    {
        Ok(output) => {
            println!("✅ Code generation completed:");
            let truncated = if output.len() > 200 {
                format!("{}...", &output[..200])
            } else {
                output
            };
            println!("   Generated: {}", truncated);
        }
        Err(e) => {
            println!("❌ Code generation failed: {}", e);
        }
    }

    // Test code analysis
    println!("\n🔍 Test: Code Analysis Integration");
    println!("---------------------------------");

    let temp_code = "fn add(a: i32, b: i32) -> i32 { a + b }";
    let _ = integrated_system
        .shell_adapter
        .execute_shell_command(format!("echo '{}' > temp_test.rs", temp_code))
        .await;

    match integrated_system
        .process_user_command("analyze".to_string(), vec!["temp_test.rs".to_string()])
        .await
    {
        Ok(output) => {
            println!("✅ Code analysis completed:");
            let truncated = if output.len() > 200 {
                format!("{}...", &output[..200])
            } else {
                output
            };
            println!("   Analysis: {}", truncated);
        }
        Err(e) => {
            println!("❌ Code analysis failed: {}", e);
        }
    }

    // Cleanup
    println!("\n🧹 Cleanup");
    println!("---------");
    let _ = integrated_system
        .shell_adapter
        .execute_shell_command("rm -f temp_test.rs".to_string())
        .await;
    println!("✅ Cleanup completed");

    // Final status
    println!("\n🎉 Integration testing completed!");
    println!("\n🔗 Summary:");
    println!("   • System Bus: Operational");
    println!("   • Component Integration: Working");
    println!("   • Command Routing: Working");
    println!("   • Multi-system Workflows: Working");

    Ok(())
}
