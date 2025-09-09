//! Agent System - coordinates multiple agents and manages task distribution

use super::{Agent, AgentError, AgentStatus, AgentMetrics};
use super::task::{AgentTask, AgentResult, TaskPriority};
use crate::ai::AIManager;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, Mutex};
use uuid::Uuid;

/// Central system for managing multiple agents and task distribution
#[derive(Debug)]
pub struct AgentSystem {
    /// Registered agents by ID
    agents: Arc<RwLock<HashMap<String, Box<dyn Agent>>>>,
    
    /// Task queue with priority handling
    task_queue: Arc<Mutex<std::collections::BinaryHeap<PrioritizedTask>>>,
    
    /// Active tasks being processed
    active_tasks: Arc<RwLock<HashMap<String, String>>>, // task_id -> agent_id
    
    /// Task results
    completed_tasks: Arc<RwLock<HashMap<String, AgentResult>>>,
    
    /// AI manager for agents that need AI capabilities
    ai_manager: Option<Arc<AIManager>>,
    
    /// System configuration
    config: AgentSystemConfig,
    
    /// Task result sender
    result_sender: Option<mpsc::UnboundedSender<AgentResult>>,
    
    /// Task result receiver
    result_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<AgentResult>>>>,
}

/// Agent system configuration
#[derive(Debug, Clone)]
pub struct AgentSystemConfig {
    pub max_concurrent_tasks: usize,
    pub task_timeout_seconds: u64,
    pub retry_failed_tasks: bool,
    pub max_retry_attempts: usize,
}

impl Default for AgentSystemConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 10,
            task_timeout_seconds: 300,
            retry_failed_tasks: true,
            max_retry_attempts: 3,
        }
    }
}

/// Wrapper for tasks in priority queue
#[derive(Debug)]
struct PrioritizedTask {
    task: AgentTask,
    priority_score: u32,
}

impl PartialEq for PrioritizedTask {
    fn eq(&self, other: &Self) -> bool {
        self.priority_score == other.priority_score
    }
}

impl Eq for PrioritizedTask {}

impl PartialOrd for PrioritizedTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority scores come first (max heap behavior)
        self.priority_score.cmp(&other.priority_score)
    }
}

impl AgentSystem {
    /// Create a new agent system
    pub fn new() -> Self {
        let (result_sender, result_receiver) = mpsc::unbounded_channel();
        
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            task_queue: Arc::new(Mutex::new(std::collections::BinaryHeap::new())),
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            completed_tasks: Arc::new(RwLock::new(HashMap::new())),
            ai_manager: None,
            config: AgentSystemConfig::default(),
            result_sender: Some(result_sender),
            result_receiver: Arc::new(Mutex::new(Some(result_receiver))),
        }
    }
    
    /// Create a new agent system with AI manager
    pub fn with_ai_manager(ai_manager: Arc<AIManager>) -> Self {
        let mut system = Self::new();
        system.ai_manager = Some(ai_manager);
        system
    }
    
    /// Initialize the agent system with default agents
    pub async fn initialize(&self) {
        // Create default agents
        use super::agent_types::{CodeGenerationAgent, AnalysisAgent, RefactoringAgent};
        
        if let Some(ai_manager) = &self.ai_manager {
            // Create code generation agent
            let code_agent = CodeGenerationAgent::with_ai_manager(ai_manager.clone());
            self.register_agent(Box::new(code_agent)).await;
            
            // Create analysis agent
            let analysis_agent = AnalysisAgent::with_ai_manager(ai_manager.clone());
            self.register_agent(Box::new(analysis_agent)).await;
            
            // Create refactoring agent
            let refactor_agent = RefactoringAgent::with_ai_manager(ai_manager.clone());
            self.register_agent(Box::new(refactor_agent)).await;
        } else {
            // Create basic agents without AI
            let code_agent = CodeGenerationAgent::new();
            self.register_agent(Box::new(code_agent)).await;
            
            let analysis_agent = AnalysisAgent::new();
            self.register_agent(Box::new(analysis_agent)).await;
            
            let refactor_agent = RefactoringAgent::new();
            self.register_agent(Box::new(refactor_agent)).await;
        }
    }
    
    /// Register an agent with the system
    pub async fn register_agent(&self, agent: Box<dyn Agent>) {
        let agent_id = agent.id().to_string();
        let mut agents = self.agents.write().await;
        agents.insert(agent_id, agent);
    }
    
    /// Unregister an agent from the system
    pub async fn unregister_agent(&self, agent_id: &str) -> Result<(), AgentError> {
        let mut agents = self.agents.write().await;
        agents.remove(agent_id);
        Ok(())
    }
    
    /// Submit a task to the system
    pub async fn submit_task(&self, task: AgentTask) -> Result<AgentResult, AgentError> {
        let priority_score = match task.priority {
            TaskPriority::Critical => 1000,
            TaskPriority::High => 100,
            TaskPriority::Normal => 10,
            TaskPriority::Low => 1,
        };
        
        let prioritized_task = PrioritizedTask {
            task,
            priority_score,
        };
        
        // Add task to queue
        {
            let mut queue = self.task_queue.lock().await;
            queue.push(prioritized_task);
        }
        
        // Process the task
        self.process_next_task().await
    }
    
    /// Process the next task in the queue
    async fn process_next_task(&self) -> Result<AgentResult, AgentError> {
        // Get the next task
        let task = {
            let mut queue = self.task_queue.lock().await;
            queue.pop()
        };
        
        let task = match task {
            Some(prioritized_task) => prioritized_task.task,
            None => return Err(AgentError::TaskExecutionFailed("No tasks in queue".to_string())),
        };
        
        // Find a suitable agent
        let agent_id = self.find_suitable_agent(&task).await?;
        
        // Record the active task
        {
            let mut active_tasks = self.active_tasks.write().await;
            active_tasks.insert(task.id.clone(), agent_id.clone());
        }
        
        // Process the task
        let result = {
            let mut agents = self.agents.write().await;
            if let Some(agent) = agents.get_mut(&agent_id) {
                agent.process_task(task).await
            } else {
                Err(AgentError::AgentUnavailable { status: AgentStatus::Offline })
            }
        };
        
        // Remove from active tasks
        {
            let mut active_tasks = self.active_tasks.write().await;
            active_tasks.remove(&result.as_ref().map(|r| r.task_id.clone()).unwrap_or_default());
        }
        
        // Store the result
        if let Ok(ref result) = result {
            let mut completed_tasks = self.completed_tasks.write().await;
            completed_tasks.insert(result.task_id.clone(), result.clone());
        }
        
        result
    }
    
    /// Find a suitable agent for a task
    async fn find_suitable_agent(&self, task: &AgentTask) -> Result<String, AgentError> {
        let agents = self.agents.read().await;
        
        for (agent_id, agent) in agents.iter() {
            if agent.can_handle(&task.task_type) && agent.status() == AgentStatus::Idle {
                return Ok(agent_id.clone());
            }
        }
        
        Err(AgentError::TaskExecutionFailed(
            format!("No suitable agent found for task type: {}", task.task_type)
        ))
    }
    
    /// Get status of all agents
    pub async fn get_agent_statuses(&self) -> HashMap<String, AgentStatus> {
        let agents = self.agents.read().await;
        let mut statuses = HashMap::new();
        
        for (agent_id, agent) in agents.iter() {
            statuses.insert(agent_id.clone(), agent.status());
        }
        
        statuses
    }
    
    /// Get metrics for all agents
    pub async fn get_agent_metrics(&self) -> HashMap<String, AgentMetrics> {
        let agents = self.agents.read().await;
        let mut metrics = HashMap::new();
        
        for (agent_id, agent) in agents.iter() {
            metrics.insert(agent_id.clone(), agent.get_metrics());
        }
        
        metrics
    }
    
    /// Get agent information for all agents
    pub async fn get_agents_info(&self) -> Vec<AgentInfo> {
        let agents = self.agents.read().await;
        let mut agents_info = Vec::new();
        
        for (_, agent) in agents.iter() {
            agents_info.push(AgentInfo {
                id: agent.id().to_string(),
                name: agent.name().to_string(),
                status: agent.status(),
                capabilities: agent.capabilities(),
                metrics: agent.get_metrics(),
            });
        }
        
        agents_info
    }
    
    /// Get system statistics
    pub async fn get_system_stats(&self) -> SystemStats {
        let agents = self.agents.read().await;
        let active_tasks = self.active_tasks.read().await;
        let completed_tasks = self.completed_tasks.read().await;
        let queue = self.task_queue.lock().await;
        
        SystemStats {
            total_agents: agents.len(),
            active_tasks: active_tasks.len(),
            queued_tasks: queue.len(),
            completed_tasks: completed_tasks.len(),
        }
    }
    
    /// Shutdown all agents
    pub async fn shutdown(&self) -> Result<(), AgentError> {
        let mut agents = self.agents.write().await;
        
        for (_, agent) in agents.iter_mut() {
            if let Err(e) = agent.shutdown().await {
                eprintln!("Warning: Failed to shutdown agent {}: {}", agent.id(), e);
            }
        }
        
        Ok(())
    }
}

/// Agent information for UI display
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub status: AgentStatus,
    pub capabilities: Vec<String>,
    pub metrics: AgentMetrics,
}

/// System statistics
#[derive(Debug, Clone)]
pub struct SystemStats {
    pub total_agents: usize,
    pub active_tasks: usize,
    pub queued_tasks: usize,
    pub completed_tasks: usize,
}

impl Default for AgentSystem {
    fn default() -> Self {
        Self::new()
    }
}
