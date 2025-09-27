//! Codebase analyzer for extracting structure and relationships.

use crate::context::{
    AnalysisConfig, ContextError, Dependency, DependencySource, DependencyType, FileContext,
};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Analyzer for examining codebases and extracting structural information
#[derive(Debug)]
pub struct CodebaseAnalyzer {
    // Configuration and state for analysis
}

impl CodebaseAnalyzer {
    /// Create a new codebase analyzer
    pub fn new() -> Result<Self, ContextError> {
        Ok(Self {})
    }

    /// Analyze files in a directory and extract context information
    pub async fn analyze_files(
        &self,
        root_path: &PathBuf,
        config: &AnalysisConfig,
    ) -> Result<Vec<FileContext>, ContextError> {
        let mut file_contexts = Vec::new();

        // Walk through the directory tree
        for entry in WalkDir::new(root_path)
            .follow_links(config.follow_symlinks)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();

            // Apply include/exclude patterns
            if !self.should_include_file(file_path, config) {
                continue;
            }

            // Check file size limits
            if let Ok(metadata) = entry.metadata() {
                if metadata.len() > (config.max_file_size_mb * 1024 * 1024) as u64 {
                    continue;
                }
            }

            // Analyze the file
            match self.analyze_single_file(file_path, root_path).await {
                Ok(file_context) => file_contexts.push(file_context),
                Err(e) => {
                    // Only log warnings for non-UTF-8 issues if it's not a known binary file
                    let error_msg = e.to_string();
                    if error_msg.contains("non-UTF-8 content") {
                        // Skip binary files silently
                        continue;
                    } else {
                        tracing::warn!("Failed to analyze file {:?}: {}", file_path, e);
                        continue;
                    }
                }
            }
        }

        Ok(file_contexts)
    }

    /// Analyze specific files (for incremental updates)
    pub async fn analyze_specific_files(
        &self,
        file_paths: &[PathBuf],
        _config: &AnalysisConfig,
    ) -> Result<Vec<FileContext>, ContextError> {
        let mut file_contexts = Vec::new();

        for file_path in file_paths {
            if let Some(root_path) = file_path.parent() {
                match self
                    .analyze_single_file(file_path, &root_path.to_path_buf())
                    .await
                {
                    Ok(file_context) => file_contexts.push(file_context),
                    Err(e) => {
                        tracing::warn!("Failed to analyze file {:?}: {}", file_path, e);
                        continue;
                    }
                }
            }
        }

        Ok(file_contexts)
    }

    /// Analyze dependencies in the codebase
    pub async fn analyze_dependencies(
        &self,
        root_path: &PathBuf,
        _files: &[FileContext],
    ) -> Result<Vec<Dependency>, ContextError> {
        let mut dependencies = Vec::new();

        // Look for common dependency files
        if let Ok(cargo_toml) = fs::read_to_string(root_path.join("Cargo.toml")) {
            dependencies.extend(self.parse_cargo_dependencies(&cargo_toml)?);
        }

        if let Ok(package_json) = fs::read_to_string(root_path.join("package.json")) {
            dependencies.extend(self.parse_npm_dependencies(&package_json)?);
        }

        if let Ok(requirements_txt) = fs::read_to_string(root_path.join("requirements.txt")) {
            dependencies.extend(self.parse_python_dependencies(&requirements_txt)?);
        }

        if let Ok(go_mod) = fs::read_to_string(root_path.join("go.mod")) {
            dependencies.extend(self.parse_go_dependencies(&go_mod)?);
        }

        Ok(dependencies)
    }

    /// Analyze a single file and extract context information
    async fn analyze_single_file(
        &self,
        file_path: &Path,
        root_path: &PathBuf,
    ) -> Result<FileContext, ContextError> {
        let metadata = fs::metadata(file_path).map_err(|e| {
            ContextError::AnalysisFailed(format!(
                "Failed to read metadata for {:?}: {}",
                file_path, e
            ))
        })?;

        // Try to read as bytes first to check if it's valid UTF-8
        let bytes = fs::read(file_path).map_err(|e| {
            ContextError::AnalysisFailed(format!("Failed to read file {:?}: {}", file_path, e))
        })?;

        // Check if the file contains valid UTF-8
        let content = match String::from_utf8(bytes) {
            Ok(content) => content,
            Err(_) => {
                // Skip binary files silently instead of generating warnings
                return Err(ContextError::AnalysisFailed(
                    "File contains non-UTF-8 content".to_string(),
                ));
            }
        };

        let language = self.detect_language(file_path, &content);
        let relative_path = file_path
            .strip_prefix(root_path)
            .unwrap_or(file_path)
            .to_path_buf();

        // Calculate content hash
        let content_hash = format!("{:x}", md5::compute(&content));

        // Extract imports and exports based on language
        let (imports, exports) = self.extract_imports_exports(&content, &language);

        // Extract symbols (basic implementation)
        let symbols = self.extract_symbols(&content, &language);

        // Determine relationships (basic implementation)
        let relationships = Vec::new(); // TODO: Implement relationship detection

        Ok(FileContext {
            path: file_path.to_path_buf(),
            relative_path,
            language,
            size_bytes: metadata.len(),
            line_count: content.lines().count(),
            last_modified: metadata
                .modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
            content_hash,
            symbols,
            imports,
            exports,
            relationships,
        })
    }

    /// Check if a file should be included based on patterns
    fn should_include_file(&self, file_path: &Path, config: &AnalysisConfig) -> bool {
        let path_str = file_path.to_string_lossy();

        // Check exclude patterns first
        for pattern in &config.exclude_patterns {
            if self.matches_glob_pattern(&path_str, pattern) {
                return false;
            }
        }

        // Additional hardcoded exclusions for performance
        if self.is_build_or_binary_file(file_path) {
            return false;
        }

        // Check include patterns - if any match, include the file
        if config.include_patterns.is_empty() {
            // If no include patterns specified, include by default (after exclude checks)
            return true;
        }

        for pattern in &config.include_patterns {
            if self.matches_glob_pattern(&path_str, pattern) {
                return true;
            }
        }

        false
    }

    /// Check if a file is a build artifact or binary file that should be excluded
    fn is_build_or_binary_file(&self, file_path: &Path) -> bool {
        let path_str = file_path.to_string_lossy();
        let path_lower = path_str.to_lowercase();

        // Exclude common build directories
        if path_str.contains("/target/")
            || path_str.contains("\\target\\")
            || path_str.contains("/build/")
            || path_str.contains("\\build\\")
            || path_str.contains("/dist/")
            || path_str.contains("\\dist\\")
            || path_str.contains("/out/")
            || path_str.contains("\\out\\")
            || path_str.contains("/bin/")
            || path_str.contains("\\bin\\")
            || path_str.contains("/.git/")
            || path_str.contains("\\.git\\")
            || path_str.contains("/node_modules/")
            || path_str.contains("\\node_modules\\")
            || path_str.contains("/__pycache__/")
            || path_str.contains("\\__pycache__\\")
        {
            return true;
        }

        // Exclude binary file extensions and other non-text files
        if let Some(extension) = file_path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            match ext.as_str() {
                // Binary executables
                "exe" | "dll" | "so" | "dylib" | "a" | "lib" | "o" | "obj" |
                // Archives
                "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" |
                // Images
                "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "svg" |
                // Videos
                "mp4" | "avi" | "mov" | "wmv" | "flv" | "webm" |
                // Audio
                "mp3" | "wav" | "flac" | "aac" | "ogg" |
                // Documents
                "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" |
                // Rust-specific build artifacts
                "rlib" | "rmeta" | "d" |
                // Lock files that are typically large/binary
                "lock" if path_lower.contains("cargo.lock") => false, // Allow Cargo.lock
                "lock" => true,
                _ => false
            }
        } else {
            // Files without extensions - check if they're common binary files
            if let Some(filename) = file_path.file_name() {
                let name = filename.to_string_lossy().to_lowercase();
                // Check for known binary files without extensions
                if name.starts_with("lib") && !name.contains(".") {
                    true // Unix libraries
                } else if name.len() > 50 {
                    true // Very long names are often build artifacts
                } else {
                    // Allow common text files without extensions
                    !matches!(
                        name.as_str(),
                        "makefile"
                            | "dockerfile"
                            | "vagrantfile"
                            | "readme"
                            | "license"
                            | "changelog"
                    )
                }
            } else {
                false
            }
        }
    }

    /// Simple glob pattern matching
    fn matches_glob_pattern(&self, path: &str, pattern: &str) -> bool {
        // Simple implementation - in production, use a proper glob library
        if pattern == "**/*" {
            return true;
        }

        if pattern.starts_with("**/") {
            let suffix = pattern.strip_prefix("**/").unwrap_or(pattern);
            // For patterns like **/*.rs, check if path ends with .rs
            if suffix.starts_with("*") {
                let extension = suffix.strip_prefix("*").unwrap_or(suffix);
                return path.ends_with(extension);
            }
            return path.contains(suffix);
        }

        if pattern.ends_with("/**") {
            let prefix = pattern.strip_suffix("/**").unwrap_or(pattern);
            return path.starts_with(prefix);
        }

        // Handle simple extension patterns like *.rs
        if pattern.starts_with("*") && !pattern.contains("/") {
            let extension = pattern.strip_prefix("*").unwrap_or(pattern);
            return path.ends_with(extension);
        }

        path.contains(pattern)
    }

    /// Detect programming language from file extension and content
    fn detect_language(&self, file_path: &Path, _content: &str) -> String {
        if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
            match extension {
                "rs" => "rust".to_string(),
                "py" => "python".to_string(),
                "js" => "javascript".to_string(),
                "ts" => "typescript".to_string(),
                "java" => "java".to_string(),
                "cpp" | "cc" | "cxx" => "cpp".to_string(),
                "c" => "c".to_string(),
                "go" => "go".to_string(),
                "rb" => "ruby".to_string(),
                "php" => "php".to_string(),
                "sh" => "shell".to_string(),
                "yml" | "yaml" => "yaml".to_string(),
                "json" => "json".to_string(),
                "md" => "markdown".to_string(),
                "toml" => "toml".to_string(),
                _ => "unknown".to_string(),
            }
        } else {
            "unknown".to_string()
        }
    }

    /// Extract imports and exports from file content
    fn extract_imports_exports(&self, content: &str, language: &str) -> (Vec<String>, Vec<String>) {
        let mut imports = Vec::new();
        let mut exports = Vec::new();

        match language {
            "rust" => {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("use ") {
                        imports.push(trimmed.to_string());
                    }
                    if trimmed.starts_with("pub ") {
                        exports.push(trimmed.to_string());
                    }
                }
            }
            "python" => {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
                        imports.push(trimmed.to_string());
                    }
                }
            }
            "javascript" | "typescript" => {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("import ") || trimmed.contains("require(") {
                        imports.push(trimmed.to_string());
                    }
                    if trimmed.starts_with("export ") || trimmed.contains("module.exports") {
                        exports.push(trimmed.to_string());
                    }
                }
            }
            _ => {} // No extraction for unknown languages
        }

        (imports, exports)
    }

    /// Extract symbols from file content (basic implementation)
    fn extract_symbols(
        &self,
        content: &str,
        language: &str,
    ) -> Vec<crate::context::symbols::Symbol> {
        // Generic symbol extraction doesn't use specific symbol types

        let mut symbols = Vec::new();

        match language {
            "rust" => {
                symbols.extend(self.extract_rust_symbols_basic(content));
            }
            "python" => {
                symbols.extend(self.extract_python_symbols_basic(content));
            }
            "javascript" | "typescript" => {
                symbols.extend(self.extract_js_symbols_basic(content));
            }
            _ => {
                // Generic extraction for other languages
                symbols.extend(self.extract_generic_symbols_basic(content));
            }
        }

        symbols
    }

    /// Basic Rust symbol extraction
    fn extract_rust_symbols_basic(&self, content: &str) -> Vec<crate::context::symbols::Symbol> {
        use crate::context::symbols::{Symbol, SymbolType, Visibility};
        let mut symbols = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Function definitions
            if let Some(start) = trimmed.find("fn ") {
                if let Some(name_start) = trimmed[start + 3..].find(char::is_alphabetic) {
                    let name_part = &trimmed[start + 3 + name_start..];
                    if let Some(name_end) = name_part.find(|c: char| c == '(' || c.is_whitespace())
                    {
                        let name = name_part[..name_end].to_string();
                        let mut symbol = Symbol::new(
                            name,
                            SymbolType::Function,
                            PathBuf::new(),
                            line_num + 1,
                            0,
                        );
                        symbol.visibility = if trimmed.starts_with("pub ") {
                            Visibility::Public
                        } else {
                            Visibility::Private
                        };
                        symbols.push(symbol);
                    }
                }
            }

            // Struct definitions
            if let Some(start) = trimmed.find("struct ") {
                if let Some(name_start) = trimmed[start + 7..].find(char::is_alphabetic) {
                    let name_part = &trimmed[start + 7 + name_start..];
                    if let Some(name_end) =
                        name_part.find(|c: char| c.is_whitespace() || c == '{' || c == '<')
                    {
                        let name = name_part[..name_end].to_string();
                        let mut symbol =
                            Symbol::new(name, SymbolType::Struct, PathBuf::new(), line_num + 1, 0);
                        symbol.visibility = if trimmed.starts_with("pub ") {
                            Visibility::Public
                        } else {
                            Visibility::Private
                        };
                        symbols.push(symbol);
                    }
                }
            }

            // Enum definitions
            if let Some(start) = trimmed.find("enum ") {
                if let Some(name_start) = trimmed[start + 5..].find(char::is_alphabetic) {
                    let name_part = &trimmed[start + 5 + name_start..];
                    if let Some(name_end) =
                        name_part.find(|c: char| c.is_whitespace() || c == '{' || c == '<')
                    {
                        let name = name_part[..name_end].to_string();
                        symbols.push(Symbol::new(
                            name,
                            SymbolType::Enum,
                            PathBuf::new(),
                            line_num + 1,
                            0,
                        ));
                    }
                }
            }

            // Trait definitions
            if let Some(start) = trimmed.find("trait ") {
                if let Some(name_start) = trimmed[start + 6..].find(char::is_alphabetic) {
                    let name_part = &trimmed[start + 6 + name_start..];
                    if let Some(name_end) =
                        name_part.find(|c: char| c.is_whitespace() || c == '{' || c == '<')
                    {
                        let name = name_part[..name_end].to_string();
                        symbols.push(Symbol::new(
                            name,
                            SymbolType::Trait,
                            PathBuf::new(),
                            line_num + 1,
                            0,
                        ));
                    }
                }
            }
        }

        symbols
    }

    /// Basic Python symbol extraction
    fn extract_python_symbols_basic(&self, content: &str) -> Vec<crate::context::symbols::Symbol> {
        use crate::context::symbols::{Symbol, SymbolType, Visibility};
        let mut symbols = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Function definitions
            if trimmed.starts_with("def ") {
                if let Some(name_start) = trimmed[4..].find(char::is_alphabetic) {
                    let name_part = &trimmed[4 + name_start..];
                    if let Some(name_end) = name_part.find('(') {
                        let name = name_part[..name_end].to_string();
                        let mut symbol = Symbol::new(
                            name.clone(),
                            SymbolType::Function,
                            PathBuf::new(),
                            line_num + 1,
                            0,
                        );
                        symbol.visibility = if name.starts_with('_') {
                            Visibility::Private
                        } else {
                            Visibility::Public
                        };
                        symbols.push(symbol);
                    }
                }
            }

            // Class definitions
            if trimmed.starts_with("class ") {
                if let Some(name_start) = trimmed[6..].find(char::is_alphabetic) {
                    let name_part = &trimmed[6 + name_start..];
                    if let Some(name_end) =
                        name_part.find(|c: char| c.is_whitespace() || c == '(' || c == ':')
                    {
                        let name = name_part[..name_end].to_string();
                        symbols.push(Symbol::new(
                            name,
                            SymbolType::Class,
                            PathBuf::new(),
                            line_num + 1,
                            0,
                        ));
                    }
                }
            }
        }

        symbols
    }

    /// Basic JavaScript/TypeScript symbol extraction
    fn extract_js_symbols_basic(&self, content: &str) -> Vec<crate::context::symbols::Symbol> {
        use crate::context::symbols::{Symbol, SymbolType};
        let mut symbols = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Function declarations
            if trimmed.starts_with("function ") {
                if let Some(name_start) = trimmed[9..].find(char::is_alphabetic) {
                    let name_part = &trimmed[9 + name_start..];
                    if let Some(name_end) = name_part.find('(') {
                        let name = name_part[..name_end].to_string();
                        symbols.push(Symbol::new(
                            name,
                            SymbolType::Function,
                            PathBuf::new(),
                            line_num + 1,
                            0,
                        ));
                    }
                }
            }

            // Arrow functions
            if trimmed.contains(" => ") {
                if let Some(arrow_pos) = trimmed.find(" => ") {
                    let before_arrow = &trimmed[..arrow_pos];
                    if let Some(equals_pos) = before_arrow.rfind('=') {
                        if let Some(_name_start) = before_arrow[..equals_pos]
                            .rfind(|c: char| c.is_alphabetic() || c == '_')
                        {
                            let name_part = &before_arrow[..equals_pos];
                            if let Some(actual_name_start) = name_part.rfind(|c: char| {
                                c.is_whitespace() || c == '{' || c == ',' || c == '('
                            }) {
                                let name = name_part[actual_name_start + 1..].trim().to_string();
                                if !name.is_empty() {
                                    symbols.push(Symbol::new(
                                        name,
                                        SymbolType::Function,
                                        PathBuf::new(),
                                        line_num + 1,
                                        0,
                                    ));
                                }
                            }
                        }
                    }
                }
            }

            // Class definitions
            if trimmed.starts_with("class ") {
                if let Some(name_start) = trimmed[6..].find(char::is_alphabetic) {
                    let name_part = &trimmed[6 + name_start..];
                    if let Some(name_end) = name_part.find(|c: char| c.is_whitespace() || c == '{')
                    {
                        let name = name_part[..name_end].to_string();
                        symbols.push(Symbol::new(
                            name,
                            SymbolType::Class,
                            PathBuf::new(),
                            line_num + 1,
                            0,
                        ));
                    }
                }
            }

            // Interface definitions (TypeScript)
            if trimmed.starts_with("interface ") {
                if let Some(name_start) = trimmed[10..].find(char::is_alphabetic) {
                    let name_part = &trimmed[10 + name_start..];
                    if let Some(name_end) =
                        name_part.find(|c: char| c.is_whitespace() || c == '{' || c == '<')
                    {
                        let name = name_part[..name_end].to_string();
                        symbols.push(Symbol::new(
                            name,
                            SymbolType::Interface,
                            PathBuf::new(),
                            line_num + 1,
                            0,
                        ));
                    }
                }
            }
        }

        symbols
    }

    /// Generic symbol extraction for unknown languages
    fn extract_generic_symbols_basic(&self, content: &str) -> Vec<crate::context::symbols::Symbol> {
        use crate::context::symbols::{Symbol, SymbolType};
        let mut symbols = Vec::new();

        // Very basic extraction - look for common patterns
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Look for lines that might be function/method definitions
            if (trimmed.contains("function") || trimmed.contains("def ") || trimmed.contains("fn "))
                && (trimmed.contains('(') || trimmed.contains('{'))
            {
                // Try to extract a name
                for word in trimmed.split_whitespace() {
                    if word.chars().all(|c| c.is_alphanumeric() || c == '_')
                        && word
                            .chars()
                            .next()
                            .map(|c| c.is_alphabetic())
                            .unwrap_or(false)
                    {
                        symbols.push(Symbol::new(
                            word.to_string(),
                            SymbolType::Function,
                            PathBuf::new(),
                            line_num + 1,
                            0,
                        ));
                        break;
                    }
                }
            }
        }

        symbols
    }

    /// Parse Cargo.toml dependencies
    fn parse_cargo_dependencies(&self, content: &str) -> Result<Vec<Dependency>, ContextError> {
        // Simple parsing - in production, use proper TOML parser
        let mut dependencies = Vec::new();

        let lines: Vec<&str> = content.lines().collect();
        let mut in_dependencies = false;

        for line in lines {
            let trimmed = line.trim();

            if trimmed == "[dependencies]" {
                in_dependencies = true;
                continue;
            }

            if trimmed.starts_with('[') && trimmed != "[dependencies]" {
                in_dependencies = false;
                continue;
            }

            if in_dependencies && trimmed.contains('=') {
                if let Some((name, _version)) = trimmed.split_once('=') {
                    dependencies.push(Dependency {
                        name: name.trim().to_string(),
                        version: None, // TODO: Parse version
                        dependency_type: DependencyType::Runtime,
                        source: DependencySource::PackageManager("cargo".to_string()),
                    });
                }
            }
        }

        Ok(dependencies)
    }

    /// Parse package.json dependencies
    fn parse_npm_dependencies(&self, _content: &str) -> Result<Vec<Dependency>, ContextError> {
        // TODO: Implement JSON parsing for package.json
        Ok(Vec::new())
    }

    /// Parse requirements.txt dependencies
    fn parse_python_dependencies(&self, content: &str) -> Result<Vec<Dependency>, ContextError> {
        let mut dependencies = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                let name = if let Some((name, _version)) = trimmed.split_once("==") {
                    name.trim().to_string()
                } else {
                    trimmed.to_string()
                };

                dependencies.push(Dependency {
                    name,
                    version: None, // TODO: Parse version
                    dependency_type: DependencyType::Runtime,
                    source: DependencySource::PackageManager("pip".to_string()),
                });
            }
        }

        Ok(dependencies)
    }

    /// Parse go.mod dependencies
    fn parse_go_dependencies(&self, _content: &str) -> Result<Vec<Dependency>, ContextError> {
        // TODO: Implement go.mod parsing
        Ok(Vec::new())
    }
}
