//! Resource limits and monitoring for sandbox processes

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{ResourceUsage, SandboxError};

/// Resource limits for sandboxed processes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum CPU time in milliseconds
    pub max_cpu_time_ms: Option<u64>,
    /// Maximum memory usage in kilobytes
    pub max_memory_kb: Option<u64>,
    /// Maximum disk space usage in bytes
    pub max_disk_bytes: Option<u64>,
    /// Maximum number of file descriptors
    pub max_file_descriptors: Option<u32>,
    /// Maximum number of child processes
    pub max_processes: Option<u32>,
    /// Maximum network bandwidth in bytes/sec
    pub max_network_bps: Option<u64>,
}

/// Resource monitor tracks usage across sandboxes
#[derive(Debug)]
pub struct ResourceMonitor {
    active_monitors: RwLock<HashMap<Uuid, SandboxMonitor>>,
}

/// Individual sandbox resource monitor
#[derive(Debug)]
struct SandboxMonitor {
    sandbox_id: Uuid,
    start_time: std::time::Instant,
    current_usage: ResourceUsage,
    peak_usage: ResourceUsage,
    limits: ResourceLimits,
    violations: Vec<super::PolicyViolation>,
}

impl ResourceLimits {
    /// Create conservative limits for untrusted code
    pub fn conservative() -> Self {
        Self {
            max_cpu_time_ms: Some(30_000), // 30 seconds
            max_memory_kb: Some(512_000), // 512 MB
            max_disk_bytes: Some(100_000_000), // 100 MB
            max_file_descriptors: Some(100),
            max_processes: Some(10),
            max_network_bps: Some(1_000_000), // 1 MB/s
        }
    }
    
    /// Create permissive limits for trusted operations
    pub fn permissive() -> Self {
        Self {
            max_cpu_time_ms: Some(300_000), // 5 minutes
            max_memory_kb: Some(2_048_000), // 2 GB
            max_disk_bytes: Some(1_000_000_000), // 1 GB
            max_file_descriptors: Some(1000),
            max_processes: Some(100),
            max_network_bps: Some(10_000_000), // 10 MB/s
        }
    }
    
    /// Check if current usage violates limits
    pub fn check_violation(&self, usage: &ResourceUsage) -> Vec<String> {
        let mut violations = Vec::new();
        
        if let Some(max_cpu) = self.max_cpu_time_ms {
            if usage.cpu_time_ms > max_cpu {
                violations.push(format!("CPU time exceeded: {} > {} ms", usage.cpu_time_ms, max_cpu));
            }
        }
        
        if let Some(max_memory) = self.max_memory_kb {
            if usage.peak_memory_kb > max_memory {
                violations.push(format!("Memory exceeded: {} > {} KB", usage.peak_memory_kb, max_memory));
            }
        }
        
        if let Some(max_disk) = self.max_disk_bytes {
            let total_disk = usage.disk_read_bytes + usage.disk_write_bytes;
            if total_disk > max_disk {
                violations.push(format!("Disk I/O exceeded: {} > {} bytes", total_disk, max_disk));
            }
        }
        
        if let Some(max_network) = self.max_network_bps {
            let total_network = usage.network_rx_bytes + usage.network_tx_bytes;
            if total_network > max_network {
                violations.push(format!("Network I/O exceeded: {} > {} bytes", total_network, max_network));
            }
        }
        
        violations
    }
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new() -> Self {
        Self {
            active_monitors: RwLock::new(HashMap::new()),
        }
    }
    
    /// Start monitoring a sandbox
    pub async fn start_monitoring(&self, sandbox_id: Uuid) {
        let monitor = SandboxMonitor {
            sandbox_id,
            start_time: std::time::Instant::now(),
            current_usage: ResourceUsage::default(),
            peak_usage: ResourceUsage::default(),
            limits: ResourceLimits::default(),
            violations: Vec::new(),
        };
        
        let mut monitors = self.active_monitors.write().await;
        monitors.insert(sandbox_id, monitor);
    }
    
    /// Stop monitoring a sandbox
    pub async fn stop_monitoring(&self, sandbox_id: Uuid) {
        let mut monitors = self.active_monitors.write().await;
        monitors.remove(&sandbox_id);
    }
    
    /// Get current resource usage for a sandbox
    pub async fn get_usage(&self, sandbox_id: Uuid) -> Option<ResourceUsage> {
        let monitors = self.active_monitors.read().await;
        monitors.get(&sandbox_id).map(|m| m.current_usage.clone())
    }
    
    /// Update resource usage for a sandbox
    pub async fn update_usage(&self, sandbox_id: Uuid, usage: ResourceUsage) -> Result<(), SandboxError> {
        let mut monitors = self.active_monitors.write().await;
        
        if let Some(monitor) = monitors.get_mut(&sandbox_id) {
            monitor.current_usage = usage.clone();
            
            // Update peak usage
            monitor.peak_usage.cpu_time_ms = monitor.peak_usage.cpu_time_ms.max(usage.cpu_time_ms);
            monitor.peak_usage.peak_memory_kb = monitor.peak_usage.peak_memory_kb.max(usage.peak_memory_kb);
            monitor.peak_usage.disk_read_bytes = monitor.peak_usage.disk_read_bytes.max(usage.disk_read_bytes);
            monitor.peak_usage.disk_write_bytes = monitor.peak_usage.disk_write_bytes.max(usage.disk_write_bytes);
            monitor.peak_usage.network_rx_bytes = monitor.peak_usage.network_rx_bytes.max(usage.network_rx_bytes);
            monitor.peak_usage.network_tx_bytes = monitor.peak_usage.network_tx_bytes.max(usage.network_tx_bytes);
            
            // Check for violations
            let violations = monitor.limits.check_violation(&usage);
            for violation in violations {
                let policy_violation = super::PolicyViolation {
                    violation_type: super::ViolationType::ResourceLimit,
                    description: violation.clone(),
                    timestamp: chrono::Utc::now(),
                    severity: super::ViolationSeverity::Warning,
                };
                monitor.violations.push(policy_violation);
                
                // Could trigger enforcement action here
                tracing::warn!("Resource limit violation in sandbox {}: {}", sandbox_id, violation);
            }
        }
        
        Ok(())
    }
    
    /// Get resource monitoring statistics
    pub async fn get_monitoring_stats(&self) -> MonitoringStats {
        let monitors = self.active_monitors.read().await;
        
        let total_sandboxes = monitors.len();
        let total_cpu_time: u64 = monitors.values().map(|m| m.current_usage.cpu_time_ms).sum();
        let total_memory: u64 = monitors.values().map(|m| m.current_usage.peak_memory_kb).sum();
        let total_violations: usize = monitors.values().map(|m| m.violations.len()).sum();
        
        MonitoringStats {
            active_sandboxes: total_sandboxes,
            total_cpu_time_ms: total_cpu_time,
            total_memory_kb: total_memory,
            total_violations,
            uptime: std::time::Instant::now().duration_since(
                monitors.values().map(|m| m.start_time).min().unwrap_or_else(std::time::Instant::now)
            ),
        }
    }
}

/// Resource monitoring statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringStats {
    pub active_sandboxes: usize,
    pub total_cpu_time_ms: u64,
    pub total_memory_kb: u64,
    pub total_violations: usize,
    pub uptime: Duration,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_time_ms: Some(60_000), // 1 minute
            max_memory_kb: Some(1_024_000), // 1 GB
            max_disk_bytes: Some(500_000_000), // 500 MB
            max_file_descriptors: Some(500),
            max_processes: Some(50),
            max_network_bps: Some(5_000_000), // 5 MB/s
        }
    }
}

#[cfg(unix)]
mod unix_monitoring {
    use super::*;
    use std::fs;
    use std::str::FromStr;
    
    /// Get process resource usage on Unix systems
    pub fn get_process_usage(pid: u32) -> Result<ResourceUsage, SandboxError> {
        // Read from /proc/[pid]/stat for basic stats
        let stat_path = format!("/proc/{}/stat", pid);
        let stat_content = fs::read_to_string(stat_path)
            .map_err(|e| SandboxError::SystemError(format!("Failed to read process stats: {}", e)))?;
        
        let fields: Vec<&str> = stat_content.split_whitespace().collect();
        if fields.len() < 24 {
            return Err(SandboxError::SystemError("Invalid stat format".to_string()));
        }
        
        // Parse CPU time (user + system time)
        let utime = u64::from_str(fields[13])
            .map_err(|_| SandboxError::SystemError("Invalid utime".to_string()))?;
        let stime = u64::from_str(fields[14])
            .map_err(|_| SandboxError::SystemError("Invalid stime".to_string()))?;
        
        // Convert from clock ticks to milliseconds (assuming 100 Hz)
        let cpu_time_ms = (utime + stime) * 10;
        
        // Parse memory usage (RSS in pages)
        let rss_pages = u64::from_str(fields[23])
            .map_err(|_| SandboxError::SystemError("Invalid RSS".to_string()))?;
        let peak_memory_kb = rss_pages * 4; // Assuming 4KB pages
        
        // For disk and network I/O, we'd need to read from /proc/[pid]/io
        let (disk_read_bytes, disk_write_bytes, network_rx_bytes, network_tx_bytes) = 
            get_io_stats(pid).unwrap_or((0, 0, 0, 0));
        
        Ok(ResourceUsage {
            cpu_time_ms,
            peak_memory_kb,
            disk_read_bytes,
            disk_write_bytes,
            network_rx_bytes,
            network_tx_bytes,
        })
    }
    
    fn get_io_stats(pid: u32) -> Option<(u64, u64, u64, u64)> {
        // Read I/O stats from /proc/[pid]/io
        let io_path = format!("/proc/{}/io", pid);
        let io_content = fs::read_to_string(io_path).ok()?;
        
        let mut read_bytes = 0;
        let mut write_bytes = 0;
        
        for line in io_content.lines() {
            if line.starts_with("read_bytes:") {
                read_bytes = line.split_whitespace().nth(1)?.parse().ok()?;
            } else if line.starts_with("write_bytes:") {
                write_bytes = line.split_whitespace().nth(1)?.parse().ok()?;
            }
        }
        
        // Network stats would require more complex parsing from /proc/net/dev
        // For now, return zero for network stats
        Some((read_bytes, write_bytes, 0, 0))
    }
}

#[cfg(unix)]
pub use unix_monitoring::get_process_usage;

#[cfg(not(unix))]
pub fn get_process_usage(_pid: u32) -> Result<ResourceUsage, SandboxError> {
    // Placeholder for non-Unix systems
    Ok(ResourceUsage::default())
}