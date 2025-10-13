//! Agent System - coordinates multiple agents and manages task distribution

use super::task::{AgentResult, AgentTask, TaskPriority};
use super::{Agent, AgentMetrics, AgentStatus};
use crate::ai::AIManager;
// TODO: Fix circular dependency with error module
// use crate::error::{DevKitError, DevKitResult, ErrorContext, WithContext};

use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, oneshot, watch, Mutex, RwLock};

/// Central system for managing multiple agents and task distribution
#[derive(Debug)]
pub struct AgentSystem {
    /// Registered agents by ID
    agents: Arc<RwLock<HashMap<String, Box<dyn Agent>>>>,

    /// Task queue with priority handling
    task_queue: Arc<Mutex<BinaryHeap<PrioritizedTask>>>,

    /// Active tasks being processed
    active_tasks: Arc<RwLock<HashMap<String, ActiveTask>>>,

    /// Completed task results
    completed_tasks: Arc<RwLock<HashMap<String, TaskResult>>>,

    /// Failed tasks with retry information
    failed_tasks: Arc<RwLock<HashMap<String, FailedTask>>>,

    /// AI manager for agents that need AI capabilities
    ai_manager: Option<Arc<AIManager>>,

    /// System configuration
    config: AgentSystemConfig,

    /// Task event broadcaster
    event_sender: broadcast::Sender<TaskEvent>,

    /// System shutdown signal
    shutdown_sender: watch::Sender<bool>,
    shutdown_receiver: watch::Receiver<bool>,

    /// Task processing worker handles
    worker_handles: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,

    /// System running state
    running: Arc<RwLock<bool>>,
}

/// Agent system configuration
#[derive(Debug, Clone)]
pub struct AgentSystemConfig {
    pub max_concurrent_tasks: usize,
    pub task_timeout_seconds: u64,
    pub retry_failed_tasks: bool,
    pub max_retry_attempts: usize,
    pub worker_count: usize,
    pub max_queue_size: usize,
    pub task_history_limit: usize,
    pub heartbeat_interval_seconds: u64,
}

impl Default for AgentSystemConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 10,
            task_timeout_seconds: 300,
            retry_failed_tasks: true,
            max_retry_attempts: 3,
            worker_count: 4,
            max_queue_size: 1000,
            task_history_limit: 10000,
            heartbeat_interval_seconds: 30,
        }
    }
}

/// Active task being processed
#[derive(Debug)]
pub struct ActiveTask {
    pub task: AgentTask,
    pub agent_id: String,
    pub started_at: Instant,
    pub result_sender: Option<oneshot::Sender<Result<AgentResult, anyhow::Error>>>,
}

/// Completed task result with metadata
#[derive(Debug, Clone)]
pub struct TaskResult {
    pub result: AgentResult,
    pub completed_at: Instant,
    pub processing_duration: Duration,
}

/// Failed task with retry information
#[derive(Debug, Clone)]
pub struct FailedTask {
    pub task: AgentTask,
    pub error: String,
    pub retry_count: usize,
    pub last_attempt_at: Instant,
    pub next_retry_at: Option<Instant>,
}

/// Task events for monitoring and coordination
#[derive(Debug, Clone)]
pub enum TaskEvent {
    TaskSubmitted {
        task_id: String,
        task_type: String,
        priority: TaskPriority,
    },
    TaskStarted {
        task_id: String,
        agent_id: String,
    },
    TaskCompleted {
        task_id: String,
        agent_id: String,
        success: bool,
        duration_ms: u64,
    },
    TaskFailed {
        task_id: String,
        agent_id: String,
        error: String,
        will_retry: bool,
    },
    TaskRetried {
        task_id: String,
        retry_count: usize,
    },
    TaskTimeout {
        task_id: String,
        agent_id: String,
    },
    AgentRegistered {
        agent_id: String,
        capabilities: Vec<String>,
    },
    AgentUnregistered {
        agent_id: String,
    },
    SystemShutdown,
}

/// Wrapper for tasks in priority queue
#[derive(Debug, Clone)]
struct PrioritizedTask {
    task: AgentTask,
    priority_score: u32,
    submitted_at: Instant,
    deadline_score: u32,
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
        // First compare by deadline urgency, then by priority
        let deadline_cmp = self.deadline_score.cmp(&other.deadline_score);
        if deadline_cmp != std::cmp::Ordering::Equal {
            return deadline_cmp;
        }
        // Then by priority score (higher priority comes first)
        let priority_cmp = self.priority_score.cmp(&other.priority_score);
        if priority_cmp != std::cmp::Ordering::Equal {
            return priority_cmp;
        }
        // Finally by submission time (FIFO for same priority)
        other.submitted_at.cmp(&self.submitted_at)
    }
}

impl AgentSystem {
    /// Create a new agent system
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        let (shutdown_sender, shutdown_receiver) = watch::channel(false);

        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            task_queue: Arc::new(Mutex::new(BinaryHeap::new())),
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            completed_tasks: Arc::new(RwLock::new(HashMap::new())),
            failed_tasks: Arc::new(RwLock::new(HashMap::new())),
            ai_manager: None,
            config: AgentSystemConfig::default(),
            event_sender,
            shutdown_sender,
            shutdown_receiver,
            worker_handles: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Create a new agent system with AI manager
    pub fn with_ai_manager(ai_manager: Arc<AIManager>) -> Self {
        let mut system = Self::new();
        system.ai_manager = Some(ai_manager);
        system
    }

    /// Create a new agent system with custom configuration
    pub fn with_config(config: AgentSystemConfig) -> Self {
        let mut system = Self::new();
        system.config = config;
        system
    }

    /// Start the agent system workers
    pub async fn start(&self) -> Result<(), anyhow::Error> {
        let start_time = std::time::Instant::now();

        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }

        *running = true;
        drop(running);

        // Log system startup
        println!(
            "Starting agent system with {} workers",
            self.config.worker_count,
        );

        // Start worker tasks
        let mut handles = self.worker_handles.lock().await;

        // Start task processor workers
        for worker_id in 0..self.config.worker_count {
            let handle = self.spawn_worker(worker_id).await;
            handles.push(handle);
        }

        // Start retry handler
        let retry_handle = self.spawn_retry_handler().await;
        handles.push(retry_handle);

        // Start cleanup handler
        let cleanup_handle = self.spawn_cleanup_handler().await;
        handles.push(cleanup_handle);

        // Start heartbeat monitor
        let heartbeat_handle = self.spawn_heartbeat_monitor().await;
        handles.push(heartbeat_handle);

        // Emit system started event
        let _ = self.event_sender.send(TaskEvent::AgentRegistered {
            agent_id: "system".to_string(),
            capabilities: vec!["coordination".to_string(), "monitoring".to_string()],
        });

        let startup_duration = start_time.elapsed();

        println!(
            "Agent system started successfully in {}ms",
            startup_duration.as_millis()
        );

        Ok(())
    }

    /// Stop the agent system and all workers
    pub async fn stop(&self) -> Result<(), anyhow::Error> {
        let start_time = std::time::Instant::now();

        let mut running = self.running.write().await;
        if !*running {
            return Ok(());
        }

        println!("Stopping agent system and all workers");

        *running = false;
        drop(running);

        // Signal shutdown
        let _ = self.shutdown_sender.send(true);
        let _ = self.event_sender.send(TaskEvent::SystemShutdown);

        // Log current system stats before shutdown
        let stats = self.get_system_stats().await;

        // Wait for all workers to finish
        let mut handles = self.worker_handles.lock().await;
        for handle in handles.drain(..) {
            let _ = handle.await;
        }

        let shutdown_duration = start_time.elapsed();

        println!(
            "Agent system stopped successfully in {}ms",
            shutdown_duration.as_millis()
        );

        Ok(())
    }

    /// Subscribe to task events
    pub fn subscribe_events(&self) -> broadcast::Receiver<TaskEvent> {
        self.event_sender.subscribe()
    }

    /// Check if the system is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Initialize the agent system with default agents
    pub async fn initialize(&self) -> Result<(), anyhow::Error> {
        use super::agent_types::{AnalysisAgent, CodeGenerationAgent, RefactoringAgent};

        if let Some(ai_manager) = &self.ai_manager {
            // Create code generation agent
            let code_agent = CodeGenerationAgent::with_ai_manager(ai_manager.clone());
            self.register_agent(Box::new(code_agent)).await?;

            // Create analysis agent
            let analysis_agent = AnalysisAgent::with_ai_manager(ai_manager.clone());
            self.register_agent(Box::new(analysis_agent)).await?;

            // Create refactoring agent
            let refactor_agent = RefactoringAgent::with_ai_manager(ai_manager.clone());
            self.register_agent(Box::new(refactor_agent)).await?;
        } else {
            // Create basic agents without AI
            let code_agent = CodeGenerationAgent::new();
            self.register_agent(Box::new(code_agent)).await?;

            let analysis_agent = AnalysisAgent::new();
            self.register_agent(Box::new(analysis_agent)).await?;

            let refactor_agent = RefactoringAgent::new();
            self.register_agent(Box::new(refactor_agent)).await?;
        }

        Ok(())
    }

    /// Register an agent with the system
    pub async fn register_agent(&self, agent: Box<dyn Agent>) -> Result<(), anyhow::Error> {
        let agent_id = agent.id().to_string();
        let capabilities = agent.capabilities();
        let agent_name = agent.name().to_string();

        println!(
            "Registering agent '{}' (ID: {}) with {} capabilities",
            agent_name,
            agent_id,
            capabilities.len()
        );

        {
            let mut agents = self.agents.write().await;
            agents.insert(agent_id.clone(), agent);
        }

        // Emit registration event
        let _ = self.event_sender.send(TaskEvent::AgentRegistered {
            agent_id: agent_id.clone(),
            capabilities,
        });

        // Metrics recording would go here

        Ok(())
    }

    /// Unregister an agent from the system
    pub async fn unregister_agent(&self, agent_id: &str) -> Result<(), anyhow::Error> {
        {
            let mut agents = self.agents.write().await;
            agents.remove(agent_id);
        }

        // Emit unregistration event
        let _ = self.event_sender.send(TaskEvent::AgentUnregistered {
            agent_id: agent_id.to_string(),
        });

        Ok(())
    }

    /// Get system statistics
    pub async fn get_system_stats(&self) -> SystemStats {
        let agents = self.agents.read().await;
        let active_tasks = self.active_tasks.read().await;
        let completed_tasks = self.completed_tasks.read().await;
        let queue = self.task_queue.lock().await;

        let stats = SystemStats {
            total_agents: agents.len(),
            active_tasks: active_tasks.len(),
            queued_tasks: queue.len(),
            completed_tasks: completed_tasks.len(),
        };

        // Emit current system metrics would go here

        stats
    }

    /// Get status of all agents
    pub async fn get_agent_statuses(&self) -> HashMap<String, AgentStatus> {
        let agents = self.agents.read().await;
        let mut statuses = HashMap::new();
        let mut status_counts = HashMap::new();

        for (agent_id, agent) in agents.iter() {
            let status = agent.status();
            statuses.insert(agent_id.clone(), status.clone());

            // Count status types for metrics
            let status_key = format!("{:?}", status).to_lowercase();
            *status_counts.entry(status_key).or_insert(0) += 1;
        }

        // Log agent status distribution would go here

        statuses
    }

    /// Get metrics for all agents
    pub async fn get_agent_metrics(&self) -> HashMap<String, AgentMetrics> {
        let agents = self.agents.read().await;
        let mut metrics = HashMap::new();
        let mut total_tasks = 0;
        let mut total_errors = 0;
        let mut total_processing_time = 0f64;

        for (agent_id, agent) in agents.iter() {
            let agent_metrics = agent.get_metrics();
            metrics.insert(agent_id.clone(), agent_metrics.clone());

            // Aggregate metrics
            total_tasks += agent_metrics.tasks_completed;
            total_errors += agent_metrics.tasks_failed;
            total_processing_time += agent_metrics.average_task_duration;

            // Emit per-agent metrics would go here
        }

        // Emit system-wide aggregated metrics would go here

        if total_tasks > 0 {
            let system_avg_processing_time = total_processing_time / total_tasks as f64;
            // Record system average processing time
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

    /// Get information about currently active tasks
    pub async fn get_active_tasks(&self) -> Vec<ActiveTaskInfo> {
        let active_tasks = self.active_tasks.read().await;
        let mut task_info = Vec::new();

        for (task_id, active_task) in active_tasks.iter() {
            task_info.push(ActiveTaskInfo {
                id: task_id.clone(),
                task_type: active_task.task.task_type.clone(),
                description: active_task.task.description.clone(),
                agent_id: active_task.agent_id.clone(),
                priority: active_task.task.priority.clone(),
                started_at: active_task.started_at,
                status: "Running".to_string(), // Could be more detailed
            });
        }

        task_info
    }

    // Background worker implementations
    async fn spawn_worker(&self, worker_id: usize) -> tokio::task::JoinHandle<()> {
        let task_queue = Arc::clone(&self.task_queue);
        let agents = Arc::clone(&self.agents);
        let active_tasks = Arc::clone(&self.active_tasks);
        let completed_tasks = Arc::clone(&self.completed_tasks);
        let failed_tasks = Arc::clone(&self.failed_tasks);
        let event_sender = self.event_sender.clone();
        let shutdown_receiver = self.shutdown_receiver.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            tracing::info!("Agent worker {} started", worker_id);
            
            let task_timeout = Duration::from_secs(config.task_timeout_seconds);
            
            loop {
                // Check for shutdown signal
                if *shutdown_receiver.borrow() {
                    tracing::info!("Worker {} shutting down", worker_id);
                    break;
                }

                // Try to get a task from the queue
                let prioritized_task = {
                    let mut queue = task_queue.lock().await;
                    queue.pop()
                };

                if let Some(prioritized_task) = prioritized_task {
                    let task = prioritized_task.task.clone();
                    let task_id = task.id.clone();
                    
                    // Find a suitable agent
                    let suitable_agent = {
                        let agents = agents.read().await;
                        agents.iter()
                            .find(|(_, agent)| {
                                agent.can_handle(&task.task_type) && 
                                agent.status() == AgentStatus::Idle
                            })
                            .map(|(id, _)| id.clone())
                    };

                    if let Some(agent_id) = suitable_agent {
                        // Get existing result_sender if the task was submitted with submit_task
                        let existing_result_sender = {
                            let mut active = active_tasks.write().await;
                            active.get_mut(&task_id).and_then(|existing_task| {
                                existing_task.result_sender.take()
                            })
                        };
                        
                        // Mark task as active with the correct agent and preserved result_sender
                        let active_task = ActiveTask {
                            task: task.clone(),
                            agent_id: agent_id.clone(),
                            started_at: Instant::now(),
                            result_sender: existing_result_sender,
                        };
                        
                        {
                            let mut active = active_tasks.write().await;
                            active.insert(task_id.clone(), active_task);
                        }

                        // Emit task started event
                        let _ = event_sender.send(TaskEvent::TaskStarted {
                            task_id: task_id.clone(),
                            agent_id: agent_id.clone(),
                        });

                        tracing::debug!("Worker {} processing task {} with agent {}", worker_id, task_id, agent_id);
                        
                        // Execute task with timeout
                        let task_start = Instant::now();
                        let task_result = {
                            let mut agents = agents.write().await;
                            if let Some(agent) = agents.get_mut(&agent_id) {
                                tokio::time::timeout(task_timeout, agent.process_task(task.clone())).await
                            } else {
                                Ok(Err(super::AgentError::TaskExecutionFailed("Agent not found".to_string())))
                            }
                        };
                        
                        let processing_duration = task_start.elapsed();
                        
                        // Remove from active tasks and get result_sender
                        let result_sender = {
                            let mut active = active_tasks.write().await;
                            active.remove(&task_id).and_then(|active_task| active_task.result_sender)
                        };

                        // Handle result
                        match task_result {
                            Ok(Ok(result)) => {
                                // Task completed successfully
                                {
                                    let mut completed = completed_tasks.write().await;
                                    completed.insert(task_id.clone(), TaskResult {
                                        result: result.clone(),
                                        completed_at: Instant::now(),
                                        processing_duration,
                                    });
                                    
                                    // Limit completed tasks history
                                    if completed.len() > config.task_history_limit {
                                        // Remove oldest entries (this is simplified - in production you'd use a proper LRU)
                                        let oldest_key = completed.keys().next().cloned();
                                        if let Some(key) = oldest_key {
                                            completed.remove(&key);
                                        }
                                    }
                                }
                                
                                // Send result back to caller if they're waiting
                                if let Some(sender) = result_sender {
                                    let _ = sender.send(Ok(result.clone()));
                                }
                                
                                let _ = event_sender.send(TaskEvent::TaskCompleted {
                                    task_id: task_id.clone(),
                                    agent_id: agent_id.clone(),
                                    success: true,
                                    duration_ms: processing_duration.as_millis() as u64,
                                });
                                
                                tracing::info!("Task {} completed successfully by agent {} in {}ms", 
                                    task_id, agent_id, processing_duration.as_millis());
                            },
                            Ok(Err(_)) | Err(_) => {
                                // Task failed
                                let error_msg = match &task_result {
                                    Ok(Err(e)) => e.to_string(),
                                    Err(_) => "Task timeout".to_string(),
                                    _ => unreachable!(),
                                };
                                
                                let will_retry = config.retry_failed_tasks;
                                
                                // Send error back to caller if they're waiting (unless retrying)
                                if let Some(sender) = result_sender {
                                    if !will_retry {
                                        let _ = sender.send(Err(anyhow::anyhow!(error_msg.clone())));
                                    }
                                    // If retrying, don't send error yet - let retry succeed or fail
                                }
                                
                                if will_retry {
                                    let mut failed = failed_tasks.write().await;
                                    let retry_count = failed.get(&task_id)
                                        .map(|f| f.retry_count + 1)
                                        .unwrap_or(1);
                                        
                                    if retry_count <= config.max_retry_attempts {
                                        let next_retry_at = Some(Instant::now() + Duration::from_secs(60 * retry_count as u64));
                                        failed.insert(task_id.clone(), FailedTask {
                                            task: task.clone(),
                                            error: error_msg.clone(),
                                            retry_count,
                                            last_attempt_at: Instant::now(),
                                            next_retry_at,
                                        });
                                    }
                                }
                                
                                let _ = event_sender.send(TaskEvent::TaskFailed {
                                    task_id: task_id.clone(),
                                    agent_id: agent_id.clone(),
                                    error: error_msg.clone(),
                                    will_retry,
                                });
                                
                                tracing::warn!("Task {} failed on agent {}: {}", task_id, agent_id, error_msg);
                            }
                        }
                    } else {
                        // No suitable agent available, put task back in queue
                        // Clone the task to avoid ownership issues
                        let new_prioritized_task = PrioritizedTask {
                            task: prioritized_task.task.clone(),
                            priority_score: prioritized_task.priority_score,
                            submitted_at: prioritized_task.submitted_at,
                            deadline_score: prioritized_task.deadline_score,
                        };
                        {
                            let mut queue = task_queue.lock().await;
                            queue.push(new_prioritized_task);
                        }
                        
                        // Wait a bit before trying again
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                } else {
                    // No tasks in queue, wait a bit
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }
            
            tracing::info!("Agent worker {} stopped", worker_id);
        })
    }

    async fn spawn_retry_handler(&self) -> tokio::task::JoinHandle<()> {
        let task_queue = Arc::clone(&self.task_queue);
        let failed_tasks = Arc::clone(&self.failed_tasks);
        let event_sender = self.event_sender.clone();
        let mut shutdown_receiver = self.shutdown_receiver.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            tracing::info!("Retry handler started");
            
            let mut retry_interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                tokio::select! {
                    _ = retry_interval.tick() => {
                        // Check for tasks ready to retry
                        let now = Instant::now();
                        let mut tasks_to_retry = Vec::new();
                        
                        {
                            let mut failed = failed_tasks.write().await;
                            let mut to_remove = Vec::new();
                            
                            for (task_id, failed_task) in failed.iter() {
                                if let Some(next_retry_at) = failed_task.next_retry_at {
                                    if now >= next_retry_at && failed_task.retry_count <= config.max_retry_attempts {
                                        tasks_to_retry.push((task_id.clone(), failed_task.task.clone(), failed_task.retry_count));
                                        to_remove.push(task_id.clone());
                                    } else if failed_task.retry_count > config.max_retry_attempts {
                                        // Max retries exceeded, remove from failed tasks
                                        to_remove.push(task_id.clone());
                                        tracing::warn!("Task {} exceeded max retry attempts ({})", task_id, config.max_retry_attempts);
                                    }
                                }
                            }
                            
                            for task_id in to_remove {
                                failed.remove(&task_id);
                            }
                        }
                        
                        // Re-queue tasks for retry
                        for (task_id, task, retry_count) in tasks_to_retry {
                            let prioritized_task = PrioritizedTask {
                                task,
                                priority_score: 100, // Higher priority for retries
                                submitted_at: Instant::now(),
                                deadline_score: 100,
                            };
                            
                            {
                                let mut queue = task_queue.lock().await;
                                queue.push(prioritized_task);
                            }
                            
                            let _ = event_sender.send(TaskEvent::TaskRetried {
                                task_id: task_id.clone(),
                                retry_count,
                            });
                            
                            tracing::info!("Retrying task {} (attempt {})", task_id, retry_count + 1);
                        }
                    },
                    _ = shutdown_receiver.changed() => {
                        if *shutdown_receiver.borrow() {
                            tracing::info!("Retry handler shutting down");
                            break;
                        }
                    }
                }
            }
            
            tracing::info!("Retry handler stopped");
        })
    }

    async fn spawn_cleanup_handler(&self) -> tokio::task::JoinHandle<()> {
        let completed_tasks = Arc::clone(&self.completed_tasks);
        let failed_tasks = Arc::clone(&self.failed_tasks);
        let mut shutdown_receiver = self.shutdown_receiver.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            tracing::info!("Cleanup handler started");
            
            let mut cleanup_interval = tokio::time::interval(Duration::from_secs(300)); // Every 5 minutes
            
            loop {
                tokio::select! {
                    _ = cleanup_interval.tick() => {
                        let now = Instant::now();
                        let retention_period = Duration::from_secs(3600); // Keep completed tasks for 1 hour
                        
                        // Clean up old completed tasks
                        {
                            let mut completed = completed_tasks.write().await;
                            let initial_count = completed.len();
                            
                            completed.retain(|_, task_result| {
                                now.duration_since(task_result.completed_at) < retention_period
                            });
                            
                            let cleaned_count = initial_count - completed.len();
                            if cleaned_count > 0 {
                                tracing::debug!("Cleaned up {} old completed tasks", cleaned_count);
                            }
                        }
                        
                        // Clean up very old failed tasks (beyond retry attempts)
                        {
                            let mut failed = failed_tasks.write().await;
                            let initial_count = failed.len();
                            
                            failed.retain(|_, failed_task| {
                                // Keep failed tasks for retry, but remove very old ones
                                let age = now.duration_since(failed_task.last_attempt_at);
                                age < Duration::from_secs(7200) && // 2 hours max
                                failed_task.retry_count <= config.max_retry_attempts
                            });
                            
                            let cleaned_count = initial_count - failed.len();
                            if cleaned_count > 0 {
                                tracing::debug!("Cleaned up {} old failed tasks", cleaned_count);
                            }
                        }
                    },
                    _ = shutdown_receiver.changed() => {
                        if *shutdown_receiver.borrow() {
                            tracing::info!("Cleanup handler shutting down");
                            break;
                        }
                    }
                }
            }
            
            tracing::info!("Cleanup handler stopped");
        })
    }

    async fn spawn_heartbeat_monitor(&self) -> tokio::task::JoinHandle<()> {
        let agents = Arc::clone(&self.agents);
        let active_tasks = Arc::clone(&self.active_tasks);
        let event_sender = self.event_sender.clone();
        let mut shutdown_receiver = self.shutdown_receiver.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            tracing::info!("Heartbeat monitor started");
            
            let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(config.heartbeat_interval_seconds));
            
            loop {
                tokio::select! {
                    _ = heartbeat_interval.tick() => {
                        let now = Instant::now();
                        let task_timeout = Duration::from_secs(config.task_timeout_seconds);
                        
                        // Check for timed out tasks
                        let timed_out_tasks = {
                            let active = active_tasks.read().await;
                            active.iter()
                                .filter(|(_, task)| now.duration_since(task.started_at) > task_timeout)
                                .map(|(id, task)| (id.clone(), task.agent_id.clone()))
                                .collect::<Vec<_>>()
                        };
                        
                        // Handle timed out tasks
                        for (task_id, agent_id) in timed_out_tasks {
                            tracing::warn!("Task {} timed out on agent {}", task_id, agent_id);
                            
                            // Remove from active tasks
                            {
                                let mut active = active_tasks.write().await;
                                active.remove(&task_id);
                            }
                            
                            // Reset agent status to idle (if possible)
                            {
                                let mut agents = agents.write().await;
                                if let Some(_agent) = agents.get_mut(&agent_id) {
                                    // Note: We can't directly set status as it's managed by the Agent trait
                                    // In a real implementation, you'd need a way to signal timeout to agents
                                }
                            }
                            
                            let _ = event_sender.send(TaskEvent::TaskTimeout {
                                task_id,
                                agent_id,
                            });
                        }
                        
                        // Log system health metrics
                        let (agent_count, active_task_count) = {
                            let agents = agents.read().await;
                            let active = active_tasks.read().await;
                            (agents.len(), active.len())
                        };
                        
                        if active_task_count > 0 {
                            tracing::debug!(
                                "System heartbeat: {} agents registered, {} active tasks",
                                agent_count, active_task_count
                            );
                        }
                    },
                    _ = shutdown_receiver.changed() => {
                        if *shutdown_receiver.borrow() {
                            tracing::info!("Heartbeat monitor shutting down");
                            break;
                        }
                    }
                }
            }
            
            tracing::info!("Heartbeat monitor stopped");
        })
    }

    /// Submit a task to the system asynchronously
    pub async fn submit_task(&self, task: AgentTask) -> Result<AgentResult, anyhow::Error> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.submit_task_async(task, Some(result_sender)).await?;

        result_receiver
            .await
            .map_err(|_| anyhow::anyhow!("Task result channel closed"))?
            .map_err(|e| e.into())
    }

    /// Submit a task without waiting for result
    pub async fn submit_task_fire_and_forget(
        &self,
        task: AgentTask,
    ) -> Result<String, anyhow::Error> {
        let task_id = task.id.clone();
        self.submit_task_async(task, None).await?;
        Ok(task_id)
    }

    /// Internal task submission with optional result channel
    async fn submit_task_async(
        &self,
        task: AgentTask,
        result_sender: Option<oneshot::Sender<Result<AgentResult, anyhow::Error>>>,
    ) -> Result<(), anyhow::Error> {
        let task_id = task.id.clone();
        let task_type = task.task_type.clone();
        let priority = task.priority.clone();

        // Check if system is running
        if !self.is_running().await {
            println!("Task submission rejected: agent system not running");
            return Err(anyhow::anyhow!("Agent system is not running"));
        }

        // Check queue size limit
        let current_queue_size = {
            let queue = self.task_queue.lock().await;
            if queue.len() >= self.config.max_queue_size {
                println!(
                    "Task queue size limit {} reached, rejecting task {}",
                    self.config.max_queue_size, task_id
                );

                return Err(anyhow::anyhow!(
                    "Task queue size limit {} reached",
                    self.config.max_queue_size
                ));
            }
            queue.len()
        };

        let priority_score = self.calculate_priority_score(&task);
        let deadline_score = self.calculate_deadline_score(&task);

        println!(
            "Submitting task '{}' (type: {}, priority: {:?}) to queue",
            task_id, task_type, priority
        );

        let prioritized_task = PrioritizedTask {
            task: task.clone(),
            priority_score,
            submitted_at: Instant::now(),
            deadline_score,
        };

        // Add to queue
        {
            let mut queue = self.task_queue.lock().await;
            queue.push(prioritized_task);
        }

        // Store result sender if provided
        if let Some(sender) = result_sender {
            let active_task = ActiveTask {
                task: task.clone(),
                agent_id: String::new(), // Will be set when picked up
                started_at: Instant::now(),
                result_sender: Some(sender),
            };

            let mut active_tasks = self.active_tasks.write().await;
            active_tasks.insert(task.id.clone(), active_task);
        }

        // Emit task submitted event
        let _ = self.event_sender.send(TaskEvent::TaskSubmitted {
            task_id: task_id.clone(),
            task_type: task_type.clone(),
            priority: priority.clone(),
        });

        // Metrics recording would go here

        Ok(())
    }

    /// Calculate priority score for a task
    fn calculate_priority_score(&self, task: &AgentTask) -> u32 {
        match task.priority {
            TaskPriority::Critical => 10000,
            TaskPriority::High => 1000,
            TaskPriority::Normal => 100,
            TaskPriority::Low => 10,
        }
    }

    /// Calculate deadline urgency score for a task
    fn calculate_deadline_score(&self, task: &AgentTask) -> u32 {
        if let Some(deadline) = &task.deadline {
            let now = chrono::Utc::now();
            let time_until_deadline = deadline.signed_duration_since(now);

            if time_until_deadline.num_seconds() < 0 {
                // Already overdue - highest urgency
                return 100000;
            }

            let minutes_until = time_until_deadline.num_minutes();
            match minutes_until {
                0..=5 => 50000,   // Very urgent
                6..=15 => 10000,  // Urgent
                16..=60 => 5000,  // Moderately urgent
                61..=240 => 1000, // Some urgency
                _ => 100,         // Normal
            }
        } else {
            100 // Normal urgency for tasks without deadlines
        }
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

/// Active task information for UI display
#[derive(Debug, Clone)]
pub struct ActiveTaskInfo {
    pub id: String,
    pub task_type: String,
    pub description: String,
    pub agent_id: String,
    pub priority: TaskPriority,
    pub started_at: std::time::Instant,
    pub status: String,
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
