//! Agent System Module
//!
//! This module provides the multi-agent system for task coordination and execution.
//! Agents can be specialized for different tasks like code generation, analysis, and debugging.
//! The system also includes customizable behavior profiles that define agent personalities,
//! decision-making patterns, and interaction styles.

pub mod agent_types;
pub mod behavior;
pub mod enhanced_agent;
pub mod progress;
pub mod review;
pub mod system;
pub mod task;
pub mod orchestrator;
pub mod state_machine;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

// Re-export commonly used types
pub use progress::{AgentProgressTracker, AgentProgressExtension, TaskMetrics};
pub use system::{AgentInfo, AgentSystem};
pub use task::{AgentResult, AgentTask, TaskPriority};

/// Core trait that all agents must implement
#[async_trait::async_trait]
pub trait Agent: Send + Sync + fmt::Debug {
    /// Get the agent's unique identifier
    fn id(&self) -> &str;

    /// Get the agent's name
    fn name(&self) -> &str;

    /// Get the agent's current status
    fn status(&self) -> AgentStatus;

    /// Get the agent's capabilities
    fn capabilities(&self) -> Vec<String>;

    /// Process a task assigned to this agent
    async fn process_task(&mut self, task: AgentTask) -> Result<AgentResult, AgentError>;

    /// Check if the agent can handle a specific task type
    fn can_handle(&self, task_type: &str) -> bool;

    /// Get agent metrics/statistics
    fn get_metrics(&self) -> AgentMetrics;

    /// Shutdown the agent gracefully
    async fn shutdown(&mut self) -> Result<(), AgentError>;
}

/// Status of an agent
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum AgentStatus {
    /// Agent is idle and ready to accept tasks
    Idle,
    /// Agent is currently processing a task
    Processing { task_id: String },
    /// Agent is busy and cannot accept new tasks
    Busy,
    /// Agent has encountered an error
    Error { message: String },
    /// Agent is shutting down
    ShuttingDown,
    /// Agent is offline
    Offline,
}

/// Agent performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub tasks_completed: usize,
    pub tasks_failed: usize,
    pub average_task_duration: f64,
    pub uptime_seconds: u64,
    pub success_rate: f64,
}

impl Default for AgentMetrics {
    fn default() -> Self {
        Self {
            tasks_completed: 0,
            tasks_failed: 0,
            average_task_duration: 0.0,
            uptime_seconds: 0,
            success_rate: 1.0,
        }
    }
}

/// Errors that can occur during agent operations
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Task execution failed: {0}")]
    TaskExecutionFailed(String),

    #[error("Agent is not available: {status:?}")]
    AgentUnavailable { status: AgentStatus },

    #[error("Invalid task type: {task_type}")]
    InvalidTaskType { task_type: String },

    #[error("Task timeout after {timeout_seconds}s")]
    TaskTimeout { timeout_seconds: u64 },

    #[error("Agent configuration error: {0}")]
    ConfigurationError(String),

    #[error("AI service error: {0}")]
    AIServiceError(String),

    #[error("Context error: {0}")]
    ContextError(String),

    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Progress tracking error: {0}")]
    ProgressTrackingError(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

impl fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentStatus::Idle => write!(f, "Idle"),
            AgentStatus::Processing { task_id } => write!(f, "Processing task {}", task_id),
            AgentStatus::Busy => write!(f, "Busy"),
            AgentStatus::Error { message } => write!(f, "Error: {}", message),
            AgentStatus::ShuttingDown => write!(f, "Shutting down"),
            AgentStatus::Offline => write!(f, "Offline"),
        }
    }
}

/// Base agent implementation with common functionality
#[derive(Debug)]
pub struct BaseAgent {
    pub id: String,
    pub name: String,
    pub status: AgentStatus,
    pub capabilities: Vec<String>,
    pub metrics: AgentMetrics,
    pub start_time: std::time::Instant,
}

impl BaseAgent {
    pub fn new(name: String, capabilities: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            status: AgentStatus::Idle,
            capabilities,
            metrics: AgentMetrics::default(),
            start_time: std::time::Instant::now(),
        }
    }

    /// Update agent metrics after task completion
    pub fn update_metrics(&mut self, success: bool, duration: std::time::Duration) {
        if success {
            self.metrics.tasks_completed += 1;
        } else {
            self.metrics.tasks_failed += 1;
        }

        let total_tasks = self.metrics.tasks_completed + self.metrics.tasks_failed;
        if total_tasks > 0 {
            self.metrics.success_rate = self.metrics.tasks_completed as f64 / total_tasks as f64;
        }

        // Update average duration
        let current_avg = self.metrics.average_task_duration;
        let new_duration = duration.as_secs_f64();
        self.metrics.average_task_duration =
            (current_avg * (total_tasks - 1) as f64 + new_duration) / total_tasks as f64;

        self.metrics.uptime_seconds = self.start_time.elapsed().as_secs();
    }
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub max_concurrent_tasks: usize,
    pub task_timeout_seconds: u64,
    pub retry_attempts: usize,
    pub priority_handling: bool,
    pub custom_settings: HashMap<String, serde_json::Value>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 1,
            task_timeout_seconds: 300,
            retry_attempts: 3,
            priority_handling: true,
            custom_settings: HashMap::new(),
        }
    }
}
