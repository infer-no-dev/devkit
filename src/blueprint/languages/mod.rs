//! Multi-Language Blueprint Support
//!
//! This module extends the blueprint system to support multiple programming languages,
//! enabling cross-language analysis, generation, and replication.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub mod analyzers;

/// Supported programming languages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    CSharp,
    Cpp,
    C,
    Shell,
    Docker,
    Yaml,
}

impl Language {
    /// Get language from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "rs" => Some(Language::Rust),
            "py" | "pyx" | "pyi" => Some(Language::Python),
            "js" | "jsx" => Some(Language::JavaScript),
            "ts" | "tsx" => Some(Language::TypeScript),
            "go" => Some(Language::Go),
            "java" => Some(Language::Java),
            "cs" => Some(Language::CSharp),
            "cpp" | "cc" | "cxx" | "c++" => Some(Language::Cpp),
            "c" | "h" => Some(Language::C),
            "sh" | "bash" | "zsh" => Some(Language::Shell),
            "dockerfile" => Some(Language::Docker),
            "yml" | "yaml" => Some(Language::Yaml),
            _ => None,
        }
    }

    /// Get typical file extensions for this language
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Language::Rust => &["rs"],
            Language::Python => &["py", "pyx", "pyi"],
            Language::JavaScript => &["js", "jsx"],
            Language::TypeScript => &["ts", "tsx"],
            Language::Go => &["go"],
            Language::Java => &["java"],
            Language::CSharp => &["cs"],
            Language::Cpp => &["cpp", "cc", "cxx", "c++"],
            Language::C => &["c", "h"],
            Language::Shell => &["sh", "bash", "zsh"],
            Language::Docker => &["dockerfile"],
            Language::Yaml => &["yml", "yaml"],
        }
    }

    /// Get the primary package manager for this language
    pub fn package_manager(&self) -> &'static str {
        match self {
            Language::Rust => "cargo",
            Language::Python => "pip",
            Language::JavaScript => "npm",
            Language::TypeScript => "npm",
            Language::Go => "go mod",
            Language::Java => "maven",
            Language::CSharp => "nuget",
            Language::Cpp => "vcpkg",
            Language::C => "pkg-config",
            Language::Shell => "none",
            Language::Docker => "none",
            Language::Yaml => "none",
        }
    }

    /// Get the build system for this language
    pub fn build_system(&self) -> &'static str {
        match self {
            Language::Rust => "cargo",
            Language::Python => "setuptools",
            Language::JavaScript => "webpack",
            Language::TypeScript => "tsc",
            Language::Go => "go build",
            Language::Java => "maven",
            Language::CSharp => "dotnet",
            Language::Cpp => "cmake",
            Language::C => "make",
            Language::Shell => "none",
            Language::Docker => "docker",
            Language::Yaml => "none",
        }
    }
}

/// Multi-language project structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiLanguageBlueprint {
    pub primary_language: Language,
    pub secondary_languages: Vec<Language>,
    pub language_modules: HashMap<Language, LanguageModule>,
    pub inter_language_interfaces: Vec<InterfaceBinding>,
    pub build_orchestration: BuildOrchestration,
    pub deployment_strategy: DeploymentStrategy,
}

/// Language-specific module information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageModule {
    pub language: Language,
    pub entry_points: Vec<String>,
    pub dependencies: Vec<Dependency>,
    pub build_config: BuildConfig,
    pub test_config: TestConfig,
    pub documentation: DocumentationConfig,
}

/// Cross-language interface binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceBinding {
    pub source_language: Language,
    pub target_language: Language,
    pub binding_type: BindingType,
    pub interface_definition: String,
    pub generation_strategy: String,
}

/// Types of cross-language bindings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BindingType {
    FFI,          // Foreign Function Interface
    WebAPI,       // REST/GraphQL API
    MessageQueue, // Message passing
    SharedMemory, // Shared memory
    RPC,          // Remote Procedure Call
    CLI,          // Command line interface
}

/// Dependency information for any language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub source: DependencySource,
    pub purpose: String,
    pub optional: bool,
}

/// Source of dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencySource {
    Registry(String), // e.g., "crates.io", "pypi", "npm"
    Git(String),      // Git repository URL
    Local(String),    // Local path
    System,           // System package
}

/// Build configuration for a language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub build_tool: String,
    pub build_file: String, // e.g., "Cargo.toml", "package.json", "pom.xml"
    pub compile_flags: Vec<String>,
    pub optimization_level: String,
    pub target_platforms: Vec<String>,
}

/// Test configuration for a language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub test_framework: String,
    pub test_directories: Vec<String>,
    pub coverage_tool: String,
    pub test_commands: Vec<String>,
}

/// Documentation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationConfig {
    pub doc_tool: String,   // e.g., "rustdoc", "sphinx", "jsdoc"
    pub doc_format: String, // e.g., "html", "markdown"
    pub doc_directory: String,
    pub auto_generate: bool,
}

/// Build orchestration for multi-language projects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildOrchestration {
    pub orchestration_tool: String, // e.g., "make", "bazel", "gradle"
    pub build_order: Vec<Language>,
    pub parallel_builds: bool,
    pub shared_artifacts: Vec<String>,
    pub cross_language_validation: bool,
}

/// Deployment strategy for multi-language systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentStrategy {
    pub containerization: ContainerConfig,
    pub orchestration: OrchestrationConfig,
    pub monitoring: MonitoringConfig,
}

/// Container configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    pub base_images: HashMap<Language, String>,
    pub multi_stage_build: bool,
    pub layer_optimization: bool,
    pub security_scanning: bool,
}

/// Orchestration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationConfig {
    pub platform: String, // "kubernetes", "docker-compose", "nomad"
    pub service_mesh: Option<String>,
    pub auto_scaling: bool,
    pub health_checks: Vec<String>,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_collection: String,
    pub log_aggregation: String,
    pub tracing: String,
    pub alerting: String,
}

/// Multi-language project analyzer
pub struct MultiLanguageAnalyzer {
    analyzers: HashMap<Language, Arc<dyn LanguageAnalyzer>>,
}

/// Trait for language-specific analyzers
#[async_trait]
pub trait LanguageAnalyzer: Send + Sync {
    /// Analyze a project in this language
    async fn analyze(&self, project_path: &Path) -> Result<LanguageModule>;

    /// Extract dependencies
    async fn extract_dependencies(&self, project_path: &Path) -> Result<Vec<Dependency>>;

    /// Analyze build configuration
    async fn analyze_build_config(&self, project_path: &Path) -> Result<BuildConfig>;

    /// Extract API interfaces
    async fn extract_interfaces(&self, project_path: &Path) -> Result<Vec<String>>;
}

impl MultiLanguageAnalyzer {
    /// Create a new multi-language analyzer
    pub fn new() -> Self {
        let mut analyzers = HashMap::new();

        // Register concrete analyzers
        analyzers.insert(
            Language::Rust,
            Arc::new(analyzers::RustAnalyzer::new()) as Arc<dyn LanguageAnalyzer>,
        );
        analyzers.insert(
            Language::Python,
            Arc::new(analyzers::PythonAnalyzer::new()) as Arc<dyn LanguageAnalyzer>,
        );
        analyzers.insert(
            Language::JavaScript,
            Arc::new(analyzers::JavaScriptAnalyzer::new()) as Arc<dyn LanguageAnalyzer>,
        );
        analyzers.insert(
            Language::TypeScript,
            Arc::new(analyzers::JavaScriptAnalyzer::new()) as Arc<dyn LanguageAnalyzer>,
        );

        Self { analyzers }
    }

    /// Register a language analyzer
    pub fn register_analyzer(&mut self, language: Language, analyzer: Arc<dyn LanguageAnalyzer>) {
        self.analyzers.insert(language, analyzer);
    }

    /// Detect languages in a project
    pub async fn detect_languages(&self, project_path: &Path) -> Result<Vec<Language>> {
        let mut languages = Vec::new();
        let mut language_files: HashMap<Language, usize> = HashMap::new();

        // Check for language-specific project files first
        if project_path.join("Cargo.toml").exists() {
            languages.push(Language::Rust);
        }

        if project_path.join("package.json").exists() {
            // Determine if JavaScript or TypeScript
            if project_path.join("tsconfig.json").exists() {
                languages.push(Language::TypeScript);
            } else {
                languages.push(Language::JavaScript);
            }
        }

        if project_path.join("requirements.txt").exists()
            || project_path.join("setup.py").exists()
            || project_path.join("pyproject.toml").exists()
        {
            languages.push(Language::Python);
        }

        if project_path.join("go.mod").exists() {
            languages.push(Language::Go);
        }

        // If we found languages via project files, return those
        if !languages.is_empty() {
            return Ok(languages);
        }

        // Fallback: Walk through project files
        let walker = walkdir::WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file());

        for entry in walker {
            if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
                if let Some(lang) = Language::from_extension(ext) {
                    *language_files.entry(lang).or_insert(0) += 1;
                }
            }
        }

        // Sort by file count to determine primary/secondary languages
        let mut sorted_languages: Vec<_> = language_files.into_iter().collect();
        sorted_languages.sort_by(|a, b| b.1.cmp(&a.1));

        for (lang, count) in sorted_languages {
            if count >= 1 {
                // Lower threshold for file-based detection
                languages.push(lang);
            }
        }

        Ok(languages)
    }

    /// Analyze a multi-language project
    pub async fn analyze_project(&self, project_path: &Path) -> Result<MultiLanguageBlueprint> {
        let languages = self.detect_languages(project_path).await?;

        if languages.is_empty() {
            return Err(anyhow::anyhow!(
                "No supported languages detected in project"
            ));
        }

        let primary_language = languages[0].clone();
        let secondary_languages = languages[1..].to_vec();

        let mut language_modules = HashMap::new();

        // Analyze each detected language
        for language in &languages {
            if let Some(analyzer) = self.analyzers.get(language) {
                let module = analyzer.analyze(project_path).await?;
                language_modules.insert(language.clone(), module);
            }
        }

        // Detect cross-language interfaces
        let interfaces = self.detect_interfaces(&languages, project_path).await?;

        // Analyze build orchestration
        let build_orchestration = self
            .analyze_build_orchestration(&languages, project_path)
            .await?;

        // Generate deployment strategy
        let deployment_strategy = self.generate_deployment_strategy(&languages).await?;

        Ok(MultiLanguageBlueprint {
            primary_language,
            secondary_languages,
            language_modules,
            inter_language_interfaces: interfaces,
            build_orchestration,
            deployment_strategy,
        })
    }

    /// Detect cross-language interfaces
    async fn detect_interfaces(
        &self,
        languages: &[Language],
        project_path: &Path,
    ) -> Result<Vec<InterfaceBinding>> {
        let mut interfaces = Vec::new();

        // Look for common interface patterns
        for source_lang in languages {
            for target_lang in languages {
                if source_lang != target_lang {
                    // Check for FFI bindings
                    if self
                        .has_ffi_bindings(source_lang, target_lang, project_path)
                        .await?
                    {
                        interfaces.push(InterfaceBinding {
                            source_language: source_lang.clone(),
                            target_language: target_lang.clone(),
                            binding_type: BindingType::FFI,
                            interface_definition: "FFI bindings detected".to_string(),
                            generation_strategy: "auto".to_string(),
                        });
                    }

                    // Check for API interfaces
                    if self
                        .has_api_interface(source_lang, target_lang, project_path)
                        .await?
                    {
                        interfaces.push(InterfaceBinding {
                            source_language: source_lang.clone(),
                            target_language: target_lang.clone(),
                            binding_type: BindingType::WebAPI,
                            interface_definition: "Web API interface detected".to_string(),
                            generation_strategy: "openapi".to_string(),
                        });
                    }
                }
            }
        }

        Ok(interfaces)
    }

    /// Check for FFI bindings between languages
    async fn has_ffi_bindings(
        &self,
        _source: &Language,
        _target: &Language,
        _path: &Path,
    ) -> Result<bool> {
        // TODO: Implement FFI detection logic
        Ok(false)
    }

    /// Check for API interfaces
    async fn has_api_interface(
        &self,
        _source: &Language,
        _target: &Language,
        _path: &Path,
    ) -> Result<bool> {
        // TODO: Implement API detection logic
        Ok(false)
    }

    /// Analyze build orchestration
    async fn analyze_build_orchestration(
        &self,
        languages: &[Language],
        project_path: &Path,
    ) -> Result<BuildOrchestration> {
        // Check for existing orchestration tools
        let orchestration_tool = if project_path.join("Makefile").exists() {
            "make".to_string()
        } else if project_path.join("BUILD").exists() || project_path.join("BUILD.bazel").exists() {
            "bazel".to_string()
        } else if project_path.join("build.gradle").exists() {
            "gradle".to_string()
        } else {
            "make".to_string() // Default fallback
        };

        // Determine build order based on dependencies
        let build_order = self.determine_build_order(languages).await?;

        Ok(BuildOrchestration {
            orchestration_tool,
            build_order,
            parallel_builds: languages.len() > 1,
            shared_artifacts: vec!["target/".to_string(), "dist/".to_string()],
            cross_language_validation: true,
        })
    }

    /// Determine optimal build order for languages
    async fn determine_build_order(&self, languages: &[Language]) -> Result<Vec<Language>> {
        // Simple heuristic: compile-time languages first, then runtime languages
        let mut ordered = languages.to_vec();
        ordered.sort_by_key(|lang| match lang {
            Language::C | Language::Cpp | Language::Rust | Language::Go => 0,
            Language::Java | Language::CSharp => 1,
            Language::TypeScript => 2,
            Language::JavaScript | Language::Python => 3,
            _ => 4,
        });

        Ok(ordered)
    }

    /// Generate deployment strategy
    async fn generate_deployment_strategy(
        &self,
        languages: &[Language],
    ) -> Result<DeploymentStrategy> {
        // Select appropriate base images for each language
        let mut base_images = HashMap::new();
        for lang in languages {
            let image = match lang {
                Language::Rust => "rust:alpine".to_string(),
                Language::Python => "python:3.11-slim".to_string(),
                Language::JavaScript => "node:18-alpine".to_string(),
                Language::TypeScript => "node:18-alpine".to_string(),
                Language::Go => "golang:alpine".to_string(),
                Language::Java => "openjdk:17-alpine".to_string(),
                Language::CSharp => "mcr.microsoft.com/dotnet/runtime:7.0-alpine".to_string(),
                _ => "alpine:latest".to_string(),
            };
            base_images.insert(lang.clone(), image);
        }

        Ok(DeploymentStrategy {
            containerization: ContainerConfig {
                base_images,
                multi_stage_build: true,
                layer_optimization: true,
                security_scanning: true,
            },
            orchestration: OrchestrationConfig {
                platform: "kubernetes".to_string(),
                service_mesh: Some("istio".to_string()),
                auto_scaling: true,
                health_checks: vec!["/health".to_string(), "/ready".to_string()],
            },
            monitoring: MonitoringConfig {
                metrics_collection: "prometheus".to_string(),
                log_aggregation: "elasticsearch".to_string(),
                tracing: "jaeger".to_string(),
                alerting: "alertmanager".to_string(),
            },
        })
    }
}

impl Default for MultiLanguageAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_extension() {
        assert_eq!(Language::from_extension("rs"), Some(Language::Rust));
        assert_eq!(Language::from_extension("py"), Some(Language::Python));
        assert_eq!(Language::from_extension("js"), Some(Language::JavaScript));
        assert_eq!(Language::from_extension("go"), Some(Language::Go));
        assert_eq!(Language::from_extension("unknown"), None);
    }

    #[test]
    fn test_language_properties() {
        assert_eq!(Language::Rust.package_manager(), "cargo");
        assert_eq!(Language::Python.package_manager(), "pip");
        assert_eq!(Language::JavaScript.build_system(), "webpack");

        assert!(Language::Rust.extensions().contains(&"rs"));
        assert!(Language::Python.extensions().contains(&"py"));
    }
}
