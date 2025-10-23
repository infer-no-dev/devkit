//! Status command implementation
//!
//! This module provides comprehensive system status checks including:
//! - Agent system health and status
//! - Configuration validation
//! - Context analysis state
//! - External dependencies
//! - Performance metrics
//! - System diagnostics

use super::utils::*;
use crate::cli::{CliRunner, OutputFormat, StatusArgs};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

/// System component status
#[derive(Debug, Clone)]
pub struct ComponentStatus {
    pub name: String,
    pub status: StatusLevel,
    pub message: String,
    pub details: HashMap<String, String>,
    pub last_check: SystemTime,
    pub response_time: Option<Duration>,
}

/// Status levels
#[derive(Debug, Clone, PartialEq)]
pub enum StatusLevel {
    Healthy,
    Warning,
    Error,
    Unknown,
}

impl StatusLevel {
    pub fn as_emoji(&self) -> &'static str {
        match self {
            StatusLevel::Healthy => "✅",
            StatusLevel::Warning => "⚠️",
            StatusLevel::Error => "❌",
            StatusLevel::Unknown => "❓",
        }
    }

    pub fn as_string(&self) -> &'static str {
        match self {
            StatusLevel::Healthy => "HEALTHY",
            StatusLevel::Warning => "WARNING",
            StatusLevel::Error => "ERROR",
            StatusLevel::Unknown => "UNKNOWN",
        }
    }
}

/// Overall system status
#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub overall_status: StatusLevel,
    pub components: Vec<ComponentStatus>,
    pub performance_metrics: Option<PerformanceMetrics>,
    pub system_info: SystemInfo,
    pub timestamp: SystemTime,
}

/// Performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub memory_usage: u64,
    pub cpu_usage: f32,
    pub disk_usage: u64,
    pub active_agents: usize,
    pub context_cache_size: usize,
    pub avg_response_time: Duration,
}

/// System information
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub rust_version: String,
    pub agentic_version: String,
    pub working_directory: PathBuf,
    pub config_file: Option<PathBuf>,
}

/// Run the status command
pub async fn run(
    runner: &mut CliRunner,
    args: StatusArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info("🔍 Checking system status...");

    let start_time = Instant::now();

    // Get system status
    let system_status = get_system_status(runner, &args).await?;

    let check_duration = start_time.elapsed();
    runner.print_verbose(&format!("Status check completed in {:?}", check_duration));

    // Display results
    match runner.format {
        OutputFormat::Json => display_json_status(&system_status)?,
        OutputFormat::Table => display_table_status(runner, &system_status, args.detailed)?,
        _ => display_text_status(runner, &system_status, args.detailed)?,
    }

    // Exit with error code if any component is in error state
    if system_status.overall_status == StatusLevel::Error {
        std::process::exit(1);
    }

    Ok(())
}

/// Get comprehensive system status
async fn get_system_status(
    runner: &mut CliRunner,
    args: &StatusArgs,
) -> Result<SystemStatus, Box<dyn std::error::Error>> {
    let mut components = Vec::new();

    // Check configuration
    let config_status = check_configuration_status(runner).await?;
    if args.components.is_empty() || args.components.contains(&"config".to_string()) {
        components.push(config_status);
    }

    // Check context system
    let context_status = check_context_status(runner).await?;
    if args.components.is_empty() || args.components.contains(&"context".to_string()) {
        components.push(context_status);
    }

    // Check agent system
    let agent_status = check_agent_status(runner).await?;
    if args.components.is_empty() || args.components.contains(&"agents".to_string()) {
        components.push(agent_status);
    }

    // Check shell integration
    let shell_status = check_shell_status().await?;
    if args.components.is_empty() || args.components.contains(&"shell".to_string()) {
        components.push(shell_status);
    }

    // Check external dependencies if requested
    if args.external || args.components.contains(&"external".to_string()) {
        let external_status = check_external_dependencies().await?;
        components.extend(external_status);
    }

    // Calculate overall status
    let overall_status = calculate_overall_status(&components);

    // Get performance metrics if requested
    let performance_metrics = if args.performance {
        Some(get_performance_metrics(runner).await?)
    } else {
        None
    };

    // Get system information
    let system_info = get_system_info();

    Ok(SystemStatus {
        overall_status,
        components,
        performance_metrics,
        system_info,
        timestamp: SystemTime::now(),
    })
}

/// Check configuration status
async fn check_configuration_status(
    runner: &mut CliRunner,
) -> Result<ComponentStatus, Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    let mut details = HashMap::new();

    let (status, message) = match runner.config_manager.validate() {
        Ok(_) => {
            details.insert("validation".to_string(), "passed".to_string());

            // Check if configuration file exists
            let config_path = runner.config_manager.config_path();
            details.insert(
                "config_file".to_string(),
                config_path.to_string_lossy().to_string(),
            );
            details.insert("file_exists".to_string(), config_path.exists().to_string());

            // Check environment
            details.insert(
                "environment".to_string(),
                runner.config_manager.environment().to_string(),
            );

            (
                StatusLevel::Healthy,
                "Configuration is valid and loaded".to_string(),
            )
        }
        Err(e) => {
            details.insert("validation_error".to_string(), e.to_string());
            (
                StatusLevel::Error,
                format!("Configuration validation failed: {}", e),
            )
        }
    };

    Ok(ComponentStatus {
        name: "Configuration".to_string(),
        status,
        message,
        details,
        last_check: SystemTime::now(),
        response_time: Some(start_time.elapsed()),
    })
}

/// Check context system status
async fn check_context_status(
    runner: &mut CliRunner,
) -> Result<ComponentStatus, Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    let mut details = HashMap::new();

    // Initialize context manager if needed
    let _ = runner.ensure_context_manager().await;

    let (status, message) = if runner.context_manager.is_some() {
        details.insert("initialized".to_string(), "true".to_string());

        // Try to get current directory context
        let current_dir = std::env::current_dir()?;
        details.insert(
            "working_directory".to_string(),
            current_dir.to_string_lossy().to_string(),
        );

        // Check if it's a valid project directory
        let is_project = detect_project_language(&current_dir).is_some();
        details.insert("is_project".to_string(), is_project.to_string());

        if is_project {
            (StatusLevel::Healthy, "Context system is ready".to_string())
        } else {
            (
                StatusLevel::Warning,
                "Not in a recognized project directory".to_string(),
            )
        }
    } else {
        details.insert("initialized".to_string(), "false".to_string());
        (
            StatusLevel::Warning,
            "Context system not initialized".to_string(),
        )
    };

    Ok(ComponentStatus {
        name: "Context System".to_string(),
        status,
        message,
        details,
        last_check: SystemTime::now(),
        response_time: Some(start_time.elapsed()),
    })
}

/// Check agent system status
async fn check_agent_status(
    runner: &mut CliRunner,
) -> Result<ComponentStatus, Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    let mut details = HashMap::new();

    // Initialize agent system if needed (this also ensures context manager)
    let _ = runner.ensure_agent_system().await;

    let (status, message) = if runner.agent_system.is_some() {
        details.insert("initialized".to_string(), "true".to_string());

        // Get agent system reference
        if let Some(agent_system) = &runner.agent_system {
            let agent_statuses = agent_system.get_agent_statuses().await;
            details.insert("total_agents".to_string(), agent_statuses.len().to_string());

            let active_count = agent_statuses
                .iter()
                .filter(|(_, status)| {
                    matches!(
                        status,
                        crate::agents::AgentStatus::Processing { task_id: _ }
                    )
                })
                .count();
            details.insert("active_agents".to_string(), active_count.to_string());

            let idle_count = agent_statuses
                .iter()
                .filter(|(_, status)| matches!(status, crate::agents::AgentStatus::Idle))
                .count();
            details.insert("idle_agents".to_string(), idle_count.to_string());

            (
                StatusLevel::Healthy,
                format!("Agent system running with {} agents", agent_statuses.len()),
            )
        } else {
            (
                StatusLevel::Warning,
                "Agent system reference not available".to_string(),
            )
        }
    } else {
        details.insert("initialized".to_string(), "false".to_string());
        (
            StatusLevel::Warning,
            "Agent system not initialized".to_string(),
        )
    };

    Ok(ComponentStatus {
        name: "Agent System".to_string(),
        status,
        message,
        details,
        last_check: SystemTime::now(),
        response_time: Some(start_time.elapsed()),
    })
}

/// Check if shell integration is actually installed
fn check_shell_integration_installed(shell: &str) -> bool {
    use std::path::PathBuf;

    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return false,
    };

    // Check for completion files
    let completion_file = match shell {
        "bash" => {
            Some(PathBuf::from(&home).join(".local/share/bash-completion/completions/devkit"))
        }
        "zsh" => Some(PathBuf::from(&home).join(".local/share/zsh/site-functions/_devkit")),
        "fish" => Some(PathBuf::from(&home).join(".config/fish/completions/devkit.fish")),
        _ => None,
    };

    // Check for shell aliases in config files
    let config_file = match shell {
        "bash" => Some(PathBuf::from(&home).join(".bashrc")),
        "zsh" => Some(PathBuf::from(&home).join(".zshrc")),
        "fish" => Some(PathBuf::from(&home).join(".config/fish/config.fish")),
        _ => None,
    };

    let completion_exists = completion_file
        .as_ref()
        .map(|p| p.exists())
        .unwrap_or(false);

    let aliases_exist = config_file
        .as_ref()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|content| content.contains("devkit shell integration"))
        .unwrap_or(false);

    completion_exists && aliases_exist
}

/// Check shell integration status
async fn check_shell_status() -> Result<ComponentStatus, Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    let mut details = HashMap::new();

    // Get current shell
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "unknown".to_string());
    details.insert("current_shell".to_string(), shell.clone());

    // Check if shell is supported
    let supported_shells = vec!["bash", "zsh", "fish"];
    let shell_name = shell.split('/').last().unwrap_or(&shell);
    let is_supported = supported_shells.contains(&shell_name);
    details.insert("supported".to_string(), is_supported.to_string());

    // Check if completions are installed
    let completions_installed = check_shell_integration_installed(shell_name);
    details.insert(
        "completions_installed".to_string(),
        completions_installed.to_string(),
    );

    let (status, message) = if is_supported {
        if completions_installed {
            (
                StatusLevel::Healthy,
                format!("Shell integration active ({})", shell_name),
            )
        } else {
            (
                StatusLevel::Warning,
                format!(
                    "Shell integration available but not installed ({})",
                    shell_name
                ),
            )
        }
    } else {
        (
            StatusLevel::Warning,
            format!("Shell not fully supported ({})", shell_name),
        )
    };

    Ok(ComponentStatus {
        name: "Shell Integration".to_string(),
        status,
        message,
        details,
        last_check: SystemTime::now(),
        response_time: Some(start_time.elapsed()),
    })
}

/// Check external dependencies
async fn check_external_dependencies() -> Result<Vec<ComponentStatus>, Box<dyn std::error::Error>> {
    let mut components = Vec::new();

    // Check Git
    let git_status = check_command_availability("git", &["--version"]).await;
    components.push(ComponentStatus {
        name: "Git".to_string(),
        status: if git_status.0 {
            StatusLevel::Healthy
        } else {
            StatusLevel::Warning
        },
        message: if git_status.0 {
            format!("Git available: {}", git_status.1.trim())
        } else {
            "Git not available".to_string()
        },
        details: HashMap::from([
            ("available".to_string(), git_status.0.to_string()),
            ("version".to_string(), git_status.1.trim().to_string()),
        ]),
        last_check: SystemTime::now(),
        response_time: Some(git_status.2),
    });

    // Check Node.js (if working with JS/TS projects)
    let node_status = check_command_availability("node", &["--version"]).await;
    components.push(ComponentStatus {
        name: "Node.js".to_string(),
        status: if node_status.0 {
            StatusLevel::Healthy
        } else {
            StatusLevel::Warning
        },
        message: if node_status.0 {
            format!("Node.js available: {}", node_status.1.trim())
        } else {
            "Node.js not available".to_string()
        },
        details: HashMap::from([
            ("available".to_string(), node_status.0.to_string()),
            ("version".to_string(), node_status.1.trim().to_string()),
        ]),
        last_check: SystemTime::now(),
        response_time: Some(node_status.2),
    });

    // Check Python (if working with Python projects)
    let python_status = check_command_availability("python3", &["--version"]).await;
    components.push(ComponentStatus {
        name: "Python".to_string(),
        status: if python_status.0 {
            StatusLevel::Healthy
        } else {
            StatusLevel::Warning
        },
        message: if python_status.0 {
            format!("Python available: {}", python_status.1.trim())
        } else {
            "Python not available".to_string()
        },
        details: HashMap::from([
            ("available".to_string(), python_status.0.to_string()),
            ("version".to_string(), python_status.1.trim().to_string()),
        ]),
        last_check: SystemTime::now(),
        response_time: Some(python_status.2),
    });

    Ok(components)
}

/// Check if a command is available
async fn check_command_availability(command: &str, args: &[&str]) -> (bool, String, Duration) {
    let start_time = Instant::now();

    match std::process::Command::new(command).args(args).output() {
        Ok(output) => {
            let duration = start_time.elapsed();
            if output.status.success() {
                (
                    true,
                    String::from_utf8_lossy(&output.stdout).to_string(),
                    duration,
                )
            } else {
                (
                    false,
                    String::from_utf8_lossy(&output.stderr).to_string(),
                    duration,
                )
            }
        }
        Err(_) => {
            let duration = start_time.elapsed();
            (false, format!("{} not found", command), duration)
        }
    }
}

/// Calculate overall system status
fn calculate_overall_status(components: &[ComponentStatus]) -> StatusLevel {
    if components.iter().any(|c| c.status == StatusLevel::Error) {
        StatusLevel::Error
    } else if components.iter().any(|c| c.status == StatusLevel::Warning) {
        StatusLevel::Warning
    } else if components.iter().all(|c| c.status == StatusLevel::Healthy) {
        StatusLevel::Healthy
    } else {
        StatusLevel::Unknown
    }
}

/// Get performance metrics
async fn get_performance_metrics(
    runner: &mut CliRunner,
) -> Result<PerformanceMetrics, Box<dyn std::error::Error>> {
    // This is a simplified implementation - in a real system you'd use proper system monitoring
    let memory_usage = get_memory_usage();
    let cpu_usage = get_cpu_usage();
    let disk_usage = get_disk_usage(&std::env::current_dir()?)?;

    let active_agents = if let Some(agent_system) = &runner.agent_system {
        agent_system.get_agent_statuses().await.len()
    } else {
        0
    };

    Ok(PerformanceMetrics {
        memory_usage,
        cpu_usage,
        disk_usage,
        active_agents,
        context_cache_size: 0,                         // Placeholder
        avg_response_time: Duration::from_millis(100), // Placeholder
    })
}

/// Get memory usage (simplified)
fn get_memory_usage() -> u64 {
    // This is a placeholder - you'd use a proper system monitoring library
    1024 * 1024 * 64 // 64MB placeholder
}

/// Get CPU usage (simplified)
fn get_cpu_usage() -> f32 {
    // This is a placeholder - you'd use a proper system monitoring library
    15.5 // 15.5% placeholder
}

/// Get disk usage for a path
fn get_disk_usage(path: &std::path::Path) -> Result<u64, Box<dyn std::error::Error>> {
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.len())
}

/// Get system information
fn get_system_info() -> SystemInfo {
    let version = env!("CARGO_PKG_VERSION");
    let git_hash = env!("BUILD_GIT_HASH");
    let build_time = env!("BUILD_TIMESTAMP");
    let enhanced_version = format!("{} (git:{}, built:{})", version, git_hash, build_time);
    
    SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        rust_version: env!("CARGO_PKG_RUST_VERSION").to_string(),
        agentic_version: enhanced_version,
        working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("unknown")),
        config_file: std::env::var("AGENTIC_CONFIG").ok().map(PathBuf::from),
    }
}

/// Display status in text format
fn display_text_status(
    runner: &CliRunner,
    status: &SystemStatus,
    detailed: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Overall status header
    runner.print_output(&format!("\n🔍 System Status Report\n"), None);
    runner.print_output(&format!("═══════════════════════\n"), None);

    let overall_emoji = status.overall_status.as_emoji();
    let overall_text = status.overall_status.as_string();
    runner.print_output(
        &format!("{} Overall Status: {}\n", overall_emoji, overall_text),
        None,
    );

    if let Ok(_duration) = status.timestamp.duration_since(std::time::UNIX_EPOCH) {
        runner.print_output(
            &format!(
                "🕐 Last Check: {}\n\n",
                chrono::DateTime::<chrono::Utc>::from(status.timestamp)
                    .format("%Y-%m-%d %H:%M:%S UTC")
            ),
            None,
        );
    }

    // Component statuses
    runner.print_output("📋 Component Status:\n", None);
    runner.print_output("───────────────────\n", None);

    for component in &status.components {
        let emoji = component.status.as_emoji();
        let status_text = component.status.as_string();

        runner.print_output(
            &format!(
                "{} {} - {} ({})\n",
                emoji, component.name, component.message, status_text
            ),
            None,
        );

        if detailed {
            for (key, value) in &component.details {
                runner.print_output(&format!("    {} = {}\n", key, value), None);
            }

            if let Some(response_time) = component.response_time {
                runner.print_output(&format!("    Response Time: {:?}\n", response_time), None);
            }
        }
    }

    // Performance metrics
    if let Some(metrics) = &status.performance_metrics {
        runner.print_output("\n📊 Performance Metrics:\n", None);
        runner.print_output("────────────────────────\n", None);
        runner.print_output(
            &format!(
                "💾 Memory Usage: {}\n",
                format_file_size(metrics.memory_usage)
            ),
            None,
        );
        runner.print_output(&format!("🔥 CPU Usage: {:.1}%\n", metrics.cpu_usage), None);
        runner.print_output(
            &format!("💿 Disk Usage: {}\n", format_file_size(metrics.disk_usage)),
            None,
        );
        runner.print_output(
            &format!("🤖 Active Agents: {}\n", metrics.active_agents),
            None,
        );
        runner.print_output(
            &format!("📦 Context Cache: {} items\n", metrics.context_cache_size),
            None,
        );
        runner.print_output(
            &format!("⚡ Avg Response Time: {:?}\n", metrics.avg_response_time),
            None,
        );
    }

    // System information
    if detailed {
        runner.print_output("\n💻 System Information:\n", None);
        runner.print_output("─────────────────────\n", None);
        runner.print_output(
            &format!(
                "OS: {} ({})\n",
                status.system_info.os, status.system_info.arch
            ),
            None,
        );
        runner.print_output(
            &format!("Rust Version: {}\n", status.system_info.rust_version),
            None,
        );
        runner.print_output(
            &format!("Agentic Version: {}\n", status.system_info.agentic_version),
            None,
        );
        runner.print_output(
            &format!(
                "Working Directory: {}\n",
                status.system_info.working_directory.display()
            ),
            None,
        );

        if let Some(config_file) = &status.system_info.config_file {
            runner.print_output(&format!("Config File: {}\n", config_file.display()), None);
        }
    }

    runner.print_output("\n", None);

    Ok(())
}

/// Display status in table format
fn display_table_status(
    runner: &CliRunner,
    status: &SystemStatus,
    detailed: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // For now, fall back to text format
    // In a real implementation, you'd use a table formatting library
    display_text_status(runner, status, detailed)
}

/// Display status in JSON format
fn display_json_status(status: &SystemStatus) -> Result<(), Box<dyn std::error::Error>> {
    let json_output = serde_json::json!({
        "overall_status": status.overall_status.as_string(),
        "timestamp": status.timestamp.duration_since(std::time::UNIX_EPOCH)?.as_secs(),
        "components": status.components.iter().map(|c| serde_json::json!({
            "name": c.name,
            "status": c.status.as_string(),
            "message": c.message,
            "details": c.details,
            "response_time_ms": c.response_time.map(|d| d.as_millis())
        })).collect::<Vec<_>>(),
        "performance_metrics": status.performance_metrics.as_ref().map(|m| serde_json::json!({
            "memory_usage": m.memory_usage,
            "cpu_usage": m.cpu_usage,
            "disk_usage": m.disk_usage,
            "active_agents": m.active_agents,
            "context_cache_size": m.context_cache_size,
            "avg_response_time_ms": m.avg_response_time.as_millis()
        })),
        "system_info": serde_json::json!({
            "os": status.system_info.os,
            "arch": status.system_info.arch,
            "rust_version": status.system_info.rust_version,
            "agentic_version": status.system_info.agentic_version,
            "working_directory": status.system_info.working_directory,
            "config_file": status.system_info.config_file
        })
    });

    println!("{}", serde_json::to_string_pretty(&json_output)?);

    Ok(())
}
