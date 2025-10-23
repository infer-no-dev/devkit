//! Diff-first apply system with quality gates
//!
//! This module provides a safe, validated workflow for applying AI-generated code changes:
//! 1. Changes are presented as diffs for review
//! 2. Quality gates validate changes (format, lint, test, security)
//! 3. Changes can be applied incrementally or rolled back
//! 4. Full audit trail of what was changed and why

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;
use tokio::process::Command as AsyncCommand;

/// A single file change represented as a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub file_path: PathBuf,
    pub original_content: Option<String>,
    pub new_content: String,
    pub diff_text: String,
    pub change_type: ChangeType,
    pub metadata: DiffMetadata,
}

/// Type of change being made
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Create,
    Modify,
    Delete,
    Rename { old_path: PathBuf },
}

/// Metadata about a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffMetadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub agent_id: String,
    pub task_id: String,
    pub confidence_score: f32,
    pub estimated_lines_changed: usize,
    pub language: Option<String>,
    pub description: String,
}

/// Collection of file changes that should be applied together
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSet {
    pub id: String,
    pub title: String,
    pub description: String,
    pub files: Vec<FileDiff>,
    pub metadata: ChangeSetMetadata,
    pub validation_results: Option<ValidationResults>,
}

/// Metadata for a changeset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSetMetadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub agent_id: String,
    pub task_id: String,
    pub total_files: usize,
    pub total_lines_added: usize,
    pub total_lines_removed: usize,
    pub affects_tests: bool,
    pub affects_dependencies: bool,
}

/// Results from quality gate validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResults {
    pub gates: HashMap<String, GateResult>,
    pub overall_status: ValidationStatus,
    pub can_auto_apply: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub validated_at: chrono::DateTime<chrono::Utc>,
}

/// Result of a single quality gate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    pub name: String,
    pub status: GateStatus,
    pub message: String,
    pub details: Option<String>,
    pub execution_time_ms: u64,
    pub is_blocking: bool,
}

/// Status of a quality gate
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateStatus {
    Passed,
    Failed,
    Warning,
    Skipped,
    Error,
}

/// Overall validation status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationStatus {
    Pending,
    Passed,
    Failed,
    Warning,
}

/// Configuration for quality gates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateConfig {
    pub enabled_gates: Vec<String>,
    pub auto_apply_on_pass: bool,
    pub require_all_gates: bool,
    pub timeout_seconds: u64,
    pub parallel_execution: bool,
    pub custom_commands: HashMap<String, String>,
}

impl Default for QualityGateConfig {
    fn default() -> Self {
        Self {
            enabled_gates: vec![
                "format".to_string(),
                "lint".to_string(),
                "compile".to_string(),
                "test".to_string(),
                "security".to_string(),
            ],
            auto_apply_on_pass: false,
            require_all_gates: true,
            timeout_seconds: 300,
            parallel_execution: true,
            custom_commands: HashMap::new(),
        }
    }
}

/// Main diff application system
pub struct DiffApplySystem {
    config: QualityGateConfig,
    backup_dir: PathBuf,
    quality_gates: Vec<Box<dyn QualityGate>>,
    applied_changesets: HashMap<String, ChangeSet>,
}

/// Trait for implementing quality gates
#[async_trait::async_trait]
pub trait QualityGate: Send + Sync {
    /// Name of this quality gate
    fn name(&self) -> &str;
    
    /// Whether this gate is blocking (prevents apply if failed)
    fn is_blocking(&self) -> bool;
    
    /// Run the quality gate validation
    async fn validate(&self, changeset: &ChangeSet, project_root: &Path) -> Result<GateResult, QualityGateError>;
    
    /// Whether this gate can run in parallel with others
    fn can_run_parallel(&self) -> bool {
        true
    }
    
    /// Basic security check for changesets (provided method)
    async fn basic_security_check(&self, changeset: &ChangeSet, duration: std::time::Duration) -> Result<GateResult, QualityGateError> {
        let mut warnings = Vec::new();
        
        // Check for common security issues in code
        for file_diff in &changeset.files {
            let content = &file_diff.new_content;
            
            // Simple pattern matching for security issues
            if content.contains("password") && content.contains("=") {
                warnings.push("Possible hardcoded password detected".to_string());
            }
            if content.contains("api_key") || content.contains("secret_key") {
                warnings.push("Possible API key in code".to_string());
            }
            if content.contains("eval(") || content.contains("exec(") {
                warnings.push("Dangerous eval/exec usage detected".to_string());
            }
            if content.contains("unsafe {") {
                warnings.push("Unsafe code block detected".to_string());
            }
        }
        
        if warnings.is_empty() {
            Ok(GateResult {
                name: "security".to_string(),
                status: GateStatus::Passed,
                message: "Basic security checks passed".to_string(),
                details: None,
                execution_time_ms: duration.as_millis() as u64,
                is_blocking: false,
            })
        } else {
            Ok(GateResult {
                name: "security".to_string(),
                status: GateStatus::Warning,
                message: "Potential security issues found".to_string(),
                details: Some(warnings.join("\n")),
                execution_time_ms: duration.as_millis() as u64,
                is_blocking: false,
            })
        }
    }
}

/// Errors in the diff apply system
#[derive(Debug, Error)]
pub enum DiffApplyError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Quality gate error: {0}")]
    QualityGateError(#[from] QualityGateError),
    
    #[error("Backup failed: {0}")]
    BackupFailed(String),
    
    #[error("Rollback failed: {0}")]
    RollbackFailed(String),
    
    #[error("Changeset not found: {0}")]
    ChangesetNotFound(String),
    
    #[error("File conflict: {0}")]
    FileConflict(String),
}

/// Errors in quality gates
#[derive(Debug, Error)]
pub enum QualityGateError {
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
}

impl DiffApplySystem {
    /// Create a new diff apply system
    pub fn new(config: QualityGateConfig, project_root: &Path) -> Result<Self, DiffApplyError> {
        let backup_dir = project_root.join(".devkit").join("backups");
        std::fs::create_dir_all(&backup_dir)?;
        
        let mut system = Self {
            config: config.clone(),
            backup_dir,
            quality_gates: Vec::new(),
            applied_changesets: HashMap::new(),
        };
        
        // Register default quality gates
        system.register_default_gates();
        
        Ok(system)
    }
    
    /// Register default quality gates based on configuration
    fn register_default_gates(&mut self) {
        for gate_name in &self.config.enabled_gates {
            match gate_name.as_str() {
                "format" => self.quality_gates.push(Box::new(FormatGate::new())),
                "lint" => self.quality_gates.push(Box::new(LintGate::new())),
                "compile" => self.quality_gates.push(Box::new(CompileGate::new())),
                "test" => self.quality_gates.push(Box::new(TestGate::new())),
                "security" => self.quality_gates.push(Box::new(SecurityGate::new())),
                _ => {
                    if let Some(command) = self.config.custom_commands.get(gate_name) {
                        self.quality_gates.push(Box::new(CustomGate::new(gate_name, command)));
                    }
                }
            }
        }
    }
    
    /// Preview a changeset as diffs without applying
    pub async fn preview_changeset(&self, changeset: &ChangeSet) -> Result<String, DiffApplyError> {
        let mut preview = String::new();
        
        preview.push_str(&format!("# Changeset: {}\n", changeset.title));
        preview.push_str(&format!("{}\n\n", changeset.description));
        preview.push_str(&format!("Files affected: {}\n", changeset.files.len()));
        preview.push_str(&format!("Lines added: {}\n", changeset.metadata.total_lines_added));
        preview.push_str(&format!("Lines removed: {}\n\n", changeset.metadata.total_lines_removed));
        
        for file_diff in &changeset.files {
            preview.push_str(&format!("## {} {}\n\n", 
                match file_diff.change_type {
                    ChangeType::Create => "CREATE",
                    ChangeType::Modify => "MODIFY", 
                    ChangeType::Delete => "DELETE",
                    ChangeType::Rename { .. } => "RENAME",
                },
                file_diff.file_path.display()
            ));
            
            if !file_diff.diff_text.is_empty() {
                preview.push_str("```diff\n");
                preview.push_str(&file_diff.diff_text);
                preview.push_str("\n```\n\n");
            }
        }
        
        Ok(preview)
    }
    
    /// Validate a changeset through all quality gates
    pub async fn validate_changeset(&self, changeset: &mut ChangeSet, project_root: &Path) -> Result<(), DiffApplyError> {
        let mut results = ValidationResults {
            gates: HashMap::new(),
            overall_status: ValidationStatus::Pending,
            can_auto_apply: true,
            warnings: Vec::new(),
            errors: Vec::new(),
            validated_at: chrono::Utc::now(),
        };
        
        // Run quality gates
        if self.config.parallel_execution {
            self.run_gates_parallel(changeset, project_root, &mut results).await?;
        } else {
            self.run_gates_sequential(changeset, project_root, &mut results).await?;
        }
        
        // Determine overall status
        let mut has_failures = false;
        let mut has_warnings = false;
        
        for gate_result in results.gates.values() {
            match gate_result.status {
                GateStatus::Failed | GateStatus::Error => {
                    has_failures = true;
                    if gate_result.is_blocking {
                        results.can_auto_apply = false;
                    }
                    results.errors.push(format!("{}: {}", gate_result.name, gate_result.message));
                }
                GateStatus::Warning => {
                    has_warnings = true;
                    results.warnings.push(format!("{}: {}", gate_result.name, gate_result.message));
                }
                _ => {}
            }
        }
        
        results.overall_status = if has_failures {
            ValidationStatus::Failed
        } else if has_warnings {
            ValidationStatus::Warning
        } else {
            ValidationStatus::Passed
        };
        
        changeset.validation_results = Some(results);
        Ok(())
    }
    
    /// Run quality gates in parallel
    async fn run_gates_parallel(&self, changeset: &ChangeSet, project_root: &Path, results: &mut ValidationResults) -> Result<(), DiffApplyError> {
        // Note: In a real implementation, you'd spawn concurrent tasks
        // For now, we'll run them sequentially to avoid complexity
        
        for gate in &self.quality_gates {
            if gate.can_run_parallel() {
                let gate_name = gate.name().to_string();
                let gate_result = gate.validate(changeset, project_root).await?;
                results.gates.insert(gate_name, gate_result);
            }
        }
        
        Ok(())
    }
    
    /// Run quality gates sequentially
    async fn run_gates_sequential(&self, changeset: &ChangeSet, project_root: &Path, results: &mut ValidationResults) -> Result<(), DiffApplyError> {
        for gate in &self.quality_gates {
            let gate_result = gate.validate(changeset, project_root).await?;
            let gate_name = gate.name().to_string();
            
            // Stop on first blocking failure if required
            if self.config.require_all_gates && 
               gate_result.status == GateStatus::Failed && 
               gate_result.is_blocking {
                results.gates.insert(gate_name, gate_result);
                break;
            }
            
            results.gates.insert(gate_name, gate_result);
        }
        
        Ok(())
    }
    
    /// Apply a validated changeset
    pub async fn apply_changeset(&mut self, changeset: &ChangeSet, project_root: &Path, force: bool) -> Result<(), DiffApplyError> {
        // Check validation results
        if !force {
            if let Some(validation) = &changeset.validation_results {
                if !validation.can_auto_apply {
                    return Err(DiffApplyError::ValidationFailed(
                        format!("Changeset failed validation: {:?}", validation.errors)
                    ));
                }
            } else {
                return Err(DiffApplyError::ValidationFailed(
                    "Changeset has not been validated".to_string()
                ));
            }
        }
        
        // Create backup before applying changes
        self.create_backup(changeset, project_root).await?;
        
        // Apply each file change
        for file_diff in &changeset.files {
            self.apply_file_diff(file_diff, project_root).await?;
        }
        
        // Store applied changeset for potential rollback
        self.applied_changesets.insert(changeset.id.clone(), changeset.clone());
        
        Ok(())
    }
    
    /// Create backup of files that will be changed
    async fn create_backup(&self, changeset: &ChangeSet, project_root: &Path) -> Result<(), DiffApplyError> {
        let backup_path = self.backup_dir.join(&changeset.id);
        fs::create_dir_all(&backup_path).await?;
        
        for file_diff in &changeset.files {
            let file_path = project_root.join(&file_diff.file_path);
            
            if file_path.exists() {
                let backup_file = backup_path.join(&file_diff.file_path);
                
                // Create parent directories
                if let Some(parent) = backup_file.parent() {
                    fs::create_dir_all(parent).await?;
                }
                
                // Copy original file
                fs::copy(&file_path, &backup_file).await?;
            }
        }
        
        // Save changeset metadata
        let metadata_file = backup_path.join("changeset.json");
        let metadata_json = serde_json::to_string_pretty(changeset)
            .map_err(|e| DiffApplyError::BackupFailed(format!("Failed to serialize changeset: {}", e)))?;
        fs::write(metadata_file, metadata_json).await?;
        
        Ok(())
    }
    
    /// Apply a single file diff
    async fn apply_file_diff(&self, file_diff: &FileDiff, project_root: &Path) -> Result<(), DiffApplyError> {
        let file_path = project_root.join(&file_diff.file_path);
        
        match &file_diff.change_type {
            ChangeType::Create | ChangeType::Modify => {
                // Create parent directories if needed
                if let Some(parent) = file_path.parent() {
                    fs::create_dir_all(parent).await?;
                }
                
                // Write new content
                fs::write(&file_path, &file_diff.new_content).await?;
            }
            ChangeType::Delete => {
                if file_path.exists() {
                    fs::remove_file(&file_path).await?;
                }
            }
            ChangeType::Rename { old_path } => {
                let old_file_path = project_root.join(old_path);
                if old_file_path.exists() {
                    // Create parent directories for new location
                    if let Some(parent) = file_path.parent() {
                        fs::create_dir_all(parent).await?;
                    }
                    
                    // Move file
                    fs::rename(&old_file_path, &file_path).await?;
                    
                    // Update content if provided
                    if !file_diff.new_content.is_empty() {
                        fs::write(&file_path, &file_diff.new_content).await?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Rollback a previously applied changeset
    pub async fn rollback_changeset(&mut self, changeset_id: &str, project_root: &Path) -> Result<(), DiffApplyError> {
        let changeset = self.applied_changesets.get(changeset_id)
            .ok_or_else(|| DiffApplyError::ChangesetNotFound(changeset_id.to_string()))?
            .clone();
            
        let backup_path = self.backup_dir.join(changeset_id);
        
        if !backup_path.exists() {
            return Err(DiffApplyError::RollbackFailed(
                format!("Backup not found for changeset: {}", changeset_id)
            ));
        }
        
        // Restore files from backup
        for file_diff in changeset.files.iter().rev() {
            self.rollback_file_diff(file_diff, project_root, &backup_path).await?;
        }
        
        // Remove from applied changesets
        self.applied_changesets.remove(changeset_id);
        
        Ok(())
    }
    
    /// Rollback a single file diff
    async fn rollback_file_diff(&self, file_diff: &FileDiff, project_root: &Path, backup_path: &Path) -> Result<(), DiffApplyError> {
        let file_path = project_root.join(&file_diff.file_path);
        let backup_file = backup_path.join(&file_diff.file_path);
        
        match &file_diff.change_type {
            ChangeType::Create => {
                // Remove the created file
                if file_path.exists() {
                    fs::remove_file(&file_path).await?;
                }
            }
            ChangeType::Modify => {
                // Restore from backup
                if backup_file.exists() {
                    fs::copy(&backup_file, &file_path).await?;
                }
            }
            ChangeType::Delete => {
                // Restore the deleted file
                if backup_file.exists() {
                    // Create parent directories
                    if let Some(parent) = file_path.parent() {
                        fs::create_dir_all(parent).await?;
                    }
                    fs::copy(&backup_file, &file_path).await?;
                }
            }
            ChangeType::Rename { old_path } => {
                let old_file_path = project_root.join(old_path);
                
                // Move file back to original location
                if file_path.exists() {
                    // Create parent directories for original location
                    if let Some(parent) = old_file_path.parent() {
                        fs::create_dir_all(parent).await?;
                    }
                    fs::rename(&file_path, &old_file_path).await?;
                }
                
                // Restore original content if backed up
                if backup_file.exists() {
                    fs::copy(&backup_file, &old_file_path).await?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Get list of applied changesets
    pub fn get_applied_changesets(&self) -> Vec<&ChangeSet> {
        self.applied_changesets.values().collect()
    }
    
    /// Generate unified diff text for a file change
    pub fn generate_diff(original: Option<&str>, new: &str, file_path: &Path) -> String {
        match original {
            Some(orig) if orig != new => {
                // Simple unified diff format
                let mut diff = format!("--- {}\n+++ {}\n", file_path.display(), file_path.display());
                
                let orig_lines: Vec<&str> = orig.lines().collect();
                let new_lines: Vec<&str> = new.lines().collect();
                
                // Simple line-by-line diff (could be enhanced with proper diff algorithm)
                let max_lines = orig_lines.len().max(new_lines.len());
                let mut context = 3; // Lines of context
                
                for i in 0..max_lines {
                    let orig_line = orig_lines.get(i).unwrap_or(&"");
                    let new_line = new_lines.get(i).unwrap_or(&"");
                    
                    if orig_line != new_line {
                        if context == 3 {
                            diff.push_str(&format!("@@ -{},{} +{},{} @@\n", 
                                i.saturating_sub(3), orig_lines.len(),
                                i.saturating_sub(3), new_lines.len()
                            ));
                        }
                        context = 0;
                        
                        if !orig_line.is_empty() && orig_lines.get(i).is_some() {
                            diff.push_str(&format!("-{}\n", orig_line));
                        }
                        if !new_line.is_empty() && new_lines.get(i).is_some() {
                            diff.push_str(&format!("+{}\n", new_line));
                        }
                    } else if context < 3 {
                        diff.push_str(&format!(" {}\n", orig_line));
                        context += 1;
                    }
                }
                
                diff
            }
            None => {
                // New file
                let mut diff = format!("--- /dev/null\n+++ {}\n", file_path.display());
                diff.push_str(&format!("@@ -0,0 +1,{} @@\n", new.lines().count()));
                for line in new.lines() {
                    diff.push_str(&format!("+{}\n", line));
                }
                diff
            }
            Some(_) => {
                // No changes
                String::new()
            }
        }
    }
}

// Quality Gate Implementations

/// Format checking gate (cargo fmt, prettier, etc.)
pub struct FormatGate;

impl FormatGate {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl QualityGate for FormatGate {
    fn name(&self) -> &str {
        "format"
    }
    
    fn is_blocking(&self) -> bool {
        false // Format issues are usually auto-fixable
    }
    
    async fn validate(&self, changeset: &ChangeSet, project_root: &Path) -> Result<GateResult, QualityGateError> {
        let start = std::time::Instant::now();
        
        // Check if this is a Rust project
        let is_rust = project_root.join("Cargo.toml").exists();
        
        if is_rust {
            let output = AsyncCommand::new("cargo")
                .arg("fmt")
                .arg("--check")
                .current_dir(project_root)
                .output()
                .await
                .map_err(|e| QualityGateError::CommandFailed(format!("cargo fmt failed: {}", e)))?;
            
            let duration = start.elapsed();
            
            if output.status.success() {
                Ok(GateResult {
                    name: "format".to_string(),
                    status: GateStatus::Passed,
                    message: "All files are properly formatted".to_string(),
                    details: None,
                    execution_time_ms: duration.as_millis() as u64,
                    is_blocking: false,
                })
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Ok(GateResult {
                    name: "format".to_string(),
                    status: GateStatus::Warning,
                    message: "Some files need formatting".to_string(),
                    details: Some(stderr.to_string()),
                    execution_time_ms: duration.as_millis() as u64,
                    is_blocking: false,
                })
            }
        } else {
            Ok(GateResult {
                name: "format".to_string(),
                status: GateStatus::Skipped,
                message: "No format checker found for this project type".to_string(),
                details: None,
                execution_time_ms: 0,
                is_blocking: false,
            })
        }
    }
}

/// Lint checking gate (clippy, eslint, etc.)
pub struct LintGate;

impl LintGate {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl QualityGate for LintGate {
    fn name(&self) -> &str {
        "lint"
    }
    
    fn is_blocking(&self) -> bool {
        true
    }
    
    async fn validate(&self, changeset: &ChangeSet, project_root: &Path) -> Result<GateResult, QualityGateError> {
        let start = std::time::Instant::now();
        
        if project_root.join("Cargo.toml").exists() {
            let output = AsyncCommand::new("cargo")
                .arg("clippy")
                .arg("--")
                .arg("-D")
                .arg("warnings")
                .current_dir(project_root)
                .output()
                .await
                .map_err(|e| QualityGateError::CommandFailed(format!("cargo clippy failed: {}", e)))?;
            
            let duration = start.elapsed();
            
            if output.status.success() {
                Ok(GateResult {
                    name: "lint".to_string(),
                    status: GateStatus::Passed,
                    message: "No lint issues found".to_string(),
                    details: None,
                    execution_time_ms: duration.as_millis() as u64,
                    is_blocking: true,
                })
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Ok(GateResult {
                    name: "lint".to_string(),
                    status: GateStatus::Failed,
                    message: "Lint issues found".to_string(),
                    details: Some(stderr.to_string()),
                    execution_time_ms: duration.as_millis() as u64,
                    is_blocking: true,
                })
            }
        } else {
            Ok(GateResult {
                name: "lint".to_string(),
                status: GateStatus::Skipped,
                message: "No linter found for this project type".to_string(),
                details: None,
                execution_time_ms: 0,
                is_blocking: false,
            })
        }
    }
}

/// Compilation checking gate
pub struct CompileGate;

impl CompileGate {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl QualityGate for CompileGate {
    fn name(&self) -> &str {
        "compile"
    }
    
    fn is_blocking(&self) -> bool {
        true
    }
    
    async fn validate(&self, changeset: &ChangeSet, project_root: &Path) -> Result<GateResult, QualityGateError> {
        let start = std::time::Instant::now();
        
        if project_root.join("Cargo.toml").exists() {
            let output = AsyncCommand::new("cargo")
                .arg("check")
                .current_dir(project_root)
                .output()
                .await
                .map_err(|e| QualityGateError::CommandFailed(format!("cargo check failed: {}", e)))?;
            
            let duration = start.elapsed();
            
            if output.status.success() {
                Ok(GateResult {
                    name: "compile".to_string(),
                    status: GateStatus::Passed,
                    message: "Code compiles successfully".to_string(),
                    details: None,
                    execution_time_ms: duration.as_millis() as u64,
                    is_blocking: true,
                })
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Ok(GateResult {
                    name: "compile".to_string(),
                    status: GateStatus::Failed,
                    message: "Compilation failed".to_string(),
                    details: Some(stderr.to_string()),
                    execution_time_ms: duration.as_millis() as u64,
                    is_blocking: true,
                })
            }
        } else {
            Ok(GateResult {
                name: "compile".to_string(),
                status: GateStatus::Skipped,
                message: "No compilation check available for this project type".to_string(),
                details: None,
                execution_time_ms: 0,
                is_blocking: false,
            })
        }
    }
}

/// Test execution gate
pub struct TestGate;

impl TestGate {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl QualityGate for TestGate {
    fn name(&self) -> &str {
        "test"
    }
    
    fn is_blocking(&self) -> bool {
        true
    }
    
    async fn validate(&self, changeset: &ChangeSet, project_root: &Path) -> Result<GateResult, QualityGateError> {
        let start = std::time::Instant::now();
        
        if project_root.join("Cargo.toml").exists() {
            let output = AsyncCommand::new("cargo")
                .arg("test")
                .current_dir(project_root)
                .output()
                .await
                .map_err(|e| QualityGateError::CommandFailed(format!("cargo test failed: {}", e)))?;
            
            let duration = start.elapsed();
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            if output.status.success() {
                Ok(GateResult {
                    name: "test".to_string(),
                    status: GateStatus::Passed,
                    message: "All tests passed".to_string(),
                    details: Some(stdout.to_string()),
                    execution_time_ms: duration.as_millis() as u64,
                    is_blocking: true,
                })
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Ok(GateResult {
                    name: "test".to_string(),
                    status: GateStatus::Failed,
                    message: "Some tests failed".to_string(),
                    details: Some(format!("STDOUT:\n{}\n\nSTDERR:\n{}", stdout, stderr)),
                    execution_time_ms: duration.as_millis() as u64,
                    is_blocking: true,
                })
            }
        } else {
            Ok(GateResult {
                name: "test".to_string(),
                status: GateStatus::Skipped,
                message: "No test runner found for this project type".to_string(),
                details: None,
                execution_time_ms: 0,
                is_blocking: false,
            })
        }
    }
}

/// Security scanning gate
pub struct SecurityGate;

impl SecurityGate {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl QualityGate for SecurityGate {
    fn name(&self) -> &str {
        "security"
    }
    
    fn is_blocking(&self) -> bool {
        true
    }
    
    async fn validate(&self, changeset: &ChangeSet, project_root: &Path) -> Result<GateResult, QualityGateError> {
        let start = std::time::Instant::now();
        
        // Check for cargo-audit in Rust projects
        if project_root.join("Cargo.toml").exists() {
            let output = AsyncCommand::new("cargo")
                .arg("audit")
                .current_dir(project_root)
                .output()
                .await;
                
            let duration = start.elapsed();
            
            match output {
                Ok(output) if output.status.success() => {
                    Ok(GateResult {
                        name: "security".to_string(),
                        status: GateStatus::Passed,
                        message: "No known security vulnerabilities found".to_string(),
                        details: None,
                        execution_time_ms: duration.as_millis() as u64,
                        is_blocking: true,
                    })
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Ok(GateResult {
                        name: "security".to_string(),
                        status: GateStatus::Warning,
                        message: "Security scan completed with warnings".to_string(),
                        details: Some(stderr.to_string()),
                        execution_time_ms: duration.as_millis() as u64,
                        is_blocking: false,
                    })
                }
                Err(_) => {
                    // cargo-audit might not be installed, do basic checks
                    self.basic_security_check(changeset, duration).await
                }
            }
        } else {
            let duration = start.elapsed();
            self.basic_security_check(changeset, duration).await
        }
    }
    
}

/// Custom command quality gate
pub struct CustomGate {
    name: String,
    command: String,
}

impl CustomGate {
    pub fn new(name: &str, command: &str) -> Self {
        Self {
            name: name.to_string(),
            command: command.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl QualityGate for CustomGate {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn is_blocking(&self) -> bool {
        true
    }
    
    async fn validate(&self, changeset: &ChangeSet, project_root: &Path) -> Result<GateResult, QualityGateError> {
        let start = std::time::Instant::now();
        
        let mut cmd = AsyncCommand::new("sh");
        cmd.arg("-c")
           .arg(&self.command)
           .current_dir(project_root);
        
        let output = cmd.output()
            .await
            .map_err(|e| QualityGateError::CommandFailed(format!("Custom command failed: {}", e)))?;
        
        let duration = start.elapsed();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        if output.status.success() {
            Ok(GateResult {
                name: self.name.clone(),
                status: GateStatus::Passed,
                message: format!("Custom gate '{}' passed", self.name),
                details: if !stdout.is_empty() { Some(stdout.to_string()) } else { None },
                execution_time_ms: duration.as_millis() as u64,
                is_blocking: true,
            })
        } else {
            Ok(GateResult {
                name: self.name.clone(),
                status: GateStatus::Failed,
                message: format!("Custom gate '{}' failed", self.name),
                details: Some(format!("STDOUT:\n{}\n\nSTDERR:\n{}", stdout, stderr)),
                execution_time_ms: duration.as_millis() as u64,
                is_blocking: true,
            })
        }
    }
}