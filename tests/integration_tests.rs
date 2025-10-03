use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use devkit_env::testing::{
    fixtures::{AgentFixtures, CodegenFixtures, ConfigFixtures, ContextFixtures, ShellFixtures},
    mocks::{MockAgent, MockCodeGenerator, MockCodebaseAnalyzer, MockCommandExecutor},
    TestChannels, TestEnvironment, TestTime,
};
use devkit_env::{agents::*, codegen::*, config::*, context::*, shell::*};

/// End-to-end integration tests for complete workflows
mod end_to_end_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_development_workflow() {
        // Setup test environment
        let env = TestEnvironment::new().expect("Failed to create test environment");
        env.create_sample_project()
            .expect("Failed to create sample project");
        env.create_git_repo().expect("Failed to create git repo");

        // Initialize core systems
        let mut agent_manager = AgentManager::new(5);
        let mut code_analyzer = MockCodebaseAnalyzer::new();
        let mut code_generator = MockCodeGenerator::new();
        let mut shell_executor = MockCommandExecutor::new();

        // Register agents
        let analysis_agent = MockAgent::new("analyzer", "analysis");
        let codegen_agent = MockAgent::new("codegen", "generation");
        let debugging_agent = MockAgent::new("debugger", "debugging");

        agent_manager
            .register_agent("analyzer", Box::new(analysis_agent))
            .await
            .unwrap();
        agent_manager
            .register_agent("codegen", Box::new(codegen_agent))
            .await
            .unwrap();
        agent_manager
            .register_agent("debugger", Box::new(debugging_agent))
            .await
            .unwrap();

        // Step 1: Analyze the codebase
        let analysis_config = AnalysisConfig::default();
        let main_file = PathBuf::from("src/main.rs");
        let file_context = code_analyzer.analyze_file(&main_file, &analysis_config);
        assert!(file_context.is_ok());

        // Step 2: Create analysis task
        let analysis_task = AgentFixtures::create_analysis_task("src/main.rs");
        let analysis_result = agent_manager.submit_task("analyzer", analysis_task).await;
        assert!(analysis_result.is_ok());

        // Step 3: Generate code based on analysis
        let codebase_context =
            ContextFixtures::create_codebase_context(env.workspace_path().to_str().unwrap());
        let generation_config = CodegenFixtures::create_generation_config();
        let generated_code = code_generator.generate_code(
            "Add error handling to the main function",
            &codebase_context,
            &generation_config,
        );
        assert!(generated_code.is_ok());

        // Step 4: Create code generation task
        let codegen_task =
            AgentFixtures::create_code_generation_task("Add comprehensive error handling");
        let codegen_result = agent_manager.submit_task("codegen", codegen_task).await;
        assert!(codegen_result.is_ok());

        // Step 5: Execute shell commands for building and testing
        shell_executor.add_command_result(
            "cargo check",
            "Finished dev [unoptimized + debuginfo] target(s)",
            0,
        );
        shell_executor.add_command_result("cargo test", "test result: ok. 3 passed; 0 failed", 0);

        let check_result = shell_executor.execute_command("cargo", &["check"]);
        assert!(check_result.is_ok());
        assert!(check_result.unwrap().contains("Finished"));

        let test_result = shell_executor.execute_command("cargo", &["test"]);
        assert!(test_result.is_ok());
        assert!(test_result.unwrap().contains("ok"));

        // Step 6: Wait for all tasks to complete
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Verify all tasks completed successfully
        let analyzer_status = agent_manager.get_agent_status("analyzer").await.unwrap();
        let codegen_status = agent_manager.get_agent_status("codegen").await.unwrap();
        let debugger_status = agent_manager.get_agent_status("debugger").await.unwrap();

        // All agents should be idle after completing their tasks
        assert!(matches!(analyzer_status, AgentStatus::Idle));
        assert!(matches!(codegen_status, AgentStatus::Idle));
        assert!(matches!(debugger_status, AgentStatus::Idle));

        let executed_commands = shell_executor.get_executed_commands();
        assert_eq!(executed_commands.len(), 2);
        assert!(executed_commands.contains(&"cargo check".to_string()));
        assert!(executed_commands.contains(&"cargo test".to_string()));
    }

    #[tokio::test]
    async fn test_multi_agent_collaboration() {
        let mut agent_manager = AgentManager::new(3);
        let message_bus = AgentMessageBus::new();

        // Register collaborative agents
        let researcher_agent = MockAgent::new("researcher", "research");
        let architect_agent = MockAgent::new("architect", "architecture");
        let implementer_agent = MockAgent::new("implementer", "implementation");

        agent_manager
            .register_agent("researcher", Box::new(researcher_agent))
            .await
            .unwrap();
        agent_manager
            .register_agent("architect", Box::new(architect_agent))
            .await
            .unwrap();
        agent_manager
            .register_agent("implementer", Box::new(implementer_agent))
            .await
            .unwrap();

        // Setup message channels
        let (tx1, mut rx1) = TestChannels::create_agent_channels::<AgentMessage>();
        let (tx2, mut rx2) = TestChannels::create_agent_channels::<AgentMessage>();
        let (tx3, mut rx3) = TestChannels::create_agent_channels::<AgentMessage>();

        message_bus.subscribe("researcher", tx1).await;
        message_bus.subscribe("architect", tx2).await;
        message_bus.subscribe("implementer", tx3).await;

        // Track task completion
        let completed_tasks = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let completed_clone = std::sync::Arc::clone(&completed_tasks);

        agent_manager.set_completion_callback(Box::new(move |result| {
            completed_clone.lock().unwrap().push(result.task_id.clone());
        }));

        // Start collaborative workflow

        // 1. Research phase
        let research_task = AgentTask {
            id: "research_phase".to_string(),
            description: "Research best practices for authentication system".to_string(),
            priority: TaskPriority::High,
            created_at: std::time::SystemTime::now(),
            context: {
                let mut ctx = HashMap::new();
                ctx.insert("domain".to_string(), "authentication".to_string());
                ctx.insert("target".to_string(), "web_application".to_string());
                ctx
            },
        };

        agent_manager
            .submit_task("researcher", research_task)
            .await
            .unwrap();

        // 2. Architecture phase (depends on research)
        let architecture_task = AgentTask {
            id: "architecture_phase".to_string(),
            description: "Design authentication system architecture".to_string(),
            priority: TaskPriority::Normal,
            created_at: std::time::SystemTime::now(),
            context: {
                let mut ctx = HashMap::new();
                ctx.insert("depends_on".to_string(), "research_phase".to_string());
                ctx.insert("components".to_string(), "jwt,oauth,session".to_string());
                ctx
            },
        };

        // Send collaboration request
        let collab_message = AgentMessage {
            id: "collab_request_1".to_string(),
            from: "researcher".to_string(),
            to: "architect".to_string(),
            message_type: MessageType::CollaborationRequest,
            payload: "Research completed, ready for architecture design".to_string(),
            timestamp: std::time::SystemTime::now(),
        };

        message_bus.send_message(collab_message).await.unwrap();

        // Architect should receive the message
        let received_msg = tokio::time::timeout(Duration::from_millis(100), rx2.recv()).await;
        assert!(received_msg.is_ok());
        let msg = received_msg.unwrap().unwrap();
        assert_eq!(msg.from, "researcher");
        assert!(matches!(
            msg.message_type,
            MessageType::CollaborationRequest
        ));

        agent_manager
            .submit_task("architect", architecture_task)
            .await
            .unwrap();

        // 3. Implementation phase (depends on architecture)
        let implementation_task = AgentTask {
            id: "implementation_phase".to_string(),
            description: "Implement authentication system".to_string(),
            priority: TaskPriority::Normal,
            created_at: std::time::SystemTime::now(),
            context: {
                let mut ctx = HashMap::new();
                ctx.insert("depends_on".to_string(), "architecture_phase".to_string());
                ctx.insert("language".to_string(), "rust".to_string());
                ctx
            },
        };

        agent_manager
            .submit_task("implementer", implementation_task)
            .await
            .unwrap();

        // Wait for all tasks to complete
        tokio::time::sleep(Duration::from_millis(200)).await;

        let completed = completed_tasks.lock().unwrap();
        assert_eq!(completed.len(), 3);
        assert!(completed.contains(&"research_phase".to_string()));
        assert!(completed.contains(&"architecture_phase".to_string()));
        assert!(completed.contains(&"implementation_phase".to_string()));
    }

    #[tokio::test]
    async fn test_error_recovery_workflow() {
        let mut agent_manager = AgentManager::new(2);

        // Register a failing agent and a recovery agent
        let failing_agent = MockAgent::new("failer", "failing").with_failure(true);
        let recovery_agent = MockAgent::new("recovery", "recovery");

        agent_manager
            .register_agent("failer", Box::new(failing_agent))
            .await
            .unwrap();
        agent_manager
            .register_agent("recovery", Box::new(recovery_agent))
            .await
            .unwrap();

        let completed_tasks = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let failed_tasks = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));

        let completed_clone = std::sync::Arc::clone(&completed_tasks);
        let failed_clone = std::sync::Arc::clone(&failed_tasks);

        agent_manager.set_completion_callback(Box::new(move |result| {
            completed_clone.lock().unwrap().push(result.task_id.clone());
        }));

        agent_manager.set_error_callback(Box::new(move |task_id, _error| {
            failed_clone.lock().unwrap().push(task_id.clone());
        }));

        // Submit a task that will fail
        let failing_task = AgentTask {
            id: "failing_task".to_string(),
            description: "This task will fail".to_string(),
            priority: TaskPriority::Normal,
            created_at: std::time::SystemTime::now(),
            context: HashMap::new(),
        };

        agent_manager
            .submit_task("failer", failing_task)
            .await
            .unwrap();

        // Wait for failure
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Check that task failed
        let failed = failed_tasks.lock().unwrap();
        assert_eq!(failed.len(), 1);
        assert!(failed.contains(&"failing_task".to_string()));

        let failer_status = agent_manager.get_agent_status("failer").await.unwrap();
        assert!(matches!(failer_status, AgentStatus::Error(_)));

        // Submit recovery task
        let recovery_task = AgentTask {
            id: "recovery_task".to_string(),
            description: "Recover from the failed task".to_string(),
            priority: TaskPriority::High,
            created_at: std::time::SystemTime::now(),
            context: {
                let mut ctx = HashMap::new();
                ctx.insert("recover_from".to_string(), "failing_task".to_string());
                ctx.insert("strategy".to_string(), "retry_with_fallback".to_string());
                ctx
            },
        };

        agent_manager
            .submit_task("recovery", recovery_task)
            .await
            .unwrap();

        // Wait for recovery
        tokio::time::sleep(Duration::from_millis(100)).await;

        let completed = completed_tasks.lock().unwrap();
        assert_eq!(completed.len(), 1);
        assert!(completed.contains(&"recovery_task".to_string()));

        let recovery_status = agent_manager.get_agent_status("recovery").await.unwrap();
        assert!(matches!(recovery_status, AgentStatus::Idle));
    }

    #[tokio::test]
    async fn test_configuration_driven_workflow() {
        let env = TestEnvironment::new().expect("Failed to create test environment");
        let config_file = env
            .create_test_config()
            .expect("Failed to create test config");

        // Load configuration
        let mut config_manager = ConfigManager::new(Some(config_file)).unwrap();
        config_manager.load().unwrap();
        let config = config_manager.get_config();

        // Verify configuration loaded correctly
        assert_eq!(config.general.log_level, "debug");
        assert_eq!(config.agents.max_concurrent_agents, 3);
        assert_eq!(config.ui.theme, "dark");

        // Initialize systems with configuration
        let mut agent_manager = AgentManager::new(config.agents.max_concurrent_agents);

        // Register agents based on configuration
        for i in 1..=config.agents.max_concurrent_agents {
            let agent_name = format!("configured_agent_{}", i);
            let agent = MockAgent::new(&agent_name, "configured");
            agent_manager
                .register_agent(&agent_name, Box::new(agent))
                .await
                .unwrap();
        }

        // Verify all agents registered
        let agent_names = agent_manager.get_agent_names();
        assert_eq!(agent_names.len(), config.agents.max_concurrent_agents);

        // Create tasks based on configuration
        let tasks = vec![AgentTask {
            id: "config_task_1".to_string(),
            description: "Task configured from settings".to_string(),
            priority: match config.agents.default_agent_priority.as_str() {
                "low" => TaskPriority::Low,
                "normal" => TaskPriority::Normal,
                "high" => TaskPriority::High,
                "critical" => TaskPriority::Critical,
                _ => TaskPriority::Normal,
            },
            created_at: std::time::SystemTime::now(),
            context: {
                let mut ctx = HashMap::new();
                ctx.insert("config_driven".to_string(), "true".to_string());
                ctx.insert("log_level".to_string(), config.general.log_level.clone());
                ctx
            },
        }];

        // Submit tasks to agents
        for (i, task) in tasks.into_iter().enumerate() {
            let agent_name = format!(
                "configured_agent_{}",
                (i % config.agents.max_concurrent_agents) + 1
            );
            let result = agent_manager.submit_task(&agent_name, task).await;
            assert!(result.is_ok());
        }

        // Wait for task completion
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Verify agents handled tasks according to configuration
        for i in 1..=config.agents.max_concurrent_agents {
            let agent_name = format!("configured_agent_{}", i);
            let status = agent_manager.get_agent_status(&agent_name).await.unwrap();
            assert!(matches!(status, AgentStatus::Idle));
        }
    }

    #[tokio::test]
    async fn test_scalability_workflow() {
        // Test system behavior under load
        let mut agent_manager = AgentManager::new(10); // More agents for load testing

        // Register many agents
        for i in 1..=10 {
            let agent_name = format!("load_agent_{}", i);
            let agent =
                MockAgent::new(&agent_name, "load_test").with_delay(Duration::from_millis(20));
            agent_manager
                .register_agent(&agent_name, Box::new(agent))
                .await
                .unwrap();
        }

        let completed_tasks = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let completed_clone = std::sync::Arc::clone(&completed_tasks);

        agent_manager.set_completion_callback(Box::new(move |result| {
            completed_clone.lock().unwrap().push(result.task_id.clone());
        }));

        // Submit many tasks rapidly
        let task_count = 50;
        let start_time = std::time::Instant::now();

        for i in 1..=task_count {
            let task = AgentTask {
                id: format!("load_task_{}", i),
                description: format!("Load test task {}", i),
                priority: match i % 4 {
                    0 => TaskPriority::Low,
                    1 => TaskPriority::Normal,
                    2 => TaskPriority::High,
                    3 => TaskPriority::Critical,
                    _ => TaskPriority::Normal,
                },
                created_at: std::time::SystemTime::now(),
                context: {
                    let mut ctx = HashMap::new();
                    ctx.insert("load_test".to_string(), "true".to_string());
                    ctx.insert("batch".to_string(), (i / 10).to_string());
                    ctx
                },
            };

            let agent_name = format!("load_agent_{}", (i % 10) + 1);
            agent_manager.submit_task(&agent_name, task).await.unwrap();
        }

        let submission_time = start_time.elapsed();
        println!("Submitted {} tasks in {:?}", task_count, submission_time);

        // Wait for all tasks to complete with timeout
        let timeout_duration = Duration::from_secs(5);
        let completion_start = std::time::Instant::now();

        loop {
            let completed = completed_tasks.lock().unwrap();
            if completed.len() == task_count {
                break;
            }

            if completion_start.elapsed() > timeout_duration {
                panic!("Tasks did not complete within timeout");
            }

            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        let total_time = start_time.elapsed();
        let completed = completed_tasks.lock().unwrap();

        println!("Completed {} tasks in {:?}", completed.len(), total_time);
        assert_eq!(completed.len(), task_count);

        // Verify all agents are back to idle state
        for i in 1..=10 {
            let agent_name = format!("load_agent_{}", i);
            let status = agent_manager.get_agent_status(&agent_name).await.unwrap();
            assert!(matches!(status, AgentStatus::Idle));
        }

        // Verify performance metrics
        let avg_task_time = total_time / task_count as u32;
        println!("Average task completion time: {:?}", avg_task_time);

        // With 10 agents and 20ms delay per task, we should complete faster than sequential
        // Sequential would be 50 * 20ms = 1000ms, parallel should be much faster
        assert!(total_time < Duration::from_millis(800));
    }
}

/// Integration tests for cross-system communication
mod cross_system_tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_codegen_integration() {
        let mut agent_manager = AgentManager::new(2);
        let mut code_generator = MockCodeGenerator::new();
        let context = ContextFixtures::create_codebase_context("/test/project");
        let config = CodegenFixtures::create_generation_config();

        // Register codegen agent
        let codegen_agent = MockAgent::new("codegen", "code_generation");
        agent_manager
            .register_agent("codegen", Box::new(codegen_agent))
            .await
            .unwrap();

        // Create a task that involves code generation
        let task = AgentTask {
            id: "integrated_codegen".to_string(),
            description: "Generate authentication module with tests".to_string(),
            priority: TaskPriority::Normal,
            created_at: std::time::SystemTime::now(),
            context: {
                let mut ctx = HashMap::new();
                ctx.insert("language".to_string(), "rust".to_string());
                ctx.insert("module".to_string(), "auth".to_string());
                ctx.insert("include_tests".to_string(), "true".to_string());
                ctx
            },
        };

        // Submit task
        agent_manager.submit_task("codegen", task).await.unwrap();

        // Simultaneously generate code
        let generated_code = code_generator.generate_code(
            "Generate authentication module with comprehensive tests",
            &context,
            &config,
        );

        assert!(generated_code.is_ok());
        let code = generated_code.unwrap();
        assert!(code.contains("authentication module"));

        // Wait for agent task completion
        tokio::time::sleep(Duration::from_millis(100)).await;

        let status = agent_manager.get_agent_status("codegen").await.unwrap();
        assert!(matches!(status, AgentStatus::Idle));

        // Verify code generation and agent task integration
        let generated = code_generator.get_generated_code();
        assert_eq!(generated.len(), 1);
    }

    #[tokio::test]
    async fn test_context_agent_integration() {
        let mut agent_manager = AgentManager::new(2);
        let mut analyzer = MockCodebaseAnalyzer::new();

        // Register analysis agent
        let analysis_agent = MockAgent::new("analyzer", "analysis");
        agent_manager
            .register_agent("analyzer", Box::new(analysis_agent))
            .await
            .unwrap();

        // Analyze files
        let config = AnalysisConfig::default();
        let files_to_analyze = vec!["src/main.rs", "src/lib.rs", "src/utils.rs"];
        let mut file_contexts = Vec::new();

        for file_path in files_to_analyze {
            let result = analyzer.analyze_file(&PathBuf::from(file_path), &config);
            assert!(result.is_ok());
            file_contexts.push(result.unwrap());
        }

        // Create codebase context
        let codebase_context = analyzer.analyze_codebase(&file_contexts);
        assert!(codebase_context.is_ok());

        // Create analysis task based on context
        let context_data = codebase_context.unwrap();
        let analysis_task = AgentTask {
            id: "context_analysis".to_string(),
            description: "Analyze codebase structure and suggest improvements".to_string(),
            priority: TaskPriority::High,
            created_at: std::time::SystemTime::now(),
            context: {
                let mut ctx = HashMap::new();
                ctx.insert(
                    "files_analyzed".to_string(),
                    file_contexts.len().to_string(),
                );
                ctx.insert(
                    "dependencies_count".to_string(),
                    context_data.dependencies.len().to_string(),
                );
                ctx.insert(
                    "root_path".to_string(),
                    context_data.root_path.to_string_lossy().to_string(),
                );
                ctx
            },
        };

        agent_manager
            .submit_task("analyzer", analysis_task)
            .await
            .unwrap();

        // Wait for completion
        tokio::time::sleep(Duration::from_millis(100)).await;

        let status = agent_manager.get_agent_status("analyzer").await.unwrap();
        assert!(matches!(status, AgentStatus::Idle));

        // Verify integration
        let analyzed_files = analyzer.get_analyzed_files();
        assert_eq!(analyzed_files.len(), 3);
        assert!(analyzed_files.contains(&"src/main.rs".to_string()));
        assert!(analyzed_files.contains(&"src/lib.rs".to_string()));
        assert!(analyzed_files.contains(&"src/utils.rs".to_string()));
    }

    #[tokio::test]
    async fn test_shell_agent_integration() {
        let mut agent_manager = AgentManager::new(2);
        let mut shell_executor = MockCommandExecutor::new();

        // Setup shell commands
        shell_executor.add_command_result("git status", "On branch main\nnothing to commit", 0);
        shell_executor.add_command_result(
            "cargo build",
            "Finished dev [unoptimized + debuginfo] target(s)",
            0,
        );
        shell_executor.add_command_result("cargo test", "test result: ok. 5 passed; 0 failed", 0);

        // Register build agent
        let build_agent = MockAgent::new("builder", "build_and_test");
        agent_manager
            .register_agent("builder", Box::new(build_agent))
            .await
            .unwrap();

        // Create build and test task
        let build_task = AgentTask {
            id: "build_and_test".to_string(),
            description: "Build project and run tests".to_string(),
            priority: TaskPriority::High,
            created_at: std::time::SystemTime::now(),
            context: {
                let mut ctx = HashMap::new();
                ctx.insert(
                    "commands".to_string(),
                    "git status,cargo build,cargo test".to_string(),
                );
                ctx.insert("working_dir".to_string(), "/test/project".to_string());
                ctx
            },
        };

        agent_manager
            .submit_task("builder", build_task)
            .await
            .unwrap();

        // Execute shell commands as part of the workflow
        let commands = vec![
            ("git", vec!["status"]),
            ("cargo", vec!["build"]),
            ("cargo", vec!["test"]),
        ];

        for (cmd, args) in commands {
            let result = shell_executor.execute_command(cmd, &args);
            assert!(result.is_ok());
        }

        // Wait for agent task completion
        tokio::time::sleep(Duration::from_millis(100)).await;

        let status = agent_manager.get_agent_status("builder").await.unwrap();
        assert!(matches!(status, AgentStatus::Idle));

        // Verify shell integration
        let executed_commands = shell_executor.get_executed_commands();
        assert_eq!(executed_commands.len(), 3);
        assert!(executed_commands.contains(&"git status".to_string()));
        assert!(executed_commands.contains(&"cargo build".to_string()));
        assert!(executed_commands.contains(&"cargo test".to_string()));
    }
}

/// Performance and stress tests
mod performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrent_agent_performance() {
        let agent_count = 20;
        let tasks_per_agent = 10;
        let total_tasks = agent_count * tasks_per_agent;

        let mut agent_manager = AgentManager::new(agent_count);

        // Register many agents
        for i in 1..=agent_count {
            let agent_name = format!("perf_agent_{}", i);
            let agent =
                MockAgent::new(&agent_name, "performance").with_delay(Duration::from_millis(10)); // Small delay
            agent_manager
                .register_agent(&agent_name, Box::new(agent))
                .await
                .unwrap();
        }

        let completed_tasks = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let completed_clone = std::sync::Arc::clone(&completed_tasks);

        agent_manager.set_completion_callback(Box::new(move |result| {
            completed_clone.lock().unwrap().push(result.task_id.clone());
        }));

        let start_time = std::time::Instant::now();

        // Submit all tasks
        for agent_id in 1..=agent_count {
            for task_id in 1..=tasks_per_agent {
                let task = AgentTask {
                    id: format!("perf_task_{}_{}", agent_id, task_id),
                    description: format!(
                        "Performance test task {} for agent {}",
                        task_id, agent_id
                    ),
                    priority: TaskPriority::Normal,
                    created_at: std::time::SystemTime::now(),
                    context: {
                        let mut ctx = HashMap::new();
                        ctx.insert("agent_id".to_string(), agent_id.to_string());
                        ctx.insert("task_id".to_string(), task_id.to_string());
                        ctx
                    },
                };

                let agent_name = format!("perf_agent_{}", agent_id);
                agent_manager.submit_task(&agent_name, task).await.unwrap();
            }
        }

        // Wait for all tasks to complete
        let timeout = Duration::from_secs(10);
        let mut elapsed = Duration::ZERO;

        while elapsed < timeout {
            let completed = completed_tasks.lock().unwrap();
            if completed.len() == total_tasks {
                break;
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
            elapsed = start_time.elapsed();
        }

        let total_time = start_time.elapsed();
        let completed = completed_tasks.lock().unwrap();

        println!("Performance test results:");
        println!("  Agents: {}", agent_count);
        println!("  Tasks per agent: {}", tasks_per_agent);
        println!("  Total tasks: {}", total_tasks);
        println!("  Completed tasks: {}", completed.len());
        println!("  Total time: {:?}", total_time);
        println!(
            "  Tasks per second: {:.2}",
            completed.len() as f64 / total_time.as_secs_f64()
        );

        assert_eq!(completed.len(), total_tasks);

        // Performance expectations:
        // With 20 agents processing 10 tasks each (200 tasks total) with 10ms delay per task,
        // parallel execution should complete much faster than sequential (2 seconds)
        assert!(total_time < Duration::from_millis(1500));
    }

    #[tokio::test]
    async fn test_memory_efficiency() {
        // Test memory usage with large number of completed tasks
        let mut agent_manager = AgentManager::new(5);

        // Register agents
        for i in 1..=5 {
            let agent_name = format!("mem_agent_{}", i);
            let agent = MockAgent::new(&agent_name, "memory_test");
            agent_manager
                .register_agent(&agent_name, Box::new(agent))
                .await
                .unwrap();
        }

        let task_batches = 10;
        let tasks_per_batch = 50;

        for batch in 1..=task_batches {
            let batch_tasks = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
            let batch_clone = std::sync::Arc::clone(&batch_tasks);

            agent_manager.set_completion_callback(Box::new(move |result| {
                batch_clone.lock().unwrap().push(result.task_id.clone());
            }));

            // Submit batch of tasks
            for task_id in 1..=tasks_per_batch {
                let task = AgentTask {
                    id: format!("mem_task_{}_{}", batch, task_id),
                    description: format!("Memory test task {} in batch {}", task_id, batch),
                    priority: TaskPriority::Normal,
                    created_at: std::time::SystemTime::now(),
                    context: {
                        let mut ctx = HashMap::new();
                        ctx.insert("batch".to_string(), batch.to_string());
                        ctx.insert("large_data".to_string(), "x".repeat(1000)); // 1KB per task
                        ctx
                    },
                };

                let agent_name = format!("mem_agent_{}", (task_id % 5) + 1);
                agent_manager.submit_task(&agent_name, task).await.unwrap();
            }

            // Wait for batch completion
            let mut completed_count = 0;
            while completed_count < tasks_per_batch {
                tokio::time::sleep(Duration::from_millis(50)).await;
                completed_count = batch_tasks.lock().unwrap().len();
            }

            println!("Completed batch {} ({} tasks)", batch, tasks_per_batch);
        }

        // Verify all agents are still responsive after processing many tasks
        for i in 1..=5 {
            let agent_name = format!("mem_agent_{}", i);
            let status = agent_manager.get_agent_status(&agent_name).await.unwrap();
            assert!(matches!(status, AgentStatus::Idle));
        }

        println!("Memory efficiency test completed successfully");
        println!(
            "Processed {} batches of {} tasks each",
            task_batches, tasks_per_batch
        );
        println!("Total tasks: {}", task_batches * tasks_per_batch);
    }
}

/// Robustness and fault tolerance tests
mod robustness_tests {
    use super::*;

    #[tokio::test]
    async fn test_cascading_failure_recovery() {
        let mut agent_manager = AgentManager::new(4);

        // Register agents with different failure patterns
        let reliable_agent = MockAgent::new("reliable", "reliable");
        let intermittent_agent = MockAgent::new("intermittent", "intermittent").with_failure(true);
        let slow_agent = MockAgent::new("slow", "slow").with_delay(Duration::from_millis(200));
        let recovery_agent = MockAgent::new("recovery", "recovery");

        agent_manager
            .register_agent("reliable", Box::new(reliable_agent))
            .await
            .unwrap();
        agent_manager
            .register_agent("intermittent", Box::new(intermittent_agent))
            .await
            .unwrap();
        agent_manager
            .register_agent("slow", Box::new(slow_agent))
            .await
            .unwrap();
        agent_manager
            .register_agent("recovery", Box::new(recovery_agent))
            .await
            .unwrap();

        let completed_tasks = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let failed_tasks = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));

        let completed_clone = std::sync::Arc::clone(&completed_tasks);
        let failed_clone = std::sync::Arc::clone(&failed_tasks);

        agent_manager.set_completion_callback(Box::new(move |result| {
            completed_clone.lock().unwrap().push(result.task_id.clone());
        }));

        agent_manager.set_error_callback(Box::new(move |task_id, error| {
            failed_clone
                .lock()
                .unwrap()
                .push((task_id.clone(), error.to_string()));
        }));

        // Submit tasks to different agents
        let tasks = vec![
            ("reliable", "reliable_task_1"),
            ("intermittent", "failing_task_1"), // This will fail
            ("slow", "slow_task_1"),            // This will take time
            ("reliable", "reliable_task_2"),
            ("intermittent", "failing_task_2"), // This will also fail
            ("recovery", "recovery_task_1"),
        ];

        for (agent_name, task_id) in tasks {
            let task = AgentTask {
                id: task_id.to_string(),
                description: format!("Task for {}", agent_name),
                priority: TaskPriority::Normal,
                created_at: std::time::SystemTime::now(),
                context: {
                    let mut ctx = HashMap::new();
                    ctx.insert("target_agent".to_string(), agent_name.to_string());
                    ctx
                },
            };

            agent_manager.submit_task(agent_name, task).await.unwrap();
        }

        // Wait for processing to complete
        tokio::time::sleep(Duration::from_millis(500)).await;

        let completed = completed_tasks.lock().unwrap();
        let failed = failed_tasks.lock().unwrap();

        // Verify that some tasks completed and some failed as expected
        assert!(!completed.is_empty(), "Some tasks should have completed");
        assert!(!failed.is_empty(), "Some tasks should have failed");

        // Verify reliable agent completed its tasks
        assert!(completed.contains(&"reliable_task_1".to_string()));
        assert!(completed.contains(&"reliable_task_2".to_string()));
        assert!(completed.contains(&"recovery_task_1".to_string()));

        // Verify intermittent agent's tasks failed
        assert!(failed.iter().any(|(id, _)| id == "failing_task_1"));
        assert!(failed.iter().any(|(id, _)| id == "failing_task_2"));

        // Verify slow task eventually completed
        assert!(completed.contains(&"slow_task_1".to_string()));

        println!("Robustness test results:");
        println!("  Completed tasks: {:?}", *completed);
        println!("  Failed tasks: {:?}", *failed);

        // System should remain operational despite failures
        let reliable_status = agent_manager.get_agent_status("reliable").await.unwrap();
        let recovery_status = agent_manager.get_agent_status("recovery").await.unwrap();

        assert!(matches!(reliable_status, AgentStatus::Idle));
        assert!(matches!(recovery_status, AgentStatus::Idle));
    }
}
