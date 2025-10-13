//! Blueprint Replicator
//!
//! This module orchestrates the complete system self-replication process,
//! combining blueprint extraction, validation, and project generation.

use super::extractor::BlueprintExtractor;
use super::generator::BlueprintGenerator;
use super::*;
use anyhow::Result;
use std::path::{Path, PathBuf};
use tokio::fs;

/// System replicator that handles the complete self-replication workflow
pub struct SystemReplicator {
    source_path: PathBuf,
    target_path: PathBuf,
    preserve_git: bool,
    validate_generated: bool,
    dry_run: bool,
}

/// Replication configuration
#[derive(Debug, Clone)]
pub struct ReplicationConfig {
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub preserve_git: bool,
    pub validate_generated: bool,
    pub dry_run: bool,
    pub include_tests: bool,
    pub include_documentation: bool,
    pub include_ci: bool,
}

/// Replication result with detailed information
#[derive(Debug)]
pub struct ReplicationResult {
    pub success: bool,
    pub blueprint_path: PathBuf,
    pub generated_files: Vec<PathBuf>,
    pub validation_results: Vec<ValidationResult>,
    pub execution_time: std::time::Duration,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

/// Validation result for generated code
#[derive(Debug)]
pub struct ValidationResult {
    pub file_path: PathBuf,
    pub validation_type: ValidationType,
    pub passed: bool,
    pub message: String,
}

/// Type of validation performed
#[derive(Debug)]
pub enum ValidationType {
    Syntax,
    Compilation,
    Tests,
    Linting,
    Formatting,
}

impl SystemReplicator {
    /// Create a new system replicator
    pub fn new(source_path: PathBuf, target_path: PathBuf) -> Self {
        Self {
            source_path,
            target_path,
            preserve_git: false,
            validate_generated: true,
            dry_run: false,
        }
    }

    /// Create a replicator with configuration
    pub fn with_config(config: ReplicationConfig) -> Self {
        Self {
            source_path: config.source_path,
            target_path: config.target_path,
            preserve_git: config.preserve_git,
            validate_generated: config.validate_generated,
            dry_run: config.dry_run,
        }
    }

    /// Set whether to preserve git history
    pub fn preserve_git(mut self, preserve: bool) -> Self {
        self.preserve_git = preserve;
        self
    }

    /// Set whether to validate generated code
    pub fn validate_generated(mut self, validate: bool) -> Self {
        self.validate_generated = validate;
        self
    }

    /// Set dry run mode (only show what would be done)
    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// Replicate the system from source to target
    pub async fn replicate(&self) -> Result<ReplicationResult> {
        let start_time = std::time::Instant::now();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        println!("🔄 Starting system replication...");
        println!("   Source: {:?}", self.source_path);
        println!("   Target: {:?}", self.target_path);

        if self.dry_run {
            println!("   Mode: DRY RUN (no files will be created)");
        }

        // Step 1: Extract blueprint from source
        println!("\n📋 Step 1: Extracting system blueprint...");
        let blueprint = self.extract_blueprint().await?;
        let blueprint_path = self.target_path.join("extracted_blueprint.toml");

        // Save blueprint for reference
        if !self.dry_run {
            if let Some(parent) = blueprint_path.parent() {
                fs::create_dir_all(parent).await?;
            }
            blueprint.save_to_file(&blueprint_path)?;
        }
        println!(
            "✅ Blueprint extracted with {} modules",
            blueprint.modules.len()
        );

        // Step 2: Validate blueprint completeness
        println!("\n🔍 Step 2: Validating blueprint...");
        let blueprint_warnings = blueprint.validate()?;
        for warning in &blueprint_warnings {
            warnings.push(format!("Blueprint validation: {}", warning));
            println!("⚠️  {}", warning);
        }

        // Step 3: Generate project from blueprint
        println!("\n🏗️  Step 3: Generating project structure...");
        let generated_files = self.generate_project(&blueprint).await?;
        println!("✅ Generated {} files", generated_files.len());

        // Step 4: Validate generated code
        let mut validation_results = Vec::new();
        if self.validate_generated && !self.dry_run {
            println!("\n✅ Step 4: Validating generated code...");
            validation_results = self.validate_generated_code(&generated_files).await?;

            let passed = validation_results.iter().filter(|r| r.passed).count();
            let total = validation_results.len();
            println!(
                "✅ Validation completed: {}/{} checks passed",
                passed, total
            );

            for result in &validation_results {
                if !result.passed {
                    errors.push(format!(
                        "Validation failed: {} - {}",
                        result.file_path.display(),
                        result.message
                    ));
                }
            }
        }

        // Step 5: Copy additional assets (if requested)
        if self.preserve_git && !self.dry_run {
            println!("\n📁 Step 5: Preserving git history...");
            self.preserve_git_history().await?;
        }

        let execution_time = start_time.elapsed();
        let success = errors.is_empty();

        if success {
            println!("\n🎉 System replication completed successfully!");
            println!("   Time taken: {:?}", execution_time);
            println!("   Files generated: {}", generated_files.len());
            if !warnings.is_empty() {
                println!("   Warnings: {}", warnings.len());
            }
        } else {
            println!("\n❌ System replication completed with errors:");
            for error in &errors {
                println!("   • {}", error);
            }
        }

        Ok(ReplicationResult {
            success,
            blueprint_path,
            generated_files,
            validation_results,
            execution_time,
            warnings,
            errors,
        })
    }

    /// Extract blueprint from source codebase
    async fn extract_blueprint(&self) -> Result<SystemBlueprint> {
        let mut extractor = BlueprintExtractor::new(self.source_path.clone())?;
        extractor.extract_blueprint().await
    }

    /// Generate project from blueprint
    async fn generate_project(&self, blueprint: &SystemBlueprint) -> Result<Vec<PathBuf>> {
        if self.dry_run {
            println!(
                "   [DRY RUN] Would generate project at: {:?}",
                self.target_path
            );
            return Ok(vec![self.target_path.join("would_be_generated")]);
        }

        let mut generator = BlueprintGenerator::new(self.target_path.clone())?;
        generator.generate_project(blueprint).await?;

        // Collect all generated files
        self.collect_generated_files().await
    }

    /// Collect all generated files for validation
    async fn collect_generated_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.collect_files_recursive(&self.target_path, &mut files)
            .await?;
        Ok(files)
    }

    /// Recursively collect files in directory
    fn collect_files_recursive<'a>(
        &'a self,
        dir: &'a Path,
        files: &'a mut Vec<PathBuf>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
            if !dir.exists() {
                return Ok(());
            }

            let mut entries = fs::read_dir(dir).await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();

                if path.is_dir() {
                    // Skip target directory and other build artifacts
                    let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if !matches!(dir_name, "target" | ".git" | "node_modules" | ".cargo") {
                        self.collect_files_recursive(&path, files).await?;
                    }
                } else {
                    files.push(path);
                }
            }

            Ok(())
        })
    }

    /// Validate generated code
    async fn validate_generated_code(&self, files: &[PathBuf]) -> Result<Vec<ValidationResult>> {
        let mut results = Vec::new();

        // Syntax validation for Rust files
        for file in files {
            if let Some(ext) = file.extension().and_then(|e| e.to_str()) {
                match ext {
                    "rs" => {
                        let syntax_result = self.validate_rust_syntax(file).await?;
                        results.push(syntax_result);
                    }
                    "toml" => {
                        let toml_result = self.validate_toml_syntax(file).await?;
                        results.push(toml_result);
                    }
                    "json" => {
                        let json_result = self.validate_json_syntax(file).await?;
                        results.push(json_result);
                    }
                    _ => {} // Skip other file types
                }
            }
        }

        // Compilation check
        if self.target_path.join("Cargo.toml").exists() {
            let compile_result = self.validate_compilation().await?;
            results.push(compile_result);
        }

        // Run tests if available
        if self.has_tests().await? {
            let test_result = self.validate_tests().await?;
            results.push(test_result);
        }

        Ok(results)
    }

    /// Validate Rust syntax
    async fn validate_rust_syntax(&self, file: &Path) -> Result<ValidationResult> {
        let content = fs::read_to_string(file).await?;

        // Use syn to parse Rust syntax
        match syn::parse_file(&content) {
            Ok(_) => Ok(ValidationResult {
                file_path: file.to_path_buf(),
                validation_type: ValidationType::Syntax,
                passed: true,
                message: "Rust syntax is valid".to_string(),
            }),
            Err(e) => Ok(ValidationResult {
                file_path: file.to_path_buf(),
                validation_type: ValidationType::Syntax,
                passed: false,
                message: format!("Syntax error: {}", e),
            }),
        }
    }

    /// Validate TOML syntax
    async fn validate_toml_syntax(&self, file: &Path) -> Result<ValidationResult> {
        let content = fs::read_to_string(file).await?;

        match toml::from_str::<toml::Value>(&content) {
            Ok(_) => Ok(ValidationResult {
                file_path: file.to_path_buf(),
                validation_type: ValidationType::Syntax,
                passed: true,
                message: "TOML syntax is valid".to_string(),
            }),
            Err(e) => Ok(ValidationResult {
                file_path: file.to_path_buf(),
                validation_type: ValidationType::Syntax,
                passed: false,
                message: format!("TOML syntax error: {}", e),
            }),
        }
    }

    /// Validate JSON syntax
    async fn validate_json_syntax(&self, file: &Path) -> Result<ValidationResult> {
        let content = fs::read_to_string(file).await?;

        match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(_) => Ok(ValidationResult {
                file_path: file.to_path_buf(),
                validation_type: ValidationType::Syntax,
                passed: true,
                message: "JSON syntax is valid".to_string(),
            }),
            Err(e) => Ok(ValidationResult {
                file_path: file.to_path_buf(),
                validation_type: ValidationType::Syntax,
                passed: false,
                message: format!("JSON syntax error: {}", e),
            }),
        }
    }

    /// Validate compilation
    async fn validate_compilation(&self) -> Result<ValidationResult> {
        let output = tokio::process::Command::new("cargo")
            .arg("check")
            .current_dir(&self.target_path)
            .output()
            .await?;

        let passed = output.status.success();
        let message = if passed {
            "Project compiles successfully".to_string()
        } else {
            format!(
                "Compilation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
        };

        Ok(ValidationResult {
            file_path: self.target_path.join("Cargo.toml"),
            validation_type: ValidationType::Compilation,
            passed,
            message,
        })
    }

    /// Check if project has tests
    async fn has_tests(&self) -> Result<bool> {
        let tests_dir = self.target_path.join("tests");
        let src_tests = self.target_path.join("src");

        let has_tests_dir = tests_dir.exists();
        let has_src_tests = if src_tests.exists() {
            self.directory_contains_tests(&src_tests).await?
        } else {
            false
        };

        Ok(has_tests_dir || has_src_tests)
    }

    /// Check if directory contains test files
    fn directory_contains_tests<'a>(
        &'a self,
        dir: &'a Path,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool>> + 'a>> {
        Box::pin(async move {
            let mut entries = fs::read_dir(dir).await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();

                if path.is_file() && path.extension() == Some(std::ffi::OsStr::new("rs")) {
                    let content = fs::read_to_string(&path).await?;
                    if content.contains("#[test]") || content.contains("#[tokio::test]") {
                        return Ok(true);
                    }
                } else if path.is_dir() {
                    if self.directory_contains_tests(&path).await? {
                        return Ok(true);
                    }
                }
            }

            Ok(false)
        })
    }

    /// Validate tests
    async fn validate_tests(&self) -> Result<ValidationResult> {
        let output = tokio::process::Command::new("cargo")
            .arg("test")
            .arg("--no-run") // Just check if tests compile
            .current_dir(&self.target_path)
            .output()
            .await?;

        let passed = output.status.success();
        let message = if passed {
            "Tests compile successfully".to_string()
        } else {
            format!(
                "Test compilation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
        };

        Ok(ValidationResult {
            file_path: self.target_path.join("tests"),
            validation_type: ValidationType::Tests,
            passed,
            message,
        })
    }

    /// Preserve git history from source
    async fn preserve_git_history(&self) -> Result<()> {
        let source_git = self.source_path.join(".git");
        let target_git = self.target_path.join(".git");

        if source_git.exists() {
            // Copy .git directory
            let output = tokio::process::Command::new("cp")
                .arg("-r")
                .arg(&source_git)
                .arg(&target_git)
                .output()
                .await?;

            if !output.status.success() {
                return Err(anyhow::anyhow!(
                    "Failed to copy .git directory: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }

            println!("✅ Git history preserved");
        } else {
            println!("ℹ️  No git history found in source");
        }

        Ok(())
    }

    /// Generate a self-replication report
    pub async fn generate_report(&self, result: &ReplicationResult) -> Result<()> {
        let report_path = self.target_path.join("REPLICATION_REPORT.md");
        let mut report = String::new();

        report.push_str("# System Replication Report\n\n");
        report.push_str(&format!(
            "**Generated**: {}\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));
        report.push_str(&format!("**Source**: `{}`\n", self.source_path.display()));
        report.push_str(&format!("**Target**: `{}`\n", self.target_path.display()));
        report.push_str(&format!("**Success**: {}\n", result.success));
        report.push_str(&format!(
            "**Execution Time**: {:?}\n\n",
            result.execution_time
        ));

        report.push_str("## Summary\n\n");
        report.push_str(&format!(
            "- **Files Generated**: {}\n",
            result.generated_files.len()
        ));
        report.push_str(&format!(
            "- **Validations**: {}\n",
            result.validation_results.len()
        ));
        report.push_str(&format!("- **Warnings**: {}\n", result.warnings.len()));
        report.push_str(&format!("- **Errors**: {}\n\n", result.errors.len()));

        if !result.warnings.is_empty() {
            report.push_str("## Warnings\n\n");
            for warning in &result.warnings {
                report.push_str(&format!("- {}\n", warning));
            }
            report.push_str("\n");
        }

        if !result.errors.is_empty() {
            report.push_str("## Errors\n\n");
            for error in &result.errors {
                report.push_str(&format!("- {}\n", error));
            }
            report.push_str("\n");
        }

        report.push_str("## Validation Results\n\n");
        for validation in &result.validation_results {
            let status = if validation.passed {
                "✅ PASS"
            } else {
                "❌ FAIL"
            };
            report.push_str(&format!(
                "- {} {:?} for `{}`: {}\n",
                status,
                validation.validation_type,
                validation.file_path.display(),
                validation.message
            ));
        }

        if !self.dry_run {
            fs::write(&report_path, report).await?;
            println!("📊 Replication report saved to: {:?}", report_path);
        } else {
            println!(
                "📊 [DRY RUN] Would save replication report to: {:?}",
                report_path
            );
        }

        Ok(())
    }
}

impl Default for ReplicationConfig {
    fn default() -> Self {
        Self {
            source_path: PathBuf::from("."),
            target_path: PathBuf::from("./replicated_system"),
            preserve_git: false,
            validate_generated: true,
            dry_run: false,
            include_tests: true,
            include_documentation: true,
            include_ci: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_system_replication() {
        let temp_source = tempdir().unwrap();
        let temp_target = tempdir().unwrap();

        // Create a minimal source structure
        let src_dir = temp_source.path().join("src");
        tokio::fs::create_dir_all(&src_dir).await.unwrap();

        let main_rs = src_dir.join("main.rs");
        tokio::fs::write(&main_rs, "fn main() { println!(\"Hello, world!\"); }")
            .await
            .unwrap();

        let cargo_toml = temp_source.path().join("Cargo.toml");
        tokio::fs::write(
            &cargo_toml,
            r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#,
        )
        .await
        .unwrap();

        let replicator = SystemReplicator::new(
            temp_source.path().to_path_buf(),
            temp_target.path().to_path_buf(),
        );

        let result = replicator.replicate().await.unwrap();

        // Basic assertions - we expect this to have some warnings/errors due to minimal setup
        // but the structure should be created
        assert!(!result.generated_files.is_empty());
        assert!(result.execution_time.as_millis() > 0);
    }

    #[test]
    fn test_replication_config() {
        let config = ReplicationConfig::default();
        assert_eq!(config.source_path, PathBuf::from("."));
        assert_eq!(config.target_path, PathBuf::from("./replicated_system"));
        assert!(!config.preserve_git);
        assert!(config.validate_generated);
        assert!(!config.dry_run);
    }
}
