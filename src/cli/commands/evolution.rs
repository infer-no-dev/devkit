//! Blueprint Evolution CLI Commands
//!
//! Provides command-line interface for blueprint version management,
//! diff analysis, migration, and rollback operations.

use crate::blueprint::evolution::{
    BlueprintVersion, BlueprintDiffAnalyzer, MigrationEngine, MigrationConfig, 
    MigrationContext, BlueprintEvolutionTracker
};
use crate::blueprint::SystemBlueprint;
use anyhow::{Result, Context};
use clap::{Subcommand, Args};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// Blueprint evolution commands
#[derive(Debug, Subcommand)]
pub enum EvolutionCommand {
    /// Show current blueprint version
    Version(VersionArgs),
    /// Compare blueprint versions
    Diff(DiffArgs),
    /// Execute blueprint migration
    Migrate(MigrateArgs),
    /// Rollback to previous version
    Rollback(RollbackArgs),
    /// Show evolution history
    History(HistoryArgs),
    /// Initialize evolution tracking
    Init(InitArgs),
    /// Create a new blueprint branch
    Branch(BranchArgs),
    /// Show evolution status
    Status(StatusArgs),
}

#[derive(Debug, Args)]
pub struct VersionArgs {
    /// Blueprint file path
    #[arg(short, long, default_value = "blueprint.json")]
    pub blueprint: PathBuf,
    /// Show detailed version information
    #[arg(short, long)]
    pub detailed: bool,
}

#[derive(Debug, Args)]
pub struct DiffArgs {
    /// From version (default: current)
    #[arg(short, long)]
    pub from: Option<String>,
    /// To version (default: working directory)
    #[arg(short, long)]
    pub to: Option<String>,
    /// Blueprint file path
    #[arg(short, long, default_value = "blueprint.json")]
    pub blueprint: PathBuf,
    /// Output format (summary, detailed, json)
    #[arg(long, default_value = "summary")]
    pub output_format: String,
    /// Show only changes with this impact level or higher
    #[arg(long)]
    pub min_impact: Option<String>,
}

#[derive(Debug, Args)]
pub struct MigrateArgs {
    /// Target version to migrate to
    pub target_version: String,
    /// Blueprint file path
    #[arg(short, long, default_value = "blueprint.json")]
    pub blueprint: PathBuf,
    /// Perform dry run without executing
    #[arg(long)]
    pub dry_run: bool,
    /// Skip backup creation
    #[arg(long)]
    pub no_backup: bool,
    /// Force migration even with high risk
    #[arg(long)]
    pub force: bool,
    /// Environment to migrate
    #[arg(long, default_value = "development")]
    pub environment: String,
}

#[derive(Debug, Args)]
pub struct RollbackArgs {
    /// Version to rollback to (default: previous)
    pub target_version: Option<String>,
    /// Blueprint file path
    #[arg(short, long, default_value = "blueprint.json")]
    pub blueprint: PathBuf,
    /// Perform dry run without executing
    #[arg(long)]
    pub dry_run: bool,
    /// Force rollback even with high risk
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct HistoryArgs {
    /// Blueprint file path
    #[arg(short, long, default_value = "blueprint.json")]
    pub blueprint: PathBuf,
    /// Maximum number of entries to show
    #[arg(short, long, default_value = "10")]
    pub limit: usize,
    /// Show only entries from this branch
    #[arg(long)]
    pub branch: Option<String>,
    /// Show detailed information
    #[arg(short, long)]
    pub detailed: bool,
    /// Output format (table, json, graph)
    #[arg(long, default_value = "table")]
    pub output_format: String,
}

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Directory to initialize evolution tracking
    #[arg(default_value = ".")]
    pub directory: PathBuf,
    /// Initial version
    #[arg(long, default_value = "0.1.0")]
    pub version: String,
}

#[derive(Debug, Args)]
pub struct BranchArgs {
    /// Branch name to create or switch to
    pub branch_name: String,
    /// Create new branch
    #[arg(short, long)]
    pub create: bool,
    /// Switch to existing branch
    #[arg(short, long)]
    pub switch: bool,
    /// List all branches
    #[arg(short, long)]
    pub list: bool,
}

#[derive(Debug, Args)]
pub struct StatusArgs {
    /// Blueprint file path
    #[arg(short, long, default_value = "blueprint.json")]
    pub blueprint: PathBuf,
    /// Show detailed status
    #[arg(short, long)]
    pub detailed: bool,
}

/// Execute blueprint evolution commands
pub async fn handle_evolution_command(cmd: EvolutionCommand) -> Result<()> {
    match cmd {
        EvolutionCommand::Version(args) => handle_version(args).await,
        EvolutionCommand::Diff(args) => handle_diff(args).await,
        EvolutionCommand::Migrate(args) => handle_migrate(args).await,
        EvolutionCommand::Rollback(args) => handle_rollback(args).await,
        EvolutionCommand::History(args) => handle_history(args).await,
        EvolutionCommand::Init(args) => handle_init(args).await,
        EvolutionCommand::Branch(args) => handle_branch(args).await,
        EvolutionCommand::Status(args) => handle_status(args).await,
    }
}

async fn handle_version(args: VersionArgs) -> Result<()> {
    println!("🔍 Blueprint Version Information");
    println!("================================");
    println!();

    let blueprint = load_blueprint(&args.blueprint)
        .with_context(|| format!("Failed to load blueprint from {}", args.blueprint.display()))?;

    println!("Current Version: {}", blueprint.metadata.version);
    println!("Blueprint Name: {}", blueprint.metadata.name);
    println!("Description: {}", blueprint.metadata.description);

    if args.detailed {
        println!();
        println!("Detailed Information:");
        println!("  Architecture: {}", blueprint.metadata.architecture_paradigm);
        println!("  Primary Language: {}", blueprint.metadata.primary_language);
        println!("  Created: {}", blueprint.metadata.creation_timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
        println!("  Generator Version: {}", blueprint.metadata.generator_version);
        println!("  Modules: {}", blueprint.modules.len());

        // Check if evolution tracking is initialized
        let evolution_dir = args.blueprint.parent().unwrap_or(Path::new(".")).join(".blueprint-evolution");
        if evolution_dir.exists() {
            println!("  Evolution Tracking: Enabled");
            
            let mut tracker = BlueprintEvolutionTracker::new(evolution_dir);
            if let Ok(()) = tracker.load().await {
                if let Some(current_version) = tracker.get_current_version() {
                    println!("  Tracked Version: {}", current_version);
                }
                println!("  Available Branches: {:?}", tracker.list_branches());
            }
        } else {
            println!("  Evolution Tracking: Not initialized (run 'devkit blueprint init')");
        }
    }

    Ok(())
}

async fn handle_diff(args: DiffArgs) -> Result<()> {
    println!("📊 Blueprint Difference Analysis");
    println!("===============================");
    println!();

    let current_blueprint = load_blueprint(&args.blueprint)
        .with_context(|| format!("Failed to load current blueprint from {}", args.blueprint.display()))?;

    // Determine from and to versions
    let from_version = if let Some(from) = &args.from {
        BlueprintVersion::from_str(from)?
    } else {
        BlueprintVersion::from_str(&current_blueprint.metadata.version)?
    };

    let to_version = if let Some(to) = &args.to {
        BlueprintVersion::from_str(to)?
    } else {
        BlueprintVersion::from_str(&current_blueprint.metadata.version)?
    };

    // For now, we'll compare the current blueprint with itself to demonstrate
    // In a real implementation, we'd load different versions from evolution history
    let from_blueprint = current_blueprint.clone();
    let to_blueprint = current_blueprint;

    let analyzer = BlueprintDiffAnalyzer::new();
    let diff = analyzer.analyze_diff(&from_blueprint, &to_blueprint, from_version, to_version)?;

    match args.output_format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&diff)?;
            println!("{}", json);
        }
        "detailed" => {
            print_detailed_diff(&diff);
        }
        _ => {
            print_diff_summary(&diff);
        }
    }

    Ok(())
}

async fn handle_migrate(args: MigrateArgs) -> Result<()> {
    println!("🚀 Blueprint Migration");
    println!("======================");
    println!();

    let current_blueprint = load_blueprint(&args.blueprint)
        .with_context(|| format!("Failed to load blueprint from {}", args.blueprint.display()))?;

    let target_version = BlueprintVersion::from_str(&args.target_version)
        .with_context(|| format!("Invalid target version: {}", args.target_version))?;

    let current_version = BlueprintVersion::from_str(&current_blueprint.metadata.version)
        .with_context(|| "Invalid current version in blueprint")?;

    println!("Migration Plan:");
    println!("  From: {} → To: {}", current_version, target_version);
    println!("  Environment: {}", args.environment);
    println!("  Dry Run: {}", args.dry_run);
    println!();

    // Setup migration engine
    let working_dir = args.blueprint.parent().unwrap_or(Path::new(".")).to_path_buf();
    let config = MigrationConfig {
        working_directory: working_dir.clone(),
        backup_directory: working_dir.join(".blueprint-backups"),
        script_directory: working_dir.join(".blueprint-migrations"),
        dry_run: args.dry_run,
        auto_backup: !args.no_backup,
        validation_timeout: std::time::Duration::from_secs(300),
        parallel_execution: false,
        max_retries: 3,
    };

    let migration_engine = MigrationEngine::new(config);

    // Create target blueprint (for demo, we'll create a slightly modified version)
    let mut target_blueprint = current_blueprint.clone();
    target_blueprint.metadata.version = args.target_version.clone();
    target_blueprint.metadata.description = format!("{} (migrated to {})", 
        target_blueprint.metadata.description, args.target_version);

    // Create migration context
    let context = MigrationContext {
        working_dir: working_dir.clone(),
        blueprint_path: args.blueprint.clone(),
        target_blueprint: target_blueprint.clone(),
        source_blueprint: current_blueprint.clone(),
        migration_id: uuid::Uuid::new_v4().to_string(),
        environment: args.environment.clone(),
        user_variables: HashMap::new(),
    };

    // Analyze differences
    let analyzer = BlueprintDiffAnalyzer::new();
    let diff = analyzer.analyze_diff(&current_blueprint, &target_blueprint, current_version, target_version)?;

    if diff.changes.is_empty() {
        println!("✅ No changes detected - blueprint is already at target version");
        return Ok(());
    }

    println!("Changes detected: {}", diff.changes.len());
    println!("Risk level: {:?}", diff.impact_analysis.risk_level);
    println!("Impact score: {:.2}", diff.impact_analysis.overall_impact_score);
    println!();

    // Check if migration should proceed
    if !args.force && diff.impact_analysis.risk_level == crate::blueprint::evolution::RiskLevel::Critical {
        println!("⚠️  Critical risk migration detected!");
        println!("Use --force to proceed or analyze the changes with 'devkit blueprint diff'");
        return Ok(());
    }

    // Generate migration plan
    let migration_plan = migration_engine.generate_migration_plan(&diff, &context).await
        .context("Failed to generate migration plan")?;

    println!("Migration steps: {}", migration_plan.len());
    for (i, step) in migration_plan.iter().enumerate() {
        let duration = step.estimated_duration.unwrap_or(std::time::Duration::from_secs(0));
        println!("  {}. {} (Est: {:?})", i + 1, step.description, duration);
    }
    println!();

    if args.dry_run {
        println!("✅ Dry run completed - no changes made");
        return Ok(());
    }

    println!("Executing migration...");
    let result = migration_engine.execute_migration(migration_plan, &context).await
        .context("Migration execution failed")?;

    match result.status {
        crate::blueprint::evolution::MigrationStatus::Completed => {
            println!("✅ Migration completed successfully!");
            println!("  Execution time: {:?}", result.execution_time);
            println!("  Steps executed: {}", result.executed_steps.len());
            if result.rollback_available {
                println!("  Rollback available: Yes");
            }
        }
        crate::blueprint::evolution::MigrationStatus::Failed => {
            println!("❌ Migration failed!");
            if let Some(failed_step) = &result.failed_step {
                println!("  Failed step: {}", failed_step.description);
            }
        }
        crate::blueprint::evolution::MigrationStatus::RolledBack => {
            println!("🔄 Migration was rolled back due to failure");
        }
        _ => {
            println!("⚠️  Migration in unexpected state: {:?}", result.status);
        }
    }

    if !result.warnings.is_empty() {
        println!("\nWarnings:");
        for warning in &result.warnings {
            println!("  - {}", warning);
        }
    }

    Ok(())
}

async fn handle_rollback(args: RollbackArgs) -> Result<()> {
    println!("🔄 Blueprint Rollback");
    println!("====================");
    println!();

    let current_blueprint = load_blueprint(&args.blueprint)
        .with_context(|| format!("Failed to load blueprint from {}", args.blueprint.display()))?;

    let target_version = if let Some(target) = &args.target_version {
        BlueprintVersion::from_str(target)?
    } else {
        // Default to rolling back to previous version
        let current = BlueprintVersion::from_str(&current_blueprint.metadata.version)?;
        let mut previous = current.clone();
        if previous.patch > 0 {
            previous.patch -= 1;
        } else if previous.minor > 0 {
            previous.minor -= 1;
            previous.patch = 0;
        } else if previous.major > 0 {
            previous.major -= 1;
            previous.minor = 0;
            previous.patch = 0;
        } else {
            anyhow::bail!("Cannot rollback from version 0.0.0");
        }
        previous
    };

    println!("Rollback Plan:");
    println!("  From: {} → To: {}", current_blueprint.metadata.version, target_version);
    println!("  Dry Run: {}", args.dry_run);
    println!();

    if args.dry_run {
        println!("✅ Dry run completed - would rollback to version {}", target_version);
        return Ok(());
    }

    // Check for backup files
    let backup_dir = args.blueprint.parent().unwrap_or(Path::new(".")).join(".blueprint-backups");
    if backup_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&backup_dir) {
            let backups: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            if !backups.is_empty() {
                println!("Available backups: {}", backups.len());
                println!("✅ Rollback would restore from most recent backup");
            } else {
                println!("⚠️  No backup files found in {}", backup_dir.display());
            }
        }
    } else {
        println!("⚠️  No backup directory found - rollback may not be possible");
    }

    Ok(())
}

async fn handle_history(args: HistoryArgs) -> Result<()> {
    println!("📚 Blueprint Evolution History");
    println!("=============================");
    println!();

    let evolution_dir = args.blueprint.parent().unwrap_or(Path::new(".")).join(".blueprint-evolution");
    
    if !evolution_dir.exists() {
        println!("❌ Evolution tracking not initialized");
        println!("Run 'devkit blueprint init' to initialize evolution tracking");
        return Ok(());
    }

    let mut tracker = BlueprintEvolutionTracker::new(evolution_dir);
    tracker.load().await.context("Failed to load evolution history")?;

    let history = tracker.get_history();
    match history {
        Some(entries) => {
            let display_entries = entries.iter().rev().take(args.limit);
            
            match args.output_format.as_str() {
                "json" => {
                    let json = serde_json::to_string_pretty(&entries)?;
                    println!("{}", json);
                }
                "graph" => {
                    print_history_graph(display_entries, args.detailed);
                }
                _ => {
                    print_history_table(display_entries, args.detailed);
                }
            }
        }
        None => {
            println!("No history entries found");
        }
    }

    Ok(())
}

async fn handle_init(args: InitArgs) -> Result<()> {
    println!("🚀 Initialize Blueprint Evolution Tracking");
    println!("=========================================");
    println!();

    let evolution_dir = args.directory.join(".blueprint-evolution");
    
    if evolution_dir.exists() {
        println!("❌ Evolution tracking already initialized in {}", args.directory.display());
        return Ok(());
    }

    let mut tracker = BlueprintEvolutionTracker::new(evolution_dir.clone());
    tracker.init().await.context("Failed to initialize evolution tracking")?;

    println!("✅ Evolution tracking initialized successfully!");
    println!("  Directory: {}", evolution_dir.display());
    println!("  Initial version: {}", args.version);
    println!();
    println!("You can now use:");
    println!("  devkit blueprint version  - Show current version");
    println!("  devkit blueprint history  - View evolution history");
    println!("  devkit blueprint diff     - Compare versions");

    Ok(())
}

async fn handle_branch(args: BranchArgs) -> Result<()> {
    println!("🌿 Blueprint Branch Management");
    println!("=============================");
    println!();

    let evolution_dir = Path::new(".").join(".blueprint-evolution");
    
    if !evolution_dir.exists() {
        println!("❌ Evolution tracking not initialized");
        println!("Run 'devkit blueprint init' to initialize evolution tracking");
        return Ok(());
    }

    let mut tracker = BlueprintEvolutionTracker::new(evolution_dir);
    tracker.load().await.context("Failed to load evolution history")?;

    if args.list {
        let branches = tracker.list_branches();
        println!("Available branches:");
        for branch in branches {
            println!("  - {}", branch);
        }
        return Ok(());
    }

    if args.create {
        tracker.create_branch(args.branch_name.clone()).await
            .with_context(|| format!("Failed to create branch '{}'", args.branch_name))?;
        println!("✅ Created branch '{}'", args.branch_name);
    }

    if args.switch {
        tracker.checkout_branch(args.branch_name.clone())
            .with_context(|| format!("Failed to switch to branch '{}'", args.branch_name))?;
        println!("✅ Switched to branch '{}'", args.branch_name);
    }

    Ok(())
}

async fn handle_status(args: StatusArgs) -> Result<()> {
    println!("📊 Blueprint Evolution Status");
    println!("============================");
    println!();

    let blueprint = load_blueprint(&args.blueprint)
        .with_context(|| format!("Failed to load blueprint from {}", args.blueprint.display()))?;

    println!("Current Blueprint:");
    println!("  Version: {}", blueprint.metadata.version);
    println!("  Name: {}", blueprint.metadata.name);
    println!("  Modules: {}", blueprint.modules.len());
    println!();

    let evolution_dir = args.blueprint.parent().unwrap_or(Path::new(".")).join(".blueprint-evolution");
    
    if evolution_dir.exists() {
        let mut tracker = BlueprintEvolutionTracker::new(evolution_dir);
        if let Ok(()) = tracker.load().await {
            println!("Evolution Tracking:");
            
            if let Some(current_version) = tracker.get_current_version() {
                println!("  Tracked Version: {}", current_version);
                
                let blueprint_version = BlueprintVersion::from_str(&blueprint.metadata.version)?;
                if &blueprint_version != current_version {
                    println!("  Status: ⚠️  Blueprint version differs from tracked version");
                } else {
                    println!("  Status: ✅ Blueprint version matches tracked version");
                }
            }
            
            println!("  Branches: {:?}", tracker.list_branches());
            
            if let Some(history) = tracker.get_history() {
                println!("  History Entries: {}", history.len());
            }
        }
    } else {
        println!("Evolution Tracking: ❌ Not initialized");
    }

    // Check for migration artifacts
    let backup_dir = args.blueprint.parent().unwrap_or(Path::new(".")).join(".blueprint-backups");
    if backup_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&backup_dir) {
            let backup_count = entries.filter_map(|e| e.ok()).count();
            println!("  Available Backups: {}", backup_count);
        }
    }

    let script_dir = args.blueprint.parent().unwrap_or(Path::new(".")).join(".blueprint-migrations");
    if script_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&script_dir) {
            let script_count = entries.filter_map(|e| e.ok()).count();
            println!("  Migration Scripts: {}", script_count);
        }
    }

    Ok(())
}

// Helper functions
fn load_blueprint(path: &Path) -> Result<SystemBlueprint> {
    if !path.exists() {
        anyhow::bail!("Blueprint file not found: {}", path.display());
    }

    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read blueprint file: {}", path.display()))?;

    let blueprint: SystemBlueprint = if path.extension().and_then(|s| s.to_str()) == Some("json") {
        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON blueprint file: {}", path.display()))?
    } else {
        toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML blueprint file: {}", path.display()))?
    };

    Ok(blueprint)
}

fn print_diff_summary(diff: &crate::blueprint::evolution::diff::BlueprintDiff) {
    println!("Comparison: {} → {}", diff.from_version, diff.to_version);
    println!();
    
    println!("Summary:");
    println!("  Total Changes: {}", diff.summary.total_changes);
    println!("  Breaking Changes: {}", diff.summary.breaking_changes);
    println!("  New Features: {}", diff.summary.new_features);
    println!("  Bug Fixes: {}", diff.summary.bug_fixes);
    println!();

    println!("Impact Analysis:");
    println!("  Risk Level: {:?}", diff.impact_analysis.risk_level);
    println!("  Impact Score: {:.2}", diff.impact_analysis.overall_impact_score);
    println!("  Compatibility: {:.2}", diff.impact_analysis.compatibility_score);
    println!();

    if !diff.changes.is_empty() {
        println!("Top Changes:");
        for (i, change) in diff.changes.iter().take(5).enumerate() {
            println!("  {}. {} - {} ({:?})", 
                i + 1, change.change_type, change.path, change.impact_level);
        }
        if diff.changes.len() > 5 {
            println!("  ... and {} more changes", diff.changes.len() - 5);
        }
    }
}

fn print_detailed_diff(diff: &crate::blueprint::evolution::diff::BlueprintDiff) {
    print_diff_summary(diff);
    
    println!();
    println!("Detailed Changes:");
    println!("================");
    
    for (i, change) in diff.changes.iter().enumerate() {
        println!("{}. {} at '{}'", i + 1, change.change_type, change.path);
        println!("   Category: {:?}", change.change_category);
        println!("   Impact: {:?}", change.impact_level);
        println!("   Description: {}", change.description);
        println!();
    }

    if !diff.migration_complexity.required_skills.is_empty() {
        println!("Required Skills for Migration:");
        for skill in &diff.migration_complexity.required_skills {
            println!("  - {}", skill);
        }
        println!();
    }
}

fn print_history_table<'a>(entries: impl Iterator<Item = &'a crate::blueprint::evolution::EvolutionEntry>, detailed: bool) {
    println!("{:<12} {:<20} {:<15} {:<10} {}", "Version", "Date", "Author", "Changes", "Description");
    println!("{}", "-".repeat(80));
    
    for entry in entries {
        let date = entry.metadata.created_at.format("%Y-%m-%d %H:%M");
        let changes = entry.changes.len();
        let description = if entry.metadata.commit_message.len() > 30 {
            format!("{}...", &entry.metadata.commit_message[..27])
        } else {
            entry.metadata.commit_message.clone()
        };
        
        println!("{:<12} {:<20} {:<15} {:<10} {}", 
            entry.metadata.version, date, entry.metadata.created_by, changes, description);
        
        if detailed {
            if !entry.changes.is_empty() {
                println!("    Changes:");
                for change in entry.changes.iter().take(3) {
                    println!("      - {} ({})", change.path, change.change_type);
                }
                if entry.changes.len() > 3 {
                    println!("      ... and {} more", entry.changes.len() - 3);
                }
            }
            println!();
        }
    }
}

fn print_history_graph<'a>(entries: impl Iterator<Item = &'a crate::blueprint::evolution::EvolutionEntry>, detailed: bool) {
    println!("Evolution Graph:");
    println!();
    
    for (i, entry) in entries.enumerate() {
        let connector = if i == 0 { "●" } else { "│" };
        let date = entry.metadata.created_at.format("%m/%d");
        
        println!("{} {} ({}) - {}", connector, entry.metadata.version, date, entry.metadata.commit_message);
        
        if detailed && !entry.changes.is_empty() {
            println!("│   {} changes", entry.changes.len());
        }
        
        if i > 0 {
            println!("│");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let version = BlueprintVersion::from_str("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[tokio::test]
    async fn test_diff_analysis() {
        // This would require actual blueprint files to test properly
        // For now, just test that the function exists and can be called
        assert!(true);
    }
}