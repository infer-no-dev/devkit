use devkit::blueprint::{
    SystemBlueprint, SystemMetadata, ArchitecturalDecisions, ModuleBlueprint, 
    DesignPatterns, ImplementationDetails, ConfigurationStrategy, 
    TestingStrategy, PerformanceOptimizations, SecurityPatterns, DeploymentStrategy
};
use devkit::blueprint::evolution::{BlueprintVersion, BlueprintDiffAnalyzer};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create original blueprint (v1.0.0)
    let original_blueprint = create_original_blueprint();

    // Create updated blueprint (v2.0.0) with changes
    let updated_blueprint = create_updated_blueprint();

    let analyzer = BlueprintDiffAnalyzer::new();
    
    let from_version = BlueprintVersion::new(1, 0, 0);
    let to_version = BlueprintVersion::new(2, 0, 0);

    // Perform diff analysis
    let diff = analyzer.analyze_diff(
        &original_blueprint,
        &updated_blueprint,
        from_version,
        to_version,
    )?;

    // Display results
    println!("Blueprint Diff Analysis Results");
    println!("===============================");
    println!();

    println!("Version Change: {} → {}", diff.from_version, diff.to_version);
    println!();

    println!("Summary:");
    println!("  Total Changes: {}", diff.summary.total_changes);
    println!("  Breaking Changes: {}", diff.summary.breaking_changes);
    println!("  New Features: {}", diff.summary.new_features);
    println!("  Bug Fixes: {}", diff.summary.bug_fixes);
    println!();

    println!("Changes by Category:");
    for (category, count) in &diff.summary.changes_by_category {
        println!("  {:?}: {}", category, count);
    }
    println!();

    println!("Changes by Impact:");
    for (impact, count) in &diff.summary.changes_by_impact {
        println!("  {:?}: {}", impact, count);
    }
    println!();

    println!("Impact Analysis:");
    println!("  Overall Impact Score: {:.2}", diff.impact_analysis.overall_impact_score);
    println!("  Risk Level: {:?}", diff.impact_analysis.risk_level);
    println!("  Compatibility Score: {:.2}", diff.impact_analysis.compatibility_score);
    println!();

    if !diff.impact_analysis.affected_modules.is_empty() {
        println!("  Affected Modules:");
        for module in &diff.impact_analysis.affected_modules {
            println!("    - {}", module);
        }
        println!();
    }

    if !diff.impact_analysis.dependency_impacts.is_empty() {
        println!("  Dependency Impacts:");
        for impact in &diff.impact_analysis.dependency_impacts {
            println!("    - {} ({:?}): {}", 
                impact.dependency_name, 
                impact.impact_type, 
                impact.risk_assessment
            );
        }
        println!();
    }

    if !diff.impact_analysis.interface_impacts.is_empty() {
        println!("  Interface Impacts:");
        for impact in &diff.impact_analysis.interface_impacts {
            println!("    - {} ({:?}, Breaking: {}): {}", 
                impact.interface_name, 
                impact.impact_type, 
                impact.breaking_change,
                impact.description
            );
        }
        println!();
    }

    println!("Migration Complexity:");
    println!("  Complexity Score: {:.2}", diff.migration_complexity.complexity_score);
    println!("  Estimated Effort: {:?}", diff.migration_complexity.estimated_effort);
    println!("  Rollback Difficulty: {:?}", diff.migration_complexity.rollback_difficulty);
    println!();

    if !diff.migration_complexity.required_skills.is_empty() {
        println!("  Required Skills:");
        for skill in &diff.migration_complexity.required_skills {
            println!("    - {}", skill);
        }
        println!();
    }

    if !diff.migration_complexity.critical_path_items.is_empty() {
        println!("  Critical Path Items:");
        for item in &diff.migration_complexity.critical_path_items {
            println!("    - {}", item);
        }
        println!();
    }

    println!("First 10 Detailed Changes:");
    println!("==========================");
    for (i, change) in diff.changes.iter().take(10).enumerate() {
        println!("{}. {} at '{}'", i + 1, change.change_type, change.path);
        println!("   Category: {:?}", change.change_category);
        println!("   Impact: {:?}", change.impact_level);
        println!("   Description: {}", change.description);
        println!();
    }

    if diff.changes.len() > 10 {
        println!("... and {} more changes", diff.changes.len() - 10);
    }

    Ok(())
}

fn create_original_blueprint() -> SystemBlueprint {
    SystemBlueprint {
        metadata: SystemMetadata {
            name: "Example System".to_string(),
            version: "1.0.0".to_string(),
            description: "Original system description".to_string(),
            architecture_paradigm: "Monolithic".to_string(),
            primary_language: "Rust".to_string(),
            creation_timestamp: chrono::Utc::now(),
            generator_version: "1.0.0".to_string(),
        },
        architecture: ArchitecturalDecisions::default(),
        modules: vec![
            ModuleBlueprint {
                name: "api".to_string(),
                purpose: "Main API server".to_string(),
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
            name: "Example System".to_string(),
            version: "2.0.0".to_string(),
            description: "Updated system with microservices architecture".to_string(),
            architecture_paradigm: "Microservices".to_string(), // Changed
            primary_language: "Rust".to_string(),
            creation_timestamp: chrono::Utc::now(),
            generator_version: "2.0.0".to_string(), // Changed
        },
        architecture: ArchitecturalDecisions::default(),
        modules: vec![
            ModuleBlueprint {
                name: "api".to_string(),
                purpose: "Enhanced API server".to_string(), // Changed
                dependencies: vec![],
                public_interface: vec![],
                internal_structure: Default::default(),
                testing_strategy: Default::default(),
                performance_characteristics: Default::default(),
            },
            // New module added
            ModuleBlueprint {
                name: "auth".to_string(),
                purpose: "Authentication service".to_string(),
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