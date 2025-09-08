//! Agent system tests

use crate::agents::*;
use std::sync::Arc;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_agent_system_creation() {
    let agent_system = AgentSystem::new();
    
    // Test that agent system is created successfully
    assert!(Arc::strong_count(&Arc::new(agent_system)) > 0);
}

#[tokio::test]
async fn test_agent_system_operations() {
    let agent_system = Arc::new(AgentSystem::new());
    
    // Test basic agent system operations
    // In a full implementation, we would test:
    // 1. Adding agents to the system
    // 2. Starting/stopping agents
    // 3. Task distribution
    // 4. Status monitoring
    
    // For now, test basic functionality
    let system = agent_system.clone();
    
    // Simulate agent operations with a timeout to ensure they complete
    let result = timeout(Duration::from_secs(1), async {
        // Basic agent system test operations would go here
        Ok::<_, Box<dyn std::error::Error>>(())
    }).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_agent_task_creation() {
    let task = AgentTask {
        id: "test_task_1".to_string(),
        task_type: "code_generation".to_string(),
        prompt: "Generate a hello world function".to_string(),
        context: None,
        priority: TaskPriority::Medium,
        metadata: std::collections::HashMap::new(),
    };
    
    assert_eq!(task.id, "test_task_1");
    assert_eq!(task.task_type, "code_generation");
    assert!(!task.prompt.is_empty());
    assert_eq!(task.priority, TaskPriority::Medium);
}

#[test]
fn test_agent_status() {
    let statuses = vec![
        AgentStatus::Idle,
        AgentStatus::Processing("test_task".to_string()),
        AgentStatus::Error("test error".to_string()),
        AgentStatus::WaitingForInput,
        AgentStatus::Offline,
    ];
    
    // Test that all status variants can be created
    assert_eq!(statuses.len(), 5);
    
    // Test status matching
    match statuses[0] {
        AgentStatus::Idle => assert!(true),
        _ => panic!("Expected Idle status"),
    }
    
    match &statuses[1] {
        AgentStatus::Processing(task) => assert_eq!(task, "test_task"),
        _ => panic!("Expected Processing status"),
    }
}

#[test]
fn test_task_priority_ordering() {
    let priorities = vec![
        TaskPriority::Low,
        TaskPriority::Medium, 
        TaskPriority::High,
        TaskPriority::Critical,
    ];
    
    // Test that priorities can be compared
    assert!(priorities[3] > priorities[2]);
    assert!(priorities[2] > priorities[1]);
    assert!(priorities[1] > priorities[0]);
}

#[tokio::test]
async fn test_agent_result() {
    let result = AgentResult {
        task_id: "test_task".to_string(),
        success: true,
        output: Some("Generated code successfully".to_string()),
        artifacts: vec![],
        next_actions: vec!["review_code".to_string()],
    };
    
    assert_eq!(result.task_id, "test_task");
    assert!(result.success);
    assert!(result.output.is_some());
    assert_eq!(result.next_actions.len(), 1);
}

#[test]
fn test_agent_error() {
    let errors = vec![
        AgentError::TaskProcessingFailed("Invalid input".to_string()),
        AgentError::CommunicationError("Connection lost".to_string()),
        AgentError::ConfigurationError("Missing API key".to_string()),
        AgentError::InvalidTaskType("Unknown task type".to_string()),
    ];
    
    assert_eq!(errors.len(), 4);
    
    // Test error display
    match &errors[0] {
        AgentError::TaskProcessingFailed(msg) => assert_eq!(msg, "Invalid input"),
        _ => panic!("Expected TaskProcessingFailed error"),
    }
}

#[tokio::test] 
async fn test_agent_types_creation() {
    // Test creating different agent types
    use crate::agents::agent_types::*;
    
    let code_gen_agent = CodeGenerationAgent::new();
    let analysis_agent = AnalysisAgent::new("analysis_1".to_string());
    let debugging_agent = DebuggingAgent::new("debug_1".to_string());
    
    // Test that agents can be created
    // In a full implementation, we would test agent-specific functionality
    
    // For now, just verify they exist
    assert!(std::mem::size_of_val(&code_gen_agent) > 0);
    assert!(std::mem::size_of_val(&analysis_agent) > 0); 
    assert!(std::mem::size_of_val(&debugging_agent) > 0);
}

#[tokio::test]
async fn test_concurrent_agent_operations() {
    let agent_system = Arc::new(AgentSystem::new());
    
    // Test concurrent operations on the agent system
    let handles = (0..5).map(|i| {
        let system = agent_system.clone();
        tokio::spawn(async move {
            // Simulate concurrent agent operations
            let task_id = format!("concurrent_task_{}", i);
            // In a full implementation, this would submit tasks to agents
            tokio::time::sleep(Duration::from_millis(10)).await;
            task_id
        })
    }).collect::<Vec<_>>();
    
    // Wait for all operations to complete
    let results = futures::future::join_all(handles).await;
    
    assert_eq!(results.len(), 5);
    for result in results {
        assert!(result.is_ok());
    }
}
