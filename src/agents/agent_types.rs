//! Specialized agent implementations for different tasks

use super::task::{AgentArtifact, AgentResult, AgentTask};
use super::{Agent, AgentError, AgentMetrics, AgentStatus, BaseAgent};
use crate::ai::AIManager;

use serde_json::json;
use std::sync::Arc;

// Re-export the review agent

/// Agent specialized for code generation tasks
#[derive(Debug)]
pub struct CodeGenerationAgent {
    base: BaseAgent,
    ai_manager: Option<Arc<AIManager>>,
}

impl CodeGenerationAgent {
    /// Create a new code generation agent
    pub fn new() -> Self {
        Self {
            base: BaseAgent::new(
                "CodeGenerationAgent".to_string(),
                vec![
                    "code_generation".to_string(),
                    "generate_function".to_string(),
                    "generate_class".to_string(),
                    "generate_module".to_string(),
                    "generate_tests".to_string(),
                    "complete_code".to_string(),
                ],
            ),
            ai_manager: None,
        }
    }

    /// Create a code generation agent with AI capabilities
    pub fn with_ai_manager(ai_manager: Arc<AIManager>) -> Self {
        let mut agent = Self::new();
        agent.ai_manager = Some(ai_manager);
        agent
    }

    async fn generate_code(&mut self, task: &AgentTask) -> Result<AgentResult, AgentError> {
        let start_time = std::time::Instant::now();

        // Extract parameters from task context
        let language = task
            .context
            .get("language")
            .and_then(|l| l.as_str())
            .unwrap_or("rust");

        let requirements = task
            .context
            .get("requirements")
            .and_then(|r| r.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();

        let _file_path = task.context.get("file_path").and_then(|p| p.as_str());

        let existing_code = task.context.get("existing_code").and_then(|c| c.as_str());

        // Generate code using AI if available
        let generated_code = if let Some(ai_manager) = &self.ai_manager {
            self.generate_with_ai(
                ai_manager,
                &task.description,
                language,
                &requirements,
                existing_code,
            )
            .await?
        } else {
            self.generate_template_code(&task.task_type, language, &task.description)
        };

        // Create artifact
        let artifact = AgentArtifact::source_code(
            format!("{}.{}", task.task_type, self.get_file_extension(language)),
            generated_code.clone(),
            language.to_string(),
        )
        .with_metadata("confidence".to_string(), json!(0.8))
        .with_metadata(
            "generation_time_ms".to_string(),
            json!(start_time.elapsed().as_millis()),
        );

        // Update metrics
        let duration = start_time.elapsed();
        self.base.update_metrics(true, duration);

        Ok(AgentResult::success(
            task.id.clone(),
            self.base.id.clone(),
            format!("Generated {} code for: {}", language, task.description),
        )
        .with_artifact(artifact)
        .with_duration(duration)
        .with_next_action("Review and test the generated code".to_string())
        .with_next_action("Consider adding documentation".to_string()))
    }

    async fn generate_with_ai(
        &self,
        ai_manager: &AIManager,
        description: &str,
        language: &str,
        requirements: &[&str],
        existing_code: Option<&str>,
    ) -> Result<String, AgentError> {
        let system_prompt = format!(
            "You are a skilled {} developer. Generate clean, well-documented code that follows best practices.",
            language
        );

        let mut user_prompt = format!("Generate {} code for: {}", language, description);

        if !requirements.is_empty() {
            user_prompt.push_str("\n\nRequirements:");
            for req in requirements {
                user_prompt.push_str(&format!("\n- {}", req));
            }
        }

        if let Some(code) = existing_code {
            user_prompt.push_str("\n\nExisting code to work with:\n");
            user_prompt.push_str(code);
        }

        user_prompt.push_str(
            "\n\nPlease provide only the code, without explanations or markdown formatting.",
        );

        ai_manager
            .generate_response(&system_prompt, &user_prompt, Some(2000), Some(0.3))
            .await
            .map_err(|e| AgentError::AIServiceError(e.to_string()))
    }

    fn generate_template_code(&self, task_type: &str, language: &str, description: &str) -> String {
        match (task_type, language) {
            ("generate_function", "rust") => format!(
                "/// {}\npub fn generated_function() -> Result<(), Box<dyn std::error::Error>> {{\n    // TODO: Implement function logic\n    Ok(())\n}}",
                description
            ),
            ("generate_function", "python") => format!(
                "def generated_function():\n    \"\"\"{}\"\"\"\n    # TODO: Implement function logic\n    pass",
                description
            ),
            ("generate_class", "rust") => format!(
                "/// {}\n#[derive(Debug)]\npub struct GeneratedStruct {{\n    // TODO: Add fields\n}}\n\nimpl GeneratedStruct {{\n    pub fn new() -> Self {{\n        Self {{\n            // TODO: Initialize fields\n        }}\n    }}\n}}",
                description
            ),
            ("generate_class", "python") => format!(
                "class GeneratedClass:\n    \"\"\"{}\"\"\"\n    \n    def __init__(self):\n        # TODO: Initialize attributes\n        pass",
                description
            ),
            _ => format!("// TODO: {} in {}\n// {}", task_type, language, description),
        }
    }

    fn get_file_extension(&self, language: &str) -> &'static str {
        match language {
            "rust" => "rs",
            "python" => "py",
            "javascript" => "js",
            "typescript" => "ts",
            "java" => "java",
            "c" => "c",
            "cpp" | "c++" => "cpp",
            "go" => "go",
            _ => "txt",
        }
    }
}

#[async_trait::async_trait]
impl Agent for CodeGenerationAgent {
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
        self.base.status = AgentStatus::Processing {
            task_id: task.id.clone(),
        };

        let result = match task.task_type.as_str() {
            "code_generation" | "generate_function" | "generate_class" | "generate_module" | "generate_tests"
            | "complete_code" | "refactor_code" => self.generate_code(&task).await,
            _ => Err(AgentError::InvalidTaskType {
                task_type: task.task_type.clone(),
            }),
        };

        self.base.status = AgentStatus::Idle;
        result
    }

    fn can_handle(&self, task_type: &str) -> bool {
        self.base.capabilities.contains(&task_type.to_string())
    }

    fn get_metrics(&self) -> AgentMetrics {
        self.base.metrics.clone()
    }

    async fn shutdown(&mut self) -> Result<(), AgentError> {
        self.base.status = AgentStatus::Offline;
        Ok(())
    }
}

/// Agent specialized for code analysis tasks
#[derive(Debug)]
pub struct AnalysisAgent {
    base: BaseAgent,
    ai_manager: Option<Arc<AIManager>>,
}

impl AnalysisAgent {
    pub fn new() -> Self {
        Self {
            base: BaseAgent::new(
                "AnalysisAgent".to_string(),
                vec![
                    "analyze_code".to_string(),
                    "analyze_file".to_string(),
                    "check_quality".to_string(),
                    "find_issues".to_string(),
                    "suggest_improvements".to_string(),
                ],
            ),
            ai_manager: None,
        }
    }

    pub fn with_ai_manager(ai_manager: Arc<AIManager>) -> Self {
        let mut agent = Self::new();
        agent.ai_manager = Some(ai_manager);
        agent
    }

    async fn analyze_code(&mut self, task: &AgentTask) -> Result<AgentResult, AgentError> {
        let start_time = std::time::Instant::now();

        let code = task
            .context
            .get("code")
            .and_then(|c| c.as_str())
            .or_else(|| task.context.get("file_path").and_then(|p| p.as_str()))
            .ok_or_else(|| {
                AgentError::TaskExecutionFailed("No code or file path provided".to_string())
            })?;

        let analysis_result = if let Some(ai_manager) = &self.ai_manager {
            self.analyze_with_ai(ai_manager, code, &task.description)
                .await?
        } else {
            format!("Basic analysis of: {}\n- Code structure looks reasonable\n- Consider adding more error handling\n- Documentation could be improved", code)
        };

        let artifact = AgentArtifact::new(
            "analysis_report".to_string(),
            "analysis".to_string(),
            analysis_result.clone(),
        )
        .with_metadata("analysis_type".to_string(), json!(task.task_type.clone()));

        let duration = start_time.elapsed();
        self.base.update_metrics(true, duration);

        Ok(
            AgentResult::success(task.id.clone(), self.base.id.clone(), analysis_result)
                .with_artifact(artifact)
                .with_duration(duration),
        )
    }

    async fn analyze_with_ai(
        &self,
        ai_manager: &AIManager,
        code: &str,
        description: &str,
    ) -> Result<String, AgentError> {
        let system_prompt = "You are an expert code reviewer. Analyze the provided code and provide detailed feedback on code quality, potential issues, and improvements.";

        let user_prompt = format!(
            "Please analyze this code for: {}\n\nCode:\n{}\n\nProvide a detailed analysis covering:\n- Code structure and organization\n- Potential bugs or issues\n- Performance considerations\n- Best practice recommendations\n- Security concerns if any",
            description, code
        );

        ai_manager
            .generate_response(system_prompt, &user_prompt, Some(1500), Some(0.1))
            .await
            .map_err(|e| AgentError::AIServiceError(e.to_string()))
    }
}

#[async_trait::async_trait]
impl Agent for AnalysisAgent {
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
        self.base.status = AgentStatus::Processing {
            task_id: task.id.clone(),
        };

        let result = self.analyze_code(&task).await;

        self.base.status = AgentStatus::Idle;
        result
    }

    fn can_handle(&self, task_type: &str) -> bool {
        self.base.capabilities.contains(&task_type.to_string())
    }

    fn get_metrics(&self) -> AgentMetrics {
        self.base.metrics.clone()
    }

    async fn shutdown(&mut self) -> Result<(), AgentError> {
        self.base.status = AgentStatus::Offline;
        Ok(())
    }
}

/// Agent specialized for code refactoring tasks
#[derive(Debug)]
pub struct RefactoringAgent {
    base: BaseAgent,
    ai_manager: Option<Arc<AIManager>>,
}

impl RefactoringAgent {
    pub fn new() -> Self {
        Self {
            base: BaseAgent::new(
                "RefactoringAgent".to_string(),
                vec![
                    "refactor_code".to_string(),
                    "optimize_performance".to_string(),
                    "improve_structure".to_string(),
                    "update_patterns".to_string(),
                ],
            ),
            ai_manager: None,
        }
    }

    pub fn with_ai_manager(ai_manager: Arc<AIManager>) -> Self {
        let mut agent = Self::new();
        agent.ai_manager = Some(ai_manager);
        agent
    }

    async fn refactor_code(&mut self, task: &AgentTask) -> Result<AgentResult, AgentError> {
        let start_time = std::time::Instant::now();

        let code = task
            .context
            .get("existing_code")
            .and_then(|c| c.as_str())
            .ok_or_else(|| {
                AgentError::TaskExecutionFailed("No existing code provided".to_string())
            })?;

        let refactored_code = if let Some(ai_manager) = &self.ai_manager {
            self.refactor_with_ai(ai_manager, code, &task.description)
                .await?
        } else {
            format!("// Refactored version of:\n// {}\n\n{}\n\n// TODO: Apply specific refactoring improvements", task.description, code)
        };

        let artifact = AgentArtifact::source_code(
            "refactored_code".to_string(),
            refactored_code.clone(),
            "auto".to_string(), // Auto-detect language
        )
        .with_metadata(
            "refactoring_type".to_string(),
            json!(task.task_type.clone()),
        );

        let duration = start_time.elapsed();
        self.base.update_metrics(true, duration);

        Ok(AgentResult::success(
            task.id.clone(),
            self.base.id.clone(),
            format!("Refactored code for: {}", task.description),
        )
        .with_artifact(artifact)
        .with_duration(duration)
        .with_next_action("Review refactored code for correctness".to_string())
        .with_next_action("Run tests to ensure functionality is preserved".to_string()))
    }

    async fn refactor_with_ai(
        &self,
        ai_manager: &AIManager,
        code: &str,
        description: &str,
    ) -> Result<String, AgentError> {
        let system_prompt = "You are an expert software engineer specializing in code refactoring. Improve the provided code while maintaining its functionality.";

        let user_prompt = format!(
            "Please refactor this code for: {}\n\nOriginal code:\n{}\n\nProvide the refactored code with improvements in:\n- Code structure and readability\n- Performance optimizations\n- Best practices compliance\n- Error handling\n\nProvide only the refactored code without explanations.",
            description, code
        );

        ai_manager
            .generate_response(system_prompt, &user_prompt, Some(2000), Some(0.2))
            .await
            .map_err(|e| AgentError::AIServiceError(e.to_string()))
    }
}

#[async_trait::async_trait]
impl Agent for RefactoringAgent {
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
        self.base.status = AgentStatus::Processing {
            task_id: task.id.clone(),
        };

        let result = self.refactor_code(&task).await;

        self.base.status = AgentStatus::Idle;
        result
    }

    fn can_handle(&self, task_type: &str) -> bool {
        self.base.capabilities.contains(&task_type.to_string())
    }

    fn get_metrics(&self) -> AgentMetrics {
        self.base.metrics.clone()
    }

    async fn shutdown(&mut self) -> Result<(), AgentError> {
        self.base.status = AgentStatus::Offline;
        Ok(())
    }
}
