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

        // Determine relationships based on imports/exports and file analysis
        let relationships = self.detect_relationships(&content, &language, file_path, root_path, &imports);

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

    /// Detect relationships between files based on imports and file content
    fn detect_relationships(
        &self,
        content: &str,
        language: &str,
        file_path: &Path,
        root_path: &PathBuf,
        imports: &[String],
    ) -> Vec<super::FileRelationship> {
        let mut relationships = Vec::new();

        // Analyze import-based relationships
        for import in imports {
            if let Some(target_path) = self.resolve_import_to_path(import, file_path, root_path, language) {
                relationships.push(super::FileRelationship {
                    target_file: target_path,
                    relationship_type: super::RelationshipType::Imports,
                    line_numbers: vec![], // Could be enhanced to track line numbers
                });
            }
        }

        // Analyze content-based relationships
        match language {
            "rust" => self.detect_rust_relationships(content, file_path, root_path, &mut relationships),
            "python" => self.detect_python_relationships(content, file_path, root_path, &mut relationships),
            "javascript" | "typescript" => self.detect_js_relationships(content, file_path, root_path, &mut relationships),
            _ => {}
        }

        // Detect test relationships
        if self.is_test_file(file_path) {
            if let Some(main_file) = self.find_main_file_for_test(file_path, root_path, language) {
                relationships.push(super::FileRelationship {
                    target_file: main_file,
                    relationship_type: super::RelationshipType::Tests,
                    line_numbers: vec![], // Could track which lines contain test references
                });
            }
        }

        // Detect configuration relationships
        if self.is_config_file(file_path) {
            // Config files typically relate to documentation or references
            // Since there's no Configuration type, we'll use Documentation for config files
            relationships.push(super::FileRelationship {
                target_file: root_path.clone(),
                relationship_type: super::RelationshipType::Documentation,
                line_numbers: vec![], // Config files generally reference the whole project
            });
        }

        relationships
    }

    /// Resolve an import statement to a file path
    fn resolve_import_to_path(
        &self,
        import: &str,
        current_file: &Path,
        root_path: &PathBuf,
        language: &str,
    ) -> Option<PathBuf> {
        match language {
            "rust" => self.resolve_rust_import(import, current_file, root_path),
            "python" => self.resolve_python_import(import, current_file, root_path),
            "javascript" | "typescript" => self.resolve_js_import(import, current_file, root_path),
            _ => None,
        }
    }

    /// Resolve Rust imports (use/mod statements)
    fn resolve_rust_import(&self, import: &str, current_file: &Path, root_path: &PathBuf) -> Option<PathBuf> {
        // Handle relative imports like "super::module" or "crate::module"
        if import.starts_with("crate::") {
            let module_path = import.strip_prefix("crate::")?;
            let path_parts: Vec<&str> = module_path.split("::").collect();
            let mut file_path = root_path.join("src");
            
            for part in &path_parts {
                file_path = file_path.join(part);
            }
            
            // Try .rs file first, then mod.rs in directory
            if file_path.with_extension("rs").exists() {
                return Some(file_path.with_extension("rs"));
            } else if file_path.join("mod.rs").exists() {
                return Some(file_path.join("mod.rs"));
            }
        } else if import.starts_with("super::") {
            // Handle parent module imports
            if let Some(parent) = current_file.parent() {
                let module_path = import.strip_prefix("super::")?;
                let path_parts: Vec<&str> = module_path.split("::").collect();
                let mut file_path = parent.to_path_buf();
                
                for part in &path_parts {
                    file_path = file_path.join(part);
                }
                
                if file_path.with_extension("rs").exists() {
                    return Some(file_path.with_extension("rs"));
                }
            }
        } else if !import.contains("::") {
            // Simple module name - look for sibling file
            if let Some(parent) = current_file.parent() {
                let sibling_file = parent.join(format!("{}.rs", import));
                if sibling_file.exists() {
                    return Some(sibling_file);
                }
            }
        }
        
        None
    }

    /// Resolve Python imports
    fn resolve_python_import(&self, import: &str, current_file: &Path, root_path: &PathBuf) -> Option<PathBuf> {
        // Handle relative imports
        if import.starts_with(".") {
            if let Some(parent) = current_file.parent() {
                let clean_import = import.trim_start_matches('.');
                let path_parts: Vec<&str> = clean_import.split('.').collect();
                let mut file_path = parent.to_path_buf();
                
                for part in &path_parts {
                    if !part.is_empty() {
                        file_path = file_path.join(part);
                    }
                }
                
                // Try .py file first, then __init__.py in directory
                if file_path.with_extension("py").exists() {
                    return Some(file_path.with_extension("py"));
                } else if file_path.join("__init__.py").exists() {
                    return Some(file_path.join("__init__.py"));
                }
            }
        } else {
            // Absolute import from project root
            let path_parts: Vec<&str> = import.split('.').collect();
            let mut file_path = root_path.clone();
            
            for part in &path_parts {
                file_path = file_path.join(part);
            }
            
            if file_path.with_extension("py").exists() {
                return Some(file_path.with_extension("py"));
            } else if file_path.join("__init__.py").exists() {
                return Some(file_path.join("__init__.py"));
            }
        }
        
        None
    }

    /// Resolve JavaScript/TypeScript imports
    fn resolve_js_import(&self, import: &str, current_file: &Path, _root_path: &PathBuf) -> Option<PathBuf> {
        // Handle relative imports
        if import.starts_with(".") {
            if let Some(parent) = current_file.parent() {
                let import_path = parent.join(import.trim_start_matches("./"));
                
                // Try different extensions
                for ext in &["js", "ts", "jsx", "tsx"] {
                    let with_ext = import_path.with_extension(ext);
                    if with_ext.exists() {
                        return Some(with_ext);
                    }
                }
                
                // Try index files
                for ext in &["js", "ts"] {
                    let index_file = import_path.join(format!("index.{}", ext));
                    if index_file.exists() {
                        return Some(index_file);
                    }
                }
            }
        }
        
        None
    }

    /// Detect Rust-specific relationships
    fn detect_rust_relationships(
        &self,
        content: &str,
        file_path: &Path,
        root_path: &PathBuf,
        relationships: &mut Vec<super::FileRelationship>,
    ) {
        // Look for trait implementations
        if content.contains("impl ") {
            // This is a simplified detection - in production, use proper parsing
            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("impl ") && trimmed.contains(" for ") {
                    relationships.push(super::FileRelationship {
                        target_file: file_path.to_path_buf(), // Self-reference for trait impl
                        relationship_type: super::RelationshipType::Implements,
                        line_numbers: vec![line_num + 1], // Track line number (1-based)
                    });
                }
            }
        }

        // Look for macro usage and references
        if content.contains('!') {
            let line_numbers: Vec<usize> = content
                .lines()
                .enumerate()
                .filter_map(|(i, line)| {
                    if line.contains('!') { Some(i + 1) } else { None }
                })
                .collect();
            
            if !line_numbers.is_empty() {
                relationships.push(super::FileRelationship {
                    target_file: file_path.to_path_buf(),
                    relationship_type: super::RelationshipType::References,
                    line_numbers,
                });
            }
        }
    }

    /// Detect Python-specific relationships
    fn detect_python_relationships(
        &self,
        content: &str,
        file_path: &Path,
        _root_path: &PathBuf,
        relationships: &mut Vec<super::FileRelationship>,
    ) {
        // Look for class inheritance
        if content.contains("class ") {
            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("class ") && trimmed.contains('(') {
                    relationships.push(super::FileRelationship {
                        target_file: file_path.to_path_buf(),
                        relationship_type: super::RelationshipType::Extends,
                        line_numbers: vec![line_num + 1],
                    });
                }
            }
        }

        // Look for decorator usage
        if content.contains('@') {
            let line_numbers: Vec<usize> = content
                .lines()
                .enumerate()
                .filter_map(|(i, line)| {
                    if line.contains('@') { Some(i + 1) } else { None }
                })
                .collect();
            
            if !line_numbers.is_empty() {
                relationships.push(super::FileRelationship {
                    target_file: file_path.to_path_buf(),
                    relationship_type: super::RelationshipType::References,
                    line_numbers,
                });
            }
        }
    }

    /// Detect JavaScript/TypeScript relationships
    fn detect_js_relationships(
        &self,
        content: &str,
        file_path: &Path,
        _root_path: &PathBuf,
        relationships: &mut Vec<super::FileRelationship>,
    ) {
        // Look for class extensions
        if content.contains("extends") {
            let line_numbers: Vec<usize> = content
                .lines()
                .enumerate()
                .filter_map(|(i, line)| {
                    if line.contains("extends") { Some(i + 1) } else { None }
                })
                .collect();
            
            if !line_numbers.is_empty() {
                relationships.push(super::FileRelationship {
                    target_file: file_path.to_path_buf(),
                    relationship_type: super::RelationshipType::Extends,
                    line_numbers,
                });
            }
        }

        // Look for interface implementations (TypeScript)
        if content.contains("implements") {
            let line_numbers: Vec<usize> = content
                .lines()
                .enumerate()
                .filter_map(|(i, line)| {
                    if line.contains("implements") { Some(i + 1) } else { None }
                })
                .collect();
            
            if !line_numbers.is_empty() {
                relationships.push(super::FileRelationship {
                    target_file: file_path.to_path_buf(),
                    relationship_type: super::RelationshipType::Implements,
                    line_numbers,
                });
            }
        }
    }

    /// Check if a file is a test file
    fn is_test_file(&self, file_path: &Path) -> bool {
        let path_str = file_path.to_string_lossy().to_lowercase();
        path_str.contains("test") || path_str.contains("spec") || 
        path_str.ends_with("_test.rs") || path_str.ends_with("_test.py") ||
        path_str.ends_with(".test.js") || path_str.ends_with(".spec.js") ||
        path_str.ends_with(".test.ts") || path_str.ends_with(".spec.ts")
    }

    /// Find the main file that a test file is testing
    fn find_main_file_for_test(&self, test_path: &Path, root_path: &PathBuf, language: &str) -> Option<PathBuf> {
        if let Some(filename) = test_path.file_stem() {
            let filename_str = filename.to_string_lossy();
            let main_filename = filename_str
                .trim_end_matches("_test")
                .trim_end_matches("_spec")
                .trim_end_matches(".test")
                .trim_end_matches(".spec");
            
            if let Some(parent) = test_path.parent() {
                let extension = match language {
                    "rust" => "rs",
                    "python" => "py",
                    "javascript" => "js",
                    "typescript" => "ts",
                    _ => return None,
                };
                
                let main_file = parent.join(format!("{}.{}", main_filename, extension));
                if main_file.exists() {
                    return Some(main_file);
                }
                
                // Also check in src directory for Rust
                if language == "rust" {
                    let src_main_file = root_path.join("src").join(format!("{}.{}", main_filename, extension));
                    if src_main_file.exists() {
                        return Some(src_main_file);
                    }
                }
            }
        }
        
        None
    }

    /// Check if a file is a configuration file
    fn is_config_file(&self, file_path: &Path) -> bool {
        if let Some(filename) = file_path.file_name() {
            let name = filename.to_string_lossy().to_lowercase();
            matches!(name.as_str(),
                "cargo.toml" | "package.json" | "requirements.txt" | "go.mod" |
                "tsconfig.json" | "webpack.config.js" | "babel.config.js" |
                "eslint.config.js" | ".eslintrc.json" | "jest.config.js" |
                "pyproject.toml" | "setup.py" | "makefile" | "dockerfile" |
                ".gitignore" | ".dockerignore" | "readme.md" | "license"
            ) || name.ends_with(".config.js") || name.ends_with(".config.ts") ||
               name.ends_with(".toml") || name.ends_with(".yml") || name.ends_with(".yaml")
        } else {
            false
        }
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
