//! Simple Enhanced Agent Progress Tracking Demo
//! 
//! This example demonstrates the enhanced agent system with progress tracking
//! in a simplified terminal application.

use devkit::agents::{
    Agent, AgentTask, AgentProgressTracker, EnhancedCodeGenAgent, CodeGenConfig,
    TaskPriority,
};
use devkit::ui::progress::ProgressManager;

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging for better output
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Starting Simple Enhanced Agent Demo");

    // Create progress manager and tracker
    let progress_manager = Arc::new(ProgressManager::new());
    let progress_tracker = Arc::new(AgentProgressTracker::new(progress_manager.clone()));

    // Create different agent configurations
    let agents = create_demo_agents(progress_tracker.clone()).await;

    // Demo tasks to execute
    let demo_tasks = vec![
        ("generate_code", "Create a simple HTTP client with error handling"),
        ("refactor", "Refactor legacy authentication system to use modern patterns"),
        ("documentation", "Generate comprehensive API documentation"),
        ("generate_code", "Implement async message queue processor"),
        ("optimization", "Optimize database queries and connection pooling"),
    ];

    info!("📋 Created {} demo tasks", demo_tasks.len());
    
    // Create a mutable copy of agents for processing
    let mut agents = agents;
    
    // Execute tasks with different agents
    for (i, (task_type, description)) in demo_tasks.iter().enumerate() {
        let task_number = i + 1;
        
        // Select agent based on task complexity - use indices to avoid borrowing issues
        let agent_index = match task_type {
            &"optimization" => {
                info!("🔧 Using thorough agent for optimization task");
                1 // thorough_agent index
            }
            &"generate_code" if description.contains("async") => {
                info!("🔧 Using thorough agent for complex async task");
                1 // thorough_agent index
            }
            _ => {
                info!("⚡ Using fast agent for standard task");
                0 // fast_agent index
            }
        };
        
        let (agent_name, mut agent) = (agents[agent_index].0.clone(), &mut agents[agent_index].1);

        let task = create_demo_task(task_type, description, task_number);
        
        info!("🎯 Task {}/{}: {} - {}", 
              task_number, demo_tasks.len(), task_type, description);
        info!("🤖 Assigned to: {}", agent_name);
        
        // Process task and measure time
        let start_time = std::time::Instant::now();
        
        match agent.process_task(task).await {
            Ok(result) => {
                let duration = start_time.elapsed();
                info!("✅ Task {} completed successfully in {:.2}s", 
                      task_number, duration.as_secs_f64());
                info!("📤 Generated {} artifacts", result.artifacts.len());
                
                if !result.next_actions.is_empty() {
                    info!("📝 Suggested next actions: {:?}", result.next_actions);
                }
                
                // Show some metrics
                let metrics = agent.get_metrics();
                info!("📊 Agent metrics - Success rate: {:.1}%, Tasks completed: {}", 
                      metrics.success_rate * 100.0, metrics.tasks_completed);
            }
            Err(e) => {
                error!("❌ Task {} failed: {}", task_number, e);
            }
        }
        
        // Short delay between tasks for better output readability
        if task_number < demo_tasks.len() {
            sleep(Duration::from_millis(500)).await;
        }
        
        println!(); // Add blank line for better readability
    }

    // Final summary
    info!("🎉 Demo completed successfully!");
    
    // Show final agent statistics
    for (name, agent) in &agents {
        let metrics = agent.get_metrics();
        info!("📈 {} final stats: {} tasks, {:.1}% success, avg {:.2}s per task",
              name, 
              metrics.tasks_completed + metrics.tasks_failed,
              metrics.success_rate * 100.0,
              metrics.average_task_duration);
    }

    Ok(())
}

/// Create demo agents with different configurations
async fn create_demo_agents(
    progress_tracker: Arc<AgentProgressTracker>,
) -> Vec<(String, EnhancedCodeGenAgent)> {
    vec![
        (
            "fast_agent".to_string(),
            EnhancedCodeGenAgent::with_progress_tracker(progress_tracker.clone())
                .with_config(CodeGenConfig {
                    enable_detailed_steps: true,
                    simulate_processing_time: true,
                    max_concurrent_tasks: 3,
                    quality_check_enabled: false, // Skip for speed
                    auto_optimization: false,
                }),
        ),
        (
            "thorough_agent".to_string(),
            EnhancedCodeGenAgent::with_progress_tracker(progress_tracker.clone())
                .with_config(CodeGenConfig {
                    enable_detailed_steps: true,
                    simulate_processing_time: true,
                    max_concurrent_tasks: 1,
                    quality_check_enabled: true,
                    auto_optimization: true, // Full optimization
                }),
        ),
        (
            "basic_agent".to_string(),
            EnhancedCodeGenAgent::new(), // No progress tracking
        ),
    ]
}

/// Create a demo task with appropriate priority
fn create_demo_task(task_type: &str, description: &str, task_number: usize) -> AgentTask {
    let mut task = AgentTask::new(
        task_type.to_string(),
        description.to_string(),
        serde_json::json!({
            "language": "rust",
            "target_file": format!("demo_output_{}.rs", task_number),
            "complexity": match task_type {
                "optimization" => "high",
                "generate_code" => "medium", 
                _ => "low"
            },
            "estimated_lines": match task_type {
                "optimization" => 200,
                "generate_code" => 150,
                "refactor" => 100,
                _ => 50
            }
        }),
    );

    // Set priority based on task type
    task.priority = match task_type {
        "optimization" => TaskPriority::High,
        "generate_code" => TaskPriority::Normal,
        _ => TaskPriority::Low,
    };

    task
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_demo_agent_creation() {
        let progress_manager = Arc::new(ProgressManager::new());
        let progress_tracker = Arc::new(AgentProgressTracker::new(progress_manager));
        let agents = create_demo_agents(progress_tracker).await;
        
        assert_eq!(agents.len(), 3);
        assert!(agents.iter().any(|(name, _)| name == "fast_agent"));
        assert!(agents.iter().any(|(name, _)| name == "thorough_agent"));
        assert!(agents.iter().any(|(name, _)| name == "basic_agent"));
    }

    #[test]
    fn test_demo_task_creation() {
        let task = create_demo_task("generate_code", "Test description", 1);
        
        assert_eq!(task.task_type, "generate_code");
        assert_eq!(task.description, "Test description");
        assert_eq!(task.priority, TaskPriority::Normal);
        assert!(task.context.is_object());
    }

    #[test]
    fn test_task_priority_assignment() {
        let high_task = create_demo_task("optimization", "Optimize queries", 1);
        let normal_task = create_demo_task("generate_code", "Create parser", 2);
        let low_task = create_demo_task("documentation", "Write docs", 3);
        
        assert_eq!(high_task.priority, TaskPriority::High);
        assert_eq!(normal_task.priority, TaskPriority::Normal);
        assert_eq!(low_task.priority, TaskPriority::Low);
    }
}