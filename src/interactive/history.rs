//! Conversation History Management System
//!
//! This module provides comprehensive conversation history management with persistence,
//! search, filtering, and retrieval capabilities for interactive DevKit sessions.

use crate::agents::{AgentInfo, AgentResult, TaskPriority};
use crate::artifacts::EnhancedArtifact;
use crate::error::DevKitError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

/// Maximum number of conversations to keep in memory
const MAX_MEMORY_CONVERSATIONS: usize = 100;
/// Maximum number of messages per conversation in memory
const MAX_MEMORY_MESSAGES: usize = 1000;

/// A complete conversation session with metadata and messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSession {
    /// Unique session identifier
    pub id: String,
    /// Session title (auto-generated or user-set)
    pub title: String,
    /// Project path associated with this conversation
    pub project_path: Option<PathBuf>,
    /// When the conversation started
    pub started_at: DateTime<Utc>,
    /// When the conversation was last updated
    pub updated_at: DateTime<Utc>,
    /// All messages in this conversation
    pub messages: Vec<ConversationMessage>,
    /// Agents that participated in this conversation
    pub participating_agents: Vec<String>,
    /// Artifacts created during this conversation
    pub created_artifacts: Vec<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Session metadata
    pub metadata: ConversationMetadata,
    /// Whether this conversation is bookmarked
    pub bookmarked: bool,
    /// Session statistics
    pub stats: ConversationStats,
}

/// A single message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    /// Message ID
    pub id: String,
    /// Message timestamp
    pub timestamp: DateTime<Utc>,
    /// Message type and source
    pub message_type: MessageType,
    /// The actual message content
    pub content: String,
    /// Associated artifacts (if any)
    pub artifacts: Vec<String>,
    /// Agent that generated this message (if applicable)
    pub agent_id: Option<String>,
    /// Task priority when this message was sent
    pub priority: Option<TaskPriority>,
    /// Message metadata
    pub metadata: MessageMetadata,
}

/// Types of messages in a conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    /// User input/command
    UserInput,
    /// Agent response
    AgentResponse,
    /// System message
    System,
    /// Error message
    Error,
    /// Artifact creation notice
    ArtifactCreated,
    /// Task status update
    TaskUpdate,
    /// Debug information
    Debug,
    /// Warning message
    Warning,
}

/// Metadata for conversation sessions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConversationMetadata {
    /// Programming languages discussed
    pub languages: Vec<String>,
    /// Topics covered
    pub topics: Vec<String>,
    /// Commands executed
    pub commands: Vec<String>,
    /// Files modified
    pub files_modified: Vec<String>,
    /// Session quality rating (0-5)
    pub quality_rating: Option<u8>,
    /// User notes
    pub notes: Option<String>,
}

/// Metadata for individual messages
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageMetadata {
    /// Processing time for this message
    pub processing_time_ms: Option<u64>,
    /// Token count (for LLM messages)
    pub token_count: Option<usize>,
    /// Cost associated with this message
    pub cost: Option<f64>,
    /// Message confidence score
    pub confidence: Option<f64>,
    /// Related file paths
    pub file_paths: Vec<String>,
}

/// Statistics for a conversation session
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConversationStats {
    /// Total message count
    pub message_count: usize,
    /// User messages
    pub user_messages: usize,
    /// Agent responses
    pub agent_responses: usize,
    /// Total artifacts created
    pub artifacts_created: usize,
    /// Total processing time
    pub total_processing_time_ms: u64,
    /// Session duration
    pub session_duration_seconds: u64,
    /// Average response time
    pub avg_response_time_ms: f64,
}

/// Search criteria for conversations and messages
#[derive(Debug, Clone)]
pub struct HistorySearchCriteria {
    /// Text to search for in content
    pub query: Option<String>,
    /// Date range filter
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    /// Message types to include
    pub message_types: Vec<MessageType>,
    /// Specific agents to filter by
    pub agents: Vec<String>,
    /// Tags to filter by
    pub tags: Vec<String>,
    /// Project path filter
    pub project_path: Option<PathBuf>,
    /// Bookmarked only
    pub bookmarked_only: bool,
    /// Sort order
    pub sort_by: HistorySortCriteria,
    /// Maximum results
    pub limit: Option<usize>,
    /// Include message content in results
    pub include_content: bool,
}

/// Sort criteria for history search results
#[derive(Debug, Clone, PartialEq)]
pub enum HistorySortCriteria {
    /// Sort by recency (newest first)
    Recent,
    /// Sort by relevance (most relevant first)
    Relevance,
    /// Sort by message count
    MessageCount,
    /// Sort by session duration
    Duration,
    /// Sort by artifact count
    ArtifactCount,
    /// Sort alphabetically by title
    Title,
}

/// Search result for conversation history
#[derive(Debug, Clone)]
pub struct HistorySearchResult {
    /// Matching conversation session
    pub session: ConversationSession,
    /// Matching messages (if content was requested)
    pub matching_messages: Vec<ConversationMessage>,
    /// Relevance score (0.0 to 1.0)
    pub relevance_score: f64,
    /// Matched terms
    pub matched_terms: Vec<String>,
}

/// Auto-completion suggestion
#[derive(Debug, Clone)]
pub struct CompletionSuggestion {
    /// The suggested text
    pub text: String,
    /// Type of suggestion
    pub suggestion_type: SuggestionType,
    /// Relevance score
    pub score: f64,
    /// Description of the suggestion
    pub description: Option<String>,
    /// Context where this suggestion applies
    pub context: Option<String>,
}

/// Types of auto-completion suggestions
#[derive(Debug, Clone, PartialEq)]
pub enum SuggestionType {
    /// Command suggestion
    Command,
    /// File path
    FilePath,
    /// Agent name
    Agent,
    /// Artifact ID
    Artifact,
    /// Conversation topic
    Topic,
    /// Previous query
    PreviousQuery,
    /// Code snippet
    CodeSnippet,
    /// Tag
    Tag,
}

/// Conversation history manager
pub struct ConversationHistoryManager {
    /// Storage directory for conversations
    storage_path: PathBuf,
    /// In-memory cache of recent conversations
    memory_cache: RwLock<VecDeque<ConversationSession>>,
    /// Index for fast searching
    search_index: RwLock<HashMap<String, Vec<String>>>, // term -> conversation IDs
    /// Auto-completion cache
    completion_cache: RwLock<HashMap<SuggestionType, Vec<CompletionSuggestion>>>,
    /// Current active conversation
    active_conversation: RwLock<Option<String>>,
    /// Manager statistics
    stats: RwLock<HistoryManagerStats>,
}

/// Statistics for the history manager
#[derive(Debug, Default)]
pub struct HistoryManagerStats {
    pub total_conversations: usize,
    pub total_messages: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub search_queries: u64,
    pub completion_requests: u64,
}

impl ConversationHistoryManager {
    /// Create a new conversation history manager
    pub fn new(storage_path: PathBuf) -> Result<Self, DevKitError> {
        // Ensure storage directory exists
        if !storage_path.exists() {
            fs::create_dir_all(&storage_path).map_err(|e| {
                DevKitError::ContextualError {
                    source: Box::new(e),
                    context: "Failed to create history storage directory".to_string(),
                }
            })?;
        }

        let manager = Self {
            storage_path,
            memory_cache: RwLock::new(VecDeque::new()),
            search_index: RwLock::new(HashMap::new()),
            completion_cache: RwLock::new(HashMap::new()),
            active_conversation: RwLock::new(None),
            stats: RwLock::new(HistoryManagerStats::default()),
        };

        info!("Initialized conversation history manager at {:?}", manager.storage_path);
        Ok(manager)
    }

    /// Start a new conversation session
    pub async fn start_conversation(
        &self,
        title: Option<String>,
        project_path: Option<PathBuf>,
    ) -> Result<String, DevKitError> {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let title = title.unwrap_or_else(|| {
            format!("Conversation {}", now.format("%Y-%m-%d %H:%M"))
        });

        let session = ConversationSession {
            id: session_id.clone(),
            title,
            project_path,
            started_at: now,
            updated_at: now,
            messages: Vec::new(),
            participating_agents: Vec::new(),
            created_artifacts: Vec::new(),
            tags: Vec::new(),
            metadata: ConversationMetadata::default(),
            bookmarked: false,
            stats: ConversationStats::default(),
        };

        // Set as active conversation
        {
            let mut active = self.active_conversation.write().await;
            *active = Some(session_id.clone());
        }

        // Add to memory cache
        {
            let mut cache = self.memory_cache.write().await;
            cache.push_front(session.clone());
            
            // Maintain cache size limit
            if cache.len() > MAX_MEMORY_CONVERSATIONS {
                if let Some(old_session) = cache.pop_back() {
                    // Save to disk before removing from cache
                    if let Err(e) = self.save_conversation_to_disk(&old_session).await {
                        warn!("Failed to save conversation to disk: {}", e);
                    }
                }
            }
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_conversations += 1;
        }

        info!("Started new conversation: {} ({})", session.title, session_id);
        Ok(session_id)
    }

    /// Add a message to the active conversation
    pub async fn add_message(
        &self,
        content: String,
        message_type: MessageType,
        agent_id: Option<String>,
        artifacts: Vec<String>,
        metadata: Option<MessageMetadata>,
    ) -> Result<String, DevKitError> {
        let message_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let message = ConversationMessage {
            id: message_id.clone(),
            timestamp: now,
            message_type: message_type.clone(),
            content: content.clone(),
            artifacts: artifacts.clone(),
            agent_id: agent_id.clone(),
            priority: None,
            metadata: metadata.unwrap_or_default(),
        };

        // Get active conversation ID
        let conversation_id = {
            let active = self.active_conversation.read().await;
            active.clone().ok_or_else(|| {
                DevKitError::ValidationError {
                    field: "conversation".to_string(),
                    message: "No active conversation".to_string(),
                }
            })?
        };

        // Update conversation in cache
        {
            let mut cache = self.memory_cache.write().await;
            if let Some(conversation) = cache.iter_mut().find(|c| c.id == conversation_id) {
                conversation.messages.push(message.clone());
                conversation.updated_at = now;
                
                // Update participating agents
                if let Some(ref agent) = agent_id {
                    if !conversation.participating_agents.contains(agent) {
                        conversation.participating_agents.push(agent.clone());
                    }
                }
                
                // Update created artifacts
                for artifact in &artifacts {
                    if !conversation.created_artifacts.contains(artifact) {
                        conversation.created_artifacts.push(artifact.clone());
                    }
                }
                
                // Update stats
                conversation.stats.message_count += 1;
                match message_type {
                    MessageType::UserInput => conversation.stats.user_messages += 1,
                    MessageType::AgentResponse => conversation.stats.agent_responses += 1,
                    MessageType::ArtifactCreated => conversation.stats.artifacts_created += 1,
                    _ => {}
                }
                
                if let Some(processing_time) = message.metadata.processing_time_ms {
                    conversation.stats.total_processing_time_ms += processing_time;
                    conversation.stats.avg_response_time_ms = 
                        conversation.stats.total_processing_time_ms as f64 / conversation.stats.message_count as f64;
                }

                // Maintain message limit per conversation
                if conversation.messages.len() > MAX_MEMORY_MESSAGES {
                    conversation.messages.remove(0);
                }
            }
        }

        // Update search index
        self.update_search_index(&conversation_id, &content).await;
        
        // Update completion cache
        self.update_completion_cache(&message).await;

        // Update global stats
        {
            let mut stats = self.stats.write().await;
            stats.total_messages += 1;
        }

        trace!("Added message {} to conversation {}", message_id, conversation_id);
        Ok(message_id)
    }

    /// Search conversations and messages
    pub async fn search(
        &self,
        criteria: HistorySearchCriteria,
    ) -> Result<Vec<HistorySearchResult>, DevKitError> {
        let mut results = Vec::new();
        
        // Update search query stats
        {
            let mut stats = self.stats.write().await;
            stats.search_queries += 1;
        }

        // Search in memory cache first
        {
            let cache = self.memory_cache.read().await;
            for conversation in cache.iter() {
                if let Some(result) = self.match_conversation(conversation, &criteria).await {
                    results.push(result);
                }
            }
        }

        // Search in disk storage if needed
        if results.len() < criteria.limit.unwrap_or(usize::MAX) {
            let disk_results = self.search_disk_conversations(&criteria).await?;
            results.extend(disk_results);
        }

        // Sort results
        self.sort_search_results(&mut results, &criteria.sort_by);

        // Apply limit
        if let Some(limit) = criteria.limit {
            results.truncate(limit);
        }

        debug!("Found {} conversations matching search criteria", results.len());
        Ok(results)
    }

    /// Get auto-completion suggestions
    pub async fn get_completions(
        &self,
        partial_input: &str,
        context: Option<&str>,
        suggestion_types: Vec<SuggestionType>,
    ) -> Result<Vec<CompletionSuggestion>, DevKitError> {
        let mut suggestions = Vec::new();
        
        // Update completion request stats
        {
            let mut stats = self.stats.write().await;
            stats.completion_requests += 1;
        }

        for suggestion_type in suggestion_types {
            let type_suggestions = self.generate_completions_for_type(
                partial_input,
                context,
                &suggestion_type,
            ).await?;
            suggestions.extend(type_suggestions);
        }

        // Sort by relevance score
        suggestions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Limit results
        suggestions.truncate(50);

        debug!("Generated {} completion suggestions for '{}'", suggestions.len(), partial_input);
        Ok(suggestions)
    }

    /// Get conversation by ID
    pub async fn get_conversation(&self, conversation_id: &str) -> Result<Option<ConversationSession>, DevKitError> {
        // Check memory cache first
        {
            let cache = self.memory_cache.read().await;
            if let Some(conversation) = cache.iter().find(|c| c.id == conversation_id) {
                let mut stats = self.stats.write().await;
                stats.cache_hits += 1;
                return Ok(Some(conversation.clone()));
            }
        }

        // Load from disk
        match self.load_conversation_from_disk(conversation_id).await {
            Ok(Some(conversation)) => {
                let mut stats = self.stats.write().await;
                stats.cache_misses += 1;
                Ok(Some(conversation))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                warn!("Failed to load conversation from disk: {}", e);
                Ok(None)
            }
        }
    }

    /// Bookmark a conversation
    pub async fn bookmark_conversation(&self, conversation_id: &str, bookmarked: bool) -> Result<(), DevKitError> {
        // Update in memory cache
        {
            let mut cache = self.memory_cache.write().await;
            if let Some(conversation) = cache.iter_mut().find(|c| c.id == conversation_id) {
                conversation.bookmarked = bookmarked;
                return Ok(());
            }
        }

        // Load from disk, update, and save back
        if let Some(mut conversation) = self.load_conversation_from_disk(conversation_id).await? {
            conversation.bookmarked = bookmarked;
            self.save_conversation_to_disk(&conversation).await?;
        }

        Ok(())
    }

    /// Add tags to a conversation
    pub async fn tag_conversation(&self, conversation_id: &str, tags: Vec<String>) -> Result<(), DevKitError> {
        // Update in memory cache
        {
            let mut cache = self.memory_cache.write().await;
            if let Some(conversation) = cache.iter_mut().find(|c| c.id == conversation_id) {
                for tag in tags {
                    if !conversation.tags.contains(&tag) {
                        conversation.tags.push(tag);
                    }
                }
                return Ok(());
            }
        }

        // Load from disk, update, and save back
        if let Some(mut conversation) = self.load_conversation_from_disk(conversation_id).await? {
            for tag in tags {
                if !conversation.tags.contains(&tag) {
                    conversation.tags.push(tag);
                }
            }
            self.save_conversation_to_disk(&conversation).await?;
        }

        Ok(())
    }

    /// Get recent conversations
    pub async fn get_recent_conversations(&self, limit: usize) -> Result<Vec<ConversationSession>, DevKitError> {
        let cache = self.memory_cache.read().await;
        let recent = cache.iter()
            .take(limit)
            .cloned()
            .collect();
        Ok(recent)
    }

    /// Get conversation statistics
    pub async fn get_stats(&self) -> HistoryManagerStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    // Private helper methods

    async fn match_conversation(
        &self,
        conversation: &ConversationSession,
        criteria: &HistorySearchCriteria,
    ) -> Option<HistorySearchResult> {
        let mut relevance_score = 0.0;
        let mut matched_terms = Vec::new();
        let mut matching_messages = Vec::new();

        // Date range filter
        if let Some((start, end)) = &criteria.date_range {
            if conversation.started_at < *start || conversation.started_at > *end {
                return None;
            }
        }

        // Project path filter
        if let Some(ref project_path) = criteria.project_path {
            if conversation.project_path.as_ref() != Some(project_path) {
                return None;
            }
        }

        // Bookmarked filter
        if criteria.bookmarked_only && !conversation.bookmarked {
            return None;
        }

        // Agent filter
        if !criteria.agents.is_empty() {
            let has_matching_agent = criteria.agents.iter()
                .any(|agent| conversation.participating_agents.contains(agent));
            if !has_matching_agent {
                return None;
            }
        }

        // Tag filter
        if !criteria.tags.is_empty() {
            let has_matching_tag = criteria.tags.iter()
                .any(|tag| conversation.tags.contains(tag));
            if !has_matching_tag {
                return None;
            }
        }

        // Text search
        if let Some(ref query) = criteria.query {
            let query_lower = query.to_lowercase();
            
            // Search in title
            if conversation.title.to_lowercase().contains(&query_lower) {
                relevance_score += 2.0;
                matched_terms.push("title".to_string());
            }

            // Search in messages
            for message in &conversation.messages {
                if criteria.message_types.is_empty() || criteria.message_types.contains(&message.message_type) {
                    if message.content.to_lowercase().contains(&query_lower) {
                        relevance_score += 1.0;
                        matched_terms.push(format!("message_{}", message.id));
                        if criteria.include_content {
                            matching_messages.push(message.clone());
                        }
                    }
                }
            }

            // If we have a query but no matches, exclude this conversation
            if matched_terms.is_empty() {
                return None;
            }
        }

        // Base relevance for matching conversations
        if relevance_score == 0.0 {
            relevance_score = 0.5;
        }

        // Normalize relevance score
        relevance_score = (relevance_score / (conversation.messages.len() as f64 + 1.0)).min(1.0);

        Some(HistorySearchResult {
            session: conversation.clone(),
            matching_messages,
            relevance_score,
            matched_terms,
        })
    }

    async fn search_disk_conversations(
        &self,
        criteria: &HistorySearchCriteria,
    ) -> Result<Vec<HistorySearchResult>, DevKitError> {
        let mut results = Vec::new();
        
        // This is a simplified implementation
        // In practice, you'd want to maintain an index on disk for faster searching
        
        if let Ok(entries) = fs::read_dir(&self.storage_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        if let Ok(conversation) = self.load_conversation_from_path(&path).await {
                            if let Some(result) = self.match_conversation(&conversation, criteria).await {
                                results.push(result);
                            }
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    fn sort_search_results(&self, results: &mut Vec<HistorySearchResult>, sort_by: &HistorySortCriteria) {
        match sort_by {
            HistorySortCriteria::Recent => {
                results.sort_by(|a, b| b.session.updated_at.cmp(&a.session.updated_at));
            }
            HistorySortCriteria::Relevance => {
                results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));
            }
            HistorySortCriteria::MessageCount => {
                results.sort_by(|a, b| b.session.stats.message_count.cmp(&a.session.stats.message_count));
            }
            HistorySortCriteria::Duration => {
                results.sort_by(|a, b| b.session.stats.session_duration_seconds.cmp(&a.session.stats.session_duration_seconds));
            }
            HistorySortCriteria::ArtifactCount => {
                results.sort_by(|a, b| b.session.created_artifacts.len().cmp(&a.session.created_artifacts.len()));
            }
            HistorySortCriteria::Title => {
                results.sort_by(|a, b| a.session.title.cmp(&b.session.title));
            }
        }
    }

    async fn generate_completions_for_type(
        &self,
        partial_input: &str,
        context: Option<&str>,
        suggestion_type: &SuggestionType,
    ) -> Result<Vec<CompletionSuggestion>, DevKitError> {
        let mut suggestions = Vec::new();
        let partial_lower = partial_input.to_lowercase();

        match suggestion_type {
            SuggestionType::Command => {
                // Common DevKit commands
                let commands = vec![
                    ("init", "Initialize a new project"),
                    ("start", "Start the development environment"),
                    ("analyze", "Analyze the codebase"),
                    ("generate", "Generate code using AI"),
                    ("search", "Search conversations and artifacts"),
                    ("help", "Show help information"),
                    ("exit", "Exit the DevKit"),
                    ("bookmark", "Bookmark current conversation"),
                    ("tag", "Add tags to conversation"),
                ];

                for (cmd, desc) in commands {
                    if cmd.starts_with(&partial_lower) {
                        suggestions.push(CompletionSuggestion {
                            text: cmd.to_string(),
                            suggestion_type: SuggestionType::Command,
                            score: self.calculate_completion_score(partial_input, cmd),
                            description: Some(desc.to_string()),
                            context: context.map(|s| s.to_string()),
                        });
                    }
                }
            }

            SuggestionType::FilePath => {
                // Get file paths from recent conversations
                let cache = self.memory_cache.read().await;
                let mut file_paths = std::collections::HashSet::new();
                
                for conversation in cache.iter() {
                    for message in &conversation.messages {
                        file_paths.extend(message.metadata.file_paths.iter().cloned());
                    }
                    file_paths.extend(conversation.metadata.files_modified.iter().cloned());
                }

                for path in file_paths {
                    if path.to_lowercase().contains(&partial_lower) {
                        suggestions.push(CompletionSuggestion {
                            text: path.clone(),
                            suggestion_type: SuggestionType::FilePath,
                            score: self.calculate_completion_score(partial_input, &path),
                            description: Some("Recently accessed file".to_string()),
                            context: context.map(|s| s.to_string()),
                        });
                    }
                }
            }

            SuggestionType::Agent => {
                // Get agent names from conversations
                let cache = self.memory_cache.read().await;
                let mut agents = std::collections::HashSet::new();
                
                for conversation in cache.iter() {
                    agents.extend(conversation.participating_agents.iter().cloned());
                }

                for agent in agents {
                    if agent.to_lowercase().contains(&partial_lower) {
                        suggestions.push(CompletionSuggestion {
                            text: agent.clone(),
                            suggestion_type: SuggestionType::Agent,
                            score: self.calculate_completion_score(partial_input, &agent),
                            description: Some("Agent that participated in conversations".to_string()),
                            context: context.map(|s| s.to_string()),
                        });
                    }
                }
            }

            SuggestionType::Topic => {
                // Get topics from conversation metadata
                let cache = self.memory_cache.read().await;
                let mut topics = std::collections::HashSet::new();
                
                for conversation in cache.iter() {
                    topics.extend(conversation.metadata.topics.iter().cloned());
                }

                for topic in topics {
                    if topic.to_lowercase().contains(&partial_lower) {
                        suggestions.push(CompletionSuggestion {
                            text: topic.clone(),
                            suggestion_type: SuggestionType::Topic,
                            score: self.calculate_completion_score(partial_input, &topic),
                            description: Some("Topic discussed in conversations".to_string()),
                            context: context.map(|s| s.to_string()),
                        });
                    }
                }
            }

            SuggestionType::Tag => {
                // Get tags from conversations
                let cache = self.memory_cache.read().await;
                let mut tags = std::collections::HashSet::new();
                
                for conversation in cache.iter() {
                    tags.extend(conversation.tags.iter().cloned());
                }

                for tag in tags {
                    if tag.to_lowercase().contains(&partial_lower) {
                        suggestions.push(CompletionSuggestion {
                            text: tag.clone(),
                            suggestion_type: SuggestionType::Tag,
                            score: self.calculate_completion_score(partial_input, &tag),
                            description: Some("Tag used in conversations".to_string()),
                            context: context.map(|s| s.to_string()),
                        });
                    }
                }
            }

            _ => {
                // Handle other suggestion types as needed
            }
        }

        Ok(suggestions)
    }

    fn calculate_completion_score(&self, partial: &str, candidate: &str) -> f64 {
        if candidate.starts_with(partial) {
            // Prefix match gets high score
            0.9 - (candidate.len() - partial.len()) as f64 * 0.01
        } else if candidate.to_lowercase().contains(&partial.to_lowercase()) {
            // Substring match gets medium score
            0.6 - (candidate.len() - partial.len()) as f64 * 0.005
        } else {
            // Fuzzy match gets low score
            0.3
        }
    }

    async fn update_search_index(&self, conversation_id: &str, content: &str) {
        let mut index = self.search_index.write().await;
        
        // Simple word-based indexing
        for word in content.split_whitespace() {
            let word = word.to_lowercase();
            index.entry(word)
                .or_insert_with(Vec::new)
                .push(conversation_id.to_string());
        }
    }

    async fn update_completion_cache(&self, message: &ConversationMessage) {
        // This would extract relevant information from messages to improve completions
        // For now, this is a placeholder
    }

    async fn save_conversation_to_disk(&self, conversation: &ConversationSession) -> Result<(), DevKitError> {
        let file_path = self.storage_path.join(format!("{}.json", conversation.id));
        let content = serde_json::to_string_pretty(conversation)
            .map_err(|e| DevKitError::ValidationError {
                field: "conversation".to_string(),
                message: format!("Failed to serialize conversation: {}", e),
            })?;

        fs::write(&file_path, content)
            .map_err(|e| DevKitError::ContextualError {
                source: Box::new(e),
                context: "Failed to save conversation to disk".to_string(),
            })?;

        trace!("Saved conversation {} to disk", conversation.id);
        Ok(())
    }

    async fn load_conversation_from_disk(&self, conversation_id: &str) -> Result<Option<ConversationSession>, DevKitError> {
        let file_path = self.storage_path.join(format!("{}.json", conversation_id));
        if !file_path.exists() {
            return Ok(None);
        }

        self.load_conversation_from_path(&file_path).await.map(Some)
    }

    async fn load_conversation_from_path(&self, file_path: &Path) -> Result<ConversationSession, DevKitError> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| DevKitError::ContextualError {
                source: Box::new(e),
                context: "Failed to read conversation file".to_string(),
            })?;

        let conversation: ConversationSession = serde_json::from_str(&content)
            .map_err(|e| DevKitError::ValidationError {
                field: "conversation".to_string(),
                message: format!("Failed to deserialize conversation: {}", e),
            })?;

        Ok(conversation)
    }
}

impl Default for HistorySearchCriteria {
    fn default() -> Self {
        Self {
            query: None,
            date_range: None,
            message_types: Vec::new(),
            agents: Vec::new(),
            tags: Vec::new(),
            project_path: None,
            bookmarked_only: false,
            sort_by: HistorySortCriteria::Recent,
            limit: Some(20),
            include_content: false,
        }
    }
}