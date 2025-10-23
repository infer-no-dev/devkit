use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};

use crate::codegen::GeneratedCode;
use crate::evaluation::EvaluationContext;

/// Metrics collector for evaluation framework
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    config: MetricsConfig,
    collected_metrics: HashMap<String, MetricValue>,
}

/// Configuration for metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled_metrics: Vec<MetricType>,
    pub code_metrics: CodeMetricsConfig,
    pub performance_metrics: PerformanceMetricsConfig,
    pub quality_metrics: QualityMetricsConfig,
    pub process_metrics: ProcessMetricsConfig,
    pub retention_period: Duration,
    pub aggregation_window: Duration,
}

/// Types of metrics to collect
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum MetricType {
    CodeComplexity,
    TestCoverage,
    CodeDuplication,
    Performance,
    MemoryUsage,
    CompileTime,
    Documentation,
    Dependencies,
    Security,
    Custom(String),
}

/// Code metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetricsConfig {
    pub complexity_analysis: bool,
    pub duplication_detection: bool,
    pub maintainability_index: bool,
    pub technical_debt: bool,
}

/// Performance metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetricsConfig {
    pub execution_time: bool,
    pub throughput: bool,
    pub latency_percentiles: Vec<u8>,
    pub resource_utilization: bool,
}

/// Quality metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetricsConfig {
    pub test_metrics: bool,
    pub code_coverage: bool,
    pub static_analysis: bool,
    pub security_scan: bool,
}

/// Process metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMetricsConfig {
    pub compilation_metrics: bool,
    pub build_metrics: bool,
    pub deployment_metrics: bool,
}

/// Metric value with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    pub value: f64,
    pub unit: String,
    pub timestamp: SystemTime,
    pub metadata: HashMap<String, String>,
    pub tags: Vec<String>,
}

/// Code complexity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeComplexityMetrics {
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub halstead_complexity: f64,
    pub lines_of_code: u32,
    pub logical_lines_of_code: u32,
    pub comment_ratio: f64,
    pub function_count: u32,
    pub class_count: u32,
    pub max_nesting_depth: u32,
}

/// Test coverage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCoverageMetrics {
    pub line_coverage: f64,
    pub branch_coverage: f64,
    pub function_coverage: f64,
    pub statement_coverage: f64,
    pub uncovered_lines: Vec<u32>,
    pub total_lines: u32,
    pub tests_passed: u32,
    pub tests_failed: u32,
    pub tests_skipped: u32,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub execution_time_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_utilization: f64,
    pub io_operations: u64,
    pub network_requests: u64,
    pub cache_hit_rate: f64,
    pub throughput_ops_per_sec: f64,
    pub latency_percentiles: HashMap<u8, f64>,
}

/// Quality assessment metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub maintainability_index: f64,
    pub technical_debt_hours: f64,
    pub code_smells_count: u32,
    pub duplicated_lines_percent: f64,
    pub security_hotspots: u32,
    pub vulnerability_count: u32,
    pub code_quality_score: f64,
}

impl MetricsCollector {
    /// Create new metrics collector
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            config,
            collected_metrics: HashMap::new(),
        }
    }

    /// Collect all enabled metrics
    pub async fn collect_metrics(
        &self,
        code: &GeneratedCode,
        context: &EvaluationContext,
    ) -> Result<HashMap<String, MetricValue>, MetricsError> {
        let mut metrics = HashMap::new();

        for metric_type in &self.config.enabled_metrics {
            let collected = self.collect_metric_type(metric_type, code, context).await?;
            metrics.extend(collected);
        }

        Ok(metrics)
    }

    /// Collect specific metric type
    async fn collect_metric_type(
        &self,
        metric_type: &MetricType,
        code: &GeneratedCode,
        context: &EvaluationContext,
    ) -> Result<HashMap<String, MetricValue>, MetricsError> {
        match metric_type {
            MetricType::CodeComplexity => self.collect_code_complexity_metrics(code, context).await,
            MetricType::TestCoverage => self.collect_test_coverage_metrics(code, context).await,
            MetricType::CodeDuplication => self.collect_duplication_metrics(code, context).await,
            MetricType::Performance => self.collect_performance_metrics(code, context).await,
            MetricType::MemoryUsage => self.collect_memory_metrics(code, context).await,
            MetricType::CompileTime => self.collect_compile_time_metrics(code, context).await,
            MetricType::Documentation => self.collect_documentation_metrics(code, context).await,
            MetricType::Dependencies => self.collect_dependency_metrics(code, context).await,
            MetricType::Security => self.collect_security_metrics(code, context).await,
            MetricType::Custom(name) => self.collect_custom_metrics(name, code, context).await,
        }
    }

    /// Collect code complexity metrics
    async fn collect_code_complexity_metrics(
        &self,
        code: &GeneratedCode,
        _context: &EvaluationContext,
    ) -> Result<HashMap<String, MetricValue>, MetricsError> {
        let mut metrics = HashMap::new();

        // Simplified complexity analysis - in real implementation would use proper AST analysis
        let lines: Vec<&str> = code.content.lines().collect();
        let total_lines = lines.len() as u32;
        let code_lines = lines.iter().filter(|line| !line.trim().is_empty() && !line.trim().starts_with("//")).count() as u32;
        let comment_lines = lines.iter().filter(|line| line.trim().starts_with("//")).count() as u32;
        
        let comment_ratio = if total_lines > 0 {
            comment_lines as f64 / total_lines as f64
        } else {
            0.0
        };

        // Estimate cyclomatic complexity based on control flow keywords
        let control_keywords = ["if", "else", "while", "for", "loop", "match", "catch"];
        let cyclomatic_complexity = control_keywords.iter()
            .map(|keyword| code.content.matches(keyword).count() as u32)
            .sum::<u32>() + 1;

        // Function count estimation
        let function_count = code.content.matches("fn ").count() as u32;

        metrics.insert("cyclomatic_complexity".to_string(), MetricValue {
            value: cyclomatic_complexity as f64,
            unit: "count".to_string(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
            tags: vec!["complexity".to_string()],
        });

        metrics.insert("lines_of_code".to_string(), MetricValue {
            value: total_lines as f64,
            unit: "lines".to_string(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
            tags: vec!["size".to_string()],
        });

        metrics.insert("logical_lines_of_code".to_string(), MetricValue {
            value: code_lines as f64,
            unit: "lines".to_string(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
            tags: vec!["size".to_string()],
        });

        metrics.insert("comment_ratio".to_string(), MetricValue {
            value: comment_ratio,
            unit: "ratio".to_string(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
            tags: vec!["documentation".to_string()],
        });

        metrics.insert("function_count".to_string(), MetricValue {
            value: function_count as f64,
            unit: "count".to_string(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
            tags: vec!["structure".to_string()],
        });

        Ok(metrics)
    }

    /// Collect test coverage metrics (stub implementation)
    async fn collect_test_coverage_metrics(
        &self,
        _code: &GeneratedCode,
        _context: &EvaluationContext,
    ) -> Result<HashMap<String, MetricValue>, MetricsError> {
        let mut metrics = HashMap::new();

        // Mock test coverage data
        metrics.insert("line_coverage".to_string(), MetricValue {
            value: 85.5,
            unit: "percent".to_string(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
            tags: vec!["testing".to_string()],
        });

        metrics.insert("branch_coverage".to_string(), MetricValue {
            value: 78.2,
            unit: "percent".to_string(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
            tags: vec!["testing".to_string()],
        });

        Ok(metrics)
    }

    /// Collect code duplication metrics (stub implementation)
    async fn collect_duplication_metrics(
        &self,
        code: &GeneratedCode,
        _context: &EvaluationContext,
    ) -> Result<HashMap<String, MetricValue>, MetricsError> {
        let mut metrics = HashMap::new();

        // Simple duplication detection based on repeated lines
        let lines: Vec<&str> = code.content.lines().collect();
        let mut line_counts = HashMap::new();
        
        for line in &lines {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with("//") {
                *line_counts.entry(trimmed).or_insert(0) += 1;
            }
        }

        let duplicated_lines = line_counts.values().filter(|&&count| count > 1).sum::<u32>();
        let total_lines = lines.len() as u32;
        let duplication_ratio = if total_lines > 0 {
            duplicated_lines as f64 / total_lines as f64 * 100.0
        } else {
            0.0
        };

        metrics.insert("duplicated_lines_percent".to_string(), MetricValue {
            value: duplication_ratio,
            unit: "percent".to_string(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
            tags: vec!["quality".to_string()],
        });

        Ok(metrics)
    }

    /// Stub implementations for remaining metric types
    async fn collect_performance_metrics(&self, _code: &GeneratedCode, _context: &EvaluationContext) -> Result<HashMap<String, MetricValue>, MetricsError> {
        let mut metrics = HashMap::new();
        
        metrics.insert("execution_time_ms".to_string(), MetricValue {
            value: 125.5,
            unit: "milliseconds".to_string(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
            tags: vec!["performance".to_string()],
        });

        Ok(metrics)
    }

    async fn collect_memory_metrics(&self, _code: &GeneratedCode, _context: &EvaluationContext) -> Result<HashMap<String, MetricValue>, MetricsError> {
        let mut metrics = HashMap::new();
        
        metrics.insert("memory_usage_mb".to_string(), MetricValue {
            value: 45.2,
            unit: "megabytes".to_string(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
            tags: vec!["memory".to_string()],
        });

        Ok(metrics)
    }

    async fn collect_compile_time_metrics(&self, _code: &GeneratedCode, _context: &EvaluationContext) -> Result<HashMap<String, MetricValue>, MetricsError> {
        let mut metrics = HashMap::new();
        
        metrics.insert("compile_time_ms".to_string(), MetricValue {
            value: 3500.0,
            unit: "milliseconds".to_string(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
            tags: vec!["compilation".to_string()],
        });

        Ok(metrics)
    }

    async fn collect_documentation_metrics(&self, code: &GeneratedCode, _context: &EvaluationContext) -> Result<HashMap<String, MetricValue>, MetricsError> {
        let mut metrics = HashMap::new();
        
        // Simple documentation coverage based on doc comments
        let lines: Vec<&str> = code.content.lines().collect();
        let doc_comments = lines.iter().filter(|line| line.trim().starts_with("///")).count();
        let functions = code.content.matches("fn ").count();
        
        let doc_coverage = if functions > 0 {
            (doc_comments as f64 / functions as f64) * 100.0
        } else {
            0.0
        };
        
        metrics.insert("documentation_coverage".to_string(), MetricValue {
            value: doc_coverage,
            unit: "percent".to_string(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
            tags: vec!["documentation".to_string()],
        });

        Ok(metrics)
    }

    async fn collect_dependency_metrics(&self, _code: &GeneratedCode, _context: &EvaluationContext) -> Result<HashMap<String, MetricValue>, MetricsError> {
        let mut metrics = HashMap::new();
        
        metrics.insert("dependency_count".to_string(), MetricValue {
            value: 15.0,
            unit: "count".to_string(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
            tags: vec!["dependencies".to_string()],
        });

        Ok(metrics)
    }

    async fn collect_security_metrics(&self, _code: &GeneratedCode, _context: &EvaluationContext) -> Result<HashMap<String, MetricValue>, MetricsError> {
        let mut metrics = HashMap::new();
        
        metrics.insert("security_score".to_string(), MetricValue {
            value: 92.0,
            unit: "score".to_string(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
            tags: vec!["security".to_string()],
        });

        Ok(metrics)
    }

    async fn collect_custom_metrics(&self, name: &str, _code: &GeneratedCode, _context: &EvaluationContext) -> Result<HashMap<String, MetricValue>, MetricsError> {
        let mut metrics = HashMap::new();
        
        metrics.insert(format!("custom_{}", name), MetricValue {
            value: 100.0,
            unit: "custom".to_string(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
            tags: vec!["custom".to_string()],
        });

        Ok(metrics)
    }

    /// Get historical metrics for comparison
    pub async fn get_historical_metrics(
        &self,
        metric_name: &str,
        time_range: Duration,
    ) -> Result<Vec<MetricValue>, MetricsError> {
        // Stub implementation - would query historical data storage
        Ok(Vec::new())
    }

    /// Calculate metric trends
    pub fn calculate_trend(&self, values: &[MetricValue]) -> MetricTrend {
        if values.len() < 2 {
            return MetricTrend::Stable;
        }

        let first_value = values.first().unwrap().value;
        let last_value = values.last().unwrap().value;
        let change_percent = ((last_value - first_value) / first_value) * 100.0;

        if change_percent > 10.0 {
            MetricTrend::Improving
        } else if change_percent < -10.0 {
            MetricTrend::Degrading
        } else {
            MetricTrend::Stable
        }
    }
}

/// Metric trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricTrend {
    Improving,
    Stable,
    Degrading,
    Volatile,
}

/// Metrics collection errors
#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    #[error("Collection failed: {0}")]
    CollectionFailed(String),
    #[error("Invalid metric type: {0}")]
    InvalidMetricType(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Analysis error: {0}")]
    Analysis(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled_metrics: vec![
                MetricType::CodeComplexity,
                MetricType::TestCoverage,
                MetricType::Performance,
                MetricType::Documentation,
            ],
            code_metrics: CodeMetricsConfig::default(),
            performance_metrics: PerformanceMetricsConfig::default(),
            quality_metrics: QualityMetricsConfig::default(),
            process_metrics: ProcessMetricsConfig::default(),
            retention_period: Duration::from_secs(30 * 24 * 3600), // 30 days
            aggregation_window: Duration::from_secs(3600), // 1 hour
        }
    }
}

impl Default for CodeMetricsConfig {
    fn default() -> Self {
        Self {
            complexity_analysis: true,
            duplication_detection: true,
            maintainability_index: true,
            technical_debt: false,
        }
    }
}

impl Default for PerformanceMetricsConfig {
    fn default() -> Self {
        Self {
            execution_time: true,
            throughput: true,
            latency_percentiles: vec![50, 95, 99],
            resource_utilization: true,
        }
    }
}

impl Default for QualityMetricsConfig {
    fn default() -> Self {
        Self {
            test_metrics: true,
            code_coverage: true,
            static_analysis: true,
            security_scan: false,
        }
    }
}

impl Default for ProcessMetricsConfig {
    fn default() -> Self {
        Self {
            compilation_metrics: true,
            build_metrics: true,
            deployment_metrics: false,
        }
    }
}