//! Agent Progress Tracking System
//!
//! This module provides comprehensive progress tracking for agent operations,
//! integrating with the UI progress indicator system for real-time feedback.

use crate::agents::{Agent, AgentTask, TaskPriority};
use crate::ui::progress::{ProgressManager, ProgressStyle, ProgressTracker};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info, trace};
use uuid::Uuid;

/// Agent progress tracking manager
#[derive(Debug)]
pub struct AgentProgressTracker {
    /// Progress manager for UI integration
    progress_manager: Arc<ProgressManager>,
    
    /// Active agent operations
    active_operations: Arc<RwLock<HashMap<String, AgentOperation>>>,
    
    /// Task progress tracking
    task_progress: Arc<RwLock<HashMap<String, TaskProgress>>>,
    
    /// Agent performance history
    agent_history: Arc<RwLock<HashMap<String, AgentPerformanceHistory>>>,
    
    /// Progress update broadcaster
    update_sender: broadcast::Sender<AgentProgressUpdate>,
    
    /// Configuration
    config: ProgressTrackerConfig,
}

/// Individual agent operation being tracked
#[derive(Debug, Clone)]
pub struct AgentOperation {
    pub operation_id: String,
    pub agent_id: String,
    pub agent_name: String,
    pub task_id: String,
    pub task_type: String,
    pub task_description: String,
    pub priority: TaskPriority,
    pub started_at: Instant,
    pub estimated_duration: Option<Duration>,
    pub current_step: usize,
    pub total_steps: usize,
    pub steps: Vec<OperationStep>,
    pub progress_tracker: Option<ProgressTracker>,
    pub status: OperationStatus,
    pub resource_usage: ResourceUsage,
}

/// Individual step in an agent operation
#[derive(Debug, Clone)]
pub struct OperationStep {
    pub name: String,
    pub description: Option<String>,
    pub progress: f64, // 0.0 to 1.0
    pub status: StepStatus,
    pub started_at: Option<Instant>,
    pub completed_at: Option<Instant>,
    pub estimated_duration: Option<Duration>,
}

/// Status of an operation step
#[derive(Debug, Clone, PartialEq)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
    Skipped,
}

/// Overall operation status
#[derive(Debug, Clone, PartialEq)]
pub enum OperationStatus {
    Starting,
    Running,
    Paused,
    Completed,
    Failed(String),
    Cancelled,
}

/// Task progress tracking information
#[derive(Debug, Clone)]
pub struct TaskProgress {
    pub task_id: String,
    pub task_type: String,
    pub agent_id: String,
    pub overall_progress: f64,
    pub current_phase: String,
    pub phases: Vec<TaskPhase>,
    pub started_at: Instant,
    pub estimated_completion: Option<Instant>,
    pub metrics: TaskMetrics,
}

/// Individual phase of task execution
#[derive(Debug, Clone)]
pub struct TaskPhase {
    pub name: String,
    pub description: String,
    pub weight: f64, // Relative weight of this phase (0.0 to 1.0)
    pub progress: f64, // Progress within this phase (0.0 to 1.0)
    pub status: StepStatus,
    pub started_at: Option<Instant>,
    pub completed_at: Option<Instant>,
}

/// Task execution metrics
#[derive(Debug, Clone)]
pub struct TaskMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: u64,
    pub io_operations: u64,
    pub api_calls_made: u64,
    pub tokens_processed: u64,
    pub files_analyzed: usize,
    pub lines_processed: usize,
}

/// Agent performance history for prediction and optimization
#[derive(Debug, Clone)]
pub struct AgentPerformanceHistory {
    pub agent_id: String,
    pub agent_name: String,
    pub task_completions: Vec<TaskCompletion>,
    pub average_duration_by_type: HashMap<String, Duration>,
    pub success_rate_by_type: HashMap<String, f64>,
    pub resource_usage_patterns: HashMap<String, ResourceUsagePattern>,
    pub last_updated: Instant,
}

/// Completed task information for performance analysis
#[derive(Debug, Clone)]
pub struct TaskCompletion {
    pub task_id: String,
    pub task_type: String,
    pub priority: TaskPriority,
    pub started_at: Instant,
    pub completed_at: Instant,
    pub duration: Duration,
    pub success: bool,
    pub error_type: Option<String>,
    pub final_metrics: TaskMetrics,
}

/// Resource usage patterns for prediction
#[derive(Debug, Clone)]
pub struct ResourceUsagePattern {
    pub average_cpu_percent: f64,
    pub peak_cpu_percent: f64,
    pub average_memory_mb: u64,
    pub peak_memory_mb: u64,
    pub typical_duration: Duration,
    pub sample_count: usize,
}

/// Resource usage information
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub cpu_percent: f64,
    pub memory_mb: u64,
    pub network_bytes_sent: u64,
    pub network_bytes_received: u64,
    pub disk_bytes_read: u64,
    pub disk_bytes_written: u64,
    pub timestamp: Instant,
}

/// Progress update events
#[derive(Debug, Clone)]
pub enum AgentProgressUpdate {
    OperationStarted {
        operation_id: String,
        agent_id: String,
        task_type: String,
        estimated_duration: Option<Duration>,
    },
    StepStarted {
        operation_id: String,
        step_index: usize,
        step_name: String,
    },
    ProgressUpdated {
        operation_id: String,
        overall_progress: f64,
        current_step: usize,
        step_progress: f64,
        message: Option<String>,
    },
    StepCompleted {
        operation_id: String,
        step_index: usize,
        step_name: String,
        duration: Duration,
    },
    OperationCompleted {
        operation_id: String,
        agent_id: String,
        success: bool,
        total_duration: Duration,
        final_metrics: TaskMetrics,
    },
    ResourceUsageUpdated {
        agent_id: String,
        usage: ResourceUsage,
    },
    PerformancePredictionUpdated {
        agent_id: String,
        task_type: String,
        estimated_duration: Duration,
        confidence: f64,
    },
}

/// Configuration for progress tracking
#[derive(Debug, Clone)]
pub struct ProgressTrackerConfig {
    pub update_interval: Duration,
    pub resource_monitoring_enabled: bool,
    pub performance_prediction_enabled: bool,
    pub history_retention_days: u32,
    pub max_concurrent_operations: usize,
    pub detailed_step_tracking: bool,
    pub ui_update_throttle_ms: u64,
}

impl Default for ProgressTrackerConfig {
    fn default() -> Self {
        Self {
            update_interval: Duration::from_millis(500),
            resource_monitoring_enabled: true,
            performance_prediction_enabled: true,
            history_retention_days: 30,
            max_concurrent_operations: 100,
            detailed_step_tracking: true,
            ui_update_throttle_ms: 100,
        }
    }
}

impl AgentProgressTracker {
    /// Create a new agent progress tracker
    pub fn new(progress_manager: Arc<ProgressManager>) -> Self {
        let (update_sender, _) = broadcast::channel(1000);
        
        Self {
            progress_manager,
            active_operations: Arc::new(RwLock::new(HashMap::new())),
            task_progress: Arc::new(RwLock::new(HashMap::new())),
            agent_history: Arc::new(RwLock::new(HashMap::new())),
            update_sender,
            config: ProgressTrackerConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(progress_manager: Arc<ProgressManager>, config: ProgressTrackerConfig) -> Self {
        let mut tracker = Self::new(progress_manager);
        tracker.config = config;
        tracker
    }

    /// Subscribe to progress updates
    pub fn subscribe_updates(&self) -> broadcast::Receiver<AgentProgressUpdate> {
        self.update_sender.subscribe()
    }

    /// Start tracking an agent operation
    pub async fn start_operation(
        &self,
        agent_id: String,
        agent_name: String,
        task: &AgentTask,
        steps: Vec<String>,
        estimated_duration: Option<Duration>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let operation_id = Uuid::new_v4().to_string();
        
        // Predict duration if not provided
        let predicted_duration = if estimated_duration.is_none() && self.config.performance_prediction_enabled {
            self.predict_task_duration(&agent_id, &task.task_type).await
        } else {
            estimated_duration
        };

        // Create operation steps
        let step_count = steps.len();
        let operation_steps: Vec<OperationStep> = steps.into_iter().enumerate().map(|(i, name)| {
            OperationStep {
                name,
                description: None,
                progress: 0.0,
                status: if i == 0 { StepStatus::Pending } else { StepStatus::Pending },
                started_at: None,
                completed_at: None,
                estimated_duration: predicted_duration.map(|d| Duration::from_millis(d.as_millis() as u64 / step_count as u64)),
            }
        }).collect();

        // Start UI progress tracking
        let progress_style = self.determine_progress_style(&task.task_type);
        let progress_tracker = self.progress_manager.start_operation(
            format!("{}: {}", agent_name, task.description),
            Some(format!("Agent: {} | Type: {}", agent_name, task.task_type)),
            progress_style,
            predicted_duration,
            operation_steps.iter().map(|s| s.name.clone()).collect(),
        ).await;

        // Create operation record
        let operation = AgentOperation {
            operation_id: operation_id.clone(),
            agent_id: agent_id.clone(),
            agent_name: agent_name.clone(),
            task_id: task.id.clone(),
            task_type: task.task_type.clone(),
            task_description: task.description.clone(),
            priority: task.priority,
            started_at: Instant::now(),
            estimated_duration: predicted_duration,
            current_step: 0,
            total_steps: operation_steps.len(),
            steps: operation_steps,
            progress_tracker: Some(progress_tracker),
            status: OperationStatus::Starting,
            resource_usage: ResourceUsage {
                cpu_percent: 0.0,
                memory_mb: 0,
                network_bytes_sent: 0,
                network_bytes_received: 0,
                disk_bytes_read: 0,
                disk_bytes_written: 0,
                timestamp: Instant::now(),
            },
        };

        // Store the operation
        {
            let mut operations = self.active_operations.write().await;
            operations.insert(operation_id.clone(), operation);
        }

        // Send progress update
        let _ = self.update_sender.send(AgentProgressUpdate::OperationStarted {
            operation_id: operation_id.clone(),
            agent_id,
            task_type: task.task_type.clone(),
            estimated_duration: predicted_duration,
        });

        trace!("Started tracking operation {} for agent {}", operation_id, agent_name);
        Ok(operation_id)
    }

    /// Update operation progress
    pub async fn update_progress(
        &self,
        operation_id: &str,
        step_index: Option<usize>,
        step_progress: f64,
        message: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut operations = self.active_operations.write().await;
        
        if let Some(operation) = operations.get_mut(operation_id) {
            // Update step progress if specified
            if let Some(step_idx) = step_index {
                if step_idx < operation.steps.len() {
                    operation.steps[step_idx].progress = step_progress.clamp(0.0, 1.0);
                    
                    // Mark step as running if it was pending
                    if operation.steps[step_idx].status == StepStatus::Pending {
                        operation.steps[step_idx].status = StepStatus::Running;
                        operation.steps[step_idx].started_at = Some(Instant::now());
                        
                        // Update current step
                        operation.current_step = step_idx;
                        
                        // Send step started event
                        let _ = self.update_sender.send(AgentProgressUpdate::StepStarted {
                            operation_id: operation_id.to_string(),
                            step_index: step_idx,
                            step_name: operation.steps[step_idx].name.clone(),
                        });
                    }
                }
            }

            // Calculate overall progress
            let overall_progress = self.calculate_overall_progress(operation);
            
            // Update UI progress tracker
            if let Some(ref tracker) = operation.progress_tracker {
                tracker.update_progress(overall_progress, message.clone());
                
                if let Some(step_idx) = step_index {
                    tracker.update_step(step_idx, step_progress, message.clone());
                }
            }

            // Send progress update event
            let _ = self.update_sender.send(AgentProgressUpdate::ProgressUpdated {
                operation_id: operation_id.to_string(),
                overall_progress,
                current_step: operation.current_step,
                step_progress,
                message,
            });

            trace!("Updated progress for operation {}: {:.1}%", operation_id, overall_progress * 100.0);
        }

        Ok(())
    }

    /// Complete a step in the operation
    pub async fn complete_step(
        &self,
        operation_id: &str,
        step_index: usize,
        success: bool,
        message: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut operations = self.active_operations.write().await;
        
        if let Some(operation) = operations.get_mut(operation_id) {
            if step_index < operation.steps.len() {
                let step = &mut operation.steps[step_index];
                let duration = step.started_at.map(|start| start.elapsed()).unwrap_or_default();
                
                step.progress = 1.0;
                step.status = if success {
                    StepStatus::Completed
                } else {
                    StepStatus::Failed(message.clone().unwrap_or_else(|| "Step failed".to_string()))
                };
                step.completed_at = Some(Instant::now());

                // Update UI tracker
                if let Some(ref tracker) = operation.progress_tracker {
                    tracker.update_step(step_index, 1.0, message.clone());
                }

                // Send step completed event
                let _ = self.update_sender.send(AgentProgressUpdate::StepCompleted {
                    operation_id: operation_id.to_string(),
                    step_index,
                    step_name: step.name.clone(),
                    duration,
                });

                // Move to next step if this one succeeded
                if success && step_index + 1 < operation.steps.len() {
                    operation.current_step = step_index + 1;
                }

                debug!("Completed step {} for operation {}: {} in {:?}", 
                       step_index, operation_id, if success { "success" } else { "failed" }, duration);
            }
        }

        Ok(())
    }

    /// Complete the entire operation
    pub async fn complete_operation(
        &self,
        operation_id: &str,
        success: bool,
        final_message: Option<String>,
        final_metrics: Option<TaskMetrics>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut operations = self.active_operations.write().await;
        
        if let Some(operation) = operations.remove(operation_id) {
            let total_duration = operation.started_at.elapsed();
            let metrics = final_metrics.unwrap_or_else(|| TaskMetrics {
                cpu_usage_percent: operation.resource_usage.cpu_percent,
                memory_usage_mb: operation.resource_usage.memory_mb,
                io_operations: 0,
                api_calls_made: 0,
                tokens_processed: 0,
                files_analyzed: 0,
                lines_processed: 0,
            });

            // Complete UI progress tracker
            if let Some(ref tracker) = operation.progress_tracker {
                if success {
                    tracker.complete(final_message.clone());
                } else {
                    tracker.fail(final_message.unwrap_or_else(|| "Operation failed".to_string()));
                }
            }

            // Update performance history
            self.update_performance_history(&operation, success, &metrics).await;

            // Send completion event
            let _ = self.update_sender.send(AgentProgressUpdate::OperationCompleted {
                operation_id: operation_id.to_string(),
                agent_id: operation.agent_id.clone(),
                success,
                total_duration,
                final_metrics: metrics,
            });

            info!("Completed operation {} for agent {} in {:?}: {}", 
                  operation_id, operation.agent_name, total_duration, 
                  if success { "success" } else { "failed" });
        }

        Ok(())
    }

    /// Update resource usage for an operation
    pub async fn update_resource_usage(
        &self,
        operation_id: &str,
        usage: ResourceUsage,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.resource_monitoring_enabled {
            return Ok(());
        }

        let mut operations = self.active_operations.write().await;
        
        if let Some(operation) = operations.get_mut(operation_id) {
            operation.resource_usage = usage.clone();
            
            // Send resource update event
            let _ = self.update_sender.send(AgentProgressUpdate::ResourceUsageUpdated {
                agent_id: operation.agent_id.clone(),
                usage,
            });
        }

        Ok(())
    }

    /// Get current operation status
    pub async fn get_operation_status(&self, operation_id: &str) -> Option<AgentOperation> {
        let operations = self.active_operations.read().await;
        operations.get(operation_id).cloned()
    }

    /// Get all active operations
    pub async fn get_active_operations(&self) -> Vec<AgentOperation> {
        let operations = self.active_operations.read().await;
        operations.values().cloned().collect()
    }

    /// Get operations for a specific agent
    pub async fn get_agent_operations(&self, agent_id: &str) -> Vec<AgentOperation> {
        let operations = self.active_operations.read().await;
        operations.values()
            .filter(|op| op.agent_id == agent_id)
            .cloned()
            .collect()
    }

    /// Predict task duration based on historical data
    async fn predict_task_duration(&self, agent_id: &str, task_type: &str) -> Option<Duration> {
        let history = self.agent_history.read().await;
        
        if let Some(agent_history) = history.get(agent_id) {
            if let Some(avg_duration) = agent_history.average_duration_by_type.get(task_type) {
                // Add some variance based on historical data
                let base_duration = *avg_duration;
                let variance = base_duration.as_millis() as f64 * 0.2; // 20% variance
                let predicted_ms = base_duration.as_millis() as f64 + variance;
                
                return Some(Duration::from_millis(predicted_ms as u64));
            }
        }

        // Default estimates based on task type
        let default_duration = match task_type {
            "generate_code" => Duration::from_secs(30),
            "analyze_file" => Duration::from_secs(10),
            "refactor" => Duration::from_secs(45),
            "review_code" => Duration::from_secs(20),
            "test_generation" => Duration::from_secs(25),
            "documentation" => Duration::from_secs(15),
            _ => Duration::from_secs(30),
        };

        Some(default_duration)
    }

    /// Determine appropriate progress style for task type
    fn determine_progress_style(&self, task_type: &str) -> ProgressStyle {
        match task_type {
            "generate_code" | "refactor" => ProgressStyle::Steps,
            "analyze_file" | "review_code" => ProgressStyle::Bar,
            "test_generation" | "documentation" => ProgressStyle::Line,
            _ => ProgressStyle::Spinner,
        }
    }

    /// Calculate overall progress for an operation
    fn calculate_overall_progress(&self, operation: &AgentOperation) -> f64 {
        if operation.steps.is_empty() {
            return 0.0;
        }

        let total_progress: f64 = operation.steps.iter()
            .map(|step| step.progress)
            .sum();

        total_progress / operation.steps.len() as f64
    }

    /// Update performance history for an agent
    async fn update_performance_history(
        &self,
        operation: &AgentOperation,
        success: bool,
        metrics: &TaskMetrics,
    ) {
        let mut history = self.agent_history.write().await;
        
        let agent_history = history.entry(operation.agent_id.clone())
            .or_insert_with(|| AgentPerformanceHistory {
                agent_id: operation.agent_id.clone(),
                agent_name: operation.agent_name.clone(),
                task_completions: Vec::new(),
                average_duration_by_type: HashMap::new(),
                success_rate_by_type: HashMap::new(),
                resource_usage_patterns: HashMap::new(),
                last_updated: Instant::now(),
            });

        // Add completion record
        let completion = TaskCompletion {
            task_id: operation.task_id.clone(),
            task_type: operation.task_type.clone(),
            priority: operation.priority,
            started_at: operation.started_at,
            completed_at: Instant::now(),
            duration: operation.started_at.elapsed(),
            success,
            error_type: None,
            final_metrics: metrics.clone(),
        };

        agent_history.task_completions.push(completion);

        // Update averages
        self.update_performance_averages(agent_history, &operation.task_type);

        agent_history.last_updated = Instant::now();
    }

    /// Update performance averages for an agent
    fn update_performance_averages(&self, history: &mut AgentPerformanceHistory, task_type: &str) {
        let task_completions: Vec<_> = history.task_completions.iter()
            .filter(|c| c.task_type == task_type)
            .collect();

        if !task_completions.is_empty() {
            // Calculate average duration
            let total_duration: Duration = task_completions.iter()
                .map(|c| c.duration)
                .sum();
            let avg_duration = total_duration / task_completions.len() as u32;
            history.average_duration_by_type.insert(task_type.to_string(), avg_duration);

            // Calculate success rate
            let successful_tasks = task_completions.iter()
                .filter(|c| c.success)
                .count();
            let success_rate = successful_tasks as f64 / task_completions.len() as f64;
            history.success_rate_by_type.insert(task_type.to_string(), success_rate);

            // Update resource usage patterns
            let avg_cpu = task_completions.iter()
                .map(|c| c.final_metrics.cpu_usage_percent)
                .sum::<f64>() / task_completions.len() as f64;
            
            let avg_memory = task_completions.iter()
                .map(|c| c.final_metrics.memory_usage_mb)
                .sum::<u64>() / task_completions.len() as u64;

            let pattern = ResourceUsagePattern {
                average_cpu_percent: avg_cpu,
                peak_cpu_percent: task_completions.iter()
                    .map(|c| c.final_metrics.cpu_usage_percent)
                    .fold(0.0, f64::max),
                average_memory_mb: avg_memory,
                peak_memory_mb: task_completions.iter()
                    .map(|c| c.final_metrics.memory_usage_mb)
                    .max()
                    .unwrap_or(0),
                typical_duration: avg_duration,
                sample_count: task_completions.len(),
            };

            history.resource_usage_patterns.insert(task_type.to_string(), pattern);
        }
    }

    /// Start resource monitoring background task
    pub async fn start_monitoring(&self) -> tokio::task::JoinHandle<()> {
        let operations = self.active_operations.clone();
        let update_sender = self.update_sender.clone();
        let interval = self.config.update_interval;
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                let ops = operations.read().await;
                for operation in ops.values() {
                    // Here you would integrate with actual system resource monitoring
                    // For now, we'll simulate resource usage updates with simple values
                    let elapsed_ms = operation.started_at.elapsed().as_millis() as u64;
                    let usage = ResourceUsage {
                        cpu_percent: 25.0 + (elapsed_ms % 50) as f64, // Simulated CPU usage
                        memory_mb: 100 + (elapsed_ms % 256), // Simulated memory usage
                        network_bytes_sent: elapsed_ms % 1024,
                        network_bytes_received: elapsed_ms % 1024,
                        disk_bytes_read: elapsed_ms % 4096,
                        disk_bytes_written: elapsed_ms % 4096,
                        timestamp: Instant::now(),
                    };

                    let _ = update_sender.send(AgentProgressUpdate::ResourceUsageUpdated {
                        agent_id: operation.agent_id.clone(),
                        usage,
                    });
                }
            }
        })
    }

    /// Clean up old operations and history
    pub async fn cleanup_old_data(&self) {
        let cutoff = Instant::now() - Duration::from_secs(self.config.history_retention_days as u64 * 24 * 60 * 60);
        
        let mut history = self.agent_history.write().await;
        for agent_history in history.values_mut() {
            agent_history.task_completions.retain(|completion| {
                completion.completed_at > cutoff
            });
        }
        
        // Remove agents with no recent history
        history.retain(|_, agent_history| {
            !agent_history.task_completions.is_empty()
        });
    }
}

impl Default for TaskMetrics {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 0.0,
            memory_usage_mb: 0,
            io_operations: 0,
            api_calls_made: 0,
            tokens_processed: 0,
            files_analyzed: 0,
            lines_processed: 0,
        }
    }
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            cpu_percent: 0.0,
            memory_mb: 0,
            network_bytes_sent: 0,
            network_bytes_received: 0,
            disk_bytes_read: 0,
            disk_bytes_written: 0,
            timestamp: Instant::now(),
        }
    }
}

/// Extension trait for agents to integrate with progress tracking
#[async_trait::async_trait]
pub trait AgentProgressExtension {
    /// Start progress tracking for a task
    async fn start_progress_tracking(
        &self,
        progress_tracker: &AgentProgressTracker,
        task: &AgentTask,
        steps: Vec<String>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;

    /// Update progress during task execution
    async fn update_task_progress(
        &self,
        progress_tracker: &AgentProgressTracker,
        operation_id: &str,
        step_index: Option<usize>,
        progress: f64,
        message: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Complete progress tracking
    async fn complete_progress_tracking(
        &self,
        progress_tracker: &AgentProgressTracker,
        operation_id: &str,
        success: bool,
        final_message: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

// Implement the extension trait for all agents
#[async_trait::async_trait]
impl<T: Agent> AgentProgressExtension for T {
    async fn start_progress_tracking(
        &self,
        progress_tracker: &AgentProgressTracker,
        task: &AgentTask,
        steps: Vec<String>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        progress_tracker.start_operation(
            self.id().to_string(),
            self.name().to_string(),
            task,
            steps,
            None,
        ).await
    }

    async fn update_task_progress(
        &self,
        progress_tracker: &AgentProgressTracker,
        operation_id: &str,
        step_index: Option<usize>,
        progress: f64,
        message: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        progress_tracker.update_progress(operation_id, step_index, progress, message).await
    }

    async fn complete_progress_tracking(
        &self,
        progress_tracker: &AgentProgressTracker,
        operation_id: &str,
        success: bool,
        final_message: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        progress_tracker.complete_operation(operation_id, success, final_message, None).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_progress_tracker_creation() {
        let progress_manager = Arc::new(ProgressManager::new());
        let tracker = AgentProgressTracker::new(progress_manager);
        
        let operations = tracker.get_active_operations().await;
        assert!(operations.is_empty());
    }

    #[tokio::test]
    async fn test_operation_lifecycle() {
        let progress_manager = Arc::new(ProgressManager::new());
        let tracker = AgentProgressTracker::new(progress_manager);
        
        let task = AgentTask::new(
            "test_task".to_string(),
            "Test task description".to_string(),
            serde_json::Value::Null,
        );

        let steps = vec!["Step 1".to_string(), "Step 2".to_string()];
        let operation_id = tracker.start_operation(
            "agent_1".to_string(),
            "Test Agent".to_string(),
            &task,
            steps,
            Some(Duration::from_secs(30)),
        ).await.unwrap();

        // Check operation was created
        let operation = tracker.get_operation_status(&operation_id).await;
        assert!(operation.is_some());
        assert_eq!(operation.unwrap().agent_id, "agent_1");

        // Update progress
        tracker.update_progress(&operation_id, Some(0), 0.5, Some("Working...".to_string())).await.unwrap();
        tracker.complete_step(&operation_id, 0, true, Some("Step 1 done".to_string())).await.unwrap();
        
        // Complete operation
        tracker.complete_operation(&operation_id, true, Some("All done!".to_string()), None).await.unwrap();

        // Check operation was removed from active operations
        let operations = tracker.get_active_operations().await;
        assert!(operations.is_empty());
    }
}