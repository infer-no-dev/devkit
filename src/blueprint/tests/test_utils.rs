//! Test utilities for blueprint evolution testing

use crate::blueprint::{SystemBlueprint, SystemMetadata, ModuleBlueprint};
use crate::blueprint::evolution::{
    BlueprintVersion, BlueprintEvolutionTracker, BlueprintDiffAnalyzer, 
    MigrationEngine, EvolutionEntry, BlueprintChange, ChangeType, ChangeCategory, 
    ImpactLevel
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use uuid::Uuid;

/// Test utilities for blueprint evolution
pub struct TestUtils;

impl TestUtils {
    /// Create a temporary directory for testing
    pub fn create_temp_dir() -> Result<TempDir> {
        tempfile::tempdir().map_err(Into::into)
    }

    /// Create a test blueprint with specified version
    pub fn create_test_blueprint(name: &str, version: &str) -> SystemBlueprint {
        SystemBlueprint {
            metadata: SystemMetadata {
                name: name.to_string(),
                version: version.to_string(),
                description: format!("Test blueprint for {}", name),
                architecture_paradigm: "microservices".to_string(),
                primary_language: "rust".to_string(),
                creation_timestamp: Utc::now(),
                generator_version: "test-1.0.0".to_string(),
            },
            architecture: Default::default(),
            modules: Vec::new(),
            patterns: Default::default(),
            implementation: Default::default(),
            configuration: Default::default(),
            testing: Default::default(),
            performance: Default::default(),
            security: Default::default(),
            deployment: Default::default(),
        }
    }

    /// Create a test blueprint with modules
    pub fn create_blueprint_with_modules(name: &str, version: &str, module_count: usize) -> SystemBlueprint {
        let mut blueprint = Self::create_test_blueprint(name, version);
        
        for i in 0..module_count {
            let module_name = format!("module_{}", i);
            blueprint.modules.push(ModuleBlueprint {
                name: module_name,
                purpose: format!("Test module {} purpose", i),
                dependencies: vec![],
                public_interface: vec![],
                internal_structure: Default::default(),
                testing_strategy: Default::default(),
                performance_characteristics: Default::default(),
            });
        }
        
        blueprint
    }

    /// Create a modified version of a blueprint
    pub fn create_modified_blueprint(original: &SystemBlueprint, changes: Vec<TestChange>) -> SystemBlueprint {
        let mut modified = original.clone();
        
        for change in changes {
            match change {
                TestChange::UpdateVersion(version) => {
                    modified.metadata.version = version;
                }
                TestChange::UpdateDescription(desc) => {
                    modified.metadata.description = desc;
                }
                TestChange::AddModule(name) => {
                    modified.modules.push(ModuleBlueprint {
                        name: name.clone(),
                        purpose: format!("Added module: {}", name),
                        dependencies: vec![],
                        public_interface: vec![],
                        internal_structure: Default::default(),
                        testing_strategy: Default::default(),
                        performance_characteristics: Default::default(),
                    });
                }
                TestChange::RemoveModule(name) => {
                    modified.modules.retain(|m| m.name != name);
                }
                TestChange::UpdateArchitecture(arch) => {
                    modified.metadata.architecture_paradigm = arch;
                }
            }
        }
        
        modified
    }

    /// Save blueprint to temporary file
    pub fn save_blueprint_to_temp(blueprint: &SystemBlueprint, temp_dir: &TempDir) -> Result<PathBuf> {
        let file_path = temp_dir.path().join("blueprint.json");
        let json = serde_json::to_string_pretty(blueprint)?;
        std::fs::write(&file_path, json)?;
        Ok(file_path)
    }

    /// Create evolution tracker with test data
    pub fn create_test_tracker(temp_dir: &TempDir) -> BlueprintEvolutionTracker {
        let evolution_dir = temp_dir.path().join(".blueprint-evolution");
        std::fs::create_dir_all(&evolution_dir).unwrap();
        BlueprintEvolutionTracker::new(evolution_dir)
    }

    /// Create test evolution entry
    pub fn create_test_entry(
        version: BlueprintVersion,
        blueprint: SystemBlueprint,
        changes: Vec<BlueprintChange>
    ) -> EvolutionEntry {
        EvolutionEntry {
            id: Uuid::new_v4().to_string(),
            metadata: crate::blueprint::evolution::BlueprintEvolutionMeta {
                blueprint_id: blueprint.metadata.name.clone(),
                version,
                created_at: Utc::now(),
                created_by: "test-user".to_string(),
                commit_message: "Test commit".to_string(),
                parent_versions: vec![],
                tags: vec!["test".to_string()],
                checksums: HashMap::new(),
            },
            blueprint,
            changes,
            migration_scripts: vec![],
        }
    }

    /// Create test blueprint change
    pub fn create_test_change(
        change_type: ChangeType,
        category: ChangeCategory,
        path: &str,
        description: &str,
        impact: ImpactLevel
    ) -> BlueprintChange {
        BlueprintChange {
            change_type,
            change_category: category,
            path: path.to_string(),
            old_value: None,
            new_value: None,
            description: description.to_string(),
            impact_level: impact,
        }
    }

    /// Assert blueprint versions are equal
    pub fn assert_versions_equal(v1: &BlueprintVersion, v2: &BlueprintVersion) {
        assert_eq!(v1.major, v2.major, "Major versions should match");
        assert_eq!(v1.minor, v2.minor, "Minor versions should match");
        assert_eq!(v1.patch, v2.patch, "Patch versions should match");
        assert_eq!(v1.pre_release, v2.pre_release, "Pre-release should match");
        assert_eq!(v1.build, v2.build, "Build should match");
    }

    /// Assert blueprints are structurally similar
    pub fn assert_blueprints_similar(b1: &SystemBlueprint, b2: &SystemBlueprint) {
        assert_eq!(b1.metadata.name, b2.metadata.name, "Blueprint names should match");
        assert_eq!(b1.modules.len(), b2.modules.len(), "Module count should match");
        assert_eq!(b1.metadata.architecture_paradigm, b2.metadata.architecture_paradigm, "Architecture should match");
    }

    /// Create a large blueprint for performance testing
    pub fn create_large_blueprint(module_count: usize, deps_per_module: usize) -> SystemBlueprint {
        let mut blueprint = Self::create_test_blueprint("large-system", "1.0.0");
        
        // Create modules with dependencies
        for i in 0..module_count {
            let module_name = format!("module_{:04}", i);
            let mut dependencies = vec![];
            
            // Add dependencies to previous modules
            for j in 0..(deps_per_module.min(i)) {
                dependencies.push(crate::blueprint::ModuleDependency {
                    module: format!("module_{:04}", j),
                    dependency_type: "required".to_string(),
                    usage_pattern: "direct_call".to_string(),
                    coupling_strength: "loose".to_string(),
                });
            }
            
            blueprint.modules.push(ModuleBlueprint {
                name: module_name,
                purpose: format!("Large system module {}", i),
                dependencies,
                public_interface: vec![],
                internal_structure: Default::default(),
                testing_strategy: Default::default(),
                performance_characteristics: Default::default(),
            });
        }
        
        blueprint
    }

    /// Measure execution time of a closure
    pub fn measure_time<T, F: FnOnce() -> T>(f: F) -> (T, std::time::Duration) {
        let start = std::time::Instant::now();
        let result = f();
        let duration = start.elapsed();
        (result, duration)
    }

    /// Create concurrent test scenario
    pub async fn run_concurrent_test<F, Fut, T>(tasks: Vec<F>) -> Vec<Result<T>>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<T>> + Send,
        T: Send + 'static,
    {
        let handles: Vec<_> = tasks.into_iter()
            .map(|task| tokio::spawn(async move { task().await }))
            .collect();

        let mut results = vec![];
        for handle in handles {
            let result = handle.await.unwrap_or_else(|e| Err(anyhow::anyhow!("Task panicked: {}", e)));
            results.push(result);
        }
        results
    }
}

/// Test change types for blueprint modification
#[derive(Debug, Clone)]
pub enum TestChange {
    UpdateVersion(String),
    UpdateDescription(String),
    AddModule(String),
    RemoveModule(String),
    UpdateArchitecture(String),
}

/// Test assertions and utilities
pub struct TestAssertions;

impl TestAssertions {
    /// Assert that a result contains a specific error message
    pub fn assert_error_contains<T>(result: &Result<T>, message: &str) {
        match result {
            Err(e) => assert!(
                e.to_string().contains(message),
                "Expected error to contain '{}', but got: {}",
                message,
                e
            ),
            Ok(_) => panic!("Expected error but got Ok result"),
        }
    }

    /// Assert that two vectors contain the same elements (order independent)
    pub fn assert_vecs_equal_unordered<T: PartialEq + std::fmt::Debug>(v1: &[T], v2: &[T]) {
        assert_eq!(v1.len(), v2.len(), "Vectors have different lengths");
        for item in v1 {
            assert!(
                v2.contains(item),
                "Item {:?} not found in second vector",
                item
            );
        }
    }

    /// Assert that a duration is within acceptable bounds
    pub fn assert_duration_within(
        actual: std::time::Duration,
        expected: std::time::Duration,
        tolerance: std::time::Duration,
    ) {
        let diff = if actual > expected {
            actual - expected
        } else {
            expected - actual
        };
        assert!(
            diff <= tolerance,
            "Duration {:?} is not within {:?} of expected {:?}",
            actual,
            tolerance,
            expected
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_blueprint() {
        let blueprint = TestUtils::create_test_blueprint("test-app", "1.0.0");
        assert_eq!(blueprint.metadata.name, "test-app");
        assert_eq!(blueprint.metadata.version, "1.0.0");
        assert!(blueprint.modules.is_empty());
    }

    #[test]
    fn test_create_blueprint_with_modules() {
        let blueprint = TestUtils::create_blueprint_with_modules("test-app", "1.0.0", 3);
        assert_eq!(blueprint.modules.len(), 3);
        assert!(blueprint.modules.iter().any(|m| m.name == "module_0"));
        assert!(blueprint.modules.iter().any(|m| m.name == "module_1"));
        assert!(blueprint.modules.iter().any(|m| m.name == "module_2"));
    }

    #[test]
    fn test_modify_blueprint() {
        let original = TestUtils::create_test_blueprint("test-app", "1.0.0");
        let modified = TestUtils::create_modified_blueprint(&original, vec![
            TestChange::UpdateVersion("2.0.0".to_string()),
            TestChange::AddModule("new_module".to_string()),
        ]);

        assert_eq!(modified.metadata.version, "2.0.0");
        assert_eq!(modified.modules.len(), 1);
        assert!(modified.modules.iter().any(|m| m.name == "new_module"));
    }

    #[test]
    fn test_assert_error_contains() {
        let result: Result<()> = Err(anyhow::anyhow!("This is a test error"));
        TestAssertions::assert_error_contains(&result, "test error");
    }

    #[test]
    fn test_assert_vecs_equal_unordered() {
        let v1 = vec![1, 2, 3];
        let v2 = vec![3, 1, 2];
        TestAssertions::assert_vecs_equal_unordered(&v1, &v2);
    }
}