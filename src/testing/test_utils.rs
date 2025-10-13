//! Comprehensive testing utilities and framework
//!
//! This module provides utilities for different types of testing:
//! - Unit testing helpers
//! - Integration testing support  
//! - Performance benchmarking
//! - Property-based testing
//! - Mock data generation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

/// Test result capture for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_name: String,
    pub success: bool,
    pub duration: Duration,
    pub error_message: Option<String>,
    pub metadata: HashMap<String, String>,
    pub timestamp: SystemTime,
}

/// Test suite configuration
#[derive(Debug, Clone)]
pub struct TestSuiteConfig {
    pub parallel_execution: bool,
    pub max_parallel_tests: usize,
    pub timeout: Duration,
    pub retry_count: usize,
    pub capture_output: bool,
    pub performance_benchmarks: bool,
}

impl Default for TestSuiteConfig {
    fn default() -> Self {
        Self {
            parallel_execution: true,
            max_parallel_tests: num_cpus::get(),
            timeout: Duration::from_secs(30),
            retry_count: 0,
            capture_output: true,
            performance_benchmarks: false,
        }
    }
}

/// Test execution context
#[derive(Debug)]
pub struct TestContext {
    pub test_name: String,
    pub config: TestSuiteConfig,
    pub start_time: Instant,
    pub temp_dir: Option<PathBuf>,
    pub metadata: HashMap<String, String>,
}

impl TestContext {
    pub fn new(test_name: &str, config: TestSuiteConfig) -> Self {
        Self {
            test_name: test_name.to_string(),
            config,
            start_time: Instant::now(),
            temp_dir: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a temporary directory for the test
    pub fn create_temp_dir(&mut self) -> Result<&Path, std::io::Error> {
        if self.temp_dir.is_none() {
            let temp_dir = std::env::temp_dir()
                .join("devkit_tests")
                .join(&self.test_name);

            std::fs::create_dir_all(&temp_dir)?;
            self.temp_dir = Some(temp_dir);
        }

        Ok(self.temp_dir.as_ref().unwrap())
    }

    /// Clean up test resources
    pub fn cleanup(&self) -> Result<(), std::io::Error> {
        if let Some(temp_dir) = &self.temp_dir {
            if temp_dir.exists() {
                std::fs::remove_dir_all(temp_dir)?;
            }
        }
        Ok(())
    }

    /// Add metadata to the test context
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    /// Get elapsed time since test start
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

/// Performance benchmark collector
#[derive(Debug)]
pub struct BenchmarkCollector {
    measurements: Arc<Mutex<Vec<BenchmarkMeasurement>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkMeasurement {
    pub name: String,
    pub duration: Duration,
    pub memory_used: Option<usize>,
    pub iterations: usize,
    pub timestamp: SystemTime,
    pub metadata: HashMap<String, String>,
}

impl BenchmarkCollector {
    pub fn new() -> Self {
        Self {
            measurements: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Record a benchmark measurement
    pub fn record(&self, name: &str, duration: Duration, iterations: usize) {
        let measurement = BenchmarkMeasurement {
            name: name.to_string(),
            duration,
            memory_used: None, // TODO: Implement memory measurement
            iterations,
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        };

        self.measurements.lock().unwrap().push(measurement);
    }

    /// Get all measurements
    pub fn get_measurements(&self) -> Vec<BenchmarkMeasurement> {
        self.measurements.lock().unwrap().clone()
    }

    /// Export measurements to JSON
    pub fn export_to_json(&self) -> Result<String, serde_json::Error> {
        let measurements = self.get_measurements();
        serde_json::to_string_pretty(&measurements)
    }
}

/// Test assertion utilities
pub struct TestAssertions;

impl TestAssertions {
    /// Assert that execution time is within expected bounds
    pub fn assert_execution_time_within(
        duration: Duration,
        min: Duration,
        max: Duration,
    ) -> Result<(), String> {
        if duration < min {
            return Err(format!("Execution too fast: {:?} < {:?}", duration, min));
        }
        if duration > max {
            return Err(format!("Execution too slow: {:?} > {:?}", duration, max));
        }
        Ok(())
    }

    /// Assert that memory usage is within acceptable bounds
    pub fn assert_memory_usage_within(used: usize, max: usize) -> Result<(), String> {
        if used > max {
            return Err(format!("Memory usage too high: {} > {}", used, max));
        }
        Ok(())
    }

    /// Assert that a collection contains expected elements
    pub fn assert_contains_all<T: PartialEq + std::fmt::Debug>(
        collection: &[T],
        expected: &[T],
    ) -> Result<(), String> {
        for item in expected {
            if !collection.contains(item) {
                return Err(format!("Collection missing expected item: {:?}", item));
            }
        }
        Ok(())
    }

    /// Assert that a result eventually succeeds within timeout
    pub async fn assert_eventually<F, Fut, T>(
        mut condition: F,
        timeout: Duration,
        check_interval: Duration,
    ) -> Result<T, String>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, String>>,
    {
        let start = Instant::now();

        while start.elapsed() < timeout {
            match condition().await {
                Ok(result) => return Ok(result),
                Err(_) => tokio::time::sleep(check_interval).await,
            }
        }

        condition().await
    }
}

/// Property-based test generator
pub struct PropertyGenerator {
    seed: u64,
}

impl PropertyGenerator {
    pub fn new() -> Self {
        Self {
            seed: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn with_seed(seed: u64) -> Self {
        Self { seed }
    }

    /// Generate random strings for testing
    pub fn generate_strings(&self, count: usize, min_len: usize, max_len: usize) -> Vec<String> {
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};

        let mut rng = StdRng::seed_from_u64(self.seed);
        let mut strings = Vec::new();

        for _ in 0..count {
            let len = rng.gen_range(min_len..=max_len);
            let s: String = (0..len)
                .map(|_| rng.gen_range(b'a'..=b'z') as char)
                .collect();
            strings.push(s);
        }

        strings
    }

    /// Generate random file paths for testing
    pub fn generate_file_paths(&self, count: usize) -> Vec<PathBuf> {
        let strings = self.generate_strings(count, 5, 20);
        strings
            .into_iter()
            .map(|s| PathBuf::from(format!("src/{}.rs", s)))
            .collect()
    }

    /// Generate random numbers within range
    pub fn generate_numbers(&self, count: usize, min: i32, max: i32) -> Vec<i32> {
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};

        let mut rng = StdRng::seed_from_u64(self.seed);
        (0..count).map(|_| rng.gen_range(min..=max)).collect()
    }
}

/// Mock data factory for tests
pub struct MockDataFactory;

impl MockDataFactory {
    /// Create mock configuration data
    pub fn create_mock_config() -> crate::config::Config {
        crate::config::Config {
            general: crate::config::GeneralConfig {
                workspace_path: Some(PathBuf::from("/mock/workspace")),
                log_level: "debug".to_string(),
                auto_save: true,
                backup_enabled: true,
                telemetry_enabled: false,
            },
            agents: crate::config::AgentConfig {
                max_concurrent_agents: 5,
                agent_timeout_seconds: 300,
                default_agent_priority: "normal".to_string(),
                notification_settings: crate::config::NotificationConfig {
                    enabled: true,
                    sound_enabled: false,
                    desktop_notifications: true,
                    auto_dismiss_timeout: 5000,
                },
                custom_agents: Vec::new(),
            },
            codegen: crate::config::CodegenConfig {
                default_style: crate::config::StyleConfig {
                    indentation: "spaces".to_string(),
                    indent_size: 4,
                    line_length: 100,
                    naming_convention: "snake_case".to_string(),
                    include_comments: true,
                    include_type_hints: true,
                },
                language_preferences: HashMap::new(),
                template_directories: Vec::new(),
                ai_model_settings: crate::config::AIModelConfig {
                    default_provider: "ollama".to_string(),
                    default_model: "llama3.2:latest".to_string(),
                    ollama: crate::config::OllamaConfig {
                        endpoint: "http://localhost:11434".to_string(),
                        timeout_seconds: 300,
                        max_retries: 3,
                        default_model: Some("llama3.2:latest".to_string()),
                    },
                    openai: None,
                    anthropic: None,
                    context_window_size: 8192,
                    temperature: 0.7,
                    max_tokens: 1000,
                },
            },
            shell: crate::config::ShellConfig {
                preferred_shell: Some("bash".to_string()),
                environment_variables: HashMap::new(),
                command_timeout: 30,
                history_enabled: true,
                custom_commands: HashMap::new(),
            },
            ui: crate::config::UIConfig {
                theme: "dark".to_string(),
                color_scheme: "dark".to_string(),
                font_size: 14,
                show_line_numbers: true,
                show_timestamps: true,
                auto_scroll: true,
                panel_layout: crate::config::PanelLayoutConfig {
                    output_panel_percentage: 70,
                    agent_panel_percentage: 30,
                    notification_panel_height: 5,
                    input_panel_height: 3,
                },
            },
            web: crate::config::WebConfig {
                enabled: false,
                host: "localhost".to_string(),
                port: 8080,
                cors_enabled: true,
                static_files_path: None,
                auth_enabled: false,
                auth_token: None,
                session_timeout_minutes: 30,
            },
            logging: crate::logging::LogConfig::default(),
            keybindings: HashMap::new(),
        }
    }

    /// Create mock agent tasks
    pub fn create_mock_agent_tasks(count: usize) -> Vec<crate::agents::AgentTask> {
        (0..count)
            .map(|i| crate::agents::AgentTask {
                id: format!("mock_task_{}", i),
                task_type: "mock".to_string(),
                description: format!("Mock task number {}", i),
                context: serde_json::json!({
                    "index": i,
                    "mock": true
                }),
                priority: match i % 4 {
                    0 => crate::agents::TaskPriority::Low,
                    1 => crate::agents::TaskPriority::Normal,
                    2 => crate::agents::TaskPriority::High,
                    3 => crate::agents::TaskPriority::Critical,
                    _ => crate::agents::TaskPriority::Normal,
                },
                deadline: None,
                metadata: HashMap::new(),
            })
            .collect()
    }

    /// Create mock file contexts
    pub fn create_mock_file_contexts(count: usize) -> Vec<crate::context::FileContext> {
        (0..count)
            .map(|i| crate::context::FileContext {
                path: PathBuf::from(format!("src/mock_{}.rs", i)),
                relative_path: PathBuf::from(format!("mock_{}.rs", i)),
                language: "rust".to_string(),
                size_bytes: (i * 1024) as u64,
                line_count: i * 10,
                last_modified: SystemTime::now(),
                content_hash: format!("mock_hash_{}", i),
                symbols: vec![crate::context::symbols::Symbol {
                    name: format!("mock_function_{}", i),
                    qualified_name: Some(format!("src/test_symbol.rs:mock_function_{}", i)),
                    symbol_type: crate::context::symbols::SymbolType::Function,
                    file_path: PathBuf::from("src/test_symbol.rs"),
                    line: 1,
                    line_number: 1,
                    column: 1,
                    signature: Some("fn test_symbol()".to_string()),
                    documentation: Some("Test symbol".to_string()),
                    visibility: crate::context::symbols::Visibility::Public,
                    references: Vec::new(),
                }],
                imports: vec![
                    "std::collections::HashMap".to_string(),
                    "serde::{Serialize, Deserialize}".to_string(),
                ],
                exports: vec![format!("mock_function_{}", i)],
                relationships: Vec::new(),
            })
            .collect()
    }
}

/// Test runner for executing test suites
pub struct TestRunner {
    config: TestSuiteConfig,
    results: Arc<Mutex<Vec<TestResult>>>,
    benchmark_collector: BenchmarkCollector,
}

impl TestRunner {
    pub fn new(config: TestSuiteConfig) -> Self {
        Self {
            config,
            results: Arc::new(Mutex::new(Vec::new())),
            benchmark_collector: BenchmarkCollector::new(),
        }
    }

    /// Run a single test with full context
    pub async fn run_test<F, Fut>(&self, test_name: &str, test_fn: F) -> TestResult
    where
        F: FnOnce(TestContext) -> Fut,
        Fut: std::future::Future<Output = Result<(), String>>,
    {
        let mut context = TestContext::new(test_name, self.config.clone());
        let start_time = Instant::now();

        let result = if self.config.timeout > Duration::ZERO {
            match tokio::time::timeout(self.config.timeout, test_fn(context)).await {
                Ok(result) => result,
                Err(_) => Err(format!(
                    "Test '{}' timed out after {:?}",
                    test_name, self.config.timeout
                )),
            }
        } else {
            test_fn(context).await
        };

        let duration = start_time.elapsed();

        let test_result = TestResult {
            test_name: test_name.to_string(),
            success: result.is_ok(),
            duration,
            error_message: result.err(),
            metadata: HashMap::new(), // TODO: Extract from context
            timestamp: SystemTime::now(),
        };

        // Record benchmark if enabled
        if self.config.performance_benchmarks {
            self.benchmark_collector.record(test_name, duration, 1);
        }

        self.results.lock().unwrap().push(test_result.clone());
        test_result
    }

    /// Run multiple tests with optional parallelization
    pub async fn run_test_suite<F, Fut>(&self, tests: Vec<(&str, F)>) -> TestSuiteReport
    where
        F: FnOnce(TestContext) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<(), String>> + Send,
    {
        let suite_start = Instant::now();

        if self.config.parallel_execution {
            self.run_tests_parallel(tests).await
        } else {
            self.run_tests_sequential(tests).await
        };

        let suite_duration = suite_start.elapsed();
        let results = self.results.lock().unwrap().clone();

        TestSuiteReport {
            total_tests: results.len(),
            passed: results.iter().filter(|r| r.success).count(),
            failed: results.iter().filter(|r| !r.success).count(),
            duration: suite_duration,
            results,
            benchmarks: if self.config.performance_benchmarks {
                Some(self.benchmark_collector.get_measurements())
            } else {
                None
            },
        }
    }

    async fn run_tests_sequential<F, Fut>(&self, tests: Vec<(&str, F)>)
    where
        F: FnOnce(TestContext) -> Fut,
        Fut: std::future::Future<Output = Result<(), String>>,
    {
        for (name, test_fn) in tests {
            self.run_test(name, test_fn).await;
        }
    }

    async fn run_tests_parallel<F, Fut>(&self, tests: Vec<(&str, F)>)
    where
        F: FnOnce(TestContext) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<(), String>> + Send,
    {
        use tokio::sync::Semaphore;

        let semaphore = Arc::new(Semaphore::new(self.config.max_parallel_tests));
        let mut handles = Vec::new();

        for (name, test_fn) in tests {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let runner = self.clone();
            let test_name = name.to_string();

            let handle = tokio::spawn(async move {
                let _permit = permit; // Keep permit alive
                runner.run_test(&test_name, test_fn).await
            });

            handles.push(handle);
        }

        // Wait for all tests to complete
        for handle in handles {
            let _ = handle.await;
        }
    }
}

// Implement Clone for TestRunner (needed for parallel execution)
impl Clone for TestRunner {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            results: Arc::clone(&self.results),
            benchmark_collector: BenchmarkCollector::new(), // New collector for clone
        }
    }
}

/// Test suite execution report
#[derive(Debug, Serialize)]
pub struct TestSuiteReport {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub duration: Duration,
    pub results: Vec<TestResult>,
    pub benchmarks: Option<Vec<BenchmarkMeasurement>>,
}

impl TestSuiteReport {
    /// Print a summary of the test results
    pub fn print_summary(&self) {
        println!("\n🧪 Test Suite Report");
        println!("==================");
        println!("Total tests: {}", self.total_tests);
        println!("✅ Passed: {}", self.passed);
        println!("❌ Failed: {}", self.failed);
        println!("⏱️  Duration: {:?}", self.duration);

        if self.failed > 0 {
            println!("\n❌ Failed Tests:");
            for result in &self.results {
                if !result.success {
                    println!(
                        "  • {} - {}",
                        result.test_name,
                        result.error_message.as_deref().unwrap_or("Unknown error")
                    );
                }
            }
        }

        if let Some(benchmarks) = &self.benchmarks {
            println!("\n📊 Performance Benchmarks:");
            for benchmark in benchmarks {
                println!(
                    "  • {} - {:?} ({} iterations)",
                    benchmark.name, benchmark.duration, benchmark.iterations
                );
            }
        }
    }

    /// Export report to JSON
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }
}

/// Macro for creating simple test cases
#[macro_export]
macro_rules! test_case {
    ($name:expr, $test_fn:expr) => {
        (
            $name,
            |_ctx: $crate::testing::test_utils::TestContext| async move { $test_fn().await },
        )
    };
}

/// Macro for creating benchmark test cases
#[macro_export]
macro_rules! benchmark_test {
    ($name:expr, $iterations:expr, $test_fn:expr) => {
        (
            $name,
            |_ctx: $crate::testing::test_utils::TestContext| async move {
                let start = std::time::Instant::now();
                for _ in 0..$iterations {
                    $test_fn().await?;
                }
                let duration = start.elapsed();

                // Record benchmark information in context metadata
                println!(
                    "Benchmark '{}': {:?} for {} iterations ({:?} per iteration)",
                    $name,
                    duration,
                    $iterations,
                    duration / $iterations
                );

                Ok(())
            },
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_test_runner_basic() {
        let config = TestSuiteConfig {
            parallel_execution: false,
            performance_benchmarks: true,
            ..Default::default()
        };

        let runner = TestRunner::new(config);

        let test_result = runner
            .run_test("basic_test", |_ctx| async {
                tokio::time::sleep(Duration::from_millis(10)).await;
                Ok(())
            })
            .await;

        assert!(test_result.success);
        assert!(test_result.duration >= Duration::from_millis(10));
        assert_eq!(test_result.test_name, "basic_test");
    }

    #[test]
    fn test_property_generator() {
        let generator = PropertyGenerator::with_seed(12345);

        let strings = generator.generate_strings(5, 3, 10);
        assert_eq!(strings.len(), 5);
        for s in &strings {
            assert!(s.len() >= 3 && s.len() <= 10);
        }

        let numbers = generator.generate_numbers(10, 1, 100);
        assert_eq!(numbers.len(), 10);
        for n in &numbers {
            assert!(*n >= 1 && *n <= 100);
        }
    }

    #[test]
    fn test_mock_data_factory() {
        let config = MockDataFactory::create_mock_config();
        assert_eq!(config.general.log_level, "debug");
        assert_eq!(config.agents.max_concurrent_agents, 5);

        let tasks = MockDataFactory::create_mock_agent_tasks(3);
        assert_eq!(tasks.len(), 3);
        for (i, task) in tasks.iter().enumerate() {
            assert_eq!(task.id, format!("mock_task_{}", i));
        }

        let contexts = MockDataFactory::create_mock_file_contexts(2);
        assert_eq!(contexts.len(), 2);
        assert_eq!(contexts[0].language, "rust");
    }
}
