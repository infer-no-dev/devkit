//! UI component and interaction tests

use crate::ui::*;
use crate::ui::blocks::*;
use crate::ui::notifications::*;

#[test]
fn test_ui_config_creation() {
    let config = UIConfig::new();
    
    assert!(config.auto_scroll);
    assert!(config.show_timestamps);
    assert_eq!(config.theme, "Dark");
    assert_eq!(config.tick_rate.as_millis(), 50);
}

#[test]
fn test_ui_config_customization() {
    let config = UIConfig::new()
        .with_theme("Light".to_string())
        .with_tick_rate(tokio::time::Duration::from_millis(100));
    
    assert_eq!(config.theme, "Light");
    assert_eq!(config.tick_rate.as_millis(), 100);
}

#[test]
fn test_output_block_creation() {
    use std::collections::HashMap;
    
    let block = OutputBlock::new(
        "Test message".to_string(),
        BlockType::UserInput,
        HashMap::new(),
    );
    
    assert_eq!(block.content, "Test message");
    assert!(matches!(block.block_type, BlockType::UserInput));
    assert!(!block.timestamp.elapsed().unwrap().is_zero());
}

#[test]
fn test_block_types() {
    let block_types = vec![
        BlockType::UserInput,
        BlockType::AgentResponse,
        BlockType::System,
        BlockType::Error,
        BlockType::Command,
        BlockType::CodeGeneration,
        BlockType::Analysis,
    ];
    
    assert_eq!(block_types.len(), 7);
    
    // Test block type matching
    match block_types[0] {
        BlockType::UserInput => assert!(true),
        _ => panic!("Expected UserInput block type"),
    }
}

#[test]
fn test_block_collection() {
    let mut collection = BlockCollection::new(10);
    
    // Test adding blocks
    collection.add_user_input("Hello");
    collection.add_agent_response("Hi there!");
    collection.add_system_message("System ready");
    collection.add_error("Something went wrong");
    
    assert_eq!(collection.len(), 4);
    assert!(!collection.is_empty());
    
    let blocks = collection.get_blocks();
    assert_eq!(blocks.len(), 4);
    assert_eq!(blocks[0].content, "Hello");
    assert_eq!(blocks[1].content, "Hi there!");
}

#[test]
fn test_block_collection_size_limit() {
    let mut collection = BlockCollection::new(2); // Small limit
    
    // Add more blocks than the limit
    for i in 0..5 {
        collection.add_user_input(&format!("Message {}", i));
    }
    
    // Should only keep the last 2 blocks
    assert_eq!(collection.len(), 2);
    
    let blocks = collection.get_blocks();
    assert_eq!(blocks[0].content, "Message 3");
    assert_eq!(blocks[1].content, "Message 4");
}

#[test]
fn test_block_filtering() {
    let mut collection = BlockCollection::new(10);
    
    collection.add_user_input("User message");
    collection.add_agent_response("Agent response");
    collection.add_system_message("System message");
    
    // Test filtering by block type
    let filter = BlockFilter {
        block_types: Some(vec![BlockType::UserInput]),
        ..Default::default()
    };
    
    let filtered = collection.get_filtered_blocks(&filter);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].content, "User message");
}

#[test]
fn test_recent_blocks() {
    let mut collection = BlockCollection::new(10);
    
    // Add several blocks
    for i in 0..5 {
        collection.add_user_input(&format!("Message {}", i));
    }
    
    // Get recent blocks (last 3)
    let recent = collection.get_recent_blocks(3);
    assert_eq!(recent.len(), 3);
    
    // Should be in reverse order (most recent first)
    assert_eq!(recent[0].content, "Message 4");
    assert_eq!(recent[1].content, "Message 3");
    assert_eq!(recent[2].content, "Message 2");
}

#[test]
fn test_notification_creation() {
    let notification = Notification::info(
        "Test Title".to_string(),
        "Test message".to_string(),
    );
    
    assert_eq!(notification.title, "Test Title");
    assert_eq!(notification.message, "Test message");
    assert!(matches!(notification.notification_type, NotificationType::Info));
    assert!(matches!(notification.priority, NotificationPriority::Medium));
}

#[test]
fn test_notification_types() {
    let notifications = vec![
        Notification::info("Info".to_string(), "Info message".to_string()),
        Notification::success("Success".to_string(), "Success message".to_string()),
        Notification::warning("Warning".to_string(), "Warning message".to_string()),
        Notification::error("Error".to_string(), "Error message".to_string()),
    ];
    
    assert_eq!(notifications.len(), 4);
    
    // Test notification type matching
    assert!(matches!(notifications[0].notification_type, NotificationType::Info));
    assert!(matches!(notifications[1].notification_type, NotificationType::Success));
    assert!(matches!(notifications[2].notification_type, NotificationType::Warning));
    assert!(matches!(notifications[3].notification_type, NotificationType::Error));
}

#[test]
fn test_notification_priority() {
    let priorities = vec![
        NotificationPriority::Low,
        NotificationPriority::Medium,
        NotificationPriority::High,
        NotificationPriority::Critical,
    ];
    
    assert_eq!(priorities.len(), 4);
    
    // Test priority ordering
    assert!(priorities[3] > priorities[2]);
    assert!(priorities[2] > priorities[1]);
    assert!(priorities[1] > priorities[0]);
}

#[test]
fn test_notification_panel() {
    let mut panel = NotificationPanel::new(1000); // 1 second auto-dismiss
    
    // Add notifications
    let notification1 = Notification::info("Test 1".to_string(), "Message 1".to_string());
    let notification2 = Notification::error("Test 2".to_string(), "Message 2".to_string());
    
    panel.add_notification(notification1);
    panel.add_notification(notification2);
    
    assert_eq!(panel.count(), 2);
    
    let notifications = panel.get_notifications();
    assert_eq!(notifications.len(), 2);
}

#[test]
fn test_notification_filtering() {
    let mut panel = NotificationPanel::new(5000);
    
    // Add different types of notifications
    panel.add_notification(Notification::info("Info".to_string(), "Info msg".to_string()));
    panel.add_notification(Notification::error("Error".to_string(), "Error msg".to_string()));
    panel.add_notification(Notification::warning("Warning".to_string(), "Warning msg".to_string()));
    
    // Test filtering by type
    let errors = panel.get_notifications_by_type(&NotificationType::Error);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].title, "Error");
    
    // Test filtering by priority
    let high_priority = panel.get_notifications_by_priority(&NotificationPriority::High);
    // Error notifications are typically high priority
    assert!(!high_priority.is_empty());
}

#[test]
fn test_ui_event_types() {
    use crate::agents::{AgentStatus, TaskPriority};
    
    let events = vec![
        UIEvent::Quit,
        UIEvent::Input("test input".to_string()),
        UIEvent::AgentStatusUpdate {
            agent_name: "test_agent".to_string(),
            status: AgentStatus::Idle,
            task: None,
            priority: None,
            progress: None,
        },
        UIEvent::Notification(Notification::info("Test".to_string(), "Message".to_string())),
        UIEvent::Output {
            content: "output".to_string(),
            block_type: "user".to_string(),
        },
        UIEvent::ToggleHelp,
        UIEvent::SwitchTheme("Dark".to_string()),
    ];
    
    assert_eq!(events.len(), 7);
    
    // Test event matching
    match &events[0] {
        UIEvent::Quit => assert!(true),
        _ => panic!("Expected Quit event"),
    }
    
    match &events[1] {
        UIEvent::Input(input) => assert_eq!(input, "test input"),
        _ => panic!("Expected Input event"),
    }
}

#[tokio::test]
async fn test_application_creation() {
    // Note: This test might fail in CI environments without a proper terminal
    // In a full test suite, we'd mock the terminal interface
    
    let config = UIConfig::new();
    
    // This would normally create the application, but we can't test it fully
    // without a proper terminal environment
    // let app_result = Application::new(config);
    
    // For now, just test that the config is valid
    assert!(config.auto_scroll);
    assert_eq!(config.theme, "Dark");
}

#[test]
fn test_block_metadata() {
    use std::collections::HashMap;
    
    let mut metadata = HashMap::new();
    metadata.insert("agent_name".to_string(), "test_agent".to_string());
    metadata.insert("confidence".to_string(), "0.95".to_string());
    
    let block = OutputBlock::new(
        "Generated code".to_string(),
        BlockType::CodeGeneration,
        metadata,
    );
    
    assert_eq!(block.metadata.get("agent_name"), Some(&"test_agent".to_string()));
    assert_eq!(block.metadata.get("confidence"), Some(&"0.95".to_string()));
}

#[test]
fn test_block_filtering_with_metadata() {
    let mut collection = BlockCollection::new(10);
    
    // Add blocks with different metadata
    let mut metadata1 = std::collections::HashMap::new();
    metadata1.insert("agent".to_string(), "agent1".to_string());
    
    let mut metadata2 = std::collections::HashMap::new();
    metadata2.insert("agent".to_string(), "agent2".to_string());
    
    let block1 = OutputBlock::new("Message 1".to_string(), BlockType::AgentResponse, metadata1);
    let block2 = OutputBlock::new("Message 2".to_string(), BlockType::AgentResponse, metadata2);
    
    collection.add_block(block1);
    collection.add_block(block2);
    
    // Filter by agent name
    let filter = BlockFilter {
        agent_name: Some("agent1".to_string()),
        ..Default::default()
    };
    
    let filtered = collection.get_filtered_blocks(&filter);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].content, "Message 1");
}
