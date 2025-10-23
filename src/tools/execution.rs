//! Tool Execution Engine
//!
//! Manages concurrent tool execution, rate limiting, and resource management.

use super::{ToolError, providers::ExecutionContext, providers::ExecutionResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore, oneshot};
use uuid::Uuid;

/// Tool executor for managing concurrent executions
#[derive(Debug)]
pub struct ToolExecutor {
    /// Maximum concurrent executions
    max_concurrent: usize,
    /// Semaphore for limiting concurrency
    semaphore: Arc<Semaphore>,
    /// Active executions
    active_executions: Arc<RwLock<HashMap<Uuid, ExecutionInfo>>>,
    /// Execution queue for overflow
    execution_queue: Arc<RwLock<Vec<QueuedExecution>>>,
    /// Rate limiter
    rate_limiter: Arc<RwLock<RateLimiter>>,
    /// Execution statistics
    stats: Arc<RwLock<ExecutionStats>>,
}

/// Information about an active execution
#[derive(Debug)]
pub struct ExecutionInfo {
    /// Execution ID
    pub id: Uuid,
    /// Tool name
    pub tool_name: String,
    /// Operation name
    pub operation: String,
    /// Start time
    pub started_at: Instant,
    /// Timeout duration
    pub timeout: Option<Duration>,
    /// Cancellation sender
    cancel_tx: Option<oneshot::Sender<()>>,
}

/// Queued execution waiting to run
#[derive(Debug)]
pub struct QueuedExecution {
    /// Execution context
    pub context: ExecutionContext,
    /// Result sender
    pub result_tx: oneshot::Sender<Result<ExecutionResult, ToolError>>,
    /// Queued timestamp
    pub queued_at: Instant,
    /// Priority
    pub priority: ExecutionPriority,
}

/// Execution priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExecutionPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Rate limiter for tool executions
#[derive(Debug)]
pub struct RateLimiter {
    /// Rate limits by tool name
    tool_limits: HashMap<String, ToolRateLimit>,
    /// Global rate limit
    global_limit: Option<GlobalRateLimit>,
}

/// Rate limit for a specific tool
#[derive(Debug, Clone)]
pub struct ToolRateLimit {
    /// Requests per minute
    per_minute: u32,
    /// Requests per hour
    per_hour: u32,
    /// Request timestamps
    requests: Vec<Instant>,
    /// Last reset time
    last_reset: Instant,
}

/// Global rate limit
#[derive(Debug, Clone)]
pub struct GlobalRateLimit {
    /// Total requests per minute
    per_minute: u32,
    /// Total requests per hour
    per_hour: u32,
    /// Request timestamps
    requests: Vec<Instant>,
    /// Last reset time
    last_reset: Instant,
}

/// Execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStats {
    /// Total executions
    pub total_executions: u64,
    /// Successful executions
    pub successful_executions: u64,
    /// Failed executions
    pub failed_executions: u64,
    /// Timed out executions
    pub timeout_executions: u64,
    /// Cancelled executions
    pub cancelled_executions: u64,
    /// Average execution time
    pub avg_execution_time: Duration,
    /// Current active executions
    pub active_executions: usize,
    /// Queued executions
    pub queued_executions: usize,
    /// Executions by tool
    pub tool_stats: HashMap<String, ToolExecutionStats>,
    /// Rate limit violations
    pub rate_limit_violations: u64,
    /// Last updated
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Per-tool execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionStats {
    /// Total executions for this tool
    pub total: u64,
    /// Successful executions
    pub successful: u64,
    /// Failed executions
    pub failed: u64,
    /// Average execution time
    pub avg_time: Duration,
    /// Last execution time
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
}

/// Execution request
#[derive(Debug)]
pub struct ExecutionRequest {
    /// Execution context
    pub context: ExecutionContext,
    /// Execution priority
    pub priority: ExecutionPriority,
    /// Whether to bypass rate limiting
    pub bypass_rate_limit: bool,
}

impl ToolExecutor {
    /// Create a new tool executor
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            max_concurrent,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            active_executions: Arc::new(RwLock::new(HashMap::new())),
            execution_queue: Arc::new(RwLock::new(Vec::new())),
            rate_limiter: Arc::new(RwLock::new(RateLimiter::new())),
            stats: Arc::new(RwLock::new(ExecutionStats::new())),
        }
    }
    
    /// Execute a tool with the given context
    pub async fn execute(&self, context: ExecutionContext) -> Result<ExecutionResult, ToolError> {
        let request = ExecutionRequest {
            context,
            priority: ExecutionPriority::Normal,
            bypass_rate_limit: false,
        };
        
        self.execute_with_priority(request).await
    }
    
    /// Execute a tool with specific priority
    pub async fn execute_with_priority(&self, request: ExecutionRequest) -> Result<ExecutionResult, ToolError> {
        let execution_id = Uuid::new_v4();
        let tool_name = request.context.tool_name.clone();
        
        // Check rate limits
        if !request.bypass_rate_limit {
            if !self.check_rate_limit(&tool_name).await {
                self.update_stats_rate_limit_violation().await;
                return Err(ToolError::RateLimitExceeded(tool_name));
            }
        }
        
        // Try to acquire semaphore permit
        match self.semaphore.try_acquire() {
            Ok(permit) => {
                // Execute immediately
                self.execute_immediately(execution_id, request.context, permit).await
            },
            Err(_) => {
                // Queue for later execution
                self.queue_execution(request).await
            }
        }
    }
    
    /// Execute immediately with acquired permit
    async fn execute_immediately(
        &self,
        execution_id: Uuid,
        context: ExecutionContext,
        _permit: tokio::sync::SemaphorePermit<'_>,
    ) -> Result<ExecutionResult, ToolError> {
        let tool_name = context.tool_name.clone();
        let operation = context.operation.clone();
        let timeout = context.timeout.unwrap_or(Duration::from_secs(30));
        
        // Create cancellation channel
        let (cancel_tx, cancel_rx) = oneshot::channel();
        
        // Register execution
        let execution_info = ExecutionInfo {
            id: execution_id,
            tool_name: tool_name.clone(),
            operation: operation.clone(),
            started_at: Instant::now(),
            timeout: Some(timeout),
            cancel_tx: Some(cancel_tx),
        };
        
        {
            let mut active = self.active_executions.write().await;
            active.insert(execution_id, execution_info);
        }
        
        // Update rate limiter
        self.record_execution(&tool_name).await;
        
        let start_time = Instant::now();
        
        // Execute with timeout and cancellation
        let result = tokio::select! {
            // Normal execution
            result = self.execute_tool(context) => {
                result
            },
            // Timeout
            _ = tokio::time::sleep(timeout) => {
                self.update_stats_timeout().await;
                Err(ToolError::ExecutionTimeout(format!("Tool '{}' operation '{}' timed out after {:?}", tool_name, operation, timeout)))
            },
            // Cancellation
            _ = cancel_rx => {
                self.update_stats_cancelled().await;
                Err(ToolError::ExecutionTimeout(format!("Tool '{}' operation '{}' was cancelled", tool_name, operation)))
            }
        };
        
        let duration = start_time.elapsed();
        
        // Clean up execution
        {
            let mut active = self.active_executions.write().await;
            active.remove(&execution_id);
        }
        
        // Update statistics
        match &result {
            Ok(_) => self.update_stats_success(&tool_name, duration).await,
            Err(_) => self.update_stats_failure(&tool_name, duration).await,
        }
        
        // Process queue if there are waiting executions
        self.process_queue().await;
        
        result
    }
    
    /// Execute the actual tool operation
    async fn execute_tool(&self, context: ExecutionContext) -> Result<ExecutionResult, ToolError> {
        // This would integrate with the actual tool providers
        // For now, return a mock result
        Ok(ExecutionResult {
            success: true,
            output: serde_json::json!({"mock": "This is a mock execution result"}),
            duration: Duration::from_millis(100),
            artifacts: vec![],
            warnings: vec![],
            debug_info: HashMap::new(),
            cost: None,
            rate_limit_remaining: None,
        })
    }
    
    /// Queue execution for later
    async fn queue_execution(&self, request: ExecutionRequest) -> Result<ExecutionResult, ToolError> {
        let (tx, rx) = oneshot::channel();
        
        let queued_execution = QueuedExecution {
            context: request.context,
            result_tx: tx,
            queued_at: Instant::now(),
            priority: request.priority,
        };
        
        {
            let mut queue = self.execution_queue.write().await;
            queue.push(queued_execution);
            // Sort by priority (highest first)
            queue.sort_by(|a, b| b.priority.cmp(&a.priority));
        }
        
        // Wait for result
        rx.await
            .map_err(|_| ToolError::ExecutionTimeout("Queued execution was cancelled".to_string()))?
    }
    
    /// Process queued executions
    async fn process_queue(&self) {
        let next_execution = {
            let mut queue = self.execution_queue.write().await;
            queue.pop()
        };
        
        if let Some(queued) = next_execution {
            let execution_id = Uuid::new_v4();
            
            // Try to acquire permit again
            if let Ok(permit) = self.semaphore.try_acquire() {
                let result = Box::pin(self.execute_immediately(execution_id, queued.context, permit)).await;
                let _ = queued.result_tx.send(result);
            } else {
                // Put back in queue if still no capacity
                let mut queue = self.execution_queue.write().await;
                queue.push(queued);
            }
        }
    }
    
    /// Check rate limits for a tool
    async fn check_rate_limit(&self, tool_name: &str) -> bool {
        let mut rate_limiter = self.rate_limiter.write().await;
        rate_limiter.check_tool_limit(tool_name) && rate_limiter.check_global_limit()
    }
    
    /// Record execution for rate limiting
    async fn record_execution(&self, tool_name: &str) {
        let mut rate_limiter = self.rate_limiter.write().await;
        rate_limiter.record_tool_execution(tool_name);
        rate_limiter.record_global_execution();
    }
    
    /// Cancel a specific execution
    pub async fn cancel_execution(&self, execution_id: Uuid) -> Result<(), ToolError> {
        let mut active = self.active_executions.write().await;
        
        if let Some(mut execution_info) = active.remove(&execution_id) {
            if let Some(cancel_tx) = execution_info.cancel_tx.take() {
                let _ = cancel_tx.send(());
            }
            Ok(())
        } else {
            Err(ToolError::ToolNotFound(format!("Execution {} not found", execution_id)))
        }
    }
    
    /// Cancel all executions for a specific tool
    pub async fn cancel_tool_executions(&self, tool_name: &str) -> usize {
        let mut active = self.active_executions.write().await;
        let mut cancelled_count = 0;
        
        let execution_ids: Vec<_> = active
            .iter()
            .filter(|(_, info)| info.tool_name == tool_name)
            .map(|(id, _)| *id)
            .collect();
        
        for execution_id in execution_ids {
            if let Some(mut execution_info) = active.remove(&execution_id) {
                if let Some(cancel_tx) = execution_info.cancel_tx.take() {
                    let _ = cancel_tx.send(());
                    cancelled_count += 1;
                }
            }
        }
        
        cancelled_count
    }
    
    /// Get current execution statistics
    pub async fn get_stats(&self) -> ExecutionStats {
        let stats = self.stats.read().await;
        let mut stats = stats.clone();
        
        // Update current counts
        let active = self.active_executions.read().await;
        let queue = self.execution_queue.read().await;
        
        stats.active_executions = active.len();
        stats.queued_executions = queue.len();
        stats.last_updated = chrono::Utc::now();
        
        stats
    }
    
    /// Get active executions summary (without cancel channels)
    pub async fn get_active_executions_summary(&self) -> Vec<(Uuid, String, String, Instant)> {
        let active = self.active_executions.read().await;
        active.iter().map(|(id, info)| {
            (*id, info.tool_name.clone(), info.operation.clone(), info.started_at)
        }).collect()
    }
    
    /// Set rate limit for a specific tool
    pub async fn set_tool_rate_limit(&self, tool_name: String, per_minute: u32, per_hour: u32) {
        let mut rate_limiter = self.rate_limiter.write().await;
        rate_limiter.set_tool_limit(tool_name, per_minute, per_hour);
    }
    
    /// Set global rate limit
    pub async fn set_global_rate_limit(&self, per_minute: u32, per_hour: u32) {
        let mut rate_limiter = self.rate_limiter.write().await;
        rate_limiter.set_global_limit(per_minute, per_hour);
    }
    
    /// Update statistics for successful execution
    async fn update_stats_success(&self, tool_name: &str, duration: Duration) {
        let mut stats = self.stats.write().await;
        stats.total_executions += 1;
        stats.successful_executions += 1;
        stats.update_avg_time(duration);
        stats.update_tool_stats(tool_name, true, duration);
    }
    
    /// Update statistics for failed execution
    async fn update_stats_failure(&self, tool_name: &str, duration: Duration) {
        let mut stats = self.stats.write().await;
        stats.total_executions += 1;
        stats.failed_executions += 1;
        stats.update_avg_time(duration);
        stats.update_tool_stats(tool_name, false, duration);
    }
    
    /// Update statistics for timeout
    async fn update_stats_timeout(&self) {
        let mut stats = self.stats.write().await;
        stats.total_executions += 1;
        stats.timeout_executions += 1;
    }
    
    /// Update statistics for cancellation
    async fn update_stats_cancelled(&self) {
        let mut stats = self.stats.write().await;
        stats.total_executions += 1;
        stats.cancelled_executions += 1;
    }
    
    /// Update statistics for rate limit violation
    async fn update_stats_rate_limit_violation(&self) {
        let mut stats = self.stats.write().await;
        stats.rate_limit_violations += 1;
    }
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            tool_limits: HashMap::new(),
            global_limit: None,
        }
    }
    
    fn set_tool_limit(&mut self, tool_name: String, per_minute: u32, per_hour: u32) {
        let limit = ToolRateLimit {
            per_minute,
            per_hour,
            requests: Vec::new(),
            last_reset: Instant::now(),
        };
        self.tool_limits.insert(tool_name, limit);
    }
    
    fn set_global_limit(&mut self, per_minute: u32, per_hour: u32) {
        self.global_limit = Some(GlobalRateLimit {
            per_minute,
            per_hour,
            requests: Vec::new(),
            last_reset: Instant::now(),
        });
    }
    
    fn check_tool_limit(&mut self, tool_name: &str) -> bool {
        if let Some(limit) = self.tool_limits.get_mut(tool_name) {
            limit.check_limit()
        } else {
            true // No limit set
        }
    }
    
    fn check_global_limit(&mut self) -> bool {
        if let Some(limit) = &mut self.global_limit {
            limit.check_limit()
        } else {
            true // No global limit
        }
    }
    
    fn record_tool_execution(&mut self, tool_name: &str) {
        if let Some(limit) = self.tool_limits.get_mut(tool_name) {
            limit.record_request();
        }
    }
    
    fn record_global_execution(&mut self) {
        if let Some(limit) = &mut self.global_limit {
            limit.record_request();
        }
    }
}

impl ToolRateLimit {
    fn check_limit(&mut self) -> bool {
        self.cleanup_old_requests();
        
        let now = Instant::now();
        let minute_ago = now - Duration::from_secs(60);
        let hour_ago = now - Duration::from_secs(3600);
        
        let minute_count = self.requests.iter().filter(|&&t| t > minute_ago).count() as u32;
        let hour_count = self.requests.iter().filter(|&&t| t > hour_ago).count() as u32;
        
        minute_count < self.per_minute && hour_count < self.per_hour
    }
    
    fn record_request(&mut self) {
        let now = Instant::now();
        self.requests.push(now);
        self.cleanup_old_requests();
    }
    
    fn cleanup_old_requests(&mut self) {
        let hour_ago = Instant::now() - Duration::from_secs(3600);
        self.requests.retain(|&t| t > hour_ago);
    }
}

impl GlobalRateLimit {
    fn check_limit(&mut self) -> bool {
        self.cleanup_old_requests();
        
        let now = Instant::now();
        let minute_ago = now - Duration::from_secs(60);
        let hour_ago = now - Duration::from_secs(3600);
        
        let minute_count = self.requests.iter().filter(|&&t| t > minute_ago).count() as u32;
        let hour_count = self.requests.iter().filter(|&&t| t > hour_ago).count() as u32;
        
        minute_count < self.per_minute && hour_count < self.per_hour
    }
    
    fn record_request(&mut self) {
        let now = Instant::now();
        self.requests.push(now);
        self.cleanup_old_requests();
    }
    
    fn cleanup_old_requests(&mut self) {
        let hour_ago = Instant::now() - Duration::from_secs(3600);
        self.requests.retain(|&t| t > hour_ago);
    }
}

impl ExecutionStats {
    fn new() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            timeout_executions: 0,
            cancelled_executions: 0,
            avg_execution_time: Duration::from_millis(0),
            active_executions: 0,
            queued_executions: 0,
            tool_stats: HashMap::new(),
            rate_limit_violations: 0,
            last_updated: chrono::Utc::now(),
        }
    }
    
    fn update_avg_time(&mut self, duration: Duration) {
        if self.total_executions > 0 {
            let total_time = self.avg_execution_time * (self.total_executions - 1) as u32 + duration;
            self.avg_execution_time = total_time / self.total_executions as u32;
        } else {
            self.avg_execution_time = duration;
        }
    }
    
    fn update_tool_stats(&mut self, tool_name: &str, success: bool, duration: Duration) {
        let stats = self.tool_stats.entry(tool_name.to_string()).or_insert_with(|| {
            ToolExecutionStats {
                total: 0,
                successful: 0,
                failed: 0,
                avg_time: Duration::from_millis(0),
                last_execution: None,
            }
        });
        
        stats.total += 1;
        if success {
            stats.successful += 1;
        } else {
            stats.failed += 1;
        }
        
        // Update average time
        if stats.total > 0 {
            let total_time = stats.avg_time * (stats.total - 1) as u32 + duration;
            stats.avg_time = total_time / stats.total as u32;
        } else {
            stats.avg_time = duration;
        }
        
        stats.last_execution = Some(chrono::Utc::now());
    }
}

impl Default for ExecutionPriority {
    fn default() -> Self {
        ExecutionPriority::Normal
    }
}