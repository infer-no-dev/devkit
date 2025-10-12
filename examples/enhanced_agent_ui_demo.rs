//! Enhanced Agent with UI Integration Demo
//! 
//! This example demonstrates how the enhanced agent system integrates with
//! the DevKit UI to provide real-time progress feedback during task execution.

use devkit::{
    agents::{
        Agent, AgentTask, AgentProgressTracker, EnhancedCodeGenAgent, CodeGenConfig,
        TaskPriority,
    },
    ui::{
        progress::{ProgressManager, ProgressStyle},
        error_handler::{ErrorHandler, UIError, UIErrorSeverity},
        enhanced_panels::{
            EnhancedPanelManager, ContentType, AgentStatusInfo, AgentOutputInfo,
            SystemStatusInfo, LogEntry, LogLevel,
        },
    },
};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};

use std::{
    collections::HashMap,
    io,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::time::sleep;
use tracing::{info, debug, warn, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting Enhanced Agent UI Demo");

    // Initialize UI components
    let progress_manager = Arc::new(ProgressManager::new());
    let error_handler = Arc::new(ErrorHandler::new());
    let panel_manager = Arc::new(EnhancedPanelManager::new(
        progress_manager.clone(),
        error_handler.clone(),
    ));

    // Create enhanced agents with different configurations
    let agents = create_demo_agents(progress_manager.clone()).await;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the demo
    let result = run_demo(
        &mut terminal,
        agents,
        progress_manager,
        error_handler,
        panel_manager,
    ).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        error!("Demo error: {:?}", err);
        return Err(err);
    }

    info!("Enhanced Agent UI Demo completed successfully");
    Ok(())
}

/// Create demo agents with different configurations
async fn create_demo_agents(
    progress_manager: Arc<ProgressManager>,
) -> Vec<(String, EnhancedCodeGenAgent)> {
    let progress_tracker = Arc::new(AgentProgressTracker::new(progress_manager));

    vec![
        (
            "fast_agent".to_string(),
            EnhancedCodeGenAgent::with_progress_tracker(progress_tracker.clone())
                .with_config(CodeGenConfig {
                    enable_detailed_steps: true,
                    simulate_processing_time: true,
                    max_concurrent_tasks: 3,
                    quality_check_enabled: true,
                    auto_optimization: false,
                }),
        ),
        (
            "thorough_agent".to_string(),
            EnhancedCodeGenAgent::with_progress_tracker(progress_tracker.clone())
                .with_config(CodeGenConfig {
                    enable_detailed_steps: true,
                    simulate_processing_time: true,
                    max_concurrent_tasks: 1,
                    quality_check_enabled: true,
                    auto_optimization: true,
                }),
        ),
        (
            "basic_agent".to_string(),
            EnhancedCodeGenAgent::new(),
        ),
    ]
}

/// Run the interactive demo
async fn run_demo<B: Backend>(
    terminal: &mut Terminal<B>,
    mut agents: Vec<(String, EnhancedCodeGenAgent)>,
    progress_manager: Arc<ProgressManager>,
    error_handler: Arc<ErrorHandler>,
    panel_manager: Arc<EnhancedPanelManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    let mut task_counter = 0;
    let mut demo_stage = DemoStage::Welcome;

    // Demo tasks to execute
    let demo_tasks = vec![
        ("generate_code", "Create a JSON parser with error handling"),
        ("refactor", "Refactor existing authentication module"),
        ("documentation", "Generate API documentation for user service"),
        ("generate_code", "Implement async file processing pipeline"),
        ("optimization", "Optimize database query performance"),
    ];

    loop {
        // Update UI with current status
        update_ui_state(
            &panel_manager,
            &agents,
            &progress_manager,
            &error_handler,
            &demo_stage,
            start_time.elapsed(),
        ).await;

        // Render the UI
        terminal.draw(|f| {
            panel_manager.render(f, f.size());
        })?;

        // Handle events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => {
                            info!("User requested quit");
                            break;
                        }
                        KeyCode::Char('r') => {
                            // Run next task
                            if task_counter < demo_tasks.len() {
                                let (task_type, description) = demo_tasks[task_counter];
                                task_counter += 1;
                                demo_stage = DemoStage::ProcessingTask {
                                    task_number: task_counter,
                                    total_tasks: demo_tasks.len(),
                                };

                                // Select agent based on task complexity
                                let agent_index = match task_type {
                                    "optimization" => 1, // thorough agent
                                    "generate_code" if description.contains("async") => 1, // thorough agent
                                    _ => 0, // fast agent
                                };

                                let task = create_demo_task(task_type, description, task_counter);
                                
                                // Process task asynchronously
                                let agent = &mut agents[agent_index].1;
                                tokio::spawn(async move {
                                    match agent.process_task(task).await {
                                        Ok(result) => {
                                            info!("Task completed successfully: {}", result.message);
                                        }
                                        Err(e) => {
                                            error!("Task failed: {}", e);
                                        }
                                    }
                                });
                            } else {
                                demo_stage = DemoStage::Completed;
                            }
                        }
                        KeyCode::Char('e') => {
                            // Simulate an error
                            error_handler.add_error(UIError::new(
                                "demo_error".to_string(),
                                "Simulated error for demonstration".to_string(),
                                UIErrorSeverity::Warning,
                                Some("This is a demo error to show error handling".to_string()),
                            ));
                        }
                        KeyCode::Char('c') => {
                            // Clear errors
                            // Note: ErrorHandler doesn't have a public clear method,
                            // but errors will naturally expire based on their timestamp
                            info!("Error clearing requested (errors will auto-expire)");
                        }
                        KeyCode::Char('s') => {
                            // Show agent status
                            demo_stage = DemoStage::ShowingStatus;
                        }
                        KeyCode::Char('h') => {
                            // Show help
                            demo_stage = DemoStage::ShowingHelp;
                        }
                        _ => {}
                    }
                }
            }
        }

        // Auto-advance demo after welcome screen
        if let DemoStage::Welcome = demo_stage {
            sleep(Duration::from_secs(2)).await;
            demo_stage = DemoStage::Ready;
        }

        // Check if all tasks are completed
        if task_counter >= demo_tasks.len() && matches!(demo_stage, DemoStage::ProcessingTask { .. }) {
            sleep(Duration::from_secs(1)).await;
            demo_stage = DemoStage::Completed;
        }
    }

    info!("Demo finished, processed {} tasks", task_counter);
    Ok(())
}

/// Update the UI state with current information
async fn update_ui_state(
    panel_manager: &Arc<EnhancedPanelManager>,
    agents: &[(String, EnhancedCodeGenAgent)],
    progress_manager: &Arc<ProgressManager>,
    error_handler: &Arc<ErrorHandler>,
    demo_stage: &DemoStage,
    uptime: Duration,
) {
    // Update agent status information
    for (agent_name, agent) in agents {
        let status_info = AgentStatusInfo {
            agent_id: agent.id().to_string(),
            agent_name: agent.name().to_string(),
            status: agent.status(),
            capabilities: agent.capabilities(),
            metrics: agent.get_metrics(),
            current_task: None, // Would be populated from actual task tracking
        };
        
        panel_manager.update_content(
            format!("agent_status_{}", agent_name),
            ContentType::AgentStatus(status_info),
        ).await;
    }

    // Update system status
    let system_status = SystemStatusInfo {
        uptime: uptime,
        memory_usage: get_mock_memory_usage(),
        cpu_usage: get_mock_cpu_usage(),
        active_tasks: progress_manager.get_all_progress().await.len(),
        completed_tasks: agents.iter()
            .map(|(_, agent)| agent.get_metrics().tasks_completed)
            .sum(),
        error_count: 0, // Would get from error_handler if it had a count method
        system_health: "Healthy".to_string(),
    };

    panel_manager.update_content(
        "system_status".to_string(),
        ContentType::SystemStatus(system_status),
    ).await;

    // Add stage-specific log entries
    match demo_stage {
        DemoStage::Welcome => {
            panel_manager.update_content(
                "welcome_log".to_string(),
                ContentType::Log(LogEntry {
                    timestamp: std::time::SystemTime::now(),
                    level: LogLevel::Info,
                    source: "Demo".to_string(),
                    message: "Enhanced Agent UI Demo starting...".to_string(),
                    metadata: HashMap::new(),
                }),
            ).await;
        }
        DemoStage::Ready => {
            panel_manager.update_content(
                "ready_log".to_string(),
                ContentType::Log(LogEntry {
                    timestamp: std::time::SystemTime::now(),
                    level: LogLevel::Info,
                    source: "Demo".to_string(),
                    message: "Demo ready! Press 'r' to run tasks, 'q' to quit".to_string(),
                    metadata: HashMap::new(),
                }),
            ).await;
        }
        DemoStage::ProcessingTask { task_number, total_tasks } => {
            panel_manager.update_content(
                format!("task_log_{}", task_number),
                ContentType::Log(LogEntry {
                    timestamp: std::time::SystemTime::now(),
                    level: LogLevel::Info,
                    source: "TaskManager".to_string(),
                    message: format!("Processing task {} of {}", task_number, total_tasks),
                    metadata: HashMap::new(),
                }),
            ).await;
        }
        DemoStage::Completed => {
            panel_manager.update_content(
                "completion_log".to_string(),
                ContentType::Log(LogEntry {
                    timestamp: std::time::SystemTime::now(),
                    level: LogLevel::Info,
                    source: "Demo".to_string(),
                    message: "All demo tasks completed successfully!".to_string(),
                    metadata: HashMap::new(),
                }),
            ).await;
        }
        _ => {}
    }
}

/// Create a demo task
fn create_demo_task(task_type: &str, description: &str, task_number: usize) -> AgentTask {
    let mut task = AgentTask::new(
        task_type.to_string(),
        description.to_string(),
        serde_json::json!({
            "language": "rust",
            "target_file": format!("demo_output_{}.rs", task_number),
            "complexity": match task_type {
                "optimization" => "high",
                "generate_code" => "medium",
                _ => "low"
            }
        }),
    );

    // Set priority based on task type
    task.priority = match task_type {
        "optimization" => TaskPriority::High,
        "generate_code" => TaskPriority::Normal,
        _ => TaskPriority::Low,
    };

    task
}

/// Get mock memory usage for demo
fn get_mock_memory_usage() -> f64 {
    // Simulate varying memory usage between 30-80%
    30.0 + (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() % 50) as f64
}

/// Get mock CPU usage for demo
fn get_mock_cpu_usage() -> f64 {
    // Simulate varying CPU usage between 10-60%
    10.0 + (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() % 50) as f64
}

/// Demo stages to control flow
#[derive(Debug)]
enum DemoStage {
    Welcome,
    Ready,
    ProcessingTask { task_number: usize, total_tasks: usize },
    ShowingStatus,
    ShowingHelp,
    Completed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_demo_agent_creation() {
        let progress_manager = Arc::new(ProgressManager::new());
        let agents = create_demo_agents(progress_manager).await;
        
        assert_eq!(agents.len(), 3);
        assert!(agents.iter().any(|(name, _)| name == "fast_agent"));
        assert!(agents.iter().any(|(name, _)| name == "thorough_agent"));
        assert!(agents.iter().any(|(name, _)| name == "basic_agent"));
    }

    #[test]
    fn test_demo_task_creation() {
        let task = create_demo_task("generate_code", "Test description", 1);
        
        assert_eq!(task.task_type, "generate_code");
        assert_eq!(task.description, "Test description");
        assert_eq!(task.priority, TaskPriority::Normal);
    }

    #[test]
    fn test_mock_system_metrics() {
        let memory = get_mock_memory_usage();
        let cpu = get_mock_cpu_usage();
        
        assert!(memory >= 30.0 && memory <= 80.0);
        assert!(cpu >= 10.0 && cpu <= 60.0);
    }
}