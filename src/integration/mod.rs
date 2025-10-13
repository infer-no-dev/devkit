//! Integration Layer for DevKit Components
//!
//! This module provides integration between different components of the DevKit system,
//! specifically wiring together the agent system, session management, AI services,
//! and context management for seamless operation.

use crate::agents::{AgentSystem, AgentTask, TaskPriority};
use crate::ai::AIManager;
use crate::config::ConfigManager;
use crate::context::ContextManager;
use crate::session::{FileSystemPersistence, Session, SessionManager, SessionUser, UserPreferences};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Integrated development environment that combines all DevKit components
pub struct IntegratedDevEnv {
    /// Agent coordination system
    pub agent_system: AgentSystem,
    /// Session management
    pub session_manager: SessionManager,
    /// AI service management
    pub ai_manager: Arc<AIManager>,
    /// Code context analysis
    pub context_manager: ContextManager,
    /// Configuration management
    pub config_manager: ConfigManager,
    /// Current user information
    pub current_user: SessionUser,
    /// Current session ID (if any)
    pub current_session: Option<String>,
}

impl IntegratedDevEnv {
    /// Create a new integrated development environment
    pub async fn new(config_manager: ConfigManager) -> Result<Self> {
        info!("Initializing Integrated Development Environment");

        // Create AI manager from configuration
        let config = config_manager.config();
        let ai_manager = AIManager::from_config(config).await
            .map_err(|e| {
                warn!("Failed to initialize AI manager: {}. Continuing without AI support.", e);
                e
            })
            .unwrap_or_else(|_| {
                // Create a minimal AI manager with default settings
                let default_config = crate::config::AIModelConfig::default();
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        AIManager::new(default_config).await.unwrap_or_else(|e| {
                            panic!("Failed to create default AI manager: {}", e);
                        })
                    })
                })
            });

        // Create agent system with AI manager
        let ai_manager_arc = Arc::new(ai_manager);
        let agent_system = AgentSystem::with_ai_manager(ai_manager_arc.clone());
        
        // Start the agent system
        agent_system.start().await.map_err(|e| anyhow::anyhow!("Failed to start agent system: {}", e))?;
        
        // Initialize agent system with default agents
        agent_system.initialize().await.map_err(|e| anyhow::anyhow!("Failed to initialize agents: {}", e))?;
        
        // Initialize context manager
        let context_manager = ContextManager::new()?;

        // Create default user
        let current_user = SessionUser {
            id: "default-user".to_string(),
            name: "DevKit User".to_string(),
            email: None,
            avatar_url: None,
            timezone: "UTC".to_string(),
            preferences: UserPreferences::default(),
        };

        // Initialize session manager with filesystem persistence
        let data_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".devkit")
            .join("sessions");

        let persistence = Arc::new(
            FileSystemPersistence::new(data_dir).await
                .map_err(|e| anyhow::anyhow!("Failed to initialize session persistence: {}", e))?
        );

        let session_manager = SessionManager::new(
            persistence,
            config_manager,
            current_user.clone(),
            crate::session::SessionManagerConfig::default(),
        )?;

        info!("Integrated Development Environment initialized successfully");

        Ok(Self {
            agent_system,
            session_manager,
            ai_manager: ai_manager_arc,
            context_manager,
            config_manager: ConfigManager::new(None)?, // Create a new instance
            current_user,
            current_session: None,
        })
    }

    /// Create a new development session
    pub async fn create_session(&mut self, name: String, description: Option<String>) -> Result<String> {
        debug!("Creating new development session: {}", name);

        let session_id = self.session_manager.create_session(name.clone(), description.clone()).await
            .map_err(|e| anyhow::anyhow!("Failed to create session: {}", e))?;

        // Initialize agents for this session
        self.initialize_session_agents(&session_id).await?;

        self.current_session = Some(session_id.clone());
        info!("Created and activated development session: {} ({})", name, session_id);

        Ok(session_id)
    }

    /// Load an existing session
    pub async fn load_session(&mut self, session_id: &str) -> Result<()> {
        debug!("Loading development session: {}", session_id);

        let session = self.session_manager.load_session(session_id).await
            .map_err(|e| anyhow::anyhow!("Failed to load session: {}", e))?;

        // Restore agents for this session
        self.restore_session_agents(&session).await?;

        self.current_session = Some(session_id.to_string());
        info!("Loaded and activated development session: {}", session.name);

        Ok(())
    }

    /// Initialize agents for a new session
    async fn initialize_session_agents(&mut self, session_id: &str) -> Result<()> {
        debug!("Initializing agents for session: {}", session_id);

        // Add core agents to the session
        let agents_to_add = vec![
            ("code-generator", "Code Generation Agent", "CodeGeneration"),
            ("analyzer", "Code Analysis Agent", "Analysis"),
            ("refactor", "Code Refactoring Agent", "Refactoring"),
        ];

        for (agent_id, agent_name, agent_type) in agents_to_add {
            self.session_manager.add_agent(
                session_id,
                agent_id.to_string(),
                agent_name.to_string(),
                agent_type.to_string(),
                None, // No specific behavior profile for now
            ).await
                .map_err(|e| anyhow::anyhow!("Failed to add agent {} to session: {}", agent_id, e))?;

            debug!("Added agent {} to session {}", agent_id, session_id);
        }

        Ok(())
    }

    /// Restore agents from an existing session
    async fn restore_session_agents(&mut self, session: &Session) -> Result<()> {
        debug!("Restoring agents for session: {}", session.id);

        for (agent_id, agent_info) in &session.agents {
            debug!("Restoring agent: {} ({})", agent_id, agent_info.name);
            // In a full implementation, you would restore the agent's state
            // and re-register it with the agent system
        }

        Ok(())
    }

    /// Submit a task to the agent system within the current session context
    pub async fn submit_task(&mut self, description: String, task_type: String, priority: Option<TaskPriority>) -> Result<String> {
        let session_id = self.current_session.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No active session. Create or load a session first."))?;

        debug!("Submitting task to session {}: {}", session_id, description);

        // Create task with session context
        let task_id = Uuid::new_v4().to_string();
        let mut task = AgentTask {
            id: task_id.clone(),
            task_type: task_type.clone(),
            description: description.clone(),
            context: serde_json::json!({
                "session_id": session_id,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }),
            priority: priority.unwrap_or(TaskPriority::Normal),
            deadline: None,
            metadata: std::collections::HashMap::new(),
        };

        // Add context information if available
        if let Ok(current_dir) = std::env::current_dir() {
            task.metadata.insert(
                "working_directory".to_string(),
                serde_json::Value::String(current_dir.display().to_string()),
            );
        }

        // Submit task to agent system
        let result = self.agent_system.submit_task(task).await
            .map_err(|e| anyhow::anyhow!("Failed to submit task to agent system: {}", e))?;

        // Update session with task information
        self.session_manager.update_agent_status(
            session_id,
            "task-coordinator",
            crate::agents::AgentStatus::Processing { task_id: task_id.clone() },
            Some(crate::session::TaskInfo {
                id: task_id.clone(),
                description: description.clone(),
                priority: priority.unwrap_or(TaskPriority::Normal),
                started_at: chrono::Utc::now(),
                estimated_completion: None,
                progress: 0.0,
                dependencies: Vec::new(),
                artifacts: Vec::new(),
            }),
        ).await.ok(); // Ignore errors for now

        info!("Submitted task {} to session {}: {}", task_id, session_id, description);
        Ok(task_id)
    }

    /// Generate code using the integrated system
    pub async fn generate_code(&mut self, prompt: String, language: Option<String>) -> Result<String> {
        let task_id = self.submit_task(
            format!("Generate code: {}", prompt),
            "code_generation".to_string(),
            Some(TaskPriority::High),
        ).await?;

        // For now, return a simple generated response
        // In a full implementation, this would wait for the agent to complete the task
        let language_str = language.as_deref().unwrap_or("auto-detected");
        let generated_code = format!(
            "// Generated code for: {}\n// Language: {}\n// Task ID: {}\n\n// TODO: Implement actual code generation\npub fn generated_function() {{\n    println!(\"Generated from prompt: {}\");\n}}\n",
            prompt,
            language_str,
            task_id,
            prompt
        );

        // Add artifact to session
        if let Some(session_id) = &self.current_session {
            let artifact = crate::session::SessionArtifact {
                id: Uuid::new_v4().to_string(),
                name: "Generated Code".to_string(),
                description: Some(format!("Generated from prompt: {}", prompt)),
                artifact_type: crate::session::ArtifactType::SourceCode,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                creator: task_id,
                content: crate::session::ArtifactContent::Text(generated_code.clone()),
                metadata: crate::session::ArtifactMetadata {
                    language: language,
                    framework: None,
                    size: Some(generated_code.len() as u64),
                    checksum: None,
                    license: None,
                    tags: std::collections::HashSet::new(),
                    quality_metrics: None,
                    custom_fields: std::collections::HashMap::new(),
                },
                dependencies: Vec::new(),
                version: "1.0.0".to_string(),
                status: crate::session::ArtifactStatus::Draft,
            };

            self.session_manager.add_artifact(session_id, artifact).await.ok();
        }

        Ok(generated_code)
    }

    /// Analyze codebase using the integrated system
    pub async fn analyze_codebase(&mut self, path: PathBuf) -> Result<crate::context::CodebaseContext> {
        debug!("Analyzing codebase at: {:?}", path);

        let analysis_config = crate::context::AnalysisConfig::default();
        let context = self.context_manager.analyze_codebase(path.clone(), analysis_config).await
            .map_err(|e| anyhow::anyhow!("Failed to analyze codebase: {}", e))?;

        info!("Analyzed codebase: {} files, {} symbols", 
            context.metadata.total_files,
            context.metadata.indexed_symbols
        );

        // Submit analysis task to track in session
        if self.current_session.is_some() {
            self.submit_task(
                format!("Analyze codebase at: {}", path.display()),
                "analysis".to_string(),
                Some(TaskPriority::Normal),
            ).await.ok();
        }

        Ok(context)
    }

    /// Get current session information
    pub async fn get_current_session(&mut self) -> Result<Option<Session>> {
        if let Some(session_id) = &self.current_session {
            let session = self.session_manager.load_session(session_id).await
                .map_err(|e| anyhow::anyhow!("Failed to load current session: {}", e))?;
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    /// List all available sessions for the current user
    pub async fn list_sessions(&self) -> Result<Vec<Session>> {
        let sessions = self.session_manager.list_sessions().await
            .map_err(|e| anyhow::anyhow!("Failed to list sessions: {}", e))?;
        Ok(sessions)
    }

    /// Get system status
    pub async fn get_system_status(&self) -> SystemStatus {
        SystemStatus {
            ai_manager_available: true, // AI manager is always created
            agents_active: {
                let stats = self.agent_system.get_system_stats().await;
                stats.total_agents > 0
            },
            current_session: self.current_session.clone(),
            context_manager_ready: true, // Context manager is always ready
        }
    }
}

/// System status information
#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub ai_manager_available: bool,
    pub agents_active: bool,
    pub current_session: Option<String>,
    pub context_manager_ready: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::config::Config;

    #[tokio::test]
    async fn test_integrated_dev_env_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Create a minimal config
        let config = Config::default();
        let config_manager = ConfigManager::new(Some(config_path)).unwrap();

        let env = IntegratedDevEnv::new(config_manager).await;
        assert!(env.is_ok(), "Should be able to create integrated environment");

        let env = env.unwrap();
        assert!(env.current_session.is_none(), "Should start with no active session");
    }

    #[tokio::test]
    async fn test_session_creation_and_loading() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config = Config::default();
        let config_manager = ConfigManager::new(Some(config_path)).unwrap();

        let mut env = IntegratedDevEnv::new(config_manager).await.unwrap();

        // Create a session
        let session_id = env.create_session(
            "Test Session".to_string(),
            Some("A test session".to_string()),
        ).await.unwrap();

        assert!(env.current_session.is_some());
        assert_eq!(env.current_session.as_ref().unwrap(), &session_id);

        // Load the session
        env.current_session = None; // Reset
        env.load_session(&session_id).await.unwrap();
        assert!(env.current_session.is_some());
    }

    #[tokio::test]
    async fn test_task_submission() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config = Config::default();
        let config_manager = ConfigManager::new(Some(config_path)).unwrap();

        let mut env = IntegratedDevEnv::new(config_manager).await.unwrap();

        // Create a session first
        env.create_session("Test Session".to_string(), None).await.unwrap();

        // Submit a task
        let task_id = env.submit_task(
            "Generate hello world function".to_string(),
            "code_generation".to_string(),
            None,
        ).await.unwrap();

        assert!(!task_id.is_empty());
    }

    #[tokio::test]
    async fn test_code_generation_integration() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config = Config::default();
        let config_manager = ConfigManager::new(Some(config_path)).unwrap();

        let mut env = IntegratedDevEnv::new(config_manager).await.unwrap();

        // Create a session first
        env.create_session("Test Session".to_string(), None).await.unwrap();

        // Generate code
        let generated_code = env.generate_code(
            "create a hello world function".to_string(),
            Some("rust".to_string()),
        ).await.unwrap();

        assert!(!generated_code.is_empty());
        assert!(generated_code.contains("generated_function"));
    }
}