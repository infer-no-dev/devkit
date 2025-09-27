//! Services Module
//!
//! This module demonstrates the comprehensive error handling patterns and provides
//! higher-level services that coordinate between different components.

use crate::ai::{AIManager, ChatMessage, ChatRequest};
use crate::config::ConfigManager;
use crate::error::{DevKitError, DevKitResult, ErrorContext, ErrorHandler, RecoveryStrategy};
use crate::shell::ShellManager;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{sleep, Duration};

/// High-level service that coordinates AI-powered code analysis
pub struct CodeAnalysisService {
    ai_manager: Arc<AIManager>,
    shell_manager: Arc<ShellManager>,
    config_manager: Arc<ConfigManager>,
    error_handler: ErrorHandler,
}

impl CodeAnalysisService {
    pub fn new(
        ai_manager: Arc<AIManager>,
        shell_manager: Arc<ShellManager>,
        config_manager: Arc<ConfigManager>,
    ) -> Self {
        Self {
            ai_manager,
            shell_manager,
            config_manager,
            error_handler: ErrorHandler::default(),
        }
    }

    /// Analyze code with comprehensive error handling and recovery
    pub async fn analyze_codebase(
        &self,
        project_path: &str,
        analysis_type: AnalysisType,
    ) -> DevKitResult<AnalysisReport> {
        let _context = ErrorContext::new("analyze_codebase", "CodeAnalysisService").with_details(
            &format!("Analyzing {} with {:?}", project_path, analysis_type),
        );

        self.analyze_codebase_with_recovery(project_path, analysis_type, 3)
            .await
    }

    async fn analyze_codebase_with_recovery(
        &self,
        project_path: &str,
        analysis_type: AnalysisType,
        max_retries: usize,
    ) -> DevKitResult<AnalysisReport> {
        let mut attempt = 0;

        loop {
            attempt += 1;

            let result = self.perform_analysis(project_path, analysis_type).await;

            match result {
                Ok(report) => return Ok(report),
                Err(error) => {
                    if attempt >= max_retries {
                        return Err(error);
                    }

                    let strategy = self.error_handler.handle_error(&error).await;

                    match strategy {
                        RecoveryStrategy::Retry { delay_ms, .. } => {
                            println!(
                                "Analysis attempt {} failed, retrying in {}ms: {}",
                                attempt, delay_ms, error
                            );
                            sleep(Duration::from_millis(delay_ms)).await;
                        }
                        RecoveryStrategy::RetryWithBackoff {
                            base_delay_ms,
                            max_delay_ms,
                            ..
                        } => {
                            let delay = std::cmp::min(
                                base_delay_ms * 2_u64.pow(attempt as u32 - 1),
                                max_delay_ms,
                            );
                            println!(
                                "Analysis attempt {} failed, retrying with backoff in {}ms: {}",
                                attempt, delay, error
                            );
                            sleep(Duration::from_millis(delay)).await;
                        }
                        RecoveryStrategy::Fallback => {
                            println!("Analysis failed, using fallback method: {}", error);
                            return self.fallback_analysis(project_path).await;
                        }
                        RecoveryStrategy::UserIntervention => {
                            return Err(DevKitError::UserError {
                                message: format!(
                                    "Analysis failed and requires user intervention: {}. \
                                     Please check your configuration and try again.",
                                    error
                                ),
                            });
                        }
                        RecoveryStrategy::Skip => {
                            println!("Skipping analysis due to error: {}", error);
                            return Ok(AnalysisReport::empty(project_path));
                        }
                        RecoveryStrategy::FailFast => {
                            return Err(error);
                        }
                    }
                }
            }
        }
    }

    async fn perform_analysis(
        &self,
        project_path: &str,
        analysis_type: AnalysisType,
    ) -> DevKitResult<AnalysisReport> {
        let start_time = Instant::now();

        // Step 1: Validate project path
        let project_info = self.validate_project_path(project_path).await?;

        // Step 2: Gather project context
        let context = self.gather_context(project_path).await?;

        // Step 3: Run analysis based on type
        let analysis_results = match analysis_type {
            AnalysisType::Security => self.run_security_analysis(&context).await?,
            AnalysisType::Performance => self.run_performance_analysis(&context).await?,
            AnalysisType::Quality => self.run_quality_analysis(&context).await?,
            AnalysisType::Dependencies => self.run_dependency_analysis(&context).await?,
            AnalysisType::Comprehensive => self.run_comprehensive_analysis(&context).await?,
        };

        let duration = start_time.elapsed();

        Ok(AnalysisReport {
            project_path: project_path.to_string(),
            analysis_type,
            project_info,
            results: analysis_results,
            duration_ms: duration.as_millis() as u64,
            timestamp: std::time::SystemTime::now(),
        })
    }

    async fn validate_project_path(&self, path: &str) -> DevKitResult<ProjectInfo> {
        // Check if path exists
        if !std::path::Path::new(path).exists() {
            return Err(DevKitError::ResourceNotFound {
                resource_type: "project directory".to_string(),
                name: path.to_string(),
            });
        }

        // Run basic project detection commands
        let git_result = self
            .shell_manager
            .execute_command("git rev-parse --show-toplevel", None)
            .await;
        let cargo_check = std::path::Path::new(path).join("Cargo.toml").exists();
        let package_check = std::path::Path::new(path).join("package.json").exists();

        let project_type = if cargo_check {
            ProjectType::Rust
        } else if package_check {
            ProjectType::JavaScript
        } else if git_result.is_ok() {
            ProjectType::Git
        } else {
            ProjectType::Unknown
        };

        Ok(ProjectInfo {
            path: path.to_string(),
            project_type,
            has_git: git_result.is_ok(),
            has_cargo: cargo_check,
            has_package_json: package_check,
        })
    }

    async fn gather_context(&self, _path: &str) -> DevKitResult<ProjectContext> {
        // This would normally use the real context manager
        // For now, return a placeholder
        Ok(ProjectContext {
            files_analyzed: 0,
            total_lines: 0,
            languages: vec![],
        })
    }

    async fn run_security_analysis(
        &self,
        _context: &ProjectContext,
    ) -> DevKitResult<Vec<AnalysisResult>> {
        // Use AI to analyze for security issues
        let request = ChatRequest {
            model: self.ai_manager.default_model().to_string(),
            messages: vec![
                ChatMessage::system("You are a security analysis expert. Analyze the provided code for security vulnerabilities."),
                ChatMessage::user("Analyze this project for common security issues like SQL injection, XSS, authentication problems, etc.")
            ],
            parameters: None,
            stream: false,
        };

        let response = self
            .ai_manager
            .chat_completion_default(request)
            .await
            .map_err(|e| DevKitError::AI(e))?;

        // Parse AI response into structured results
        Ok(vec![AnalysisResult {
            category: "security".to_string(),
            severity: Severity::Medium,
            message: response.message.content,
            file_path: None,
            line_number: None,
            suggestions: vec![],
        }])
    }

    async fn run_performance_analysis(
        &self,
        _context: &ProjectContext,
    ) -> DevKitResult<Vec<AnalysisResult>> {
        // Implementation would analyze for performance issues
        Ok(vec![])
    }

    async fn run_quality_analysis(
        &self,
        _context: &ProjectContext,
    ) -> DevKitResult<Vec<AnalysisResult>> {
        // Implementation would analyze code quality
        Ok(vec![])
    }

    async fn run_dependency_analysis(
        &self,
        _context: &ProjectContext,
    ) -> DevKitResult<Vec<AnalysisResult>> {
        // Implementation would analyze dependencies
        Ok(vec![])
    }

    async fn run_comprehensive_analysis(
        &self,
        context: &ProjectContext,
    ) -> DevKitResult<Vec<AnalysisResult>> {
        let mut all_results = Vec::new();

        // Run all analysis types and combine results
        let security_results = self
            .run_security_analysis(context)
            .await
            .unwrap_or_default();
        let performance_results = self
            .run_performance_analysis(context)
            .await
            .unwrap_or_default();
        let quality_results = self.run_quality_analysis(context).await.unwrap_or_default();
        let dependency_results = self
            .run_dependency_analysis(context)
            .await
            .unwrap_or_default();

        all_results.extend(security_results);
        all_results.extend(performance_results);
        all_results.extend(quality_results);
        all_results.extend(dependency_results);

        Ok(all_results)
    }

    async fn fallback_analysis(&self, project_path: &str) -> DevKitResult<AnalysisReport> {
        // Provide basic analysis when AI services fail
        println!("Using fallback analysis for {}", project_path);

        let project_info = ProjectInfo {
            path: project_path.to_string(),
            project_type: ProjectType::Unknown,
            has_git: false,
            has_cargo: false,
            has_package_json: false,
        };

        Ok(AnalysisReport {
            project_path: project_path.to_string(),
            analysis_type: AnalysisType::Quality, // Default fallback type
            project_info,
            results: vec![AnalysisResult {
                category: "fallback".to_string(),
                severity: Severity::Info,
                message: "Analysis completed using fallback method due to service unavailability"
                    .to_string(),
                file_path: None,
                line_number: None,
                suggestions: vec!["Retry when services are available".to_string()],
            }],
            duration_ms: 0,
            timestamp: std::time::SystemTime::now(),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AnalysisType {
    Security,
    Performance,
    Quality,
    Dependencies,
    Comprehensive,
}

#[derive(Debug, Clone)]
pub struct AnalysisReport {
    pub project_path: String,
    pub analysis_type: AnalysisType,
    pub project_info: ProjectInfo,
    pub results: Vec<AnalysisResult>,
    pub duration_ms: u64,
    pub timestamp: std::time::SystemTime,
}

impl AnalysisReport {
    pub fn empty(project_path: &str) -> Self {
        Self {
            project_path: project_path.to_string(),
            analysis_type: AnalysisType::Quality,
            project_info: ProjectInfo {
                path: project_path.to_string(),
                project_type: ProjectType::Unknown,
                has_git: false,
                has_cargo: false,
                has_package_json: false,
            },
            results: vec![],
            duration_ms: 0,
            timestamp: std::time::SystemTime::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub path: String,
    pub project_type: ProjectType,
    pub has_git: bool,
    pub has_cargo: bool,
    pub has_package_json: bool,
}

#[derive(Debug, Clone)]
pub enum ProjectType {
    Rust,
    JavaScript,
    Python,
    Git,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ProjectContext {
    pub files_analyzed: usize,
    pub total_lines: usize,
    pub languages: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub category: String,
    pub severity: Severity,
    pub message: String,
    pub file_path: Option<String>,
    pub line_number: Option<usize>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_error_recovery_patterns() {
        // This would test the error recovery mechanisms
        // Implementation would create mock services and test various error scenarios
    }
}
