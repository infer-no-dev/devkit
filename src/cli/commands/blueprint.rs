//! Blueprint command implementations
//!
//! This module provides CLI commands for system blueprint operations including
//! extraction, generation, replication, validation, and comparison.

use crate::blueprint::{
    extractor::BlueprintExtractor,
    generator::BlueprintGenerator,
    replicator::{ReplicationConfig, SystemReplicator},
    SystemBlueprint,
};
use crate::cli::{BlueprintCommands, CliRunner};
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Run blueprint commands
pub async fn run(
    cli: &mut CliRunner,
    command: BlueprintCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        BlueprintCommands::Extract {
            source,
            output,
            detailed,
        } => extract_blueprint(cli, source, output, detailed).await,
        BlueprintCommands::Generate {
            blueprint,
            output,
            preview,
        } => generate_from_blueprint(cli, blueprint, output, preview).await,
        BlueprintCommands::Replicate {
            target,
            preserve_git,
            skip_validation,
            dry_run,
        } => replicate_system(cli, target, preserve_git, !skip_validation, dry_run).await,
        BlueprintCommands::Validate { blueprint } => validate_blueprint(cli, blueprint).await,
        BlueprintCommands::Info {
            blueprint,
            detailed,
        } => show_blueprint_info(cli, blueprint, detailed).await,
        BlueprintCommands::Compare {
            blueprint1,
            blueprint2,
        } => compare_blueprints(cli, blueprint1, blueprint2).await,
        BlueprintCommands::Evolution(evolution_cmd) => {
            super::evolution::handle_evolution_command(evolution_cmd)
                .await
                .map_err(|e| e.into())
        }
    }
}

/// Extract system blueprint from codebase
async fn extract_blueprint(
    cli: &mut CliRunner,
    source: PathBuf,
    output: PathBuf,
    detailed: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    cli.print_info(&format!("Extracting blueprint from: {:?}", source));

    if detailed {
        cli.print_verbose("Running detailed analysis...");
    }

    let mut extractor =
        BlueprintExtractor::new(source.clone()).context("Failed to create blueprint extractor")?;

    cli.print_verbose("Analyzing codebase structure...");
    let blueprint = extractor
        .extract_blueprint()
        .await
        .context("Failed to extract blueprint")?;

    cli.print_verbose("Validating extracted blueprint...");
    let warnings = blueprint
        .validate()
        .context("Failed to validate blueprint")?;

    if !warnings.is_empty() {
        cli.print_warning("Blueprint validation warnings:");
        for warning in &warnings {
            cli.print_output(&format!("  • {}", warning), None);
        }
    }

    cli.print_verbose(&format!("Saving blueprint to: {:?}", output));
    blueprint
        .save_to_file(&output)
        .context("Failed to save blueprint file")?;

    cli.print_success(&format!("Blueprint extracted successfully!"));
    cli.print_output(&format!("  • Source: {:?}", source), None);
    cli.print_output(&format!("  • Output: {:?}", output), None);
    cli.print_output(&format!("  • Modules: {}", blueprint.modules.len()), None);
    cli.print_output(
        &format!(
            "  • Dependencies: {}",
            blueprint.implementation.third_party_dependencies.len()
        ),
        None,
    );

    if !warnings.is_empty() {
        cli.print_output(&format!("  • Warnings: {}", warnings.len()), None);
    }

    if detailed {
        show_blueprint_summary(cli, &blueprint)?;
    }

    Ok(())
}

/// Generate project from blueprint
async fn generate_from_blueprint(
    cli: &mut CliRunner,
    blueprint_path: PathBuf,
    output: PathBuf,
    preview: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    cli.print_info(&format!("Loading blueprint from: {:?}", blueprint_path));

    let blueprint = SystemBlueprint::load_from_file(&blueprint_path)
        .context("Failed to load blueprint file")?;

    cli.print_verbose(&format!(
        "Blueprint: {} v{}",
        blueprint.metadata.name, blueprint.metadata.version
    ));

    if preview {
        cli.print_info("Preview mode - showing what would be generated:");
        show_generation_preview(cli, &blueprint, &output)?;
        return Ok(());
    }

    cli.print_info(&format!("Generating project at: {:?}", output));

    let mut generator =
        BlueprintGenerator::new(output.clone()).context("Failed to create blueprint generator")?;

    generator
        .generate_project(&blueprint)
        .await
        .context("Failed to generate project")?;

    cli.print_success("Project generated successfully!");
    cli.print_output(&format!("  • Location: {:?}", output), None);
    cli.print_output(&format!("  • System: {}", blueprint.metadata.name), None);
    cli.print_output(&format!("  • Modules: {}", blueprint.modules.len()), None);

    // Show next steps
    cli.print_info("Next steps:");
    cli.print_command(&format!("cd {:?}", output));
    cli.print_command("cargo build");
    cli.print_command("cargo test");

    Ok(())
}

/// Replicate the current system
async fn replicate_system(
    cli: &mut CliRunner,
    target: PathBuf,
    preserve_git: bool,
    validate_generated: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = std::env::current_dir().context("Failed to get current directory")?;

    cli.print_info("🔄 Starting system self-replication...");
    cli.print_output(&format!("  Source: {:?}", source), None);
    cli.print_output(&format!("  Target: {:?}", target), None);

    if dry_run {
        cli.print_warning("DRY RUN MODE - No files will be created");
    }

    let config = ReplicationConfig {
        source_path: source,
        target_path: target.clone(),
        preserve_git,
        validate_generated,
        dry_run,
        include_tests: true,
        include_documentation: true,
        include_ci: true,
    };

    let replicator = SystemReplicator::with_config(config);

    let result = replicator
        .replicate()
        .await
        .context("System replication failed")?;

    if result.success {
        cli.print_success("🎉 System replication completed successfully!");

        cli.print_info("Replication Summary:");
        cli.print_output(
            &format!("  • Files generated: {}", result.generated_files.len()),
            None,
        );
        cli.print_output(
            &format!("  • Execution time: {:?}", result.execution_time),
            None,
        );

        if !result.warnings.is_empty() {
            cli.print_output(&format!("  • Warnings: {}", result.warnings.len()), None);
        }

        if result.validation_results.len() > 0 {
            let passed = result
                .validation_results
                .iter()
                .filter(|r| r.passed)
                .count();
            cli.print_output(
                &format!(
                    "  • Validations: {}/{} passed",
                    passed,
                    result.validation_results.len()
                ),
                None,
            );
        }

        // Generate detailed report
        replicator
            .generate_report(&result)
            .await
            .context("Failed to generate replication report")?;

        if !dry_run {
            cli.print_info("Next steps:");
            cli.print_command(&format!("cd {:?}", target));
            cli.print_command("cargo build --release");
            cli.print_command("cargo test");

            cli.print_info("To test self-replication capability:");
            cli.print_command("cargo run -- blueprint replicate --target ./replicated_again");
        }
    } else {
        cli.print_error("❌ System replication failed");

        for error in &result.errors {
            cli.print_error(&format!("  • {}", error));
        }

        return Err("System replication failed".into());
    }

    Ok(())
}

/// Validate blueprint file
async fn validate_blueprint(
    cli: &mut CliRunner,
    blueprint_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    cli.print_info(&format!("Validating blueprint: {:?}", blueprint_path));

    let blueprint = SystemBlueprint::load_from_file(&blueprint_path)
        .context("Failed to load blueprint file")?;

    let warnings = blueprint
        .validate()
        .context("Failed to validate blueprint")?;

    if warnings.is_empty() {
        cli.print_success("✅ Blueprint is valid!");
    } else {
        cli.print_warning(&format!("Blueprint has {} warnings:", warnings.len()));
        for warning in &warnings {
            cli.print_output(&format!("  • {}", warning), None);
        }
    }

    // Show basic stats
    cli.print_info("Blueprint Statistics:");
    cli.print_output(&format!("  • Name: {}", blueprint.metadata.name), None);
    cli.print_output(
        &format!("  • Version: {}", blueprint.metadata.version),
        None,
    );
    cli.print_output(&format!("  • Modules: {}", blueprint.modules.len()), None);
    cli.print_output(
        &format!(
            "  • Dependencies: {}",
            blueprint.implementation.third_party_dependencies.len()
        ),
        None,
    );
    cli.print_output(
        &format!("  • Architecture: {}", blueprint.architecture.system_type),
        None,
    );
    cli.print_output(
        &format!("  • Language: {}", blueprint.metadata.primary_language),
        None,
    );

    Ok(())
}

/// Show blueprint information
async fn show_blueprint_info(
    cli: &mut CliRunner,
    blueprint_path: PathBuf,
    detailed: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    cli.print_info(&format!("Loading blueprint: {:?}", blueprint_path));

    let blueprint = SystemBlueprint::load_from_file(&blueprint_path)
        .context("Failed to load blueprint file")?;

    show_blueprint_summary(cli, &blueprint)?;

    if detailed {
        show_detailed_blueprint_info(cli, &blueprint)?;
    }

    Ok(())
}

/// Compare two blueprints
async fn compare_blueprints(
    cli: &mut CliRunner,
    blueprint1_path: PathBuf,
    blueprint2_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    cli.print_info("Loading blueprints for comparison...");

    let blueprint1 = SystemBlueprint::load_from_file(&blueprint1_path)
        .context("Failed to load first blueprint")?;

    let blueprint2 = SystemBlueprint::load_from_file(&blueprint2_path)
        .context("Failed to load second blueprint")?;

    cli.print_info("Blueprint Comparison:");

    // Compare basic metadata
    cli.print_output("\n📋 Metadata Comparison:", None);
    cli.print_output(
        &format!(
            "  Name:         {} vs {}",
            blueprint1.metadata.name, blueprint2.metadata.name
        ),
        None,
    );
    cli.print_output(
        &format!(
            "  Version:      {} vs {}",
            blueprint1.metadata.version, blueprint2.metadata.version
        ),
        None,
    );
    cli.print_output(
        &format!(
            "  Architecture: {} vs {}",
            blueprint1.architecture.system_type, blueprint2.architecture.system_type
        ),
        None,
    );

    // Compare modules
    cli.print_output("\n🧩 Module Comparison:", None);
    cli.print_output(
        &format!(
            "  Module count: {} vs {}",
            blueprint1.modules.len(),
            blueprint2.modules.len()
        ),
        None,
    );

    let modules1: std::collections::HashSet<_> =
        blueprint1.modules.iter().map(|m| &m.name).collect();
    let modules2: std::collections::HashSet<_> =
        blueprint2.modules.iter().map(|m| &m.name).collect();

    let only_in_1: Vec<_> = modules1.difference(&modules2).collect();
    let only_in_2: Vec<_> = modules2.difference(&modules1).collect();
    let common: Vec<_> = modules1.intersection(&modules2).collect();

    cli.print_output(&format!("  Common modules: {}", common.len()), None);
    if !only_in_1.is_empty() {
        cli.print_output(&format!("  Only in first: {:?}", only_in_1), None);
    }
    if !only_in_2.is_empty() {
        cli.print_output(&format!("  Only in second: {:?}", only_in_2), None);
    }

    // Compare dependencies
    cli.print_output("\n📦 Dependency Comparison:", None);
    let deps1_count = blueprint1.implementation.third_party_dependencies.len();
    let deps2_count = blueprint2.implementation.third_party_dependencies.len();
    cli.print_output(
        &format!("  Dependency count: {} vs {}", deps1_count, deps2_count),
        None,
    );

    // Compare patterns
    cli.print_output("\n🏗️  Pattern Comparison:", None);
    let patterns1_count = blueprint1.patterns.architectural_patterns.len()
        + blueprint1.patterns.behavioral_patterns.len()
        + blueprint1.patterns.structural_patterns.len();
    let patterns2_count = blueprint2.patterns.architectural_patterns.len()
        + blueprint2.patterns.behavioral_patterns.len()
        + blueprint2.patterns.structural_patterns.len();
    cli.print_output(
        &format!(
            "  Pattern count: {} vs {}",
            patterns1_count, patterns2_count
        ),
        None,
    );

    Ok(())
}

/// Show blueprint summary
fn show_blueprint_summary(
    cli: &CliRunner,
    blueprint: &SystemBlueprint,
) -> Result<(), Box<dyn std::error::Error>> {
    cli.print_output("\n📋 Blueprint Summary:", None);
    cli.print_output(&format!("  Name: {}", blueprint.metadata.name), None);
    cli.print_output(&format!("  Version: {}", blueprint.metadata.version), None);
    cli.print_output(
        &format!("  Description: {}", blueprint.metadata.description),
        None,
    );
    cli.print_output(
        &format!("  Architecture: {}", blueprint.architecture.system_type),
        None,
    );
    cli.print_output(
        &format!("  Language: {}", blueprint.metadata.primary_language),
        None,
    );
    cli.print_output(
        &format!(
            "  Created: {}",
            blueprint
                .metadata
                .creation_timestamp
                .format("%Y-%m-%d %H:%M:%S UTC")
        ),
        None,
    );

    cli.print_output("\n🧩 Modules:", None);
    for module in &blueprint.modules {
        cli.print_output(&format!("  • {} - {}", module.name, module.purpose), None);
    }

    cli.print_output(&format!("\n📊 Statistics:"), None);
    cli.print_output(&format!("  • Modules: {}", blueprint.modules.len()), None);
    cli.print_output(
        &format!(
            "  • Dependencies: {}",
            blueprint.implementation.third_party_dependencies.len()
        ),
        None,
    );
    let total_patterns = blueprint.patterns.architectural_patterns.len()
        + blueprint.patterns.behavioral_patterns.len()
        + blueprint.patterns.structural_patterns.len();
    cli.print_output(&format!("  • Design patterns: {}", total_patterns), None);

    Ok(())
}

/// Show detailed blueprint information
fn show_detailed_blueprint_info(
    cli: &CliRunner,
    blueprint: &SystemBlueprint,
) -> Result<(), Box<dyn std::error::Error>> {
    // Architecture details
    cli.print_output("\n🏗️  Architecture Details:", None);
    cli.print_output(
        &format!("  System type: {}", blueprint.architecture.system_type),
        None,
    );
    cli.print_output(
        &format!(
            "  Concurrency model: {}",
            blueprint.architecture.concurrency_model.primary_pattern
        ),
        None,
    );
    cli.print_output(
        &format!(
            "  Data flow: {}",
            blueprint.architecture.data_flow.primary_pattern
        ),
        None,
    );
    cli.print_output(
        &format!(
            "  Error handling: {}",
            blueprint.architecture.error_handling.propagation_strategy
        ),
        None,
    );

    // Key decisions
    if !blueprint.architecture.key_decisions.is_empty() {
        cli.print_output("\n🎯 Key Architectural Decisions:", None);
        for decision in &blueprint.architecture.key_decisions {
            cli.print_output(
                &format!("  • {}: {}", decision.decision, decision.reasoning),
                None,
            );
        }
    }

    // Dependencies
    if !blueprint.implementation.third_party_dependencies.is_empty() {
        cli.print_output("\n📦 Dependencies:", None);
        for dep in &blueprint.implementation.third_party_dependencies {
            cli.print_output(
                &format!("  • {} ({}): {}", dep.crate_name, dep.version, dep.purpose),
                None,
            );
        }
    }

    // Design patterns
    if !blueprint.patterns.architectural_patterns.is_empty() {
        cli.print_output("\n🎨 Architectural Patterns:", None);
        for pattern in &blueprint.patterns.architectural_patterns {
            cli.print_output(
                &format!("  • {}: {}", pattern.pattern_name, pattern.usage_context),
                None,
            );
        }
    }

    Ok(())
}

/// Show generation preview
fn show_generation_preview(
    cli: &CliRunner,
    blueprint: &SystemBlueprint,
    output: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    cli.print_output("\n🏗️  Generation Preview:", None);
    cli.print_output(&format!("  Target: {:?}", output), None);
    cli.print_output(
        &format!(
            "  Project: {} v{}",
            blueprint.metadata.name, blueprint.metadata.version
        ),
        None,
    );

    cli.print_output("\n📁 Directory Structure:", None);
    cli.print_output("  .", None);
    cli.print_output("  ├── Cargo.toml", None);
    cli.print_output("  ├── README.md", None);
    cli.print_output("  ├── WARP.md", None);
    cli.print_output("  ├── .agentic-config.toml", None);
    cli.print_output("  ├── src/", None);
    cli.print_output("  │   ├── main.rs", None);
    cli.print_output("  │   ├── lib.rs", None);

    for module in &blueprint.modules {
        cli.print_output(&format!("  │   ├── {}/", module.name), None);
        cli.print_output("  │   │   └── mod.rs", None);
    }

    cli.print_output("  ├── tests/", None);
    cli.print_output("  ├── benches/", None);
    cli.print_output("  ├── examples/", None);
    cli.print_output("  ├── docs/", None);
    cli.print_output("  └── .github/", None);
    cli.print_output("      └── workflows/", None);
    cli.print_output("          └── ci.yml", None);

    cli.print_output(&format!("\n📊 Generation Stats:"), None);
    cli.print_output(
        &format!("  • Estimated files: ~{}", 15 + blueprint.modules.len() * 3),
        None,
    );
    cli.print_output(
        &format!("  • Modules to generate: {}", blueprint.modules.len()),
        None,
    );
    cli.print_output(
        &format!("  • Tests to generate: {}", blueprint.modules.len()),
        None,
    );

    Ok(())
}
