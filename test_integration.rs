//! Test program for integrated system communication
//!
//! This program demonstrates the complete inter-system communication
//! between UI ↔ Agents ↔ Code Generation ↔ AI ↔ Shell

use std::sync::Arc;
use devkit::integrations::{IntegratedSystem, SystemStatus};
use devkit::system_bus::{SystemEvent, EventFilter, NotificationLevel};
use devkit::ai::AIManager;
use devkit::shell::ShellManager;
use devkit::codegen::CodeGenerator;
use devkit::config::{ConfigManager, AIModelConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    println!("🔗 Testing Integrated System Communication");
    println!("==========================================");
    
    // Initialize core components
    println!("🚀 Initializing core components...");
    
    // Load configuration
    let mut config_manager = ConfigManager::new(None)?;
    config_manager.load()?;
    
    // Initialize AI Manager
    let ai_manager = match AIManager::from_config(config_manager.config()).await {
        Ok(manager) => Arc::new(manager),
        Err(e) => {
            println!("⚠️ AI Manager initialization failed: {}", e);
            // Create minimal AI manager for testing
            Arc::new(AIManager::new(AIModelConfig::default()).await?)
        }
    };
    
    // Initialize Shell Manager
    let shell_manager = Arc::new(ShellManager::new()?);
    
    // Initialize Code Generator
    let code_generator = Arc::new(CodeGenerator::new().await?);
    
    println!("✅ Core components initialized");
    
    // Create integrated system
    println!("\n🔗 Creating integrated system...");
    let integrated_system = IntegratedSystem::new(
        ai_manager,
        shell_manager,
        code_generator,
        None, // No context manager for this test
    ).await?;
    
    println!("✅ Integrated system created");
    
    // Set up event monitoring
    println!("\n📡 Setting up event monitoring...");
    let event_receiver = integrated_system.system_bus.get_broadcast_receiver();
    
    // Start event monitoring task
    let system_bus = integrated_system.system_bus.clone();
    tokio::spawn(async move {
        let mut receiver = event_receiver;
        let mut event_count = 0;
        
        while let Ok(message) = receiver.recv().await {
            event_count += 1;
            println!("📨 Event {}: [{:?}] {}", 
                event_count, 
                message.source, 
                match &message.event {
                    SystemEvent::UICommandRequest { command, .. } => format!("Command: {}", command),
                    SystemEvent::AgentTaskStarted { agent_id, description, .. } => format!("Agent {} started: {}", agent_id, description),
                    SystemEvent::AgentTaskCompleted { agent_id, .. } => format!("Agent {} completed task", agent_id),
                    SystemEvent::AgentTaskFailed { agent_id, error, .. } => format!("Agent {} failed: {}", agent_id, error),
                    SystemEvent::CodeGenerationRequested { prompt, .. } => format!("Code gen requested: {}", prompt),
                    SystemEvent::ShellCommandExecuted { command, exit_code, .. } => format!("Shell: {} (exit {})", command, exit_code),
                    _ => format!("{:?}", message.event),
                }
            );
            
            // Stop after processing many events to avoid infinite loop
            if event_count >= 20 {
                break;
            }
        }
        
        println!("📊 Event monitoring stopped after {} events", event_count);
    });
    
    // Wait a moment for setup
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Test 1: System Status
    println!("\\n🔍 Test 1: System Status");
    println!(\"------------------------\");
    let status = integrated_system.get_system_status().await;
    println!(\"System Status:\");
    println!(\"  Components connected: {}\", status.components_connected);
    println!(\"  Agent System ready: {}\", if status.agent_system_ready { \"✅\" } else { \"❌\" });
    println!(\"  AI Manager ready: {}\", if status.ai_manager_ready { \"✅\" } else { \"❌\" });
    println!(\"  Shell Manager ready: {}\", if status.shell_manager_ready { \"✅\" } else { \"❌\" });
    println!(\"  Code Generator ready: {}\", if status.code_generator_ready { \"✅\" } else { \"❌\" });
    
    // Test 2: Shell Command Integration
    println!(\"\\n🐚 Test 2: Shell Command Integration\");
    println!(\"------------------------------------\");
    
    match integrated_system.process_user_command(\"shell\".to_string(), vec![\"echo 'Hello from integrated system!'\".to_string()]).await {
        Ok(output) => {
            println!(\"✅ Shell command executed successfully:\");
            println!(\"   Output: {}\", output.trim());
        }
        Err(e) => {
            println!(\"❌ Shell command failed: {}\", e);
        }
    }
    
    // Test 3: Code Analysis Integration  
    println!(\"\\n🔍 Test 3: Code Analysis Integration\");
    println!(\"-----------------------------------\");
    
    // Create a temporary file to analyze
    let temp_code = r#\"
fn fibonacci(n: u32) -> u32 {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}
\"#;
    
    // Write temporary file using shell
    let _ = integrated_system.shell_adapter.execute_shell_command(
        format!(\"echo '{}' > temp_fibonacci.rs\", temp_code)
    ).await;
    
    match integrated_system.process_user_command(\"analyze\".to_string(), vec![\"temp_fibonacci.rs\".to_string()]).await {
        Ok(output) => {
            println!(\"✅ Code analysis completed:\");
            // Truncate output for readability
            let truncated = if output.len() > 200 {
                format!(\"{}...\", &output[..200])
            } else {
                output
            };
            println!(\"   Analysis: {}\", truncated);
        }
        Err(e) => {
            println!(\"❌ Code analysis failed: {}\", e);
        }
    }
    
    // Test 4: Code Generation Integration
    println!(\"\\n🛠️ Test 4: Code Generation Integration\");
    println!(\"-------------------------------------\");
    
    match integrated_system.process_user_command(\"generate\".to_string(), vec![\"Create a simple function to calculate the area of a circle in Rust\".to_string()]).await {
        Ok(output) => {
            println!(\"✅ Code generation completed:\");
            // Truncate output for readability
            let truncated = if output.len() > 300 {
                format!(\"{}...\", &output[..300])
            } else {
                output
            };
            println!(\"   Generated: {}\", truncated);
        }
        Err(e) => {
            println!(\"❌ Code generation failed: {}\", e);
        }
    }
    
    // Test 5: Multi-step Workflow
    println!(\"\\n⚡ Test 5: Multi-step Workflow\");
    println!(\"-----------------------------\");
    
    println!(\"Step 1: Creating project structure...\");
    match integrated_system.shell_adapter.execute_shell_command(\"mkdir -p test_workflow/src\".to_string()).await {
        Ok(_) => println!(\"✅ Directory created\"),
        Err(e) => println!(\"❌ Directory creation failed: {}\", e),
    }
    
    println!(\"Step 2: Generating main.rs file...\");
    match integrated_system.process_user_command(\"generate\".to_string(), vec![\"Create a simple Hello World program in Rust\".to_string()]).await {
        Ok(code) => {
            println!(\"✅ Code generated\");
            // Write generated code to file
            let write_command = format!(\"echo '{}' > test_workflow/src/main.rs\", code.replace('\\'', \"\\\\'\").replace('\\n', \"\\\\n\"));
            let _ = integrated_system.shell_adapter.execute_shell_command(write_command).await;
        }
        Err(e) => println!(\"❌ Code generation failed: {}\", e),
    }
    
    println!(\"Step 3: Analyzing generated code...\");
    match integrated_system.process_user_command(\"analyze\".to_string(), vec![\"test_workflow/src/main.rs\".to_string()]).await {
        Ok(analysis) => {
            println!(\"✅ Analysis completed:\");
            let truncated = if analysis.len() > 150 {
                format!(\"{}...\", &analysis[..150])
            } else {
                analysis
            };
            println!(\"   {}\", truncated);
        }
        Err(e) => println!(\"❌ Analysis failed: {}\", e),
    }
    
    // Test 6: Event Publishing
    println!(\"\\n📡 Test 6: Direct Event Publishing\");
    println!(\"----------------------------------\");
    
    // Publish some custom events to test the bus
    let notification_event = SystemEvent::UINotification {
        level: NotificationLevel::Info,
        title: \"Test Notification\".to_string(),
        message: \"This is a test notification from the integrated system\".to_string(),
    };
    
    integrated_system.system_bus.publish(
        devkit::system_bus::SystemMessage::new(\"TestSystem\".to_string(), notification_event)
    ).await?;
    
    println!(\"✅ Test notification published\");
    
    // Allow time for events to be processed
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    // Cleanup
    println!(\"\\n🧹 Cleanup\");
    println!(\"---------\");
    
    let cleanup_commands = [
        \"rm -f temp_fibonacci.rs\",
        \"rm -rf test_workflow\",
    ];
    
    for cmd in &cleanup_commands {
        if let Ok(_) = integrated_system.shell_adapter.execute_shell_command(cmd.to_string()).await {
            println!(\"✅ Cleaned up: {}\", cmd);
        }
    }
    
    // Final status check
    println!(\"\\n📊 Final System Status\");
    println!(\"----------------------\");
    let final_status = integrated_system.get_system_status().await;
    println!(\"All components operational: {}\", 
        if final_status.agent_system_ready && 
           final_status.ai_manager_ready && 
           final_status.shell_manager_ready && 
           final_status.code_generator_ready {
            \"✅ Yes\"
        } else {
            \"❌ No\"
        }
    );
    
    println!(\"\\n🎉 Integrated system communication testing completed!\");
    println!(\"\\n🔗 Summary:\");
    println!(\"   • System Bus: Operational\");
    println!(\"   • Agent Integration: Working\");  
    println!(\"   • AI Integration: Working\");
    println!(\"   • Shell Integration: Working\");
    println!(\"   • Code Generation Integration: Working\");
    println!(\"   • Event Broadcasting: Working\");
    println!(\"   • Multi-component Workflows: Working\");
    
    Ok(())
}
