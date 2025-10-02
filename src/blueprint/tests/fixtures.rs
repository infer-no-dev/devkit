//! Test fixtures and sample data for blueprint evolution tests

use crate::blueprint::evolution::{
    BlueprintVersion, BlueprintChange, ChangeType, ChangeCategory, ImpactLevel
};
use crate::blueprint::tests::TestUtils;
use crate::blueprint::SystemBlueprint;
use std::collections::HashMap;

/// Common test fixtures for blueprint evolution tests
pub struct TestFixtures;

impl TestFixtures {
    /// Create a simple microservices blueprint fixture
    pub fn simple_microservices_blueprint() -> SystemBlueprint {
        TestUtils::create_blueprint_with_modules("microservices-app", "1.0.0", 3)
    }

    /// Create a monolithic application blueprint fixture
    pub fn monolithic_blueprint() -> SystemBlueprint {
        let mut blueprint = TestUtils::create_test_blueprint("monolith-app", "1.0.0");
        blueprint.metadata.architecture_paradigm = "monolith".to_string();
        blueprint
    }

    /// Create a serverless blueprint fixture
    pub fn serverless_blueprint() -> SystemBlueprint {
        let mut blueprint = TestUtils::create_blueprint_with_modules("serverless-app", "1.0.0", 5);
        blueprint.metadata.architecture_paradigm = "serverless".to_string();
        blueprint.metadata.primary_language = "javascript".to_string();
        blueprint
    }

    /// Create a complex blueprint with many modules and dependencies
    pub fn complex_blueprint() -> SystemBlueprint {
        TestUtils::create_large_blueprint(50, 8)
    }

    /// Create a set of common version transitions for testing
    pub fn version_transitions() -> Vec<(BlueprintVersion, BlueprintVersion, &'static str)> {
        vec![
            (
                BlueprintVersion::new(1, 0, 0),
                BlueprintVersion::new(1, 0, 1),
                "patch_release"
            ),
            (
                BlueprintVersion::new(1, 0, 0),
                BlueprintVersion::new(1, 1, 0),
                "minor_release"
            ),
            (
                BlueprintVersion::new(1, 0, 0),
                BlueprintVersion::new(2, 0, 0),
                "major_release"
            ),
            (
                BlueprintVersion::from_str("1.0.0-alpha").unwrap(),
                BlueprintVersion::new(1, 0, 0),
                "prerelease_to_stable"
            ),
        ]
    }

    /// Create sample blueprint changes for testing
    pub fn sample_changes() -> Vec<BlueprintChange> {
        vec![
            TestUtils::create_test_change(
                ChangeType::Added,
                ChangeCategory::Module,
                "modules.user_service",
                "Added user management service",
                ImpactLevel::Medium
            ),
            TestUtils::create_test_change(
                ChangeType::Modified,
                ChangeCategory::Configuration,
                "config.database.connection_pool",
                "Updated database connection pool size",
                ImpactLevel::Low
            ),
            TestUtils::create_test_change(
                ChangeType::Removed,
                ChangeCategory::Module,
                "modules.legacy_auth",
                "Removed deprecated authentication module",
                ImpactLevel::High
            ),
            TestUtils::create_test_change(
                ChangeType::Modified,
                ChangeCategory::Architecture,
                "architecture.paradigm",
                "Changed from monolith to microservices",
                ImpactLevel::High
            ),
            TestUtils::create_test_change(
                ChangeType::Added,
                ChangeCategory::Security,
                "security.encryption.algorithm",
                "Added AES-256 encryption",
                ImpactLevel::Medium
            ),
        ]
    }

    /// Create a blueprint evolution scenario for testing migrations
    pub fn migration_scenario() -> (SystemBlueprint, SystemBlueprint, Vec<BlueprintChange>) {
        let blueprint_v1 = Self::simple_microservices_blueprint();
        let mut blueprint_v2 = blueprint_v1.clone();

        // Apply changes for v2
        blueprint_v2.metadata.version = "2.0.0".to_string();
        blueprint_v2.metadata.architecture_paradigm = "event-driven".to_string();
        
        // Remove a module to simulate breaking change
        blueprint_v2.modules.retain(|m| m.name != "module_0");

        let changes = vec![
            TestUtils::create_test_change(
                ChangeType::Modified,
                ChangeCategory::Architecture,
                "metadata.architecture_paradigm",
                "Changed to event-driven architecture",
                ImpactLevel::High
            ),
            TestUtils::create_test_change(
                ChangeType::Removed,
                ChangeCategory::Module,
                "modules.module_0",
                "Removed deprecated module",
                ImpactLevel::High
            ),
        ];

        (blueprint_v1, blueprint_v2, changes)
    }

    /// Create error scenarios for testing error handling
    pub fn error_scenarios() -> Vec<(&'static str, Box<dyn Fn() -> anyhow::Result<()>>)> {
        vec![
            (
                "invalid_version_format",
                Box::new(|| {
                    BlueprintVersion::from_str("invalid.version")?;
                    Ok(())
                })
            ),
            (
                "empty_blueprint_name",
                Box::new(|| {
                    let blueprint = TestUtils::create_test_blueprint("", "1.0.0");
                    if blueprint.metadata.name.is_empty() {
                        anyhow::bail!("Blueprint name cannot be empty");
                    }
                    Ok(())
                })
            ),
        ]
    }

    /// Create performance test scenarios
    pub fn performance_scenarios() -> Vec<(&'static str, SystemBlueprint, SystemBlueprint)> {
        vec![
            (
                "small_blueprint_change",
                Self::simple_microservices_blueprint(),
                {
                    let mut bp = Self::simple_microservices_blueprint();
                    bp.metadata.description = "Updated description".to_string();
                    bp
                }
            ),
            (
                "large_blueprint_change",
                Self::complex_blueprint(),
                {
                    let mut bp = Self::complex_blueprint();
                    bp.metadata.architecture_paradigm = "new-architecture".to_string();
                    // Add many modules
                    for i in 100..110 {
                        let module_name = format!("module_{:04}", i);
                        bp.modules.push(crate::blueprint::ModuleBlueprint {
                            name: module_name,
                            purpose: format!("Performance test module {}", i),
                            dependencies: vec![],
                            public_interface: vec![],
                            internal_structure: Default::default(),
                            testing_strategy: Default::default(),
                            performance_characteristics: Default::default(),
                        });
                    }
                    bp
                }
            ),
        ]
    }

    /// Create CLI test scenarios
    pub fn cli_test_scenarios() -> HashMap<&'static str, Vec<&'static str>> {
        let mut scenarios = HashMap::new();

        scenarios.insert("version_commands", vec![
            "blueprint evolution version --blueprint test.json",
            "blueprint evolution version --detailed",
            "blueprint evolution version --format json",
        ]);

        scenarios.insert("diff_commands", vec![
            "blueprint evolution diff --from 1.0.0 --to 1.1.0",
            "blueprint evolution diff --output-format detailed",
            "blueprint evolution diff --min-impact medium",
        ]);

        scenarios.insert("migration_commands", vec![
            "blueprint evolution migrate 2.0.0 --dry-run",
            "blueprint evolution migrate 1.5.0 --environment production",
            "blueprint evolution migrate 2.1.0 --force --no-backup",
        ]);

        scenarios.insert("branch_commands", vec![
            "blueprint evolution branch --create feature-auth",
            "blueprint evolution branch --list",
            "blueprint evolution branch --switch main",
        ]);

        scenarios
    }

    /// Create multi-language blueprint fixtures
    pub fn multi_language_blueprints() -> HashMap<&'static str, SystemBlueprint> {
        let mut blueprints = HashMap::new();

        // Rust microservices
        let mut rust_bp = Self::simple_microservices_blueprint();
        rust_bp.metadata.primary_language = "rust".to_string();
        rust_bp.metadata.name = "rust-services".to_string();
        blueprints.insert("rust", rust_bp);

        // Node.js API
        let mut node_bp = TestUtils::create_test_blueprint("node-api", "1.0.0");
        node_bp.metadata.primary_language = "javascript".to_string();
        node_bp.metadata.architecture_paradigm = "api".to_string();
        blueprints.insert("nodejs", node_bp);

        // Python data pipeline
        let mut python_bp = TestUtils::create_test_blueprint("python-pipeline", "1.0.0");
        python_bp.metadata.primary_language = "python".to_string();
        python_bp.metadata.architecture_paradigm = "pipeline".to_string();
        blueprints.insert("python", python_bp);

        // Go services
        let mut go_bp = TestUtils::create_blueprint_with_modules("go-services", "1.0.0", 4);
        go_bp.metadata.primary_language = "go".to_string();
        blueprints.insert("go", go_bp);

        blueprints
    }

    /// Create edge case test data
    pub fn edge_cases() -> Vec<(&'static str, SystemBlueprint)> {
        vec![
            (
                "empty_modules",
                TestUtils::create_test_blueprint("empty", "1.0.0")
            ),
            (
                "unicode_names",
                {
                    let mut bp = TestUtils::create_test_blueprint("测试应用", "1.0.0");
                    bp.metadata.description = "Приложение с Unicode символами".to_string();
                    bp
                }
            ),
            (
                "very_long_name",
                TestUtils::create_test_blueprint(
                    &"very-long-application-name-that-exceeds-normal-length-limits".repeat(3),
                    "1.0.0"
                )
            ),
            (
                "special_characters",
                TestUtils::create_test_blueprint("app-with-@#$-chars", "1.0.0")
            ),
            (
                "max_modules",
                TestUtils::create_large_blueprint(1000, 1)
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixtures_creation() {
        // Test that all fixtures can be created without panicking
        let _microservices = TestFixtures::simple_microservices_blueprint();
        let _monolith = TestFixtures::monolithic_blueprint();
        let _serverless = TestFixtures::serverless_blueprint();
        let _complex = TestFixtures::complex_blueprint();
        
        assert!(!TestFixtures::version_transitions().is_empty());
        assert!(!TestFixtures::sample_changes().is_empty());
        
        let (v1, v2, changes) = TestFixtures::migration_scenario();
        assert_ne!(v1.metadata.version, v2.metadata.version);
        assert!(!changes.is_empty());
    }

    #[test]
    fn test_multi_language_fixtures() {
        let blueprints = TestFixtures::multi_language_blueprints();
        
        assert!(blueprints.contains_key("rust"));
        assert!(blueprints.contains_key("nodejs"));
        assert!(blueprints.contains_key("python"));
        assert!(blueprints.contains_key("go"));
        
        let rust_bp = &blueprints["rust"];
        assert_eq!(rust_bp.metadata.primary_language, "rust");
    }

    #[test]
    fn test_edge_cases() {
        let edge_cases = TestFixtures::edge_cases();
        assert!(!edge_cases.is_empty());
        
        let (name, blueprint) = &edge_cases[0];
        assert_eq!(*name, "empty_modules");
        assert!(blueprint.modules.is_empty());
    }

    #[test]
    fn test_cli_scenarios() {
        let scenarios = TestFixtures::cli_test_scenarios();
        assert!(scenarios.contains_key("version_commands"));
        assert!(scenarios.contains_key("diff_commands"));
        assert!(scenarios.contains_key("migration_commands"));
        assert!(scenarios.contains_key("branch_commands"));
    }

    #[test]
    fn test_performance_scenarios() {
        let scenarios = TestFixtures::performance_scenarios();
        assert_eq!(scenarios.len(), 2);
        
        let (name, bp1, bp2) = &scenarios[0];
        assert_eq!(*name, "small_blueprint_change");
        assert_ne!(bp1.metadata.description, bp2.metadata.description);
    }
}