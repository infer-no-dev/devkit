use devkit::blueprint::{SystemBlueprint, SystemMetadata, ArchitecturalDecisions, ModuleBlueprint, 
    DesignPatterns, ImplementationDetails, ConfigurationStrategy, 
    TestingStrategy, PerformanceOptimizations, SecurityPatterns, DeploymentStrategy};
use devkit::blueprint::evolution::{BlueprintVersion, BlueprintDiffAnalyzer, MigrationEngine, MigrationConfig, MigrationContext};
use anyhow::Result;
use std::path::PathBuf;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Blueprint Migration Engine Test");
    println!("==============================");
    println!();

    // Create test blueprints
    let original_blueprint = create_original_blueprint();
    let updated_blueprint = create_updated_blueprint();
    
    // Create working directories
    let temp_dir = std::env::temp_dir().join("blueprint_migration_test");
    tokio::fs::create_dir_all(&temp_dir).await?;
    println!("Working directory: {}", temp_dir.display());

    // Setup migration engine
    let config = MigrationConfig {
        working_directory: temp_dir.clone(),
        backup_directory: temp_dir.join(".backups"),
        script_directory: temp_dir.join(".scripts"),
        dry_run: false, // Set to true to see what would happen without executing
        auto_backup: true,
        validation_timeout: std::time::Duration::from_secs(60),
        parallel_execution: false,
        max_retries: 2,
    };

    let mut migration_engine = MigrationEngine::new(config);
    
    // Create migration context
    let context = MigrationContext {
        working_dir: temp_dir.clone(),
        blueprint_path: temp_dir.join("blueprint.json"),
        target_blueprint: updated_blueprint.clone(),
        source_blueprint: original_blueprint.clone(),
        migration_id: "test_migration_001".to_string(),
        environment: "development".to_string(),
        user_variables: HashMap::new(),
    };

    println!("Step 1: Analyzing blueprint differences...");
    
    // Analyze differences between blueprints
    let diff_analyzer = BlueprintDiffAnalyzer::new();
    let diff = diff_analyzer.analyze_diff(
        &original_blueprint,
        &updated_blueprint,
        BlueprintVersion::new(1, 0, 0),
        BlueprintVersion::new(2, 0, 0),
    )?;

    println!("Found {} changes:", diff.summary.total_changes);
    for (i, change) in diff.changes.iter().take(5).enumerate() {
        println!("  {}. {} - {} ({:?})", 
            i + 1, 
            change.change_type, 
            change.path,
            change.impact_level
        );
    }
    
    if diff.changes.len() > 5 {
        println!("  ... and {} more changes", diff.changes.len() - 5);
    }
    println!();

    println!("Step 2: Generating migration plan...");
    
    // Generate migration plan from diff
    let migration_plan = migration_engine.generate_migration_plan(&diff, &context).await?;
    
    println!("Generated {} migration steps:", migration_plan.len());
    for (i, step) in migration_plan.iter().enumerate() {
        println!("  {}. {} - {} (Est: {:?})", 
            i + 1, 
            step.step_id,
            step.description,
            step.estimated_duration.unwrap_or(std::time::Duration::from_secs(0))
        );
        
        if !step.dependencies.is_empty() {
            println!("     Dependencies: {:?}", step.dependencies);
        }
        
        if !step.validation_checks.is_empty() {
            println!("     Validation checks: {}", step.validation_checks.len());
        }
    }
    println!();

    println!("Step 3: Executing migration...");
    
    // Execute the migration
    let migration_result = migration_engine.execute_migration(migration_plan, &context).await?;
    
    println!("Migration Result:");
    println!("  Status: {:?}", migration_result.status);
    println!("  Migration ID: {}", migration_result.migration_id);
    println!("  Execution Time: {:?}", migration_result.execution_time);
    println!("  Steps Executed: {}", migration_result.executed_steps.len());
    println!("  Rollback Available: {}", migration_result.rollback_available);
    
    if !migration_result.warnings.is_empty() {
        println!("  Warnings:");
        for warning in &migration_result.warnings {
            println!("    - {}", warning);
        }
    }
    
    if !migration_result.artifacts.is_empty() {
        println!("  Artifacts Created:");
        for artifact in &migration_result.artifacts {
            println!("    - {:?}: {}", artifact.artifact_type, artifact.path.display());
        }
    }
    println!();

    println!("Step 4: Migration step details...");
    
    for (i, step) in migration_result.executed_steps.iter().enumerate() {
        println!("  Step {}: {} ({:?})", 
            i + 1, 
            step.description, 
            step.step_type
        );
        
        if let Some(result) = &step.execution_result {
            println!("    Success: {}", result.success);
            if !result.output.is_empty() {
                println!("    Output: {}", result.output.lines().next().unwrap_or(""));
            }
            if let Some(error) = &result.error_message {
                println!("    Error: {}", error);
            }
            println!("    Duration: {:?}", result.execution_time);
            
            if !result.validation_results.is_empty() {
                println!("    Validation Results:");
                for validation in &result.validation_results {
                    println!("      - {}: {} ({})", 
                        validation.check_name, 
                        if validation.passed { "PASS" } else { "FAIL" },
                        validation.severity
                    );
                }
            }
        }
        println!();
    }

    // Show created files
    println!("Step 5: Inspecting created files...");
    
    let backup_dir = temp_dir.join(".blueprint-backups");
    if backup_dir.exists() {
        println!("Backup files created:");
        if let Ok(entries) = std::fs::read_dir(&backup_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    println!("  - {}", entry.file_name().to_string_lossy());
                }
            }
        }
    }
    
    // Check if migration scripts were created
    let scripts = ["migration_architecture.sh", "migration_module.sh", "migration_config.sh"];
    println!("Migration scripts:");
    for script in &scripts {
        let script_path = temp_dir.join(script);
        if script_path.exists() {
            println!("  ✓ {} (created)", script);
            // Show first few lines of the script
            if let Ok(content) = std::fs::read_to_string(&script_path) {
                let lines: Vec<&str> = content.lines().take(5).collect();
                for line in lines {
                    println!("    {}", line);
                }
                if content.lines().count() > 5 {
                    println!("    ...");
                }
            }
        } else {
            println!("  ✗ {} (not created)", script);
        }
        println!();
    }

    println!("Migration engine test completed successfully!");
    println!("Temporary files are in: {}", temp_dir.display());
    
    Ok(())
}

fn create_original_blueprint() -> SystemBlueprint {
    SystemBlueprint {
        metadata: SystemMetadata {
            name: "Test System".to_string(),
            version: "1.0.0".to_string(),
            description: "Original test system".to_string(),
            architecture_paradigm: "Monolithic".to_string(),
            primary_language: "Rust".to_string(),
            creation_timestamp: chrono::Utc::now(),
            generator_version: "1.0.0".to_string(),
        },
        architecture: ArchitecturalDecisions::default(),
        modules: vec![
            ModuleBlueprint {
                name: "core".to_string(),
                purpose: "Core functionality".to_string(),
                dependencies: vec![],
                public_interface: vec![],
                internal_structure: Default::default(),
                testing_strategy: Default::default(),
                performance_characteristics: Default::default(),
            }
        ],
        patterns: DesignPatterns::default(),
        implementation: ImplementationDetails::default(),
        configuration: ConfigurationStrategy::default(),
        testing: TestingStrategy::default(),
        performance: PerformanceOptimizations::default(),
        security: SecurityPatterns::default(),
        deployment: DeploymentStrategy::default(),
    }
}

fn create_updated_blueprint() -> SystemBlueprint {
    SystemBlueprint {
        metadata: SystemMetadata {
            name: "Test System".to_string(),
            version: "2.0.0".to_string(),
            description: "Updated test system with microservices".to_string(),
            architecture_paradigm: "Microservices".to_string(), // Changed
            primary_language: "Rust".to_string(),
            creation_timestamp: chrono::Utc::now(),
            generator_version: "2.0.0".to_string(), // Changed
        },
        architecture: ArchitecturalDecisions::default(),
        modules: vec![
            ModuleBlueprint {
                name: "core".to_string(),
                purpose: "Enhanced core functionality".to_string(), // Changed
                dependencies: vec![],
                public_interface: vec![],
                internal_structure: Default::default(),
                testing_strategy: Default::default(),
                performance_characteristics: Default::default(),
            },
            // New module added
            ModuleBlueprint {
                name: "service_mesh".to_string(),
                purpose: "Service mesh for microservices".to_string(),
                dependencies: vec![],
                public_interface: vec![],
                internal_structure: Default::default(),
                testing_strategy: Default::default(),
                performance_characteristics: Default::default(),
            }
        ],
        patterns: DesignPatterns::default(),
        implementation: ImplementationDetails::default(),
        configuration: ConfigurationStrategy::default(),
        testing: TestingStrategy::default(),
        performance: PerformanceOptimizations::default(),
        security: SecurityPatterns::default(),
        deployment: DeploymentStrategy::default(),
    }
}