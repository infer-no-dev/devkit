//! Deterministic orchestration primitives for agents: retries, timeouts, cancellation, snapshots

use crate::agents::task::AgentTask;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio_util::sync::CancellationToken;

#[cfg(test)]
use tempfile;

/// Retry policy with deterministic backoff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: usize,
    pub strategy: BackoffStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    Fixed { delay_secs: u64 },
    Exponential { base_secs: u64, factor: u32, max_secs: u64 },
}

impl RetryPolicy {
    pub fn next_delay(&self, attempt: usize) -> Option<Duration> {
        if attempt >= self.max_retries {
            return None;
        }
        let d = match self.strategy {
            BackoffStrategy::Fixed { delay_secs } => Duration::from_secs(delay_secs),
            BackoffStrategy::Exponential { base_secs, factor, max_secs } => {
                let pow = factor.saturating_pow(attempt as u32).max(1);
                let secs = (base_secs.saturating_mul(pow as u64)).min(max_secs);
                Duration::from_secs(secs)
            }
        };
        Some(d)
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self { max_retries: 3, strategy: BackoffStrategy::Exponential { base_secs: 10, factor: 2, max_secs: 300 } }
    }
}

/// Snapshot status for a task
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskSnapshotStatus {
    Pending,
    Running,
    Failed,
    Completed,
    Canceled,
}

/// Serializable snapshot of a task's orchestration state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSnapshot {
    pub task_id: String,
    pub task: AgentTask,
    pub status: TaskSnapshotStatus,
    pub attempt: usize,
    pub last_error: Option<String>,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TaskSnapshot {
    pub fn new_pending(task: AgentTask) -> Self {
        let now = Utc::now();
        Self {
            task_id: task.id.clone(),
            task,
            status: TaskSnapshotStatus::Pending,
            attempt: 0,
            last_error: None,
            next_retry_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Persist snapshots to disk
#[derive(Debug, Clone)]
pub struct FileTaskSnapshotStore {
    root: PathBuf,
}

impl FileTaskSnapshotStore {
    pub async fn new<P: AsRef<Path>>(root: P) -> Result<Self, std::io::Error> {
        let path = root.as_ref().to_path_buf();
        fs::create_dir_all(&path).await?;
        Ok(Self { root: path })
    }

    fn path_for(&self, task_id: &str) -> PathBuf {
        self.root.join(format!("{}.json", task_id))
    }

    pub async fn save(&self, snap: &TaskSnapshot) -> Result<(), std::io::Error> {
        let p = self.path_for(&snap.task_id);
        let data = serde_json::to_vec_pretty(snap).expect("serialize snapshot");
        let mut f = fs::File::create(&p).await?;
        f.write_all(&data).await?;
        Ok(())
    }

    pub async fn load(&self, task_id: &str) -> Result<Option<TaskSnapshot>, std::io::Error> {
        let p = self.path_for(task_id);
        if !p.exists() { return Ok(None); }
        let data = fs::read(&p).await?;
        let snap: TaskSnapshot = serde_json::from_slice(&data).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Some(snap))
    }

    pub async fn delete(&self, task_id: &str) -> Result<(), std::io::Error> {
        let p = self.path_for(task_id);
        if p.exists() { fs::remove_file(p).await?; }
        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<TaskSnapshot>, std::io::Error> {
        let mut out = Vec::new();
        let mut rd = fs::read_dir(&self.root).await?;
        while let Some(entry) = rd.next_entry().await? {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(data) = fs::read(entry.path()).await {
                    if let Ok(snap) = serde_json::from_slice::<TaskSnapshot>(&data) {
                        out.push(snap);
                    }
                }
            }
        }
        Ok(out)
    }
}

/// Cancellation registry
#[derive(Debug, Default)]
pub struct CancellationRegistry;

impl CancellationRegistry {
    pub fn new_token() -> CancellationToken { CancellationToken::new() }
}
