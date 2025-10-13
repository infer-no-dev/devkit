//! Advanced Session Management System
//!
//! This module provides comprehensive session management for DevKit, including
//! session persistence, recovery, branching, collaboration, and state management.
//! It enables users to save, load, branch, merge, and collaborate on development
//! sessions with full context preservation.

// pub mod analytics; // TODO: Implement analytics submodule
// pub mod collaboration; // TODO: Implement collaboration submodule
// pub mod persistence; // TODO: Implement persistence submodule
// pub mod recovery; // TODO: Implement recovery submodule
// pub mod state; // TODO: Implement state submodule

use crate::agents::{AgentStatus, TaskPriority, AgentResult};
use crate::config::ConfigManager;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info};
use uuid::Uuid;

// Expose persistence module
pub mod persistence;
pub use persistence::FileSystemPersistence;

// TODO: Re-enable when other submodules are implemented
// pub use analytics::{SessionAnalytics, SessionMetrics, PerformanceMetrics};
// pub use collaboration::{CollaborationManager, SessionShare, SharePermissions};
// pub use recovery::{RecoveryManager, CheckpointManager, SessionSnapshot};
// pub use state::{SessionState, StateManager, StateTransition};

// Placeholder types until submodules are implemented
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAnalytics {
    pub placeholder: bool,
}

impl SessionAnalytics {
    pub fn new() -> Self {
        Self { placeholder: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub placeholder: bool,
}

impl SessionState {
    pub fn new() -> Self {
        Self { placeholder: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Gzip,
    Zstd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharePermissions {
    pub read: bool,
    pub write: bool,
    pub admin: bool,
}

#[async_trait::async_trait]
pub trait SessionPersistence {
    async fn save_session(&self, session: &Session) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn load_session(&self, session_id: &str) -> Result<Option<Session>, Box<dyn std::error::Error + Send + Sync>>;
    async fn delete_session(&self, session_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn list_sessions(&self, user_id: &str) -> Result<Vec<Session>, Box<dyn std::error::Error + Send + Sync>>;
    async fn search_sessions(&self, user_id: &str, query: &str, filters: SessionFilters) -> Result<Vec<Session>, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct RecoveryManager {
    persistence: Arc<dyn SessionPersistence + Send + Sync>,
    checkpoint_interval: u64,
    max_snapshots: usize,
    checkpoints: Arc<RwLock<HashMap<String, Vec<SessionCheckpoint>>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCheckpoint {
    pub id: String,
    pub session_id: String,
    pub created_at: DateTime<Utc>,
    pub session_state: SessionState,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl RecoveryManager {
    pub fn new(
        persistence: Arc<dyn SessionPersistence + Send + Sync>,
        checkpoint_interval: u64,
        max_snapshots: usize,
    ) -> Result<Self, SessionError> {
        Ok(Self {
            persistence,
            checkpoint_interval,
            max_snapshots,
            checkpoints: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn create_checkpoint(&self, session_id: &str, session: &Session) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let checkpoint_id = format!("cp_{}_{}", session_id, Utc::now().timestamp());
        
        let checkpoint = SessionCheckpoint {
            id: checkpoint_id.clone(),
            session_id: session_id.to_string(),
            created_at: Utc::now(),
            session_state: session.state.clone(),
            metadata: HashMap::new(),
        };
        
        // Store checkpoint in memory
        {
            let mut checkpoints = self.checkpoints.write().await;
            let session_checkpoints = checkpoints.entry(session_id.to_string()).or_insert_with(Vec::new);
            session_checkpoints.push(checkpoint.clone());
            
            // Limit number of checkpoints
            if session_checkpoints.len() > self.max_snapshots {
                session_checkpoints.remove(0);
            }
        }
        
        // TODO: In a full implementation, you would also persist this to storage
        tracing::debug!("Created checkpoint {} for session {}", checkpoint_id, session_id);
        
        Ok(checkpoint_id)
    }

    pub async fn restore_checkpoint(&self, session_id: &str, checkpoint_id: &str) -> Result<Option<SessionCheckpoint>, Box<dyn std::error::Error + Send + Sync>> {
        let checkpoints = self.checkpoints.read().await;
        if let Some(session_checkpoints) = checkpoints.get(session_id) {
            let checkpoint = session_checkpoints.iter().find(|cp| cp.id == checkpoint_id).cloned();
            Ok(checkpoint)
        } else {
            Ok(None)
        }
    }

    pub async fn list_checkpoints(&self, session_id: &str) -> Result<Vec<SessionCheckpoint>, Box<dyn std::error::Error + Send + Sync>> {
        let checkpoints = self.checkpoints.read().await;
        if let Some(session_checkpoints) = checkpoints.get(session_id) {
            Ok(session_checkpoints.clone())
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn cleanup_session(&self, session_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut checkpoints = self.checkpoints.write().await;
        checkpoints.remove(session_id);
        tracing::debug!("Cleaned up checkpoints for session {}", session_id);
        Ok(())
    }

    pub async fn cleanup_old_checkpoints(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let cutoff_time = Utc::now() - chrono::Duration::hours(24); // Keep checkpoints for 24 hours
        let mut checkpoints = self.checkpoints.write().await;
        
        for (_session_id, session_checkpoints) in checkpoints.iter_mut() {
            session_checkpoints.retain(|cp| cp.created_at > cutoff_time);
        }
        
        // Remove empty entries
        checkpoints.retain(|_, checkpoints| !checkpoints.is_empty());
        
        Ok(())
    }
}

#[derive(Debug)]
pub struct CollaborationManager {
    shares: Arc<RwLock<HashMap<String, Vec<SessionShare>>>>, // session_id -> shares
    active_collaborators: Arc<RwLock<HashMap<String, HashSet<String>>>>, // session_id -> user_ids
    share_counter: Arc<RwLock<u64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionShare {
    pub id: String,
    pub session_id: String,
    pub permissions: SharePermissions,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationInfo {
    pub is_shared: bool,
    pub shares: Vec<SessionShare>,
    pub active_collaborators: HashSet<String>,
    pub owner: String,
}

impl CollaborationManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            shares: Arc::new(RwLock::new(HashMap::new())),
            active_collaborators: Arc::new(RwLock::new(HashMap::new())),
            share_counter: Arc::new(RwLock::new(0)),
        })
    }

    pub async fn create_share(&mut self, session_id: &str, permissions: SharePermissions) -> Result<SessionShare, Box<dyn std::error::Error + Send + Sync>> {
        let share_id = {
            let mut counter = self.share_counter.write().await;
            *counter += 1;
            format!("share_{}", *counter)
        };
        
        let share = SessionShare {
            id: share_id,
            session_id: session_id.to_string(),
            permissions,
            created_at: Utc::now(),
            created_by: "system".to_string(), // In real implementation, this would be the current user
            expires_at: None, // Could be configurable
            is_active: true,
        };
        
        // Store the share
        {
            let mut shares = self.shares.write().await;
            let session_shares = shares.entry(session_id.to_string()).or_insert_with(Vec::new);
            session_shares.push(share.clone());
        }
        
        tracing::debug!("Created share {} for session {} with permissions: read={}, write={}, admin={}", 
            share.id, session_id, share.permissions.read, share.permissions.write, share.permissions.admin);
        
        Ok(share)
    }

    pub async fn revoke_share(&mut self, session_id: &str, share_id: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut shares = self.shares.write().await;
        if let Some(session_shares) = shares.get_mut(session_id) {
            if let Some(share) = session_shares.iter_mut().find(|s| s.id == share_id) {
                share.is_active = false;
                tracing::debug!("Revoked share {} for session {}", share_id, session_id);
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub async fn list_shares(&self, session_id: &str) -> Result<Vec<SessionShare>, Box<dyn std::error::Error + Send + Sync>> {
        let shares = self.shares.read().await;
        if let Some(session_shares) = shares.get(session_id) {
            Ok(session_shares.iter().filter(|s| s.is_active).cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn add_collaborator(&mut self, session_id: &str, user_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut collaborators = self.active_collaborators.write().await;
        let session_collaborators = collaborators.entry(session_id.to_string()).or_insert_with(HashSet::new);
        session_collaborators.insert(user_id.to_string());
        
        tracing::debug!("Added collaborator {} to session {}", user_id, session_id);
        Ok(())
    }

    pub async fn remove_collaborator(&mut self, session_id: &str, user_id: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut collaborators = self.active_collaborators.write().await;
        if let Some(session_collaborators) = collaborators.get_mut(session_id) {
            let removed = session_collaborators.remove(user_id);
            if removed {
                tracing::debug!("Removed collaborator {} from session {}", user_id, session_id);
            }
            Ok(removed)
        } else {
            Ok(false)
        }
    }

    pub async fn get_collaborators(&self, session_id: &str) -> Result<HashSet<String>, Box<dyn std::error::Error + Send + Sync>> {
        let collaborators = self.active_collaborators.read().await;
        if let Some(session_collaborators) = collaborators.get(session_id) {
            Ok(session_collaborators.clone())
        } else {
            Ok(HashSet::new())
        }
    }

    pub async fn cleanup_session(&mut self, session_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        {
            let mut shares = self.shares.write().await;
            shares.remove(session_id);
        }
        
        {
            let mut collaborators = self.active_collaborators.write().await;
            collaborators.remove(session_id);
        }
        
        tracing::debug!("Cleaned up collaboration data for session {}", session_id);
        Ok(())
    }
}

/// Maximum number of sessions to keep in memory
const MAX_ACTIVE_SESSIONS: usize = 100;
/// Maximum number of session snapshots per session
const MAX_SNAPSHOTS_PER_SESSION: usize = 50;
/// Default session timeout in minutes
const DEFAULT_SESSION_TIMEOUT: u64 = 240; // 4 hours
/// Checkpoint interval in minutes
const CHECKPOINT_INTERVAL: u64 = 15;

/// Comprehensive session information and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier
    pub id: String,
    /// Human-readable session name
    pub name: String,
    /// Optional session description
    pub description: Option<String>,
    /// Session creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modification timestamp
    pub updated_at: DateTime<Utc>,
    /// Last access timestamp
    pub accessed_at: DateTime<Utc>,
    /// Session creator information
    pub creator: SessionUser,
    /// Current session status
    pub status: SessionStatus,
    /// Session configuration
    pub config: SessionConfig,
    /// Current session state
    pub state: SessionState,
    /// Session metadata and tags
    pub metadata: SessionMetadata,
    /// Active agents in this session
    pub agents: HashMap<String, AgentSessionInfo>,
    /// Session conversation history
    pub conversations: Vec<ConversationThread>,
    /// Generated artifacts
    pub artifacts: Vec<SessionArtifact>,
    /// Session branches (for experimentation)
    pub branches: HashMap<String, SessionBranch>,
    /// Current active branch
    pub active_branch: String,
    /// Collaboration information
    pub collaboration: Option<CollaborationInfo>,
    /// Session analytics
    pub analytics: SessionAnalytics,
    /// Custom session variables
    pub variables: HashMap<String, serde_json::Value>,
}

/// Session status enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionStatus {
    /// Session is active and ready for use
    Active,
    /// Session is temporarily paused
    Paused,
    /// Session is archived (read-only)
    Archived,
    /// Session has expired
    Expired,
    /// Session encountered an error
    Error { message: String, recoverable: bool },
    /// Session is being synchronized
    Syncing,
    /// Session is locked for exclusive access
    Locked { by_user: String, since: DateTime<Utc> },
}

/// Session configuration and preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session timeout in minutes (0 = no timeout)
    pub timeout_minutes: u64,
    /// Whether to auto-save session state
    pub auto_save: bool,
    /// Auto-save interval in minutes
    pub auto_save_interval: u64,
    /// Maximum number of conversation turns to keep
    pub max_conversation_history: usize,
    /// Maximum number of artifacts to keep
    pub max_artifacts: usize,
    /// Whether to enable real-time collaboration
    pub collaboration_enabled: bool,
    /// Compression settings for persistence
    pub compression: CompressionType,
    /// Backup settings
    pub backup_enabled: bool,
    pub backup_interval: Duration,
    pub max_backups: usize,
    /// Privacy and security settings
    pub privacy_level: PrivacyLevel,
    pub encryption_enabled: bool,
}

/// Privacy level for sessions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrivacyLevel {
    /// Session is private to creator only
    Private,
    /// Session can be shared with specific users
    Restricted,
    /// Session is publicly discoverable
    Public,
}

/// Session metadata and organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Categorization tags
    pub tags: HashSet<String>,
    /// Project association
    pub project: Option<String>,
    /// Priority level
    pub priority: SessionPriority,
    /// Associated repositories
    pub repositories: Vec<String>,
    /// Programming languages used
    pub languages: HashSet<String>,
    /// Frameworks and technologies
    pub technologies: HashSet<String>,
    /// Session goals and objectives
    pub objectives: Vec<String>,
    /// Custom metadata fields
    pub custom_fields: HashMap<String, String>,
}

/// Session priority levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// User information for session management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUser {
    /// User identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Email address (optional)
    pub email: Option<String>,
    /// User avatar URL (optional)
    pub avatar_url: Option<String>,
    /// User timezone
    pub timezone: String,
    /// User preferences
    pub preferences: UserPreferences,
}

/// User preferences for sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Default session configuration
    pub default_config: SessionConfig,
    /// Preferred theme
    pub theme: String,
    /// Language preferences
    pub language: String,
    /// Notification preferences
    pub notifications: NotificationPreferences,
    /// UI preferences
    pub ui_preferences: HashMap<String, serde_json::Value>,
}

/// Notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    /// Enable session status notifications
    pub session_status: bool,
    /// Enable agent completion notifications
    pub agent_completion: bool,
    /// Enable collaboration notifications
    pub collaboration: bool,
    /// Enable error notifications
    pub errors: bool,
    /// Notification delivery methods
    pub delivery_methods: Vec<NotificationMethod>,
}

/// Notification delivery methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationMethod {
    InApp,
    Email,
    Webhook { url: String },
    Slack { webhook_url: String },
    Discord { webhook_url: String },
}

/// Agent information within a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSessionInfo {
    /// Agent unique identifier
    pub id: String,
    /// Agent display name
    pub name: String,
    /// Agent type/specialization
    pub agent_type: String,
    /// Current status
    pub status: AgentStatus,
    /// Current task (if any)
    pub current_task: Option<TaskInfo>,
    /// Agent configuration for this session
    pub config: serde_json::Value,
    /// Behavior profile ID
    pub behavior_profile: Option<String>,
    /// Agent start time in this session
    pub started_at: DateTime<Utc>,
    /// Agent metrics for this session
    pub metrics: AgentSessionMetrics,
    /// Agent's working directory
    pub working_directory: Option<PathBuf>,
    /// Agent-specific variables
    pub variables: HashMap<String, serde_json::Value>,
}

/// Task information for session tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    /// Task identifier
    pub id: String,
    /// Task description
    pub description: String,
    /// Task priority
    pub priority: TaskPriority,
    /// Task start time
    pub started_at: DateTime<Utc>,
    /// Expected completion time
    pub estimated_completion: Option<DateTime<Utc>>,
    /// Task progress (0.0 to 1.0)
    pub progress: f64,
    /// Task dependencies
    pub dependencies: Vec<String>,
    /// Task artifacts produced
    pub artifacts: Vec<String>,
}

/// Agent metrics within a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSessionMetrics {
    /// Number of tasks completed
    pub tasks_completed: usize,
    /// Number of tasks failed
    pub tasks_failed: usize,
    /// Total processing time
    pub total_processing_time: Duration,
    /// Average task completion time
    pub average_completion_time: Duration,
    /// Number of artifacts generated
    pub artifacts_generated: usize,
    /// Number of interactions with other agents
    pub agent_interactions: usize,
    /// Error count
    pub errors: usize,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
}

/// Conversation thread within a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationThread {
    /// Thread identifier
    pub id: String,
    /// Thread title/topic
    pub title: String,
    /// Thread creation time
    pub created_at: DateTime<Utc>,
    /// Last activity time
    pub updated_at: DateTime<Utc>,
    /// Participant information
    pub participants: Vec<String>, // User IDs and Agent IDs
    /// Conversation messages
    pub messages: Vec<ConversationMessage>,
    /// Thread metadata
    pub metadata: HashMap<String, String>,
    /// Thread status
    pub status: ConversationStatus,
}

/// Status of a conversation thread
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConversationStatus {
    Active,
    Paused,
    Completed,
    Archived,
}

/// Individual message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    /// Message identifier
    pub id: String,
    /// Sender identifier (user or agent)
    pub sender_id: String,
    /// Sender type
    pub sender_type: MessageSenderType,
    /// Message timestamp
    pub timestamp: DateTime<Utc>,
    /// Message content
    pub content: MessageContent,
    /// Message metadata
    pub metadata: MessageMetadata,
    /// References to other messages
    pub references: Vec<String>,
    /// Message reactions/feedback
    pub reactions: Vec<MessageReaction>,
}

/// Type of message sender
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageSenderType {
    User,
    Agent,
    System,
}

/// Message content with different types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    /// Plain text message
    Text(String),
    /// Formatted text with markup
    Formatted { text: String, format: String },
    /// Code snippet
    Code { code: String, language: String },
    /// File attachment
    File { path: String, mime_type: String },
    /// Image content
    Image { url: String, alt_text: Option<String> },
    /// Artifact reference
    Artifact { artifact_id: String, description: String },
    /// Task result
    TaskResult { task_id: String, result: AgentResult },
    /// System notification
    SystemNotification { notification_type: String, data: serde_json::Value },
}

/// Message metadata for additional context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    /// Message importance level
    pub importance: MessageImportance,
    /// Whether message is editable
    pub editable: bool,
    /// Message edit history
    pub edit_history: Vec<MessageEdit>,
    /// Message thread information
    pub thread_info: Option<ThreadInfo>,
    /// Associated artifacts
    pub artifacts: Vec<String>,
    /// Custom metadata fields
    pub custom_fields: HashMap<String, serde_json::Value>,
}

/// Message importance levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageImportance {
    Low,
    Normal,
    High,
    Critical,
}

/// Message edit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEdit {
    /// Edit timestamp
    pub edited_at: DateTime<Utc>,
    /// Editor identifier
    pub edited_by: String,
    /// Edit reason/note
    pub reason: Option<String>,
    /// Previous content
    pub previous_content: MessageContent,
}

/// Thread information for message organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadInfo {
    /// Parent message ID
    pub parent_id: Option<String>,
    /// Thread depth
    pub depth: usize,
    /// Number of replies
    pub reply_count: usize,
}

/// Message reaction/feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReaction {
    /// Reaction type (emoji, vote, etc.)
    pub reaction_type: String,
    /// User who reacted
    pub user_id: String,
    /// Reaction timestamp
    pub timestamp: DateTime<Utc>,
    /// Optional reaction data
    pub data: Option<serde_json::Value>,
}

/// Session artifact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionArtifact {
    /// Artifact identifier
    pub id: String,
    /// Artifact name
    pub name: String,
    /// Artifact description
    pub description: Option<String>,
    /// Artifact type
    pub artifact_type: ArtifactType,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modification timestamp
    pub updated_at: DateTime<Utc>,
    /// Creator information
    pub creator: String, // User or Agent ID
    /// Artifact content or reference
    pub content: ArtifactContent,
    /// Artifact metadata
    pub metadata: ArtifactMetadata,
    /// Artifact dependencies
    pub dependencies: Vec<String>,
    /// Artifact version
    pub version: String,
    /// Artifact status
    pub status: ArtifactStatus,
}

/// Types of artifacts that can be created
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArtifactType {
    /// Source code file
    SourceCode,
    /// Configuration file
    Configuration,
    /// Documentation
    Documentation,
    /// Test file
    Test,
    /// Build script
    BuildScript,
    /// Database schema
    DatabaseSchema,
    /// API specification
    APISpecification,
    /// Design document
    Design,
    /// Custom artifact type
    Custom(String),
}

/// Artifact content representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactContent {
    /// Inline text content
    Text(String),
    /// File path reference
    File(PathBuf),
    /// Binary data (base64 encoded)
    Binary(String),
    /// External URL reference
    URL(String),
    /// Reference to another artifact
    Reference(String),
}

/// Artifact metadata and properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    /// Programming language
    pub language: Option<String>,
    /// Framework/technology
    pub framework: Option<String>,
    /// File size in bytes
    pub size: Option<u64>,
    /// File checksum
    pub checksum: Option<String>,
    /// License information
    pub license: Option<String>,
    /// Tags for organization
    pub tags: HashSet<String>,
    /// Quality metrics
    pub quality_metrics: Option<QualityMetrics>,
    /// Custom metadata fields
    pub custom_fields: HashMap<String, serde_json::Value>,
}

/// Code quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Lines of code
    pub lines_of_code: usize,
    /// Cyclomatic complexity
    pub complexity: Option<f64>,
    /// Test coverage percentage
    pub test_coverage: Option<f64>,
    /// Code quality score (0.0 to 1.0)
    pub quality_score: Option<f64>,
    /// Number of potential issues
    pub issues: usize,
}

/// Artifact status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArtifactStatus {
    Draft,
    InReview,
    Approved,
    Published,
    Deprecated,
    Archived,
}

/// Session branch for experimentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionBranch {
    /// Branch identifier
    pub id: String,
    /// Branch name
    pub name: String,
    /// Branch description
    pub description: Option<String>,
    /// Parent branch ID
    pub parent_id: Option<String>,
    /// Branch creation time
    pub created_at: DateTime<Utc>,
    /// Creator information
    pub creator: String,
    /// Branch point (snapshot ID)
    pub branch_point: String,
    /// Branch status
    pub status: BranchStatus,
    /// Merge information (if merged)
    pub merge_info: Option<MergeInfo>,
    /// Branch metadata
    pub metadata: HashMap<String, String>,
}

/// Branch status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BranchStatus {
    Active,
    Merged,
    Abandoned,
    Locked,
}

/// Merge information for branches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeInfo {
    /// Target branch ID
    pub target_branch: String,
    /// Merge timestamp
    pub merged_at: DateTime<Utc>,
    /// User who performed the merge
    pub merged_by: String,
    /// Merge strategy used
    pub strategy: MergeStrategy,
    /// Conflicts resolved during merge
    pub conflicts_resolved: Vec<ConflictResolution>,
}

/// Merge strategies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MergeStrategy {
    /// Fast-forward merge
    FastForward,
    /// Three-way merge
    ThreeWay,
    /// Squash and merge
    Squash,
    /// Cherry-pick merge
    CherryPick,
}

/// Conflict resolution information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    /// Conflict identifier
    pub conflict_id: String,
    /// Conflict type
    pub conflict_type: ConflictType,
    /// Resolution strategy
    pub resolution: ResolutionStrategy,
    /// Resolved by user
    pub resolved_by: String,
    /// Resolution timestamp
    pub resolved_at: DateTime<Utc>,
}

/// Types of merge conflicts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConflictType {
    /// Content conflict in artifacts
    ContentConflict,
    /// Metadata conflict
    MetadataConflict,
    /// Configuration conflict
    ConfigurationConflict,
    /// State conflict
    StateConflict,
}

/// Conflict resolution strategies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    /// Use source branch version
    AcceptSource,
    /// Use target branch version
    AcceptTarget,
    /// Manual merge
    ManualMerge,
    /// Use newer version
    UseNewer,
    /// Custom resolution
    Custom,
}


/// Individual collaborator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collaborator {
    /// User information
    pub user: SessionUser,
    /// Collaboration permissions
    pub permissions: CollaboratorPermissions,
    /// Join timestamp
    pub joined_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_active: DateTime<Utc>,
    /// Online status
    pub online: bool,
    /// Current cursor/focus position
    pub cursor_position: Option<CursorPosition>,
}

/// Permissions for individual collaborators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaboratorPermissions {
    /// Can read session content
    pub read: bool,
    /// Can modify session content
    pub write: bool,
    /// Can manage collaborators
    pub manage_collaborators: bool,
    /// Can manage session settings
    pub manage_settings: bool,
    /// Can create branches
    pub create_branches: bool,
    /// Can merge branches
    pub merge_branches: bool,
    /// Can export session
    pub export: bool,
}

/// Collaboration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSettings {
    /// Real-time editing enabled
    pub real_time_editing: bool,
    /// Show collaborator cursors
    pub show_cursors: bool,
    /// Enable voice chat
    pub voice_chat: bool,
    /// Enable video chat
    pub video_chat: bool,
    /// Notification settings
    pub notifications: CollaborationNotifications,
}

/// Collaboration notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationNotifications {
    /// Notify when users join/leave
    pub user_join_leave: bool,
    /// Notify on significant changes
    pub significant_changes: bool,
    /// Notify on mentions
    pub mentions: bool,
    /// Notify on comments
    pub comments: bool,
}

/// Real-time synchronization status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SyncStatus {
    /// Fully synchronized
    Synchronized,
    /// Synchronization in progress
    Syncing { progress: f64 },
    /// Synchronization conflict
    Conflict { conflicts: Vec<String> },
    /// Synchronization error
    Error { message: String },
    /// Offline mode
    Offline,
}

/// Cursor position for collaboration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    /// Current conversation thread
    pub conversation_id: Option<String>,
    /// Current message
    pub message_id: Option<String>,
    /// Current artifact
    pub artifact_id: Option<String>,
    /// Text position within artifact/message
    pub text_position: Option<TextPosition>,
}

/// Text position within a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPosition {
    /// Line number (0-indexed)
    pub line: u32,
    /// Column number (0-indexed)  
    pub column: u32,
    /// Selection range (if any)
    pub selection: Option<Box<TextRange>>,
}

/// Text selection range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRange {
    /// Start position
    pub start: Box<TextPosition>,
    /// End position
    pub end: Box<TextPosition>,
}

/// Comprehensive session manager
pub struct SessionManager {
    /// Active sessions in memory
    active_sessions: Arc<RwLock<HashMap<String, Session>>>,
    /// Session persistence layer
    persistence: Arc<dyn SessionPersistence + Send + Sync>,
    /// Recovery manager
    recovery_manager: RecoveryManager,
    /// Collaboration manager
    collaboration_manager: Option<CollaborationManager>,
    /// Analytics engine
    analytics: SessionAnalytics,
    /// Configuration manager
    config_manager: ConfigManager,
    /// Current user information
    current_user: SessionUser,
    /// Session manager configuration
    config: SessionManagerConfig,
    /// Background task handles
    background_tasks: tokio::task::JoinSet<()>,
}

/// Configuration for session manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionManagerConfig {
    /// Maximum number of active sessions
    pub max_active_sessions: usize,
    /// Session timeout in minutes
    pub default_session_timeout: u64,
    /// Auto-save interval in minutes
    pub auto_save_interval: u64,
    /// Checkpoint interval in minutes
    pub checkpoint_interval: u64,
    /// Maximum snapshots per session
    pub max_snapshots_per_session: usize,
    /// Enable collaboration features
    pub collaboration_enabled: bool,
    /// Storage backend configuration
    pub storage_backend: String,
    /// Compression settings
    pub compression: CompressionType,
    /// Encryption settings
    pub encryption_enabled: bool,
    pub encryption_key: Option<String>,
    /// Cleanup settings
    pub cleanup_enabled: bool,
    pub cleanup_interval: Duration,
    pub max_archived_sessions: usize,
}

impl Default for SessionManagerConfig {
    fn default() -> Self {
        Self {
            max_active_sessions: MAX_ACTIVE_SESSIONS,
            default_session_timeout: DEFAULT_SESSION_TIMEOUT,
            auto_save_interval: 5,
            checkpoint_interval: CHECKPOINT_INTERVAL,
            max_snapshots_per_session: MAX_SNAPSHOTS_PER_SESSION,
            collaboration_enabled: true,
            storage_backend: "filesystem".to_string(),
            compression: CompressionType::Gzip,
            encryption_enabled: false,
            encryption_key: None,
            cleanup_enabled: true,
            cleanup_interval: Duration::from_secs(3600), // 1 hour
            max_archived_sessions: 1000,
        }
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            timeout_minutes: DEFAULT_SESSION_TIMEOUT,
            auto_save: true,
            auto_save_interval: 5,
            max_conversation_history: 1000,
            max_artifacts: 500,
            collaboration_enabled: false,
            compression: CompressionType::Gzip,
            backup_enabled: true,
            backup_interval: Duration::from_secs(1800), // 30 minutes
            max_backups: 10,
            privacy_level: PrivacyLevel::Private,
            encryption_enabled: false,
        }
    }
}

impl Default for SessionMetadata {
    fn default() -> Self {
        Self {
            tags: HashSet::new(),
            project: None,
            priority: SessionPriority::Normal,
            repositories: Vec::new(),
            languages: HashSet::new(),
            technologies: HashSet::new(),
            objectives: Vec::new(),
            custom_fields: HashMap::new(),
        }
    }
}

impl Default for AgentSessionMetrics {
    fn default() -> Self {
        Self {
            tasks_completed: 0,
            tasks_failed: 0,
            total_processing_time: Duration::default(),
            average_completion_time: Duration::default(),
            artifacts_generated: 0,
            agent_interactions: 0,
            errors: 0,
            success_rate: 1.0,
        }
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            default_config: SessionConfig::default(),
            theme: "dark".to_string(),
            language: "en".to_string(),
            notifications: NotificationPreferences {
                session_status: true,
                agent_completion: true,
                collaboration: true,
                errors: true,
                delivery_methods: vec![NotificationMethod::InApp],
            },
            ui_preferences: HashMap::new(),
        }
    }
}

impl Default for CollaboratorPermissions {
    fn default() -> Self {
        Self {
            read: true,
            write: false,
            manage_collaborators: false,
            manage_settings: false,
            create_branches: false,
            merge_branches: false,
            export: false,
        }
    }
}

/// Session management errors
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Session not found: {session_id}")]
    SessionNotFound { session_id: String },

    #[error("Session access denied: {reason}")]
    AccessDenied { reason: String },

    #[error("Session is locked: {locked_by}")]
    SessionLocked { locked_by: String },

    #[error("Invalid session state: {message}")]
    InvalidState { message: String },

    #[error("Persistence error: {0}")]
    PersistenceError(String),

    #[error("Collaboration error: {0}")]
    CollaborationError(String),

    #[error("Recovery error: {0}")]
    RecoveryError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl From<Box<dyn std::error::Error + Send + Sync>> for SessionError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        SessionError::CollaborationError(err.to_string())
    }
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(
        persistence: Arc<dyn SessionPersistence + Send + Sync>,
        config_manager: ConfigManager,
        current_user: SessionUser,
        config: SessionManagerConfig,
    ) -> Result<Self, SessionError> {
        let recovery_manager = RecoveryManager::new(
            persistence.clone(),
            config.checkpoint_interval,
            config.max_snapshots_per_session,
        )?;

        let collaboration_manager = if config.collaboration_enabled {
            Some(CollaborationManager::new()?)
        } else {
            None
        };

        let analytics = SessionAnalytics::new();

        Ok(Self {
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            persistence,
            recovery_manager,
            collaboration_manager,
            analytics,
            config_manager,
            current_user,
            config,
            background_tasks: tokio::task::JoinSet::new(),
        })
    }

    /// Create a new session
    pub async fn create_session(&mut self, name: String, description: Option<String>) -> Result<String, SessionError> {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let session = Session {
            id: session_id.clone(),
            name,
            description,
            created_at: now,
            updated_at: now,
            accessed_at: now,
            creator: self.current_user.clone(),
            status: SessionStatus::Active,
            config: self.current_user.preferences.default_config.clone(),
            state: SessionState::new(),
            metadata: SessionMetadata::default(),
            agents: HashMap::new(),
            conversations: Vec::new(),
            artifacts: Vec::new(),
            branches: {
                let mut branches = HashMap::new();
                let main_branch = SessionBranch {
                    id: "main".to_string(),
                    name: "main".to_string(),
                    description: Some("Main branch".to_string()),
                    parent_id: None,
                    created_at: now,
                    creator: self.current_user.id.clone(),
                    branch_point: "initial".to_string(),
                    status: BranchStatus::Active,
                    merge_info: None,
                    metadata: HashMap::new(),
                };
                branches.insert("main".to_string(), main_branch);
                branches
            },
            active_branch: "main".to_string(),
            collaboration: None,
            analytics: SessionAnalytics::new(),
            variables: HashMap::new(),
        };

        // Add to active sessions
        {
            let mut active_sessions = self.active_sessions.write().await;
            active_sessions.insert(session_id.clone(), session.clone());
        }

        // Persist session
        self.persistence.save_session(&session).await
            .map_err(|e| SessionError::PersistenceError(e.to_string()))?;

        // Create initial checkpoint
        self.recovery_manager.create_checkpoint(&session_id, &session).await
            .map_err(|e| SessionError::RecoveryError(e.to_string()))?;

        // Start background tasks for this session
        self.start_session_tasks(&session_id).await?;

        info!("Created new session: {} ({})", session.name, session_id);
        Ok(session_id)
    }

    /// Load an existing session
    pub async fn load_session(&mut self, session_id: &str) -> Result<Session, SessionError> {
        // Check if already in memory
        {
            let active_sessions = self.active_sessions.read().await;
            if let Some(session) = active_sessions.get(session_id) {
                let mut session = session.clone();
                session.accessed_at = Utc::now();
                return Ok(session);
            }
        }

        // Load from persistence
        let mut session = self.persistence.load_session(session_id).await
            .map_err(|e| SessionError::PersistenceError(e.to_string()))?
            .ok_or_else(|| SessionError::SessionNotFound { session_id: session_id.to_string() })?;

        session.accessed_at = Utc::now();

        // Add to active sessions
        {
            let mut active_sessions = self.active_sessions.write().await;
            
            // Check if we need to evict old sessions
            if active_sessions.len() >= self.config.max_active_sessions {
                self.evict_least_recently_used(&mut active_sessions).await?;
            }
            
            active_sessions.insert(session_id.to_string(), session.clone());
        }

        // Start background tasks
        self.start_session_tasks(session_id).await?;

        info!("Loaded session: {} ({})", session.name, session_id);
        Ok(session)
    }

    /// Save a session
    pub async fn save_session(&self, session: &Session) -> Result<(), SessionError> {
        // Update in memory
        {
            let mut active_sessions = self.active_sessions.write().await;
            active_sessions.insert(session.id.clone(), session.clone());
        }

        // Persist to storage
        self.persistence.save_session(session).await
            .map_err(|e| SessionError::PersistenceError(e.to_string()))?;

        debug!("Saved session: {}", session.id);
        Ok(())
    }

    /// Delete a session
    pub async fn delete_session(&mut self, session_id: &str) -> Result<(), SessionError> {
        // Remove from memory
        {
            let mut active_sessions = self.active_sessions.write().await;
            active_sessions.remove(session_id);
        }

        // Delete from persistence
        self.persistence.delete_session(session_id).await
            .map_err(|e| SessionError::PersistenceError(e.to_string()))?;

        // Clean up recovery data
        self.recovery_manager.cleanup_session(session_id).await
            .map_err(|e| SessionError::RecoveryError(e.to_string()))?;

        info!("Deleted session: {}", session_id);
        Ok(())
    }

    /// List all sessions for current user
    pub async fn list_sessions(&self) -> Result<Vec<Session>, SessionError> {
        self.persistence.list_sessions(&self.current_user.id).await
            .map_err(|e| SessionError::PersistenceError(e.to_string()))
    }

    /// Search sessions by criteria
    pub async fn search_sessions(&self, query: &str, filters: SessionFilters) -> Result<Vec<Session>, SessionError> {
        self.persistence.search_sessions(&self.current_user.id, query, filters).await
            .map_err(|e| SessionError::PersistenceError(e.to_string()))
    }

    /// Create a session branch
    pub async fn create_branch(
        &mut self,
        session_id: &str,
        branch_name: String,
        description: Option<String>,
        from_branch: Option<String>,
    ) -> Result<String, SessionError> {
        let mut session = self.load_session(session_id).await?;
        
        let branch_id = Uuid::new_v4().to_string();
        let parent_branch = from_branch.unwrap_or_else(|| session.active_branch.clone());
        
        // Create snapshot for branch point
        let snapshot_id = self.recovery_manager.create_checkpoint(session_id, &session).await
            .map_err(|e| SessionError::RecoveryError(e.to_string()))?;
        
        let branch = SessionBranch {
            id: branch_id.clone(),
            name: branch_name.clone(),
            description,
            parent_id: Some(parent_branch),
            created_at: Utc::now(),
            creator: self.current_user.id.clone(),
            branch_point: snapshot_id,
            status: BranchStatus::Active,
            merge_info: None,
            metadata: HashMap::new(),
        };

        session.branches.insert(branch_id.clone(), branch);
        session.updated_at = Utc::now();

        self.save_session(&session).await?;

        info!("Created branch '{}' for session {}", branch_name, session_id);
        Ok(branch_id)
    }

    /// Switch to a different branch
    pub async fn switch_branch(&mut self, session_id: &str, branch_id: &str) -> Result<(), SessionError> {
        let mut session = self.load_session(session_id).await?;
        
        if !session.branches.contains_key(branch_id) {
            return Err(SessionError::ValidationError(
                format!("Branch '{}' does not exist", branch_id)
            ));
        }

        session.active_branch = branch_id.to_string();
        session.updated_at = Utc::now();

        self.save_session(&session).await?;

        info!("Switched to branch '{}' for session {}", branch_id, session_id);
        Ok(())
    }

    /// Get session analytics
    pub async fn get_analytics(&mut self, session_id: &str) -> Result<SessionAnalytics, SessionError> {
        let session = self.load_session(session_id).await?;
        Ok(session.analytics.clone())
    }

    /// Enable collaboration for a session
    pub async fn enable_collaboration(
        &mut self,
        session_id: &str,
        permissions: SharePermissions,
    ) -> Result<(), SessionError> {
        let mut session = self.load_session(session_id).await?;
        
        if let Some(collab_manager) = &mut self.collaboration_manager {
            let share_info = collab_manager.create_share(&session.id, permissions).await
                .map_err(|e| SessionError::CollaborationError(e.to_string()))?;
            
            session.collaboration = Some(CollaborationInfo {
                is_shared: true,
                shares: vec![share_info],
                active_collaborators: HashSet::new(),
                owner: self.current_user.id.clone(),
            });
            
            session.updated_at = Utc::now();
            self.save_session(&session).await?;
            
            info!("Enabled collaboration for session {}", session_id);
        } else {
            return Err(SessionError::CollaborationError(
                "Collaboration is not enabled".to_string()
            ));
        }

        Ok(())
    }

    /// Add agent to session
    pub async fn add_agent(
        &mut self,
        session_id: &str,
        agent_id: String,
        agent_name: String,
        agent_type: String,
        behavior_profile: Option<String>,
    ) -> Result<(), SessionError> {
        let mut session = self.load_session(session_id).await?;
        
        let agent_info = AgentSessionInfo {
            id: agent_id.clone(),
            name: agent_name,
            agent_type,
            status: AgentStatus::Idle,
            current_task: None,
            config: serde_json::Value::Object(serde_json::Map::new()),
            behavior_profile,
            started_at: Utc::now(),
            metrics: AgentSessionMetrics::default(),
            working_directory: None,
            variables: HashMap::new(),
        };

        session.agents.insert(agent_id.clone(), agent_info);
        session.updated_at = Utc::now();

        self.save_session(&session).await?;

        info!("Added agent '{}' to session {}", agent_id, session_id);
        Ok(())
    }

    /// Update agent status in session
    pub async fn update_agent_status(
        &mut self,
        session_id: &str,
        agent_id: &str,
        status: AgentStatus,
        current_task: Option<TaskInfo>,
    ) -> Result<(), SessionError> {
        let mut session = self.load_session(session_id).await?;
        
        if let Some(agent_info) = session.agents.get_mut(agent_id) {
            agent_info.status = status;
            agent_info.current_task = current_task;
            session.updated_at = Utc::now();

            self.save_session(&session).await?;
            
            debug!("Updated agent '{}' status in session {}", agent_id, session_id);
        } else {
            return Err(SessionError::ValidationError(
                format!("Agent '{}' not found in session", agent_id)
            ));
        }

        Ok(())
    }

    /// Add artifact to session
    pub async fn add_artifact(
        &mut self,
        session_id: &str,
        artifact: SessionArtifact,
    ) -> Result<(), SessionError> {
        let mut session = self.load_session(session_id).await?;
        
        // Check artifact limit
        if session.artifacts.len() >= session.config.max_artifacts {
            // Remove oldest artifact if at limit
            session.artifacts.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            session.artifacts.remove(0);
        }

        session.artifacts.push(artifact);
        session.updated_at = Utc::now();

        self.save_session(&session).await?;

        debug!("Added artifact to session {}", session_id);
        Ok(())
    }

    /// Start background tasks for a session
    async fn start_session_tasks(&mut self, session_id: &str) -> Result<(), SessionError> {
        let session_id = session_id.to_string();
        let active_sessions = Arc::clone(&self.active_sessions);
        let persistence = Arc::clone(&self.persistence);
        let auto_save_interval = self.config.auto_save_interval;

        // Auto-save task - create as async closure that returns ()
        let session_id_clone = session_id.clone();
        let active_sessions_clone = Arc::clone(&active_sessions);
        let persistence_clone = Arc::clone(&persistence);
        
        let auto_save_future = async move {
            let mut interval = tokio::time::interval(Duration::from_secs(auto_save_interval * 60));
            loop {
                interval.tick().await;
                
                // Clone the session data to avoid holding the lock across await points
                let session_data = {
                    let sessions = active_sessions_clone.read().await;
                    if let Some(session) = sessions.get(&session_id_clone) {
                        if session.config.auto_save {
                            Some(session.clone())
                        } else {
                            None
                        }
                    } else {
                        // Session no longer active, exit task
                        return;
                    }
                };
                
                // Save the session without holding any locks
                if let Some(session) = session_data {
                    if let Err(e) = persistence_clone.save_session(&session).await {
                        error!("Auto-save failed for session {}: {}", session_id_clone, e);
                    }
                }
            }
        };

        self.background_tasks.spawn(auto_save_future);
        Ok(())
    }

    /// Evict least recently used session from memory
    async fn evict_least_recently_used(
        &self,
        active_sessions: &mut HashMap<String, Session>,
    ) -> Result<(), SessionError> {
        if let Some((lru_id, _)) = active_sessions
            .iter()
            .min_by_key(|(_, session)| session.accessed_at)
            .map(|(id, session)| (id.clone(), session.clone()))
        {
            let session = active_sessions.remove(&lru_id).unwrap();
            
            // Save before evicting
            self.persistence.save_session(&session).await
                .map_err(|e| SessionError::PersistenceError(e.to_string()))?;
                
            debug!("Evicted session from memory: {}", lru_id);
        }

        Ok(())
    }
}

/// Session search filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionFilters {
    /// Filter by session status
    pub status: Option<SessionStatus>,
    /// Filter by tags
    pub tags: Option<Vec<String>>,
    /// Filter by project
    pub project: Option<String>,
    /// Filter by priority
    pub priority: Option<SessionPriority>,
    /// Filter by date range
    pub date_range: Option<DateRange>,
    /// Filter by collaborators
    pub collaborators: Option<Vec<String>>,
    /// Filter by programming languages
    pub languages: Option<Vec<String>>,
}

/// Date range filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl Default for SessionFilters {
    fn default() -> Self {
        Self {
            status: None,
            tags: None,
            project: None,
            priority: None,
            date_range: None,
            collaborators: None,
            languages: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_session_creation() {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let user = SessionUser {
            id: "test-user".to_string(),
            name: "Test User".to_string(),
            email: Some("test@example.com".to_string()),
            avatar_url: None,
            timezone: "UTC".to_string(),
            preferences: UserPreferences::default(),
        };

        let session = Session {
            id: session_id.clone(),
            name: "Test Session".to_string(),
            description: Some("A test session".to_string()),
            created_at: now,
            updated_at: now,
            accessed_at: now,
            creator: user,
            status: SessionStatus::Active,
            config: SessionConfig::default(),
            state: SessionState::new(),
            metadata: SessionMetadata::default(),
            agents: HashMap::new(),
            conversations: Vec::new(),
            artifacts: Vec::new(),
            branches: HashMap::new(),
            active_branch: "main".to_string(),
            collaboration: None,
            analytics: SessionAnalytics::new(),
            variables: HashMap::new(),
        };

        assert_eq!(session.id, session_id);
        assert_eq!(session.name, "Test Session");
        assert_eq!(session.status, SessionStatus::Active);
    }

    #[test]
    fn test_session_config_defaults() {
        let config = SessionConfig::default();
        
        assert_eq!(config.timeout_minutes, DEFAULT_SESSION_TIMEOUT);
        assert!(config.auto_save);
        assert_eq!(config.auto_save_interval, 5);
        assert_eq!(config.max_conversation_history, 1000);
        assert_eq!(config.max_artifacts, 500);
        assert!(!config.collaboration_enabled);
        assert_eq!(config.privacy_level, PrivacyLevel::Private);
    }

    #[test]
    fn test_artifact_metadata() {
        let mut metadata = ArtifactMetadata {
            language: Some("rust".to_string()),
            framework: Some("tokio".to_string()),
            size: Some(1024),
            checksum: Some("abc123".to_string()),
            license: Some("MIT".to_string()),
            tags: HashSet::new(),
            quality_metrics: Some(QualityMetrics {
                lines_of_code: 100,
                complexity: Some(5.2),
                test_coverage: Some(85.5),
                quality_score: Some(0.9),
                issues: 2,
            }),
            custom_fields: HashMap::new(),
        };

        metadata.tags.insert("backend".to_string());
        metadata.tags.insert("async".to_string());

        assert_eq!(metadata.language, Some("rust".to_string()));
        assert_eq!(metadata.tags.len(), 2);
        assert!(metadata.tags.contains("backend"));
        assert!(metadata.quality_metrics.is_some());
    }
}