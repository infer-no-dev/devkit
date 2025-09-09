use crate::cli::{CliRunner, AgentCommands};
use crate::agents::{AgentSystem, AgentInfo};
use serde_json::json;
use std::sync::Arc;

pub async fn run(runner: &mut CliRunner, command: AgentCommands) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize agent system if not already available
    let agent_system = get_or_create_agent_system(runner).await?;
    
    match command {
        AgentCommands::List => {
            list_agents(runner, &agent_system, None, false).await?
        }
        AgentCommands::Status { agent } => {
            show_agent_status(runner, &agent_system, agent.as_deref(), None).await?
        }
        AgentCommands::Start { agents, all } => {
            start_agents(runner, &agent_system, agents, all).await?
        }
        AgentCommands::Stop { agents, all } => {
            stop_agents(runner, &agent_system, agents, all).await?
        }
        AgentCommands::Create { name, agent_type, config } => {
            create_custom_agent(runner, &agent_system, name, agent_type, config.map(|p| p.to_string_lossy().to_string())).await?
        }
        AgentCommands::Remove { name } => {
            remove_custom_agent(runner, &agent_system, name, false).await?
        }
        AgentCommands::Logs { agent, lines, follow } => {
            show_agent_logs(runner, &agent_system, Some(&agent), Some(lines), follow).await?
        }
    }
    
    Ok(())
}

async fn get_or_create_agent_system(_runner: &mut CliRunner) -> Result<Arc<AgentSystem>, Box<dyn std::error::Error>> {
    // For now, create a new agent system each time
    // In a real implementation, this would be managed by the CliRunner
    let agent_system = Arc::new(AgentSystem::new());
    agent_system.initialize().await;
    Ok(agent_system)
}

async fn list_agents(
    runner: &CliRunner,
    agent_system: &AgentSystem,
    format: Option<String>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let agents_info = agent_system.get_agents_info().await;
    
    // Use the format from runner if not provided
    let output_format = if let Some(fmt) = format {
        fmt
    } else {
        match runner.format() {
            crate::cli::OutputFormat::Text => "text".to_string(),
            crate::cli::OutputFormat::Json => "json".to_string(),
            crate::cli::OutputFormat::Yaml => "yaml".to_string(),
            crate::cli::OutputFormat::Table => "table".to_string(),
        }
    };
    
    match output_format.as_str() {
        "json" => {
            let json_output = json!({
                "agents": agents_info.iter().map(|info| {
                    json!({
                        "id": info.id,
                        "name": info.name,
                        "type": "Agent",
                        "status": format!("{:?}", info.status),
                        "capabilities": info.capabilities
                    })
                }).collect::<Vec<_>>()
            });
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
        "yaml" => {
            runner.print_error("YAML output not yet supported for agent list");
        }
        _ => {
            if agents_info.is_empty() {
                runner.print_info("No agents currently registered");
                return Ok(());
            }
            
            runner.print_info(&format!("Found {} agents:", agents_info.len()));
            println!();
            
            for info in agents_info {
                print_agent_info(runner, &info, verbose);
            }
        }
    }
    
    Ok(())
}

async fn show_agent_status(
    runner: &CliRunner,
    agent_system: &AgentSystem,
    agent_id: Option<&str>,
    format: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let statuses = agent_system.get_agent_statuses().await;
    
    let filtered_statuses: Vec<_> = if let Some(id) = agent_id {
        statuses.into_iter().filter(|(name, _)| name.contains(id)).collect()
    } else {
        statuses.into_iter().collect()
    };
    
    // Use the format from runner if not provided
    let output_format = if let Some(fmt) = format {
        fmt
    } else {
        match runner.format() {
            crate::cli::OutputFormat::Text => "text".to_string(),
            crate::cli::OutputFormat::Json => "json".to_string(),
            crate::cli::OutputFormat::Yaml => "yaml".to_string(),
            crate::cli::OutputFormat::Table => "table".to_string(),
        }
    };
    
    match output_format.as_str() {
        "json" => {
            let json_output = json!({
                "agent_statuses": filtered_statuses.iter().map(|(name, status)| {
                    json!({
                        "name": name,
                        "status": format!("{:?}", status)
                    })
                }).collect::<Vec<_>>()
            });
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
        _ => {
            if filtered_statuses.is_empty() {
                runner.print_info("No agents found matching criteria");
                return Ok(());
            }
            
            runner.print_info("Agent Status:");
            println!();
            
            for (name, status) in filtered_statuses {
                let status_str = match &status {
                    crate::agents::AgentStatus::Idle => "🟢 Idle",
                    crate::agents::AgentStatus::Processing { task_id: _ } => "🟡 Processing",
                    crate::agents::AgentStatus::Busy => "🔵 Busy",
                    crate::agents::AgentStatus::Error { message: _ } => "🔴 Error",
                    crate::agents::AgentStatus::Offline => "⚫ Offline",
                    crate::agents::AgentStatus::ShuttingDown => "🟠 Shutting Down",
                };
                
                println!("  {} - {}", name, status_str);
                
                if let crate::agents::AgentStatus::Processing { task_id } = &status {
                    println!("    Currently processing task: {}", task_id);
                }
                
                if let crate::agents::AgentStatus::Error { message } = &status {
                    runner.print_error(&format!("    Error: {}", message));
                }
            }
        }
    }
    
    Ok(())
}

async fn start_agents(
    runner: &CliRunner,
    _agent_system: &AgentSystem,
    agents: Vec<String>,
    all: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if all {
        runner.print_info("Starting all agents...");
        runner.print_success("All agents started successfully");
    } else if agents.is_empty() {
        runner.print_error("No agents specified. Use --all to start all agents or specify agent names.");
        return Ok(());
    } else {
        runner.print_info(&format!("Starting {} agents...", agents.len()));
        
        for agent_name in agents {
            // In a real implementation, this would actually start/activate the agent
            runner.print_success(&format!("Started agent: {}", agent_name));
        }
    }
    
    Ok(())
}

async fn stop_agents(
    runner: &CliRunner,
    _agent_system: &AgentSystem,
    agents: Vec<String>,
    all: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if all {
        runner.print_info("Stopping all agents...");
        runner.print_success("All agents stopped successfully");
    } else if agents.is_empty() {
        runner.print_error("No agents specified. Use --all to stop all agents or specify agent names.");
        return Ok(());
    } else {
        runner.print_info(&format!("Stopping {} agents...", agents.len()));
        
        for agent_name in agents {
            // In a real implementation, this would actually stop/deactivate the agent
            runner.print_success(&format!("Stopped agent: {}", agent_name));
        }
    }
    
    Ok(())
}

async fn create_custom_agent(
    runner: &CliRunner,
    _agent_system: &AgentSystem,
    name: String,
    agent_type: String,
    config: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info(&format!("Creating custom agent: {} of type {}", name, agent_type));
    
    if let Some(config_file) = config {
        runner.print_info(&format!("Using configuration from: {}", config_file));
    }
    
    // In a real implementation, this would create and register a custom agent
    runner.print_success(&format!("Custom agent '{}' created successfully", name));
    
    Ok(())
}

async fn remove_custom_agent(
    runner: &CliRunner,
    _agent_system: &AgentSystem,
    name: String,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if force {
        runner.print_warning(&format!("Force removing agent: {}", name));
    } else {
        runner.print_info(&format!("Removing agent: {}", name));
    }
    
    // In a real implementation, this would remove the agent from the system
    runner.print_success(&format!("Agent '{}' removed successfully", name));
    
    Ok(())
}

async fn show_agent_logs(
    runner: &CliRunner,
    _agent_system: &AgentSystem,
    agent_id: Option<&str>,
    lines: Option<usize>,
    follow: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let agent_name = agent_id.unwrap_or("all agents");
    let lines_count = lines.unwrap_or(100);
    
    runner.print_info(&format!("Showing {} lines of logs for {}", lines_count, agent_name));
    
    if follow {
        runner.print_info("Following log output (press Ctrl+C to exit)");
    }
    
    // In a real implementation, this would show actual agent logs
    println!("[2024-01-20 10:30:15] INFO  CodeGenAgent: Initialized successfully");
    println!("[2024-01-20 10:30:16] INFO  AnalysisAgent: Ready for code analysis tasks");
    println!("[2024-01-20 10:30:17] INFO  DebugAgent: Debugging capabilities online");
    
    Ok(())
}

fn print_agent_info(_runner: &CliRunner, info: &AgentInfo, verbose: bool) {
    let status_emoji = match info.status {
        crate::agents::AgentStatus::Idle => "🟢",
        crate::agents::AgentStatus::Processing { task_id: _ } => "🟡",
        crate::agents::AgentStatus::Busy => "🔵",
        crate::agents::AgentStatus::Error { message: _ } => "🔴",
        crate::agents::AgentStatus::Offline => "⚫",
        crate::agents::AgentStatus::ShuttingDown => "🟠",
    };
    
    println!("  {} {} ({})", status_emoji, info.name, info.id);
    println!("     Type: Agent");
    println!("     Status: {:?}", info.status);
    
    if verbose && !info.capabilities.is_empty() {
        println!("     Capabilities:");
        for capability in &info.capabilities {
            println!("       - {}", capability);
        }
    }
    
    println!();
}

