//! JavaScript/TypeScript Language Analyzer
//!
//! Analyzes JavaScript and TypeScript projects to extract module structure,
//! dependencies, and build configuration for blueprint generation.

use super::super::{
    BuildConfig, Dependency, DependencySource, DocumentationConfig, Language, LanguageAnalyzer,
    LanguageModule, TestConfig,
};
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

pub struct JavaScriptAnalyzer;

impl JavaScriptAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Parse package.json dependencies
    async fn parse_package_json(&self, project_path: &Path) -> Result<Vec<Dependency>> {
        let package_path = project_path.join("package.json");
        if !package_path.exists() {
            return Ok(vec![]);
        }

        let content = tokio::fs::read_to_string(&package_path).await?;
        let package_json: Value = serde_json::from_str(&content)?;

        let mut dependencies = Vec::new();

        // Parse dependencies
        if let Some(deps) = package_json.get("dependencies").and_then(|d| d.as_object()) {
            for (name, version_spec) in deps {
                let version = version_spec.as_str().unwrap_or("*").to_string();
                dependencies.push(Dependency {
                    name: name.clone(),
                    version,
                    source: DependencySource::Registry("npm".to_string()),
                    purpose: self.infer_dependency_purpose(name),
                    optional: false,
                });
            }
        }

        // Parse devDependencies
        if let Some(dev_deps) = package_json
            .get("devDependencies")
            .and_then(|d| d.as_object())
        {
            for (name, version_spec) in dev_deps {
                let version = version_spec.as_str().unwrap_or("*").to_string();
                dependencies.push(Dependency {
                    name: name.clone(),
                    version,
                    source: DependencySource::Registry("npm".to_string()),
                    purpose: self.infer_dependency_purpose(name),
                    optional: true,
                });
            }
        }

        // Parse peerDependencies
        if let Some(peer_deps) = package_json
            .get("peerDependencies")
            .and_then(|d| d.as_object())
        {
            for (name, version_spec) in peer_deps {
                let version = version_spec.as_str().unwrap_or("*").to_string();
                dependencies.push(Dependency {
                    name: name.clone(),
                    version,
                    source: DependencySource::Registry("npm".to_string()),
                    purpose: format!("Peer dependency: {}", self.infer_dependency_purpose(name)),
                    optional: true,
                });
            }
        }

        Ok(dependencies)
    }

    /// Parse yarn.lock or package-lock.json for more precise versions
    async fn parse_lock_files(&self, project_path: &Path) -> Result<HashMap<String, String>> {
        let mut locked_versions = HashMap::new();

        // Check for yarn.lock
        let yarn_lock = project_path.join("yarn.lock");
        if yarn_lock.exists() {
            let content = tokio::fs::read_to_string(&yarn_lock).await?;
            locked_versions.extend(self.parse_yarn_lock(&content)?);
        }

        // Check for package-lock.json
        let package_lock = project_path.join("package-lock.json");
        if package_lock.exists() {
            let content = tokio::fs::read_to_string(&package_lock).await?;
            locked_versions.extend(self.parse_package_lock(&content)?);
        }

        Ok(locked_versions)
    }

    /// Parse yarn.lock content
    fn parse_yarn_lock(&self, _content: &str) -> Result<HashMap<String, String>> {
        // Simplified yarn.lock parsing - in practice, you'd want a proper parser
        // For now, we'll return empty map as it's complex to parse properly
        Ok(HashMap::new())
    }

    /// Parse package-lock.json content
    fn parse_package_lock(&self, content: &str) -> Result<HashMap<String, String>> {
        let lock_json: Value = serde_json::from_str(content)?;
        let mut locked_versions = HashMap::new();

        if let Some(deps) = lock_json.get("dependencies").and_then(|d| d.as_object()) {
            for (name, dep_info) in deps {
                if let Some(version) = dep_info.get("version").and_then(|v| v.as_str()) {
                    locked_versions.insert(name.clone(), version.to_string());
                }
            }
        }

        Ok(locked_versions)
    }

    /// Detect project type and framework
    async fn detect_project_type(&self, project_path: &Path) -> Result<String> {
        let package_path = project_path.join("package.json");
        if !package_path.exists() {
            return Ok("vanilla".to_string());
        }

        let content = tokio::fs::read_to_string(&package_path).await?;
        let package_json: Value = serde_json::from_str(&content)?;

        // Check dependencies and devDependencies for framework indicators
        let mut deps = Vec::new();
        if let Some(dependencies) = package_json.get("dependencies").and_then(|d| d.as_object()) {
            deps.extend(dependencies.keys());
        }
        if let Some(dev_dependencies) = package_json
            .get("devDependencies")
            .and_then(|d| d.as_object())
        {
            deps.extend(dev_dependencies.keys());
        }

        // React
        if deps.iter().any(|dep| dep.contains("react")) {
            if deps.iter().any(|dep| dep.contains("next")) {
                return Ok("nextjs".to_string());
            }
            if deps.iter().any(|dep| dep.contains("gatsby")) {
                return Ok("gatsby".to_string());
            }
            return Ok("react".to_string());
        }

        // Vue
        if deps.iter().any(|dep| dep.contains("vue")) {
            if deps.iter().any(|dep| dep.contains("nuxt")) {
                return Ok("nuxtjs".to_string());
            }
            return Ok("vue".to_string());
        }

        // Angular
        if deps.iter().any(|dep| dep.contains("@angular")) {
            return Ok("angular".to_string());
        }

        // Node.js backend frameworks
        if deps.iter().any(|dep| dep == &"express") {
            return Ok("express".to_string());
        }
        if deps.iter().any(|dep| dep == &"fastify") {
            return Ok("fastify".to_string());
        }
        if deps.iter().any(|dep| dep == &"koa") {
            return Ok("koa".to_string());
        }
        if deps.iter().any(|dep| dep == &"nestjs") {
            return Ok("nestjs".to_string());
        }

        // Check for TypeScript
        if deps.iter().any(|dep| dep == &"typescript")
            || project_path.join("tsconfig.json").exists()
        {
            return Ok("typescript".to_string());
        }

        Ok("nodejs".to_string())
    }

    /// Find entry points from package.json
    async fn find_entry_points(&self, project_path: &Path) -> Result<Vec<String>> {
        let mut entry_points = Vec::new();

        let package_path = project_path.join("package.json");
        if package_path.exists() {
            let content = tokio::fs::read_to_string(&package_path).await?;
            let package_json: Value = serde_json::from_str(&content)?;

            // Main entry point
            if let Some(main) = package_json.get("main").and_then(|m| m.as_str()) {
                entry_points.push(main.to_string());
            }

            // Module entry point
            if let Some(module) = package_json.get("module").and_then(|m| m.as_str()) {
                entry_points.push(module.to_string());
            }

            // TypeScript entry point
            if let Some(types) = package_json.get("types").and_then(|t| t.as_str()) {
                entry_points.push(types.to_string());
            }

            // Bin entries
            if let Some(bin) = package_json.get("bin") {
                match bin {
                    Value::String(bin_path) => entry_points.push(bin_path.clone()),
                    Value::Object(bin_obj) => {
                        for bin_path in bin_obj.values() {
                            if let Some(path) = bin_path.as_str() {
                                entry_points.push(path.to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Scripts that might be entry points
            if let Some(scripts) = package_json.get("scripts").and_then(|s| s.as_object()) {
                if scripts.contains_key("start") {
                    entry_points.push("package.json:start".to_string());
                }
                if scripts.contains_key("dev") {
                    entry_points.push("package.json:dev".to_string());
                }
            }
        }

        // Check common entry point files
        let common_entries = [
            "index.js",
            "index.ts",
            "app.js",
            "app.ts",
            "main.js",
            "main.ts",
            "server.js",
            "server.ts",
            "src/index.js",
            "src/index.ts",
            "src/app.js",
            "src/app.ts",
            "src/main.js",
            "src/main.ts",
            "src/server.js",
            "src/server.ts",
        ];

        for entry in &common_entries {
            if project_path.join(entry).exists() {
                entry_points.push(entry.to_string());
            }
        }

        if entry_points.is_empty() {
            entry_points.push("index.js".to_string()); // Default assumption
        }

        // Remove duplicates
        entry_points.sort();
        entry_points.dedup();

        Ok(entry_points)
    }

    /// Infer dependency purpose based on package name
    fn infer_dependency_purpose(&self, name: &str) -> String {
        match name {
            // React ecosystem
            name if name.contains("react") => "React library".to_string(),
            "redux" | "@reduxjs/toolkit" | "recoil" | "zustand" => "State management".to_string(),

            // Vue ecosystem
            name if name.contains("vue") => "Vue library".to_string(),
            "vuex" | "pinia" => "State management".to_string(),

            // Angular ecosystem
            name if name.contains("@angular") => "Angular library".to_string(),

            // Build tools
            "webpack" | "rollup" | "vite" | "parcel" => "Build tool".to_string(),
            name if name == "babel" || name.starts_with("@babel") => "Transpiler".to_string(),

            // Testing
            "jest" | "mocha" | "jasmine" | "vitest" | "cypress" | "playwright" => {
                "Testing".to_string()
            }

            // Linting/Formatting
            "eslint" | "prettier" | "stylelint" => "Code quality".to_string(),

            // CSS frameworks
            "tailwindcss" | "bootstrap" | "bulma" | "antd" | "@mui/material" => {
                "CSS framework".to_string()
            }

            // HTTP clients
            "axios" | "fetch" | "node-fetch" | "cross-fetch" => "HTTP client".to_string(),

            // Server frameworks
            "express" | "fastify" | "koa" | "hapi" => "Server framework".to_string(),

            // Database
            "mongoose" | "sequelize" | "typeorm" | "prisma" => "Database ORM".to_string(),

            // Utilities
            "lodash" | "ramda" | "underscore" => "Utility library".to_string(),
            "moment" | "date-fns" | "dayjs" => "Date/time utility".to_string(),

            // TypeScript
            name if name == "typescript" || name.starts_with("@types/") => {
                "TypeScript support".to_string()
            }

            _ => "Application dependency".to_string(),
        }
    }

    /// Detect test framework
    async fn detect_test_framework(&self, project_path: &Path) -> Result<String> {
        let deps = self.extract_dependencies(project_path).await?;

        // Check dependencies for test frameworks
        for dep in &deps {
            match dep.name.as_str() {
                "jest" => return Ok("jest".to_string()),
                "mocha" => return Ok("mocha".to_string()),
                "jasmine" => return Ok("jasmine".to_string()),
                "vitest" => return Ok("vitest".to_string()),
                "cypress" => return Ok("cypress".to_string()),
                "playwright" => return Ok("playwright".to_string()),
                _ => {}
            }
        }

        // Check for config files
        if project_path.join("jest.config.js").exists()
            || project_path.join("jest.config.json").exists()
        {
            return Ok("jest".to_string());
        }

        if project_path.join("vitest.config.js").exists()
            || project_path.join("vitest.config.ts").exists()
        {
            return Ok("vitest".to_string());
        }

        if project_path.join("cypress.json").exists()
            || project_path.join("cypress.config.js").exists()
        {
            return Ok("cypress".to_string());
        }

        // Default based on project type
        let project_type = self.detect_project_type(project_path).await?;
        match project_type.as_str() {
            "vue" | "nuxtjs" => Ok("vitest".to_string()),
            _ => Ok("jest".to_string()),
        }
    }

    /// Detect build tool
    async fn detect_build_tool(&self, project_path: &Path) -> Result<String> {
        // Check for config files first
        if project_path.join("webpack.config.js").exists()
            || project_path.join("webpack.config.ts").exists()
        {
            return Ok("webpack".to_string());
        }

        if project_path.join("rollup.config.js").exists()
            || project_path.join("rollup.config.ts").exists()
        {
            return Ok("rollup".to_string());
        }

        if project_path.join("vite.config.js").exists()
            || project_path.join("vite.config.ts").exists()
        {
            return Ok("vite".to_string());
        }

        // Check dependencies
        let deps = self.extract_dependencies(project_path).await?;
        for dep in &deps {
            match dep.name.as_str() {
                "vite" => return Ok("vite".to_string()),
                "webpack" => return Ok("webpack".to_string()),
                "rollup" => return Ok("rollup".to_string()),
                "parcel" => return Ok("parcel".to_string()),
                _ => {}
            }
        }

        // Default based on project type
        let project_type = self.detect_project_type(project_path).await?;
        match project_type.as_str() {
            "vue" | "nuxtjs" => Ok("vite".to_string()),
            "react" => Ok("webpack".to_string()),
            _ => Ok("npm".to_string()),
        }
    }

    /// Determine if project is TypeScript
    async fn is_typescript_project(&self, project_path: &Path) -> Result<bool> {
        // Check for tsconfig.json
        if project_path.join("tsconfig.json").exists() {
            return Ok(true);
        }

        // Check for TypeScript files
        let walker = walkdir::WalkDir::new(project_path)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file());

        for entry in walker {
            if let Some(ext) = entry.path().extension() {
                if ext == "ts" || ext == "tsx" {
                    return Ok(true);
                }
            }
        }

        // Check dependencies
        let deps = self.extract_dependencies(project_path).await?;
        if deps.iter().any(|dep| dep.name == "typescript") {
            return Ok(true);
        }

        Ok(false)
    }
}

#[async_trait::async_trait]
impl LanguageAnalyzer for JavaScriptAnalyzer {
    async fn analyze(&self, project_path: &Path) -> Result<LanguageModule> {
        let dependencies = self.extract_dependencies(project_path).await?;
        let entry_points = self.find_entry_points(project_path).await?;
        let build_config = self.analyze_build_config(project_path).await?;
        let test_framework = self.detect_test_framework(project_path).await?;

        let test_config = TestConfig {
            test_framework: test_framework.clone(),
            test_directories: vec![
                "test/".to_string(),
                "tests/".to_string(),
                "__tests__/".to_string(),
            ],
            coverage_tool: if test_framework == "jest" {
                "jest"
            } else {
                "nyc"
            }
            .to_string(),
            test_commands: vec![format!("npm test"), "npm run test:unit".to_string()],
        };

        let documentation = DocumentationConfig {
            doc_tool: "jsdoc".to_string(),
            doc_format: "html".to_string(),
            doc_directory: "docs/".to_string(),
            auto_generate: true,
        };

        let language = if self.is_typescript_project(project_path).await? {
            Language::TypeScript
        } else {
            Language::JavaScript
        };

        Ok(LanguageModule {
            language,
            entry_points,
            dependencies,
            build_config,
            test_config,
            documentation,
        })
    }

    async fn extract_dependencies(&self, project_path: &Path) -> Result<Vec<Dependency>> {
        self.parse_package_json(project_path).await
    }

    async fn analyze_build_config(&self, project_path: &Path) -> Result<BuildConfig> {
        let build_tool = self.detect_build_tool(project_path).await?;

        let build_file = match build_tool.as_str() {
            "webpack" => "webpack.config.js".to_string(),
            "rollup" => "rollup.config.js".to_string(),
            "vite" => "vite.config.js".to_string(),
            "parcel" => "package.json".to_string(),
            _ => "package.json".to_string(),
        };

        let mut compile_flags = Vec::new();

        // Add TypeScript compilation if applicable
        if self.is_typescript_project(project_path).await? {
            compile_flags.push("--typescript".to_string());
        }

        Ok(BuildConfig {
            build_tool,
            build_file,
            compile_flags,
            optimization_level: "production".to_string(),
            target_platforms: vec!["node".to_string(), "browser".to_string()],
        })
    }

    async fn extract_interfaces(&self, project_path: &Path) -> Result<Vec<String>> {
        let mut interfaces = Vec::new();
        let project_type = self.detect_project_type(project_path).await?;

        // Add interfaces based on project type
        match project_type.as_str() {
            "react" | "nextjs" | "gatsby" => {
                interfaces.push("React Web UI".to_string());
            }
            "vue" | "nuxtjs" => {
                interfaces.push("Vue Web UI".to_string());
            }
            "angular" => {
                interfaces.push("Angular Web UI".to_string());
            }
            "express" | "fastify" | "koa" | "nestjs" => {
                interfaces.push("REST API".to_string());
            }
            _ => {}
        }

        // Check package.json for additional interface hints
        let package_path = project_path.join("package.json");
        if package_path.exists() {
            let content = tokio::fs::read_to_string(&package_path).await?;
            let package_json: Value = serde_json::from_str(&content)?;

            if let Some(scripts) = package_json.get("scripts").and_then(|s| s.as_object()) {
                if scripts.contains_key("start") || scripts.contains_key("serve") {
                    interfaces.push("CLI Application".to_string());
                }
            }

            if let Some(bin) = package_json.get("bin") {
                if !bin.is_null() {
                    interfaces.push("Command Line Tool".to_string());
                }
            }
        }

        if interfaces.is_empty() {
            interfaces.push("JavaScript Module".to_string());
        }

        interfaces.dedup();
        Ok(interfaces)
    }
}

impl Default for JavaScriptAnalyzer {
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
    async fn test_package_json_parsing() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create package.json
        let package_json = r#"{
            "name": "test-project",
            "version": "1.0.0",
            "dependencies": {
                "react": "^18.0.0",
                "axios": "^0.27.0"
            },
            "devDependencies": {
                "jest": "^28.0.0",
                "typescript": "^4.7.0"
            }
        }"#;

        let mut file = File::create(temp_path.join("package.json")).await.unwrap();
        file.write_all(package_json.as_bytes()).await.unwrap();

        let analyzer = JavaScriptAnalyzer::new();
        let deps = analyzer.parse_package_json(temp_path).await.unwrap();

        assert_eq!(deps.len(), 4);
        assert!(deps.iter().any(|d| d.name == "react"));
        assert!(deps.iter().any(|d| d.name == "jest" && d.optional));
        assert!(deps.iter().any(|d| d.name == "axios" && !d.optional));
    }

    #[tokio::test]
    async fn test_project_type_detection() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create React project package.json
        let package_json = r#"{
            "dependencies": {
                "react": "^18.0.0",
                "react-dom": "^18.0.0"
            }
        }"#;

        let mut file = File::create(temp_path.join("package.json")).await.unwrap();
        file.write_all(package_json.as_bytes()).await.unwrap();

        let analyzer = JavaScriptAnalyzer::new();
        let project_type = analyzer.detect_project_type(temp_path).await.unwrap();

        assert_eq!(project_type, "react");
    }

    #[tokio::test]
    async fn test_typescript_detection() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create tsconfig.json
        let mut tsconfig = File::create(temp_path.join("tsconfig.json")).await.unwrap();
        tsconfig.write_all(b"{}").await.unwrap();

        let analyzer = JavaScriptAnalyzer::new();
        let is_ts = analyzer.is_typescript_project(temp_path).await.unwrap();

        assert!(is_ts);
    }
}
