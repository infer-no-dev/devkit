//! Multi-session management for interactive mode

use crate::interactive::InteractiveSession;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use uuid::Uuid;

/// Manager for multiple interactive sessions
#[derive(Debug)]
pub struct SessionManager {
    sessions: HashMap<String, InteractiveSession>,
    active_session_id: Option<String>,
    session_history: VecDeque<String>, // Recently accessed sessions
    bookmarks: HashMap<String, SessionBookmark>,
    max_sessions: usize,
    max_history: usize,
}

/// Bookmark for quick session access
#[derive(Debug, Clone)]
pub struct SessionBookmark {
    pub name: String,
    pub description: String,
    pub session_id: String,
    pub created_at: std::time::SystemTime,
    pub tags: Vec<String>,
}

/// Session information for display
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub id: String,
    pub project_path: Option<PathBuf>,
    pub created_at: std::time::SystemTime,
    pub last_accessed: std::time::SystemTime,
    pub history_count: usize,
    pub artifact_count: usize,
    pub is_active: bool,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            active_session_id: None,
            session_history: VecDeque::new(),
            bookmarks: HashMap::new(),
            max_sessions: 10, // Configurable limit
            max_history: 50,
        }
    }

    /// Create a new session
    pub fn create_session(&mut self, project_path: Option<PathBuf>) -> String {
        let session_id = Uuid::new_v4().to_string();
        let session = InteractiveSession::new(project_path);

        // Remove oldest session if at limit
        if self.sessions.len() >= self.max_sessions {
            self.cleanup_old_sessions();
        }

        self.sessions.insert(session_id.clone(), session);
        self.set_active_session(&session_id);

        session_id
    }

    /// Switch to a different session
    pub fn switch_session(&mut self, session_id: &str) -> Result<(), String> {
        if !self.sessions.contains_key(session_id) {
            return Err(format!("Session '{}' not found", session_id));
        }

        self.set_active_session(session_id);
        Ok(())
    }

    /// Set the active session and update history
    fn set_active_session(&mut self, session_id: &str) {
        self.active_session_id = Some(session_id.to_string());

        // Update session history
        self.session_history.retain(|id| id != session_id);
        self.session_history.push_front(session_id.to_string());

        // Keep history size limited
        if self.session_history.len() > self.max_history {
            self.session_history.truncate(self.max_history);
        }
    }

    /// Get the active session
    pub fn get_active_session(&self) -> Option<&InteractiveSession> {
        if let Some(id) = &self.active_session_id {
            self.sessions.get(id)
        } else {
            None
        }
    }

    /// Get mutable reference to active session
    pub fn get_active_session_mut(&mut self) -> Option<&mut InteractiveSession> {
        if let Some(id) = &self.active_session_id {
            self.sessions.get_mut(id)
        } else {
            None
        }
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: &str) -> Option<&InteractiveSession> {
        self.sessions.get(session_id)
    }

    /// Get mutable session by ID
    pub fn get_session_mut(&mut self, session_id: &str) -> Option<&mut InteractiveSession> {
        self.sessions.get_mut(session_id)
    }

    /// List all sessions
    pub fn list_sessions(&self) -> Vec<SessionInfo> {
        let mut sessions: Vec<SessionInfo> = self
            .sessions
            .iter()
            .map(|(id, session)| SessionInfo {
                id: id.clone(),
                project_path: session.project_path.clone(),
                created_at: session.created_at,
                last_accessed: session.created_at, // TODO: Track actual last access
                history_count: session.history.len(),
                artifact_count: session.artifacts.len(),
                is_active: Some(id) == self.active_session_id.as_ref(),
            })
            .collect();

        // Sort by creation time, most recent first
        sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        sessions
    }

    /// Delete a session
    pub fn delete_session(&mut self, session_id: &str) -> Result<(), String> {
        if !self.sessions.contains_key(session_id) {
            return Err(format!("Session '{}' not found", session_id));
        }

        // Don't delete the active session
        if Some(session_id) == self.active_session_id.as_deref() {
            return Err("Cannot delete the active session".to_string());
        }

        self.sessions.remove(session_id);
        self.session_history.retain(|id| id != session_id);

        // Remove associated bookmarks
        let bookmark_ids: Vec<String> = self
            .bookmarks
            .iter()
            .filter(|(_, bookmark)| bookmark.session_id == session_id)
            .map(|(id, _)| id.clone())
            .collect();

        for bookmark_id in bookmark_ids {
            self.bookmarks.remove(&bookmark_id);
        }

        Ok(())
    }

    /// Save a session to file
    pub fn save_session(&self, session_id: &str, file_path: PathBuf) -> Result<(), String> {
        if let Some(session) = self.sessions.get(session_id) {
            session
                .save_to_file(file_path)
                .map_err(|e| format!("Failed to save session: {}", e))
        } else {
            Err(format!("Session '{}' not found", session_id))
        }
    }

    /// Load a session from file
    pub fn load_session(&mut self, file_path: PathBuf) -> Result<String, String> {
        let session = InteractiveSession::load_from_file(file_path)
            .map_err(|e| format!("Failed to load session: {}", e))?;

        let session_id = session.session_id.clone();

        // Remove existing session with same ID if it exists
        if self.sessions.contains_key(&session_id) {
            self.sessions.remove(&session_id);
        }

        self.sessions.insert(session_id.clone(), session);
        self.set_active_session(&session_id);

        Ok(session_id)
    }

    /// Create a bookmark for quick access
    pub fn create_bookmark(
        &mut self,
        name: String,
        description: String,
        session_id: String,
        tags: Vec<String>,
    ) -> Result<String, String> {
        if !self.sessions.contains_key(&session_id) {
            return Err(format!("Session '{}' not found", session_id));
        }

        let bookmark_id = Uuid::new_v4().to_string();
        let bookmark = SessionBookmark {
            name,
            description,
            session_id,
            created_at: std::time::SystemTime::now(),
            tags,
        };

        self.bookmarks.insert(bookmark_id.clone(), bookmark);
        Ok(bookmark_id)
    }

    /// Get all bookmarks
    pub fn list_bookmarks(&self) -> Vec<(&String, &SessionBookmark)> {
        let mut bookmarks: Vec<(&String, &SessionBookmark)> = self.bookmarks.iter().collect();
        bookmarks.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));
        bookmarks
    }

    /// Get a bookmark by ID
    pub fn get_bookmark(&self, bookmark_id: &str) -> Option<&SessionBookmark> {
        self.bookmarks.get(bookmark_id)
    }

    /// Switch to a bookmarked session
    pub fn switch_to_bookmark(&mut self, bookmark_id: &str) -> Result<(), String> {
        if let Some(bookmark) = self.bookmarks.get(bookmark_id) {
            let session_id = bookmark.session_id.clone();
            self.switch_session(&session_id)
        } else {
            Err(format!("Bookmark '{}' not found", bookmark_id))
        }
    }

    /// Delete a bookmark
    pub fn delete_bookmark(&mut self, bookmark_id: &str) -> Result<(), String> {
        if self.bookmarks.remove(bookmark_id).is_some() {
            Ok(())
        } else {
            Err(format!("Bookmark '{}' not found", bookmark_id))
        }
    }

    /// Get recent session history
    pub fn get_recent_sessions(&self) -> Vec<&String> {
        self.session_history.iter().collect()
    }

    /// Clean up old sessions to maintain limits
    fn cleanup_old_sessions(&mut self) {
        if self.sessions.len() < self.max_sessions {
            return;
        }

        // Find sessions to remove (oldest, not active, not in recent history)
        let mut candidates: Vec<String> = self
            .sessions
            .keys()
            .filter(|id| {
                // Don't remove active session
                if Some(*id) == self.active_session_id.as_ref() {
                    return false;
                }

                // Don't remove recently accessed sessions
                if self.session_history.contains(id) {
                    return false;
                }

                true
            })
            .cloned()
            .collect();

        // Sort by creation time, oldest first
        candidates.sort_by(|a, b| {
            let a_time = self
                .sessions
                .get(a)
                .map(|s| s.created_at)
                .unwrap_or(std::time::SystemTime::now());
            let b_time = self
                .sessions
                .get(b)
                .map(|s| s.created_at)
                .unwrap_or(std::time::SystemTime::now());
            a_time.cmp(&b_time)
        });

        // Remove sessions until we're under the limit
        let target_count = self.max_sessions - 1; // Leave room for new session
        while self.sessions.len() > target_count && !candidates.is_empty() {
            if let Some(session_id) = candidates.pop() {
                let _ = self.delete_session(&session_id);
            }
        }
    }

    /// Get active session ID
    pub fn get_active_session_id(&self) -> Option<&String> {
        self.active_session_id.as_ref()
    }

    /// Get session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Check if session exists
    pub fn has_session(&self, session_id: &str) -> bool {
        self.sessions.contains_key(session_id)
    }

    /// Clone a session with a new ID
    pub fn clone_session(&mut self, session_id: &str) -> Result<String, String> {
        if let Some(session) = self.sessions.get(session_id) {
            let new_session_id = Uuid::new_v4().to_string();
            let mut new_session = session.clone();
            new_session.session_id = new_session_id.clone();
            new_session.created_at = std::time::SystemTime::now();

            self.sessions.insert(new_session_id.clone(), new_session);
            Ok(new_session_id)
        } else {
            Err(format!("Session '{}' not found", session_id))
        }
    }

    /// Rename a session (via bookmark)
    pub fn rename_session_via_bookmark(
        &mut self,
        session_id: &str,
        new_name: String,
    ) -> Result<(), String> {
        if !self.sessions.contains_key(session_id) {
            return Err(format!("Session '{}' not found", session_id));
        }

        // Create or update bookmark with new name
        let bookmark_id = format!("session_{}", session_id);
        let bookmark = SessionBookmark {
            name: new_name,
            description: format!("Quick access to session {}", session_id),
            session_id: session_id.to_string(),
            created_at: std::time::SystemTime::now(),
            tags: vec!["session".to_string()],
        };

        self.bookmarks.insert(bookmark_id, bookmark);
        Ok(())
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
