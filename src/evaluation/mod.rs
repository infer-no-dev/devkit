use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use std::sync::Arc;

pub mod framework;
pub mod quality_gates;
pub mod regression;
pub mod benchmarks;
pub mod metrics;
pub mod reports;

use crate::agents::AgentResult;
use crate::codegen::GeneratedCode;

/// Main evaluation framework coordinating quality assurance
#[derive(Debug, Clone)]
pub struct EvaluationFramework {
    config: EvaluationConfig,
    quality_gates: quality_gates::QualityGateManager,
    regression_detector: regression::RegressionDetector,
    benchmark_runner: benchmarks::BenchmarkRunner,
    metrics_collector: metrics::MetricsCollector,
    report_generator: reports::ReportGenerator,
    evaluation_history: Arc<RwLock<Vec<EvaluationResult>>>,
}

/// Configuration for evaluation framework
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationConfig {
    pub quality_gates: quality_gates::QualityGateConfig,
    pub regression: regression::RegressionConfig,
    pub benchmarks: benchmarks::BenchmarkConfig,
    pub metrics: metrics::MetricsConfig,
    pub reporting: reports::ReportConfig,
    pub parallel_execution: bool,
    pub timeout: Duration,
    pub retry_attempts: u32,
}

/// Overall evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub duration: Duration,
    pub overall_score: f64,
    pub quality_gate_results: Vec<quality_gates::QualityGateResult>,
    pub regression_results: Vec<regression::RegressionResult>,
    pub benchmark_results: Vec<benchmarks::BenchmarkResult>,
    pub metrics: HashMap<String, metrics::MetricValue>,
    pub success: bool,
    pub issues: Vec<EvaluationIssue>,
    pub recommendations: Vec<String>,
}

/// Issue found during evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationIssue {
    pub id: String,
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub description: String,
    pub location: Option<IssueLocation>,
    pub suggested_fix: Option<String>,
    pub confidence: f64,
}

/// Issue severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Issue categories
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum IssueCategory {
    CodeQuality,
    Performance,
    Security,
    Maintainability,
    Testing,
    Documentation,
    Compliance,
    Regression,
}

/// Location information for issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueLocation {
    pub file_path: String,
    pub line_range: Option<(u32, u32)>,
    pub function: Option<String>,
    pub module: Option<String>,
}

/// Evaluation context for running assessments
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    pub project_path: String,
    pub target_files: Vec<String>,
    pub baseline_commit: Option<String>,
    pub environment: EvaluationEnvironment,
    pub metadata: HashMap<String, String>,
}

/// Environment information for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationEnvironment {
    pub platform: String,
    pub architecture: String,
    pub rust_version: String,
    pub dependencies: HashMap<String, String>,
    pub build_profile: BuildProfile,
}

/// Build profile for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildProfile {
    Debug,
    Release,
    Test,
    Benchmark,
}

impl EvaluationFramework {
    /// Create new evaluation framework with configuration
    pub fn new(config: EvaluationConfig) -> Self {
        Self {
            quality_gates: quality_gates::QualityGateManager::new(config.quality_gates.clone()),
            regression_detector: regression::RegressionDetector::new(config.regression.clone()),
            benchmark_runner: benchmarks::BenchmarkRunner::new(config.benchmarks.clone()),
            metrics_collector: metrics::MetricsCollector::new(config.metrics.clone()),
            report_generator: reports::ReportGenerator::new(config.reporting.clone()),
            config,
            evaluation_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Run complete evaluation on generated code
    pub async fn evaluate_generated_code(
        &self,
        code: &GeneratedCode,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult, EvaluationError> {
        let start_time = Instant::now();
        let evaluation_id = uuid::Uuid::new_v4().to_string();

        let mut result = EvaluationResult {
            id: evaluation_id.clone(),
            timestamp: chrono::Utc::now(),
            duration: Duration::from_secs(0),
            overall_score: 0.0,
            quality_gate_results: Vec::new(),
            regression_results: Vec::new(),
            benchmark_results: Vec::new(),
            metrics: HashMap::new(),
            success: true,
            issues: Vec::new(),
            recommendations: Vec::new(),
        };

        // Run evaluations
        if self.config.parallel_execution {
            self.run_parallel_evaluation(code, context, &mut result).await?;
        } else {
            self.run_sequential_evaluation(code, context, &mut result).await?;
        }

        // Calculate overall metrics
        result.duration = start_time.elapsed();
        result.overall_score = self.calculate_overall_score(&result);
        result.success = self.determine_success(&result);
        result.recommendations = self.generate_recommendations(&result);

        // Store result
        self.evaluation_history.write().await.push(result.clone());

        Ok(result)
    }

    /// Evaluate agent results
    pub async fn evaluate_agent_result(
        &self,
        agent_result: &AgentResult,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult, EvaluationError> {
        // Convert agent result to evaluation format
        let code_content = agent_result.artifacts
            .iter()
            .find(|a| a.artifact_type == "code" || a.artifact_type == "source_code")
            .map(|a| a.content.clone())
            .unwrap_or_default();
        
        let language = agent_result.metadata.get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        
        let file_path = agent_result.metadata.get("file_path")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
            
        let mut metadata = HashMap::new();
        for (key, value) in &agent_result.metadata {
            if let Some(str_value) = value.as_str() {
                metadata.insert(key.clone(), str_value.to_string());
            }
        }
        
        let generated_code = GeneratedCode {
            content: code_content,
            language,
            file_path,
            dependencies: Vec::new(),
            metadata,
        };

        self.evaluate_generated_code(&generated_code, context).await
    }

    /// Run parallel evaluation
    async fn run_parallel_evaluation(
        &self,
        code: &GeneratedCode,
        context: &EvaluationContext,
        result: &mut EvaluationResult,
    ) -> Result<(), EvaluationError> {
        let (quality_gates, regression_results, benchmark_results, metrics) = tokio::join!(
            self.quality_gates.run_gates(code, context),
            self.regression_detector.detect_regressions(code, context),
            self.benchmark_runner.run_benchmarks(code, context),
            self.metrics_collector.collect_metrics(code, context)
        );

        result.quality_gate_results = quality_gates?;
        result.regression_results = regression_results?;
        result.benchmark_results = benchmark_results?;
        result.metrics = metrics?;

        Ok(())
    }

    /// Run sequential evaluation
    async fn run_sequential_evaluation(
        &self,
        code: &GeneratedCode,
        context: &EvaluationContext,
        result: &mut EvaluationResult,
    ) -> Result<(), EvaluationError> {
        result.quality_gate_results = self.quality_gates.run_gates(code, context).await?;
        result.regression_results = self.regression_detector.detect_regressions(code, context).await?;
        result.benchmark_results = self.benchmark_runner.run_benchmarks(code, context).await?;
        result.metrics = self.metrics_collector.collect_metrics(code, context).await?;

        Ok(())
    }

    /// Calculate overall quality score
    fn calculate_overall_score(&self, result: &EvaluationResult) -> f64 {
        let mut total_score = 0.0;
        let mut weight_sum = 0.0;

        // Quality gates score (40% weight)
        let quality_score = result.quality_gate_results.iter()
            .map(|r| r.score)
            .sum::<f64>() / result.quality_gate_results.len().max(1) as f64;
        total_score += quality_score * 0.4;
        weight_sum += 0.4;

        // Regression score (30% weight)
        let regression_penalty = result.regression_results.iter()
            .filter(|r| r.is_regression)
            .count() as f64 * 10.0;
        let regression_score = (100.0 - regression_penalty).max(0.0);
        total_score += regression_score * 0.3;
        weight_sum += 0.3;

        // Benchmark score (20% weight)
        let benchmark_score = result.benchmark_results.iter()
            .map(|r| if r.passed { 100.0 } else { 0.0 })
            .sum::<f64>() / result.benchmark_results.len().max(1) as f64;
        total_score += benchmark_score * 0.2;
        weight_sum += 0.2;

        // Issue severity penalty (10% weight)
        let issue_penalty = result.issues.iter()
            .map(|issue| match issue.severity {
                IssueSeverity::Critical => 25.0,
                IssueSeverity::High => 15.0,
                IssueSeverity::Medium => 8.0,
                IssueSeverity::Low => 3.0,
                IssueSeverity::Info => 1.0,
            })
            .sum::<f64>();
        let issue_score = (100.0 - issue_penalty).max(0.0);
        total_score += issue_score * 0.1;
        weight_sum += 0.1;

        if weight_sum > 0.0 {
            total_score / weight_sum
        } else {
            0.0
        }
    }

    /// Determine if evaluation was successful
    fn determine_success(&self, result: &EvaluationResult) -> bool {
        let has_critical_issues = result.issues.iter()
            .any(|issue| issue.severity == IssueSeverity::Critical);

        let quality_gates_passed = result.quality_gate_results.iter()
            .all(|gate| gate.passed);

        let no_regressions = !result.regression_results.iter()
            .any(|r| r.is_regression && r.severity >= regression::RegressionSeverity::High);

        !has_critical_issues && quality_gates_passed && no_regressions && result.overall_score >= 70.0
    }

    /// Generate recommendations based on evaluation results
    fn generate_recommendations(&self, result: &EvaluationResult) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Score-based recommendations
        if result.overall_score < 50.0 {
            recommendations.push("Overall code quality is below acceptable threshold. Consider major refactoring.".to_string());
        } else if result.overall_score < 70.0 {
            recommendations.push("Code quality needs improvement. Focus on addressing high-severity issues.".to_string());
        }

        // Issue-based recommendations
        let critical_count = result.issues.iter().filter(|i| i.severity == IssueSeverity::Critical).count();
        if critical_count > 0 {
            recommendations.push(format!("Address {} critical issues immediately before proceeding.", critical_count));
        }

        // Regression-based recommendations
        let regression_count = result.regression_results.iter().filter(|r| r.is_regression).count();
        if regression_count > 0 {
            recommendations.push(format!("Fix {} detected regressions to maintain code quality.", regression_count));
        }

        // Performance recommendations
        let slow_benchmarks = result.benchmark_results.iter()
            .filter(|b| !b.passed)
            .count();
        if slow_benchmarks > 0 {
            recommendations.push("Optimize performance in failing benchmarks.".to_string());
        }

        recommendations
    }

    /// Get evaluation history
    pub async fn get_evaluation_history(&self) -> Vec<EvaluationResult> {
        self.evaluation_history.read().await.clone()
    }

    /// Generate comprehensive report
    pub async fn generate_report(
        &self,
        results: &[EvaluationResult],
        format: reports::ReportFormat,
    ) -> Result<String, EvaluationError> {
        self.report_generator.generate_report(results, format).await
            .map_err(|e| EvaluationError::Report(e))
    }
}

/// Evaluation framework errors
#[derive(Debug, thiserror::Error)]
pub enum EvaluationError {
    #[error("Quality gate error: {0}")]
    QualityGate(#[from] quality_gates::QualityGateError),
    #[error("Regression detection error: {0}")]
    Regression(#[from] regression::RegressionError),
    #[error("Benchmark error: {0}")]
    Benchmark(#[from] benchmarks::BenchmarkError),
    #[error("Metrics collection error: {0}")]
    Metrics(#[from] metrics::MetricsError),
    #[error("Report generation error: {0}")]
    Report(#[from] reports::ReportError),
    #[error("Timeout error: evaluation took too long")]
    Timeout,
    #[error("Configuration error: {0}")]
    Configuration(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl Default for EvaluationConfig {
    fn default() -> Self {
        Self {
            quality_gates: quality_gates::QualityGateConfig::default(),
            regression: regression::RegressionConfig::default(),
            benchmarks: benchmarks::BenchmarkConfig::default(),
            metrics: metrics::MetricsConfig::default(),
            reporting: reports::ReportConfig::default(),
            parallel_execution: true,
            timeout: Duration::from_secs(300), // 5 minutes
            retry_attempts: 3,
        }
    }
}