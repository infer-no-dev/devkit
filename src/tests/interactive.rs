//! Interactive session and management tests

use crate::interactive::*;
use std::path::PathBuf;

#[test]
fn test_interactive_session_creation() {
    let project_path = Some(PathBuf::from("/test/project"));
    let session = InteractiveSession::new(project_path.clone());
    
    assert!(!session.session_id.is_empty());
    assert_eq!(session.project_path, project_path);
    assert!(session.history.is_empty());
    assert!(session.artifacts.is_empty());
}

#[test]
fn test_session_config() {
    let config = SessionConfig::default();
    
    assert!(!config.auto_save);
    assert!(config.default_language.is_none());
    assert!(config.show_confidence);
    assert!(!config.verbose);
    assert_eq!(config.max_history, 100);
}

#[test]
fn test_conversation_entry() {
    let entry = ConversationEntry {
        timestamp: std::time::SystemTime::now(),
        role: ConversationRole::User,
        content: "Generate a function".to_string(),
        result: None,
        entry_type: EntryType::Generate,
    };
    
    assert!(matches!(entry.role, ConversationRole::User));
    assert!(matches!(entry.entry_type, EntryType::Generate));
    assert_eq!(entry.content, "Generate a function");
}

#[test]
fn test_code_artifact() {
    let artifact = CodeArtifact {
        id: "artifact_1".to_string(),
        name: "Hello World Function".to_string(),
        language: "rust".to_string(),
        code: "fn hello() { println!(\"Hello, World!\"); }".to_string(),
        file_path: None,
        created_at: std::time::SystemTime::now(),
        modified_at: std::time::SystemTime::now(),
        confidence: 0.95,
        versions: Vec::new(),
    };
    
    assert_eq!(artifact.id, "artifact_1");
    assert_eq!(artifact.language, "rust");
    assert!(artifact.code.contains("Hello, World!"));
    assert_eq!(artifact.confidence, 0.95);
}

#[test]
fn test_session_history_management() {
    let mut session = InteractiveSession::new(None);
    
    // Test adding entries
    let entry1 = ConversationEntry {
        timestamp: std::time::SystemTime::now(),
        role: ConversationRole::User,
        content: "First message".to_string(),
        result: None,
        entry_type: EntryType::Chat,
    };
    
    let entry2 = ConversationEntry {
        timestamp: std::time::SystemTime::now(),
        role: ConversationRole::Assistant,
        content: "First response".to_string(),
        result: None,
        entry_type: EntryType::Chat,
    };
    
    session.add_history_entry(entry1);
    session.add_history_entry(entry2);
    
    assert_eq!(session.history.len(), 2);
    assert_eq!(session.history[0].content, "First message");
    assert_eq!(session.history[1].content, "First response");
}

#[test]
fn test_artifact_management() {
    let mut session = InteractiveSession::new(None);
    
    let artifact = CodeArtifact {
        id: "test_artifact".to_string(),
        name: "Test Function".to_string(),
        language: "rust".to_string(),
        code: "fn test() {}".to_string(),
        file_path: None,
        created_at: std::time::SystemTime::now(),
        modified_at: std::time::SystemTime::now(),
        confidence: 0.8,
        versions: Vec::new(),
    };
    
    session.add_artifact(artifact);
    
    assert_eq!(session.artifacts.len(), 1);
    assert!(session.artifacts.contains_key("test_artifact"));
    
    // Test retrieving artifact
    let retrieved = session.get_artifact("test_artifact");
    assert!(retrieved.is_some());
    
    if let Some(artifact) = retrieved {
        assert_eq!(artifact.name, "Test Function");
        assert_eq!(artifact.language, "rust");
    }
}

#[test]
fn test_artifact_versioning() {
    let mut session = InteractiveSession::new(None);
    
    let mut artifact = CodeArtifact {
        id: "versioned_artifact".to_string(),
        name: "Versioned Function".to_string(),
        language: "rust".to_string(),
        code: "fn original() {}".to_string(),
        file_path: None,
        created_at: std::time::SystemTime::now(),
        modified_at: std::time::SystemTime::now(),
        confidence: 0.9,
        versions: Vec::new(),
    };
    
    // Add initial artifact
    session.add_artifact(artifact.clone());
    
    // Update artifact
    artifact.code = "fn updated() {}".to_string();
    session.add_artifact(artifact);
    
    // Check that version history is maintained
    let retrieved = session.get_artifact("versioned_artifact").unwrap();
    assert_eq!(retrieved.code, "fn updated() {}");
    assert_eq!(retrieved.versions.len(), 1);
    assert_eq!(retrieved.versions[0].code, "fn original() {}");
}

#[test]
fn test_history_size_limit() {
    let mut config = SessionConfig::default();
    config.max_history = 2; // Set small limit for testing
    
    let mut session = InteractiveSession::new(None);
    session.config = config;
    
    // Add entries beyond the limit
    for i in 0..5 {
        let entry = ConversationEntry {
            timestamp: std::time::SystemTime::now(),
            role: ConversationRole::User,
            content: format!("Message {}", i),
            result: None,
            entry_type: EntryType::Chat,
        };
        session.add_history_entry(entry);
    }
    
    // Should only keep the last 2 entries
    assert_eq!(session.history.len(), 2);
    assert_eq!(session.history[0].content, "Message 3");
    assert_eq!(session.history[1].content, "Message 4");
}

#[test]
fn test_entry_types() {
    let entry_types = vec![
        EntryType::Generate,
        EntryType::Refine,
        EntryType::Explain,
        EntryType::Optimize,
        EntryType::AddTests,
        EntryType::Debug,
        EntryType::Chat,
        EntryType::Status,
    ];
    
    assert_eq!(entry_types.len(), 8);
    
    // Test that entry types can be matched
    match entry_types[0] {
        EntryType::Generate => assert!(true),
        _ => panic!("Expected Generate entry type"),
    }
}

#[test]
fn test_conversation_roles() {
    let roles = vec![
        ConversationRole::User,
        ConversationRole::Assistant,
        ConversationRole::System,
    ];
    
    assert_eq!(roles.len(), 3);
    
    // Test role matching
    match roles[0] {
        ConversationRole::User => assert!(true),
        _ => panic!("Expected User role"),
    }
}

#[test] 
fn test_code_version() {
    let version = CodeVersion {
        version: 1,
        code: "fn test() {}".to_string(),
        description: "Initial version".to_string(),
        timestamp: std::time::SystemTime::now(),
    };
    
    assert_eq!(version.version, 1);
    assert_eq!(version.description, "Initial version");
    assert!(version.code.contains("fn test"));
}

#[tokio::test]
async fn test_session_persistence() {
    use tempfile::NamedTempFile;
    
    // Create a session with some data
    let mut session = InteractiveSession::new(Some(PathBuf::from("/test")));
    
    let entry = ConversationEntry {
        timestamp: std::time::SystemTime::now(),
        role: ConversationRole::User,
        content: "Test message".to_string(),
        result: None,
        entry_type: EntryType::Chat,
    };
    session.add_history_entry(entry);
    
    // Save to temporary file
    let temp_file = NamedTempFile::new().unwrap();
    let save_result = session.save_to_file(temp_file.path().to_path_buf());
    assert!(save_result.is_ok());
    
    // Load from file
    let load_result = InteractiveSession::load_from_file(temp_file.path().to_path_buf());
    assert!(load_result.is_ok());
    
    if let Ok(loaded_session) = load_result {
        assert_eq!(loaded_session.session_id, session.session_id);
        assert_eq!(loaded_session.history.len(), 1);
        assert_eq!(loaded_session.history[0].content, "Test message");
    }
}
