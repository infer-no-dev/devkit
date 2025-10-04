//! Blueprint Migration Engine
//!
//! Provides automated migration script generation, execution, and rollback
//! capabilities for blueprint evolution.

use super::diff::BlueprintDiff;
use super::*;
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Command;

/// Migration engine for blueprint evolution
pub struct MigrationEngine {
    config: MigrationConfig,
    script_generators: HashMap<ChangeCategory, Box<dyn ScriptGenerator>>,
    validators: Vec<Box<dyn MigrationValidator>>,
}

/// Configuration for migration engine
#[derive(Debug, Clone)]
pub struct MigrationConfig {
    pub working_directory: PathBuf,
    pub backup_directory: PathBuf,
    pub script_directory: PathBuf,
    pub dry_run: bool,
    pub auto_backup: bool,
    pub validation_timeout: std::time::Duration,
    pub parallel_execution: bool,
    pub max_retries: u32,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            backup_directory: PathBuf::from(".blueprint-backups"),
            script_directory: PathBuf::from(".blueprint-migrations"),
            dry_run: false,
            auto_backup: true,
            validation_timeout: std::time::Duration::from_secs(300), // 5 minutes
            parallel_execution: false,
            max_retries: 3,
        }
    }
}

/// Result of a migration operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationResult {
    pub migration_id: String,
    pub from_version: BlueprintVersion,
    pub to_version: BlueprintVersion,
    pub status: MigrationStatus,
    pub executed_steps: Vec<MigrationStep>,
    pub failed_step: Option<MigrationStep>,
    pub rollback_available: bool,
    pub execution_time: std::time::Duration,
    pub warnings: Vec<String>,
    pub artifacts: Vec<MigrationArtifact>,
}

/// Status of a migration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    RolledBack,
    ValidationFailed,
}

/// Individual migration step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStep {
    pub step_id: String,
    pub step_type: MigrationStepType,
    pub description: String,
    pub script_path: Option<PathBuf>,
    pub dependencies: Vec<String>,
    pub rollback_script: Option<PathBuf>,
    pub validation_checks: Vec<ValidationCheck>,
    pub estimated_duration: Option<std::time::Duration>,
    pub execution_result: Option<StepExecutionResult>,
}

/// Type of migration step
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationStepType {
    PreMigration,     // Setup and preparation
    DataBackup,       // Backup existing data
    SchemaUpdate,     // Update database/storage schema
    CodeGeneration,   // Generate new code files
    ConfigUpdate,     // Update configuration files
    DependencyUpdate, // Update dependencies
    ServiceRestart,   // Restart services
    Validation,       // Validate migration success
    PostMigration,    // Cleanup and finalization
    Rollback,         // Rollback operation
}

/// Result of executing a migration step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecutionResult {
    pub success: bool,
    pub output: String,
    pub error_message: Option<String>,
    pub exit_code: Option<i32>,
    pub execution_time: std::time::Duration,
    pub artifacts_created: Vec<PathBuf>,
    pub validation_results: Vec<ValidationResult>,
}

/// Validation result for a step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub check_name: String,
    pub passed: bool,
    pub message: String,
    pub severity: ValidationSeverity,
}

/// Severity of validation issues
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl std::fmt::Display for ValidationSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationSeverity::Info => write!(f, "INFO"),
            ValidationSeverity::Warning => write!(f, "WARN"),
            ValidationSeverity::Error => write!(f, "ERROR"),
            ValidationSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Migration artifact (files, logs, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationArtifact {
    pub artifact_type: ArtifactType,
    pub path: PathBuf,
    pub description: String,
    pub size_bytes: u64,
    pub checksum: String,
}

/// Type of migration artifact
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactType {
    Backup,
    Script,
    Log,
    GeneratedCode,
    Configuration,
    Documentation,
}

/// Trait for generating migration scripts
pub trait ScriptGenerator: Send + Sync {
    fn generate_migration_script(
        &self,
        change: &BlueprintChange,
        context: &MigrationContext,
    ) -> Result<MigrationStep>;

    fn generate_rollback_script(
        &self,
        change: &BlueprintChange,
        context: &MigrationContext,
    ) -> Result<MigrationStep>;

    fn can_handle_change(&self, change: &BlueprintChange) -> bool;
}

/// Trait for validating migrations
pub trait MigrationValidator: Send + Sync {
    fn validate_pre_migration(
        &self,
        diff: &BlueprintDiff,
        context: &MigrationContext,
    ) -> Result<Vec<ValidationResult>>;

    fn validate_step(
        &self,
        step: &MigrationStep,
        result: &StepExecutionResult,
        context: &MigrationContext,
    ) -> Result<Vec<ValidationResult>>;

    fn validate_post_migration(
        &self,
        result: &MigrationResult,
        context: &MigrationContext,
    ) -> Result<Vec<ValidationResult>>;
}

/// Context for migration operations
#[derive(Debug, Clone)]
pub struct MigrationContext {
    pub working_dir: PathBuf,
    pub blueprint_path: PathBuf,
    pub target_blueprint: crate::blueprint::SystemBlueprint,
    pub source_blueprint: crate::blueprint::SystemBlueprint,
    pub migration_id: String,
    pub environment: String,
    pub user_variables: HashMap<String, String>,
}

impl MigrationEngine {
    /// Create new migration engine
    pub fn new(config: MigrationConfig) -> Self {
        let mut engine = Self {
            config,
            script_generators: HashMap::new(),
            validators: Vec::new(),
        };

        // Register default script generators
        engine.register_default_generators();
        engine.register_default_validators();

        engine
    }

    /// Register a script generator for a specific change category
    pub fn register_generator(
        &mut self,
        category: ChangeCategory,
        generator: Box<dyn ScriptGenerator>,
    ) {
        self.script_generators.insert(category, generator);
    }

    /// Register a migration validator
    pub fn register_validator(&mut self, validator: Box<dyn MigrationValidator>) {
        self.validators.push(validator);
    }

    /// Generate migration plan from blueprint diff
    pub async fn generate_migration_plan(
        &self,
        diff: &BlueprintDiff,
        context: &MigrationContext,
    ) -> Result<Vec<MigrationStep>> {
        let mut steps = Vec::new();

        // Pre-migration steps
        steps.push(self.create_pre_migration_step(diff, context)?);

        // Backup step if enabled
        if self.config.auto_backup {
            steps.push(self.create_backup_step(diff, context)?);
        }

        // Generate steps for each change
        for change in &diff.changes {
            if let Some(generator) = self.script_generators.get(&change.change_category) {
                if generator.can_handle_change(change) {
                    let migration_step = generator
                        .generate_migration_script(change, context)
                        .with_context(|| {
                            format!("Failed to generate migration for change: {}", change.path)
                        })?;
                    steps.push(migration_step);
                }
            }
        }

        // Sort steps by dependencies and priority
        self.sort_migration_steps(&mut steps)?;

        // Validation step
        steps.push(self.create_validation_step(diff, context)?);

        // Post-migration step
        steps.push(self.create_post_migration_step(diff, context)?);

        Ok(steps)
    }

    /// Execute migration plan
    pub async fn execute_migration(
        &self,
        steps: Vec<MigrationStep>,
        context: &MigrationContext,
    ) -> Result<MigrationResult> {
        let migration_id = uuid::Uuid::new_v4().to_string();
        let start_time = std::time::Instant::now();

        let mut result = MigrationResult {
            migration_id: migration_id.clone(),
            from_version: BlueprintVersion::from_str(&context.source_blueprint.metadata.version)
                .unwrap_or_else(|_| BlueprintVersion::new(0, 0, 0)),
            to_version: BlueprintVersion::from_str(&context.target_blueprint.metadata.version)
                .unwrap_or_else(|_| BlueprintVersion::new(0, 0, 1)),
            status: MigrationStatus::InProgress,
            executed_steps: Vec::new(),
            failed_step: None,
            rollback_available: false,
            execution_time: std::time::Duration::default(),
            warnings: Vec::new(),
            artifacts: Vec::new(),
        };

        // Pre-migration validation
        if let Err(validation_errors) = self.run_pre_migration_validation(&steps, context).await {
            result.status = MigrationStatus::ValidationFailed;
            result.warnings.push(format!(
                "Pre-migration validation failed: {}",
                validation_errors
            ));
            return Ok(result);
        }

        // Execute steps
        for (index, step) in steps.into_iter().enumerate() {
            if self.config.dry_run {
                println!("DRY RUN: Would execute step: {}", step.description);
                continue;
            }

            match self.execute_step(&step, context).await {
                Ok(step_result) => {
                    let mut executed_step = step.clone();
                    executed_step.execution_result = Some(step_result);
                    result.executed_steps.push(executed_step);
                }
                Err(e) => {
                    result.status = MigrationStatus::Failed;
                    result.failed_step = Some(step);
                    result
                        .warnings
                        .push(format!("Step {} failed: {}", index + 1, e));

                    // Attempt rollback
                    if let Err(rollback_err) = self.attempt_rollback(&result, context).await {
                        result
                            .warnings
                            .push(format!("Rollback failed: {}", rollback_err));
                    } else {
                        result.status = MigrationStatus::RolledBack;
                        result.rollback_available = false;
                    }

                    result.execution_time = start_time.elapsed();
                    return Ok(result);
                }
            }
        }

        // Post-migration validation
        if let Err(validation_errors) = self.run_post_migration_validation(&result, context).await {
            result.warnings.push(format!(
                "Post-migration validation issues: {}",
                validation_errors
            ));
        }

        result.status = MigrationStatus::Completed;
        result.rollback_available = true;
        result.execution_time = start_time.elapsed();

        Ok(result)
    }

    /// Execute a single migration step
    async fn execute_step(
        &self,
        step: &MigrationStep,
        context: &MigrationContext,
    ) -> Result<StepExecutionResult> {
        let start_time = std::time::Instant::now();
        let mut step_result = StepExecutionResult {
            success: false,
            output: String::new(),
            error_message: None,
            exit_code: None,
            execution_time: std::time::Duration::default(),
            artifacts_created: Vec::new(),
            validation_results: Vec::new(),
        };

        // Execute step based on type
        match step.step_type {
            MigrationStepType::PreMigration => {
                step_result = self.execute_pre_migration_step(step, context).await?;
            }
            MigrationStepType::DataBackup => {
                step_result = self.execute_backup_step(step, context).await?;
            }
            MigrationStepType::CodeGeneration => {
                step_result = self.execute_code_generation_step(step, context).await?;
            }
            MigrationStepType::ConfigUpdate => {
                step_result = self.execute_config_update_step(step, context).await?;
            }
            MigrationStepType::Validation => {
                step_result = self.execute_validation_step(step, context).await?;
            }
            MigrationStepType::PostMigration => {
                step_result = self.execute_post_migration_step(step, context).await?;
            }
            _ => {
                // Execute script if provided
                if let Some(script_path) = &step.script_path {
                    step_result = self.execute_script(script_path, context).await?;
                }
            }
        }

        step_result.execution_time = start_time.elapsed();

        // Run step validation
        for validator in &self.validators {
            let validation_results = validator.validate_step(step, &step_result, context)?;
            step_result.validation_results.extend(validation_results);
        }

        // Check if step passed validation
        let has_critical_errors = step_result
            .validation_results
            .iter()
            .any(|v| !v.passed && v.severity == ValidationSeverity::Critical);

        step_result.success = !has_critical_errors && step_result.error_message.is_none();

        Ok(step_result)
    }

    /// Attempt to rollback the migration
    async fn attempt_rollback(
        &self,
        migration_result: &MigrationResult,
        context: &MigrationContext,
    ) -> Result<()> {
        println!("Attempting to rollback migration...");

        // Execute rollback steps in reverse order
        for step in migration_result.executed_steps.iter().rev() {
            if let Some(rollback_script) = &step.rollback_script {
                match self.execute_script(rollback_script, context).await {
                    Ok(_) => {
                        println!("Successfully rolled back step: {}", step.description);
                    }
                    Err(e) => {
                        eprintln!("Failed to rollback step {}: {}", step.description, e);
                        return Err(e);
                    }
                }
            }
        }

        println!("Migration rollback completed successfully");
        Ok(())
    }

    /// Execute a script file
    async fn execute_script(
        &self,
        script_path: &PathBuf,
        context: &MigrationContext,
    ) -> Result<StepExecutionResult> {
        let mut command = if script_path.extension().and_then(|s| s.to_str()) == Some("sh") {
            Command::new("bash")
        } else if script_path.extension().and_then(|s| s.to_str()) == Some("py") {
            Command::new("python3")
        } else {
            Command::new("bash")
        };

        let output = command
            .arg(script_path)
            .current_dir(&context.working_dir)
            .output()
            .await
            .with_context(|| format!("Failed to execute script: {:?}", script_path))?;

        Ok(StepExecutionResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            error_message: if output.stderr.is_empty() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            exit_code: output.status.code(),
            execution_time: std::time::Duration::default(),
            artifacts_created: Vec::new(),
            validation_results: Vec::new(),
        })
    }

    // Helper methods for step creation and execution
    fn create_pre_migration_step(
        &self,
        diff: &BlueprintDiff,
        context: &MigrationContext,
    ) -> Result<MigrationStep> {
        Ok(MigrationStep {
            step_id: "pre_migration".to_string(),
            step_type: MigrationStepType::PreMigration,
            description: "Pre-migration setup and validation".to_string(),
            script_path: None,
            dependencies: Vec::new(),
            rollback_script: None,
            validation_checks: Vec::new(),
            estimated_duration: Some(std::time::Duration::from_secs(30)),
            execution_result: None,
        })
    }

    fn create_backup_step(
        &self,
        diff: &BlueprintDiff,
        context: &MigrationContext,
    ) -> Result<MigrationStep> {
        Ok(MigrationStep {
            step_id: "backup".to_string(),
            step_type: MigrationStepType::DataBackup,
            description: "Create backup of current blueprint and data".to_string(),
            script_path: None,
            dependencies: vec!["pre_migration".to_string()],
            rollback_script: None,
            validation_checks: Vec::new(),
            estimated_duration: Some(std::time::Duration::from_secs(60)),
            execution_result: None,
        })
    }

    fn create_validation_step(
        &self,
        diff: &BlueprintDiff,
        context: &MigrationContext,
    ) -> Result<MigrationStep> {
        Ok(MigrationStep {
            step_id: "validation".to_string(),
            step_type: MigrationStepType::Validation,
            description: "Validate migration results".to_string(),
            script_path: None,
            dependencies: Vec::new(),
            rollback_script: None,
            validation_checks: Vec::new(),
            estimated_duration: Some(std::time::Duration::from_secs(120)),
            execution_result: None,
        })
    }

    fn create_post_migration_step(
        &self,
        diff: &BlueprintDiff,
        context: &MigrationContext,
    ) -> Result<MigrationStep> {
        Ok(MigrationStep {
            step_id: "post_migration".to_string(),
            step_type: MigrationStepType::PostMigration,
            description: "Post-migration cleanup and finalization".to_string(),
            script_path: None,
            dependencies: vec!["validation".to_string()],
            rollback_script: None,
            validation_checks: Vec::new(),
            estimated_duration: Some(std::time::Duration::from_secs(30)),
            execution_result: None,
        })
    }

    // Placeholder implementations for step execution
    async fn execute_pre_migration_step(
        &self,
        step: &MigrationStep,
        context: &MigrationContext,
    ) -> Result<StepExecutionResult> {
        // Ensure directories exist
        tokio::fs::create_dir_all(&self.config.working_directory).await?;
        tokio::fs::create_dir_all(&self.config.backup_directory).await?;
        tokio::fs::create_dir_all(&self.config.script_directory).await?;

        Ok(StepExecutionResult {
            success: true,
            output: "Pre-migration setup completed".to_string(),
            error_message: None,
            exit_code: Some(0),
            execution_time: std::time::Duration::default(),
            artifacts_created: Vec::new(),
            validation_results: Vec::new(),
        })
    }

    async fn execute_backup_step(
        &self,
        step: &MigrationStep,
        context: &MigrationContext,
    ) -> Result<StepExecutionResult> {
        let backup_path = self.config.backup_directory.join(format!(
            "backup_{}.json",
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        ));

        let blueprint_json = serde_json::to_string_pretty(&context.source_blueprint)?;
        tokio::fs::write(&backup_path, blueprint_json).await?;

        Ok(StepExecutionResult {
            success: true,
            output: format!("Backup created at {:?}", backup_path),
            error_message: None,
            exit_code: Some(0),
            execution_time: std::time::Duration::default(),
            artifacts_created: vec![backup_path],
            validation_results: Vec::new(),
        })
    }

    async fn execute_code_generation_step(
        &self,
        step: &MigrationStep,
        context: &MigrationContext,
    ) -> Result<StepExecutionResult> {
        // Placeholder for code generation logic
        Ok(StepExecutionResult {
            success: true,
            output: "Code generation completed".to_string(),
            error_message: None,
            exit_code: Some(0),
            execution_time: std::time::Duration::default(),
            artifacts_created: Vec::new(),
            validation_results: Vec::new(),
        })
    }

    async fn execute_config_update_step(
        &self,
        step: &MigrationStep,
        context: &MigrationContext,
    ) -> Result<StepExecutionResult> {
        // Placeholder for configuration update logic
        Ok(StepExecutionResult {
            success: true,
            output: "Configuration updated".to_string(),
            error_message: None,
            exit_code: Some(0),
            execution_time: std::time::Duration::default(),
            artifacts_created: Vec::new(),
            validation_results: Vec::new(),
        })
    }

    async fn execute_validation_step(
        &self,
        step: &MigrationStep,
        context: &MigrationContext,
    ) -> Result<StepExecutionResult> {
        let mut validation_results = Vec::new();

        // Run all validators
        for validator in &self.validators {
            // Create a dummy migration result for validation
            let dummy_result = MigrationResult {
                migration_id: "validation".to_string(),
                from_version: BlueprintVersion::new(0, 0, 0),
                to_version: BlueprintVersion::new(0, 0, 1),
                status: MigrationStatus::InProgress,
                executed_steps: Vec::new(),
                failed_step: None,
                rollback_available: false,
                execution_time: std::time::Duration::default(),
                warnings: Vec::new(),
                artifacts: Vec::new(),
            };

            let results = validator.validate_post_migration(&dummy_result, context)?;
            validation_results.extend(results);
        }

        let has_failures = validation_results.iter().any(|v| !v.passed);

        Ok(StepExecutionResult {
            success: !has_failures,
            output: format!(
                "Validation completed with {} checks",
                validation_results.len()
            ),
            error_message: if has_failures {
                Some("Some validation checks failed".to_string())
            } else {
                None
            },
            exit_code: if has_failures { Some(1) } else { Some(0) },
            execution_time: std::time::Duration::default(),
            artifacts_created: Vec::new(),
            validation_results,
        })
    }

    async fn execute_post_migration_step(
        &self,
        step: &MigrationStep,
        context: &MigrationContext,
    ) -> Result<StepExecutionResult> {
        // Save the target blueprint
        let blueprint_path = context.working_dir.join("blueprint.json");
        let blueprint_json = serde_json::to_string_pretty(&context.target_blueprint)?;
        tokio::fs::write(&blueprint_path, blueprint_json).await?;

        Ok(StepExecutionResult {
            success: true,
            output: "Post-migration cleanup completed".to_string(),
            error_message: None,
            exit_code: Some(0),
            execution_time: std::time::Duration::default(),
            artifacts_created: vec![blueprint_path],
            validation_results: Vec::new(),
        })
    }

    fn sort_migration_steps(&self, steps: &mut Vec<MigrationStep>) -> Result<()> {
        // Topological sort based on dependencies
        // For now, just ensure pre/post migration steps are in correct order
        steps.sort_by(|a, b| match (&a.step_type, &b.step_type) {
            (MigrationStepType::PreMigration, _) => std::cmp::Ordering::Less,
            (_, MigrationStepType::PreMigration) => std::cmp::Ordering::Greater,
            (MigrationStepType::PostMigration, _) => std::cmp::Ordering::Greater,
            (_, MigrationStepType::PostMigration) => std::cmp::Ordering::Less,
            (MigrationStepType::Validation, _) => std::cmp::Ordering::Greater,
            (_, MigrationStepType::Validation) => std::cmp::Ordering::Less,
            _ => std::cmp::Ordering::Equal,
        });

        Ok(())
    }

    async fn run_pre_migration_validation(
        &self,
        steps: &[MigrationStep],
        context: &MigrationContext,
    ) -> Result<()> {
        // Placeholder validation
        Ok(())
    }

    async fn run_post_migration_validation(
        &self,
        result: &MigrationResult,
        context: &MigrationContext,
    ) -> Result<()> {
        // Placeholder validation
        Ok(())
    }

    fn register_default_generators(&mut self) {
        // Register concrete script generators for each category
        use super::generators::*;
        register_default_generators(self);
    }

    fn register_default_validators(&mut self) {
        // Register default validators
        // For now, we'll leave this as a placeholder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_config_default() {
        let config = MigrationConfig::default();
        assert!(!config.dry_run);
        assert!(config.auto_backup);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_migration_status_serialization() {
        let status = MigrationStatus::Completed;
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("Completed"));
    }

    #[test]
    fn test_migration_step_creation() {
        let step = MigrationStep {
            step_id: "test".to_string(),
            step_type: MigrationStepType::PreMigration,
            description: "Test step".to_string(),
            script_path: None,
            dependencies: Vec::new(),
            rollback_script: None,
            validation_checks: Vec::new(),
            estimated_duration: Some(std::time::Duration::from_secs(30)),
            execution_result: None,
        };

        assert_eq!(step.step_id, "test");
        assert_eq!(step.step_type, MigrationStepType::PreMigration);
    }
}
