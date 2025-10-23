//! Version Control System integration with Git hooks and PR automation
//!
//! This module provides comprehensive Git integration including automatic branch
//! creation, PR workflows, commit message generation, and CI/CD hooks for quality gates.

pub mod git;
pub mod hooks;
pub mod pull_requests;
pub mod commit_messages;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;

pub use git::{GitRepository, GitError};
pub use hooks::{HookManager, GitHook, HookEvent};
pub use pull_requests::{PullRequestManager, PullRequest, PullRequestStatus};
pub use commit_messages::{CommitMessageGenerator, CommitTemplate};

/// VCS integration manager
#[derive(Debug)]
pub struct VcsManager {
    config: VcsConfig,
    repositories: RwLock<HashMap<PathBuf, GitRepository>>,
    hook_manager: HookManager,
    pr_manager: PullRequestManager,
    commit_generator: CommitMessageGenerator,
}

/// Configuration for VCS integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcsConfig {
    /// Git configuration
    pub git: GitConfig,
    /// GitHub/GitLab integration
    pub remote_integration: RemoteIntegrationConfig,
    /// Hook configuration
    pub hooks: HooksConfig,
    /// Commit message generation
    pub commit_messages: CommitMessageConfig,
    /// PR automation settings
    pub pull_requests: PullRequestConfig,
}

/// Git-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    /// Default branch name for new repositories
    pub default_branch: String,
    /// Whether to automatically create .gitignore files
    pub auto_gitignore: bool,
    /// Whether to sign commits by default
    pub sign_commits: bool,
    /// GPG key ID for signing (if enabled)
    pub signing_key: Option<String>,
    /// Whether to auto-stage changes before commit
    pub auto_stage: bool,
}

/// Remote integration configuration (GitHub, GitLab, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteIntegrationConfig {
    /// Platform type (GitHub, GitLab, Bitbucket, etc.)
    pub platform: VcsPlatform,
    /// API base URL (for self-hosted instances)
    pub api_url: Option<String>,
    /// Authentication token (stored securely)
    pub token_name: Option<String>,
    /// Default organization/username
    pub default_owner: Option<String>,
    /// Whether to enable PR automation
    pub enable_pr_automation: bool,
    /// Whether to enable issue linking
    pub enable_issue_linking: bool,
}

/// VCS platform types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VcsPlatform {
    GitHub,
    GitLab,
    Bitbucket,
    Azure,
    Custom(String),
}

/// Hook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksConfig {
    /// Whether to automatically install hooks
    pub auto_install: bool,
    /// Pre-commit hooks to enable
    pub pre_commit_hooks: Vec<String>,
    /// Pre-push hooks to enable
    pub pre_push_hooks: Vec<String>,
    /// Post-commit hooks to enable
    pub post_commit_hooks: Vec<String>,
    /// Custom hook configurations
    pub custom_hooks: HashMap<String, HookDefinition>,
}

/// Definition of a custom hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookDefinition {
    pub name: String,
    pub command: String,
    pub working_directory: Option<PathBuf>,
    pub environment: HashMap<String, String>,
    pub timeout_seconds: u64,
    pub fail_on_error: bool,
}

/// Commit message configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitMessageConfig {
    /// Whether to generate commit messages automatically
    pub auto_generate: bool,
    /// Commit message format/template
    pub template: CommitMessageTemplate,
    /// Whether to include file change summaries
    pub include_change_summary: bool,
    /// Maximum commit message length
    pub max_length: usize,
    /// Whether to follow conventional commits format
    pub conventional_commits: bool,
}

/// Template for commit message generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitMessageTemplate {
    /// Template for the commit title
    pub title_template: String,
    /// Template for the commit body
    pub body_template: Option<String>,
    /// Available variables for substitution
    pub variables: Vec<String>,
}

/// Pull request configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestConfig {
    /// Whether to automatically create PRs for feature branches
    pub auto_create_pr: bool,
    /// Default PR template
    pub template: Option<String>,
    /// Whether to auto-assign reviewers
    pub auto_assign_reviewers: bool,
    /// Default reviewers to assign
    pub default_reviewers: Vec<String>,
    /// Whether to auto-add labels
    pub auto_add_labels: bool,
    /// Default labels to add
    pub default_labels: Vec<String>,
    /// Whether to link PRs to issues automatically
    pub auto_link_issues: bool,
    /// Whether to enable draft PRs by default
    pub default_draft: bool,
}

/// Branch workflow strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BranchStrategy {
    /// Git Flow (feature/*, hotfix/*, release/*)
    GitFlow,
    /// GitHub Flow (feature branches to main)
    GitHubFlow,
    /// Simple feature branches
    FeatureBranches,
    /// Custom strategy
    Custom(CustomBranchStrategy),
}

/// Custom branch strategy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomBranchStrategy {
    pub main_branch: String,
    pub feature_prefix: String,
    pub hotfix_prefix: String,
    pub release_prefix: String,
}

/// Workflow automation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub branch_created: Option<String>,
    pub commits_created: Vec<String>,
    pub pr_created: Option<PullRequest>,
    pub hooks_executed: Vec<String>,
    pub quality_checks_passed: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// VCS integration errors
#[derive(Debug, thiserror::Error)]
pub enum VcsError {
    #[error("Git error: {0}")]
    GitError(#[from] GitError),
    
    #[error("Hook execution failed: {0}")]
    HookFailed(String),
    
    #[error("Remote API error: {0}")]
    RemoteApiError(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Branch operation failed: {0}")]
    BranchOperationFailed(String),
    
    #[error("PR creation failed: {0}")]
    PrCreationFailed(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl VcsManager {
    /// Create a new VCS manager
    pub fn new(config: VcsConfig) -> Result<Self, VcsError> {
        let hook_manager = HookManager::new(config.hooks.clone());
        let pr_manager = PullRequestManager::new(config.remote_integration.clone())?;
        let commit_generator = CommitMessageGenerator::new(config.commit_messages.clone());
        
        Ok(Self {
            config,
            repositories: RwLock::new(HashMap::new()),
            hook_manager,
            pr_manager,
            commit_generator,
        })
    }
    
    /// Initialize or get existing Git repository
    pub async fn get_repository(&self, path: &Path) -> Result<GitRepository, VcsError> {
        let mut repos = self.repositories.write().await;
        
        if let Some(repo) = repos.get(path) {
            Ok(repo.clone())
        } else {
            let repo = if path.join(".git").exists() {
                GitRepository::open(path)?
            } else {
                GitRepository::init(path, &self.config.git)?
            };
            
            repos.insert(path.to_path_buf(), repo.clone());
            Ok(repo)
        }
    }
    
    /// Create a new feature branch for agent work
    pub async fn create_feature_branch(
        &self,
        repo_path: &Path,
        task_description: &str,
        base_branch: Option<&str>,
    ) -> Result<String, VcsError> {
        let repo = self.get_repository(repo_path).await?;
        
        // Generate branch name from task description
        let branch_name = self.generate_branch_name(task_description);
        
        // Ensure we're on the base branch
        let base = base_branch.unwrap_or(&self.config.git.default_branch);
        repo.checkout_branch(base).await?;
        
        // Pull latest changes
        repo.pull().await?;
        
        // Create new feature branch
        repo.create_branch(&branch_name, Some(base)).await?;
        repo.checkout_branch(&branch_name).await?;
        
        Ok(branch_name)
    }
    
    /// Commit changes with auto-generated message
    pub async fn commit_changes(
        &self,
        repo_path: &Path,
        message: Option<&str>,
        files: Option<Vec<&Path>>,
    ) -> Result<String, VcsError> {
        let repo = self.get_repository(repo_path).await?;
        
        // Stage files
        if let Some(file_paths) = files {
            for file_path in file_paths {
                repo.add_file(file_path).await?;
            }
        } else if self.config.git.auto_stage {
            repo.add_all().await?;
        }
        
        // Generate commit message if not provided
        let commit_message = if let Some(msg) = message {
            msg.to_string()
        } else {
            self.commit_generator.generate_commit_message(&repo).await?
        };
        
        // Run pre-commit hooks
        self.hook_manager.run_hooks(HookEvent::PreCommit, repo_path).await?;
        
        // Create commit
        let commit_hash = repo.commit(&commit_message).await?;
        
        // Run post-commit hooks
        let _ = self.hook_manager.run_hooks(HookEvent::PostCommit, repo_path).await;
        
        Ok(commit_hash)
    }
    
    /// Create pull request for current branch
    pub async fn create_pull_request(
        &self,
        repo_path: &Path,
        title: Option<&str>,
        description: Option<&str>,
        target_branch: Option<&str>,
    ) -> Result<PullRequest, VcsError> {
        let repo = self.get_repository(repo_path).await?;
        
        // Get current branch
        let current_branch = repo.get_current_branch().await?;
        let target = target_branch.unwrap_or(&self.config.git.default_branch);
        
        // Push current branch to remote
        repo.push(&current_branch).await?;
        
        // Generate PR title and description if not provided
        let pr_title = if let Some(t) = title {
            t.to_string()
        } else {
            self.generate_pr_title(&current_branch, &repo).await?
        };
        
        let pr_description = if let Some(d) = description {
            d.to_string()
        } else {
            self.generate_pr_description(&current_branch, &repo).await?
        };
        
        // Create pull request via remote API
        let pr = self.pr_manager.create_pull_request(
            &repo.get_remote_url()?,
            &current_branch,
            target,
            &pr_title,
            &pr_description,
            &self.config.pull_requests,
        ).await?;
        
        Ok(pr)
    }
    
    /// Complete full workflow: branch -> commit -> PR
    pub async fn complete_workflow(
        &self,
        repo_path: &Path,
        task_description: &str,
        changed_files: Vec<&Path>,
        commit_message: Option<&str>,
    ) -> Result<WorkflowResult, VcsError> {
        let mut result = WorkflowResult {
            branch_created: None,
            commits_created: Vec::new(),
            pr_created: None,
            hooks_executed: Vec::new(),
            quality_checks_passed: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };
        
        // Create feature branch
        match self.create_feature_branch(repo_path, task_description, None).await {
            Ok(branch_name) => result.branch_created = Some(branch_name),
            Err(e) => {
                result.errors.push(format!("Branch creation failed: {}", e));
                return Ok(result);
            }
        }
        
        // Commit changes
        match self.commit_changes(repo_path, commit_message, Some(changed_files)).await {
            Ok(commit_hash) => result.commits_created.push(commit_hash),
            Err(e) => {
                result.errors.push(format!("Commit failed: {}", e));
                result.quality_checks_passed = false;
                return Ok(result);
            }
        }
        
        // Create PR if configured
        if self.config.pull_requests.auto_create_pr {
            match self.create_pull_request(repo_path, None, None, None).await {
                Ok(pr) => result.pr_created = Some(pr),
                Err(e) => {
                    result.warnings.push(format!("PR creation failed: {}", e));
                }
            }
        }
        
        Ok(result)
    }
    
    /// Install Git hooks for a repository
    pub async fn install_hooks(&self, repo_path: &Path) -> Result<(), VcsError> {
        if self.config.hooks.auto_install {
            self.hook_manager.install_hooks(repo_path).await?;
        }
        Ok(())
    }
    
    /// Generate branch name from task description
    fn generate_branch_name(&self, task_description: &str) -> String {
        // Convert to lowercase, replace spaces with hyphens, remove special chars
        let sanitized = task_description
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { ' ' })
            .collect::<String>();
        
        let words: Vec<&str> = sanitized.split_whitespace().collect();
        let branch_name = words.join("-");
        
        // Truncate if too long
        if branch_name.len() > 50 {
            format!("feature/{}", &branch_name[..47])
        } else {
            format!("feature/{}", branch_name)
        }
    }
    
    /// Generate PR title from branch name and commits
    async fn generate_pr_title(&self, branch_name: &str, repo: &GitRepository) -> Result<String, VcsError> {
        // Extract feature name from branch
        let feature_name = branch_name
            .strip_prefix("feature/")
            .unwrap_or(branch_name)
            .replace("-", " ")
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<String>>()
            .join(" ");
        
        Ok(format!("feat: {}", feature_name))
    }
    
    /// Generate PR description from commits and changes
    async fn generate_pr_description(&self, branch_name: &str, repo: &GitRepository) -> Result<String, VcsError> {
        let mut description = String::new();
        
        description.push_str("## Changes\n\n");
        description.push_str(&format!("This PR implements changes from the `{}` branch.\n\n", branch_name));
        
        // Get recent commits
        if let Ok(commits) = repo.get_recent_commits(10).await {
            if !commits.is_empty() {
                description.push_str("## Recent Commits\n\n");
                for commit in &commits {
                    description.push_str(&format!("- {}\n", commit.message));
                }
                description.push('\n');
            }
        }
        
        // Add template sections if configured
        if let Some(template) = &self.config.pull_requests.template {
            description.push_str(template);
        } else {
            description.push_str("## Testing\n\n");
            description.push_str("- [ ] Manual testing completed\n");
            description.push_str("- [ ] Automated tests pass\n\n");
            description.push_str("## Review Checklist\n\n");
            description.push_str("- [ ] Code follows project standards\n");
            description.push_str("- [ ] Changes are well documented\n");
            description.push_str("- [ ] No breaking changes introduced\n");
        }
        
        Ok(description)
    }
    
    /// Get repository status and suggestions
    pub async fn get_repository_status(&self, repo_path: &Path) -> Result<RepositoryStatus, VcsError> {
        let repo = self.get_repository(repo_path).await?;
        
        let current_branch = repo.get_current_branch().await?;
        let staged_files = repo.get_staged_files().await?;
        let unstaged_files = repo.get_unstaged_files().await?;
        let untracked_files = repo.get_untracked_files().await?;
        let ahead_behind = repo.get_ahead_behind_count(&self.config.git.default_branch).await?;
        
        let suggestions = self.generate_workflow_suggestions(
            &current_branch,
            &staged_files,
            &unstaged_files,
            &ahead_behind,
        );
        
        Ok(RepositoryStatus {
            current_branch,
            staged_files,
            unstaged_files,
            untracked_files,
            commits_ahead: ahead_behind.0,
            commits_behind: ahead_behind.1,
            suggestions,
        })
    }
    
    /// Generate workflow suggestions based on repository state
    fn generate_workflow_suggestions(
        &self,
        current_branch: &str,
        staged_files: &[String],
        unstaged_files: &[String],
        ahead_behind: &(usize, usize),
    ) -> Vec<WorkflowSuggestion> {
        let mut suggestions = Vec::new();
        
        // Suggest staging files if there are unstaged changes
        if !unstaged_files.is_empty() {
            suggestions.push(WorkflowSuggestion {
                action: "stage_files".to_string(),
                description: format!("Stage {} unstaged files for commit", unstaged_files.len()),
                priority: SuggestionPriority::Medium,
            });
        }
        
        // Suggest committing if there are staged changes
        if !staged_files.is_empty() {
            suggestions.push(WorkflowSuggestion {
                action: "commit".to_string(),
                description: format!("Commit {} staged files", staged_files.len()),
                priority: SuggestionPriority::High,
            });
        }
        
        // Suggest pushing if commits are ahead
        if ahead_behind.0 > 0 {
            suggestions.push(WorkflowSuggestion {
                action: "push".to_string(),
                description: format!("Push {} commits to remote", ahead_behind.0),
                priority: SuggestionPriority::High,
            });
        }
        
        // Suggest pulling if commits are behind
        if ahead_behind.1 > 0 {
            suggestions.push(WorkflowSuggestion {
                action: "pull".to_string(),
                description: format!("Pull {} commits from remote", ahead_behind.1),
                priority: SuggestionPriority::Medium,
            });
        }
        
        // Suggest creating PR if on feature branch with pushed commits
        if current_branch.starts_with("feature/") && ahead_behind.0 == 0 && !staged_files.is_empty() {
            suggestions.push(WorkflowSuggestion {
                action: "create_pr".to_string(),
                description: "Create pull request for this feature branch".to_string(),
                priority: SuggestionPriority::Medium,
            });
        }
        
        suggestions
    }
}

/// Repository status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryStatus {
    pub current_branch: String,
    pub staged_files: Vec<String>,
    pub unstaged_files: Vec<String>,
    pub untracked_files: Vec<String>,
    pub commits_ahead: usize,
    pub commits_behind: usize,
    pub suggestions: Vec<WorkflowSuggestion>,
}

/// Workflow suggestion for the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSuggestion {
    pub action: String,
    pub description: String,
    pub priority: SuggestionPriority,
}

/// Priority level for workflow suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for VcsConfig {
    fn default() -> Self {
        Self {
            git: GitConfig::default(),
            remote_integration: RemoteIntegrationConfig::default(),
            hooks: HooksConfig::default(),
            commit_messages: CommitMessageConfig::default(),
            pull_requests: PullRequestConfig::default(),
        }
    }
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            default_branch: "main".to_string(),
            auto_gitignore: true,
            sign_commits: false,
            signing_key: None,
            auto_stage: false,
        }
    }
}

impl Default for RemoteIntegrationConfig {
    fn default() -> Self {
        Self {
            platform: VcsPlatform::GitHub,
            api_url: None,
            token_name: None,
            default_owner: None,
            enable_pr_automation: true,
            enable_issue_linking: true,
        }
    }
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            auto_install: true,
            pre_commit_hooks: vec![
                "format_check".to_string(),
                "lint_check".to_string(),
            ],
            pre_push_hooks: vec![
                "test_check".to_string(),
            ],
            post_commit_hooks: vec![],
            custom_hooks: HashMap::new(),
        }
    }
}

impl Default for CommitMessageConfig {
    fn default() -> Self {
        Self {
            auto_generate: true,
            template: CommitMessageTemplate::default(),
            include_change_summary: true,
            max_length: 72,
            conventional_commits: true,
        }
    }
}

impl Default for CommitMessageTemplate {
    fn default() -> Self {
        Self {
            title_template: "{type}: {description}".to_string(),
            body_template: Some("{details}\n\n{file_changes}".to_string()),
            variables: vec![
                "type".to_string(),
                "description".to_string(),
                "details".to_string(),
                "file_changes".to_string(),
            ],
        }
    }
}

impl Default for PullRequestConfig {
    fn default() -> Self {
        Self {
            auto_create_pr: false, // Conservative default
            template: None,
            auto_assign_reviewers: false,
            default_reviewers: Vec::new(),
            auto_add_labels: false,
            default_labels: Vec::new(),
            auto_link_issues: true,
            default_draft: false,
        }
    }
}