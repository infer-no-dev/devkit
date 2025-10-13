//! Rust Language Analyzer
//!
//! Analyzes Rust projects to extract module structure, dependencies,
//! and build configuration for blueprint generation.

use super::super::{
    BuildConfig, Dependency, DependencySource, DocumentationConfig, Language, LanguageAnalyzer,
    LanguageModule, TestConfig,
};
use anyhow::Result;
use std::path::Path;

pub struct RustAnalyzer;

impl RustAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Parse Cargo.toml dependencies
    async fn parse_cargo_toml(&self, project_path: &Path) -> Result<Vec<Dependency>> {
        let cargo_path = project_path.join("Cargo.toml");
        if !cargo_path.exists() {
            return Ok(vec![]);
        }

        let content = tokio::fs::read_to_string(&cargo_path).await?;
        let cargo_toml: toml::Value = toml::from_str(&content)?;

        let mut dependencies = Vec::new();

        // Parse [dependencies]
        if let Some(deps) = cargo_toml.get("dependencies").and_then(|d| d.as_table()) {
            for (name, dep_spec) in deps {
                let (version, optional) = match dep_spec {
                    toml::Value::String(v) => (v.clone(), false),
                    toml::Value::Table(t) => {
                        let version = t
                            .get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("*")
                            .to_string();
                        let optional = t.get("optional").and_then(|o| o.as_bool()).unwrap_or(false);
                        (version, optional)
                    }
                    _ => ("*".to_string(), false),
                };

                dependencies.push(Dependency {
                    name: name.clone(),
                    version,
                    source: DependencySource::Registry("crates.io".to_string()),
                    purpose: self.infer_dependency_purpose(name),
                    optional,
                });
            }
        }

        // Parse [dev-dependencies]
        if let Some(dev_deps) = cargo_toml
            .get("dev-dependencies")
            .and_then(|d| d.as_table())
        {
            for (name, dep_spec) in dev_deps {
                let version = match dep_spec {
                    toml::Value::String(v) => v.clone(),
                    toml::Value::Table(t) => t
                        .get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("*")
                        .to_string(),
                    _ => "*".to_string(),
                };

                dependencies.push(Dependency {
                    name: name.clone(),
                    version,
                    source: DependencySource::Registry("crates.io".to_string()),
                    purpose: self.infer_dependency_purpose(name),
                    optional: true, // All dev-dependencies are optional
                });
            }
        }

        // Parse [build-dependencies]
        if let Some(build_deps) = cargo_toml
            .get("build-dependencies")
            .and_then(|d| d.as_table())
        {
            for (name, dep_spec) in build_deps {
                let version = match dep_spec {
                    toml::Value::String(v) => v.clone(),
                    toml::Value::Table(t) => t
                        .get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("*")
                        .to_string(),
                    _ => "*".to_string(),
                };

                dependencies.push(Dependency {
                    name: name.clone(),
                    version,
                    source: DependencySource::Registry("crates.io".to_string()),
                    purpose: format!("Build dependency: {}", self.infer_dependency_purpose(name)),
                    optional: true,
                });
            }
        }

        Ok(dependencies)
    }

    /// Find entry points from Cargo.toml
    async fn find_entry_points(&self, project_path: &Path) -> Result<Vec<String>> {
        let mut entry_points = Vec::new();
        let cargo_path = project_path.join("Cargo.toml");

        if cargo_path.exists() {
            let content = tokio::fs::read_to_string(&cargo_path).await?;
            let cargo_toml: toml::Value = toml::from_str(&content)?;

            // Main library entry point
            if let Some(lib) = cargo_toml.get("lib") {
                if let Some(lib_table) = lib.as_table() {
                    if let Some(path) = lib_table.get("path").and_then(|p| p.as_str()) {
                        entry_points.push(path.to_string());
                    } else {
                        entry_points.push("src/lib.rs".to_string());
                    }
                }
            } else if project_path.join("src/lib.rs").exists() {
                entry_points.push("src/lib.rs".to_string());
            }

            // Binary entry points
            if let Some(bins) = cargo_toml.get("bin") {
                match bins {
                    toml::Value::Array(bin_array) => {
                        for bin in bin_array {
                            if let Some(bin_table) = bin.as_table() {
                                if let Some(path) = bin_table.get("path").and_then(|p| p.as_str()) {
                                    entry_points.push(path.to_string());
                                } else if let Some(name) =
                                    bin_table.get("name").and_then(|n| n.as_str())
                                {
                                    entry_points.push(format!("src/bin/{}.rs", name));
                                }
                            }
                        }
                    }
                    toml::Value::Table(bin_table) => {
                        if let Some(path) = bin_table.get("path").and_then(|p| p.as_str()) {
                            entry_points.push(path.to_string());
                        }
                    }
                    _ => {}
                }
            }

            // Default binary entry point
            if project_path.join("src/main.rs").exists() {
                entry_points.push("src/main.rs".to_string());
            }

            // Check for bins directory
            let bins_dir = project_path.join("src/bin");
            if bins_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&bins_dir) {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.ends_with(".rs") {
                                entry_points.push(format!("src/bin/{}", name));
                            }
                        }
                    }
                }
            }
        }

        if entry_points.is_empty() {
            // Default assumptions
            if project_path.join("src/main.rs").exists() {
                entry_points.push("src/main.rs".to_string());
            } else if project_path.join("src/lib.rs").exists() {
                entry_points.push("src/lib.rs".to_string());
            } else {
                entry_points.push("src/main.rs".to_string()); // Assumption
            }
        }

        entry_points.sort();
        entry_points.dedup();
        Ok(entry_points)
    }

    /// Infer dependency purpose based on crate name
    fn infer_dependency_purpose(&self, name: &str) -> String {
        match name {
            // Web frameworks
            "actix-web" | "axum" | "warp" | "tide" | "rocket" => "Web framework".to_string(),

            // Async runtimes
            "tokio" | "async-std" | "smol" => "Async runtime".to_string(),

            // Serialization
            "serde" | "serde_json" | "toml" | "bincode" => "Serialization".to_string(),

            // Database
            "sqlx" | "diesel" | "sea-orm" | "rusqlite" => "Database".to_string(),

            // HTTP clients
            "reqwest" | "surf" | "ureq" => "HTTP client".to_string(),

            // CLI
            "clap" | "structopt" | "argh" => "CLI framework".to_string(),

            // Logging
            "log" | "env_logger" | "tracing" | "slog" => "Logging".to_string(),

            // Error handling
            "anyhow" | "thiserror" | "eyre" => "Error handling".to_string(),

            // Testing
            "assert_cmd" | "proptest" | "quickcheck" | "mockall" => "Testing".to_string(),

            // Crypto
            "ring" | "rustls" | "openssl" | "sha2" | "rand" => "Cryptography".to_string(),

            // Parsing
            "nom" | "pest" | "lalrpop" | "regex" => "Parsing".to_string(),

            // Utility
            "once_cell" | "lazy_static" | "parking_lot" | "rayon" => "Utility".to_string(),

            _ => "Application dependency".to_string(),
        }
    }

    /// Detect test framework
    fn detect_test_framework(&self, _project_path: &Path) -> String {
        // Rust has built-in testing, but check for additional frameworks
        "built-in".to_string() // Rust's built-in test framework
    }

    /// Analyze build configuration
    async fn analyze_rust_build_config(&self, project_path: &Path) -> Result<BuildConfig> {
        let mut compile_flags = Vec::new();

        // Check for common Rust flags in .cargo/config.toml
        let cargo_config_path = project_path.join(".cargo/config.toml");
        if cargo_config_path.exists() {
            let content = tokio::fs::read_to_string(&cargo_config_path).await?;
            if content.contains("target-cpu") {
                compile_flags.push("--target-cpu=native".to_string());
            }
        }

        // Check Cargo.toml for optimization settings
        let cargo_path = project_path.join("Cargo.toml");
        if cargo_path.exists() {
            let content = tokio::fs::read_to_string(&cargo_path).await?;
            let cargo_toml: toml::Value = toml::from_str(&content)?;

            if let Some(profile) = cargo_toml.get("profile") {
                if profile.get("release").is_some() {
                    compile_flags.push("--release".to_string());
                }
            }
        }

        Ok(BuildConfig {
            build_tool: "cargo".to_string(),
            build_file: "Cargo.toml".to_string(),
            compile_flags,
            optimization_level: "release".to_string(),
            target_platforms: vec![
                "x86_64-unknown-linux-gnu".to_string(),
                "x86_64-pc-windows-msvc".to_string(),
                "x86_64-apple-darwin".to_string(),
                "aarch64-apple-darwin".to_string(),
            ],
        })
    }
}

#[async_trait::async_trait]
impl LanguageAnalyzer for RustAnalyzer {
    async fn analyze(&self, project_path: &Path) -> Result<LanguageModule> {
        let dependencies = self.extract_dependencies(project_path).await?;
        let entry_points = self.find_entry_points(project_path).await?;
        let build_config = self.analyze_build_config(project_path).await?;
        let test_framework = self.detect_test_framework(project_path);

        let test_config = TestConfig {
            test_framework,
            test_directories: vec!["tests/".to_string()],
            coverage_tool: "tarpaulin".to_string(),
            test_commands: vec!["cargo test".to_string()],
        };

        let documentation = DocumentationConfig {
            doc_tool: "rustdoc".to_string(),
            doc_format: "html".to_string(),
            doc_directory: "target/doc/".to_string(),
            auto_generate: true,
        };

        Ok(LanguageModule {
            language: Language::Rust,
            entry_points,
            dependencies,
            build_config,
            test_config,
            documentation,
        })
    }

    async fn extract_dependencies(&self, project_path: &Path) -> Result<Vec<Dependency>> {
        self.parse_cargo_toml(project_path).await
    }

    async fn analyze_build_config(&self, project_path: &Path) -> Result<BuildConfig> {
        self.analyze_rust_build_config(project_path).await
    }

    async fn extract_interfaces(&self, project_path: &Path) -> Result<Vec<String>> {
        let mut interfaces = Vec::new();
        let deps = self.extract_dependencies(project_path).await?;

        // Infer interfaces from dependencies
        for dep in &deps {
            match dep.name.as_str() {
                "actix-web" | "axum" | "warp" | "tide" | "rocket" => {
                    interfaces.push("REST API".to_string());
                }
                "clap" | "structopt" | "argh" => {
                    interfaces.push("CLI Application".to_string());
                }
                _ => {}
            }
        }

        // Check entry points
        let entry_points = self.find_entry_points(project_path).await?;
        for entry_point in &entry_points {
            if entry_point.contains("main.rs") || entry_point.contains("bin/") {
                if !interfaces.contains(&"CLI Application".to_string()) {
                    interfaces.push("CLI Application".to_string());
                }
            }
            if entry_point.contains("lib.rs") {
                interfaces.push("Rust Library".to_string());
            }
        }

        if interfaces.is_empty() {
            interfaces.push("Rust Application".to_string());
        }

        interfaces.dedup();
        Ok(interfaces)
    }
}

impl Default for RustAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_cargo_toml_parsing() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create Cargo.toml
        let cargo_toml = r#"
[package]
name = "test-project"
version = "0.1.0"

[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }

[dev-dependencies]
assert_cmd = "2.0"
"#;

        let mut file = File::create(temp_path.join("Cargo.toml")).await.unwrap();
        file.write_all(cargo_toml.as_bytes()).await.unwrap();

        let analyzer = RustAnalyzer::new();
        let deps = analyzer.parse_cargo_toml(temp_path).await.unwrap();

        assert_eq!(deps.len(), 3);
        assert!(deps.iter().any(|d| d.name == "serde" && !d.optional));
        assert!(deps.iter().any(|d| d.name == "tokio" && !d.optional));
        assert!(deps.iter().any(|d| d.name == "assert_cmd" && d.optional));
    }

    #[tokio::test]
    async fn test_entry_point_detection() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create src directory and main.rs
        tokio::fs::create_dir_all(temp_path.join("src"))
            .await
            .unwrap();
        let mut main_rs = File::create(temp_path.join("src/main.rs")).await.unwrap();
        main_rs.write_all(b"fn main() {}").await.unwrap();

        let analyzer = RustAnalyzer::new();
        let entry_points = analyzer.find_entry_points(temp_path).await.unwrap();

        assert!(entry_points.contains(&"src/main.rs".to_string()));
    }
}
