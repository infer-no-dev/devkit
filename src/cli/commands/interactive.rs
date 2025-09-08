use crate::cli::{CliRunner, InteractiveArgs};
use crate::ui::{Application, UIConfig, UIEvent};
use crate::agents::AgentSystem;
use crate::interactive::{InteractiveSession, ConversationEntry, ConversationRole, EntryType};
use crate::ui::notifications::Notification;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Duration};
use uuid::Uuid;

pub async fn run(runner: &mut CliRunner, _args: InteractiveArgs) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info("Starting interactive mode...");
    
    // Ensure we have necessary managers
    runner.ensure_context_manager().await?;
    
    // Create interactive session
    let project_path = std::env::current_dir().ok();
    let session = InteractiveSession::new(project_path);
    
    // Create UI configuration
    let ui_config = UIConfig::new()
        .with_theme("Dark".to_string())
        .with_tick_rate(Duration::from_millis(50));
    
    // Initialize UI application
    let mut app = Application::new(ui_config)?;
    
    // Create and initialize agent system
    let agent_system = Arc::new(AgentSystem::new());
    agent_system.initialize().await;
    
    // Create communication channels
    let (ui_tx, _ui_rx) = mpsc::unbounded_channel::<UIEvent>();
    let (command_tx, command_rx) = mpsc::unbounded_channel::<String>();
    
    // Connect command sender to UI
    app.set_command_sender(command_tx.clone());
    
    runner.print_info("Interactive mode initialized. Starting UI...");
    
    // Add initial system notification
    let welcome_notification = Notification::info(
        "Welcome to Agentic Dev Environment".to_string(),
        "Interactive mode is now active. Type commands to interact with AI agents.".to_string(),
    );
    app.add_notification(welcome_notification);
    
    // Create interactive manager to handle the session
    let interactive_manager = InteractiveManager::new(
        session,
        agent_system.clone(),
        None, // TODO: Fix context manager integration
        ui_tx.clone(),
        command_tx,
    );
    
    // Spawn background tasks
    let agent_monitor = spawn_agent_monitor(agent_system.clone(), ui_tx.clone());
    let command_processor = spawn_command_processor(interactive_manager, command_rx);
    
    // Run the main UI event loop
    let ui_result = tokio::select! {
        result = app.run() => result,
        _ = tokio::signal::ctrl_c() => {
            runner.print_info("Received shutdown signal");
            Ok(())
        }
    };
    
    // Cleanup background tasks
    agent_monitor.abort();
    command_processor.abort();
    
    match ui_result {
        Ok(_) => {
            runner.print_info("Interactive mode ended successfully");
            Ok(())
        }
        Err(e) => {
            runner.print_error(&format!("Interactive mode error: {}", e));
            Err(e.into())
        }
    }
}

/// Interactive manager to handle session state and agent communication
struct InteractiveManager {
    session: Arc<RwLock<InteractiveSession>>,
    agent_system: Arc<AgentSystem>,
    context_manager: Option<crate::context::ContextManager>,
    ui_sender: mpsc::UnboundedSender<UIEvent>,
    command_sender: mpsc::UnboundedSender<String>,
}

impl InteractiveManager {
    fn new(
        session: InteractiveSession,
        agent_system: Arc<AgentSystem>,
        context_manager: Option<crate::context::ContextManager>,
        ui_sender: mpsc::UnboundedSender<UIEvent>,
        command_sender: mpsc::UnboundedSender<String>,
    ) -> Arc<Self> {
        Arc::new(Self {
            session: Arc::new(RwLock::new(session)),
            agent_system,
            context_manager,
            ui_sender,
            command_sender,
        })
    }
    
    /// Process a user command
    async fn process_command(&self, command: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Add user input to session history
        let entry = ConversationEntry {
            timestamp: std::time::SystemTime::now(),
            role: ConversationRole::User,
            content: command.clone(),
            result: None,
            entry_type: self.classify_command(&command),
        };
        
        {
            let mut session = self.session.write().await;
            session.add_history_entry(entry);
        }
        
        // Send user input to UI
        let _ = self.ui_sender.send(UIEvent::Output {
            content: command.clone(),
            block_type: "user".to_string(),
        });
        
        // Process different types of commands
        let response = if command.starts_with("/") {
            self.process_system_command(&command[1..]).await?
        } else {
            self.process_natural_language_command(&command).await?
        };
        
        // Add response to session history
        let response_entry = ConversationEntry {
            timestamp: std::time::SystemTime::now(),
            role: ConversationRole::Assistant,
            content: response.clone(),
            result: None,
            entry_type: EntryType::Chat,
        };
        
        {
            let mut session = self.session.write().await;
            session.add_history_entry(response_entry);
        }
        
        // Send response to UI
        let _ = self.ui_sender.send(UIEvent::Output {
            content: response,
            block_type: "agent".to_string(),
        });
        
        Ok(())
    }
    
    /// Classify command type for better processing
    fn classify_command(&self, command: &str) -> EntryType {
        let lower = command.to_lowercase();
        
        if lower.contains("generate") || lower.contains("create") || lower.contains("write") {
            EntryType::Generate
        } else if lower.contains("test") {
            EntryType::AddTests
        } else if lower.contains("debug") || lower.contains("fix") {
            EntryType::Debug
        } else if lower.contains("explain") || lower.contains("what") || lower.contains("how") {
            EntryType::Explain
        } else if lower.contains("optimize") || lower.contains("improve") {
            EntryType::Optimize
        } else if lower.contains("refine") || lower.contains("modify") {
            EntryType::Refine
        } else {
            EntryType::Chat
        }
    }
    
    /// Process system commands (starting with /)
    async fn process_system_command(&self, command: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd = parts.get(0).unwrap_or(&"");
        
        match *cmd {
            "help" => Ok(self.get_help_text()),
            "status" => Ok(self.get_status().await),
            "agents" => Ok(self.list_agents().await),
            "clear" => {
                let _ = self.ui_sender.send(UIEvent::Output {
                    content: "\n\n\n\n\n\n\n\n\n\n".to_string(),
                    block_type: "system".to_string(),
                });
                Ok("Screen cleared".to_string())
            }
            "save" => {
                let filename = parts.get(1).map_or("session.json", |v| *v);
                let session = self.session.read().await;
                match session.save_to_file(std::path::PathBuf::from(filename)) {
                    Ok(_) => Ok(format!("Session saved to {}", filename)),
                    Err(e) => Ok(format!("Failed to save session: {}", e)),
                }
            }
            "quit" | "exit" => {
                let _ = self.ui_sender.send(UIEvent::Quit);
                Ok("Goodbye!".to_string())
            }
            _ => Ok(format!("Unknown command: /{}. Type /help for available commands.", cmd)),
        }
    }
    
    /// Process natural language commands through agents
    async fn process_natural_language_command(&self, command: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use crate::agents::{AgentTask, TaskPriority};
        
        // Classify the command and route to appropriate agent
        let task_type = match self.classify_command(command) {
            EntryType::Generate => "generate_code",
            EntryType::AddTests => "generate_tests",
            EntryType::Debug => "debug_issue",
            EntryType::Explain => "analyze_code",
            EntryType::Optimize => "optimize_code",
            EntryType::Refine => "refactor_code",
            _ => "general_chat",
        };
        
        // Create agent task
        let task = AgentTask {
            id: format!("task_{}", Uuid::new_v4()),
            task_type: task_type.to_string(),
            description: command.to_string(),
            context: serde_json::json!({
                "user_input": command,
                "session_id": "interactive",
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            }),
            priority: TaskPriority::Normal,
        };
        
        // Send task to agent system
        match self.agent_system.submit_task(task).await {
            Ok(result) => {
                // Send agent status update to UI
                let _ = self.ui_sender.send(UIEvent::AgentStatusUpdate {
                    agent_name: "Processing".to_string(),
                    status: crate::agents::AgentStatus::Idle,
                    task: None,
                    priority: None,
                    progress: None,
                });
                
                Ok(format!("Agent Response: {}\n\nArtifacts generated: {}", 
                    result.output,
                    result.artifacts.len()))
            }
            Err(e) => {
                Ok(format!("Agent processing failed: {}", e))
            }
        }
    }
    
    /// Get help text for commands
    fn get_help_text(&self) -> String {
        r#"Available Commands:
  /help     - Show this help message
  /status   - Show system status
  /agents   - List active agents
  /clear    - Clear screen
  /save [filename] - Save session to file
  /quit     - Exit interactive mode

Natural Language Commands:
  - "generate a function to..."
  - "explain this code"
  - "optimize this algorithm"
  - "add tests for..."
  - "debug this issue"

Press Ctrl+C to exit at any time."#.to_string()
    }
    
    /// Get system status
    async fn get_status(&self) -> String {
        let session = self.session.read().await;
        format!("Session ID: {}\nProject: {:?}\nHistory entries: {}\nArtifacts: {}", 
            session.session_id,
            session.project_path,
            session.history.len(),
            session.artifacts.len()
        )
    }
    
    /// List active agents
    async fn list_agents(&self) -> String {
        let agents_info = self.agent_system.get_agents_info().await;
        
        if agents_info.is_empty() {
            return "No agents currently active".to_string();
        }
        
        let mut output = String::from("Active Agents:\n");
        for (i, agent_info) in agents_info.iter().enumerate() {
            output.push_str(&format!("{}. {} ({}) - Status: {:?}\n",
                i + 1,
                agent_info.name,
                agent_info.agent_type,
                agent_info.status
            ));
            
            if !agent_info.capabilities.is_empty() {
                output.push_str(&format!("   Capabilities: {}\n", agent_info.capabilities.join(", ")));
            }
        }
        
        output
    }
}

/// Spawn agent monitoring task
fn spawn_agent_monitor(
    agent_system: Arc<AgentSystem>, 
    ui_sender: mpsc::UnboundedSender<UIEvent>
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_millis(500));
        
        loop {
            interval.tick().await;
            
            // Get real-time agent status updates
            let agents_info = agent_system.get_agents_info().await;
            
            for agent_info in agents_info {
                let _ = ui_sender.send(UIEvent::AgentStatusUpdate {
                    agent_name: agent_info.name.clone(),
                    status: agent_info.status.clone(),
                    task: None, // TODO: Add current task info
                    priority: None, // TODO: Add priority info
                    progress: None, // TODO: Add progress tracking
                });
            }
            
            // Send periodic system status
            let agent_statuses = agent_system.get_agent_statuses().await;
            if !agent_statuses.is_empty() {
                let status_summary = agent_statuses.iter()
                    .map(|(name, status)| format!("{}: {:?}", name, status))
                    .collect::<Vec<_>>()
                    .join(", ");
                    
                // Send agent status as notification instead
                let _ = ui_sender.send(UIEvent::Output {
                    content: format!("System Status: {}", status_summary),
                    block_type: "status".to_string(),
                });
            }
        }
    })
}

/// Spawn command processing task
fn spawn_command_processor(
    manager: Arc<InteractiveManager>,
    mut command_rx: mpsc::UnboundedReceiver<String>
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(command) = command_rx.recv().await {
            if let Err(e) = manager.process_command(command).await {
                eprintln!("Error processing command: {}", e);
            }
        }
    })
}
