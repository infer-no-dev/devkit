//! Mock objects and test doubles for testing the agentic development environment.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::agents::{Agent, AgentTask, AgentResult, AgentStatus, AgentError, TaskPriority, Artifact};
use crate::codegen::language_detection::LanguageDetector;
use crate::codegen::templates::TemplateManager;
use crate::context::analyzer::CodebaseAnalyzer;
use crate::context::symbols::SymbolIndex;
use crate::shell::executor::CommandExecutor;

/// Mock agent for testing agent system functionality
#[derive(Debug)]
pub struct MockAgent {
    pub name: String,
    pub agent_type: String,
    pub status: Arc<Mutex<AgentStatus>>,
    pub processed_tasks: Arc<Mutex<Vec<AgentTask>>>,
    pub should_fail: bool,
    pub processing_delay: std::time::Duration,
}

impl MockAgent {
    pub fn new(name: &str, agent_type: &str) -> Self {
        Self {
            name: name.to_string(),
            agent_type: agent_type.to_string(),
            status: Arc::new(Mutex::new(AgentStatus::Idle)),
            processed_tasks: Arc::new(Mutex::new(Vec::new())),
            should_fail: false,
            processing_delay: std::time::Duration::from_millis(10),
        }
    }
    
    pub fn with_failure(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }
    
    pub fn with_delay(mut self, delay: std::time::Duration) -> Self {
        self.processing_delay = delay;
        self
    }
    
    pub fn get_processed_tasks(&self) -> Vec<AgentTask> {
        self.processed_tasks.lock().unwrap().clone()
    }
    
    pub fn get_status(&self) -> AgentStatus {
        self.status.lock().unwrap().clone()
    }
}

#[async_trait]
impl Agent for MockAgent {
    fn id(&self) -> &str {
        &self.name  // Using name as id for simplicity in tests
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn get_type_name(&self) -> &str {
        &self.agent_type
    }

    fn status(&self) -> AgentStatus {
        self.status.lock().unwrap().clone()
    }

    fn can_handle(&self, _task_type: &str) -> bool {
        true  // Mock agent can handle any task type
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["mock".to_string(), "test".to_string()]
    }
    
    
    async fn process_task(&mut self, task: AgentTask) -> Result<AgentResult, AgentError> {
        // Update status
        *self.status.lock().unwrap() = AgentStatus::Processing("Mock processing".to_string());
        
        // Store the task
        self.processed_tasks.lock().unwrap().push(task.clone());
        
        // Simulate processing time
        tokio::time::sleep(self.processing_delay).await;
        
        if self.should_fail {
            *self.status.lock().unwrap() = AgentStatus::Error("Mock error".to_string());
            return Err(AgentError::ProcessingFailed("Mock processing failed".to_string()));
        }
        
        *self.status.lock().unwrap() = AgentStatus::Idle;
        
        Ok(AgentResult {
            task_id: task.id,
            success: true,
            output: format!("Processed task: {}", task.description),
            artifacts: vec![Artifact {
                name: "mock_output".to_string(),
                artifact_type: "text".to_string(),
                content: "Mock processing result".to_string(),
                metadata: serde_json::json!({}),
            }],
            next_actions: Vec::new(),
        })
    }
    
}

/// Mock code generator for testing code generation functionality
pub struct MockCodeGenerator {
    pub generated_code: Arc<Mutex<Vec<String>>>,
    pub should_fail: bool,
    pub languages: Vec<String>,
}

impl MockCodeGenerator {
    pub fn new() -> Self {
        Self {
            generated_code: Arc::new(Mutex::new(Vec::new())),
            should_fail: false,
            languages: vec!["rust".to_string(), "python".to_string(), "javascript".to_string()],
        }
    }
    
    pub fn with_failure(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }
    
    pub fn get_generated_code(&self) -> Vec<String> {
        self.generated_code.lock().unwrap().clone()
    }
}

impl MockCodeGenerator {
    fn generate_code(
        &mut self,
        prompt: &str,
        _context: &crate::context::CodebaseContext,
        _config: &crate::codegen::GenerationConfig,
    ) -> Result<String, crate::codegen::CodeGenError> {
        if self.should_fail {
            return Err(crate::codegen::CodeGenError::GenerationFailed("Mock generation failed".to_string()));
        }
        
        let generated = format!("// Generated code for: {}\npub fn mock_function() {{\n    // Mock implementation\n}}", prompt);
        self.generated_code.lock().unwrap().push(generated.clone());
        Ok(generated)
    }
    
    fn detect_language(&self, _code: &str) -> Option<String> {
        Some("rust".to_string())
    }
    
    pub fn validate_code(&self, _code: &str, _language: &str) -> Result<Vec<String>, crate::codegen::CodeGenError> {
        Ok(vec!["Mock validation passed".to_string()])
    }
}

/// Mock language detector for testing language detection
pub struct MockLanguageDetector {
    pub detected_languages: HashMap<String, String>,
}

impl MockLanguageDetector {
    pub fn new() -> Self {
        let mut detected_languages = HashMap::new();
        detected_languages.insert("fn main()".to_string(), "rust".to_string());
        detected_languages.insert("def main()".to_string(), "python".to_string());
        detected_languages.insert("function main()".to_string(), "javascript".to_string());
        
        Self { detected_languages }
    }
}

impl MockLanguageDetector {
    fn detect_language(&self, code: &str) -> Option<String> {
        for (pattern, language) in &self.detected_languages {
            if code.contains(pattern) {
                return Some(language.clone());
            }
        }
        None
    }
    
    fn supported_languages(&self) -> Vec<String> {
        vec!["rust".to_string(), "python".to_string(), "javascript".to_string()]
    }
}

/// Mock template manager for testing template functionality
pub struct MockTemplateManager {
    pub templates: HashMap<String, String>,
}

impl MockTemplateManager {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        templates.insert(
            "rust_function".to_string(),
            "pub fn {{name}}({{params}}) -> {{return_type}} {\n    {{body}}\n}".to_string(),
        );
        templates.insert(
            "python_function".to_string(),
            "def {{name}}({{params}}) -> {{return_type}}:\n    {{body}}".to_string(),
        );
        
        Self { templates }
    }
}

impl MockTemplateManager {
    fn get_template(&self, name: &str) -> Option<String> {
        self.templates.get(name).cloned()
    }
    
    fn list_templates(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }
    
    fn render_template(&self, template: &str, variables: &HashMap<String, String>) -> Result<String, crate::codegen::CodeGenError> {
        let mut rendered = template.to_string();
        for (key, value) in variables {
            rendered = rendered.replace(&format!("{{{{{}}}}}", key), value);
        }
        Ok(rendered)
    }
}

/// Mock codebase analyzer for testing context analysis
pub struct MockCodebaseAnalyzer {
    pub analyzed_files: Arc<Mutex<Vec<String>>>,
    pub analysis_results: HashMap<String, String>,
    pub should_fail: bool,
}

impl MockCodebaseAnalyzer {
    pub fn new() -> Self {
        let mut analysis_results = HashMap::new();
        analysis_results.insert("main.rs".to_string(), "Main entry point".to_string());
        analysis_results.insert("lib.rs".to_string(), "Library module".to_string());
        
        Self {
            analyzed_files: Arc::new(Mutex::new(Vec::new())),
            analysis_results,
            should_fail: false,
        }
    }
    
    pub fn with_failure(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }
    
    pub fn get_analyzed_files(&self) -> Vec<String> {
        self.analyzed_files.lock().unwrap().clone()
    }
}

impl MockCodebaseAnalyzer {
    fn analyze_file(&mut self, file_path: &std::path::Path, _config: &crate::context::AnalysisConfig) -> Result<crate::context::FileContext, crate::context::ContextError> {
        if self.should_fail {
            return Err(crate::context::ContextError::AnalysisFailed("Mock analysis failed".to_string()));
        }
        
        let path_str = file_path.to_string_lossy().to_string();
        self.analyzed_files.lock().unwrap().push(path_str.clone());
        
        let description = self.analysis_results
            .get(&path_str)
            .unwrap_or(&"Unknown file".to_string())
            .clone();
        
        Ok(crate::context::FileContext {
            path: file_path.to_path_buf(),
            relative_path: file_path.to_path_buf(),
            language: "rust".to_string(),
            size_bytes: 0,
            line_count: 0,
            last_modified: std::time::SystemTime::now(),
            content_hash: "mock_hash".to_string(),
            symbols: Vec::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            relationships: Vec::new(),
        })
    }
    
    fn analyze_codebase(&mut self, _files: &[crate::context::FileContext]) -> Result<crate::context::CodebaseContext, crate::context::ContextError> {
        if self.should_fail {
            return Err(crate::context::ContextError::AnalysisFailed("Mock codebase analysis failed".to_string()));
        }
        
        Ok(crate::context::CodebaseContext {
            root_path: std::path::PathBuf::from("/mock/path"),
            files: Vec::new(),
            symbols: crate::context::symbols::SymbolIndex::new(),
            dependencies: Vec::new(),
            repository_info: None,
            semantic_analysis: None,
            metadata: crate::context::ContextMetadata {
                analysis_timestamp: std::time::SystemTime::now(),
                total_files: 0,
                total_lines: 0,
                languages: std::collections::HashMap::new(),
                analysis_duration_ms: 0,
                indexed_symbols: 0,
                semantic_patterns_found: 0,
                semantic_relationships: 0,
            },
        })
    }
}

/// Mock command executor for testing shell integration
pub struct MockCommandExecutor {
    pub executed_commands: Arc<Mutex<Vec<String>>>,
    pub command_results: HashMap<String, (String, i32)>, // Command -> (Output, Exit code)
    pub should_fail: bool,
}

impl MockCommandExecutor {
    pub fn new() -> Self {
        let mut command_results = HashMap::new();
        command_results.insert("echo test".to_string(), ("test\n".to_string(), 0));
        command_results.insert("ls".to_string(), ("file1.txt\nfile2.txt\n".to_string(), 0));
        command_results.insert("cargo check".to_string(), ("Finished dev [unoptimized + debuginfo] target(s)\n".to_string(), 0));
        
        Self {
            executed_commands: Arc::new(Mutex::new(Vec::new())),
            command_results,
            should_fail: false,
        }
    }
    
    pub fn with_failure(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }
    
    pub fn add_command_result(&mut self, command: &str, output: &str, exit_code: i32) {
        self.command_results.insert(command.to_string(), (output.to_string(), exit_code));
    }
    
    pub fn get_executed_commands(&self) -> Vec<String> {
        self.executed_commands.lock().unwrap().clone()
    }
}

impl MockCommandExecutor {
    fn execute_command(&mut self, command: &str, _args: &[&str]) -> Result<String, crate::shell::ShellError> {
        if self.should_fail {
            return Err(crate::shell::ShellError::ExecutionFailed("Mock command failed".to_string()));
        }
        
        let full_command = format!("{} {}", command, _args.join(" ")).trim().to_string();
        self.executed_commands.lock().unwrap().push(full_command.clone());
        
        if let Some((output, exit_code)) = self.command_results.get(&full_command) {
            if *exit_code == 0 {
                Ok(output.clone())
            } else {
                Err(crate::shell::ShellError::ExecutionFailed(format!("Command failed with exit code: {}", exit_code)))
            }
        } else {
            Ok("Mock command output".to_string())
        }
    }
    
    fn execute_command_async(&mut self, command: &str, args: &[&str]) -> Result<tokio::process::Child, crate::shell::ShellError> {
        // For testing, we'll simulate this
        let full_command = format!("{} {}", command, args.join(" ")).trim().to_string();
        self.executed_commands.lock().unwrap().push(full_command);
        
        if self.should_fail {
            return Err(crate::shell::ShellError::ExecutionFailed("Mock async command failed".to_string()));
        }
        
        // This is a mock implementation - in real tests we might use a different approach
        Err(crate::shell::ShellError::ExecutionFailed("Mock async not implemented".to_string()))
    }
    
    fn get_shell_type(&self) -> String {
        "mock_shell".to_string()
    }
}

/// Mock event sender for testing UI components
pub struct MockEventSender<T> {
    pub sent_events: Arc<Mutex<Vec<T>>>,
}

impl<T: Clone> MockEventSender<T> {
    pub fn new() -> Self {
        Self {
            sent_events: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub fn send(&self, event: T) {
        self.sent_events.lock().unwrap().push(event);
    }
    
    pub fn get_sent_events(&self) -> Vec<T> {
        self.sent_events.lock().unwrap().clone()
    }
    
    pub fn clear_events(&self) {
        self.sent_events.lock().unwrap().clear();
    }
}

/// Mock time provider for testing time-dependent functionality
pub struct MockTimeProvider {
    pub current_time: Arc<Mutex<std::time::SystemTime>>,
}

impl MockTimeProvider {
    pub fn new() -> Self {
        Self {
            current_time: Arc::new(Mutex::new(std::time::UNIX_EPOCH + std::time::Duration::from_secs(1234567890))),
        }
    }
    
    pub fn set_time(&self, time: std::time::SystemTime) {
        *self.current_time.lock().unwrap() = time;
    }
    
    pub fn advance_time(&self, duration: std::time::Duration) {
        let mut time = self.current_time.lock().unwrap();
        *time += duration;
    }
    
    pub fn now(&self) -> std::time::SystemTime {
        *self.current_time.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mock_agent() {
        let mut agent = MockAgent::new("test_agent", "mock");
        
        let task = AgentTask {
            id: "test_task".to_string(),
            task_type: "test".to_string(),
            description: "Test task".to_string(),
            priority: TaskPriority::Normal,
            context: serde_json::json!({}),
        };
        
        let result = agent.process_task(task.clone()).await;
        assert!(result.is_ok());
        
        let processed_tasks = agent.get_processed_tasks();
        assert_eq!(processed_tasks.len(), 1);
        assert_eq!(processed_tasks[0].id, "test_task");
    }
    
    #[tokio::test]
    async fn test_mock_agent_failure() {
        let mut agent = MockAgent::new("test_agent", "mock").with_failure(true);
        
        let task = AgentTask {
            id: "test_task".to_string(),
            task_type: "test".to_string(),
            description: "Test task".to_string(),
            priority: TaskPriority::Normal,
            context: serde_json::json!({}),
        };
        
        let result = agent.process_task(task).await;
        assert!(result.is_err());
        
        let status = agent.get_status();
        assert!(matches!(status, AgentStatus::Error(_)));
    }
    
    #[test]
    fn test_mock_code_generator() {
        let mut generator = MockCodeGenerator::new();
        let context = crate::context::CodebaseContext {
            root_path: std::path::PathBuf::from("/test"),
            files: Vec::new(),
            symbols: crate::context::symbols::SymbolIndex::new(),
            dependencies: Vec::new(),
            repository_info: None,
            semantic_analysis: None,
            metadata: crate::context::ContextMetadata {
                analysis_timestamp: std::time::SystemTime::now(),
                total_files: 0,
                total_lines: 0,
                languages: std::collections::HashMap::new(),
                analysis_duration_ms: 0,
                indexed_symbols: 0,
                semantic_patterns_found: 0,
                semantic_relationships: 0,
            },
        };
        
        let config = crate::codegen::GenerationConfig::default();
        let result = generator.generate_code("create a function", &context, &config);
        
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("Generated code for: create a function"));
        
        let generated = generator.get_generated_code();
        assert_eq!(generated.len(), 1);
    }
    
    #[test]
    fn test_mock_language_detector() {
        let detector = MockLanguageDetector::new();
        
        assert_eq!(detector.detect_language("fn main() {}"), Some("rust".to_string()));
        assert_eq!(detector.detect_language("def main():"), Some("python".to_string()));
        assert_eq!(detector.detect_language("function main() {}"), Some("javascript".to_string()));
        assert_eq!(detector.detect_language("unknown code"), None);
    }
    
    #[test]
    fn test_mock_command_executor() {
        let mut executor = MockCommandExecutor::new();
        
        let result = executor.execute_command("echo", &["test"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test\n");
        
        let executed = executor.get_executed_commands();
        assert_eq!(executed.len(), 1);
        assert_eq!(executed[0], "echo test");
    }
    
    #[test]
    fn test_mock_event_sender() {
        let sender = MockEventSender::new();
        
        sender.send("event1".to_string());
        sender.send("event2".to_string());
        
        let events = sender.get_sent_events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], "event1");
        assert_eq!(events[1], "event2");
        
        sender.clear_events();
        let events = sender.get_sent_events();
        assert_eq!(events.len(), 0);
    }
    
    #[test]
    fn test_mock_time_provider() {
        let time_provider = MockTimeProvider::new();
        let initial_time = time_provider.now();
        
        time_provider.advance_time(std::time::Duration::from_secs(60));
        let advanced_time = time_provider.now();
        
        assert!(advanced_time > initial_time);
        assert_eq!(
            advanced_time.duration_since(initial_time).unwrap(),
            std::time::Duration::from_secs(60)
        );
    }
}
