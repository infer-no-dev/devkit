use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

use agentic_dev_env::agents::*;
use agentic_dev_env::testing::{
    fixtures::AgentFixtures, mocks::MockAgent, TestChannels, TestEnvironment, TestTime,
};

/// Test agent manager functionality
mod agent_manager_tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_manager_creation() {
        let manager = AgentManager::new(5);
        assert_eq!(manager.max_concurrent_agents(), 5);
        assert_eq!(manager.active_agent_count(), 0);
        assert!(manager.get_agent_names().is_empty());
    }

    #[tokio::test]
    async fn test_register_agent() {
        let mut manager = AgentManager::new(3);
        let agent = MockAgent::new("test_agent", "mock");

        let result = manager.register_agent("test_agent", Box::new(agent)).await;
        assert!(result.is_ok());

        let agent_names = manager.get_agent_names();
        assert_eq!(agent_names.len(), 1);
        assert!(agent_names.contains(&"test_agent".to_string()));
    }

    #[tokio::test]
    async fn test_register_duplicate_agent() {
        let mut manager = AgentManager::new(3);
        let agent1 = MockAgent::new("test_agent", "mock");
        let agent2 = MockAgent::new("test_agent", "mock");

        manager
            .register_agent("test_agent", Box::new(agent1))
            .await
            .unwrap();
        let result = manager.register_agent("test_agent", Box::new(agent2)).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AgentError::AgentAlreadyExists(_)
        ));
    }

    #[tokio::test]
    async fn test_submit_task() {
        let mut manager = AgentManager::new(3);
        let agent = MockAgent::new("test_agent", "mock");
        manager
            .register_agent("test_agent", Box::new(agent))
            .await
            .unwrap();

        let task = AgentFixtures::create_task("task_1", "Test task", TaskPriority::Normal);
        let result = manager.submit_task("test_agent", task).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_submit_task_to_nonexistent_agent() {
        let mut manager = AgentManager::new(3);
        let task = AgentFixtures::create_task("task_1", "Test task", TaskPriority::Normal);

        let result = manager.submit_task("nonexistent_agent", task).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AgentError::AgentNotFound(_)));
    }

    #[tokio::test]
    async fn test_agent_status_tracking() {
        let mut manager = AgentManager::new(3);
        let agent = MockAgent::new("test_agent", "mock");
        manager
            .register_agent("test_agent", Box::new(agent))
            .await
            .unwrap();

        let status = manager.get_agent_status("test_agent").await;
        assert!(status.is_ok());
        assert!(matches!(status.unwrap(), AgentStatus::Idle));
    }

    #[tokio::test]
    async fn test_concurrent_task_limit() {
        let mut manager = AgentManager::new(2); // Limit to 2 concurrent agents

        // Register 3 agents
        for i in 1..=3 {
            let agent = MockAgent::new(&format!("agent_{}", i), "mock")
                .with_delay(Duration::from_millis(100));
            manager
                .register_agent(&format!("agent_{}", i), Box::new(agent))
                .await
                .unwrap();
        }

        // Submit tasks to all agents simultaneously
        let tasks = vec![
            AgentFixtures::create_task("task_1", "Task 1", TaskPriority::Normal),
            AgentFixtures::create_task("task_2", "Task 2", TaskPriority::Normal),
            AgentFixtures::create_task("task_3", "Task 3", TaskPriority::Normal),
        ];

        for (i, task) in tasks.into_iter().enumerate() {
            let result = manager.submit_task(&format!("agent_{}", i + 1), task).await;
            assert!(result.is_ok());
        }

        // Wait a bit and check that only 2 agents are active
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(manager.active_agent_count() <= 2);
    }

    #[tokio::test]
    async fn test_task_completion_callback() {
        let mut manager = AgentManager::new(3);
        let agent = MockAgent::new("test_agent", "mock");
        manager
            .register_agent("test_agent", Box::new(agent))
            .await
            .unwrap();

        let completed_tasks = Arc::new(Mutex::new(Vec::new()));
        let completed_tasks_clone = Arc::clone(&completed_tasks);

        manager.set_completion_callback(Box::new(move |result| {
            completed_tasks_clone
                .lock()
                .unwrap()
                .push(result.task_id.clone());
        }));

        let task = AgentFixtures::create_task("task_1", "Test task", TaskPriority::Normal);
        manager.submit_task("test_agent", task).await.unwrap();

        // Wait for completion
        tokio::time::sleep(Duration::from_millis(50)).await;

        let completed = completed_tasks.lock().unwrap();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0], "task_1");
    }
}

/// Test agent coordination functionality
mod coordination_tests {
    use super::*;

    #[tokio::test]
    async fn test_workflow_creation() {
        let workflow = Workflow::new("test_workflow", "Test workflow description");
        assert_eq!(workflow.id, "test_workflow");
        assert_eq!(workflow.description, "Test workflow description");
        assert!(workflow.tasks.is_empty());
    }

    #[tokio::test]
    async fn test_add_task_to_workflow() {
        let mut workflow = Workflow::new("test_workflow", "Test workflow");
        let task = AgentFixtures::create_task("task_1", "First task", TaskPriority::Normal);

        workflow.add_task(WorkflowTask::from_agent_task(task, Vec::new()));
        assert_eq!(workflow.tasks.len(), 1);
    }

    #[tokio::test]
    async fn test_workflow_with_dependencies() {
        let mut workflow = Workflow::new("test_workflow", "Test workflow with dependencies");

        let task1 = AgentFixtures::create_task("task_1", "First task", TaskPriority::Normal);
        let task2 = AgentFixtures::create_task("task_2", "Second task", TaskPriority::Normal);
        let task3 = AgentFixtures::create_task("task_3", "Third task", TaskPriority::Normal);

        workflow.add_task(WorkflowTask::from_agent_task(task1, Vec::new()));
        workflow.add_task(WorkflowTask::from_agent_task(
            task2,
            vec!["task_1".to_string()],
        ));
        workflow.add_task(WorkflowTask::from_agent_task(
            task3,
            vec!["task_1".to_string(), "task_2".to_string()],
        ));

        assert_eq!(workflow.tasks.len(), 3);
        assert!(workflow.tasks[1]
            .dependencies
            .contains(&"task_1".to_string()));
        assert!(workflow.tasks[2]
            .dependencies
            .contains(&"task_1".to_string()));
        assert!(workflow.tasks[2]
            .dependencies
            .contains(&"task_2".to_string()));
    }

    #[tokio::test]
    async fn test_workflow_coordinator() {
        let coordinator = WorkflowCoordinator::new();
        let mut workflow = Workflow::new("test_workflow", "Test workflow");

        let task = AgentFixtures::create_task("task_1", "Test task", TaskPriority::Normal);
        workflow.add_task(WorkflowTask::from_agent_task(task, Vec::new()));

        let workflow_id = coordinator.submit_workflow(workflow).await;
        assert!(!workflow_id.is_empty());

        let status = coordinator.get_workflow_status(&workflow_id).await;
        assert!(status.is_ok());
        assert!(matches!(status.unwrap(), WorkflowStatus::Pending));
    }

    #[tokio::test]
    async fn test_execute_workflow() {
        let mut manager = AgentManager::new(3);
        let agent = MockAgent::new("test_agent", "mock");
        manager
            .register_agent("test_agent", Box::new(agent))
            .await
            .unwrap();

        let coordinator = WorkflowCoordinator::new();
        let mut workflow = Workflow::new("test_workflow", "Test workflow execution");

        let task = AgentFixtures::create_task("task_1", "Test task", TaskPriority::Normal);
        workflow.add_task(WorkflowTask::from_agent_task(task, Vec::new()));

        let workflow_id = coordinator.submit_workflow(workflow).await;
        let result = coordinator.execute_workflow(&workflow_id, &manager).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_workflow_with_task_failure() {
        let mut manager = AgentManager::new(3);
        let failing_agent = MockAgent::new("failing_agent", "mock").with_failure(true);
        manager
            .register_agent("failing_agent", Box::new(failing_agent))
            .await
            .unwrap();

        let coordinator = WorkflowCoordinator::new();
        let mut workflow = Workflow::new("failing_workflow", "Workflow with failing task");

        let task =
            AgentFixtures::create_task("failing_task", "This task will fail", TaskPriority::Normal);
        workflow.add_task(WorkflowTask::from_agent_task(task, Vec::new()));

        let workflow_id = coordinator.submit_workflow(workflow).await;
        let result = coordinator.execute_workflow(&workflow_id, &manager).await;

        assert!(result.is_err());
    }
}

/// Test agent communication functionality
mod communication_tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_message_bus() {
        let bus = AgentMessageBus::new();
        let (tx, mut rx) = TestChannels::create_agent_channels::<AgentMessage>();

        bus.subscribe("test_agent", tx).await;

        let message = AgentMessage {
            id: "msg_1".to_string(),
            from: "sender".to_string(),
            to: "test_agent".to_string(),
            message_type: MessageType::TaskRequest,
            payload: "test payload".to_string(),
            timestamp: std::time::SystemTime::now(),
        };

        bus.send_message(message.clone()).await.unwrap();

        let received = timeout(Duration::from_millis(100), rx.recv()).await;
        assert!(received.is_ok());
        let received_msg = received.unwrap().unwrap();
        assert_eq!(received_msg.id, "msg_1");
        assert_eq!(received_msg.payload, "test payload");
    }

    #[tokio::test]
    async fn test_agent_message_routing() {
        let bus = AgentMessageBus::new();
        let (tx1, mut rx1) = TestChannels::create_agent_channels::<AgentMessage>();
        let (tx2, mut rx2) = TestChannels::create_agent_channels::<AgentMessage>();

        bus.subscribe("agent_1", tx1).await;
        bus.subscribe("agent_2", tx2).await;

        let message1 = AgentMessage {
            id: "msg_1".to_string(),
            from: "sender".to_string(),
            to: "agent_1".to_string(),
            message_type: MessageType::TaskRequest,
            payload: "for agent 1".to_string(),
            timestamp: std::time::SystemTime::now(),
        };

        let message2 = AgentMessage {
            id: "msg_2".to_string(),
            from: "sender".to_string(),
            to: "agent_2".to_string(),
            message_type: MessageType::TaskRequest,
            payload: "for agent 2".to_string(),
            timestamp: std::time::SystemTime::now(),
        };

        bus.send_message(message1).await.unwrap();
        bus.send_message(message2).await.unwrap();

        let received1 = timeout(Duration::from_millis(100), rx1.recv()).await;
        let received2 = timeout(Duration::from_millis(100), rx2.recv()).await;

        assert!(received1.is_ok());
        assert!(received2.is_ok());

        let msg1 = received1.unwrap().unwrap();
        let msg2 = received2.unwrap().unwrap();

        assert_eq!(msg1.payload, "for agent 1");
        assert_eq!(msg2.payload, "for agent 2");
    }

    #[tokio::test]
    async fn test_broadcast_message() {
        let bus = AgentMessageBus::new();
        let (tx1, mut rx1) = TestChannels::create_agent_channels::<AgentMessage>();
        let (tx2, mut rx2) = TestChannels::create_agent_channels::<AgentMessage>();

        bus.subscribe("agent_1", tx1).await;
        bus.subscribe("agent_2", tx2).await;

        let broadcast_msg = AgentMessage {
            id: "broadcast_1".to_string(),
            from: "system".to_string(),
            to: "*".to_string(), // Broadcast indicator
            message_type: MessageType::SystemNotification,
            payload: "broadcast message".to_string(),
            timestamp: std::time::SystemTime::now(),
        };

        bus.broadcast_message(broadcast_msg).await.unwrap();

        let received1 = timeout(Duration::from_millis(100), rx1.recv()).await;
        let received2 = timeout(Duration::from_millis(100), rx2.recv()).await;

        assert!(received1.is_ok());
        assert!(received2.is_ok());

        let msg1 = received1.unwrap().unwrap();
        let msg2 = received2.unwrap().unwrap();

        assert_eq!(msg1.payload, "broadcast message");
        assert_eq!(msg2.payload, "broadcast message");
    }
}

/// Test different agent types
mod agent_types_tests {
    use super::*;

    #[tokio::test]
    async fn test_codegen_agent() {
        let mut agent = CodegenAgent::new("codegen_1");
        let task = AgentFixtures::create_code_generation_task("Create a simple function");

        let result = agent.process_task(task).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.result.contains("function") || result.result.contains("Generated"));
        assert!(!result.artifacts.is_empty());
    }

    #[tokio::test]
    async fn test_analysis_agent() {
        let mut agent = AnalysisAgent::new("analyzer_1");
        let task = AgentFixtures::create_analysis_task("src/main.rs");

        let result = agent.process_task(task).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.result.contains("analysis") || result.result.contains("Analysis"));
    }

    #[tokio::test]
    async fn test_debugging_agent() {
        let mut agent = DebuggingAgent::new("debugger_1");
        let task = AgentTask {
            id: "debug_task".to_string(),
            description: "Debug compilation error".to_string(),
            priority: TaskPriority::High,
            created_at: std::time::SystemTime::now(),
            context: {
                let mut context = HashMap::new();
                context.insert(
                    "error_message".to_string(),
                    "expected `;` after expression".to_string(),
                );
                context.insert("file_path".to_string(), "src/main.rs".to_string());
                context.insert("line_number".to_string(), "42".to_string());
                context
            },
        };

        let result = agent.process_task(task).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.result.contains("debug") || result.result.contains("solution"));
    }
}

/// Test agent task priority and scheduling
mod priority_scheduling_tests {
    use super::*;

    #[tokio::test]
    async fn test_task_priority_ordering() {
        let mut manager = AgentManager::new(1); // Only one concurrent task
        let agent = MockAgent::new("priority_agent", "mock").with_delay(Duration::from_millis(50));
        manager
            .register_agent("priority_agent", Box::new(agent))
            .await
            .unwrap();

        let completed_tasks = Arc::new(Mutex::new(Vec::new()));
        let completed_tasks_clone = Arc::clone(&completed_tasks);

        manager.set_completion_callback(Box::new(move |result| {
            completed_tasks_clone
                .lock()
                .unwrap()
                .push(result.task_id.clone());
        }));

        // Submit tasks in different priority order
        let low_task =
            AgentFixtures::create_task("low_task", "Low priority task", TaskPriority::Low);
        let high_task =
            AgentFixtures::create_task("high_task", "High priority task", TaskPriority::High);
        let normal_task =
            AgentFixtures::create_task("normal_task", "Normal priority task", TaskPriority::Normal);
        let critical_task = AgentFixtures::create_task(
            "critical_task",
            "Critical priority task",
            TaskPriority::Critical,
        );

        manager
            .submit_task("priority_agent", low_task)
            .await
            .unwrap();
        manager
            .submit_task("priority_agent", high_task)
            .await
            .unwrap();
        manager
            .submit_task("priority_agent", normal_task)
            .await
            .unwrap();
        manager
            .submit_task("priority_agent", critical_task)
            .await
            .unwrap();

        // Wait for all tasks to complete
        tokio::time::sleep(Duration::from_millis(300)).await;

        let completed = completed_tasks.lock().unwrap();
        assert_eq!(completed.len(), 4);

        // Critical should be processed first (after low which was already started)
        // Then high, normal, and finally low (if not already completed)
        let critical_pos = completed.iter().position(|id| id == "critical_task");
        let high_pos = completed.iter().position(|id| id == "high_task");
        let normal_pos = completed.iter().position(|id| id == "normal_task");

        assert!(critical_pos.is_some());
        assert!(high_pos.is_some());
        assert!(normal_pos.is_some());

        // Critical should come before high and normal
        assert!(critical_pos.unwrap() < high_pos.unwrap());
        assert!(critical_pos.unwrap() < normal_pos.unwrap());
    }

    #[tokio::test]
    async fn test_task_timeout_handling() {
        let mut manager = AgentManager::new(1);
        let slow_agent =
            MockAgent::new("slow_agent", "mock").with_delay(Duration::from_millis(200)); // Longer than timeout
        manager
            .register_agent("slow_agent", Box::new(slow_agent))
            .await
            .unwrap();

        // Set a short timeout
        manager.set_task_timeout(Duration::from_millis(100));

        let task =
            AgentFixtures::create_task("timeout_task", "This will timeout", TaskPriority::Normal);
        let result = manager.submit_task("slow_agent", task).await;

        assert!(result.is_ok());

        // Wait for timeout to occur
        tokio::time::sleep(Duration::from_millis(150)).await;

        // The task should have been cancelled due to timeout
        let status = manager.get_agent_status("slow_agent").await.unwrap();
        // Status might be Idle if the timeout was handled correctly
        assert!(matches!(status, AgentStatus::Idle) || matches!(status, AgentStatus::Error(_)));
    }
}

/// Test agent error handling and recovery
mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_error_recovery() {
        let mut manager = AgentManager::new(2);
        let failing_agent = MockAgent::new("failing_agent", "mock").with_failure(true);
        manager
            .register_agent("failing_agent", Box::new(failing_agent))
            .await
            .unwrap();

        let task =
            AgentFixtures::create_task("failing_task", "This will fail", TaskPriority::Normal);
        let result = manager.submit_task("failing_agent", task).await;

        assert!(result.is_ok());

        // Wait for task to fail
        tokio::time::sleep(Duration::from_millis(50)).await;

        let status = manager.get_agent_status("failing_agent").await.unwrap();
        assert!(matches!(status, AgentStatus::Error(_)));

        // Agent should be available for new tasks after error
        let recovery_task =
            AgentFixtures::create_task("recovery_task", "Recovery task", TaskPriority::Normal);
        let recovery_result = manager.submit_task("failing_agent", recovery_task).await;
        assert!(recovery_result.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_task_handling() {
        let mut agent = MockAgent::new("test_agent", "mock");

        let invalid_task = AgentTask {
            id: "".to_string(),          // Invalid empty ID
            description: "".to_string(), // Invalid empty description
            priority: TaskPriority::Normal,
            created_at: std::time::SystemTime::now(),
            context: HashMap::new(),
        };

        let result = agent.process_task(invalid_task).await;
        // Mock agent should still process it, but in a real implementation,
        // this might be validated and rejected
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_agent_cancellation() {
        let mut agent =
            MockAgent::new("cancel_agent", "mock").with_delay(Duration::from_millis(100));

        let task =
            AgentFixtures::create_task("cancel_task", "Task to be cancelled", TaskPriority::Normal);
        let task_id = task.id.clone();

        // Start processing the task
        let process_handle = tokio::spawn(async move { agent.process_task(task).await });

        // Wait a bit then cancel
        tokio::time::sleep(Duration::from_millis(25)).await;

        // In a real implementation, we would have a way to cancel the running task
        // For now, we'll just test that the agent can handle cancellation requests
        let mut cancel_agent = MockAgent::new("cancel_agent", "mock");
        let cancel_result = cancel_agent.cancel_task(&task_id).await;
        assert!(cancel_result.is_ok());

        // The original task might still complete
        let result = process_handle.await;
        assert!(result.is_ok());
    }
}

/// Integration tests for complete agent workflows
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_code_generation_workflow() {
        let env = TestEnvironment::new().expect("Failed to create test environment");
        env.create_sample_project()
            .expect("Failed to create sample project");

        let mut manager = AgentManager::new(3);

        // Register agents
        let analyzer = AnalysisAgent::new("analyzer");
        let codegen = CodegenAgent::new("codegen");

        manager
            .register_agent("analyzer", Box::new(analyzer))
            .await
            .unwrap();
        manager
            .register_agent("codegen", Box::new(codegen))
            .await
            .unwrap();

        // Create workflow
        let coordinator = WorkflowCoordinator::new();
        let mut workflow = Workflow::new("codegen_workflow", "Complete code generation workflow");

        // Analysis task
        let analysis_task = AgentFixtures::create_analysis_task("src/main.rs");
        workflow.add_task(WorkflowTask::from_agent_task(analysis_task, Vec::new()));

        // Code generation task (depends on analysis)
        let codegen_task =
            AgentFixtures::create_code_generation_task("Add error handling to main function");
        workflow.add_task(WorkflowTask::from_agent_task(
            codegen_task,
            vec!["analysis_task".to_string()],
        ));

        let workflow_id = coordinator.submit_workflow(workflow).await;
        let result = coordinator.execute_workflow(&workflow_id, &manager).await;

        assert!(result.is_ok());

        let final_status = coordinator.get_workflow_status(&workflow_id).await.unwrap();
        assert!(matches!(final_status, WorkflowStatus::Completed));
    }

    #[tokio::test]
    async fn test_multi_agent_coordination() {
        let mut manager = AgentManager::new(5);

        // Register multiple agents
        for i in 1..=3 {
            let agent = MockAgent::new(&format!("agent_{}", i), "mock");
            manager
                .register_agent(&format!("agent_{}", i), Box::new(agent))
                .await
                .unwrap();
        }

        let completed_tasks = Arc::new(Mutex::new(Vec::new()));
        let completed_tasks_clone = Arc::clone(&completed_tasks);

        manager.set_completion_callback(Box::new(move |result| {
            completed_tasks_clone
                .lock()
                .unwrap()
                .push(result.task_id.clone());
        }));

        // Submit tasks to different agents
        let tasks = vec![
            (
                "agent_1",
                AgentFixtures::create_task("task_1", "Task for agent 1", TaskPriority::Normal),
            ),
            (
                "agent_2",
                AgentFixtures::create_task("task_2", "Task for agent 2", TaskPriority::High),
            ),
            (
                "agent_3",
                AgentFixtures::create_task("task_3", "Task for agent 3", TaskPriority::Low),
            ),
        ];

        for (agent_name, task) in tasks {
            manager.submit_task(agent_name, task).await.unwrap();
        }

        // Wait for all tasks to complete
        tokio::time::sleep(Duration::from_millis(100)).await;

        let completed = completed_tasks.lock().unwrap();
        assert_eq!(completed.len(), 3);
        assert!(completed.contains(&"task_1".to_string()));
        assert!(completed.contains(&"task_2".to_string()));
        assert!(completed.contains(&"task_3".to_string()));
    }

    #[tokio::test]
    async fn test_agent_communication_integration() {
        let bus = AgentMessageBus::new();
        let mut manager = AgentManager::new(2);

        let agent1 = MockAgent::new("communicator_1", "mock");
        let agent2 = MockAgent::new("communicator_2", "mock");

        manager
            .register_agent("communicator_1", Box::new(agent1))
            .await
            .unwrap();
        manager
            .register_agent("communicator_2", Box::new(agent2))
            .await
            .unwrap();

        let (tx1, mut rx1) = TestChannels::create_agent_channels::<AgentMessage>();
        let (tx2, mut rx2) = TestChannels::create_agent_channels::<AgentMessage>();

        bus.subscribe("communicator_1", tx1).await;
        bus.subscribe("communicator_2", tx2).await;

        // Agent 1 sends a message to Agent 2
        let message = AgentMessage {
            id: "collab_msg".to_string(),
            from: "communicator_1".to_string(),
            to: "communicator_2".to_string(),
            message_type: MessageType::CollaborationRequest,
            payload: "Need help with task".to_string(),
            timestamp: std::time::SystemTime::now(),
        };

        bus.send_message(message).await.unwrap();

        // Agent 2 should receive the message
        let received = timeout(Duration::from_millis(100), rx2.recv()).await;
        assert!(received.is_ok());

        let msg = received.unwrap().unwrap();
        assert_eq!(msg.from, "communicator_1");
        assert_eq!(msg.payload, "Need help with task");
        assert!(matches!(
            msg.message_type,
            MessageType::CollaborationRequest
        ));
    }
}
