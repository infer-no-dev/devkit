use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::time::Instant;
use std::sync::Arc;

use crate::agents::{Agent, AgentError, AgentStatus, AgentMetrics, BaseAgent};
use crate::agents::task::{AgentTask, AgentResult, AgentArtifact};
use crate::ai::AIManager;
use serde_json::json;

/// Code review agent that performs comprehensive code analysis
#[derive(Debug)]
pub struct CodeReviewAgent {
    base: BaseAgent,
    ai_manager: Option<Arc<AIManager>>,
}

/// Severity levels for review issues
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReviewSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Categories of review issues
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReviewCategory {
    Security,
    Performance,
    CodeSmell,
    Documentation,
    Testing,
    Maintainability,
    Style,
    Bug,
    Vulnerability,
}

/// A specific issue found during code review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewIssue {
    pub category: ReviewCategory,
    pub severity: ReviewSeverity,
    pub title: String,
    pub description: String,
    pub file_path: PathBuf,
    pub line_start: Option<usize>,
    pub line_end: Option<usize>,
    pub suggestion: Option<String>,
    pub auto_fixable: bool,
    pub code_snippet: Option<String>,
}

/// Configuration for code review focus areas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewConfig {
    pub focus_areas: Vec<ReviewCategory>,
    pub exclude_files: Vec<String>,
    pub severity_threshold: ReviewSeverity,
    pub enable_auto_fix: bool,
    pub include_style_issues: bool,
    pub max_issues_per_file: usize,
}

impl Default for ReviewConfig {
    fn default() -> Self {
        Self {
            focus_areas: vec![
                ReviewCategory::Security,
                ReviewCategory::Performance,
                ReviewCategory::Bug,
                ReviewCategory::CodeSmell,
            ],
            exclude_files: vec![
                "target/**".to_string(),
                "node_modules/**".to_string(),
                "*.log".to_string(),
            ],
            severity_threshold: ReviewSeverity::Low,
            enable_auto_fix: false,
            include_style_issues: true,
            max_issues_per_file: 50,
        }
    }
}

/// Results from a code review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub issues: Vec<ReviewIssue>,
    pub summary: ReviewSummary,
    pub files_reviewed: usize,
    pub total_lines: usize,
    pub review_duration: std::time::Duration,
}

/// Summary statistics from code review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSummary {
    pub total_issues: usize,
    pub issues_by_severity: HashMap<String, usize>,
    pub issues_by_category: HashMap<String, usize>,
    pub auto_fixable_issues: usize,
    pub most_common_issue: Option<String>,
}

impl CodeReviewAgent {
    pub fn new() -> Self {
        Self {
            base: BaseAgent::new(
                "CodeReviewAgent".to_string(),
                vec![
                    "review_code".to_string(),
                    "security_analysis".to_string(),
                    "performance_analysis".to_string(),
                    "code_smells".to_string(),
                    "documentation_review".to_string(),
                    "test_coverage_review".to_string(),
                ]
            ),
            ai_manager: None,
        }
    }
    
    /// Create a code review agent with AI capabilities
    pub fn with_ai_manager(ai_manager: Arc<AIManager>) -> Self {
        let mut agent = Self::new();
        agent.ai_manager = Some(ai_manager);
        agent
    }

    /// Perform code review on specified paths
    pub async fn review_code(
        &mut self,
        paths: &[PathBuf],
        config: ReviewConfig,
    ) -> Result<ReviewResult, AgentError> {
        self.base.status = AgentStatus::Processing { task_id: "review".to_string() };
        let start_time = Instant::now();
        let mut all_issues = Vec::new();
        let mut files_reviewed = 0;
        let mut total_lines = 0;

        for path in paths {
            if path.is_file() {
                if let Ok(issues) = self.review_file(path, &config).await {
                    all_issues.extend(issues);
                    files_reviewed += 1;
                    
                    // Count lines in file
                    if let Ok(content) = tokio::fs::read_to_string(path).await {
                        total_lines += content.lines().count();
                    }
                }
            } else if path.is_dir() {
                let dir_result = self.review_directory(path, &config).await?;
                all_issues.extend(dir_result.issues);
                files_reviewed += dir_result.files_reviewed;
                total_lines += dir_result.total_lines;
            }
        }

        // Sort issues by severity (highest first)
        all_issues.sort_by(|a, b| b.severity.cmp(&a.severity));

        let summary = self.generate_summary(&all_issues);
        let review_duration = start_time.elapsed();

        self.base.status = AgentStatus::Idle;

        Ok(ReviewResult {
            issues: all_issues,
            summary,
            files_reviewed,
            total_lines,
            review_duration,
        })
    }

    /// Review a single file
    async fn review_file(
        &self,
        file_path: &PathBuf,
        config: &ReviewConfig,
    ) -> Result<Vec<ReviewIssue>, AgentError> {
        // Skip excluded files
        if self.should_exclude_file(file_path, &config.exclude_files) {
            return Ok(Vec::new());
        }

        let content = tokio::fs::read_to_string(file_path)
            .await
            .map_err(|e| AgentError::TaskExecutionFailed(format!("Failed to read file: {}", e)))?;

        let mut issues = Vec::new();

        // Run different types of analysis based on focus areas
        for category in &config.focus_areas {
            match category {
                ReviewCategory::Security => {
                    issues.extend(self.analyze_security(&content, file_path).await?);
                }
                ReviewCategory::Performance => {
                    issues.extend(self.analyze_performance(&content, file_path).await?);
                }
                ReviewCategory::CodeSmell => {
                    issues.extend(self.analyze_code_smells(&content, file_path).await?);
                }
                ReviewCategory::Documentation => {
                    issues.extend(self.analyze_documentation(&content, file_path).await?);
                }
                ReviewCategory::Testing => {
                    issues.extend(self.analyze_testing(&content, file_path).await?);
                }
                _ => {
                    // Generic analysis for other categories
                    issues.extend(self.analyze_generic(&content, file_path, category).await?);
                }
            }
        }

        // Filter by severity threshold
        issues.retain(|issue| issue.severity >= config.severity_threshold);

        // Limit issues per file
        issues.truncate(config.max_issues_per_file);

        Ok(issues)
    }

    /// Review all files in a directory
    fn review_directory<'a>(
        &'a mut self,
        dir_path: &'a PathBuf,
        config: &'a ReviewConfig,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ReviewResult, AgentError>> + 'a + Send>> {
        Box::pin(async move {
        let mut all_issues = Vec::new();
        let mut files_reviewed = 0;
        let mut total_lines = 0;

        let mut entries = tokio::fs::read_dir(dir_path)
            .await
            .map_err(|e| AgentError::TaskExecutionFailed(format!("Failed to read directory: {}", e)))?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            AgentError::TaskExecutionFailed(format!("Failed to read directory entry: {}", e))
        })? {
            let path = entry.path();
            
            if path.is_file() {
                if let Ok(issues) = self.review_file(&path, config).await {
                    all_issues.extend(issues);
                    files_reviewed += 1;
                    
                    if let Ok(content) = tokio::fs::read_to_string(&path).await {
                        total_lines += content.lines().count();
                    }
                }
            } else if path.is_dir() && !self.should_exclude_file(&path, &config.exclude_files) {
                let sub_result = self.review_directory(&path, config).await?;
                all_issues.extend(sub_result.issues);
                files_reviewed += sub_result.files_reviewed;
                total_lines += sub_result.total_lines;
            }
        }

        let summary = self.generate_summary(&all_issues);

        Ok(ReviewResult {
            issues: all_issues,
            summary,
            files_reviewed,
            total_lines,
            review_duration: std::time::Duration::from_secs(0), // Will be set by caller
        })
        })
    }

    /// Analyze security vulnerabilities
    async fn analyze_security(
        &self,
        content: &str,
        file_path: &PathBuf,
    ) -> Result<Vec<ReviewIssue>, AgentError> {
        let prompt = format!(
            "Analyze this code for security vulnerabilities. Look for:
- SQL injection risks
- Cross-site scripting (XSS) vulnerabilities
- Insecure cryptographic practices
- Unsafe memory operations
- Input validation issues
- Authentication/authorization flaws
- Hardcoded secrets or credentials
- Path traversal vulnerabilities

Code:
```
{}
```

Return findings as JSON with format: {{\"issues\": [{{\"title\": \"Issue title\", \"description\": \"Detailed description\", \"line\": 10, \"severity\": \"High\", \"suggestion\": \"How to fix\"}}]}}",
            content
        );

        self.analyze_with_ai(&prompt, file_path, ReviewCategory::Security).await
    }

    /// Analyze performance issues
    async fn analyze_performance(
        &self,
        content: &str,
        file_path: &PathBuf,
    ) -> Result<Vec<ReviewIssue>, AgentError> {
        let prompt = format!(
            "Analyze this code for performance issues. Look for:
- Inefficient algorithms or data structures
- Unnecessary memory allocations
- Blocking operations in async code
- Expensive operations in loops
- Missing caching opportunities
- Database query inefficiencies
- Resource leaks
- Unnecessary cloning or copying

Code:
```
{}
```

Return findings as JSON with format: {{\"issues\": [{{\"title\": \"Issue title\", \"description\": \"Detailed description\", \"line\": 10, \"severity\": \"Medium\", \"suggestion\": \"How to optimize\"}}]}}",
            content
        );

        self.analyze_with_ai(&prompt, file_path, ReviewCategory::Performance).await
    }

    /// Analyze code smells and maintainability issues
    async fn analyze_code_smells(
        &self,
        content: &str,
        file_path: &PathBuf,
    ) -> Result<Vec<ReviewIssue>, AgentError> {
        let prompt = format!(
            "Analyze this code for code smells and maintainability issues. Look for:
- Long methods or functions
- Large classes or modules
- Duplicate code
- Complex conditional logic
- Poor naming conventions
- Tight coupling
- High cyclomatic complexity
- Dead code
- Magic numbers or strings
- Inconsistent coding style

Code:
```
{}
```

Return findings as JSON with format: {{\"issues\": [{{\"title\": \"Issue title\", \"description\": \"Detailed description\", \"line\": 10, \"severity\": \"Low\", \"suggestion\": \"How to refactor\"}}]}}",
            content
        );

        self.analyze_with_ai(&prompt, file_path, ReviewCategory::CodeSmell).await
    }

    /// Analyze documentation coverage
    async fn analyze_documentation(
        &self,
        content: &str,
        file_path: &PathBuf,
    ) -> Result<Vec<ReviewIssue>, AgentError> {
        let prompt = format!(
            "Analyze this code for documentation issues. Look for:
- Missing function/method documentation
- Undocumented public APIs
- Unclear or outdated comments
- Missing README or usage examples
- Undocumented complex algorithms
- Missing type annotations where beneficial
- Incomplete error handling documentation

Code:
```
{}
```

Return findings as JSON with format: {{\"issues\": [{{\"title\": \"Issue title\", \"description\": \"Detailed description\", \"line\": 10, \"severity\": \"Low\", \"suggestion\": \"Documentation to add\"}}]}}",
            content
        );

        self.analyze_with_ai(&prompt, file_path, ReviewCategory::Documentation).await
    }

    /// Analyze testing coverage and quality
    async fn analyze_testing(
        &self,
        content: &str,
        file_path: &PathBuf,
    ) -> Result<Vec<ReviewIssue>, AgentError> {
        let prompt = format!(
            "Analyze this code for testing issues. Look for:
- Functions without test coverage
- Missing edge case tests
- Poor test organization
- Tests that are too broad or too narrow
- Missing integration tests
- Flaky or unreliable tests
- Test code duplication
- Missing error condition tests

Code:
```
{}
```

Return findings as JSON with format: {{\"issues\": [{{\"title\": \"Issue title\", \"description\": \"Detailed description\", \"line\": 10, \"severity\": \"Medium\", \"suggestion\": \"Test to add\"}}]}}",
            content
        );

        self.analyze_with_ai(&prompt, file_path, ReviewCategory::Testing).await
    }

    /// Generic analysis for other categories
    async fn analyze_generic(
        &self,
        content: &str,
        file_path: &PathBuf,
        category: &ReviewCategory,
    ) -> Result<Vec<ReviewIssue>, AgentError> {
        let prompt = format!(
            "Analyze this code for {:?} issues. Provide specific, actionable feedback.

Code:
```
{}
```

Return findings as JSON with format: {{\"issues\": [{{\"title\": \"Issue title\", \"description\": \"Detailed description\", \"line\": 10, \"severity\": \"Medium\", \"suggestion\": \"How to fix\"}}]}}",
            category, content
        );

        self.analyze_with_ai(&prompt, file_path, category.clone()).await
    }

    /// Use AI to analyze code and return structured issues
    async fn analyze_with_ai(
        &self,
        prompt: &str,
        file_path: &PathBuf,
        category: ReviewCategory,
    ) -> Result<Vec<ReviewIssue>, AgentError> {
        if let Some(ai_manager) = &self.ai_manager {
            let response = ai_manager
                .generate_response(prompt, "", Some(1500), Some(0.1))
                .await
                .map_err(|e| AgentError::AIServiceError(format!("AI analysis failed: {}", e)))?;

            // Parse AI response - this is a simplified version
            // In practice, you'd want more robust JSON parsing
            let issues = self.parse_ai_response(&response, file_path, category)?;
            Ok(issues)
        } else {
            // Return mock issue when no AI manager is available
            Ok(vec![ReviewIssue {
                category: category.clone(),
                severity: ReviewSeverity::Low,
                title: format!("Basic {} check", format!("{:?}", category).to_lowercase()),
                description: "Basic static analysis - AI manager not available for detailed review".to_string(),
                file_path: file_path.clone(),
                line_start: Some(1),
                line_end: Some(1),
                suggestion: Some("Enable AI manager for detailed code review".to_string()),
                auto_fixable: false,
                code_snippet: None,
            }])
        }
    }

    /// Parse AI response into structured review issues
    fn parse_ai_response(
        &self,
        response: &str,
        file_path: &PathBuf,
        category: ReviewCategory,
    ) -> Result<Vec<ReviewIssue>, AgentError> {
        // Try to parse as JSON first, fallback to text parsing
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
            if let Some(issues_array) = parsed.get("issues").and_then(|v| v.as_array()) {
                let mut issues = Vec::new();
                for issue_val in issues_array {
                    if let Ok(issue) = self.parse_issue_from_json(issue_val, file_path, &category) {
                        issues.push(issue);
                    }
                }
                if !issues.is_empty() {
                    return Ok(issues);
                }
            }
        }
        
        // Fallback: create issue from plain text response
        Ok(vec![ReviewIssue {
            category: category.clone(),
            severity: ReviewSeverity::Medium,
            title: format!("{:?} Review Finding", category),
            description: response.chars().take(200).collect(),
            file_path: file_path.clone(),
            line_start: Some(1),
            line_end: Some(1),
            suggestion: Some("Review the AI analysis for more details".to_string()),
            auto_fixable: false,
            code_snippet: None,
        }])
    }
    
    /// Parse a single issue from JSON
    fn parse_issue_from_json(
        &self,
        json_val: &serde_json::Value,
        file_path: &PathBuf,
        category: &ReviewCategory,
    ) -> Result<ReviewIssue, AgentError> {
        let title = json_val.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown issue")
            .to_string();
            
        let description = json_val.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("No description provided")
            .to_string();
            
        let line = json_val.get("line")
            .and_then(|v| v.as_u64())
            .map(|l| l as usize);
            
        let severity_str = json_val.get("severity")
            .and_then(|v| v.as_str())
            .unwrap_or("Medium");
            
        let severity = match severity_str.to_lowercase().as_str() {
            "critical" => ReviewSeverity::Critical,
            "high" => ReviewSeverity::High,
            "medium" => ReviewSeverity::Medium,
            "low" => ReviewSeverity::Low,
            "info" => ReviewSeverity::Info,
            _ => ReviewSeverity::Medium,
        };
        
        let suggestion = json_val.get("suggestion")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        Ok(ReviewIssue {
            category: category.clone(),
            severity,
            title,
            description,
            file_path: file_path.clone(),
            line_start: line,
            line_end: line,
            suggestion,
            auto_fixable: false,
            code_snippet: None,
        })
    }

    /// Check if file should be excluded from review
    fn should_exclude_file(&self, file_path: &PathBuf, exclude_patterns: &[String]) -> bool {
        let path_str = file_path.to_string_lossy();
        exclude_patterns
            .iter()
            .any(|pattern| path_str.contains(pattern))
    }

    /// Generate summary statistics from review issues
    fn generate_summary(&self, issues: &[ReviewIssue]) -> ReviewSummary {
        let mut issues_by_severity = HashMap::new();
        let mut issues_by_category = HashMap::new();
        let mut auto_fixable_issues = 0;

        for issue in issues {
            let severity_key = format!("{:?}", issue.severity);
            *issues_by_severity.entry(severity_key).or_insert(0) += 1;

            let category_key = format!("{:?}", issue.category);
            *issues_by_category.entry(category_key).or_insert(0) += 1;

            if issue.auto_fixable {
                auto_fixable_issues += 1;
            }
        }

        let most_common_issue = issues_by_category
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(category, _)| category.clone());

        ReviewSummary {
            total_issues: issues.len(),
            issues_by_severity,
            issues_by_category,
            auto_fixable_issues,
            most_common_issue,
        }
    }
}

#[async_trait::async_trait]
impl Agent for CodeReviewAgent {
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
        self.base.status = AgentStatus::Processing { task_id: task.id.clone() };
        
        let start_time = std::time::Instant::now();
        
        // Parse task parameters for code review
        let review_config = serde_json::from_str::<ReviewConfig>(&task.task_type)
            .unwrap_or_else(|_| ReviewConfig::default());

        // Extract file paths from task context  
        let paths = vec![PathBuf::from(".")];

        // Perform the code review
        let review_result = self.review_code(&paths, review_config).await?;

        // Convert result to JSON for the agent result
        let result_json = serde_json::to_string_pretty(&review_result)
            .map_err(|e| AgentError::TaskExecutionFailed(format!("Failed to serialize result: {}", e)))?;

        // Create artifact
        let artifact = AgentArtifact::new(
            "review_report".to_string(),
            "analysis".to_string(),
            result_json.clone(),
        )
        .with_metadata("total_issues".to_string(), json!(review_result.issues.len()))
        .with_metadata("files_reviewed".to_string(), json!(review_result.files_reviewed));
        
        // Update metrics
        let duration = start_time.elapsed();
        self.base.update_metrics(true, duration);
        self.base.status = AgentStatus::Idle;

        Ok(AgentResult::success(
            task.id.clone(),
            self.base.id.clone(),
            format!("Code review completed: {} issues found in {} files", 
                   review_result.summary.total_issues, review_result.files_reviewed),
        )
        .with_artifact(artifact)
        .with_duration(duration)
        .with_next_action("Review findings and consider implementing fixes".to_string()))
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
