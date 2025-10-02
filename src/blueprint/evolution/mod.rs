//! Blueprint Evolution Tracking
//!
//! This module provides comprehensive blueprint versioning, evolution tracking,
//! and migration capabilities to manage blueprint changes over time.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub mod diff;
pub mod migration;
pub mod generators;
pub use diff::*;
pub use migration::*;
pub use generators::*;

/// Semantic version for blueprints
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BlueprintVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre_release: Option<String>,
    pub build: Option<String>,
}

impl BlueprintVersion {
    /// Create a new blueprint version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
            build: None,
        }
    }

    /// Create version from string (e.g., "1.2.3-alpha+build")
    pub fn from_str(version_str: &str) -> Result<Self> {
        let parts: Vec<&str> = version_str.split('+').collect();
        let (version_part, build) = if parts.len() > 1 {
            (parts[0], Some(parts[1].to_string()))
        } else {
            (version_str, None)
        };

        let parts: Vec<&str> = version_part.split('-').collect();
        let (core_version, pre_release) = if parts.len() > 1 {
            (parts[0], Some(parts[1].to_string()))
        } else {
            (version_part, None)
        };

        let version_nums: Vec<&str> = core_version.split('.').collect();
        if version_nums.len() != 3 {
            return Err(anyhow::anyhow!("Invalid version format: {}", version_str));
        }

        let major = version_nums[0].parse()?;
        let minor = version_nums[1].parse()?;
        let patch = version_nums[2].parse()?;

        Ok(Self {
            major,
            minor,
            patch,
            pre_release,
            build,
        })
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        let mut version = format!("{}.{}.{}", self.major, self.minor, self.patch);
        
        if let Some(ref pre_release) = self.pre_release {
            version.push_str(&format!("-{}", pre_release));
        }
        
        if let Some(ref build) = self.build {
            version.push_str(&format!("+{}", build));
        }
        
        version
    }

    /// Check if this is a breaking change from another version
    pub fn is_breaking_change_from(&self, other: &BlueprintVersion) -> bool {
        self.major > other.major
    }

    /// Check if this is a feature change from another version
    pub fn is_feature_change_from(&self, other: &BlueprintVersion) -> bool {
        self.major == other.major && self.minor > other.minor
    }

    /// Check if this is a patch change from another version
    pub fn is_patch_change_from(&self, other: &BlueprintVersion) -> bool {
        self.major == other.major && self.minor == other.minor && self.patch > other.patch
    }

    /// Increment major version (breaking change)
    pub fn increment_major(&mut self) {
        self.major += 1;
        self.minor = 0;
        self.patch = 0;
        self.pre_release = None;
        self.build = None;
    }

    /// Increment minor version (feature addition)
    pub fn increment_minor(&mut self) {
        self.minor += 1;
        self.patch = 0;
        self.pre_release = None;
        self.build = None;
    }

    /// Increment patch version (bug fix)
    pub fn increment_patch(&mut self) {
        self.patch += 1;
        self.pre_release = None;
        self.build = None;
    }
}

/// Evolution metadata for a blueprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintEvolutionMeta {
    pub blueprint_id: String,
    pub version: BlueprintVersion,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub commit_message: String,
    pub parent_versions: Vec<BlueprintVersion>,
    pub tags: Vec<String>,
    pub checksums: HashMap<String, String>, // File path -> hash
}

/// Evolution entry representing a single point in blueprint history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionEntry {
    pub id: String,
    pub metadata: BlueprintEvolutionMeta,
    pub blueprint: crate::blueprint::SystemBlueprint,
    pub changes: Vec<BlueprintChange>,
    pub migration_scripts: Vec<MigrationScript>,
}

/// Type of change in blueprint evolution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Modified,
    Removed,
    Moved,
    Renamed,
}

impl std::fmt::Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeType::Added => write!(f, "Added"),
            ChangeType::Modified => write!(f, "Modified"),
            ChangeType::Removed => write!(f, "Removed"),
            ChangeType::Moved => write!(f, "Moved"),
            ChangeType::Renamed => write!(f, "Renamed"),
        }
    }
}

/// Specific change in blueprint evolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintChange {
    pub change_type: ChangeType,
    pub change_category: ChangeCategory,
    pub path: String, // JSON path to changed element
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub description: String,
    pub impact_level: ImpactLevel,
}

/// Category of blueprint change
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChangeCategory {
    Architecture,
    Dependencies,
    Configuration,
    Module,
    Interface,
    Performance,
    Security,
    Testing,
    Documentation,
}

/// Impact level of a blueprint change
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ImpactLevel {
    Low,      // Cosmetic changes, documentation
    Medium,   // Feature additions, configuration changes
    High,     // Interface changes, dependency updates
    Critical, // Breaking changes, security issues
}

/// Migration script for blueprint evolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationScript {
    pub id: String,
    pub from_version: BlueprintVersion,
    pub to_version: BlueprintVersion,
    pub script_type: MigrationType,
    pub script_content: String,
    pub rollback_script: Option<String>,
    pub validation_checks: Vec<ValidationCheck>,
    pub estimated_duration: Option<std::time::Duration>,
}

/// Type of migration operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationType {
    Automatic,      // Can be applied automatically
    SemiAutomatic,  // Requires user input
    Manual,         // Requires manual intervention
    Rollback,       // Rollback operation
}

/// Validation check for migration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCheck {
    pub name: String,
    pub description: String,
    pub check_type: ValidationType,
    pub expected_result: String,
    pub failure_action: FailureAction,
}

/// Type of validation check
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationType {
    FileExists,
    DirectoryExists,
    CommandSuccess,
    ConfigurationValid,
    DependencyResolved,
    CompilationSuccess,
    TestPass,
    CustomScript,
}

/// Action to take on validation failure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureAction {
    Abort,
    Warn,
    Retry,
    Skip,
    Rollback,
}

/// Blueprint evolution tracker - main interface for evolution management
pub struct BlueprintEvolutionTracker {
    history_path: PathBuf,
    current_branch: String,
    branches: HashMap<String, Vec<EvolutionEntry>>,
}

impl BlueprintEvolutionTracker {
    /// Create new evolution tracker
    pub fn new(history_path: PathBuf) -> Self {
        Self {
            history_path,
            current_branch: "main".to_string(),
            branches: HashMap::new(),
        }
    }

    /// Initialize evolution tracking in a directory
    pub async fn init(&mut self) -> Result<()> {
        tokio::fs::create_dir_all(&self.history_path).await?;
        
        let config_path = self.history_path.join("evolution.json");
        let config = EvolutionConfig {
            version: BlueprintVersion::new(1, 0, 0),
            created_at: Utc::now(),
            current_branch: self.current_branch.clone(),
            branches: vec!["main".to_string()],
        };
        
        let config_json = serde_json::to_string_pretty(&config)?;
        tokio::fs::write(&config_path, config_json).await?;
        
        Ok(())
    }

    /// Load evolution history from disk
    pub async fn load(&mut self) -> Result<()> {
        let config_path = self.history_path.join("evolution.json");
        if !config_path.exists() {
            return Err(anyhow::anyhow!("Evolution tracking not initialized"));
        }

        let config_content = tokio::fs::read_to_string(&config_path).await?;
        let config: EvolutionConfig = serde_json::from_str(&config_content)?;
        
        self.current_branch = config.current_branch;

        // Load all branches
        for branch_name in &config.branches {
            let branch_path = self.history_path.join(format!("{}.json", branch_name));
            if branch_path.exists() {
                let branch_content = tokio::fs::read_to_string(&branch_path).await?;
                let entries: Vec<EvolutionEntry> = serde_json::from_str(&branch_content)?;
                self.branches.insert(branch_name.clone(), entries);
            }
        }

        Ok(())
    }

    /// Save evolution history to disk
    pub async fn save(&self) -> Result<()> {
        // Save evolution config
        let config = EvolutionConfig {
            version: BlueprintVersion::new(1, 0, 0),
            created_at: Utc::now(),
            current_branch: self.current_branch.clone(),
            branches: self.branches.keys().cloned().collect(),
        };
        
        let config_path = self.history_path.join("evolution.json");
        let config_json = serde_json::to_string_pretty(&config)?;
        tokio::fs::write(&config_path, config_json).await?;

        // Save all branches
        for (branch_name, entries) in &self.branches {
            let branch_path = self.history_path.join(format!("{}.json", branch_name));
            let branch_json = serde_json::to_string_pretty(entries)?;
            tokio::fs::write(&branch_path, branch_json).await?;
        }

        Ok(())
    }

    /// Add new evolution entry
    pub async fn add_entry(&mut self, entry: EvolutionEntry) -> Result<()> {
        let branch_entries = self.branches
            .entry(self.current_branch.clone())
            .or_insert_with(Vec::new);
        
        branch_entries.push(entry);
        self.save().await?;
        
        Ok(())
    }

    /// Get current blueprint version
    pub fn get_current_version(&self) -> Option<&BlueprintVersion> {
        self.branches
            .get(&self.current_branch)?
            .last()
            .map(|entry| &entry.metadata.version)
    }

    /// Get evolution history for current branch
    pub fn get_history(&self) -> Option<&Vec<EvolutionEntry>> {
        self.branches.get(&self.current_branch)
    }

    /// Get specific version from history
    pub fn get_version(&self, version: &BlueprintVersion) -> Option<&EvolutionEntry> {
        self.branches
            .get(&self.current_branch)?
            .iter()
            .find(|entry| &entry.metadata.version == version)
    }

    /// Create new branch from current state
    pub async fn create_branch(&mut self, branch_name: String) -> Result<()> {
        if self.branches.contains_key(&branch_name) {
            return Err(anyhow::anyhow!("Branch '{}' already exists", branch_name));
        }

        // Copy current branch history to new branch
        let current_history = self.branches
            .get(&self.current_branch)
            .cloned()
            .unwrap_or_default();
        
        self.branches.insert(branch_name, current_history);
        self.save().await?;
        
        Ok(())
    }

    /// Switch to a different branch
    pub fn checkout_branch(&mut self, branch_name: String) -> Result<()> {
        if !self.branches.contains_key(&branch_name) {
            return Err(anyhow::anyhow!("Branch '{}' does not exist", branch_name));
        }

        self.current_branch = branch_name;
        Ok(())
    }

    /// List all branches
    pub fn list_branches(&self) -> Vec<String> {
        self.branches.keys().cloned().collect()
    }
}

/// Evolution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EvolutionConfig {
    version: BlueprintVersion,
    created_at: DateTime<Utc>,
    current_branch: String,
    branches: Vec<String>,
}

impl std::fmt::Display for BlueprintVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::str::FromStr for BlueprintVersion {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_blueprint_version_parsing() {
        let version = BlueprintVersion::from_str("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.pre_release, None);
        assert_eq!(version.build, None);
        
        let version = BlueprintVersion::from_str("2.0.0-alpha+build123").unwrap();
        assert_eq!(version.major, 2);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 0);
        assert_eq!(version.pre_release, Some("alpha".to_string()));
        assert_eq!(version.build, Some("build123".to_string()));
    }
    
    #[test]
    fn test_version_comparison() {
        let v1 = BlueprintVersion::new(1, 0, 0);
        let v2 = BlueprintVersion::new(2, 0, 0);
        let v3 = BlueprintVersion::new(1, 1, 0);
        let v4 = BlueprintVersion::new(1, 0, 1);
        
        assert!(v2.is_breaking_change_from(&v1));
        assert!(v3.is_feature_change_from(&v1));
        assert!(v4.is_patch_change_from(&v1));
        
        assert!(!v1.is_breaking_change_from(&v2));
        assert!(!v1.is_feature_change_from(&v3));
        assert!(!v1.is_patch_change_from(&v4));
    }
    
    #[test]
    fn test_version_increment() {
        let mut version = BlueprintVersion::new(1, 2, 3);
        
        version.increment_patch();
        assert_eq!(version.to_string(), "1.2.4");
        
        version.increment_minor();
        assert_eq!(version.to_string(), "1.3.0");
        
        version.increment_major();
        assert_eq!(version.to_string(), "2.0.0");
    }
}