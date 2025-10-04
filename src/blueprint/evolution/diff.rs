//! Blueprint Diff Analysis
//!
//! Provides intelligent diff analysis between blueprint versions with
//! structural comparison and impact assessment.

use super::*;
use anyhow::Result;
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};

/// Blueprint diff analyzer
pub struct BlueprintDiffAnalyzer {
    ignore_paths: HashSet<String>,
    weight_config: DiffWeightConfig,
}

/// Configuration for diff analysis weights
#[derive(Debug, Clone)]
pub struct DiffWeightConfig {
    pub architecture_weight: f64,
    pub dependency_weight: f64,
    pub interface_weight: f64,
    pub configuration_weight: f64,
    pub documentation_weight: f64,
}

impl Default for DiffWeightConfig {
    fn default() -> Self {
        Self {
            architecture_weight: 1.0,
            dependency_weight: 0.8,
            interface_weight: 0.9,
            configuration_weight: 0.6,
            documentation_weight: 0.3,
        }
    }
}

/// Result of blueprint diff analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintDiff {
    pub from_version: BlueprintVersion,
    pub to_version: BlueprintVersion,
    pub changes: Vec<BlueprintChange>,
    pub summary: DiffSummary,
    pub impact_analysis: ImpactAnalysis,
    pub migration_complexity: MigrationComplexity,
}

/// Summary of changes in a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    pub total_changes: usize,
    pub changes_by_category: HashMap<ChangeCategory, usize>,
    pub changes_by_impact: HashMap<ImpactLevel, usize>,
    pub breaking_changes: usize,
    pub new_features: usize,
    pub bug_fixes: usize,
}

/// Impact analysis of blueprint changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    pub overall_impact_score: f64,
    pub risk_level: RiskLevel,
    pub affected_modules: Vec<String>,
    pub dependency_impacts: Vec<DependencyImpact>,
    pub interface_impacts: Vec<InterfaceImpact>,
    pub compatibility_score: f64,
}

/// Risk level assessment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    None, // No significant risk
    Low,
    Medium,
    High,
    Critical,
}

/// Impact on dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyImpact {
    pub dependency_name: String,
    pub impact_type: DependencyImpactType,
    pub old_version: Option<String>,
    pub new_version: Option<String>,
    pub risk_assessment: String,
}

/// Type of dependency impact
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyImpactType {
    Added,
    Removed,
    VersionChanged,
    ConfigurationChanged,
}

/// Impact on interfaces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceImpact {
    pub interface_name: String,
    pub impact_type: InterfaceImpactType,
    pub breaking_change: bool,
    pub migration_required: bool,
    pub description: String,
}

/// Type of interface impact
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceImpactType {
    Added,
    Removed,
    Modified,
    Deprecated,
}

/// Migration complexity assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationComplexity {
    pub complexity_score: f64,
    pub estimated_effort: EffortLevel,
    pub required_skills: Vec<String>,
    pub critical_path_items: Vec<String>,
    pub rollback_difficulty: RollbackDifficulty,
}

/// Effort level for migration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffortLevel {
    Trivial,  // < 1 hour
    Low,      // 1-4 hours
    Medium,   // 1-2 days
    High,     // 3-7 days
    VeryHigh, // > 1 week
}

/// Difficulty of rolling back changes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RollbackDifficulty {
    Easy,       // Automatic rollback possible
    Medium,     // Manual steps required
    Hard,       // Significant effort to rollback
    Impossible, // Cannot rollback (e.g., data migration)
}

impl BlueprintDiffAnalyzer {
    /// Create new diff analyzer
    pub fn new() -> Self {
        Self {
            ignore_paths: HashSet::new(),
            weight_config: DiffWeightConfig::default(),
        }
    }

    /// Create diff analyzer with custom configuration
    pub fn with_config(weight_config: DiffWeightConfig) -> Self {
        Self {
            ignore_paths: HashSet::new(),
            weight_config,
        }
    }

    /// Add paths to ignore during diff analysis
    pub fn ignore_paths(mut self, paths: Vec<String>) -> Self {
        self.ignore_paths.extend(paths);
        self
    }

    /// Analyze differences between two blueprints
    pub fn analyze_diff(
        &self,
        from_blueprint: &crate::blueprint::SystemBlueprint,
        to_blueprint: &crate::blueprint::SystemBlueprint,
        from_version: BlueprintVersion,
        to_version: BlueprintVersion,
    ) -> Result<BlueprintDiff> {
        // Convert blueprints to JSON for structural comparison
        let from_json = serde_json::to_value(from_blueprint)?;
        let to_json = serde_json::to_value(to_blueprint)?;

        // Perform deep diff analysis
        let mut changes = Vec::new();
        self.analyze_json_diff(&from_json, &to_json, "", &mut changes)?;

        // Filter out ignored paths
        changes.retain(|change| !self.is_path_ignored(&change.path));

        // Generate summary
        let summary = self.generate_summary(&changes);

        // Perform impact analysis
        let impact_analysis = self.analyze_impact(&changes, from_blueprint, to_blueprint)?;

        // Assess migration complexity
        let migration_complexity = self.assess_migration_complexity(&changes, &impact_analysis)?;

        Ok(BlueprintDiff {
            from_version,
            to_version,
            changes,
            summary,
            impact_analysis,
            migration_complexity,
        })
    }

    /// Recursive JSON diff analysis
    fn analyze_json_diff(
        &self,
        from: &Value,
        to: &Value,
        path: &str,
        changes: &mut Vec<BlueprintChange>,
    ) -> Result<()> {
        match (from, to) {
            // Both are objects - compare keys and values
            (Value::Object(from_obj), Value::Object(to_obj)) => {
                self.compare_objects(from_obj, to_obj, path, changes)?;
            }

            // Both are arrays - compare elements
            (Value::Array(from_arr), Value::Array(to_arr)) => {
                self.compare_arrays(from_arr, to_arr, path, changes)?;
            }

            // Different types or values - record as modification
            (from_val, to_val) if from_val != to_val => {
                changes.push(BlueprintChange {
                    change_type: ChangeType::Modified,
                    change_category: self.categorize_path(path),
                    path: path.to_string(),
                    old_value: Some(from_val.clone()),
                    new_value: Some(to_val.clone()),
                    description: self.generate_change_description(path, from_val, to_val),
                    impact_level: self.assess_impact_level(path, from_val, to_val),
                });
            }

            // Values are the same - no change
            _ => {}
        }

        Ok(())
    }

    /// Compare two JSON objects
    fn compare_objects(
        &self,
        from_obj: &Map<String, Value>,
        to_obj: &Map<String, Value>,
        path: &str,
        changes: &mut Vec<BlueprintChange>,
    ) -> Result<()> {
        let from_keys: HashSet<_> = from_obj.keys().collect();
        let to_keys: HashSet<_> = to_obj.keys().collect();

        // Find added keys
        for key in to_keys.difference(&from_keys) {
            let new_path = if path.is_empty() {
                key.to_string()
            } else {
                format!("{}.{}", path, key)
            };
            changes.push(BlueprintChange {
                change_type: ChangeType::Added,
                change_category: self.categorize_path(&new_path),
                path: new_path.clone(),
                old_value: None,
                new_value: Some(to_obj[*key].clone()),
                description: format!("Added {}", new_path),
                impact_level: self.assess_impact_level(&new_path, &Value::Null, &to_obj[*key]),
            });
        }

        // Find removed keys
        for key in from_keys.difference(&to_keys) {
            let new_path = if path.is_empty() {
                key.to_string()
            } else {
                format!("{}.{}", path, key)
            };
            changes.push(BlueprintChange {
                change_type: ChangeType::Removed,
                change_category: self.categorize_path(&new_path),
                path: new_path.clone(),
                old_value: Some(from_obj[*key].clone()),
                new_value: None,
                description: format!("Removed {}", new_path),
                impact_level: self.assess_impact_level(&new_path, &from_obj[*key], &Value::Null),
            });
        }

        // Compare common keys
        for key in from_keys.intersection(&to_keys) {
            let new_path = if path.is_empty() {
                key.to_string()
            } else {
                format!("{}.{}", path, key)
            };
            self.analyze_json_diff(&from_obj[*key], &to_obj[*key], &new_path, changes)?;
        }

        Ok(())
    }

    /// Compare two JSON arrays
    fn compare_arrays(
        &self,
        from_arr: &[Value],
        to_arr: &[Value],
        path: &str,
        changes: &mut Vec<BlueprintChange>,
    ) -> Result<()> {
        // For arrays, we do a simple index-based comparison
        // TODO: Implement more sophisticated array diffing (LCS algorithm)

        let max_len = from_arr.len().max(to_arr.len());

        for i in 0..max_len {
            let new_path = format!("{}[{}]", path, i);

            match (from_arr.get(i), to_arr.get(i)) {
                (Some(from_val), Some(to_val)) => {
                    // Both exist - compare
                    self.analyze_json_diff(from_val, to_val, &new_path, changes)?;
                }
                (Some(from_val), None) => {
                    // Removed from array
                    changes.push(BlueprintChange {
                        change_type: ChangeType::Removed,
                        change_category: self.categorize_path(&new_path),
                        path: new_path,
                        old_value: Some(from_val.clone()),
                        new_value: None,
                        description: format!("Removed array element at index {}", i),
                        impact_level: ImpactLevel::Medium,
                    });
                }
                (None, Some(to_val)) => {
                    // Added to array
                    changes.push(BlueprintChange {
                        change_type: ChangeType::Added,
                        change_category: self.categorize_path(&new_path),
                        path: new_path,
                        old_value: None,
                        new_value: Some(to_val.clone()),
                        description: format!("Added array element at index {}", i),
                        impact_level: ImpactLevel::Medium,
                    });
                }
                (None, None) => unreachable!(),
            }
        }

        Ok(())
    }

    /// Categorize a path into a change category
    fn categorize_path(&self, path: &str) -> ChangeCategory {
        let path_lower = path.to_lowercase();

        if path_lower.contains("architecture") || path_lower.contains("system_type") {
            ChangeCategory::Architecture
        } else if path_lower.contains("dependencies") || path_lower.contains("dependency") {
            ChangeCategory::Dependencies
        } else if path_lower.contains("interface") || path_lower.contains("public_interface") {
            ChangeCategory::Interface
        } else if path_lower.contains("config") || path_lower.contains("configuration") {
            ChangeCategory::Configuration
        } else if path_lower.contains("module") || path_lower.contains("modules") {
            ChangeCategory::Module
        } else if path_lower.contains("performance") {
            ChangeCategory::Performance
        } else if path_lower.contains("security") {
            ChangeCategory::Security
        } else if path_lower.contains("test") || path_lower.contains("testing") {
            ChangeCategory::Testing
        } else if path_lower.contains("doc") || path_lower.contains("documentation") {
            ChangeCategory::Documentation
        } else {
            ChangeCategory::Configuration // Default
        }
    }

    /// Assess impact level of a change
    fn assess_impact_level(&self, path: &str, old_value: &Value, new_value: &Value) -> ImpactLevel {
        let category = self.categorize_path(path);

        // Base impact level by category
        let base_impact = match category {
            ChangeCategory::Architecture => ImpactLevel::Critical,
            ChangeCategory::Interface => ImpactLevel::High,
            ChangeCategory::Dependencies => ImpactLevel::High,
            ChangeCategory::Security => ImpactLevel::Critical,
            ChangeCategory::Module => ImpactLevel::Medium,
            ChangeCategory::Configuration => ImpactLevel::Medium,
            ChangeCategory::Performance => ImpactLevel::Medium,
            ChangeCategory::Testing => ImpactLevel::Low,
            ChangeCategory::Documentation => ImpactLevel::Low,
        };

        // Adjust based on change type
        match (old_value.is_null(), new_value.is_null()) {
            (true, false) => base_impact, // Addition
            (false, true) => {
                // Removal - generally higher impact
                match base_impact {
                    ImpactLevel::Low => ImpactLevel::Medium,
                    ImpactLevel::Medium => ImpactLevel::High,
                    ImpactLevel::High => ImpactLevel::Critical,
                    ImpactLevel::Critical => ImpactLevel::Critical,
                }
            }
            (false, false) => base_impact,    // Modification
            (true, true) => ImpactLevel::Low, // Shouldn't happen
        }
    }

    /// Generate description for a change
    fn generate_change_description(
        &self,
        path: &str,
        old_value: &Value,
        new_value: &Value,
    ) -> String {
        let category = self.categorize_path(path);

        match (old_value.is_null(), new_value.is_null()) {
            (true, false) => format!("Added {} to {}", self.value_summary(new_value), path),
            (false, true) => format!("Removed {} from {}", self.value_summary(old_value), path),
            (false, false) => format!(
                "Changed {} from {} to {}",
                path,
                self.value_summary(old_value),
                self.value_summary(new_value)
            ),
            (true, true) => format!("No change in {}", path),
        }
    }

    /// Generate a short summary of a JSON value
    fn value_summary(&self, value: &Value) -> String {
        match value {
            Value::String(s) => format!("\"{}\"", s),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Array(arr) => format!("array[{}]", arr.len()),
            Value::Object(obj) => format!("object with {} keys", obj.len()),
            Value::Null => "null".to_string(),
        }
    }

    /// Check if a path should be ignored
    fn is_path_ignored(&self, path: &str) -> bool {
        self.ignore_paths
            .iter()
            .any(|ignored| path.starts_with(ignored))
    }

    /// Generate diff summary
    fn generate_summary(&self, changes: &[BlueprintChange]) -> DiffSummary {
        let mut changes_by_category = HashMap::new();
        let mut changes_by_impact = HashMap::new();

        let mut breaking_changes = 0;
        let mut new_features = 0;
        let mut bug_fixes = 0;

        for change in changes {
            *changes_by_category
                .entry(change.change_category.clone())
                .or_insert(0) += 1;
            *changes_by_impact
                .entry(change.impact_level.clone())
                .or_insert(0) += 1;

            match change.impact_level {
                ImpactLevel::Critical => breaking_changes += 1,
                ImpactLevel::High => {
                    if change.change_type == ChangeType::Added {
                        new_features += 1;
                    }
                }
                ImpactLevel::Medium | ImpactLevel::Low => {
                    if change.change_type == ChangeType::Modified {
                        bug_fixes += 1;
                    }
                }
            }
        }

        DiffSummary {
            total_changes: changes.len(),
            changes_by_category,
            changes_by_impact,
            breaking_changes,
            new_features,
            bug_fixes,
        }
    }

    /// Analyze impact of changes
    fn analyze_impact(
        &self,
        changes: &[BlueprintChange],
        from_blueprint: &crate::blueprint::SystemBlueprint,
        to_blueprint: &crate::blueprint::SystemBlueprint,
    ) -> Result<ImpactAnalysis> {
        // Calculate overall impact score
        let impact_score = self.calculate_impact_score(changes);

        // Determine risk level
        let risk_level = if impact_score >= 0.8 {
            RiskLevel::Critical
        } else if impact_score >= 0.6 {
            RiskLevel::High
        } else if impact_score >= 0.3 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        // Find affected modules
        let affected_modules = self.find_affected_modules(changes);

        // Analyze dependency impacts
        let dependency_impacts = self.analyze_dependency_impacts(from_blueprint, to_blueprint)?;

        // Analyze interface impacts
        let interface_impacts = self.analyze_interface_impacts(changes);

        // Calculate compatibility score
        let compatibility_score =
            self.calculate_compatibility_score(&dependency_impacts, &interface_impacts);

        Ok(ImpactAnalysis {
            overall_impact_score: impact_score,
            risk_level,
            affected_modules,
            dependency_impacts,
            interface_impacts,
            compatibility_score,
        })
    }

    /// Calculate overall impact score (0.0 to 1.0)
    fn calculate_impact_score(&self, changes: &[BlueprintChange]) -> f64 {
        if changes.is_empty() {
            return 0.0;
        }

        let mut total_weighted_impact = 0.0;
        let mut total_weight = 0.0;

        for change in changes {
            let category_weight = match change.change_category {
                ChangeCategory::Architecture => self.weight_config.architecture_weight,
                ChangeCategory::Dependencies => self.weight_config.dependency_weight,
                ChangeCategory::Interface => self.weight_config.interface_weight,
                ChangeCategory::Configuration => self.weight_config.configuration_weight,
                ChangeCategory::Documentation => self.weight_config.documentation_weight,
                _ => 0.5, // Default weight
            };

            let impact_weight = match change.impact_level {
                ImpactLevel::Critical => 1.0,
                ImpactLevel::High => 0.75,
                ImpactLevel::Medium => 0.5,
                ImpactLevel::Low => 0.25,
            };

            total_weighted_impact += category_weight * impact_weight;
            total_weight += category_weight;
        }

        if total_weight == 0.0 {
            0.0
        } else {
            (total_weighted_impact / total_weight).min(1.0)
        }
    }

    /// Find modules affected by changes
    fn find_affected_modules(&self, changes: &[BlueprintChange]) -> Vec<String> {
        let mut modules = HashSet::new();

        for change in changes {
            if change.path.contains("modules") || change.path.contains("module") {
                // Extract module name from path
                if let Some(module_name) = self.extract_module_name(&change.path) {
                    modules.insert(module_name);
                }
            }
        }

        modules.into_iter().collect()
    }

    /// Extract module name from path
    fn extract_module_name(&self, path: &str) -> Option<String> {
        // Look for patterns like "modules[0].name" or "modules.some_module"
        if let Some(modules_pos) = path.find("modules") {
            let after_modules = &path[modules_pos + 7..]; // Skip "modules"

            // Handle array index format: modules[0].name
            if after_modules.starts_with('[') {
                if let Some(close_bracket) = after_modules.find(']') {
                    let index_str = &after_modules[1..close_bracket];
                    return Some(format!("module_{}", index_str));
                }
            }

            // Handle object format: modules.module_name
            if after_modules.starts_with('.') {
                let after_dot = &after_modules[1..];
                if let Some(next_dot) = after_dot.find('.') {
                    return Some(after_dot[..next_dot].to_string());
                } else {
                    return Some(after_dot.to_string());
                }
            }
        }

        None
    }

    /// Analyze dependency impacts
    fn analyze_dependency_impacts(
        &self,
        from_blueprint: &crate::blueprint::SystemBlueprint,
        to_blueprint: &crate::blueprint::SystemBlueprint,
    ) -> Result<Vec<DependencyImpact>> {
        // This is a simplified implementation
        // In a real system, we'd deeply analyze module dependencies
        let mut impacts = Vec::new();

        // For now, we'll just check if there are module changes
        if from_blueprint.modules.len() != to_blueprint.modules.len() {
            impacts.push(DependencyImpact {
                dependency_name: "module_count".to_string(),
                impact_type: DependencyImpactType::ConfigurationChanged,
                old_version: Some(from_blueprint.modules.len().to_string()),
                new_version: Some(to_blueprint.modules.len().to_string()),
                risk_assessment: "Module count changed, may affect dependencies".to_string(),
            });
        }

        Ok(impacts)
    }

    /// Analyze interface impacts
    fn analyze_interface_impacts(&self, changes: &[BlueprintChange]) -> Vec<InterfaceImpact> {
        let mut impacts = Vec::new();

        for change in changes {
            if change.change_category == ChangeCategory::Interface {
                let impact_type = match change.change_type {
                    ChangeType::Added => InterfaceImpactType::Added,
                    ChangeType::Removed => InterfaceImpactType::Removed,
                    ChangeType::Modified => InterfaceImpactType::Modified,
                    _ => InterfaceImpactType::Modified,
                };

                let breaking_change = change.impact_level == ImpactLevel::Critical;

                impacts.push(InterfaceImpact {
                    interface_name: change.path.clone(),
                    impact_type,
                    breaking_change,
                    migration_required: breaking_change,
                    description: change.description.clone(),
                });
            }
        }

        impacts
    }

    /// Calculate compatibility score
    fn calculate_compatibility_score(
        &self,
        dependency_impacts: &[DependencyImpact],
        interface_impacts: &[InterfaceImpact],
    ) -> f64 {
        if dependency_impacts.is_empty() && interface_impacts.is_empty() {
            return 1.0; // Perfect compatibility
        }

        let breaking_changes = interface_impacts
            .iter()
            .filter(|impact| impact.breaking_change)
            .count();

        let major_dependency_changes = dependency_impacts
            .iter()
            .filter(|impact| {
                matches!(
                    impact.impact_type,
                    DependencyImpactType::Removed | DependencyImpactType::VersionChanged
                )
            })
            .count();

        let total_significant_changes = breaking_changes + major_dependency_changes;
        let total_changes = interface_impacts.len() + dependency_impacts.len();

        if total_changes == 0 {
            1.0
        } else {
            1.0 - (total_significant_changes as f64 / total_changes as f64)
        }
    }

    /// Assess migration complexity
    fn assess_migration_complexity(
        &self,
        changes: &[BlueprintChange],
        impact_analysis: &ImpactAnalysis,
    ) -> Result<MigrationComplexity> {
        // Calculate complexity score based on various factors
        let mut complexity_score = 0.0;

        // Factor in number and severity of changes
        complexity_score += changes.len() as f64 * 0.1;
        complexity_score += impact_analysis.overall_impact_score * 0.5;

        // Factor in breaking changes
        if impact_analysis.risk_level == RiskLevel::Critical {
            complexity_score += 0.3;
        }

        // Cap at 1.0
        complexity_score = complexity_score.min(1.0);

        // Determine effort level
        let estimated_effort = if complexity_score < 0.2 {
            EffortLevel::Trivial
        } else if complexity_score < 0.4 {
            EffortLevel::Low
        } else if complexity_score < 0.6 {
            EffortLevel::Medium
        } else if complexity_score < 0.8 {
            EffortLevel::High
        } else {
            EffortLevel::VeryHigh
        };

        // Determine required skills
        let mut required_skills = HashSet::new();
        for change in changes {
            match change.change_category {
                ChangeCategory::Architecture => {
                    required_skills.insert("System Architecture".to_string());
                }
                ChangeCategory::Dependencies => {
                    required_skills.insert("Dependency Management".to_string());
                }
                ChangeCategory::Interface => {
                    required_skills.insert("API Design".to_string());
                }
                ChangeCategory::Security => {
                    required_skills.insert("Security Engineering".to_string());
                }
                _ => {}
            }
        }

        // Find critical path items
        let critical_path_items = changes
            .iter()
            .filter(|change| change.impact_level == ImpactLevel::Critical)
            .map(|change| change.path.clone())
            .collect();

        // Assess rollback difficulty
        let rollback_difficulty = if impact_analysis
            .interface_impacts
            .iter()
            .any(|impact| impact.breaking_change)
        {
            RollbackDifficulty::Hard
        } else if complexity_score > 0.6 {
            RollbackDifficulty::Medium
        } else {
            RollbackDifficulty::Easy
        };

        Ok(MigrationComplexity {
            complexity_score,
            estimated_effort,
            required_skills: required_skills.into_iter().collect(),
            critical_path_items,
            rollback_difficulty,
        })
    }
}

impl Default for BlueprintDiffAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_impact_level_assessment() {
        let analyzer = BlueprintDiffAnalyzer::new();

        // Architecture change should be critical
        let impact = analyzer.assess_impact_level(
            "architecture.system_type",
            &Value::String("monolith".to_string()),
            &Value::String("microservices".to_string()),
        );
        assert_eq!(impact, ImpactLevel::Critical);

        // Documentation change should be low
        let impact = analyzer.assess_impact_level(
            "documentation.readme",
            &Value::String("old".to_string()),
            &Value::String("new".to_string()),
        );
        assert_eq!(impact, ImpactLevel::Low);
    }

    #[test]
    fn test_module_name_extraction() {
        let analyzer = BlueprintDiffAnalyzer::new();

        assert_eq!(
            analyzer.extract_module_name("modules[0].name"),
            Some("module_0".to_string())
        );

        assert_eq!(
            analyzer.extract_module_name("modules.user_service.config"),
            Some("user_service".to_string())
        );

        assert_eq!(analyzer.extract_module_name("config.database"), None);
    }
}
