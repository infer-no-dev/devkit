//! Sandboxing and process management for safe agent execution
//!
//! This module provides secure execution environments for agent operations,
//! including process isolation, resource limits, timeout enforcement,
//! and cleanup of orphaned processes.

pub mod container;
pub mod limits;
pub mod isolation;
pub mod cleanup;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Child;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

pub use container::Container;
pub use limits::{ResourceLimits, ResourceMonitor};
pub use isolation::{IsolationLevel, Namespace};
pub use cleanup::ProcessCleanup;

/// Sandbox manager for safe agent execution
#[derive(Debug)]
pub struct SandboxManager {
    config: SandboxConfig,
    active_sandboxes: Arc<RwLock<HashMap<Uuid, ActiveSandbox>>>,
    resource_monitor: Arc<ResourceMonitor>,
    cleanup_service: Arc<ProcessCleanup>,
}

/// Configuration for sandbox behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Default resource limits for sandboxed processes
    pub default_limits: ResourceLimits,
    /// Isolation level to apply
    pub isolation_level: IsolationLevel,
    /// Whether to enable container-based isolation
    pub use_containers: bool,
    /// Container runtime to use (docker, podman, etc.)
    pub container_runtime: ContainerRuntime,
    /// Default timeout for sandbox operations
    pub default_timeout: Duration,
    /// Maximum number of concurrent sandboxes
    pub max_concurrent_sandboxes: usize,
    /// Directory for sandbox temporary files
    pub temp_dir: PathBuf,
    /// Whether to preserve sandbox artifacts after completion
    pub preserve_artifacts: bool,
    /// Network access policy
    pub network_policy: NetworkPolicy,
    /// File system access policy  
    pub filesystem_policy: FilesystemPolicy,
}

/// Active sandbox instance
#[derive(Debug)]
pub struct ActiveSandbox {
    id: Uuid,
    config: SandboxConfig,
    container: Option<Container>,
    processes: Vec<ManagedProcess>,
    start_time: Instant,
    status: SandboxStatus,
    working_dir: PathBuf,
    environment: HashMap<String, String>,
}

/// Managed process within a sandbox
#[derive(Debug)]
pub struct ManagedProcess {
    id: Uuid,
    pid: Option<u32>,
    child: Option<Child>,
    command: String,
    start_time: Instant,
    timeout: Option<Duration>,
    limits: ResourceLimits,
    status: ProcessStatus,
}

/// Status of a sandbox
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SandboxStatus {
    Initializing,
    Running,
    Stopping,
    Stopped,
    Failed(String),
}

/// Status of a managed process
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessStatus {
    Starting,
    Running,
    Completed(i32), // Exit code
    Killed,
    TimedOut,
    Failed(String),
}

/// Container runtime options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContainerRuntime {
    Docker,
    Podman,
    Containerd,
    None, // Use OS-level isolation only
}

/// Network access policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkPolicy {
    /// No network access
    None,
    /// Only localhost access
    Localhost,
    /// Limited outbound access to specific hosts
    Limited(Vec<String>),
    /// Full network access
    Full,
}

/// Filesystem access policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilesystemPolicy {
    /// Read-only access to specified directories
    ReadOnly(Vec<PathBuf>),
    /// Read-write access to specified directories
    Limited(Vec<PathBuf>),
    /// Full filesystem access
    Full,
}

/// Result of sandbox execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxResult {
    pub sandbox_id: Uuid,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub execution_time: Duration,
    pub resources_used: ResourceUsage,
    pub violations: Vec<PolicyViolation>,
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_time_ms: u64,
    pub peak_memory_kb: u64,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
}

/// Policy violation detected during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub violation_type: ViolationType,
    pub description: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub severity: ViolationSeverity,
}

/// Types of policy violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    ResourceLimit,
    NetworkAccess,
    FilesystemAccess,
    ProcessSpawn,
    Syscall,
}

/// Severity levels for violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Errors in sandboxing operations
#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("Sandbox creation failed: {0}")]
    CreationFailed(String),
    
    #[error("Process execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    
    #[error("Timeout exceeded: {0:?}")]
    TimeoutExceeded(Duration),
    
    #[error("Policy violation: {0}")]
    PolicyViolation(String),
    
    #[error("Container error: {0}")]
    ContainerError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("System error: {0}")]
    SystemError(String),
}

impl SandboxManager {
    /// Create a new sandbox manager
    pub fn new(config: SandboxConfig) -> Result<Self, SandboxError> {
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let cleanup_service = Arc::new(ProcessCleanup::new());
        
        // Ensure temp directory exists
        std::fs::create_dir_all(&config.temp_dir)
            .map_err(|e| SandboxError::CreationFailed(format!("Failed to create temp dir: {}", e)))?;
        
        Ok(Self {
            config,
            active_sandboxes: Arc::new(RwLock::new(HashMap::new())),
            resource_monitor,
            cleanup_service,
        })
    }
    
    /// Create a new sandbox
    pub async fn create_sandbox(&self) -> Result<Uuid, SandboxError> {
        let sandbox_id = Uuid::new_v4();
        
        // Check if we're at capacity
        let active_count = self.active_sandboxes.read().await.len();
        if active_count >= self.config.max_concurrent_sandboxes {
            return Err(SandboxError::CreationFailed(
                "Maximum concurrent sandboxes reached".to_string()
            ));
        }
        
        // Create working directory
        let working_dir = self.config.temp_dir.join(format!("sandbox_{}", sandbox_id));
        std::fs::create_dir_all(&working_dir)?;
        
        // Initialize container if configured
        let container = if self.config.use_containers {
            Some(Container::new(&self.config, &working_dir).await?)
        } else {
            None
        };
        
        let sandbox = ActiveSandbox {
            id: sandbox_id,
            config: self.config.clone(),
            container,
            processes: Vec::new(),
            start_time: Instant::now(),
            status: SandboxStatus::Initializing,
            working_dir,
            environment: HashMap::new(),
        };
        
        // Register sandbox
        self.active_sandboxes.write().await.insert(sandbox_id, sandbox);
        
        // Start monitoring
        self.resource_monitor.start_monitoring(sandbox_id).await;
        
        Ok(sandbox_id)
    }
    
    /// Execute a command in a sandbox
    pub async fn execute(
        &self,
        sandbox_id: Uuid,
        command: &str,
        args: &[&str],
        timeout: Option<Duration>,
    ) -> Result<SandboxResult, SandboxError> {
        let start_time = Instant::now();
        
        // Get sandbox
        let mut sandboxes = self.active_sandboxes.write().await;
        let sandbox = sandboxes.get_mut(&sandbox_id)
            .ok_or_else(|| SandboxError::ExecutionFailed("Sandbox not found".to_string()))?;
        
        sandbox.status = SandboxStatus::Running;
        
        // Prepare execution environment
        let execution_timeout = timeout.unwrap_or(self.config.default_timeout);
        let limits = self.config.default_limits.clone();
        
        // Execute based on isolation method
        let result = if let Some(ref container) = sandbox.container {
            // Container execution
            self.execute_in_container(container, command, args, execution_timeout, &limits).await?
        } else {
            // OS-level execution
            self.execute_with_limits(sandbox, command, args, execution_timeout, &limits).await?
        };
        
        sandbox.status = SandboxStatus::Running;
        
        Ok(result)
    }
    
    /// Execute command in container
    async fn execute_in_container(
        &self,
        container: &Container,
        command: &str,
        args: &[&str],
        timeout: Duration,
        limits: &ResourceLimits,
    ) -> Result<SandboxResult, SandboxError> {
        container.execute(command, args, timeout, limits).await
    }
    
    /// Execute command with OS-level limits
    async fn execute_with_limits(
        &self,
        sandbox: &mut ActiveSandbox,
        command: &str,
        args: &[&str],
        timeout: Duration,
        limits: &ResourceLimits,
    ) -> Result<SandboxResult, SandboxError> {
        use tokio::process::Command;
        use tokio::time::timeout as tokio_timeout;
        
        let process_id = Uuid::new_v4();
        let start_time = Instant::now();
        
        // Create command with limits
        let mut cmd = Command::new(command);
        cmd.args(args);
        cmd.current_dir(&sandbox.working_dir);
        
        // Apply environment variables
        for (key, value) in &sandbox.environment {
            cmd.env(key, value);
        }
        
        // Apply resource limits (Unix-specific)
        #[cfg(unix)]
        {
            cmd.process_group(0); // New process group for cleanup
        }
        
        // Start process with timeout
        let child_result = tokio_timeout(timeout, async { cmd.spawn() }).await;
        
        let mut child = match child_result {
            Ok(Ok(child)) => child,
            Ok(Err(e)) => return Err(SandboxError::ExecutionFailed(format!("Failed to spawn: {}", e))),
            Err(_) => return Err(SandboxError::TimeoutExceeded(timeout)),
        };
        
        let pid = child.id();
        
        // Create managed process entry
        let managed_process = ManagedProcess {
            id: process_id,
            pid,
            child: Some(child),
            command: format!("{} {}", command, args.join(" ")),
            start_time,
            timeout: Some(timeout),
            limits: limits.clone(),
            status: ProcessStatus::Running,
        };
        
        sandbox.processes.push(managed_process);
        
        // Wait for completion with resource monitoring
        let child = sandbox.processes.last_mut().unwrap().child.take().unwrap();
        let output_result = tokio_timeout(timeout, child.wait_with_output()).await;
        
        let execution_time = start_time.elapsed();
        
        match output_result {
            Ok(Ok(output)) => {
                let success = output.status.success();
                let exit_code = output.status.code();
                
                // Get resource usage
                let resources_used = self.resource_monitor.get_usage(sandbox.id).await
                    .unwrap_or_default();
                
                Ok(SandboxResult {
                    sandbox_id: sandbox.id,
                    success,
                    exit_code,
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    execution_time,
                    resources_used,
                    violations: Vec::new(), // Would be populated by monitoring
                })
            }
            Ok(Err(e)) => Err(SandboxError::ExecutionFailed(format!("Process failed: {}", e))),
            Err(_) => {
                // Timeout - kill the process
                if let Some(ref mut child) = sandbox.processes.last_mut().unwrap().child {
                    let _ = child.kill();
                }
                Err(SandboxError::TimeoutExceeded(timeout))
            }
        }
    }
    
    /// Set environment variable in sandbox
    pub async fn set_env_var(&self, sandbox_id: Uuid, key: String, value: String) -> Result<(), SandboxError> {
        let mut sandboxes = self.active_sandboxes.write().await;
        let sandbox = sandboxes.get_mut(&sandbox_id)
            .ok_or_else(|| SandboxError::ExecutionFailed("Sandbox not found".to_string()))?;
        
        sandbox.environment.insert(key, value);
        Ok(())
    }
    
    /// Copy file into sandbox
    pub async fn copy_to_sandbox(
        &self,
        sandbox_id: Uuid,
        src: &PathBuf,
        dest: &str,
    ) -> Result<(), SandboxError> {
        let sandboxes = self.active_sandboxes.read().await;
        let sandbox = sandboxes.get(&sandbox_id)
            .ok_or_else(|| SandboxError::ExecutionFailed("Sandbox not found".to_string()))?;
        
        let dest_path = sandbox.working_dir.join(dest);
        
        // Ensure parent directory exists
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::copy(src, dest_path)?;
        Ok(())
    }
    
    /// Copy file from sandbox
    pub async fn copy_from_sandbox(
        &self,
        sandbox_id: Uuid,
        src: &str,
        dest: &PathBuf,
    ) -> Result<(), SandboxError> {
        let sandboxes = self.active_sandboxes.read().await;
        let sandbox = sandboxes.get(&sandbox_id)
            .ok_or_else(|| SandboxError::ExecutionFailed("Sandbox not found".to_string()))?;
        
        let src_path = sandbox.working_dir.join(src);
        std::fs::copy(src_path, dest)?;
        Ok(())
    }
    
    /// Destroy a sandbox and cleanup resources
    pub async fn destroy_sandbox(&self, sandbox_id: Uuid) -> Result<(), SandboxError> {
        let mut sandboxes = self.active_sandboxes.write().await;
        
        if let Some(mut sandbox) = sandboxes.remove(&sandbox_id) {
            sandbox.status = SandboxStatus::Stopping;
            
            // Kill any running processes
            for process in &mut sandbox.processes {
                if let Some(ref mut child) = process.child {
                    let _ = child.kill();
                }
            }
            
            // Stop container if used
            if let Some(ref container) = sandbox.container {
                container.stop().await?;
            }
            
            // Stop monitoring
            self.resource_monitor.stop_monitoring(sandbox_id).await;
            
            // Cleanup filesystem
            if !self.config.preserve_artifacts {
                let _ = std::fs::remove_dir_all(&sandbox.working_dir);
            }
            
            sandbox.status = SandboxStatus::Stopped;
        }
        
        Ok(())
    }
    
    /// Get status of a sandbox
    pub async fn get_sandbox_status(&self, sandbox_id: Uuid) -> Option<SandboxStatus> {
        let sandboxes = self.active_sandboxes.read().await;
        sandboxes.get(&sandbox_id).map(|s| s.status.clone())
    }
    
    /// List all active sandboxes
    pub async fn list_sandboxes(&self) -> Vec<Uuid> {
        let sandboxes = self.active_sandboxes.read().await;
        sandboxes.keys().cloned().collect()
    }
    
    /// Get resource usage for a sandbox
    pub async fn get_resource_usage(&self, sandbox_id: Uuid) -> Option<ResourceUsage> {
        self.resource_monitor.get_usage(sandbox_id).await
    }
    
    /// Shutdown sandbox manager and cleanup all sandboxes
    pub async fn shutdown(&self) -> Result<(), SandboxError> {
        let sandbox_ids: Vec<Uuid> = self.list_sandboxes().await;
        
        for sandbox_id in sandbox_ids {
            self.destroy_sandbox(sandbox_id).await?;
        }
        
        self.cleanup_service.cleanup_all().await?;
        
        Ok(())
    }
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            default_limits: ResourceLimits::default(),
            isolation_level: IsolationLevel::Process,
            use_containers: false,
            container_runtime: ContainerRuntime::None,
            default_timeout: Duration::from_secs(30),
            max_concurrent_sandboxes: 10,
            temp_dir: PathBuf::from("/tmp/devkit_sandboxes"),
            preserve_artifacts: false,
            network_policy: NetworkPolicy::Limited(vec![
                "github.com".to_string(),
                "api.openai.com".to_string(),
            ]),
            filesystem_policy: FilesystemPolicy::Limited(vec![
                PathBuf::from("/tmp"),
                PathBuf::from("/var/tmp"),
            ]),
        }
    }
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            cpu_time_ms: 0,
            peak_memory_kb: 0,
            disk_read_bytes: 0,
            disk_write_bytes: 0,
            network_rx_bytes: 0,
            network_tx_bytes: 0,
        }
    }
}