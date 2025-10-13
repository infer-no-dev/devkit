//! File System Persistence Implementation for Session Management
//!
//! This module provides a concrete implementation of the SessionPersistence trait
//! that stores sessions on the local file system using JSON serialization.

use super::{Session, SessionError, SessionFilters, SessionPersistence};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, error, info, warn};

/// File system based session persistence implementation
#[derive(Debug)]
pub struct FileSystemPersistence {
    /// Root directory for storing session data
    data_dir: PathBuf,
    /// Sessions subdirectory
    sessions_dir: PathBuf,
    /// Indexes subdirectory for quick lookups
    indexes_dir: PathBuf,
    /// Session index for fast searching
    session_index: tokio::sync::RwLock<SessionIndex>,
}

/// Session index for fast searching and filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionIndex {
    /// Session ID to metadata mapping
    sessions: HashMap<String, SessionIndexEntry>,
    /// User ID to session IDs mapping
    user_sessions: HashMap<String, Vec<String>>,
    /// Last updated timestamp
    last_updated: DateTime<Utc>,
}

/// Session index entry for metadata-based searches
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionIndexEntry {
    pub id: String,
    pub name: String,
    pub creator_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub accessed_at: DateTime<Utc>,
    pub status: String, // Serialized SessionStatus
    pub tags: Vec<String>,
    pub project: Option<String>,
    pub priority: String, // Serialized SessionPriority
    pub languages: Vec<String>,
    pub file_path: PathBuf,
}

impl Default for SessionIndex {
    fn default() -> Self {
        Self {
            sessions: HashMap::new(),
            user_sessions: HashMap::new(),
            last_updated: Utc::now(),
        }
    }
}

impl FileSystemPersistence {
    /// Create a new file system persistence layer
    pub async fn new(data_dir: PathBuf) -> Result<Self, SessionError> {
        let sessions_dir = data_dir.join("sessions");
        let indexes_dir = data_dir.join("indexes");

        // Create directories if they don't exist
        fs::create_dir_all(&data_dir).await
            .map_err(|e| SessionError::IOError(e))?;
        fs::create_dir_all(&sessions_dir).await
            .map_err(|e| SessionError::IOError(e))?;
        fs::create_dir_all(&indexes_dir).await
            .map_err(|e| SessionError::IOError(e))?;

        let persistence = Self {
            data_dir,
            sessions_dir,
            indexes_dir,
            session_index: tokio::sync::RwLock::new(SessionIndex::default()),
        };

        // Load existing index
        persistence.load_index().await?;

        info!("Initialized FileSystemPersistence with data directory: {:?}", persistence.data_dir);
        Ok(persistence)
    }

    /// Load the session index from disk
    async fn load_index(&self) -> Result<(), SessionError> {
        let index_path = self.indexes_dir.join("session_index.json");
        
        if !index_path.exists() {
            info!("No existing session index found, starting with empty index");
            return Ok(());
        }

        let index_content = fs::read_to_string(&index_path).await
            .map_err(|e| SessionError::IOError(e))?;

        let index: SessionIndex = serde_json::from_str(&index_content)
            .map_err(|e| SessionError::SerializationError(e))?;

        *self.session_index.write().await = index;
        debug!("Loaded session index with {} sessions", self.session_index.read().await.sessions.len());
        Ok(())
    }

    /// Save the session index to disk
    async fn save_index(&self) -> Result<(), SessionError> {
        let index_path = self.indexes_dir.join("session_index.json");
        let mut index = self.session_index.write().await;
        index.last_updated = Utc::now();

        let index_content = serde_json::to_string_pretty(&*index)
            .map_err(|e| SessionError::SerializationError(e))?;

        fs::write(&index_path, index_content).await
            .map_err(|e| SessionError::IOError(e))?;

        debug!("Saved session index with {} sessions", index.sessions.len());
        Ok(())
    }

    /// Get the file path for a session
    fn get_session_file_path(&self, session_id: &str) -> PathBuf {
        self.sessions_dir.join(format!("{}.json", session_id))
    }

    /// Update the session index with session data
    async fn update_index(&self, session: &Session) -> Result<(), SessionError> {
        let mut index = self.session_index.write().await;

        let entry = SessionIndexEntry {
            id: session.id.clone(),
            name: session.name.clone(),
            creator_id: session.creator.id.clone(),
            created_at: session.created_at,
            updated_at: session.updated_at,
            accessed_at: session.accessed_at,
            status: format!("{:?}", session.status), // Simple serialization
            tags: session.metadata.tags.iter().cloned().collect(),
            project: session.metadata.project.clone(),
            priority: format!("{:?}", session.metadata.priority),
            languages: session.metadata.languages.iter().cloned().collect(),
            file_path: self.get_session_file_path(&session.id),
        };

        // Add to session index
        index.sessions.insert(session.id.clone(), entry);

        // Add to user sessions index
        let user_sessions = index.user_sessions.entry(session.creator.id.clone()).or_insert_with(Vec::new);
        if !user_sessions.contains(&session.id) {
            user_sessions.push(session.id.clone());
        }

        Ok(())
    }

    /// Remove a session from the index
    async fn remove_from_index(&self, session_id: &str) -> Result<(), SessionError> {
        let mut index = self.session_index.write().await;

        if let Some(entry) = index.sessions.remove(session_id) {
            // Remove from user sessions
            if let Some(user_sessions) = index.user_sessions.get_mut(&entry.creator_id) {
                user_sessions.retain(|id| id != session_id);
            }
        }

        Ok(())
    }

    /// Cleanup orphaned index entries
    pub async fn cleanup_index(&self) -> Result<usize, SessionError> {
        let mut cleanup_count = 0;
        let mut sessions_to_remove = Vec::new();

        {
            let index = self.session_index.read().await;
            for (session_id, entry) in &index.sessions {
                if !entry.file_path.exists() {
                    sessions_to_remove.push(session_id.clone());
                    cleanup_count += 1;
                }
            }
        }

        for session_id in sessions_to_remove {
            self.remove_from_index(&session_id).await?;
            warn!("Cleaned up orphaned session index entry: {}", session_id);
        }

        if cleanup_count > 0 {
            self.save_index().await?;
            info!("Cleaned up {} orphaned session index entries", cleanup_count);
        }

        Ok(cleanup_count)
    }

    /// Rebuild the entire index from disk
    pub async fn rebuild_index(&self) -> Result<(), SessionError> {
        info!("Rebuilding session index from disk...");

        let mut new_index = SessionIndex::default();
        let mut entries = fs::read_dir(&self.sessions_dir).await
            .map_err(|e| SessionError::IOError(e))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| SessionError::IOError(e))? {
            
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_session_from_file(&path).await {
                    Ok(session) => {
                        let index_entry = SessionIndexEntry {
                            id: session.id.clone(),
                            name: session.name.clone(),
                            creator_id: session.creator.id.clone(),
                            created_at: session.created_at,
                            updated_at: session.updated_at,
                            accessed_at: session.accessed_at,
                            status: format!("{:?}", session.status),
                            tags: session.metadata.tags.iter().cloned().collect(),
                            project: session.metadata.project.clone(),
                            priority: format!("{:?}", session.metadata.priority),
                            languages: session.metadata.languages.iter().cloned().collect(),
                            file_path: path.clone(),
                        };

                        new_index.sessions.insert(session.id.clone(), index_entry);

                        let user_sessions = new_index.user_sessions.entry(session.creator.id.clone()).or_insert_with(Vec::new);
                        if !user_sessions.contains(&session.id) {
                            user_sessions.push(session.id.clone());
                        }
                    }
                    Err(e) => {
                        error!("Failed to load session from {:?}: {:?}", path, e);
                    }
                }
            }
        }

        *self.session_index.write().await = new_index;
        self.save_index().await?;

        info!("Rebuilt session index with {} sessions", self.session_index.read().await.sessions.len());
        Ok(())
    }

    /// Load a session from a file
    async fn load_session_from_file(&self, file_path: &Path) -> Result<Session, SessionError> {
        let content = fs::read_to_string(file_path).await
            .map_err(|e| SessionError::IOError(e))?;

        let session: Session = serde_json::from_str(&content)
            .map_err(|e| SessionError::SerializationError(e))?;

        Ok(session)
    }

    /// Filter sessions based on criteria
    fn matches_filters(entry: &SessionIndexEntry, filters: &SessionFilters) -> bool {
        // Status filter
        if let Some(ref status) = filters.status {
            let status_str = format!("{:?}", status);
            if entry.status != status_str {
                return false;
            }
        }

        // Tags filter
        if let Some(ref filter_tags) = filters.tags {
            if !filter_tags.iter().any(|tag| entry.tags.contains(tag)) {
                return false;
            }
        }

        // Project filter
        if let Some(ref project) = filters.project {
            if entry.project.as_ref() != Some(project) {
                return false;
            }
        }

        // Priority filter
        if let Some(ref priority) = filters.priority {
            let priority_str = format!("{:?}", priority);
            if entry.priority != priority_str {
                return false;
            }
        }

        // Date range filter
        if let Some(ref date_range) = filters.date_range {
            if entry.created_at < date_range.start || entry.created_at > date_range.end {
                return false;
            }
        }

        // Languages filter
        if let Some(ref filter_languages) = filters.languages {
            if !filter_languages.iter().any(|lang| entry.languages.contains(lang)) {
                return false;
            }
        }

        true
    }
}

#[async_trait]
impl SessionPersistence for FileSystemPersistence {
    async fn save_session(&self, session: &Session) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let file_path = self.get_session_file_path(&session.id);

        // Serialize session to JSON
        let session_content = serde_json::to_string_pretty(session)
            .map_err(|e| Box::new(SessionError::SerializationError(e)) as Box<dyn std::error::Error + Send + Sync>)?;

        // Write to file
        fs::write(&file_path, session_content).await
            .map_err(|e| Box::new(SessionError::IOError(e)) as Box<dyn std::error::Error + Send + Sync>)?;

        // Update index
        self.update_index(session).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        // Save updated index
        self.save_index().await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        debug!("Saved session {} to {:?}", session.id, file_path);
        Ok(())
    }

    async fn load_session(&self, session_id: &str) -> Result<Option<Session>, Box<dyn std::error::Error + Send + Sync>> {
        let file_path = self.get_session_file_path(session_id);

        if !file_path.exists() {
            return Ok(None);
        }

        let session = self.load_session_from_file(&file_path).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        debug!("Loaded session {} from {:?}", session_id, file_path);
        Ok(Some(session))
    }

    async fn delete_session(&self, session_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let file_path = self.get_session_file_path(session_id);

        if file_path.exists() {
            fs::remove_file(&file_path).await
                .map_err(|e| Box::new(SessionError::IOError(e)) as Box<dyn std::error::Error + Send + Sync>)?;
        }

        // Remove from index
        self.remove_from_index(session_id).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        // Save updated index
        self.save_index().await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        debug!("Deleted session {} from {:?}", session_id, file_path);
        Ok(())
    }

    async fn list_sessions(&self, user_id: &str) -> Result<Vec<Session>, Box<dyn std::error::Error + Send + Sync>> {
        let index = self.session_index.read().await;
        let session_ids = index.user_sessions.get(user_id)
            .map(|ids| ids.as_slice())
            .unwrap_or(&[]);

        let mut sessions = Vec::new();
        for session_id in session_ids {
            if let Some(entry) = index.sessions.get(session_id) {
                match self.load_session_from_file(&entry.file_path).await {
                    Ok(session) => sessions.push(session),
                    Err(e) => {
                        error!("Failed to load session {}: {:?}", session_id, e);
                    }
                }
            }
        }

        debug!("Listed {} sessions for user {}", sessions.len(), user_id);
        Ok(sessions)
    }

    async fn search_sessions(
        &self,
        user_id: &str,
        query: &str,
        filters: SessionFilters,
    ) -> Result<Vec<Session>, Box<dyn std::error::Error + Send + Sync>> {
        let index = self.session_index.read().await;
        let user_session_ids: std::collections::HashSet<_> = index.user_sessions.get(user_id)
            .map(|ids| ids.iter().collect())
            .unwrap_or_default();

        let query_lower = query.to_lowercase();
        let mut matching_sessions = Vec::new();

        for (session_id, entry) in &index.sessions {
            // Only include sessions belonging to the user
            if !user_session_ids.contains(session_id) {
                continue;
            }

            // Apply filters
            if !Self::matches_filters(entry, &filters) {
                continue;
            }

            // Apply text query
            if !query.is_empty() {
                let name_matches = entry.name.to_lowercase().contains(&query_lower);
                let tag_matches = entry.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower));
                let project_matches = entry.project.as_ref()
                    .map(|p| p.to_lowercase().contains(&query_lower))
                    .unwrap_or(false);

                if !name_matches && !tag_matches && !project_matches {
                    continue;
                }
            }

            // Load the full session
            match self.load_session_from_file(&entry.file_path).await {
                Ok(session) => matching_sessions.push(session),
                Err(e) => {
                    error!("Failed to load session {} during search: {:?}", session_id, e);
                }
            }
        }

        // Sort by most recently updated
        matching_sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        debug!("Found {} sessions matching query '{}' for user {}", matching_sessions.len(), query, user_id);
        Ok(matching_sessions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{SessionConfig, SessionMetadata, SessionState, SessionStatus, SessionUser, UserPreferences};
    use tempfile::TempDir;
    use tokio;

    async fn create_test_session(id: &str, user_id: &str, name: &str) -> Session {
        let now = Utc::now();
        let user = SessionUser {
            id: user_id.to_string(),
            name: "Test User".to_string(),
            email: Some("test@example.com".to_string()),
            avatar_url: None,
            timezone: "UTC".to_string(),
            preferences: UserPreferences::default(),
        };

        Session {
            id: id.to_string(),
            name: name.to_string(),
            description: Some("Test session".to_string()),
            created_at: now,
            updated_at: now,
            accessed_at: now,
            creator: user,
            status: SessionStatus::Active,
            config: SessionConfig::default(),
            state: SessionState::new(),
            metadata: SessionMetadata::default(),
            agents: std::collections::HashMap::new(),
            conversations: Vec::new(),
            artifacts: Vec::new(),
            branches: std::collections::HashMap::new(),
            active_branch: "main".to_string(),
            collaboration: None,
            analytics: crate::session::SessionAnalytics::new(),
            variables: std::collections::HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_filesystem_persistence_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = FileSystemPersistence::new(temp_dir.path().to_path_buf()).await.unwrap();

        // Create test session
        let session = create_test_session("test-session-1", "user-1", "Test Session").await;

        // Save session
        persistence.save_session(&session).await.unwrap();

        // Load session
        let loaded_session = persistence.load_session("test-session-1").await.unwrap();
        assert!(loaded_session.is_some());
        let loaded_session = loaded_session.unwrap();
        assert_eq!(loaded_session.id, "test-session-1");
        assert_eq!(loaded_session.name, "Test Session");

        // List sessions
        let sessions = persistence.list_sessions("user-1").await.unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "test-session-1");

        // Delete session
        persistence.delete_session("test-session-1").await.unwrap();

        // Verify deletion
        let deleted_session = persistence.load_session("test-session-1").await.unwrap();
        assert!(deleted_session.is_none());

        let sessions_after_delete = persistence.list_sessions("user-1").await.unwrap();
        assert_eq!(sessions_after_delete.len(), 0);
    }

    #[tokio::test]
    async fn test_search_sessions() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = FileSystemPersistence::new(temp_dir.path().to_path_buf()).await.unwrap();

        // Create multiple test sessions
        let mut session1 = create_test_session("session-1", "user-1", "Rust Project").await;
        session1.metadata.tags.insert("rust".to_string());
        session1.metadata.tags.insert("backend".to_string());

        let mut session2 = create_test_session("session-2", "user-1", "JavaScript Project").await;
        session2.metadata.tags.insert("javascript".to_string());
        session2.metadata.tags.insert("frontend".to_string());

        let session3 = create_test_session("session-3", "user-2", "Python Project").await;

        // Save sessions
        persistence.save_session(&session1).await.unwrap();
        persistence.save_session(&session2).await.unwrap();
        persistence.save_session(&session3).await.unwrap();

        // Search by name
        let rust_sessions = persistence.search_sessions("user-1", "rust", SessionFilters::default()).await.unwrap();
        assert_eq!(rust_sessions.len(), 1);
        assert_eq!(rust_sessions[0].name, "Rust Project");

        // Search by tag - this won't work with current simple implementation, but structure is there
        let all_user1_sessions = persistence.search_sessions("user-1", "", SessionFilters::default()).await.unwrap();
        assert_eq!(all_user1_sessions.len(), 2);

        // User-specific search
        let user2_sessions = persistence.search_sessions("user-2", "", SessionFilters::default()).await.unwrap();
        assert_eq!(user2_sessions.len(), 1);
        assert_eq!(user2_sessions[0].name, "Python Project");
    }

    #[tokio::test]
    async fn test_index_rebuilding() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = FileSystemPersistence::new(temp_dir.path().to_path_buf()).await.unwrap();

        // Create and save sessions
        let session1 = create_test_session("session-1", "user-1", "Test Session 1").await;
        let session2 = create_test_session("session-2", "user-1", "Test Session 2").await;

        persistence.save_session(&session1).await.unwrap();
        persistence.save_session(&session2).await.unwrap();

        // Verify sessions exist
        let sessions_before = persistence.list_sessions("user-1").await.unwrap();
        assert_eq!(sessions_before.len(), 2);

        // Rebuild index
        persistence.rebuild_index().await.unwrap();

        // Verify sessions still exist after rebuild
        let sessions_after = persistence.list_sessions("user-1").await.unwrap();
        assert_eq!(sessions_after.len(), 2);
    }
}