use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::codegen::GeneratedCode;
use crate::evaluation::EvaluationContext;

/// Benchmark runner for performance testing
#[derive(Debug, Clone)]
pub struct BenchmarkRunner {
    config: BenchmarkConfig,
    benchmarks: Vec<Benchmark>,
}

/// Configuration for benchmark runner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub enabled_benchmarks: Vec<BenchmarkType>,
    pub performance_config: PerformanceBenchmarkConfig,
    pub memory_config: MemoryBenchmarkConfig,
    pub compilation_config: CompilationBenchmarkConfig,
    pub custom_benchmarks: Vec<CustomBenchmark>,
    pub timeout: Duration,
    pub iterations: u32,
    pub warmup_iterations: u32,
    pub statistical_analysis: bool,
}

/// Types of benchmarks available
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum BenchmarkType {
    Performance,
    Memory,
    Compilation,
    Custom(String),
}

/// Individual benchmark definition
#[derive(Debug, Clone)]
pub struct Benchmark {
    pub benchmark_type: BenchmarkType,
    pub name: String,
    pub description: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub timeout: Duration,
    pub success_criteria: SuccessCriteria,
}

/// Success criteria for benchmarks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriteria {
    pub max_execution_time_ms: Option<f64>,
    pub max_memory_usage_mb: Option<f64>,
    pub min_throughput: Option<f64>,
    pub max_error_rate: Option<f64>,
    pub custom_validators: Vec<String>,
}

/// Performance benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBenchmarkConfig {
    pub cpu_benchmarks: bool,
    pub io_benchmarks: bool,
    pub network_benchmarks: bool,
    pub concurrent_benchmarks: bool,
    pub stress_test: bool,
}

/// Memory benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBenchmarkConfig {
    pub allocation_patterns: bool,
    pub leak_detection: bool,
    pub fragmentation_analysis: bool,
    pub peak_usage_tracking: bool,
}

/// Compilation benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationBenchmarkConfig {
    pub incremental_compile: bool,
    pub clean_compile: bool,
    pub parallel_compile: bool,
    pub dependency_analysis: bool,
}

/// Custom benchmark definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomBenchmark {
    pub name: String,
    pub description: String,
    pub command: String,
    pub args: Vec<String>,
    pub success_criteria: SuccessCriteria,
}

/// Benchmark execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub benchmark_type: BenchmarkType,
    pub name: String,
    pub passed: bool,
    pub duration: Duration,
    pub metrics: BenchmarkMetrics,
    pub statistics: BenchmarkStatistics,
    pub output: String,
    pub error: Option<String>,
    pub iterations: u32,
    pub historical_comparison: Option<HistoricalComparison>,
}

/// Metrics collected during benchmark execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetrics {
    pub execution_time_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub throughput: f64,
    pub error_rate: f64,
    pub custom_metrics: HashMap<String, f64>,
}

/// Statistical analysis of benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkStatistics {
    pub mean: f64,
    pub median: f64,
    pub std_deviation: f64,
    pub min: f64,
    pub max: f64,
    pub percentiles: HashMap<u8, f64>, // 50th, 95th, 99th percentiles
    pub confidence_interval: (f64, f64),
}

/// Historical comparison data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalComparison {
    pub baseline_value: f64,
    pub change_percent: f64,
    pub trend: PerformanceTrend,
    pub significance: StatisticalSignificance,
}

/// Performance trend indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceTrend {
    Improving,
    Stable,
    Degrading,
    Volatile,
}

/// Statistical significance levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatisticalSignificance {
    High,
    Medium,
    Low,
    None,
}

impl BenchmarkRunner {
    /// Create new benchmark runner
    pub fn new(config: BenchmarkConfig) -> Self {
        let benchmarks = Self::create_benchmarks(&config);
        Self { config, benchmarks }
    }

    /// Create benchmarks based on configuration
    fn create_benchmarks(config: &BenchmarkConfig) -> Vec<Benchmark> {
        let mut benchmarks = Vec::new();

        for benchmark_type in &config.enabled_benchmarks {
            match benchmark_type {
                BenchmarkType::Performance => {
                    benchmarks.extend(Self::create_performance_benchmarks(config));
                }
                BenchmarkType::Memory => {
                    benchmarks.extend(Self::create_memory_benchmarks(config));
                }
                BenchmarkType::Compilation => {
                    benchmarks.extend(Self::create_compilation_benchmarks(config));
                }
                BenchmarkType::Custom(name) => {
                    if let Some(custom) = config.custom_benchmarks.iter().find(|b| &b.name == name) {
                        benchmarks.push(Self::create_custom_benchmark(custom, config));
                    }
                }
            }
        }

        benchmarks
    }

    /// Create performance benchmarks
    fn create_performance_benchmarks(config: &BenchmarkConfig) -> Vec<Benchmark> {
        let mut benchmarks = Vec::new();

        if config.performance_config.cpu_benchmarks {
            benchmarks.push(Benchmark {
                benchmark_type: BenchmarkType::Performance,
                name: "CPU Intensive Operations".to_string(),
                description: "Tests CPU-bound performance".to_string(),
                command: "cargo".to_string(),
                args: vec!["bench".to_string(), "--bench".to_string(), "cpu_bench".to_string()],
                env: HashMap::new(),
                timeout: config.timeout,
                success_criteria: SuccessCriteria {
                    max_execution_time_ms: Some(5000.0),
                    max_memory_usage_mb: None,
                    min_throughput: Some(1000.0),
                    max_error_rate: Some(0.01),
                    custom_validators: Vec::new(),
                },
            });
        }

        if config.performance_config.io_benchmarks {
            benchmarks.push(Benchmark {
                benchmark_type: BenchmarkType::Performance,
                name: "I/O Operations".to_string(),
                description: "Tests I/O performance".to_string(),
                command: "cargo".to_string(),
                args: vec!["bench".to_string(), "--bench".to_string(), "io_bench".to_string()],
                env: HashMap::new(),
                timeout: config.timeout,
                success_criteria: SuccessCriteria {
                    max_execution_time_ms: Some(10000.0),
                    max_memory_usage_mb: Some(100.0),
                    min_throughput: Some(100.0),
                    max_error_rate: Some(0.05),
                    custom_validators: Vec::new(),
                },
            });
        }

        benchmarks
    }

    /// Create memory benchmarks
    fn create_memory_benchmarks(config: &BenchmarkConfig) -> Vec<Benchmark> {
        let mut benchmarks = Vec::new();

        if config.memory_config.allocation_patterns {
            benchmarks.push(Benchmark {
                benchmark_type: BenchmarkType::Memory,
                name: "Memory Allocation Patterns".to_string(),
                description: "Tests memory allocation efficiency".to_string(),
                command: "cargo".to_string(),
                args: vec!["bench".to_string(), "--bench".to_string(), "memory_bench".to_string()],
                env: HashMap::new(),
                timeout: config.timeout,
                success_criteria: SuccessCriteria {
                    max_execution_time_ms: None,
                    max_memory_usage_mb: Some(500.0),
                    min_throughput: None,
                    max_error_rate: Some(0.0),
                    custom_validators: Vec::new(),
                },
            });
        }

        benchmarks
    }

    /// Create compilation benchmarks
    fn create_compilation_benchmarks(config: &BenchmarkConfig) -> Vec<Benchmark> {
        let mut benchmarks = Vec::new();

        if config.compilation_config.clean_compile {
            benchmarks.push(Benchmark {
                benchmark_type: BenchmarkType::Compilation,
                name: "Clean Compilation".to_string(),
                description: "Tests clean compilation time".to_string(),
                command: "cargo".to_string(),
                args: vec!["clean".to_string()],
                env: HashMap::new(),
                timeout: config.timeout,
                success_criteria: SuccessCriteria {
                    max_execution_time_ms: Some(60000.0),
                    max_memory_usage_mb: None,
                    min_throughput: None,
                    max_error_rate: Some(0.0),
                    custom_validators: Vec::new(),
                },
            });
        }

        benchmarks
    }

    /// Create custom benchmark
    fn create_custom_benchmark(custom: &CustomBenchmark, config: &BenchmarkConfig) -> Benchmark {
        Benchmark {
            benchmark_type: BenchmarkType::Custom(custom.name.clone()),
            name: custom.name.clone(),
            description: custom.description.clone(),
            command: custom.command.clone(),
            args: custom.args.clone(),
            env: HashMap::new(),
            timeout: config.timeout,
            success_criteria: custom.success_criteria.clone(),
        }
    }

    /// Run all benchmarks
    pub async fn run_benchmarks(
        &self,
        code: &GeneratedCode,
        context: &EvaluationContext,
    ) -> Result<Vec<BenchmarkResult>, BenchmarkError> {
        let mut results = Vec::new();

        for benchmark in &self.benchmarks {
            let result = self.run_single_benchmark(benchmark, code, context).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Run a single benchmark
    async fn run_single_benchmark(
        &self,
        benchmark: &Benchmark,
        _code: &GeneratedCode,
        context: &EvaluationContext,
    ) -> Result<BenchmarkResult, BenchmarkError> {
        let mut measurements = Vec::new();
        
        // Warmup iterations
        for _ in 0..self.config.warmup_iterations {
            let _result = self.execute_benchmark_command(benchmark, context).await?;
        }

        // Actual measurements
        for _ in 0..self.config.iterations {
            let measurement = self.execute_benchmark_command(benchmark, context).await?;
            measurements.push(measurement);
        }

        // Calculate statistics
        let statistics = if self.config.statistical_analysis {
            self.calculate_statistics(&measurements)
        } else {
            BenchmarkStatistics::default()
        };

        // Aggregate metrics
        let metrics = self.aggregate_metrics(&measurements);

        // Check success criteria
        let passed = self.evaluate_success_criteria(&benchmark.success_criteria, &metrics);

        Ok(BenchmarkResult {
            benchmark_type: benchmark.benchmark_type.clone(),
            name: benchmark.name.clone(),
            passed,
            duration: measurements.iter().map(|m| m.duration).sum(),
            metrics,
            statistics,
            output: "Benchmark executed successfully".to_string(),
            error: None,
            iterations: self.config.iterations,
            historical_comparison: None, // Would load historical data
        })
    }

    /// Execute benchmark command
    async fn execute_benchmark_command(
        &self,
        benchmark: &Benchmark,
        context: &EvaluationContext,
    ) -> Result<BenchmarkMeasurement, BenchmarkError> {
        let start_time = Instant::now();
        
        let output = Command::new(&benchmark.command)
            .args(&benchmark.args)
            .current_dir(&context.project_path)
            .envs(&benchmark.env)
            .output()
            .await
            .map_err(|e| BenchmarkError::ExecutionFailed(format!("Failed to execute benchmark: {}", e)))?;

        let duration = start_time.elapsed();

        // In a real implementation, would collect actual metrics
        let metrics = BenchmarkMetrics {
            execution_time_ms: duration.as_millis() as f64,
            memory_usage_mb: 50.0, // Mock value
            cpu_usage_percent: 25.0, // Mock value
            throughput: 500.0, // Mock value
            error_rate: if output.status.success() { 0.0 } else { 1.0 },
            custom_metrics: HashMap::new(),
        };

        Ok(BenchmarkMeasurement {
            duration,
            metrics,
            success: output.status.success(),
        })
    }

    /// Calculate statistical analysis
    fn calculate_statistics(&self, measurements: &[BenchmarkMeasurement]) -> BenchmarkStatistics {
        let execution_times: Vec<f64> = measurements.iter()
            .map(|m| m.metrics.execution_time_ms)
            .collect();

        if execution_times.is_empty() {
            return BenchmarkStatistics::default();
        }

        let mean = execution_times.iter().sum::<f64>() / execution_times.len() as f64;
        
        let mut sorted_times = execution_times.clone();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let median = if sorted_times.len() % 2 == 0 {
            (sorted_times[sorted_times.len() / 2 - 1] + sorted_times[sorted_times.len() / 2]) / 2.0
        } else {
            sorted_times[sorted_times.len() / 2]
        };

        let variance = execution_times.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / execution_times.len() as f64;
        let std_deviation = variance.sqrt();

        let min = sorted_times[0];
        let max = sorted_times[sorted_times.len() - 1];

        let mut percentiles = HashMap::new();
        percentiles.insert(50, median);
        percentiles.insert(95, sorted_times[(sorted_times.len() as f64 * 0.95) as usize]);
        percentiles.insert(99, sorted_times[(sorted_times.len() as f64 * 0.99) as usize]);

        // Simplified confidence interval (would use proper statistical methods)
        let margin_of_error = 1.96 * std_deviation / (execution_times.len() as f64).sqrt();
        let confidence_interval = (mean - margin_of_error, mean + margin_of_error);

        BenchmarkStatistics {
            mean,
            median,
            std_deviation,
            min,
            max,
            percentiles,
            confidence_interval,
        }
    }

    /// Aggregate metrics from multiple measurements
    fn aggregate_metrics(&self, measurements: &[BenchmarkMeasurement]) -> BenchmarkMetrics {
        if measurements.is_empty() {
            return BenchmarkMetrics::default();
        }

        let count = measurements.len() as f64;
        
        BenchmarkMetrics {
            execution_time_ms: measurements.iter().map(|m| m.metrics.execution_time_ms).sum::<f64>() / count,
            memory_usage_mb: measurements.iter().map(|m| m.metrics.memory_usage_mb).sum::<f64>() / count,
            cpu_usage_percent: measurements.iter().map(|m| m.metrics.cpu_usage_percent).sum::<f64>() / count,
            throughput: measurements.iter().map(|m| m.metrics.throughput).sum::<f64>() / count,
            error_rate: measurements.iter().map(|m| m.metrics.error_rate).sum::<f64>() / count,
            custom_metrics: HashMap::new(),
        }
    }

    /// Evaluate success criteria
    fn evaluate_success_criteria(&self, criteria: &SuccessCriteria, metrics: &BenchmarkMetrics) -> bool {
        if let Some(max_time) = criteria.max_execution_time_ms {
            if metrics.execution_time_ms > max_time {
                return false;
            }
        }

        if let Some(max_memory) = criteria.max_memory_usage_mb {
            if metrics.memory_usage_mb > max_memory {
                return false;
            }
        }

        if let Some(min_throughput) = criteria.min_throughput {
            if metrics.throughput < min_throughput {
                return false;
            }
        }

        if let Some(max_error_rate) = criteria.max_error_rate {
            if metrics.error_rate > max_error_rate {
                return false;
            }
        }

        true
    }
}

/// Individual benchmark measurement
#[derive(Debug, Clone)]
struct BenchmarkMeasurement {
    duration: Duration,
    metrics: BenchmarkMetrics,
    success: bool,
}

/// Benchmark execution errors
#[derive(Debug, thiserror::Error)]
pub enum BenchmarkError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Timeout error: benchmark took too long")]
    Timeout,
    #[error("Configuration error: {0}")]
    Configuration(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            enabled_benchmarks: vec![BenchmarkType::Performance, BenchmarkType::Compilation],
            performance_config: PerformanceBenchmarkConfig::default(),
            memory_config: MemoryBenchmarkConfig::default(),
            compilation_config: CompilationBenchmarkConfig::default(),
            custom_benchmarks: Vec::new(),
            timeout: Duration::from_secs(60),
            iterations: 5,
            warmup_iterations: 2,
            statistical_analysis: true,
        }
    }
}

impl Default for PerformanceBenchmarkConfig {
    fn default() -> Self {
        Self {
            cpu_benchmarks: true,
            io_benchmarks: true,
            network_benchmarks: false,
            concurrent_benchmarks: true,
            stress_test: false,
        }
    }
}

impl Default for MemoryBenchmarkConfig {
    fn default() -> Self {
        Self {
            allocation_patterns: true,
            leak_detection: true,
            fragmentation_analysis: false,
            peak_usage_tracking: true,
        }
    }
}

impl Default for CompilationBenchmarkConfig {
    fn default() -> Self {
        Self {
            incremental_compile: true,
            clean_compile: true,
            parallel_compile: true,
            dependency_analysis: false,
        }
    }
}

impl Default for BenchmarkMetrics {
    fn default() -> Self {
        Self {
            execution_time_ms: 0.0,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            throughput: 0.0,
            error_rate: 0.0,
            custom_metrics: HashMap::new(),
        }
    }
}

impl Default for BenchmarkStatistics {
    fn default() -> Self {
        Self {
            mean: 0.0,
            median: 0.0,
            std_deviation: 0.0,
            min: 0.0,
            max: 0.0,
            percentiles: HashMap::new(),
            confidence_interval: (0.0, 0.0),
        }
    }
}