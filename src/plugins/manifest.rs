//! Plugin Manifest Parser
//!
//! Provides robust TOML parsing for plugin manifests with validation,
//! error handling, and schema enforcement.

use crate::plugins::{PluginError, PluginMetadata, PluginDependency, PluginCapability};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::{debug, info};

/// Plugin manifest parser
#[derive(Debug, Clone)]
pub struct PluginManifestParser {
    /// Validation settings
    validation_config: ManifestValidationConfig,
    /// Schema registry for different manifest versions
    schema_registry: HashMap<String, ManifestSchema>,
}

/// Manifest validation configuration
#[derive(Debug, Clone)]
pub struct ManifestValidationConfig {
    /// Require strict schema validation
    pub strict_validation: bool,
    /// Maximum plugin name length
    pub max_name_length: usize,
    /// Maximum description length
    pub max_description_length: usize,
    /// Allowed license types
    pub allowed_licenses: Vec<String>,
    /// Maximum dependency count
    pub max_dependencies: usize,
}

impl Default for ManifestValidationConfig {
    fn default() -> Self {
        Self {
            strict_validation: true,
            max_name_length: 100,
            max_description_length: 1000,
            allowed_licenses: vec![
                "MIT".to_string(),
                "Apache-2.0".to_string(),
                "GPL-3.0".to_string(),
                "BSD-2-Clause".to_string(),
                "BSD-3-Clause".to_string(),
                "MPL-2.0".to_string(),
                "LGPL-2.1".to_string(),
                "LGPL-3.0".to_string(),
                "UNLICENSE".to_string(),
                "ISC".to_string(),
            ],
            max_dependencies: 50,
        }
    }
}

/// Raw plugin manifest from TOML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin metadata
    pub plugin: PluginMetadataRaw,
    /// Dependencies section
    pub dependencies: Option<Vec<PluginDependencyRaw>>,
    /// Build configuration
    pub build: Option<BuildConfig>,
    /// Runtime configuration
    pub runtime: Option<RuntimeConfig>,
    /// Development configuration
    pub dev: Option<DevConfig>,
}

/// Raw plugin metadata from manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadataRaw {
    /// Plugin ID
    pub id: String,
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// Homepage URL
    pub homepage: Option<String>,
    /// Repository URL
    pub repository: Option<String>,
    /// License
    pub license: String,
    /// Plugin tags
    pub tags: Option<Vec<String>>,
    /// Plugin capabilities
    pub capabilities: Option<Vec<String>>,
    /// Required permissions
    pub permissions: Option<Vec<String>>,
    /// Entry point (relative path to main file/binary)
    pub entry_point: String,
    /// Minimum Devkit version required
    pub min_devkit_version: Option<String>,
    /// Maximum Devkit version supported
    pub max_devkit_version: Option<String>,
}

/// Raw plugin dependency from manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependencyRaw {
    /// Dependency ID
    pub id: String,
    /// Version requirement (semver)
    pub version: String,
    /// Whether dependency is optional
    pub optional: Option<bool>,
    /// Reason for dependency
    pub reason: Option<String>,
    /// Source of dependency
    pub source: Option<String>,
}

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Build command
    pub command: Option<String>,
    /// Build arguments
    pub args: Option<Vec<String>>,
    /// Build environment variables
    pub env: Option<HashMap<String, String>>,
    /// Pre-build scripts
    pub pre_build: Option<Vec<String>>,
    /// Post-build scripts
    pub post_build: Option<Vec<String>>,
    /// Output directory
    pub output_dir: Option<String>,
    /// Build target platform
    pub target: Option<String>,
}

/// Runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Runtime type (native, wasm, etc.)
    pub runtime_type: Option<String>,
    /// Environment variables
    pub env: Option<HashMap<String, String>>,
    /// Resource limits
    pub limits: Option<ResourceLimits>,
    /// Timeout configuration
    pub timeouts: Option<TimeoutConfig>,
    /// Security settings
    pub security: Option<SecurityConfig>,
}

/// Resource limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory in MB
    pub max_memory_mb: Option<u64>,
    /// Maximum CPU usage percentage
    pub max_cpu_percent: Option<f64>,
    /// Maximum file descriptors
    pub max_file_descriptors: Option<u32>,
    /// Maximum network connections
    pub max_network_connections: Option<u32>,
    /// Maximum disk usage in MB
    pub max_disk_usage_mb: Option<u64>,
}

/// Timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Initialization timeout in seconds
    pub init_timeout_seconds: Option<u64>,
    /// Execution timeout in seconds
    pub execution_timeout_seconds: Option<u64>,
    /// Shutdown timeout in seconds
    pub shutdown_timeout_seconds: Option<u64>,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable sandboxing
    pub enable_sandbox: Option<bool>,
    /// Allowed file paths
    pub allowed_paths: Option<Vec<String>>,
    /// Allowed network hosts
    pub allowed_hosts: Option<Vec<String>>,
    /// Allowed environment variables
    pub allowed_env_vars: Option<Vec<String>>,
}

/// Development configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevConfig {
    /// Enable hot reload
    pub hot_reload: Option<bool>,
    /// Watch paths for changes
    pub watch_paths: Option<Vec<String>>,
    /// Development server settings
    pub dev_server: Option<DevServerConfig>,
    /// Testing configuration
    pub testing: Option<TestConfig>,
}

/// Development server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevServerConfig {
    /// Server port
    pub port: Option<u16>,
    /// Server host
    pub host: Option<String>,
    /// Auto-restart on changes
    pub auto_restart: Option<bool>,
}

/// Testing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    /// Test command
    pub command: Option<String>,
    /// Test arguments
    pub args: Option<Vec<String>>,
    /// Test environment variables
    pub env: Option<HashMap<String, String>>,
}

/// Manifest schema for validation
#[derive(Debug, Clone)]
pub struct ManifestSchema {
    /// Schema version
    pub version: String,
    /// Required fields
    pub required_fields: Vec<String>,
    /// Field validators
    pub validators: HashMap<String, FieldValidator>,
}

/// Field validator
#[derive(Debug, Clone)]
pub enum FieldValidator {
    /// String length validation
    StringLength { min: Option<usize>, max: Option<usize> },
    /// Regex pattern validation
    Pattern(String),
    /// Enum validation (allowed values)
    Enum(Vec<String>),
    /// Semver validation
    SemVer,
    /// URL validation
    Url,
    /// Custom validation function name
    Custom(String),
}

/// Manifest parsing result
#[derive(Debug, Clone)]
pub struct ManifestParseResult {
    /// Parsed plugin metadata
    pub metadata: PluginMetadata,
    /// Build configuration
    pub build_config: Option<BuildConfig>,
    /// Runtime configuration
    pub runtime_config: Option<RuntimeConfig>,
    /// Development configuration
    pub dev_config: Option<DevConfig>,
    /// Validation warnings
    pub warnings: Vec<String>,
}

impl PluginManifestParser {
    /// Create a new manifest parser
    pub fn new(validation_config: ManifestValidationConfig) -> Self {
        let mut parser = Self {
            validation_config,
            schema_registry: HashMap::new(),
        };
        
        // Register default schemas
        parser.register_default_schemas();
        parser
    }

    /// Parse a plugin manifest from a file
    pub async fn parse_manifest(&self, manifest_path: &Path) -> Result<ManifestParseResult, PluginError> {
        debug!("Parsing plugin manifest: {:?}", manifest_path);

        if !manifest_path.exists() {
            return Err(PluginError::InvalidManifest(
                format!("Manifest file not found: {:?}", manifest_path)
            ));
        }

        // Read manifest file
        let content = fs::read_to_string(manifest_path)
            .await
            .map_err(|e| PluginError::IoError(format!("Failed to read manifest: {}", e)))?;

        self.parse_manifest_content(&content, Some(manifest_path)).await
    }

    /// Parse a plugin manifest from content
    pub async fn parse_manifest_content(
        &self,
        content: &str,
        manifest_path: Option<&Path>,
    ) -> Result<ManifestParseResult, PluginError> {
        debug!("Parsing plugin manifest content");

        // Parse TOML
        let manifest: PluginManifest = toml::from_str(content)
            .map_err(|e| PluginError::InvalidManifest(format!("TOML parse error: {}", e)))?;

        // Validate manifest
        let mut warnings = Vec::new();
        self.validate_manifest(&manifest, &mut warnings)?;

        // Convert to internal format
        let metadata = self.convert_metadata(&manifest, manifest_path, &mut warnings)?;

        info!("Successfully parsed plugin manifest: {}", metadata.id);

        Ok(ManifestParseResult {
            metadata,
            build_config: manifest.build,
            runtime_config: manifest.runtime,
            dev_config: manifest.dev,
            warnings,
        })
    }

    /// Validate a manifest against schema and rules
    fn validate_manifest(&self, manifest: &PluginManifest, warnings: &mut Vec<String>) -> Result<(), PluginError> {
        let plugin = &manifest.plugin;

        // Basic validation
        if plugin.id.is_empty() {
            return Err(PluginError::InvalidManifest("Plugin ID cannot be empty".to_string()));
        }

        if plugin.name.is_empty() {
            return Err(PluginError::InvalidManifest("Plugin name cannot be empty".to_string()));
        }

        if plugin.version.is_empty() {
            return Err(PluginError::InvalidManifest("Plugin version cannot be empty".to_string()));
        }

        if plugin.entry_point.is_empty() {
            return Err(PluginError::InvalidManifest("Plugin entry point cannot be empty".to_string()));
        }

        // Length validation
        if plugin.name.len() > self.validation_config.max_name_length {
            return Err(PluginError::InvalidManifest(
                format!("Plugin name too long: {} > {}", 
                        plugin.name.len(), self.validation_config.max_name_length)
            ));
        }

        if plugin.description.len() > self.validation_config.max_description_length {
            return Err(PluginError::InvalidManifest(
                format!("Plugin description too long: {} > {}", 
                        plugin.description.len(), self.validation_config.max_description_length)
            ));
        }

        // License validation
        if self.validation_config.strict_validation && 
           !self.validation_config.allowed_licenses.is_empty() &&
           !self.validation_config.allowed_licenses.contains(&plugin.license) {
            warnings.push(format!("Unrecognized license: {}", plugin.license));
        }

        // Version validation
        if let Err(_) = semver::Version::parse(&plugin.version) {
            return Err(PluginError::InvalidManifest(
                format!("Invalid semver version: {}", plugin.version)
            ));
        }

        // URL validation
        if let Some(ref homepage) = plugin.homepage {
            if !self.is_valid_url(homepage) {
                warnings.push(format!("Invalid homepage URL: {}", homepage));
            }
        }

        if let Some(ref repository) = plugin.repository {
            if !self.is_valid_url(repository) {
                warnings.push(format!("Invalid repository URL: {}", repository));
            }
        }

        // Dependencies validation
        if let Some(ref dependencies) = manifest.dependencies {
            if dependencies.len() > self.validation_config.max_dependencies {
                return Err(PluginError::InvalidManifest(
                    format!("Too many dependencies: {} > {}", 
                            dependencies.len(), self.validation_config.max_dependencies)
                ));
            }

            for dep in dependencies {
                if dep.id.is_empty() {
                    return Err(PluginError::InvalidManifest(
                        "Dependency ID cannot be empty".to_string()
                    ));
                }
                if dep.version.is_empty() {
                    return Err(PluginError::InvalidManifest(
                        format!("Dependency {} version cannot be empty", dep.id)
                    ));
                }
            }
        }

        // Devkit version validation
        if let Some(ref min_version) = plugin.min_devkit_version {
            if let Err(_) = semver::Version::parse(min_version) {
                warnings.push(format!("Invalid min_devkit_version: {}", min_version));
            }
        }

        if let Some(ref max_version) = plugin.max_devkit_version {
            if let Err(_) = semver::Version::parse(max_version) {
                warnings.push(format!("Invalid max_devkit_version: {}", max_version));
            }
        }

        Ok(())
    }

    /// Convert raw manifest to internal metadata format
    fn convert_metadata(
        &self,
        manifest: &PluginManifest,
        manifest_path: Option<&Path>,
        warnings: &mut Vec<String>,
    ) -> Result<PluginMetadata, PluginError> {
        let plugin = &manifest.plugin;

        // Convert capabilities
        let capabilities = if let Some(ref caps) = plugin.capabilities {
            caps.iter()
                .map(|cap| self.parse_capability(cap))
                .collect::<Result<Vec<_>, _>>()?
        } else {
            Vec::new()
        };

        // Convert dependencies
        let dependencies = if let Some(ref deps) = manifest.dependencies {
            deps.iter()
                .map(|dep| PluginDependency {
                    id: dep.id.clone(),
                    version: dep.version.clone(),
                    optional: dep.optional.unwrap_or(false),
                    reason: dep.reason.clone(),
                })
                .collect()
        } else {
            Vec::new()
        };

        // Determine entry point path
        let entry_point = if let Some(base_path) = manifest_path.and_then(|p| p.parent()) {
            base_path.join(&plugin.entry_point).to_string_lossy().to_string()
        } else {
            plugin.entry_point.clone()
        };

        Ok(PluginMetadata {
            id: plugin.id.clone(),
            name: plugin.name.clone(),
            version: plugin.version.clone(),
            description: plugin.description.clone(),
            author: plugin.author.clone(),
            homepage: plugin.homepage.clone(),
            repository: plugin.repository.clone(),
            license: plugin.license.clone(),
            tags: plugin.tags.clone().unwrap_or_default(),
            dependencies,
            permissions: plugin.permissions.clone().unwrap_or_default(),
            entry_point,
            created_at: chrono::Utc::now(), // Will be updated from actual file/metadata
            updated_at: chrono::Utc::now(),
            capabilities,
        })
    }

    /// Parse capability string to enum
    fn parse_capability(&self, capability: &str) -> Result<PluginCapability, PluginError> {
        match capability.to_lowercase().as_str() {
            "code_analysis" | "code-analysis" => Ok(PluginCapability::CodeAnalysis),
            "code_generation" | "code-generation" => Ok(PluginCapability::CodeGeneration),
            "code_formatting" | "code-formatting" => Ok(PluginCapability::CodeFormatting),
            "completion" => Ok(PluginCapability::Completion),
            "diagnostics" => Ok(PluginCapability::Diagnostics),
            "version_control" | "version-control" => Ok(PluginCapability::VersionControl),
            "dependency_management" | "dependency-management" => Ok(PluginCapability::DependencyManagement),
            "testing" => Ok(PluginCapability::Testing),
            "documentation" => Ok(PluginCapability::Documentation),
            _ => Ok(PluginCapability::Custom(capability.to_string())),
        }
    }

    /// Validate URL format
    fn is_valid_url(&self, url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://") || url.starts_with("git://")
    }

    /// Register default manifest schemas
    fn register_default_schemas(&mut self) {
        // Register v1.0 schema
        let v1_schema = ManifestSchema {
            version: "1.0".to_string(),
            required_fields: vec![
                "plugin.id".to_string(),
                "plugin.name".to_string(),
                "plugin.version".to_string(),
                "plugin.description".to_string(),
                "plugin.author".to_string(),
                "plugin.license".to_string(),
                "plugin.entry_point".to_string(),
            ],
            validators: HashMap::new(), // TODO: Implement field validators
        };
        
        self.schema_registry.insert("1.0".to_string(), v1_schema);
    }

    /// Generate a template manifest
    pub fn generate_template(&self, plugin_id: &str, plugin_name: &str) -> String {
        format!(r#"# Plugin Manifest
# This file defines the plugin metadata and configuration

[plugin]
id = "{}"
name = "{}"
version = "0.1.0"
description = "A plugin for Devkit"
author = "Your Name <your.email@example.com>"
homepage = "https://github.com/yourusername/{}"
repository = "https://github.com/yourusername/{}.git"
license = "MIT"
tags = ["devkit", "plugin"]
capabilities = ["code_analysis"]
permissions = ["filesystem_read"]
entry_point = "src/main.rs"
min_devkit_version = "0.1.0"

# Dependencies (optional)
# [[dependencies]]
# id = "some-plugin"
# version = "^1.0.0"
# optional = false
# reason = "Required for core functionality"

# Build configuration (optional)
[build]
command = "cargo build --release"
output_dir = "target/release"

# Runtime configuration (optional)
[runtime]
runtime_type = "native"

[runtime.limits]
max_memory_mb = 256
max_cpu_percent = 50.0

[runtime.timeouts]
init_timeout_seconds = 30
execution_timeout_seconds = 300

[runtime.security]
enable_sandbox = true
allowed_paths = ["./data", "./config"]

# Development configuration (optional)
[dev]
hot_reload = true
watch_paths = ["src/", "assets/"]

[dev.testing]
command = "cargo test"
"#, plugin_id, plugin_name, plugin_id, plugin_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_manifest_template_generation() {
        let parser = PluginManifestParser::new(ManifestValidationConfig::default());
        let template = parser.generate_template("test-plugin", "Test Plugin");
        
        assert!(template.contains("test-plugin"));
        assert!(template.contains("Test Plugin"));
        assert!(template.contains("[plugin]"));
    }

    #[tokio::test]
    async fn test_parse_valid_manifest() {
        let manifest_content = r#"
[plugin]
id = "test-plugin"
name = "Test Plugin"
version = "1.0.0"
description = "A test plugin"
author = "Test Author"
license = "MIT"
entry_point = "main.rs"
capabilities = ["code_analysis"]

[[dependencies]]
id = "dep-plugin"
version = "^1.0.0"
optional = false
"#;

        let parser = PluginManifestParser::new(ManifestValidationConfig::default());
        let result = parser.parse_manifest_content(manifest_content, None).await;
        
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.metadata.id, "test-plugin");
        assert_eq!(parsed.metadata.name, "Test Plugin");
        assert_eq!(parsed.metadata.dependencies.len(), 1);
    }

    #[tokio::test]
    async fn test_parse_invalid_manifest() {
        let manifest_content = r#"
[plugin]
# Missing required fields
name = "Test Plugin"
"#;

        let parser = PluginManifestParser::new(ManifestValidationConfig::default());
        let result = parser.parse_manifest_content(manifest_content, None).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_manifest_validation_warnings() {
        let manifest_content = r#"
[plugin]
id = "test-plugin"
name = "Test Plugin"
version = "1.0.0"
description = "A test plugin"
author = "Test Author"
license = "CUSTOM_LICENSE"  # This should generate a warning
entry_point = "main.rs"
homepage = "not-a-valid-url"  # This should generate a warning
"#;

        let parser = PluginManifestParser::new(ManifestValidationConfig::default());
        let result = parser.parse_manifest_content(manifest_content, None).await;
        
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(parsed.warnings.len() > 0);
        assert!(parsed.warnings.iter().any(|w| w.contains("license")));
        assert!(parsed.warnings.iter().any(|w| w.contains("homepage")));
    }

    #[tokio::test]
    async fn test_capability_parsing() {
        let parser = PluginManifestParser::new(ManifestValidationConfig::default());
        
        assert!(matches!(
            parser.parse_capability("code_analysis").unwrap(),
            PluginCapability::CodeAnalysis
        ));
        
        assert!(matches!(
            parser.parse_capability("custom-capability").unwrap(),
            PluginCapability::Custom(_)
        ));
    }

    #[tokio::test]
    async fn test_manifest_file_parsing() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("plugin.toml");
        
        let manifest_content = r#"
[plugin]
id = "file-test-plugin"
name = "File Test Plugin"
version = "1.0.0"
description = "Testing file parsing"
author = "Test Author"
license = "MIT"
entry_point = "main.rs"
"#;

        fs::write(&manifest_path, manifest_content).await.unwrap();

        let parser = PluginManifestParser::new(ManifestValidationConfig::default());
        let result = parser.parse_manifest(&manifest_path).await;
        
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.metadata.id, "file-test-plugin");
    }
}