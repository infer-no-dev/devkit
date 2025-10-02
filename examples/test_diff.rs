use devkit::blueprint::{SystemBlueprint, SystemType, Module, ModuleType, Dependency, DependencyType, Interface, InterfaceProtocol};
use devkit::blueprint::evolution::{BlueprintVersion, BlueprintDiffAnalyzer};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create original blueprint (v1.0.0)
    let original_blueprint = SystemBlueprint {
        name: "Example System".to_string(),
        version: "1.0.0".to_string(),
        system_type: SystemType::WebApi,
        description: "Original system description".to_string(),
        modules: vec![
            Module {
                name: "api".to_string(),
                module_type: ModuleType::Service,
                entry_point: Some("main.rs".to_string()),
                dependencies: vec![
                    Dependency {
                        name: "serde".to_string(),
                        version: "1.0".to_string(),
                        dependency_type: DependencyType::Direct,
                        purpose: Some("Serialization".to_string()),
                    },
                    Dependency {
                        name: "tokio".to_string(),
                        version: "1.0".to_string(),
                        dependency_type: DependencyType::Direct,
                        purpose: Some("Async runtime".to_string()),
                    },
                ],
                interfaces: vec![
                    Interface {
                        name: "api".to_string(),
                        protocol: InterfaceProtocol::Http,
                        methods: vec!["GET /health".to_string(), "POST /users".to_string()],
                        description: "REST API interface".to_string(),
                    },
                ],
            },
        ],
        dependencies: vec![],
        configuration: serde_json::json!({}),
        documentation: None,
        metadata: std::collections::HashMap::new(),
    };

    // Create updated blueprint (v2.0.0) with breaking changes
    let updated_blueprint = SystemBlueprint {
        name: "Example System".to_string(),
        version: "2.0.0".to_string(),
        system_type: SystemType::Microservices, // Changed architecture
        description: "Updated system with microservices architecture".to_string(),
        modules: vec![
            Module {
                name: "api".to_string(),
                module_type: ModuleType::Service,
                entry_point: Some("main.rs".to_string()),
                dependencies: vec![
                    Dependency {
                        name: "serde".to_string(),
                        version: "1.5".to_string(), // Version bump
                        dependency_type: DependencyType::Direct,
                        purpose: Some("Serialization".to_string()),
                    },
                    Dependency {
                        name: "tokio".to_string(),
                        version: "1.0".to_string(),
                        dependency_type: DependencyType::Direct,
                        purpose: Some("Async runtime".to_string()),
                    },
                    // New dependency added
                    Dependency {
                        name: "tracing".to_string(),
                        version: "0.1".to_string(),
                        dependency_type: DependencyType::Direct,
                        purpose: Some("Structured logging".to_string()),
                    },
                ],
                interfaces: vec![
                    Interface {
                        name: "api".to_string(),
                        protocol: InterfaceProtocol::Http,
                        methods: vec![
                            "GET /health".to_string(), 
                            "POST /users".to_string(),
                            "DELETE /users/{id}".to_string() // New endpoint
                        ],
                        description: "Enhanced REST API interface".to_string(),
                    },
                ],
            },
            // New module added
            Module {
                name: "auth".to_string(),
                module_type: ModuleType::Service,
                entry_point: Some("auth/main.rs".to_string()),
                dependencies: vec![
                    Dependency {
                        name: "jsonwebtoken".to_string(),
                        version: "8.0".to_string(),
                        dependency_type: DependencyType::Direct,
                        purpose: Some("JWT handling".to_string()),
                    },
                ],
                interfaces: vec![
                    Interface {
                        name: "auth".to_string(),
                        protocol: InterfaceProtocol::Http,
                        methods: vec!["POST /auth/login".to_string(), "POST /auth/refresh".to_string()],
                        description: "Authentication service interface".to_string(),
                    },
                ],
            },
        ],
        dependencies: vec![],
        configuration: serde_json::json!({"database_url": "postgres://localhost"}),
        documentation: Some("Updated documentation with microservices details".to_string()),
        metadata: {
            let mut map = std::collections::HashMap::new();
            map.insert("migration_version".to_string(), serde_json::Value::String("2.0.0".to_string()));
            map
        },
    };

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

    println!("Detailed Changes:");
    println!("================");
    for (i, change) in diff.changes.iter().enumerate() {
        println!("{}. {} at '{}'", i + 1, change.change_type, change.path);
        println!("   Category: {:?}", change.change_category);
        println!("   Impact: {:?}", change.impact_level);
        println!("   Description: {}", change.description);
        
        if let Some(old_val) = &change.old_value {
            println!("   Old Value: {}", serde_json::to_string(old_val).unwrap_or_else(|_| "N/A".to_string()));
        }
        
        if let Some(new_val) = &change.new_value {
            println!("   New Value: {}", serde_json::to_string(new_val).unwrap_or_else(|_| "N/A".to_string()));
        }
        
        println!();
    }

    Ok(())
}