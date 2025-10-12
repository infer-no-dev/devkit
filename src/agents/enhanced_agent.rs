//! Enhanced agent implementation with integrated progress tracking
//! 
//! This example shows how to build an agent that provides real-time progress
//! feedback through the UI system.

use super::{
    Agent, AgentError, AgentResult, AgentStatus, AgentTask, BaseAgent, 
    AgentProgressTracker, AgentProgressExtension, TaskMetrics
};
use crate::ui::progress::ProgressManager;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, info, warn};
use async_trait::async_trait;

/// Enhanced code generation agent with progress tracking
#[derive(Debug)]
pub struct EnhancedCodeGenAgent {
    base: BaseAgent,
    progress_tracker: Option<Arc<AgentProgressTracker>>,
    config: CodeGenConfig,
}

/// Configuration for code generation
#[derive(Debug, Clone)]
pub struct CodeGenConfig {
    pub enable_detailed_steps: bool,
    pub simulate_processing_time: bool,
    pub max_concurrent_tasks: usize,
    pub quality_check_enabled: bool,
    pub auto_optimization: bool,
}

impl Default for CodeGenConfig {
    fn default() -> Self {
        Self {
            enable_detailed_steps: true,
            simulate_processing_time: true,
            max_concurrent_tasks: 2,
            quality_check_enabled: true,
            auto_optimization: false,
        }
    }
}

impl EnhancedCodeGenAgent {
    /// Create a new enhanced code generation agent
    pub fn new() -> Self {
        let base = BaseAgent::new(
            "Enhanced Code Generator".to_string(),
            vec![
                "code_generation".to_string(),
                "refactoring".to_string(),
                "optimization".to_string(),
                "documentation".to_string(),
            ],
        );

        Self {
            base,
            progress_tracker: None,
            config: CodeGenConfig::default(),
        }
    }

    /// Create with progress tracking
    pub fn with_progress_tracker(progress_tracker: Arc<AgentProgressTracker>) -> Self {
        let mut agent = Self::new();
        agent.progress_tracker = Some(progress_tracker);
        agent
    }

    /// Create with custom configuration  
    pub fn with_config(mut self, config: CodeGenConfig) -> Self {
        self.config = config;
        self
    }

    /// Define the steps for code generation
    fn get_code_generation_steps(&self, task_type: &str) -> Vec<String> {
        match task_type {
            "generate_code" => vec![
                "Analyzing requirements".to_string(),
                "Designing code structure".to_string(),
                "Generating core logic".to_string(),
                "Adding error handling".to_string(),
                "Writing documentation".to_string(),
                "Code quality check".to_string(),
                "Final optimization".to_string(),
            ],
            "refactor" => vec![
                "Analyzing existing code".to_string(),
                "Identifying improvement opportunities".to_string(),
                "Planning refactoring strategy".to_string(),
                "Applying code changes".to_string(),
                "Updating tests".to_string(),
                "Verifying functionality".to_string(),
            ],
            "documentation" => vec![
                "Analyzing code structure".to_string(),
                "Generating API documentation".to_string(),
                "Writing usage examples".to_string(),
                "Creating README content".to_string(),
            ],
            _ => vec![
                "Initializing".to_string(),
                "Processing".to_string(),
                "Finalizing".to_string(),
            ],
        }
    }

    /// Perform code generation with detailed progress tracking
    async fn generate_code_with_progress(
        &mut self,
        task: &AgentTask,
        operation_id: &str,
    ) -> Result<AgentResult, AgentError> {
        info!("Starting code generation for task: {}", task.id);
        
        let steps = self.get_code_generation_steps(&task.task_type);
        let mut generated_code = String::new();
        let mut artifacts = Vec::new();
        let mut final_metrics = TaskMetrics::default();

        // Step 1: Analyzing requirements
        if let Some(tracker) = &self.progress_tracker {
            tracker.update_progress(operation_id, Some(0), 0.1, 
                Some("Parsing task requirements...".to_string())).await?;
        }
        
        if self.config.simulate_processing_time {
            sleep(Duration::from_millis(500)).await;
        }

        // Simulate requirement analysis
        let requirements = self.analyze_requirements(task).await?;
        final_metrics.files_analyzed = requirements.complexity_score as usize;

        if let Some(tracker) = &self.progress_tracker {
            tracker.complete_step(operation_id, 0, true, 
                Some(format!("Found {} requirements", requirements.requirement_count))).await?;
            tracker.update_progress(operation_id, Some(1), 0.0, 
                Some("Designing code architecture...".to_string())).await?;
        }

        // Step 2: Designing code structure
        if self.config.simulate_processing_time {
            sleep(Duration::from_millis(800)).await;
        }

        let design = self.design_code_structure(&requirements).await?;
        final_metrics.lines_processed = design.estimated_lines;

        if let Some(tracker) = &self.progress_tracker {
            tracker.complete_step(operation_id, 1, true, 
                Some(format!("Designed {} components", design.component_count))).await?;
            tracker.update_progress(operation_id, Some(2), 0.0, 
                Some("Generating core implementation...".to_string())).await?;
        }

        // Step 3: Generating core logic
        for i in 0..design.component_count {
            if self.config.simulate_processing_time {
                sleep(Duration::from_millis(300)).await;
            }
            
            let component_code = self.generate_component(&design, i).await?;
            generated_code.push_str(&component_code);
            final_metrics.api_calls_made += 1;
            
            let progress = (i + 1) as f64 / design.component_count as f64;
            if let Some(tracker) = &self.progress_tracker {
                tracker.update_progress(operation_id, Some(2), progress,
                    Some(format!("Generated component {} of {}", i + 1, design.component_count))).await?;
            }
        }

        if let Some(tracker) = &self.progress_tracker {
            tracker.complete_step(operation_id, 2, true, 
                Some("Core logic generation completed".to_string())).await?;
            tracker.update_progress(operation_id, Some(3), 0.1, 
                Some("Adding error handling...".to_string())).await?;
        }

        // Step 4: Adding error handling
        if self.config.simulate_processing_time {
            sleep(Duration::from_millis(600)).await;
        }

        let error_handling = self.add_error_handling(&generated_code).await?;
        generated_code = error_handling;
        final_metrics.tokens_processed = generated_code.len() as u64;

        if let Some(tracker) = &self.progress_tracker {
            tracker.complete_step(operation_id, 3, true, 
                Some("Error handling added successfully".to_string())).await?;
            tracker.update_progress(operation_id, Some(4), 0.2, 
                Some("Writing documentation...".to_string())).await?;
        }

        // Step 5: Writing documentation
        if self.config.simulate_processing_time {
            sleep(Duration::from_millis(400)).await;
        }

        let documentation = self.generate_documentation(&generated_code).await?;
        
        if let Some(tracker) = &self.progress_tracker {
            tracker.update_progress(operation_id, Some(4), 1.0, 
                Some("Documentation completed".to_string())).await?;
            tracker.complete_step(operation_id, 4, true, 
                Some("Documentation written successfully".to_string())).await?;
        }

        // Step 6: Quality check (if enabled)
        if self.config.quality_check_enabled {
            if let Some(tracker) = &self.progress_tracker {
                tracker.update_progress(operation_id, Some(5), 0.1, 
                    Some("Running quality checks...".to_string())).await?;
            }

            if self.config.simulate_processing_time {
                sleep(Duration::from_millis(700)).await;
            }

            let quality_result = self.perform_quality_check(&generated_code).await?;
            
            if quality_result.passed {
                if let Some(tracker) = &self.progress_tracker {
                    tracker.complete_step(operation_id, 5, true, 
                        Some(format!("Quality check passed with score: {}", quality_result.score))).await?;
                }
            } else {
                warn!("Quality check failed, but proceeding with generated code");
                if let Some(tracker) = &self.progress_tracker {
                    tracker.complete_step(operation_id, 5, false, 
                        Some(format!("Quality check failed with score: {}", quality_result.score))).await?;
                }
            }
        } else {
            if let Some(tracker) = &self.progress_tracker {
                tracker.complete_step(operation_id, 5, true, 
                    Some("Quality check skipped".to_string())).await?;
            }
        }

        // Step 7: Final optimization (if enabled)
        if self.config.auto_optimization {
            if let Some(tracker) = &self.progress_tracker {
                tracker.update_progress(operation_id, Some(6), 0.2, 
                    Some("Optimizing generated code...".to_string())).await?;
            }

            if self.config.simulate_processing_time {
                sleep(Duration::from_millis(500)).await;
            }

            generated_code = self.optimize_code(&generated_code).await?;
            
            if let Some(tracker) = &self.progress_tracker {
                tracker.complete_step(operation_id, 6, true, 
                    Some("Code optimization completed".to_string())).await?;
            }
        } else {
            if let Some(tracker) = &self.progress_tracker {
                tracker.complete_step(operation_id, 6, true, 
                    Some("Optimization skipped".to_string())).await?;
            }
        }

        // Create artifacts
        artifacts.push(crate::agents::task::AgentArtifact {
            id: format!("{}_code", task.id),
            name: "generated_code.rs".to_string(),
            artifact_type: "source_code".to_string(),
            content: generated_code.clone(),
            file_path: Some("generated_code.rs".to_string()),
            mime_type: Some("text/x-rust".to_string()),
            metadata: std::collections::HashMap::new(),
        });

        artifacts.push(crate::agents::task::AgentArtifact {
            id: format!("{}_docs", task.id),
            name: "README.md".to_string(),
            artifact_type: "documentation".to_string(),
            content: documentation,
            file_path: Some("README.md".to_string()),
            mime_type: Some("text/markdown".to_string()),
            metadata: std::collections::HashMap::new(),
        });

        // Create successful result
        let result = AgentResult::success(task.id.clone(), self.base.id.clone(), 
                                       format!("Generated {} lines of code with {} components", 
                                               generated_code.lines().count(), design.component_count))
            .with_artifact(artifacts[0].clone())
            .with_artifact(artifacts[1].clone())
            .with_next_action("Review generated code".to_string())
            .with_next_action("Run tests".to_string())
            .with_metadata("lines_generated".to_string(), 
                          serde_json::Value::Number(serde_json::Number::from(generated_code.lines().count())));

        // Complete progress tracking
        if let Some(tracker) = &self.progress_tracker {
            tracker.complete_operation(&operation_id, true, 
                Some("Code generation completed successfully!".to_string()), 
                Some(final_metrics)).await?;
        }

        info!("Code generation completed successfully for task: {}", task.id);
        Ok(result)
    }

    // Mock implementation of requirement analysis
    async fn analyze_requirements(&self, task: &AgentTask) -> Result<Requirements, AgentError> {
        // Simulate analysis based on task description
        let requirement_count = task.description.split_whitespace().count();
        let complexity_score = std::cmp::min(requirement_count * 2, 100);
        
        Ok(Requirements {
            requirement_count,
            complexity_score,
            language: "rust".to_string(),
            framework: None,
        })
    }

    // Mock implementation of code structure design
    async fn design_code_structure(&self, requirements: &Requirements) -> Result<CodeDesign, AgentError> {
        let component_count = std::cmp::max(1, requirements.requirement_count / 3);
        let estimated_lines = component_count * 50;
        
        Ok(CodeDesign {
            component_count,
            estimated_lines,
            architecture: "modular".to_string(),
        })
    }

    // Mock implementation of component generation
    async fn generate_component(&self, design: &CodeDesign, component_index: usize) -> Result<String, AgentError> {
        Ok(format!(
            "// Component {}\npub struct Component{} {{\n    // Implementation here\n}}\n\n",
            component_index + 1,
            component_index + 1
        ))
    }

    // Mock implementation of error handling addition
    async fn add_error_handling(&self, code: &str) -> Result<String, AgentError> {
        Ok(format!(
            "use std::error::Error;\nuse std::result::Result;\n\n{}",
            code
        ))
    }

    // Mock implementation of documentation generation
    async fn generate_documentation(&self, _code: &str) -> Result<String, AgentError> {
        Ok("# Generated Code Documentation\n\nThis code was generated automatically.\n\n## Usage\n\n```rust\n// Example usage here\n```".to_string())
    }

    // Mock implementation of quality check
    async fn perform_quality_check(&self, _code: &str) -> Result<QualityResult, AgentError> {
        // Simulate quality scoring
        Ok(QualityResult {
            passed: true,
            score: 85,
            issues: vec![],
        })
    }

    // Mock implementation of code optimization
    async fn optimize_code(&self, code: &str) -> Result<String, AgentError> {
        Ok(format!("// Optimized version\n{}", code))
    }
}

#[async_trait]
impl Agent for EnhancedCodeGenAgent {
    fn id(&self) -> &str {
        &self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn status(&self) -> AgentStatus {
        self.base.status.clone()
    }

    fn capabilities(&self) -> Vec<String> {
        self.base.capabilities.clone()
    }

    async fn process_task(&mut self, task: AgentTask) -> Result<AgentResult, AgentError> {
        let start_time = Instant::now();
        
        // Update agent status
        self.base.status = AgentStatus::Processing { task_id: task.id.clone() };
        
        // Start progress tracking if available
        let operation_id = if let Some(tracker) = &self.progress_tracker {
            let steps = self.get_code_generation_steps(&task.task_type);
            match self.start_progress_tracking(tracker, &task, steps).await {
                Ok(id) => Some(id),
                Err(e) => {
                    warn!("Failed to start progress tracking: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Process the task
        let result = if operation_id.is_some() {
            self.generate_code_with_progress(&task, &operation_id.unwrap()).await
        } else {
            // Fallback processing without progress tracking
            self.generate_basic_code(&task).await
        };

        // Update metrics
        let duration = start_time.elapsed();
        let success = result.is_ok();
        self.base.update_metrics(success, duration);
        
        // Update status
        self.base.status = if success {
            AgentStatus::Idle
        } else {
            AgentStatus::Error { 
                message: result.as_ref().err().unwrap().to_string() 
            }
        };

        result
    }

    fn can_handle(&self, task_type: &str) -> bool {
        matches!(task_type, "generate_code" | "refactor" | "documentation" | "optimization")
    }

    fn get_metrics(&self) -> crate::agents::AgentMetrics {
        self.base.metrics.clone()
    }

    async fn shutdown(&mut self) -> Result<(), AgentError> {
        info!("Enhanced Code Generation Agent {} shutting down", self.base.name);
        self.base.status = AgentStatus::ShuttingDown;
        Ok(())
    }
}

impl EnhancedCodeGenAgent {
    /// Basic code generation without progress tracking
    async fn generate_basic_code(&self, task: &AgentTask) -> Result<AgentResult, AgentError> {
        info!("Generating code for task: {} (basic mode)", task.id);
        
        if self.config.simulate_processing_time {
            sleep(Duration::from_secs(2)).await; // Simulate work
        }

        let basic_code = format!(
            "// Generated code for task: {}\n// Description: {}\n\nfn main() {{\n    println!(\"Hello, World!\");\n}}",
            task.id, task.description
        );

        let result = AgentResult::success(task.id.clone(), self.base.id.clone(), 
                                       "Basic code generation completed".to_string())
            .with_next_action("Review generated code".to_string());

        Ok(result)
    }
}

/// Mock data structures for the example
#[derive(Debug)]
struct Requirements {
    requirement_count: usize,
    complexity_score: usize,
    language: String,
    framework: Option<String>,
}

#[derive(Debug)]
struct CodeDesign {
    component_count: usize,
    estimated_lines: usize,
    architecture: String,
}

#[derive(Debug)]
struct QualityResult {
    passed: bool,
    score: u32,
    issues: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::progress::ProgressManager;

    #[tokio::test]
    async fn test_enhanced_agent_creation() {
        let agent = EnhancedCodeGenAgent::new();
        assert_eq!(agent.name(), "Enhanced Code Generator");
        assert!(agent.can_handle("generate_code"));
        assert!(agent.can_handle("refactor"));
        assert!(!agent.can_handle("unknown_task"));
    }

    #[tokio::test]
    async fn test_agent_with_progress_tracking() {
        let progress_manager = Arc::new(ProgressManager::new());
        let progress_tracker = Arc::new(AgentProgressTracker::new(progress_manager));
        let mut agent = EnhancedCodeGenAgent::with_progress_tracker(progress_tracker);

        let task = AgentTask::new(
            "generate_code".to_string(),
            "Create a simple Rust function".to_string(),
            serde_json::Value::Null,
        );

        let result = agent.process_task(task).await;
        assert!(result.is_ok());
        
        let agent_result = result.unwrap();
        assert!(agent_result.success);
        assert!(!agent_result.artifacts.is_empty());
    }

    #[tokio::test]
    async fn test_code_generation_steps() {
        let agent = EnhancedCodeGenAgent::new();
        let steps = agent.get_code_generation_steps("generate_code");
        
        assert!(steps.len() >= 5);
        assert!(steps.contains(&"Analyzing requirements".to_string()));
        assert!(steps.contains(&"Generating core logic".to_string()));
    }
}