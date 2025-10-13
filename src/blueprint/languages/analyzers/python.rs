//! Python Language Analyzer
//!
//! Analyzes Python projects to extract module structure, dependencies,
//! and build configuration for blueprint generation.

use super::super::{
    BuildConfig, Dependency, DependencySource, DocumentationConfig, Language, LanguageAnalyzer,
    LanguageModule, TestConfig,
};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

pub struct PythonAnalyzer;

impl PythonAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Parse requirements.txt file
    async fn parse_requirements(&self, project_path: &Path) -> Result<Vec<Dependency>> {
        let mut dependencies = Vec::new();

        let requirements_files = [
            "requirements.txt",
            "requirements-dev.txt",
            "requirements-test.txt",
            "dev-requirements.txt",
        ];

        for req_file in &requirements_files {
            let req_path = project_path.join(req_file);
            if req_path.exists() {
                let content = tokio::fs::read_to_string(&req_path).await?;

                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }

                    let (name, version) = if line.contains("==") {
                        let parts: Vec<&str> = line.split("==").collect();
                        (
                            parts[0].to_string(),
                            parts.get(1).unwrap_or(&"*").to_string(),
                        )
                    } else if line.contains(">=") {
                        let parts: Vec<&str> = line.split(">=").collect();
                        (
                            parts[0].to_string(),
                            format!(">={}", parts.get(1).unwrap_or(&"0")),
                        )
                    } else {
                        (line.to_string(), "*".to_string())
                    };

                    dependencies.push(Dependency {
                        name: name.clone(),
                        version,
                        source: DependencySource::Registry("pypi".to_string()),
                        purpose: self.infer_dependency_purpose(&name),
                        optional: req_file.contains("dev") || req_file.contains("test"),
                    });
                }
            }
        }

        Ok(dependencies)
    }

    /// Parse setup.py or setup.cfg
    async fn parse_setup_files(&self, project_path: &Path) -> Result<Vec<Dependency>> {
        let mut dependencies = Vec::new();

        // Check setup.py
        let setup_py = project_path.join("setup.py");
        if setup_py.exists() {
            let content = tokio::fs::read_to_string(&setup_py).await?;
            dependencies.extend(self.extract_setup_py_deps(&content)?);
        }

        // Check setup.cfg
        let setup_cfg = project_path.join("setup.cfg");
        if setup_cfg.exists() {
            let content = tokio::fs::read_to_string(&setup_cfg).await?;
            dependencies.extend(self.extract_setup_cfg_deps(&content)?);
        }

        Ok(dependencies)
    }

    /// Parse pyproject.toml
    async fn parse_pyproject_toml(&self, project_path: &Path) -> Result<Vec<Dependency>> {
        let pyproject_path = project_path.join("pyproject.toml");
        if !pyproject_path.exists() {
            return Ok(vec![]);
        }

        let content = tokio::fs::read_to_string(&pyproject_path).await?;
        let parsed: toml::Value = toml::from_str(&content)?;

        let mut dependencies = Vec::new();

        // Poetry dependencies
        if let Some(poetry) = parsed.get("tool").and_then(|t| t.get("poetry")) {
            if let Some(deps) = poetry.get("dependencies").and_then(|d| d.as_table()) {
                for (name, version_spec) in deps {
                    if name == "python" {
                        continue; // Skip Python version requirement
                    }

                    let version = match version_spec {
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
                        source: DependencySource::Registry("pypi".to_string()),
                        purpose: self.infer_dependency_purpose(name),
                        optional: false,
                    });
                }
            }

            // Development dependencies
            if let Some(dev_deps) = poetry
                .get("group")
                .and_then(|g| g.get("dev"))
                .and_then(|d| d.get("dependencies"))
                .and_then(|d| d.as_table())
            {
                for (name, version_spec) in dev_deps {
                    let version = match version_spec {
                        toml::Value::String(v) => v.clone(),
                        _ => "*".to_string(),
                    };

                    dependencies.push(Dependency {
                        name: name.clone(),
                        version,
                        source: DependencySource::Registry("pypi".to_string()),
                        purpose: self.infer_dependency_purpose(name),
                        optional: true,
                    });
                }
            }
        }

        Ok(dependencies)
    }

    /// Extract dependencies from setup.py content
    fn extract_setup_py_deps(&self, content: &str) -> Result<Vec<Dependency>> {
        let mut dependencies = Vec::new();

        // Simple regex-based extraction (could be improved with AST parsing)
        if let Some(install_requires_start) = content.find("install_requires") {
            let remaining = &content[install_requires_start..];
            if let Some(bracket_start) = remaining.find('[') {
                if let Some(bracket_end) = remaining.find(']') {
                    let deps_str = &remaining[bracket_start + 1..bracket_end];

                    for line in deps_str.lines() {
                        let line = line
                            .trim()
                            .trim_matches('"')
                            .trim_matches('\'')
                            .trim_end_matches(',');
                        if !line.is_empty() && !line.starts_with('#') {
                            let (name, version) = if line.contains(">=") {
                                let parts: Vec<&str> = line.split(">=").collect();
                                (
                                    parts[0].to_string(),
                                    format!(">={}", parts.get(1).unwrap_or(&"0")),
                                )
                            } else if line.contains("==") {
                                let parts: Vec<&str> = line.split("==").collect();
                                (
                                    parts[0].to_string(),
                                    parts.get(1).unwrap_or(&"*").to_string(),
                                )
                            } else {
                                (line.to_string(), "*".to_string())
                            };

                            dependencies.push(Dependency {
                                name: name.clone(),
                                version,
                                source: DependencySource::Registry("pypi".to_string()),
                                purpose: self.infer_dependency_purpose(&name),
                                optional: false,
                            });
                        }
                    }
                }
            }
        }

        Ok(dependencies)
    }

    /// Extract dependencies from setup.cfg content
    fn extract_setup_cfg_deps(&self, _content: &str) -> Result<Vec<Dependency>> {
        // TODO: Implement setup.cfg parsing
        Ok(vec![])
    }

    /// Find Python entry points
    async fn find_entry_points(&self, project_path: &Path) -> Result<Vec<String>> {
        let mut entry_points = Vec::new();

        // Common entry point patterns
        let potential_entries = ["main.py", "__main__.py", "app.py", "cli.py", "run.py"];

        for entry in &potential_entries {
            if project_path.join(entry).exists() {
                entry_points.push(entry.to_string());
            }
        }

        // Look for setup.py entry points
        let setup_py = project_path.join("setup.py");
        if setup_py.exists() {
            let content = tokio::fs::read_to_string(&setup_py).await?;
            if content.contains("entry_points") {
                // Extract entry points from setup.py
                // This is a simplified extraction
                entry_points.push("setup.py defined entry points".to_string());
            }
        }

        // Check for __init__.py in common package locations
        if project_path.join("src").join("__init__.py").exists() {
            entry_points.push("src/__init__.py".to_string());
        }

        if entry_points.is_empty() {
            entry_points.push("main.py".to_string()); // Default assumption
        }

        Ok(entry_points)
    }

    /// Infer the purpose of a dependency based on its name
    fn infer_dependency_purpose(&self, name: &str) -> String {
        match name.to_lowercase().as_str() {
            // Web frameworks
            "django" | "flask" | "fastapi" | "tornado" | "bottle" => "Web framework".to_string(),

            // Data science
            "pandas" | "numpy" | "scipy" | "matplotlib" | "seaborn" => "Data science".to_string(),
            "tensorflow" | "torch" | "scikit-learn" | "keras" => "Machine learning".to_string(),

            // Database
            "psycopg2" | "pymongo" | "sqlalchemy" | "django-db" => "Database".to_string(),

            // Testing
            "pytest" | "unittest2" | "nose" | "coverage" => "Testing".to_string(),

            // Development tools
            "black" | "flake8" | "mypy" | "pylint" | "isort" => "Development tools".to_string(),

            // HTTP clients
            "requests" | "httpx" | "aiohttp" => "HTTP client".to_string(),

            // CLI tools
            "click" | "argparse" | "fire" => "CLI framework".to_string(),

            _ => "Application dependency".to_string(),
        }
    }

    /// Detect Python test framework
    async fn detect_test_framework(&self, project_path: &Path) -> Result<String> {
        // Check for pytest
        if project_path.join("pytest.ini").exists()
            || project_path.join("pyproject.toml").exists()
            || self.has_pytest_in_deps(project_path).await?
        {
            return Ok("pytest".to_string());
        }

        // Check for unittest
        if self.has_unittest_tests(project_path).await? {
            return Ok("unittest".to_string());
        }

        // Default to pytest
        Ok("pytest".to_string())
    }

    /// Check if pytest is in dependencies
    async fn has_pytest_in_deps(&self, project_path: &Path) -> Result<bool> {
        let deps = self.extract_dependencies(project_path).await?;
        Ok(deps.iter().any(|dep| dep.name.contains("pytest")))
    }

    /// Check for unittest test files
    async fn has_unittest_tests(&self, project_path: &Path) -> Result<bool> {
        let walker = walkdir::WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file());

        for entry in walker {
            if let Some(ext) = entry.path().extension() {
                if ext == "py" {
                    let content = tokio::fs::read_to_string(entry.path())
                        .await
                        .unwrap_or_default();
                    if content.contains("import unittest") || content.contains("from unittest") {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }
}

#[async_trait::async_trait]
impl LanguageAnalyzer for PythonAnalyzer {
    async fn analyze(&self, project_path: &Path) -> Result<LanguageModule> {
        let dependencies = self.extract_dependencies(project_path).await?;
        let entry_points = self.find_entry_points(project_path).await?;
        let build_config = self.analyze_build_config(project_path).await?;
        let test_framework = self.detect_test_framework(project_path).await?;

        let test_config = TestConfig {
            test_framework,
            test_directories: vec!["tests/".to_string(), "test/".to_string()],
            coverage_tool: "coverage".to_string(),
            test_commands: vec!["python -m pytest".to_string()],
        };

        let documentation = DocumentationConfig {
            doc_tool: "sphinx".to_string(),
            doc_format: "html".to_string(),
            doc_directory: "docs/".to_string(),
            auto_generate: true,
        };

        Ok(LanguageModule {
            language: Language::Python,
            entry_points,
            dependencies,
            build_config,
            test_config,
            documentation,
        })
    }

    async fn extract_dependencies(&self, project_path: &Path) -> Result<Vec<Dependency>> {
        let mut all_deps = Vec::new();

        // Try different dependency sources
        all_deps.extend(self.parse_requirements(project_path).await?);
        all_deps.extend(self.parse_setup_files(project_path).await?);
        all_deps.extend(self.parse_pyproject_toml(project_path).await?);

        // Deduplicate dependencies
        let mut unique_deps = HashMap::new();
        for dep in all_deps {
            unique_deps.insert(dep.name.clone(), dep);
        }

        Ok(unique_deps.into_values().collect())
    }

    async fn analyze_build_config(&self, project_path: &Path) -> Result<BuildConfig> {
        let build_tool = if project_path.join("pyproject.toml").exists() {
            // Check if it's a Poetry project
            let content = tokio::fs::read_to_string(project_path.join("pyproject.toml")).await?;
            if content.contains("[tool.poetry]") {
                "poetry"
            } else {
                "setuptools"
            }
        } else if project_path.join("setup.py").exists() {
            "setuptools"
        } else {
            "pip" // Default
        }
        .to_string();

        let build_file = if build_tool == "poetry" {
            "pyproject.toml"
        } else if project_path.join("setup.py").exists() {
            "setup.py"
        } else {
            "requirements.txt"
        }
        .to_string();

        Ok(BuildConfig {
            build_tool,
            build_file,
            compile_flags: vec![], // Python doesn't typically have compile flags
            optimization_level: "default".to_string(),
            target_platforms: vec![
                "linux".to_string(),
                "windows".to_string(),
                "macos".to_string(),
            ],
        })
    }

    async fn extract_interfaces(&self, project_path: &Path) -> Result<Vec<String>> {
        let mut interfaces = Vec::new();

        // Look for Flask/Django apps
        let walker = walkdir::WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file());

        for entry in walker {
            if let Some(ext) = entry.path().extension() {
                if ext == "py" {
                    let content = tokio::fs::read_to_string(entry.path())
                        .await
                        .unwrap_or_default();

                    if content.contains("from flask") || content.contains("import flask") {
                        interfaces.push("Flask Web API".to_string());
                    }

                    if content.contains("from django") || content.contains("import django") {
                        interfaces.push("Django Web API".to_string());
                    }

                    if content.contains("from fastapi") || content.contains("import fastapi") {
                        interfaces.push("FastAPI Web API".to_string());
                    }

                    if content.contains("if __name__ == \"__main__\"") {
                        interfaces.push("CLI Interface".to_string());
                    }
                }
            }
        }

        interfaces.dedup();
        Ok(interfaces)
    }
}

impl Default for PythonAnalyzer {
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
    async fn test_python_requirements_parsing() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create requirements.txt
        let mut requirements = File::create(temp_path.join("requirements.txt"))
            .await
            .unwrap();
        requirements
            .write_all(b"django==4.0.0\nrequests>=2.25.0\nflask\n# comment\n\n")
            .await
            .unwrap();

        let analyzer = PythonAnalyzer::new();
        let deps = analyzer.parse_requirements(temp_path).await.unwrap();

        assert_eq!(deps.len(), 3);
        assert!(deps
            .iter()
            .any(|d| d.name == "django" && d.version == "4.0.0"));
        assert!(deps
            .iter()
            .any(|d| d.name == "requests" && d.version == ">=2.25.0"));
        assert!(deps.iter().any(|d| d.name == "flask" && d.version == "*"));
    }

    #[tokio::test]
    async fn test_python_entry_point_detection() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create main.py
        let mut main_py = File::create(temp_path.join("main.py")).await.unwrap();
        main_py.write_all(b"print('Hello, world!')").await.unwrap();

        let analyzer = PythonAnalyzer::new();
        let entry_points = analyzer.find_entry_points(temp_path).await.unwrap();

        assert!(entry_points.contains(&"main.py".to_string()));
    }
}
