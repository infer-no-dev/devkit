//! Git repository analysis and integration.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::context::ContextError;

/// Information about a Git repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    pub root_path: PathBuf,
    pub current_branch: Option<String>,
    pub remote_url: Option<String>,
    pub commit_count: usize,
    pub status: RepositoryStatus,
    pub recent_commits: Vec<CommitInfo>,
}

/// Status of a Git repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryStatus {
    pub modified_files: Vec<PathBuf>,
    pub untracked_files: Vec<PathBuf>,
    pub staged_files: Vec<PathBuf>,
    pub is_clean: bool,
}

/// Information about a Git commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub author: String,
    pub message: String,
    pub timestamp: std::time::SystemTime,
    pub files_changed: Vec<PathBuf>,
}

/// Repository analyzer for extracting Git information
#[derive(Debug)]
pub struct RepositoryAnalyzer {
    // Analyzer state
}

impl RepositoryAnalyzer {
    /// Create a new repository analyzer
    pub fn new() -> Result<Self, ContextError> {
        Ok(Self {})
    }
    
    /// Analyze a Git repository at the given path
    pub async fn analyze(&self, path: &PathBuf) -> Result<RepositoryInfo, ContextError> {
        // Check if this is a Git repository
        let git_dir = path.join(".git");
        if !git_dir.exists() {
            return Err(ContextError::RepositoryAnalysisFailed(
                "Not a Git repository".to_string()
            ));
        }
        
        let current_branch = self.get_current_branch(path)?;
        let remote_url = self.get_remote_url(path)?;
        let status = self.get_repository_status(path)?;
        let recent_commits = self.get_recent_commits(path, 10)?;
        let commit_count = recent_commits.len();
        
        Ok(RepositoryInfo {
            root_path: path.clone(),
            current_branch,
            remote_url,
            commit_count,
            status,
            recent_commits,
        })
    }
    
    /// Get the current branch name
    fn get_current_branch(&self, repo_path: &PathBuf) -> Result<Option<String>, ContextError> {
        let head_file = repo_path.join(".git/HEAD");
        if let Ok(head_content) = std::fs::read_to_string(&head_file) {
            if head_content.starts_with("ref: refs/heads/") {
                let branch_name = head_content
                    .strip_prefix("ref: refs/heads/")
                    .unwrap_or("")
                    .trim()
                    .to_string();
                return Ok(Some(branch_name));
            }
        }
        Ok(None)
    }
    
    /// Get the remote URL
    fn get_remote_url(&self, repo_path: &PathBuf) -> Result<Option<String>, ContextError> {
        let config_file = repo_path.join(".git/config");
        if let Ok(config_content) = std::fs::read_to_string(&config_file) {
            // Simple parsing of git config - look for remote "origin" url
            let lines: Vec<&str> = config_content.lines().collect();
            let mut in_remote_origin = false;
            
            for line in lines {
                let trimmed = line.trim();
                if trimmed == r#"[remote "origin"]"# {
                    in_remote_origin = true;
                    continue;
                }
                
                if trimmed.starts_with('[') && trimmed != r#"[remote "origin"]"# {
                    in_remote_origin = false;
                    continue;
                }
                
                if in_remote_origin && trimmed.starts_with("url = ") {
                    let url = trimmed.strip_prefix("url = ").unwrap_or("").to_string();
                    return Ok(Some(url));
                }
            }
        }
        Ok(None)
    }
    
    /// Get repository status (enhanced implementation using git commands)
    fn get_repository_status(&self, repo_path: &PathBuf) -> Result<RepositoryStatus, ContextError> {
        use std::process::Command;
        
        let mut modified_files = Vec::new();
        let mut untracked_files = Vec::new();
        let mut staged_files = Vec::new();
        
        // Run git status --porcelain to get machine-readable status
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(repo_path)
            .output();
            
        if let Ok(output) = output {
            if output.status.success() {
                let status_text = String::from_utf8_lossy(&output.stdout);
                
                for line in status_text.lines() {
                    if line.len() < 3 {
                        continue;
                    }
                    
                    let index_status = &line[0..1];
                    let work_status = &line[1..2];
                    let file_path = PathBuf::from(&line[3..]);
                    
                    // Parse git status codes
                    match (index_status, work_status) {
                        (" ", "M") | (" ", "T") => modified_files.push(file_path),
                        ("?", "?") => untracked_files.push(file_path),
                        ("A", " ") | ("M", " ") | ("D", " ") | ("R", " ") | ("C", " ") => staged_files.push(file_path),
                        ("A", "M") | ("M", "M") => {
                            staged_files.push(file_path.clone());
                            modified_files.push(file_path);
                        }
                        _ => {}
                    }
                }
            }
        }
        
        let is_clean = modified_files.is_empty() && untracked_files.is_empty() && staged_files.is_empty();
        
        Ok(RepositoryStatus {
            modified_files,
            untracked_files,
            staged_files,
            is_clean,
        })
    }
    
    /// Get recent commits (enhanced implementation using git log)
    fn get_recent_commits(&self, repo_path: &PathBuf, limit: usize) -> Result<Vec<CommitInfo>, ContextError> {
        use std::process::Command;
        
        let output = Command::new("git")
            .args([
                "log",
                &format!("-{}", limit),
                "--pretty=format:%H|%an|%s|%ct",
                "--name-only"
            ])
            .current_dir(repo_path)
            .output();
            
        if let Ok(output) = output {
            if output.status.success() {
                let log_text = String::from_utf8_lossy(&output.stdout);
                return self.parse_git_log(&log_text);
            }
        }
        
        // Fallback to empty list if git command fails
        Ok(Vec::new())
    }
    
    /// Parse git log output into CommitInfo structs
    fn parse_git_log(&self, log_text: &str) -> Result<Vec<CommitInfo>, ContextError> {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let mut commits = Vec::new();
        let mut current_commit: Option<CommitInfo> = None;
        
        for line in log_text.lines() {
            let line = line.trim();
            
            if line.is_empty() {
                continue;
            }
            
            // Check if this is a commit header line (contains |)
            if line.contains('|') {
                // Save previous commit if exists
                if let Some(commit) = current_commit.take() {
                    commits.push(commit);
                }
                
                // Parse new commit header
                let parts: Vec<&str> = line.splitn(4, '|').collect();
                if parts.len() == 4 {
                    let hash = parts[0].to_string();
                    let author = parts[1].to_string();
                    let message = parts[2].to_string();
                    
                    let timestamp = if let Ok(ts) = parts[3].parse::<u64>() {
                        UNIX_EPOCH + std::time::Duration::from_secs(ts)
                    } else {
                        SystemTime::now()
                    };
                    
                    current_commit = Some(CommitInfo {
                        hash,
                        author,
                        message,
                        timestamp,
                        files_changed: Vec::new(),
                    });
                }
            } else if let Some(ref mut commit) = current_commit {
                // This should be a changed file
                if !line.is_empty() {
                    commit.files_changed.push(PathBuf::from(line));
                }
            }
        }
        
        // Don't forget the last commit
        if let Some(commit) = current_commit {
            commits.push(commit);
        }
        
        Ok(commits)
    }
    
    /// Check if a path is within a Git repository
    pub fn is_git_repository(&self, path: &PathBuf) -> bool {
        let mut current_path = path.clone();
        
        loop {
            if current_path.join(".git").exists() {
                return true;
            }
            
            if let Some(parent) = current_path.parent() {
                current_path = parent.to_path_buf();
            } else {
                break;
            }
        }
        
        false
    }
    
    /// Find the root of a Git repository
    pub fn find_repository_root(&self, path: &PathBuf) -> Option<PathBuf> {
        let mut current_path = path.clone();
        
        loop {
            if current_path.join(".git").exists() {
                return Some(current_path);
            }
            
            if let Some(parent) = current_path.parent() {
                current_path = parent.to_path_buf();
            } else {
                break;
            }
        }
        
        None
    }
}
