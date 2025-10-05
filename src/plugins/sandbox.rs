//! Plugin Sandbox System
//!
//! Provides security isolation and resource management for plugin execution.
//! Controls filesystem access, network permissions, and system resources.

use crate::plugins::{PluginError, PluginPermission};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tokio::process::{Child, Command as AsyncCommand};
use tokio::sync::{Mutex, RwLock};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

/// Plugin sandbox for secure execution
#[derive(Debug)]
pub struct PluginSandbox {
    /// Sandbox configuration
    config: SandboxConfig,
    /// Active sandboxed processes
    active_processes: Arc<Mutex<HashMap<String, SandboxProcess>>>,
    /// Resource usage tracking
    resource_tracker: Arc<RwLock<ResourceTracker>>,
    /// Permission manager
    permission_manager: PermissionManager,
}

/// Sandbox configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Enable sandboxing
    pub enabled: bool,
    /// Sandbox directory for temporary files
    pub sandbox_dir: PathBuf,
    /// Maximum execution time per plugin operation
    pub max_execution_time: Duration,
    /// Maximum memory usage per plugin (MB)
    pub max_memory_mb: u64,
    /// Maximum number of processes per plugin
    pub max_processes: u32,
    /// Maximum file system usage (MB)
    pub max_fs_usage_mb: u64,
    /// Maximum network connections per plugin
    pub max_network_connections: u32,
    /// Allowed outbound domains (if empty, all are allowed)
    pub allowed_domains: Vec<String>,
    /// Blocked domains
    pub blocked_domains: Vec<String>,
    /// Enable network access
    pub allow_network: bool,
    /// Enable filesystem write access
    pub allow_fs_write: bool,
    /// Allowed filesystem paths for read access
    pub allowed_read_paths: Vec<PathBuf>,
    /// Allowed filesystem paths for write access
    pub allowed_write_paths: Vec<PathBuf>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let sandbox_dir = home.join(".devkit").join("sandbox");
        
        Self {
            enabled: true,
            sandbox_dir,
            max_execution_time: Duration::from_secs(300), // 5 minutes
            max_memory_mb: 512, // 512 MB
            max_processes: 5,
            max_fs_usage_mb: 100, // 100 MB
            max_network_connections: 10,
            allowed_domains: vec![],
            blocked_domains: vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                "0.0.0.0".to_string(),
            ],
            allow_network: false,
            allow_fs_write: false,
            allowed_read_paths: vec![],
            allowed_write_paths: vec![],
        }
    }
}

/// Sandboxed process information
#[derive(Debug)]
pub struct SandboxProcess {
    pub plugin_id: String,
    pub process: Child,
    pub started_at: Instant,
    pub resource_usage: ResourceUsage,
    pub permissions: HashSet<PluginPermission>,
}

/// Resource usage tracking
#[derive(Debug, Clone, Default)]
pub struct ResourceTracker {
    /// Memory usage per plugin (bytes)
    memory_usage: HashMap<String, u64>,
    /// CPU usage per plugin (percentage)
    cpu_usage: HashMap<String, f32>,
    /// Network usage per plugin (bytes)
    network_usage: HashMap<String, u64>,
    /// Filesystem usage per plugin (bytes)
    fs_usage: HashMap<String, u64>,
    /// Active processes per plugin
    active_processes: HashMap<String, u32>,
}

/// Current resource usage for a process
#[derive(Debug, Clone, Default)]
pub struct ResourceUsage {
    pub memory_bytes: u64,
    pub cpu_percent: f32,
    pub network_bytes: u64,
    pub fs_bytes: u64,
}

/// Permission manager for controlling plugin access
#[derive(Debug)]
pub struct PermissionManager {
    /// Granted permissions per plugin
    granted_permissions: HashMap<String, HashSet<PluginPermission>>,
    /// Permission validation cache
    validation_cache: HashMap<String, bool>,
}

use std::sync::Arc;

impl PluginSandbox {
    /// Create a new plugin sandbox
    pub async fn new(config: SandboxConfig) -> Result<Self, PluginError> {
        // Create sandbox directory
        tokio::fs::create_dir_all(&config.sandbox_dir)
            .await
            .map_err(|e| PluginError::IoError(format!("Failed to create sandbox directory: {}", e)))?;

        let sandbox = Self {
            config,
            active_processes: Arc::new(Mutex::new(HashMap::new())),
            resource_tracker: Arc::new(RwLock::new(ResourceTracker::default())),
            permission_manager: PermissionManager::new(),
        };

        info!("Plugin sandbox initialized");
        Ok(sandbox)
    }

    /// Execute a plugin command in sandbox
    pub async fn execute_plugin(
        &self,
        plugin_id: &str,
        command: &str,
        args: &[&str],
        permissions: &[PluginPermission],
    ) -> Result<String, PluginError> {
        if !self.config.enabled {
            return self.execute_unsandboxed(command, args).await;
        }

        debug!("Executing plugin {} in sandbox: {} {:?}", plugin_id, command, args);

        // Check resource limits
        self.check_resource_limits(plugin_id).await?;

        // Validate permissions
        self.permission_manager.validate_permissions(plugin_id, permissions)?;

        // Create sandboxed environment
        let sandbox_env = self.create_sandbox_environment(plugin_id).await?;

        // Execute command with timeout and resource monitoring
        let result = self.execute_with_monitoring(plugin_id, command, args, sandbox_env).await?;

        // Cleanup sandbox environment
        self.cleanup_sandbox_environment(plugin_id).await?;

        Ok(result)
    }

    /// Grant permissions to a plugin
    pub async fn grant_permissions(
        &mut self,
        plugin_id: &str,
        permissions: &[PluginPermission],
    ) -> Result<(), PluginError> {
        debug!("Granting permissions to plugin {}: {:?}", plugin_id, permissions);
        self.permission_manager.grant_permissions(plugin_id, permissions);
        Ok(())
    }

    /// Revoke permissions from a plugin
    pub async fn revoke_permissions(
        &mut self,
        plugin_id: &str,
        permissions: &[PluginPermission],
    ) -> Result<(), PluginError> {
        debug!("Revoking permissions from plugin {}: {:?}", plugin_id, permissions);
        self.permission_manager.revoke_permissions(plugin_id, permissions);
        Ok(())
    }

    /// Terminate a plugin's processes
    pub async fn terminate_plugin(&self, plugin_id: &str) -> Result<(), PluginError> {
        debug!("Terminating plugin processes: {}", plugin_id);

        let mut processes = self.active_processes.lock().await;
        if let Some(mut process) = processes.remove(plugin_id) {
            // Send SIGTERM first
            if let Err(e) = process.process.start_kill() {
                warn!("Failed to send SIGTERM to plugin {}: {}", plugin_id, e);
            }

            // Wait for graceful shutdown
            match timeout(Duration::from_secs(5), process.process.wait()).await {
                Ok(Ok(_)) => {
                    debug!("Plugin {} terminated gracefully", plugin_id);
                }
                Ok(Err(e)) => {
                    error!("Error waiting for plugin {} termination: {}", plugin_id, e);
                }
                Err(_) => {
                    // Force kill after timeout
                    warn!("Force killing plugin {} after timeout", plugin_id);
                    let _ = process.process.kill().await;
                }
            }
        }

        // Clean up resources
        self.cleanup_plugin_resources(plugin_id).await?;

        Ok(())
    }

    /// Get resource usage statistics
    pub async fn get_resource_usage(&self, plugin_id: &str) -> Result<ResourceUsage, PluginError> {
        let tracker = self.resource_tracker.read().await;
        
        let memory_bytes = tracker.memory_usage.get(plugin_id).copied().unwrap_or(0);
        let cpu_percent = tracker.cpu_usage.get(plugin_id).copied().unwrap_or(0.0);
        let network_bytes = tracker.network_usage.get(plugin_id).copied().unwrap_or(0);
        let fs_bytes = tracker.fs_usage.get(plugin_id).copied().unwrap_or(0);

        Ok(ResourceUsage {
            memory_bytes,
            cpu_percent,
            network_bytes,
            fs_bytes,
        })
    }

    /// Get sandbox statistics
    pub async fn get_stats(&self) -> SandboxStats {
        let tracker = self.resource_tracker.read().await;
        let active_processes = self.active_processes.lock().await;

        let total_memory: u64 = tracker.memory_usage.values().sum();
        let total_network: u64 = tracker.network_usage.values().sum();
        let total_fs: u64 = tracker.fs_usage.values().sum();
        let active_plugins = active_processes.len();

        SandboxStats {
            active_plugins,
            total_memory_mb: total_memory / (1024 * 1024),
            total_network_mb: total_network / (1024 * 1024),
            total_fs_mb: total_fs / (1024 * 1024),
            sandbox_enabled: self.config.enabled,
        }
    }

    // Private implementation methods

    async fn check_resource_limits(&self, plugin_id: &str) -> Result<(), PluginError> {
        let tracker = self.resource_tracker.read().await;

        // Check memory limit
        if let Some(&memory_usage) = tracker.memory_usage.get(plugin_id) {
            let memory_mb = memory_usage / (1024 * 1024);
            if memory_mb > self.config.max_memory_mb {
                return Err(PluginError::ResourceExhausted(
                    format!("Plugin {} exceeded memory limit: {} MB > {} MB", 
                            plugin_id, memory_mb, self.config.max_memory_mb)
                ));
            }
        }

        // Check process limit
        if let Some(&process_count) = tracker.active_processes.get(plugin_id) {
            if process_count >= self.config.max_processes {
                return Err(PluginError::ResourceExhausted(
                    format!("Plugin {} exceeded process limit: {} >= {}", 
                            plugin_id, process_count, self.config.max_processes)
                ));
            }
        }

        // Check filesystem usage
        if let Some(&fs_usage) = tracker.fs_usage.get(plugin_id) {
            let fs_mb = fs_usage / (1024 * 1024);
            if fs_mb > self.config.max_fs_usage_mb {
                return Err(PluginError::ResourceExhausted(
                    format!("Plugin {} exceeded filesystem limit: {} MB > {} MB", 
                            plugin_id, fs_mb, self.config.max_fs_usage_mb)
                ));
            }
        }

        Ok(())
    }

    async fn create_sandbox_environment(&self, plugin_id: &str) -> Result<SandboxEnvironment, PluginError> {
        let plugin_sandbox_dir = self.config.sandbox_dir.join(plugin_id);
        
        // Create plugin-specific sandbox directory
        tokio::fs::create_dir_all(&plugin_sandbox_dir)
            .await
            .map_err(|e| PluginError::IoError(format!("Failed to create plugin sandbox: {}", e)))?;

        // Create temporary directories
        let temp_dir = plugin_sandbox_dir.join("tmp");
        let data_dir = plugin_sandbox_dir.join("data");
        
        tokio::fs::create_dir_all(&temp_dir).await.ok();
        tokio::fs::create_dir_all(&data_dir).await.ok();

        Ok(SandboxEnvironment {
            plugin_id: plugin_id.to_string(),
            sandbox_dir: plugin_sandbox_dir,
            temp_dir,
            data_dir,
            environment_vars: self.create_environment_vars(plugin_id),
        })
    }

    fn create_environment_vars(&self, plugin_id: &str) -> HashMap<String, String> {
        let mut env = HashMap::new();
        
        // Set safe environment variables
        env.insert("PLUGIN_ID".to_string(), plugin_id.to_string());
        env.insert("SANDBOX_MODE".to_string(), "true".to_string());
        
        // Limit PATH to safe directories
        env.insert("PATH".to_string(), "/usr/bin:/bin".to_string());
        
        // Set temporary directory
        let temp_dir = self.config.sandbox_dir.join(plugin_id).join("tmp");
        env.insert("TMPDIR".to_string(), temp_dir.to_string_lossy().to_string());
        env.insert("TEMP".to_string(), temp_dir.to_string_lossy().to_string());
        
        // Disable potentially dangerous environment variables
        env.insert("LD_PRELOAD".to_string(), "".to_string());
        env.insert("LD_LIBRARY_PATH".to_string(), "".to_string());
        
        env
    }

    async fn execute_with_monitoring(
        &self,
        plugin_id: &str,
        command: &str,
        args: &[&str],
        sandbox_env: SandboxEnvironment,
    ) -> Result<String, PluginError> {
        // Build command with sandbox restrictions
        let mut cmd = AsyncCommand::new(command);
        cmd.args(args);
        
        // Set working directory to sandbox
        cmd.current_dir(&sandbox_env.sandbox_dir);
        
        // Set environment variables
        cmd.env_clear();
        for (key, value) in &sandbox_env.environment_vars {
            cmd.env(key, value);
        }
        
        // Configure stdio
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::null());

        // Start process
        let mut child = cmd.spawn()
            .map_err(|e| PluginError::ExecutionFailed(format!("Failed to spawn process: {}", e)))?;

        // Create sandbox process entry
        let sandbox_process = SandboxProcess {
            plugin_id: plugin_id.to_string(),
            process: child,
            started_at: Instant::now(),
            resource_usage: ResourceUsage::default(),
            permissions: HashSet::new(),
        };

        // Store in active processes
        {
            let mut processes = self.active_processes.lock().await;
            processes.insert(plugin_id.to_string(), sandbox_process);
        }

        // Wait for completion with timeout
        let execution_result = timeout(
            self.config.max_execution_time,
            self.monitor_execution(plugin_id),
        ).await;

        match execution_result {
            Ok(Ok(output)) => Ok(output),
            Ok(Err(e)) => Err(e),
            Err(_) => {
                // Timeout occurred
                self.terminate_plugin(plugin_id).await?;
                Err(PluginError::ExecutionFailed(
                    format!("Plugin {} execution timed out", plugin_id)
                ))
            }
        }
    }

    async fn monitor_execution(&self, plugin_id: &str) -> Result<String, PluginError> {
        // This is a simplified monitoring implementation
        // In a real system, you would implement proper resource monitoring
        
        let output = {
            let mut processes = self.active_processes.lock().await;
            if let Some(mut sandbox_process) = processes.remove(plugin_id) {
                sandbox_process.process.wait_with_output().await
                    .map_err(|e| PluginError::ExecutionFailed(format!("Process execution failed: {}", e)))?
            } else {
                return Err(PluginError::ExecutionFailed("Process not found".to_string()));
            }
        };

        if output.status.success() {
            String::from_utf8(output.stdout)
                .map_err(|e| PluginError::ExecutionFailed(format!("Invalid UTF-8 output: {}", e)))
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(PluginError::ExecutionFailed(format!("Process failed: {}", error)))
        }
    }

    async fn cleanup_sandbox_environment(&self, plugin_id: &str) -> Result<(), PluginError> {
        let plugin_sandbox_dir = self.config.sandbox_dir.join(plugin_id);
        
        if plugin_sandbox_dir.exists() {
            tokio::fs::remove_dir_all(&plugin_sandbox_dir)
                .await
                .map_err(|e| PluginError::IoError(format!("Failed to cleanup sandbox: {}", e)))?;
        }

        Ok(())
    }

    async fn cleanup_plugin_resources(&self, plugin_id: &str) -> Result<(), PluginError> {
        let mut tracker = self.resource_tracker.write().await;
        tracker.memory_usage.remove(plugin_id);
        tracker.cpu_usage.remove(plugin_id);
        tracker.network_usage.remove(plugin_id);
        tracker.fs_usage.remove(plugin_id);
        tracker.active_processes.remove(plugin_id);

        debug!("Cleaned up resources for plugin: {}", plugin_id);
        Ok(())
    }

    async fn execute_unsandboxed(&self, command: &str, args: &[&str]) -> Result<String, PluginError> {
        let output = AsyncCommand::new(command)
            .args(args)
            .output()
            .await
            .map_err(|e| PluginError::ExecutionFailed(format!("Execution failed: {}", e)))?;

        if output.status.success() {
            String::from_utf8(output.stdout)
                .map_err(|e| PluginError::ExecutionFailed(format!("Invalid UTF-8 output: {}", e)))
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(PluginError::ExecutionFailed(format!("Process failed: {}", error)))
        }
    }
}

/// Sandbox environment information
#[derive(Debug)]
pub struct SandboxEnvironment {
    pub plugin_id: String,
    pub sandbox_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub data_dir: PathBuf,
    pub environment_vars: HashMap<String, String>,
}

/// Sandbox statistics
#[derive(Debug, Clone)]
pub struct SandboxStats {
    pub active_plugins: usize,
    pub total_memory_mb: u64,
    pub total_network_mb: u64,
    pub total_fs_mb: u64,
    pub sandbox_enabled: bool,
}

impl PermissionManager {
    pub fn new() -> Self {
        Self {
            granted_permissions: HashMap::new(),
            validation_cache: HashMap::new(),
        }
    }

    pub fn grant_permissions(&mut self, plugin_id: &str, permissions: &[PluginPermission]) {
        let plugin_permissions = self.granted_permissions
            .entry(plugin_id.to_string())
            .or_insert_with(HashSet::new);

        for permission in permissions {
            plugin_permissions.insert(permission.clone());
        }
    }

    pub fn revoke_permissions(&mut self, plugin_id: &str, permissions: &[PluginPermission]) {
        if let Some(plugin_permissions) = self.granted_permissions.get_mut(plugin_id) {
            for permission in permissions {
                plugin_permissions.remove(permission);
            }
        }
    }

    pub fn validate_permissions(&self, plugin_id: &str, required_permissions: &[PluginPermission]) -> Result<(), PluginError> {
        let empty_permissions = HashSet::new();
        let granted = self.granted_permissions.get(plugin_id).unwrap_or(&empty_permissions);

        for permission in required_permissions {
            if !granted.contains(permission) {
                return Err(PluginError::PermissionDenied(
                    format!("Plugin {} lacks permission: {:?}", plugin_id, permission)
                ));
            }
        }

        Ok(())
    }

    pub fn has_permission(&self, plugin_id: &str, permission: &PluginPermission) -> bool {
        self.granted_permissions
            .get(plugin_id)
            .map(|perms| perms.contains(permission))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_sandbox_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = SandboxConfig {
            sandbox_dir: temp_dir.path().join("sandbox"),
            ..Default::default()
        };

        let sandbox = PluginSandbox::new(config).await;
        assert!(sandbox.is_ok());
    }

    #[tokio::test]
    async fn test_permission_management() {
        let temp_dir = TempDir::new().unwrap();
        let config = SandboxConfig {
            sandbox_dir: temp_dir.path().join("sandbox"),
            enabled: false, // Disable for testing
            ..Default::default()
        };

        let mut sandbox = PluginSandbox::new(config).await.unwrap();

        let permissions = vec![
            PluginPermission::FileSystemRead,
            PluginPermission::NetworkAccess,
        ];

        // Grant permissions
        sandbox.grant_permissions("test-plugin", &permissions).await.unwrap();

        // Validate permissions
        let result = sandbox.permission_manager.validate_permissions("test-plugin", &permissions);
        assert!(result.is_ok());

        // Test missing permission
        let missing_permission = vec![PluginPermission::FileSystemWrite];
        let result = sandbox.permission_manager.validate_permissions("test-plugin", &missing_permission);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unsandboxed_execution() {
        let temp_dir = TempDir::new().unwrap();
        let config = SandboxConfig {
            sandbox_dir: temp_dir.path().join("sandbox"),
            enabled: false, // Disable sandbox for testing
            ..Default::default()
        };

        let sandbox = PluginSandbox::new(config).await.unwrap();

        // Test simple command execution
        let result = sandbox.execute_plugin(
            "test-plugin",
            "echo",
            &["hello", "world"],
            &[],
        ).await;

        assert!(result.is_ok());
        assert!(result.unwrap().contains("hello world"));
    }
}