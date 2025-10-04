use crate::agents::AgentSystem;
use crate::cli::{session_manager::SessionManager, CliRunner, InteractiveArgs};
use crate::interactive::{ConversationEntry, ConversationRole, EntryType, InteractiveSession};
use crate::ui::notifications::Notification;
use crate::ui::{Application, UIConfig, UIEvent};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Duration};
use uuid::Uuid;

pub async fn run(
    runner: &mut CliRunner,
    _args: InteractiveArgs,
) -> Result<(), Box<dyn std::error::Error>> {
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
    let _ = agent_system.initialize().await;

    // Create communication channels
    let (ui_tx, _ui_rx) = mpsc::unbounded_channel::<UIEvent>();
    let (command_tx, command_rx) = mpsc::unbounded_channel::<String>();

    // Connect command sender to UI
    app.set_command_sender(command_tx.clone());

    runner.print_info("Interactive mode initialized. Starting UI...");

    // Add initial system notification
    let welcome_notification = Notification::info(
        "Welcome to Devkit Interactive Mode".to_string(),
        "🚀 Ready! Type commands or use natural language to interact with AI agents. Type /help for commands.".to_string(),
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
    session_manager: Arc<RwLock<SessionManager>>,
}

impl InteractiveManager {
    fn new(
        session: InteractiveSession,
        agent_system: Arc<AgentSystem>,
        context_manager: Option<crate::context::ContextManager>,
        ui_sender: mpsc::UnboundedSender<UIEvent>,
        command_sender: mpsc::UnboundedSender<String>,
    ) -> Arc<Self> {
        let mut session_manager = SessionManager::new();
        let session_id = session_manager.create_session(session.project_path.clone());

        Arc::new(Self {
            session: Arc::new(RwLock::new(session)),
            agent_system,
            context_manager,
            ui_sender,
            command_sender,
            session_manager: Arc::new(RwLock::new(session_manager)),
        })
    }

    /// Process a user command
    async fn process_command(
        &self,
        command: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    async fn process_system_command(
        &self,
        command: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
            "load" => {
                let filename = parts.get(1).ok_or("Usage: /load <filename>")?.to_string();
                match crate::interactive::InteractiveSession::load_from_file(
                    std::path::PathBuf::from(&filename),
                ) {
                    Ok(loaded_session) => {
                        *self.session.write().await = loaded_session;
                        Ok(format!("Session loaded from {}", filename))
                    }
                    Err(e) => Ok(format!("Failed to load session: {}", e)),
                }
            }
            "ls" | "list" => {
                if let Some(path_arg) = parts.get(1) {
                    self.list_directory(path_arg).await
                } else {
                    self.list_current_directory().await
                }
            }
            "cd" => {
                if let Some(path_arg) = parts.get(1) {
                    self.change_directory(path_arg).await
                } else {
                    Ok("Usage: /cd <directory>".to_string())
                }
            }
            "pwd" => match std::env::current_dir() {
                Ok(path) => Ok(format!("Current directory: {}", path.display())),
                Err(e) => Ok(format!("Failed to get current directory: {}", e)),
            },
            "history" => Ok(self.show_history().await),
            "artifacts" => Ok(self.show_artifacts().await),
            "tasks" => Ok(self.show_active_tasks().await),
            "config" => {
                if parts.len() > 2 {
                    self.update_config(&parts[1], &parts[2]).await
                } else {
                    Ok(self.show_config().await)
                }
            }
            "theme" => {
                if let Some(theme_name) = parts.get(1) {
                    let _ = self
                        .ui_sender
                        .send(UIEvent::SwitchTheme(theme_name.to_string()));
                    Ok(format!("Switched to {} theme", theme_name))
                } else {
                    Ok("Available themes: dark, light, blue, green".to_string())
                }
            }
            "sessions" => Ok(self.list_sessions().await),
            "session" => match parts.get(1).map(|&s| s) {
                Some("new") => {
                    let project_path = parts.get(2).map(|&p| std::path::PathBuf::from(p));
                    Ok(self.create_new_session(project_path).await)
                }
                Some("switch") => {
                    if let Some(&session_id) = parts.get(2) {
                        Ok(self.switch_session(session_id).await)
                    } else {
                        Ok("Usage: /session switch <session_id>".to_string())
                    }
                }
                Some("delete") => {
                    if let Some(&session_id) = parts.get(2) {
                        Ok(self.delete_session(session_id).await)
                    } else {
                        Ok("Usage: /session delete <session_id>".to_string())
                    }
                }
                Some("clone") => {
                    if let Some(&session_id) = parts.get(2) {
                        Ok(self.clone_session(session_id).await)
                    } else {
                        Ok("Usage: /session clone <session_id>".to_string())
                    }
                }
                _ => Ok("Usage: /session <new|switch|delete|clone> [args]".to_string()),
            },
            "bookmark" => match parts.get(1).map(|&s| s) {
                Some("create") => {
                    if let (Some(&name), Some(&description)) = (parts.get(2), parts.get(3)) {
                        Ok(self.create_bookmark(name, description).await)
                    } else {
                        Ok("Usage: /bookmark create <name> <description>".to_string())
                    }
                }
                Some("list") => Ok(self.list_bookmarks().await),
                Some("goto") => {
                    if let Some(&bookmark_id) = parts.get(2) {
                        Ok(self.goto_bookmark(bookmark_id).await)
                    } else {
                        Ok("Usage: /bookmark goto <bookmark_id>".to_string())
                    }
                }
                Some("delete") => {
                    if let Some(&bookmark_id) = parts.get(2) {
                        Ok(self.delete_bookmark(bookmark_id).await)
                    } else {
                        Ok("Usage: /bookmark delete <bookmark_id>".to_string())
                    }
                }
                _ => Ok("Usage: /bookmark <create|list|goto|delete> [args]".to_string()),
            },
            "layout" => match parts.get(1).map(|&s| s) {
                Some("single") => {
                    let _ = self
                        .ui_sender
                        .send(UIEvent::SetLayout("single".to_string()));
                    Ok("Switched to single panel layout".to_string())
                }
                Some("split") => {
                    let _ = self.ui_sender.send(UIEvent::SetLayout("split".to_string()));
                    Ok("Switched to split panel layout".to_string())
                }
                Some("three") => {
                    let _ = self.ui_sender.send(UIEvent::SetLayout("three".to_string()));
                    Ok("Switched to three panel layout".to_string())
                }
                Some("quad") => {
                    let _ = self.ui_sender.send(UIEvent::SetLayout("quad".to_string()));
                    Ok("Switched to quad panel layout".to_string())
                }
                _ => Ok("Available layouts: single, split, three, quad".to_string()),
            },
            "quit" | "exit" => {
                let _ = self.ui_sender.send(UIEvent::Quit);
                Ok("Goodbye!".to_string())
            }
            _ => Ok(format!(
                "Unknown command: /{}. Type /help for available commands.",
                cmd
            )),
        }
    }

    /// Process natural language commands through agents
    async fn process_natural_language_command(
        &self,
        command: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
            deadline: None,
            metadata: std::collections::HashMap::new(),
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

                Ok(format!(
                    "Agent Response: {}\n\nArtifacts generated: {}",
                    result.output,
                    result.artifacts.len()
                ))
            }
            Err(e) => Ok(format!("Agent processing failed: {}", e)),
        }
    }

    /// Get help text for commands
    fn get_help_text(&self) -> String {
        r#"🚀 Devkit Interactive Mode Commands

📁 File System:
  /ls [path]    - List directory contents
  /cd <path>    - Change directory
  /pwd          - Show current directory

💾 Session Management:
  /save [file]  - Save current session (default: session.json)
  /load <file>  - Load session from file
  /sessions     - List all available sessions
  /session new [path] - Create new session with optional project path
  /session switch <id> - Switch to a different session
  /session delete <id> - Delete a session
  /session clone <id>  - Clone an existing session
  /history      - Show conversation history
  /artifacts    - Show code artifacts

🔖 Bookmarks:
  /bookmark create <name> <description> - Create bookmark for current session
  /bookmark list - List all bookmarks
  /bookmark goto <id> - Go to bookmarked session
  /bookmark delete <id> - Delete a bookmark

🤖 Agent & System:
  /status       - Show system status
  /agents       - List active agents and capabilities
  /tasks        - Show active agent tasks
  /config [key] [value] - Show or update configuration

🎨 Interface:
  /clear        - Clear screen
  /theme [name] - Change UI theme (dark/light/blue/green)
  /layout [type] - Change layout (single/split/three/quad)
  /help         - Show this help message
  /quit         - Exit interactive mode

💬 Natural Language Commands:
  - "generate a function to..."
  - "explain this code in the current file"
  - "optimize this algorithm"
  - "add tests for the main function"
  - "debug this compilation error"
  - "refactor this code to use better patterns"

⌨️  Tips:
  - Use Tab for command completion (enhanced with new features)
  - Press Ctrl+C to exit at any time
  - Commands are case-insensitive
  - Multiple sessions allow parallel work on different projects
  - Bookmarks let you quickly return to important sessions"#
            .to_string()
    }

    /// Get system status
    async fn get_status(&self) -> String {
        let session = self.session.read().await;
        format!(
            "Session ID: {}\nProject: {:?}\nHistory entries: {}\nArtifacts: {}",
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
            output.push_str(&format!(
                "{}. {} ({}) - Status: {:?}\n",
                i + 1,
                agent_info.name,
                agent_info.id,
                agent_info.status
            ));

            if !agent_info.capabilities.is_empty() {
                output.push_str(&format!(
                    "   Capabilities: {}\n",
                    agent_info.capabilities.join(", ")
                ));
            }
        }

        output
    }

    /// List files in the current directory
    async fn list_current_directory(
        &self,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match std::fs::read_dir(".") {
            Ok(entries) => {
                let mut output = String::from("📁 Current Directory Contents:\n");
                for (i, entry) in entries.enumerate() {
                    if let Ok(entry) = entry {
                        let metadata = entry.metadata().ok();
                        let file_type = if metadata.as_ref().map_or(false, |m| m.is_dir()) {
                            "📁"
                        } else {
                            "📄"
                        };
                        let size = metadata
                            .as_ref()
                            .map(|m| format!(" ({}B)", m.len()))
                            .unwrap_or_default();

                        output.push_str(&format!(
                            "  {}{} {}{}\n",
                            file_type,
                            if i < 9 {
                                format!(" {}", i + 1)
                            } else {
                                format!("{}", i + 1)
                            },
                            entry.file_name().to_string_lossy(),
                            size
                        ));
                    }
                }
                Ok(output)
            }
            Err(e) => Ok(format!("Failed to read directory: {}", e)),
        }
    }

    /// List files in a specific directory
    async fn list_directory(
        &self,
        path: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match std::fs::read_dir(path) {
            Ok(entries) => {
                let mut output = String::from(&format!("📁 Contents of {}:\n", path));
                for entry in entries {
                    if let Ok(entry) = entry {
                        let metadata = entry.metadata().ok();
                        let file_type = if metadata.as_ref().map_or(false, |m| m.is_dir()) {
                            "📁"
                        } else {
                            "📄"
                        };
                        output.push_str(&format!(
                            "  {} {}\n",
                            file_type,
                            entry.file_name().to_string_lossy()
                        ));
                    }
                }
                Ok(output)
            }
            Err(e) => Ok(format!("Failed to read directory '{}': {}", path, e)),
        }
    }

    /// Change current directory
    async fn change_directory(
        &self,
        path: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match std::env::set_current_dir(path) {
            Ok(_) => match std::env::current_dir() {
                Ok(new_path) => Ok(format!("Changed directory to: {}", new_path.display())),
                Err(_) => Ok("Directory changed successfully".to_string()),
            },
            Err(e) => Ok(format!("Failed to change directory to '{}': {}", path, e)),
        }
    }

    /// Show conversation history
    async fn show_history(&self) -> String {
        let session = self.session.read().await;
        if session.history.is_empty() {
            return "📝 No conversation history yet.".to_string();
        }

        let mut output = String::from("📝 Conversation History:\n\n");
        let recent_entries = session.get_recent_context(10);

        for (i, entry) in recent_entries.iter().enumerate() {
            let role_icon = match entry.role {
                ConversationRole::User => "👤",
                ConversationRole::Assistant => "🤖",
                ConversationRole::System => "⚙️",
            };

            let type_badge = match entry.entry_type {
                EntryType::Generate => "[GEN]",
                EntryType::Debug => "[DEBUG]",
                EntryType::Explain => "[EXPLAIN]",
                EntryType::Optimize => "[OPT]",
                EntryType::Refine => "[REFINE]",
                EntryType::AddTests => "[TEST]",
                _ => "",
            };

            output.push_str(&format!(
                "{}. {} {} {}\n   {}\n\n",
                i + 1,
                role_icon,
                type_badge,
                entry
                    .timestamp
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                entry.content.chars().take(100).collect::<String>()
                    + if entry.content.len() > 100 { "..." } else { "" }
            ));
        }

        output.push_str(&format!(
            "\nShowing {} of {} total entries",
            recent_entries.len(),
            session.history.len()
        ));
        output
    }

    /// Show code artifacts
    async fn show_artifacts(&self) -> String {
        let session = self.session.read().await;
        if session.artifacts.is_empty() {
            return "📦 No code artifacts created yet.".to_string();
        }

        let mut output = String::from("📦 Code Artifacts:\n\n");
        for (i, artifact) in session.artifacts.values().enumerate() {
            output.push_str(&format!(
                "{}. {} ({})\n   Language: {} | Confidence: {:.1}% | {} lines\n   Created: {}\n\n",
                i + 1,
                artifact.name,
                artifact.id,
                artifact.language,
                artifact.confidence * 100.0,
                artifact.code.lines().count(),
                artifact
                    .created_at
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            ));
        }

        output
    }

    /// Show active agent tasks
    async fn show_active_tasks(&self) -> String {
        // Get active tasks from agent system
        let tasks_info = self.agent_system.get_active_tasks().await;

        if tasks_info.is_empty() {
            return "⚡ No active tasks currently running.".to_string();
        }

        let mut output = String::from("⚡ Active Agent Tasks:\n\n");
        for (i, task_info) in tasks_info.iter().enumerate() {
            output.push_str(&format!(
                "{}. {} ({})\n   Agent: {} | Priority: {:?}\n   Description: {}\n\n",
                i + 1,
                task_info.id,
                task_info.status,
                task_info.agent_id,
                task_info.priority,
                task_info.description.chars().take(80).collect::<String>()
                    + if task_info.description.len() > 80 {
                        "..."
                    } else {
                        ""
                    }
            ));
        }

        output
    }

    /// Show current configuration
    async fn show_config(&self) -> String {
        let session = self.session.read().await;
        format!(
            "⚙️ Current Configuration:\n\n\
            Auto-save: {}\n\
            Default Language: {}\n\
            Show Confidence: {}\n\
            Verbose Mode: {}\n\
            Max History: {} entries\n\
            Session ID: {}\n\
            Project Path: {:?}",
            session.config.auto_save,
            session
                .config
                .default_language
                .as_deref()
                .unwrap_or("Auto-detect"),
            session.config.show_confidence,
            session.config.verbose,
            session.config.max_history,
            session.session_id,
            session.project_path
        )
    }

    /// Update configuration setting
    async fn update_config(
        &self,
        key: &str,
        value: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut session = self.session.write().await;

        match key.to_lowercase().as_str() {
            "auto-save" | "auto_save" => {
                session.config.auto_save = value.parse().unwrap_or(false);
                Ok(format!("Set auto-save to: {}", session.config.auto_save))
            }
            "default-language" | "default_language" => {
                session.config.default_language = if value.is_empty() { None } else { Some(value.to_string()) };
                Ok(format!("Set default language to: {}", value))
            }
            "show-confidence" | "show_confidence" => {
                session.config.show_confidence = value.parse().unwrap_or(true);
                Ok(format!("Set show-confidence to: {}", session.config.show_confidence))
            }
            "verbose" => {
                session.config.verbose = value.parse().unwrap_or(false);
                Ok(format!("Set verbose mode to: {}", session.config.verbose))
            }
            "max-history" | "max_history" => {
                if let Ok(max_hist) = value.parse::<usize>() {
                    session.config.max_history = max_hist;
                    Ok(format!("Set max history to: {} entries", max_hist))
                } else {
                    Ok("Invalid number for max-history".to_string())
                }
            }
            _ => Ok(format!(
                "Unknown config key: {}. Available keys: auto-save, default-language, show-confidence, verbose, max-history",
                key
            )),
        }
    }

    /// Session Management Methods

    async fn list_sessions(&self) -> String {
        let session_manager = self.session_manager.read().await;
        let sessions = session_manager.list_sessions();

        if sessions.is_empty() {
            return "📁 No sessions available.".to_string();
        }

        let mut output = String::from("📁 Available Sessions:\n\n");
        let current_session = self.session.read().await;
        let current_session_id = &current_session.session_id;

        for session in sessions {
            let marker = if session.id == *current_session_id {
                "► "
            } else {
                "  "
            };
            output.push_str(&format!(
                "{}{} - {} ({})",
                marker,
                session.id, // Use id instead of name since SessionInfo doesn't have a name field
                session
                    .project_path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "No project".to_string()),
                "<timestamp>" // Format time differently since SystemTime doesn't have format method
            ));
            if session.id == *current_session_id {
                output.push_str(" ✨ Current");
            }
            output.push('\n');
        }

        output
    }

    async fn create_new_session(&self, project_path: Option<std::path::PathBuf>) -> String {
        let mut session_manager = self.session_manager.write().await;
        let session_id = session_manager.create_session(project_path.clone());

        format!(
            "✨ Created new session: {} with project path: {:?}",
            session_id, project_path
        )
    }

    async fn switch_session(&self, session_id: &str) -> String {
        let session_manager = self.session_manager.read().await;

        if let Some(session_info) = session_manager.get_session(session_id) {
            // Create new session from the stored info
            let new_session = InteractiveSession::new(session_info.project_path.clone());

            // Replace current session
            *self.session.write().await = new_session;

            format!(
                "🔄 Switched to session: {} ({})",
                session_id, // Use the session_id parameter since that's what we're switching to
                session_info
                    .project_path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "No project".to_string())
            )
        } else {
            format!("❌ Session not found: {}", session_id)
        }
    }

    async fn delete_session(&self, session_id: &str) -> String {
        let mut session_manager = self.session_manager.write().await;
        let current_session_id = self.session.read().await.session_id.clone();

        if session_id == current_session_id {
            return "❌ Cannot delete the current active session.".to_string();
        }

        match session_manager.delete_session(session_id) {
            Ok(()) => format!("🗑️  Deleted session: {}", session_id),
            Err(e) => format!("❌ Failed to delete session: {}", e),
        }
    }

    async fn clone_session(&self, session_id: &str) -> String {
        let mut session_manager = self.session_manager.write().await;

        match session_manager.clone_session(session_id) {
            Ok(new_session_id) => {
                format!(
                    "📋 Cloned session {} to new session: {}",
                    session_id, new_session_id
                )
            }
            Err(e) => {
                format!("❌ Failed to clone session {}: {}", session_id, e)
            }
        }
    }

    async fn create_bookmark(&self, name: &str, description: &str) -> String {
        let mut session_manager = self.session_manager.write().await;
        let current_session_id = self.session.read().await.session_id.clone();

        match session_manager.create_bookmark(
            name.to_string(),
            description.to_string(),
            current_session_id,
            Vec::new(), // Empty tags
        ) {
            Ok(bookmark_id) => {
                format!(
                    "🔖 Created bookmark '{}': {} (ID: {})",
                    name, description, bookmark_id
                )
            }
            Err(e) => {
                format!("❌ Failed to create bookmark: {}", e)
            }
        }
    }

    async fn list_bookmarks(&self) -> String {
        let session_manager = self.session_manager.read().await;
        let bookmarks = session_manager.list_bookmarks();

        if bookmarks.is_empty() {
            return "🔖 No bookmarks created yet.".to_string();
        }

        let mut output = String::from("🔖 Bookmarks:\n\n");
        for (i, (bookmark_id, bookmark)) in bookmarks.iter().enumerate() {
            output.push_str(&format!(
                "{}. {} - {}\n   Session: {} | ID: {}\n\n",
                i + 1,
                bookmark.name,
                bookmark.description,
                bookmark.session_id,
                bookmark_id
            ));
        }

        output
    }

    async fn goto_bookmark(&self, bookmark_id: &str) -> String {
        let session_manager = self.session_manager.read().await;

        if let Some(bookmark) = session_manager.get_bookmark(bookmark_id) {
            let session_id = bookmark.session_id.clone();
            drop(session_manager); // Release the read lock
            self.switch_session(&session_id).await
        } else {
            format!("❌ Bookmark not found: {}", bookmark_id)
        }
    }

    async fn delete_bookmark(&self, bookmark_id: &str) -> String {
        let mut session_manager = self.session_manager.write().await;

        match session_manager.delete_bookmark(bookmark_id) {
            Ok(()) => format!("🗑️  Deleted bookmark: {}", bookmark_id),
            Err(e) => format!("❌ Failed to delete bookmark: {}", e),
        }
    }
}

/// Spawn agent monitoring task
fn spawn_agent_monitor(
    agent_system: Arc<AgentSystem>,
    ui_sender: mpsc::UnboundedSender<UIEvent>,
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
                    task: None,     // TODO: Add current task info
                    priority: None, // TODO: Add priority info
                    progress: None, // TODO: Add progress tracking
                });
            }

            // Send periodic system status
            let agent_statuses = agent_system.get_agent_statuses().await;
            if !agent_statuses.is_empty() {
                let status_summary = agent_statuses
                    .iter()
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
    mut command_rx: mpsc::UnboundedReceiver<String>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(command) = command_rx.recv().await {
            if let Err(e) = manager.process_command(command).await {
                eprintln!("Error processing command: {}", e);
            }
        }
    })
}
