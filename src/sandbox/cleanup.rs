//! Process cleanup and orphaned process management

use std::collections::HashSet;
use tokio::sync::RwLock;
use super::SandboxError;

#[derive(Debug)]
pub struct ProcessCleanup {
    tracked_processes: RwLock<HashSet<u32>>,
}

impl ProcessCleanup {
    pub fn new() -> Self {
        Self {
            tracked_processes: RwLock::new(HashSet::new()),
        }
    }
    
    pub async fn track_process(&self, pid: u32) {
        let mut processes = self.tracked_processes.write().await;
        processes.insert(pid);
    }
    
    pub async fn untrack_process(&self, pid: u32) {
        let mut processes = self.tracked_processes.write().await;
        processes.remove(&pid);
    }
    
    pub async fn cleanup_all(&self) -> Result<(), SandboxError> {
        let processes = self.tracked_processes.read().await;
        
        for &pid in processes.iter() {
            #[cfg(unix)]
            {
                // Send SIGTERM first, then SIGKILL if needed
                unsafe {
                    libc::kill(pid as i32, libc::SIGTERM);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    libc::kill(pid as i32, libc::SIGKILL);
                }
            }
        }
        
        Ok(())
    }
}