//! Test program for shell integration
//!
//! This program demonstrates the shell integration functionality

use devkit::shell::{CommandOperation, ShellConfig, ShellManager};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🐚 Testing Shell Integration");
    println!("=============================");

    // Initialize shell manager
    let shell_manager = ShellManager::new()?;
    println!("✅ Shell Manager initialized");
    println!("   Current shell: {}", shell_manager.current_shell());

    // Test 1: Basic command execution
    println!("\n📋 Test 1: Basic Command Execution");
    println!("----------------------------------");

    let echo_result = shell_manager
        .execute_command("echo 'Hello from shell!'", None)
        .await?;
    println!("Echo command result:");
    println!("  Exit code: {}", echo_result.exit_code);
    println!("  Output: {}", echo_result.stdout.trim());
    println!("  Execution time: {} ms", echo_result.execution_time_ms);

    // Test 2: File operations
    println!("\n📁 Test 2: File Operations");
    println!("-------------------------");

    // Create a test directory
    let test_dir = "shell_test_dir";
    let create_dir_op = CommandOperation::CreateDirectory {
        path: test_dir.to_string(),
        recursive: true,
    };

    let mkdir_result = shell_manager.execute_operation(create_dir_op, None).await?;
    println!(
        "Create directory result: exit code {}",
        mkdir_result.exit_code
    );

    // Write to a test file
    let test_content = "Hello from Shell Manager!\\nThis is a test file.";
    let test_file = format!("{}/test.txt", test_dir);

    let write_result = shell_manager
        .write_file_content(&test_file, test_content)
        .await?;
    println!("Write file result: exit code {}", write_result.exit_code);

    // Read from the test file
    match shell_manager.read_file_content(&test_file).await {
        Ok(content) => {
            println!("File content read successfully:");
            println!("  Content: {}", content.trim());
        }
        Err(e) => {
            println!("Failed to read file: {}", e);
        }
    }

    // Test 3: Directory listing
    println!("\n📂 Test 3: Directory Listing");
    println!("----------------------------");

    let ls_result = shell_manager
        .execute_operation(CommandOperation::ListFiles, None)
        .await?;
    if ls_result.exit_code == 0 {
        println!("Directory contents:");
        for line in ls_result.stdout.lines().take(5) {
            println!("  {}", line);
        }
    } else {
        println!("Failed to list directory: {}", ls_result.stderr);
    }

    // Test 4: Command existence check
    println!("\n🔍 Test 4: Command Existence Checks");
    println!("-----------------------------------");

    let commands_to_check = ["git", "cargo", "npm", "node", "python", "python3"];

    for cmd in &commands_to_check {
        let exists = shell_manager.command_exists(cmd).await;
        let status = if exists {
            "✅ Available"
        } else {
            "❌ Not found"
        };
        println!("  {}: {}", cmd, status);
    }

    // Test 5: Git operations (if git is available)
    if shell_manager.command_exists("git").await {
        println!("\n🔧 Test 5: Git Operations");
        println!("------------------------");

        // Check git status in current directory
        match shell_manager.git_status().await {
            Ok(status_result) => {
                println!("Git status result:");
                println!("  Exit code: {}", status_result.exit_code);
                if status_result.exit_code == 0 {
                    println!(
                        "  Status: {}",
                        status_result
                            .stdout
                            .lines()
                            .take(3)
                            .collect::<Vec<_>>()
                            .join("\\n")
                    );
                } else {
                    println!("  Error: {}", status_result.stderr.trim());
                }
            }
            Err(e) => {
                println!("Git status error: {}", e);
            }
        }
    }

    // Test 6: Rust/Cargo operations (if cargo is available)
    if shell_manager.command_exists("cargo").await {
        println!("\n🦀 Test 6: Cargo Operations");
        println!("--------------------------");

        // Test cargo check
        let cargo_op = CommandOperation::CargoCheck;
        let check_result = shell_manager.execute_operation(cargo_op, None).await?;
        println!("Cargo check result:");
        println!("  Exit code: {}", check_result.exit_code);
        if check_result.exit_code == 0 {
            println!("  ✅ Project compiles successfully");
        } else {
            println!("  ❌ Compilation errors found");
            println!(
                "  Error output: {}",
                check_result
                    .stderr
                    .lines()
                    .take(3)
                    .collect::<Vec<_>>()
                    .join("\\n")
            );
        }
    }

    // Test 7: Multiple command execution
    println!("\n📝 Test 7: Multiple Commands");
    println!("----------------------------");

    let commands = ["echo 'Command 1'", "echo 'Command 2'", "date"];

    match shell_manager.execute_commands(&commands, None).await {
        Ok(results) => {
            println!("Executed {} commands:", results.len());
            for (i, result) in results.iter().enumerate() {
                println!(
                    "  Command {}: exit code {} - {}",
                    i + 1,
                    result.exit_code,
                    result.stdout.trim()
                );
            }
        }
        Err(e) => {
            println!("Failed to execute commands: {}", e);
        }
    }

    // Test 8: Environment variables
    println!("\n🌍 Test 8: Environment Variables");
    println!("--------------------------------");

    println!("Current environment variables (first 5):");
    let env_vars: Vec<_> = shell_manager.get_environment().iter().take(5).collect();
    for (key, value) in env_vars {
        println!("  {} = {}", key, value);
    }

    // Test 9: Project setup (if in a temporary location)
    if env::current_dir()?.join("temp_test").exists() == false {
        println!("\n🏗️ Test 9: Project Setup");
        println!("------------------------");

        let temp_project = "temp_rust_project";
        match shell_manager.setup_project("rust", temp_project).await {
            Ok(results) => {
                println!("Project setup completed!");
                println!("Operations performed: {}", results.len());
                for (i, result) in results.iter().enumerate() {
                    println!(
                        "  Operation {}: exit code {} - execution time {} ms",
                        i + 1,
                        result.exit_code,
                        result.execution_time_ms
                    );
                }
            }
            Err(e) => {
                println!("Project setup failed: {}", e);
            }
        }
    }

    // Cleanup
    println!("\n🧹 Cleanup");
    println!("---------");

    let cleanup_commands = [
        format!("rm -rf {}", test_dir),
        "rm -rf temp_rust_project".to_string(),
    ];

    for cmd in &cleanup_commands {
        if let Ok(result) = shell_manager.execute_command(cmd, None).await {
            if result.exit_code == 0 {
                println!("✅ Cleaned up: {}", cmd);
            }
        }
    }

    println!("\n🎉 Shell integration testing completed!");
    Ok(())
}
