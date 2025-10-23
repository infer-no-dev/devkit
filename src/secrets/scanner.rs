//! Secret scanner for detecting exposed credentials in code

use crate::secrets::{SecretsError, ScanConfig, CustomScanRule, SeverityLevel};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use walkdir::WalkDir;

/// Secret scanner for detecting potential secrets in files
#[derive(Debug)]
pub struct SecretScanner {
    config: ScanConfig,
    rules: Vec<ScanRule>,
}

/// A scan rule for detecting secrets
#[derive(Debug, Clone)]
pub struct ScanRule {
    pub name: String,
    pub description: String,
    pub pattern: Regex,
    pub severity: SeverityLevel,
    pub confidence: f64,
    pub keywords: Vec<String>,
    pub file_extensions: Vec<String>,
    pub entropy_threshold: Option<f64>,
}

/// Result of a secret scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub rule_name: String,
    pub file_path: String,
    pub line_number: usize,
    pub column_start: usize,
    pub column_end: usize,
    pub matched_text: String,
    pub context_before: String,
    pub context_after: String,
    pub severity: SeverityLevel,
    pub confidence: f64,
    pub description: String,
    pub remediation_advice: String,
}

/// Summary of scan results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSummary {
    pub total_files_scanned: usize,
    pub total_secrets_found: usize,
    pub secrets_by_severity: HashMap<String, usize>,
    pub secrets_by_type: HashMap<String, usize>,
    pub high_confidence_secrets: usize,
    pub scan_duration_ms: u64,
}

/// File scan context
#[derive(Debug)]
struct FileContext {
    path: PathBuf,
    content: String,
    lines: Vec<String>,
    extension: Option<String>,
}

impl SecretScanner {
    /// Create a new secret scanner
    pub fn new(config: ScanConfig) -> Self {
        let mut scanner = Self {
            config: config.clone(),
            rules: Vec::new(),
        };

        // Initialize built-in rules
        scanner.initialize_builtin_rules();
        
        // Add custom rules
        for custom_rule in &config.custom_rules {
            if let Ok(regex) = Regex::new(&custom_rule.pattern) {
                scanner.rules.push(ScanRule {
                    name: custom_rule.name.clone(),
                    description: custom_rule.description.clone(),
                    pattern: regex,
                    severity: custom_rule.severity.clone(),
                    confidence: custom_rule.confidence,
                    keywords: vec![],
                    file_extensions: vec![],
                    entropy_threshold: None,
                });
            }
        }

        scanner
    }

    /// Scan a single file for secrets
    pub async fn scan_file(&self, file_path: &Path) -> Result<Vec<ScanResult>, SecretsError> {
        if !self.should_scan_file(file_path) {
            return Ok(vec![]);
        }

        // Read file content
        let metadata = fs::metadata(file_path).await?;
        if metadata.len() > self.config.max_file_size as u64 {
            return Ok(vec![]);
        }

        let content = fs::read_to_string(file_path).await?;
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        
        let file_context = FileContext {
            path: file_path.to_path_buf(),
            content,
            lines,
            extension: file_path.extension().map(|ext| ext.to_string_lossy().to_string()),
        };

        let mut results = Vec::new();

        // Apply each rule to the file
        for rule in &self.rules {
            if self.rule_applies_to_file(rule, &file_context) {
                results.extend(self.apply_rule(rule, &file_context)?);
            }
        }

        Ok(results)
    }

    /// Scan a directory recursively
    pub async fn scan_path(&self, path: &Path) -> Result<Vec<ScanResult>, SecretsError> {
        let start_time = std::time::Instant::now();
        let mut all_results = Vec::new();
        let mut files_scanned = 0;

        if path.is_file() {
            all_results.extend(self.scan_file(path).await?);
            files_scanned = 1;
        } else if path.is_dir() {
            let walker = if self.config.recursive {
                WalkDir::new(path).into_iter()
            } else {
                WalkDir::new(path).max_depth(1).into_iter()
            };

            for entry in walker {
                match entry {
                    Ok(entry) if entry.file_type().is_file() => {
                        if self.should_scan_file(entry.path()) {
                            match self.scan_file(entry.path()).await {
                                Ok(results) => {
                                    all_results.extend(results);
                                    files_scanned += 1;
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to scan {}: {}", entry.path().display(), e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Error walking directory: {}", e);
                    }
                    _ => {}
                }
            }
        }

        let duration = start_time.elapsed();
        tracing::info!(
            "Scanned {} files in {}ms, found {} potential secrets",
            files_scanned,
            duration.as_millis(),
            all_results.len()
        );

        Ok(all_results)
    }

    /// Get scan summary
    pub fn get_summary(&self, results: &[ScanResult]) -> ScanSummary {
        let mut secrets_by_severity = HashMap::new();
        let mut secrets_by_type = HashMap::new();
        let mut high_confidence_count = 0;

        for result in results {
            // Count by severity
            let severity_str = match result.severity {
                SeverityLevel::Low => "low",
                SeverityLevel::Medium => "medium",
                SeverityLevel::High => "high",
                SeverityLevel::Critical => "critical",
            };
            *secrets_by_severity.entry(severity_str.to_string()).or_insert(0) += 1;

            // Count by type (rule name)
            *secrets_by_type.entry(result.rule_name.clone()).or_insert(0) += 1;

            // Count high confidence
            if result.confidence >= 0.8 {
                high_confidence_count += 1;
            }
        }

        ScanSummary {
            total_files_scanned: 0, // This would be tracked during scanning
            total_secrets_found: results.len(),
            secrets_by_severity,
            secrets_by_type,
            high_confidence_secrets: high_confidence_count,
            scan_duration_ms: 0, // This would be tracked during scanning
        }
    }

    /// Check if a file should be scanned
    fn should_scan_file(&self, file_path: &Path) -> bool {
        let path_str = file_path.to_string_lossy();

        // Check exclude patterns
        for pattern in &self.config.exclude_patterns {
            if self.matches_pattern(&path_str, pattern) {
                return false;
            }
        }

        // Check include patterns
        if self.config.file_patterns.is_empty() {
            return true;
        }

        for pattern in &self.config.file_patterns {
            if self.matches_pattern(&path_str, pattern) {
                return true;
            }
        }

        false
    }

    /// Check if a pattern matches a path
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        // Simple glob-style matching
        if pattern.contains('*') {
            let regex_pattern = pattern
                .replace(".", "\\.")
                .replace("*", ".*")
                .replace("?", ".");
            if let Ok(regex) = Regex::new(&format!("^{}$", regex_pattern)) {
                return regex.is_match(path);
            }
        }
        path.contains(pattern)
    }

    /// Check if a rule applies to a specific file
    fn rule_applies_to_file(&self, rule: &ScanRule, file_context: &FileContext) -> bool {
        // Check file extension filters
        if !rule.file_extensions.is_empty() {
            if let Some(ref ext) = file_context.extension {
                if !rule.file_extensions.contains(ext) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check keyword presence for performance
        if !rule.keywords.is_empty() {
            let content_lower = file_context.content.to_lowercase();
            let has_keyword = rule.keywords.iter().any(|keyword| {
                content_lower.contains(&keyword.to_lowercase())
            });
            if !has_keyword {
                return false;
            }
        }

        true
    }

    /// Apply a rule to a file
    fn apply_rule(&self, rule: &ScanRule, file_context: &FileContext) -> Result<Vec<ScanResult>, SecretsError> {
        let mut results = Vec::new();

        for (line_number, line) in file_context.lines.iter().enumerate() {
            for mat in rule.pattern.find_iter(line) {
                let matched_text = mat.as_str().to_string();
                
                // Apply entropy check if configured
                let mut confidence = rule.confidence;
                if let Some(entropy_threshold) = rule.entropy_threshold {
                    let entropy = self.calculate_entropy(&matched_text);
                    if entropy < entropy_threshold {
                        confidence *= 0.5; // Reduce confidence for low-entropy matches
                    } else {
                        confidence = (confidence + (entropy / entropy_threshold) * 0.2).min(1.0);
                    }
                }

                // Get context lines
                let context_before = if line_number > 0 {
                    file_context.lines[line_number - 1].clone()
                } else {
                    String::new()
                };
                
                let context_after = if line_number + 1 < file_context.lines.len() {
                    file_context.lines[line_number + 1].clone()
                } else {
                    String::new()
                };

                results.push(ScanResult {
                    rule_name: rule.name.clone(),
                    file_path: file_context.path.to_string_lossy().to_string(),
                    line_number: line_number + 1,
                    column_start: mat.start(),
                    column_end: mat.end(),
                    matched_text,
                    context_before,
                    context_after,
                    severity: rule.severity.clone(),
                    confidence,
                    description: rule.description.clone(),
                    remediation_advice: self.get_remediation_advice(&rule.name),
                });
            }
        }

        Ok(results)
    }

    /// Calculate Shannon entropy of a string
    fn calculate_entropy(&self, text: &str) -> f64 {
        if text.is_empty() {
            return 0.0;
        }

        let mut char_counts = HashMap::new();
        for c in text.chars() {
            *char_counts.entry(c).or_insert(0) += 1;
        }

        let length = text.len() as f64;
        let mut entropy = 0.0;

        for &count in char_counts.values() {
            let probability = count as f64 / length;
            entropy -= probability * probability.log2();
        }

        entropy
    }

    /// Get remediation advice for a specific rule
    fn get_remediation_advice(&self, rule_name: &str) -> String {
        match rule_name {
            "aws_access_key" | "aws_secret_key" => {
                "Use AWS IAM roles or environment variables. Consider AWS Secrets Manager.".to_string()
            }
            "github_token" => {
                "Use GitHub's secret scanning and rotate this token immediately.".to_string()
            }
            "private_key" => {
                "Remove private keys from code. Use secure key management systems.".to_string()
            }
            "password" => {
                "Never hardcode passwords. Use environment variables or secret managers.".to_string()
            }
            "api_key" => {
                "Store API keys in environment variables or secure vaults.".to_string()
            }
            "database_connection" => {
                "Use connection strings with environment variables for credentials.".to_string()
            }
            _ => {
                "Remove sensitive data from code and use secure alternatives.".to_string()
            }
        }
    }

    /// Initialize built-in scanning rules
    fn initialize_builtin_rules(&mut self) {
        let rules = vec![
            // AWS credentials
            ScanRule {
                name: "aws_access_key".to_string(),
                description: "AWS Access Key ID".to_string(),
                pattern: Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
                severity: SeverityLevel::Critical,
                confidence: 0.9,
                keywords: vec!["aws".to_string(), "access".to_string(), "key".to_string()],
                file_extensions: vec![],
                entropy_threshold: Some(3.0),
            },
            ScanRule {
                name: "aws_secret_key".to_string(),
                description: "AWS Secret Access Key".to_string(),
                pattern: Regex::new(r#"aws_secret_access_key\s*[=:]\s*['"]?([A-Za-z0-9/+=]{40})['"]?"#).unwrap(),
                severity: SeverityLevel::Critical,
                confidence: 0.95,
                keywords: vec!["aws".to_string(), "secret".to_string()],
                file_extensions: vec![],
                entropy_threshold: Some(4.0),
            },
            // GitHub tokens
            ScanRule {
                name: "github_token".to_string(),
                description: "GitHub Personal Access Token".to_string(),
                pattern: Regex::new(r"ghp_[A-Za-z0-9_]{36}").unwrap(),
                severity: SeverityLevel::Critical,
                confidence: 0.95,
                keywords: vec!["github".to_string(), "token".to_string()],
                file_extensions: vec![],
                entropy_threshold: None,
            },
            // Generic API keys
            ScanRule {
                name: "api_key".to_string(),
                description: "Generic API Key".to_string(),
                pattern: Regex::new(r#"(?i)(api[_-]?key|apikey)\s*[=:]\s*['"]?([A-Za-z0-9_\-]{20,})['"]?"#).unwrap(),
                severity: SeverityLevel::High,
                confidence: 0.7,
                keywords: vec!["api".to_string(), "key".to_string()],
                file_extensions: vec![],
                entropy_threshold: Some(3.5),
            },
            // Passwords
            ScanRule {
                name: "password".to_string(),
                description: "Hardcoded Password".to_string(),
                pattern: Regex::new(r#"(?i)(password|pwd|pass)\s*[=:]\s*['"]([^'"]{8,})['"]?"#).unwrap(),
                severity: SeverityLevel::High,
                confidence: 0.6,
                keywords: vec!["password".to_string(), "pwd".to_string(), "pass".to_string()],
                file_extensions: vec![],
                entropy_threshold: Some(2.5),
            },
            // Private keys
            ScanRule {
                name: "private_key".to_string(),
                description: "Private Key".to_string(),
                pattern: Regex::new(r"-----BEGIN\s+(RSA\s+)?PRIVATE KEY-----").unwrap(),
                severity: SeverityLevel::Critical,
                confidence: 0.95,
                keywords: vec!["private".to_string(), "key".to_string(), "begin".to_string()],
                file_extensions: vec![],
                entropy_threshold: None,
            },
            // Database connections
            ScanRule {
                name: "database_connection".to_string(),
                description: "Database Connection String".to_string(),
                pattern: Regex::new(r"(?i)(mysql|postgres|mongodb|redis)://[^:\s]*:[^@\s]*@").unwrap(),
                severity: SeverityLevel::High,
                confidence: 0.8,
                keywords: vec!["mysql".to_string(), "postgres".to_string(), "mongodb".to_string()],
                file_extensions: vec![],
                entropy_threshold: None,
            },
            // JWT tokens
            ScanRule {
                name: "jwt_token".to_string(),
                description: "JSON Web Token".to_string(),
                pattern: Regex::new(r"eyJ[A-Za-z0-9_-]*\.eyJ[A-Za-z0-9_-]*\.[A-Za-z0-9_-]*").unwrap(),
                severity: SeverityLevel::Medium,
                confidence: 0.7,
                keywords: vec!["jwt".to_string(), "token".to_string(), "bearer".to_string()],
                file_extensions: vec![],
                entropy_threshold: Some(4.0),
            },
            // Slack tokens
            ScanRule {
                name: "slack_token".to_string(),
                description: "Slack Token".to_string(),
                pattern: Regex::new(r"xox[baprs]-([0-9a-zA-Z]{10,48})").unwrap(),
                severity: SeverityLevel::High,
                confidence: 0.9,
                keywords: vec!["slack".to_string(), "xox".to_string()],
                file_extensions: vec![],
                entropy_threshold: None,
            },
            // Generic secrets with high entropy
            ScanRule {
                name: "high_entropy_string".to_string(),
                description: "High Entropy String (Potential Secret)".to_string(),
                pattern: Regex::new(r#"['"]([A-Za-z0-9+/=]{32,})['"]?"#).unwrap(),
                severity: SeverityLevel::Low,
                confidence: 0.3,
                keywords: vec![],
                file_extensions: vec![],
                entropy_threshold: Some(4.5),
            },
        ];

        self.rules.extend(rules);
    }
}

impl std::fmt::Display for SeverityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SeverityLevel::Low => write!(f, "LOW"),
            SeverityLevel::Medium => write!(f, "MEDIUM"),
            SeverityLevel::High => write!(f, "HIGH"),
            SeverityLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

impl std::fmt::Display for ScanResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} in {} (line {}, confidence: {:.1}%): {}",
            self.severity,
            self.rule_name,
            self.file_path,
            self.line_number,
            self.confidence * 100.0,
            self.description
        )
    }
}