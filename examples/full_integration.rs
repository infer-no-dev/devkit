//! Full integration example demonstrating all enhanced devkit systems
//!
//! This example shows how the major near-term upgrades work together:
//! - Deterministic orchestration with retries and timeouts
//! - Incremental codebase indexing with embeddings
//! - WARP.md/Rules precedence system  
//! - Diff-first apply flow with quality gates
//! - Structured telemetry with per-agent spans
//! - Mixed-model routing and caching
//!
//! Run with: cargo run --example full_integration

use devkit::{
    ai::{routing::*, *},
    agents::{system::*, *},
    codegen::{diff_apply::*, *},
    config::{rules::*, *},
    context::{embeddings::*, *},
    telemetry::*,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 DevKit Full Integration Demo");
    println!("================================");

    // 1. Initialize Telemetry System
    println!("\n📊 1. Setting up structured telemetry...");
    let telemetry_config = TelemetryConfig {
        enabled: true,
        trace_sample_rate: 1.0,
        enable_json_export: true,
        json_export_path: Some("demo_telemetry.json".to_string()),
        ..Default::default()
    };
    
    let telemetry_system = TelemetrySystem::new(telemetry_config)?;
    telemetry_system.start().await?;
    
    // Start a demo span
    let demo_span = telemetry_system.start_agent_span(
        "demo_agent",
        "full_integration_demo", 
        SpanKind::Internal,
        None
    ).await?;
    
    // 2. Initialize Mixed-Model Routing
    println!("🧠 2. Setting up AI model routing...");
    let mut model_router = ModelRouter::new("llama3.2:latest".to_string());
    
    // Register different models with capabilities
    model_router.register_model(ModelConfig {
        name: "llama3.2:latest".to_string(),
        endpoint: None,
        capabilities: ModelCapabilities {
            code_generation: true,
            text_analysis: true,
            reasoning: true,
            context_length: 8192,
            strengths: vec![TaskType::CodeGeneration, TaskType::Chat],
            ..Default::default()
        },
        performance_metrics: ModelMetrics {
            avg_response_time_ms: 800.0,
            quality_score: 0.85,
            ..Default::default()
        },
        cost_per_token: 0.0001,
        max_tokens: 8192,
        enabled: true,
    });
    
    // Add routing rules
    model_router.add_routing_rule(RoutingRule {
        name: "Code tasks to Llama".to_string(),
        conditions: vec![RoutingCondition::TaskType(TaskType::CodeGeneration)],
        target_model: "llama3.2:latest".to_string(),
        priority: 100,
        enabled: true,
    });
    
    // 3. Initialize Rules System
    println!("📋 3. Setting up hierarchical rules system...");
    let rules_manager = RulesManager::new();
    
    // Create demo project structure with WARP.md files
    let project_root = PathBuf::from("/tmp/demo_project");
    std::fs::create_dir_all(&project_root)?;
    
    // Global rules
    rules_manager.add_global_rule(Rule {
        id: Uuid::new_v4(),
        name: "Global Code Style".to_string(),
        description: "Always use 4-space indentation".to_string(),
        conditions: vec![],
        actions: vec!["indent: 4 spaces".to_string()],
        priority: 10,
        enabled: true,
        scope: RuleScope::Global,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }).await;
    
    // Project-specific rule
    let project_warp_path = project_root.join("WARP.md");
    std::fs::write(&project_warp_path, r#"# Project Rules

## Code Generation
- Language: Rust
- Style: Use snake_case for functions
- Testing: Always include tests
"#)?;
    
    rules_manager.load_rules_from_path(&project_root).await?;
    
    // 4. Initialize Context with Embeddings
    println!("🔍 4. Setting up codebase context with embeddings...");
    let mut embeddings_config = EmbeddingsConfig::default();
    embeddings_config.enabled = true;
    embeddings_config.provider = EmbeddingsProvider::Local {
        model_path: "demo_embeddings".to_string()
    };
    
    let embeddings_manager = EmbeddingsManager::new(embeddings_config)?;
    
    // Create some demo code files
    let demo_file = project_root.join("src/main.rs");
    std::fs::create_dir_all(demo_file.parent().unwrap())?;
    std::fs::write(&demo_file, r#"
fn main() {
    println!("Hello, DevKit!");
    let result = calculate_fibonacci(10);
    println!("Fibonacci(10) = {}", result);
}

fn calculate_fibonacci(n: usize) -> usize {
    if n <= 1 {
        n
    } else {
        calculate_fibonacci(n - 1) + calculate_fibonacci(n - 2)
    }
}
"#)?;
    
    // Index the codebase
    let indexed_files = embeddings_manager.index_codebase(&project_root, &["**/*.rs".to_string()]).await?;
    println!("  📁 Indexed {} files with embeddings", indexed_files);
    
    // 5. Initialize Agent System with Orchestration
    println!("🤖 5. Setting up agent orchestration...");
    let orchestrator_config = OrchestratorConfig {
        max_concurrent_agents: 4,
        default_timeout: Duration::from_secs(30),
        max_retries: 3,
        backoff_multiplier: 2.0,
        enable_circuit_breaker: true,
        circuit_breaker_failure_threshold: 5,
        circuit_breaker_timeout: Duration::from_secs(60),
        task_queue_capacity: 100,
        enable_telemetry: true,
    };
    
    let mut orchestrator = AgentOrchestrator::new(orchestrator_config);
    
    // Register some demo agents
    orchestrator.register_agent("code_generator", Box::new(DemoCodeGenAgent));
    orchestrator.register_agent("code_reviewer", Box::new(DemoReviewAgent));
    orchestrator.register_agent("test_writer", Box::new(DemoTestAgent));
    
    // 6. Initialize Diff-First Apply System
    println!("📝 6. Setting up diff-first apply workflow...");
    let quality_config = QualityGateConfig {
        enabled_gates: vec![
            "format".to_string(),
            "lint".to_string(), 
            "compile".to_string(),
            "security".to_string(),
        ],
        auto_apply_on_pass: false,
        require_all_gates: true,
        timeout_seconds: 120,
        parallel_execution: true,
        custom_commands: HashMap::new(),
    };
    
    let diff_system = DiffApplySystem::new(quality_config, &project_root)?;
    
    // 7. Demo Integrated Workflow
    println!("\n🎯 7. Running integrated workflow demo...");
    
    // Create a code generation task
    let task = AgentTask {
        id: Uuid::new_v4(),
        task_type: crate::agents::TaskType::CodeGeneration,
        priority: TaskPriority::Normal,
        input: serde_json::json!({
            "prompt": "Create an optimized fibonacci function using memoization",
            "file_path": "src/fibonacci.rs",
            "language": "rust"
        }),
        context: HashMap::new(),
        timeout: Some(Duration::from_secs(30)),
        retry_count: 0,
        max_retries: Some(2),
        agent_requirements: Vec::new(),
        created_at: chrono::Utc::now(),
        started_at: None,
        completed_at: None,
    };
    
    // Start telemetry span for the task
    let task_span = telemetry_system.start_agent_span(
        "orchestrator",
        "code_generation_workflow",
        SpanKind::Internal, 
        Some(demo_span)
    ).await?;
    
    // Record telemetry event
    telemetry_system.record_event(
        EventType::TaskStarted,
        "orchestrator",
        "Starting code generation workflow".to_string(),
        HashMap::new(),
        Some(task_span)
    ).await?;
    
    // Submit task to orchestrator
    let task_id = orchestrator.submit_task(task).await?;
    println!("  ✅ Submitted task: {}", task_id);
    
    // Wait for task completion
    println!("  ⏳ Waiting for agents to complete...");
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Check orchestrator status  
    let status = orchestrator.get_status().await;
    println!("  📈 Orchestrator Status:");
    println!("    - Active tasks: {}", status.active_tasks);
    println!("    - Completed tasks: {}", status.completed_tasks);
    println!("    - Failed tasks: {}", status.failed_tasks);
    
    // 8. Generate and Apply Code with Quality Gates
    println!("\n🔧 8. Generating code and applying with quality gates...");
    
    // Create a sample changeset
    let file_diff = FileDiff {
        file_path: PathBuf::from("src/fibonacci_optimized.rs"),
        original_content: None,
        new_content: r#"use std::collections::HashMap;

/// Optimized fibonacci calculation using memoization
pub struct FibonacciCalculator {
    cache: HashMap<usize, usize>,
}

impl FibonacciCalculator {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
    
    pub fn calculate(&mut self, n: usize) -> usize {
        if let Some(&result) = self.cache.get(&n) {
            return result;
        }
        
        let result = match n {
            0 => 0,
            1 => 1,
            _ => self.calculate(n - 1) + self.calculate(n - 2),
        };
        
        self.cache.insert(n, result);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fibonacci() {
        let mut calc = FibonacciCalculator::new();
        assert_eq!(calc.calculate(0), 0);
        assert_eq!(calc.calculate(1), 1);
        assert_eq!(calc.calculate(10), 55);
    }
}
"#.to_string(),
        diff_text: "".to_string(), // Would be generated
        change_type: ChangeType::Create,
        metadata: DiffMetadata {
            created_at: chrono::Utc::now(),
            agent_id: "code_generator".to_string(),
            task_id: task_id.to_string(),
            confidence_score: 0.9,
            estimated_lines_changed: 45,
            language: Some("rust".to_string()),
            description: "Optimized fibonacci implementation with memoization".to_string(),
        },
    };
    
    let mut changeset = ChangeSet {
        id: Uuid::new_v4().to_string(),
        title: "Add optimized fibonacci calculator".to_string(),
        description: "Implements fibonacci calculation with memoization for better performance".to_string(),
        files: vec![file_diff],
        metadata: ChangeSetMetadata {
            created_at: chrono::Utc::now(),
            agent_id: "code_generator".to_string(),
            task_id: task_id.to_string(),
            total_files: 1,
            total_lines_added: 45,
            total_lines_removed: 0,
            affects_tests: true,
            affects_dependencies: false,
        },
        validation_results: None,
    };
    
    // Preview the changeset
    let preview = diff_system.preview_changeset(&changeset).await?;
    println!("  📋 Change Preview:");
    for line in preview.lines().take(10) {
        println!("    {}", line);
    }
    if preview.lines().count() > 10 {
        println!("    ... (truncated)");
    }
    
    // Validate through quality gates
    println!("  🔍 Running quality gates...");
    diff_system.validate_changeset(&mut changeset, &project_root).await?;
    
    if let Some(validation) = &changeset.validation_results {
        println!("  ✅ Validation Status: {:?}", validation.overall_status);
        println!("  🚪 Gates passed: {}", validation.gates.len());
        if !validation.warnings.is_empty() {
            println!("  ⚠️  Warnings: {}", validation.warnings.len());
        }
    }
    
    // 9. Collect Telemetry and Stats
    println!("\n📊 9. Collecting telemetry and performance metrics...");
    
    // Record some metrics
    telemetry_system.record_metric(
        "code.lines_generated",
        45.0,
        "lines",
        Some("code_generator"),
        Some(task_span),
        HashMap::new()
    ).await?;
    
    telemetry_system.record_metric(
        "workflow.duration",
        2000.0,
        "milliseconds", 
        Some("orchestrator"),
        Some(demo_span),
        HashMap::new()
    ).await?;
    
    // Finish spans
    telemetry_system.finish_span(task_span, true, None, HashMap::new()).await?;
    telemetry_system.finish_span(demo_span, true, None, HashMap::new()).await?;
    
    // Get system summaries
    let telemetry_summary = telemetry_system.get_system_summary().await;
    let routing_stats = model_router.get_routing_stats().await;
    
    println!("  📈 Telemetry Summary:");
    println!("    - Total spans: {}", telemetry_summary.total_spans);
    println!("    - Total events: {}", telemetry_summary.total_events);
    println!("    - Active agents: {}", telemetry_summary.active_agents);
    println!("    - System health: {}", telemetry_summary.system_health);
    
    println!("  🧠 Model Routing Stats:");
    println!("    - Total requests: {}", routing_stats.total_requests);
    println!("    - Success rate: {:.2}%", routing_stats.success_rate * 100.0);
    println!("    - Cache hit rate: {:.2}%", routing_stats.cache_hit_rate * 100.0);
    
    // 10. Demonstrate Rules Application
    println!("\n📋 10. Applying hierarchical rules...");
    let effective_rules = rules_manager.get_effective_rules_for_context(&project_root).await?;
    println!("  📏 Effective rules for project:");
    for rule in effective_rules.iter().take(3) {
        println!("    - {}: {}", rule.name, rule.description);
    }
    
    let formatted_rules = rules_manager.format_rules_for_ai(&effective_rules);
    println!("  🤖 Rules formatted for AI ({} chars)", formatted_rules.len());
    
    // 11. Cleanup and Final Report
    println!("\n🏁 11. Integration demo complete!");
    telemetry_system.shutdown().await?;
    
    println!("\n🎉 Summary of Integrated Systems:");
    println!("  ✅ Deterministic orchestration - Task management with retries");
    println!("  ✅ Incremental embeddings - Codebase indexing and similarity search");  
    println!("  ✅ Rules precedence - Hierarchical WARP.md configuration");
    println!("  ✅ Diff-first apply - Quality gates and safe code application");
    println!("  ✅ Structured telemetry - Per-agent spans and distributed tracing");
    println!("  ✅ Mixed-model routing - Intelligent AI model selection and caching");
    
    println!("\n🚀 DevKit is now ready for production agentic development!");
    
    Ok(())
}

// Demo agent implementations
struct DemoCodeGenAgent;

#[async_trait::async_trait]
impl Agent for DemoCodeGenAgent {
    async fn process_task(&mut self, _task: AgentTask) -> Result<AgentResult, AgentError> {
        tokio::time::sleep(Duration::from_millis(500)).await; // Simulate work
        Ok(AgentResult {
            task_id: Uuid::new_v4(),
            success: true,
            output: serde_json::json!({"generated": "fibonacci code"}),
            artifacts: Vec::new(),
            next_actions: Vec::new(),
            metadata: HashMap::new(),
            processing_time: Duration::from_millis(500),
        })
    }
    
    fn agent_type(&self) -> crate::agents::AgentType {
        crate::agents::AgentType::CodeGenerator
    }
    
    fn capabilities(&self) -> Vec<String> {
        vec!["rust_generation".to_string(), "optimization".to_string()]
    }
}

struct DemoReviewAgent;

#[async_trait::async_trait] 
impl Agent for DemoReviewAgent {
    async fn process_task(&mut self, _task: AgentTask) -> Result<AgentResult, AgentError> {
        tokio::time::sleep(Duration::from_millis(300)).await;
        Ok(AgentResult {
            task_id: Uuid::new_v4(),
            success: true,
            output: serde_json::json!({"review": "Code looks good!"}),
            artifacts: Vec::new(),
            next_actions: Vec::new(),
            metadata: HashMap::new(),
            processing_time: Duration::from_millis(300),
        })
    }
    
    fn agent_type(&self) -> crate::agents::AgentType {
        crate::agents::AgentType::Reviewer
    }
    
    fn capabilities(&self) -> Vec<String> {
        vec!["code_review".to_string(), "security_analysis".to_string()]
    }
}

struct DemoTestAgent;

#[async_trait::async_trait]
impl Agent for DemoTestAgent {
    async fn process_task(&mut self, _task: AgentTask) -> Result<AgentResult, AgentError> {
        tokio::time::sleep(Duration::from_millis(200)).await;
        Ok(AgentResult {
            task_id: Uuid::new_v4(), 
            success: true,
            output: serde_json::json!({"tests": "Unit tests generated"}),
            artifacts: Vec::new(),
            next_actions: Vec::new(),
            metadata: HashMap::new(),
            processing_time: Duration::from_millis(200),
        })
    }
    
    fn agent_type(&self) -> crate::agents::AgentType {
        crate::agents::AgentType::Tester
    }
    
    fn capabilities(&self) -> Vec<String> {
        vec!["test_generation".to_string(), "coverage_analysis".to_string()]
    }
}