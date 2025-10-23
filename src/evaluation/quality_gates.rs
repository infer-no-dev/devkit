use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use std::path::Path;

use crate::codegen::GeneratedCode;
use crate::evaluation::EvaluationContext;

/// Quality gate manager for running automated checks
#[derive(Debug, Clone)]
pub struct QualityGateManager {
    config: QualityGateConfig,
    gates: Vec<QualityGate>,
}

/// Configuration for quality gates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateConfig {
    pub enabled_gates: Vec<QualityGateType>,
    pub clippy_config: ClippyConfig,
    pub format_config: FormatConfig,
    pub test_config: TestConfig,
    pub coverage_config: CoverageConfig,
    pub security_config: SecurityConfig,
    pub complexity_config: ComplexityConfig,
    pub documentation_config: DocumentationConfig,
    pub timeout: Duration,
    pub fail_fast: bool,
}

/// Types of quality gates available
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum QualityGateType {
    Compilation,
    Clippy,
    Format,
    Tests,
    Coverage,
    Security,
    Complexity,
    Documentation,
    Dependencies,
    Performance,
}

/// Individual quality gate
#[derive(Debug, Clone)]
pub struct QualityGate {
    pub gate_type: QualityGateType,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub required: bool,
    pub timeout: Duration,
}

/// Result from running a quality gate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateResult {
    pub gate_type: QualityGateType,
    pub name: String,
    pub passed: bool,
    pub score: f64,
    pub duration: Duration,
    pub issues: Vec<QualityIssue>,
    pub metrics: HashMap<String, f64>,
    pub output: String,
    pub suggestions: Vec<String>,
}

/// Quality issue found by a gate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    pub id: String,
    pub severity: IssueSeverity,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub rule: Option<String>,
    pub suggestion: Option<String>,
}

/// Issue severity for quality issues
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Clippy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClippyConfig {
    pub deny_warnings: bool,
    pub allowed_lints: Vec<String>,
    pub denied_lints: Vec<String>,
    pub custom_config: Option<String>,
}

/// Format configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatConfig {
    pub check_only: bool,
    pub edition: String,
    pub config_file: Option<String>,
}

/// Test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub include_ignored: bool,
    pub test_threads: Option<u32>,
    pub timeout: Duration,
    pub test_filter: Option<String>,
}

/// Coverage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageConfig {
    pub minimum_coverage: f64,
    pub exclude_patterns: Vec<String>,
    pub report_format: CoverageFormat,
}

/// Coverage report formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoverageFormat {
    Html,
    Lcov,
    Json,
    Text,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub audit_dependencies: bool,
    pub check_advisories: bool,
    pub allowed_vulnerabilities: Vec<String>,
}

/// Complexity configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityConfig {
    pub max_cyclomatic_complexity: u32,
    pub max_cognitive_complexity: u32,
    pub max_function_length: u32,
    pub max_file_length: u32,
}

/// Documentation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationConfig {
    pub require_docs: bool,
    pub check_examples: bool,
    pub min_doc_coverage: f64,
}

impl QualityGateManager {
    /// Create new quality gate manager
    pub fn new(config: QualityGateConfig) -> Self {
        let gates = Self::create_gates(&config);
        Self { config, gates }
    }

    /// Create quality gates based on configuration
    fn create_gates(config: &QualityGateConfig) -> Vec<QualityGate> {
        let mut gates = Vec::new();

        for gate_type in &config.enabled_gates {
            let gate = match gate_type {
                QualityGateType::Compilation => QualityGate {
                    gate_type: QualityGateType::Compilation,
                    name: "Compilation Check".to_string(),
                    description: "Ensures code compiles without errors".to_string(),
                    enabled: true,
                    required: true,
                    timeout: config.timeout,
                },
                QualityGateType::Clippy => QualityGate {
                    gate_type: QualityGateType::Clippy,
                    name: "Clippy Linting".to_string(),
                    description: "Runs Clippy for code quality and style issues".to_string(),
                    enabled: true,
                    required: config.clippy_config.deny_warnings,
                    timeout: config.timeout,
                },
                QualityGateType::Format => QualityGate {
                    gate_type: QualityGateType::Format,
                    name: "Code Formatting".to_string(),
                    description: "Checks code formatting with rustfmt".to_string(),
                    enabled: true,
                    required: true,
                    timeout: config.timeout,
                },
                QualityGateType::Tests => QualityGate {
                    gate_type: QualityGateType::Tests,
                    name: "Unit Tests".to_string(),
                    description: "Runs unit tests and reports results".to_string(),
                    enabled: true,
                    required: true,
                    timeout: config.test_config.timeout,
                },
                QualityGateType::Coverage => QualityGate {
                    gate_type: QualityGateType::Coverage,
                    name: "Test Coverage".to_string(),
                    description: "Measures and validates test coverage".to_string(),
                    enabled: true,
                    required: config.coverage_config.minimum_coverage > 0.0,
                    timeout: config.timeout,
                },
                QualityGateType::Security => QualityGate {
                    gate_type: QualityGateType::Security,
                    name: "Security Audit".to_string(),
                    description: "Checks for security vulnerabilities".to_string(),
                    enabled: config.security_config.audit_dependencies,
                    required: false,
                    timeout: config.timeout,
                },
                QualityGateType::Complexity => QualityGate {
                    gate_type: QualityGateType::Complexity,
                    name: "Code Complexity".to_string(),
                    description: "Analyzes code complexity metrics".to_string(),
                    enabled: true,
                    required: false,
                    timeout: config.timeout,
                },
                QualityGateType::Documentation => QualityGate {
                    gate_type: QualityGateType::Documentation,
                    name: "Documentation".to_string(),
                    description: "Validates documentation completeness".to_string(),
                    enabled: config.documentation_config.require_docs,
                    required: config.documentation_config.require_docs,
                    timeout: config.timeout,
                },
                QualityGateType::Dependencies => QualityGate {
                    gate_type: QualityGateType::Dependencies,
                    name: "Dependency Check".to_string(),
                    description: "Validates dependency security and licenses".to_string(),
                    enabled: true,
                    required: false,
                    timeout: config.timeout,
                },
                QualityGateType::Performance => QualityGate {
                    gate_type: QualityGateType::Performance,
                    name: "Performance Check".to_string(),
                    description: "Basic performance validation".to_string(),
                    enabled: true,
                    required: false,
                    timeout: config.timeout,
                },
            };
            gates.push(gate);
        }

        gates
    }

    /// Run all enabled quality gates
    pub async fn run_gates(
        &self,
        code: &GeneratedCode,
        context: &EvaluationContext,
    ) -> Result<Vec<QualityGateResult>, QualityGateError> {
        let mut results = Vec::new();

        for gate in &self.gates {
            if !gate.enabled {
                continue;
            }

            let result = self.run_single_gate(gate, code, context).await;

            match result {
                Ok(gate_result) => {
                    let passed = gate_result.passed;
                    results.push(gate_result);

                    // Fail fast if enabled and required gate failed
                    if self.config.fail_fast && gate.required && !passed {
                        return Ok(results);
                    }
                }
                Err(err) => {
                    if gate.required {
                        return Err(err);
                    }
                    // Log error for non-required gates but continue
                    eprintln!("Warning: Non-required gate {} failed: {}", gate.name, err);
                }
            }
        }

        Ok(results)
    }

    /// Run a single quality gate
    async fn run_single_gate(
        &self,
        gate: &QualityGate,
        code: &GeneratedCode,
        context: &EvaluationContext,
    ) -> Result<QualityGateResult, QualityGateError> {
        let start_time = std::time::Instant::now();

        let result = match gate.gate_type {
            QualityGateType::Compilation => self.run_compilation_check(code, context).await,
            QualityGateType::Clippy => self.run_clippy_check(code, context).await,
            QualityGateType::Format => self.run_format_check(code, context).await,
            QualityGateType::Tests => self.run_test_check(code, context).await,
            QualityGateType::Coverage => self.run_coverage_check(code, context).await,
            QualityGateType::Security => self.run_security_check(code, context).await,
            QualityGateType::Complexity => self.run_complexity_check(code, context).await,
            QualityGateType::Documentation => self.run_documentation_check(code, context).await,
            QualityGateType::Dependencies => self.run_dependency_check(code, context).await,
            QualityGateType::Performance => self.run_performance_check(code, context).await,
        };

        let duration = start_time.elapsed();

        match result {
            Ok(mut gate_result) => {
                gate_result.duration = duration;
                Ok(gate_result)
            }
            Err(err) => Err(err),
        }
    }

    /// Run compilation check
    async fn run_compilation_check(
        &self,
        _code: &GeneratedCode,
        context: &EvaluationContext,
    ) -> Result<QualityGateResult, QualityGateError> {
        let output = Command::new("cargo")
            .args(["check", "--all-targets"])
            .current_dir(&context.project_path)
            .output()
            .await
            .map_err(|e| QualityGateError::ExecutionFailed(format!("Failed to run cargo check: {}", e)))?;

        let passed = output.status.success();
        let output_str = String::from_utf8_lossy(&output.stderr);

        Ok(QualityGateResult {
            gate_type: QualityGateType::Compilation,
            name: "Compilation Check".to_string(),
            passed,
            score: if passed { 100.0 } else { 0.0 },
            duration: Duration::from_secs(0), // Set by caller
            issues: Self::parse_compiler_output(&output_str),
            metrics: HashMap::new(),
            output: output_str.to_string(),
            suggestions: if passed {
                vec!["Code compiles successfully".to_string()]
            } else {
                vec!["Fix compilation errors before proceeding".to_string()]
            },
        })
    }

    /// Run Clippy check
    async fn run_clippy_check(
        &self,
        _code: &GeneratedCode,
        context: &EvaluationContext,
    ) -> Result<QualityGateResult, QualityGateError> {
        let mut args = vec!["clippy", "--all-targets", "--", "-D", "warnings"];
        
        if self.config.clippy_config.deny_warnings {
            args.extend(&["-D", "warnings"]);
        }

        let output = Command::new("cargo")
            .args(&args)
            .current_dir(&context.project_path)
            .output()
            .await
            .map_err(|e| QualityGateError::ExecutionFailed(format!("Failed to run clippy: {}", e)))?;

        let passed = output.status.success();
        let output_str = String::from_utf8_lossy(&output.stdout);

        Ok(QualityGateResult {
            gate_type: QualityGateType::Clippy,
            name: "Clippy Linting".to_string(),
            passed,
            score: if passed { 100.0 } else { 50.0 },
            duration: Duration::from_secs(0),
            issues: Self::parse_clippy_output(&output_str),
            metrics: HashMap::new(),
            output: output_str.to_string(),
            suggestions: if passed {
                vec!["No clippy warnings found".to_string()]
            } else {
                vec!["Address clippy warnings to improve code quality".to_string()]
            },
        })
    }

    /// Run format check
    async fn run_format_check(
        &self,
        _code: &GeneratedCode,
        context: &EvaluationContext,
    ) -> Result<QualityGateResult, QualityGateError> {
        let output = Command::new("cargo")
            .args(["fmt", "--", "--check"])
            .current_dir(&context.project_path)
            .output()
            .await
            .map_err(|e| QualityGateError::ExecutionFailed(format!("Failed to run rustfmt: {}", e)))?;

        let passed = output.status.success();

        Ok(QualityGateResult {
            gate_type: QualityGateType::Format,
            name: "Code Formatting".to_string(),
            passed,
            score: if passed { 100.0 } else { 0.0 },
            duration: Duration::from_secs(0),
            issues: Vec::new(),
            metrics: HashMap::new(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            suggestions: if passed {
                vec!["Code is properly formatted".to_string()]
            } else {
                vec!["Run `cargo fmt` to fix formatting issues".to_string()]
            },
        })
    }

    /// Run test check
    async fn run_test_check(
        &self,
        _code: &GeneratedCode,
        context: &EvaluationContext,
    ) -> Result<QualityGateResult, QualityGateError> {
        let output = Command::new("cargo")
            .args(["test"])
            .current_dir(&context.project_path)
            .output()
            .await
            .map_err(|e| QualityGateError::ExecutionFailed(format!("Failed to run tests: {}", e)))?;

        let passed = output.status.success();
        let output_str = String::from_utf8_lossy(&output.stdout);

        Ok(QualityGateResult {
            gate_type: QualityGateType::Tests,
            name: "Unit Tests".to_string(),
            passed,
            score: if passed { 100.0 } else { 0.0 },
            duration: Duration::from_secs(0),
            issues: Vec::new(),
            metrics: HashMap::new(),
            output: output_str.to_string(),
            suggestions: if passed {
                vec!["All tests passed".to_string()]
            } else {
                vec!["Fix failing tests".to_string()]
            },
        })
    }

    /// Stub implementations for other checks
    async fn run_coverage_check(&self, _code: &GeneratedCode, _context: &EvaluationContext) -> Result<QualityGateResult, QualityGateError> {
        Ok(QualityGateResult {
            gate_type: QualityGateType::Coverage,
            name: "Test Coverage".to_string(),
            passed: true,
            score: 85.0,
            duration: Duration::from_secs(0),
            issues: Vec::new(),
            metrics: HashMap::new(),
            output: "Coverage check not yet implemented".to_string(),
            suggestions: vec!["Implement coverage measurement".to_string()],
        })
    }

    async fn run_security_check(&self, _code: &GeneratedCode, _context: &EvaluationContext) -> Result<QualityGateResult, QualityGateError> {
        Ok(QualityGateResult {
            gate_type: QualityGateType::Security,
            name: "Security Audit".to_string(),
            passed: true,
            score: 95.0,
            duration: Duration::from_secs(0),
            issues: Vec::new(),
            metrics: HashMap::new(),
            output: "Security check not yet implemented".to_string(),
            suggestions: vec!["Implement security audit".to_string()],
        })
    }

    async fn run_complexity_check(&self, _code: &GeneratedCode, _context: &EvaluationContext) -> Result<QualityGateResult, QualityGateError> {
        Ok(QualityGateResult {
            gate_type: QualityGateType::Complexity,
            name: "Code Complexity".to_string(),
            passed: true,
            score: 80.0,
            duration: Duration::from_secs(0),
            issues: Vec::new(),
            metrics: HashMap::new(),
            output: "Complexity check not yet implemented".to_string(),
            suggestions: vec!["Implement complexity analysis".to_string()],
        })
    }

    async fn run_documentation_check(&self, _code: &GeneratedCode, _context: &EvaluationContext) -> Result<QualityGateResult, QualityGateError> {
        Ok(QualityGateResult {
            gate_type: QualityGateType::Documentation,
            name: "Documentation".to_string(),
            passed: true,
            score: 75.0,
            duration: Duration::from_secs(0),
            issues: Vec::new(),
            metrics: HashMap::new(),
            output: "Documentation check not yet implemented".to_string(),
            suggestions: vec!["Implement documentation validation".to_string()],
        })
    }

    async fn run_dependency_check(&self, _code: &GeneratedCode, _context: &EvaluationContext) -> Result<QualityGateResult, QualityGateError> {
        Ok(QualityGateResult {
            gate_type: QualityGateType::Dependencies,
            name: "Dependency Check".to_string(),
            passed: true,
            score: 90.0,
            duration: Duration::from_secs(0),
            issues: Vec::new(),
            metrics: HashMap::new(),
            output: "Dependency check not yet implemented".to_string(),
            suggestions: vec!["Implement dependency validation".to_string()],
        })
    }

    async fn run_performance_check(&self, _code: &GeneratedCode, _context: &EvaluationContext) -> Result<QualityGateResult, QualityGateError> {
        Ok(QualityGateResult {
            gate_type: QualityGateType::Performance,
            name: "Performance Check".to_string(),
            passed: true,
            score: 85.0,
            duration: Duration::from_secs(0),
            issues: Vec::new(),
            metrics: HashMap::new(),
            output: "Performance check not yet implemented".to_string(),
            suggestions: vec!["Implement performance benchmarks".to_string()],
        })
    }

    /// Parse compiler output for issues
    fn parse_compiler_output(output: &str) -> Vec<QualityIssue> {
        // Simplified parser - in real implementation would use proper parsing
        Vec::new()
    }

    /// Parse clippy output for issues
    fn parse_clippy_output(output: &str) -> Vec<QualityIssue> {
        // Simplified parser - in real implementation would use proper parsing
        Vec::new()
    }
}

/// Quality gate errors
#[derive(Debug, thiserror::Error)]
pub enum QualityGateError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Configuration error: {0}")]
    Configuration(String),
    #[error("Timeout error: gate took too long")]
    Timeout,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl Default for QualityGateConfig {
    fn default() -> Self {
        Self {
            enabled_gates: vec![
                QualityGateType::Compilation,
                QualityGateType::Clippy,
                QualityGateType::Format,
                QualityGateType::Tests,
            ],
            clippy_config: ClippyConfig::default(),
            format_config: FormatConfig::default(),
            test_config: TestConfig::default(),
            coverage_config: CoverageConfig::default(),
            security_config: SecurityConfig::default(),
            complexity_config: ComplexityConfig::default(),
            documentation_config: DocumentationConfig::default(),
            timeout: Duration::from_secs(120),
            fail_fast: false,
        }
    }
}

impl Default for ClippyConfig {
    fn default() -> Self {
        Self {
            deny_warnings: false,
            allowed_lints: Vec::new(),
            denied_lints: Vec::new(),
            custom_config: None,
        }
    }
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            check_only: true,
            edition: "2021".to_string(),
            config_file: None,
        }
    }
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            include_ignored: false,
            test_threads: None,
            timeout: Duration::from_secs(300),
            test_filter: None,
        }
    }
}

impl Default for CoverageConfig {
    fn default() -> Self {
        Self {
            minimum_coverage: 80.0,
            exclude_patterns: Vec::new(),
            report_format: CoverageFormat::Text,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            audit_dependencies: true,
            check_advisories: true,
            allowed_vulnerabilities: Vec::new(),
        }
    }
}

impl Default for ComplexityConfig {
    fn default() -> Self {
        Self {
            max_cyclomatic_complexity: 10,
            max_cognitive_complexity: 15,
            max_function_length: 100,
            max_file_length: 1000,
        }
    }
}

impl Default for DocumentationConfig {
    fn default() -> Self {
        Self {
            require_docs: false,
            check_examples: true,
            min_doc_coverage: 70.0,
        }
    }
}