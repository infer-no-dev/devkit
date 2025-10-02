//! Context management system for analyzing and indexing codebases.
//!
//! This module provides comprehensive context analysis including file structure,
//! symbol definitions, dependencies, and semantic relationships within codebases.

pub mod analyzer;
pub mod indexer;
pub mod repository;
pub mod semantic;
pub mod symbols;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use semantic::{SemanticAnalysis, SemanticAnalyzer};

/// Complete context information about a codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseContext {
    pub root_path: PathBuf,
    pub files: Vec<FileContext>,
    pub symbols: symbols::SymbolIndex,
    pub dependencies: Vec<Dependency>,
    pub repository_info: Option<repository::RepositoryInfo>,
    pub semantic_analysis: Option<SemanticAnalysis>,
    pub metadata: ContextMetadata,
}

/// Context information for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContext {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub language: String,
    pub size_bytes: u64,
    pub line_count: usize,
    pub last_modified: std::time::SystemTime,
    pub content_hash: String,
    pub symbols: Vec<symbols::Symbol>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub relationships: Vec<FileRelationship>,
}

/// Relationship between files in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRelationship {
    pub target_file: PathBuf,
    pub relationship_type: RelationshipType,
    pub line_numbers: Vec<usize>,
}

/// Types of relationships between files
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RelationshipType {
    Imports,
    Extends,
    Implements,
    References,
    Tests,
    Documentation,
}

/// External dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: Option<String>,
    pub dependency_type: DependencyType,
    pub source: DependencySource,
}

/// Types of dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Runtime,
    Development,
    Build,
    Optional,
}

/// Source of dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencySource {
    PackageManager(String), // npm, cargo, pip, etc.
    System,
    Manual,
}

/// Metadata about the context analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    pub analysis_timestamp: std::time::SystemTime,
    pub total_files: usize,
    pub total_lines: usize,
    pub languages: HashMap<String, usize>,
    pub analysis_duration_ms: u64,
    pub indexed_symbols: usize,
    pub semantic_patterns_found: usize,
    pub semantic_relationships: usize,
    pub total_size_bytes: u64,
    pub language_breakdown: HashMap<String, usize>,
}

impl Default for ContextMetadata {
    fn default() -> Self {
        Self {
            analysis_timestamp: std::time::SystemTime::UNIX_EPOCH,
            total_files: 0,
            total_lines: 0,
            languages: HashMap::new(),
            analysis_duration_ms: 0,
            indexed_symbols: 0,
            semantic_patterns_found: 0,
            semantic_relationships: 0,
            total_size_bytes: 0,
            language_breakdown: HashMap::new(),
        }
    }
}

impl Default for CodebaseContext {
    fn default() -> Self {
        Self {
            root_path: PathBuf::from("."),
            files: Vec::new(),
            symbols: symbols::SymbolIndex::new(),
            dependencies: Vec::new(),
            repository_info: None,
            semantic_analysis: None,
            metadata: ContextMetadata::default(),
        }
    }
}

/// Context manager for analyzing and maintaining codebase context
#[derive(Debug)]
pub struct ContextManager {
    analyzer: analyzer::CodebaseAnalyzer,
    indexer: indexer::SymbolIndexer,
    repository: repository::RepositoryAnalyzer,
    semantic_analyzer: SemanticAnalyzer,
    cache: HashMap<PathBuf, CodebaseContext>,
    semantic_cache: HashMap<PathBuf, SemanticAnalysis>,
}

/// Configuration for context analysis
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub max_file_size_mb: usize,
    pub follow_symlinks: bool,
    pub analyze_dependencies: bool,
    pub deep_analysis: bool,
    pub cache_results: bool,
}

/// Errors that can occur during context analysis
#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),

    #[error("Indexing failed: {0}")]
    IndexingFailed(String),

    #[error("Repository analysis failed: {0}")]
    RepositoryAnalysisFailed(String),

    #[error("Cache error: {0}")]
    CacheError(String),
}

impl ContextManager {
    /// Create a new context manager
    pub fn new() -> Result<Self, ContextError> {
        Ok(Self {
            analyzer: analyzer::CodebaseAnalyzer::new()?,
            indexer: indexer::SymbolIndexer::new(),
            repository: repository::RepositoryAnalyzer::new()?,
            semantic_analyzer: SemanticAnalyzer::new(),
            cache: HashMap::new(),
            semantic_cache: HashMap::new(),
        })
    }

    /// Analyze a codebase and build comprehensive context
    pub async fn analyze_codebase(
        &mut self,
        path: PathBuf,
        config: AnalysisConfig,
    ) -> Result<CodebaseContext, ContextError> {
        let path_str = path.to_string_lossy().to_string();
        
        println!(
            "Starting codebase analysis for: {}",
            path_str
        );
        
        // Check cache first
        if config.cache_results {
            if let Some(cached_context) = self.cache.get(&path) {
                println!(
                    "Cache hit for codebase analysis: {}",
                    path_str
                );
                
                return Ok(cached_context.clone());
            }
        }

        let start_time = std::time::Instant::now();

        // Analyze the file structure and content
        println!("Analyzing file structure and content for: {}", path_str);
        let files = match self.analyzer.analyze_files(&path, &config).await {
            Ok(files) => {
                println!(
                    "Successfully analyzed {} files",
                    files.len()
                );
                files
            }
            Err(e) => {
                println!(
                    "Failed to analyze files for {}: {}",
                    path_str,
                    e
                );
                return Err(e);
            }
        };

        // Build symbol index
        println!(
            "Building symbol index for {} files",
            files.len()
        );
        
        let symbols = match self.indexer.index_symbols(&files).await {
            Ok(symbols) => {
                println!(
                    "Successfully indexed {} symbols",
                    symbols.total_symbols()
                );
                symbols
            }
            Err(e) => {
                println!(
                    "Failed to build symbol index for {}: {}",
                    path_str,
                    e
                );
                return Err(e);
            }
        };

        // Analyze dependencies
        let dependencies = if config.analyze_dependencies {
            println!("Analyzing dependencies for: {}", path_str);
            match self.analyzer.analyze_dependencies(&path, &files).await {
                Ok(deps) => {
                    println!(
                        "Found {} dependencies",
                        deps.len()
                    );
                    deps
                }
                Err(e) => {
                    println!(
                        "Dependency analysis failed for {}: {}",
                        path_str,
                        e
                    );
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        // Get repository information if available
        let repository_info = self.repository.analyze(&path).await.ok();

        // Perform semantic analysis if deep analysis is enabled
        let semantic_analysis = if config.deep_analysis {
            println!("Performing deep semantic analysis for: {}", path_str);
            
            // Create a preliminary context for semantic analysis
            let temp_context = CodebaseContext {
                root_path: path.clone(),
                files: files.clone(),
                symbols: symbols.clone(),
                dependencies: dependencies.clone(),
                repository_info: repository_info.clone(),
                semantic_analysis: None,
                metadata: ContextMetadata::default(), // Temporary metadata
            };

            // Check semantic cache first
            if let Some(cached_semantic) = self.semantic_cache.get(&path) {
                println!("Semantic analysis cache hit for: {}", path_str);
                Some(cached_semantic.clone())
            } else {
                let semantic_start = std::time::Instant::now();
                match self.semantic_analyzer.analyze(&temp_context).await {
                    Ok(analysis) => {
                        let semantic_duration = semantic_start.elapsed();
                        
                        println!(
                            "Semantic analysis completed in {}ms with {} patterns and {} relationships",
                            semantic_duration.as_millis(),
                            analysis.patterns.len(),
                            analysis.relationships.len()
                        );
                        
                        // Cache semantic analysis
                        if config.cache_results {
                            self.semantic_cache.insert(path.clone(), analysis.clone());
                        }
                        Some(analysis)
                    }
                    Err(err) => {
                        println!(
                            "Semantic analysis failed for {}: {} (took {}ms)",
                            path_str,
                            err,
                            semantic_start.elapsed().as_millis()
                        );
                        
                        None
                    }
                }
            }
        } else {
            None
        };

        let analysis_duration = start_time.elapsed();
        let total_lines: usize = files.iter().map(|f| f.line_count).sum();
        let total_size_bytes: u64 = files.iter().map(|f| f.size_bytes).sum();
        let languages = Self::count_languages(&files);
        let total_symbols = symbols.total_symbols();
        let semantic_patterns = semantic_analysis.as_ref().map(|s| s.patterns.len()).unwrap_or(0);
        let semantic_relationships = semantic_analysis.as_ref().map(|s| s.relationships.len()).unwrap_or(0);

        // Build metadata
        let metadata = ContextMetadata {
            analysis_timestamp: std::time::SystemTime::now(),
            total_files: files.len(),
            total_lines,
            languages: languages.clone(),
            analysis_duration_ms: analysis_duration.as_millis() as u64,
            indexed_symbols: total_symbols,
            semantic_patterns_found: semantic_patterns,
            semantic_relationships,
            total_size_bytes,
            language_breakdown: languages.clone(),
        };

        let context = CodebaseContext {
            root_path: path.clone(),
            files,
            symbols,
            dependencies,
            repository_info,
            semantic_analysis,
            metadata,
        };

        // Cache the result
        if config.cache_results {
            self.cache.insert(path.clone(), context.clone());
        }

        println!(
            "Codebase analysis completed for {} in {}ms: {} files, {} lines, {} symbols, {} languages",
            path_str,
            analysis_duration.as_millis(),
            context.metadata.total_files,
            total_lines,
            total_symbols,
            languages.len()
        );

        Ok(context)
    }

    /// Get context for specific files within a codebase
    pub async fn get_file_context(
        &self,
        file_paths: &[PathBuf],
        context: &CodebaseContext,
    ) -> Result<Vec<FileContext>, ContextError> {
        let mut file_contexts = Vec::new();

        for path in file_paths {
            if let Some(file_context) = context
                .files
                .iter()
                .find(|f| f.path == *path || f.relative_path == *path)
            {
                file_contexts.push(file_context.clone());
            }
        }

        Ok(file_contexts)
    }

    /// Find related files based on relationships
    pub fn find_related_files(
        &self,
        file_path: &PathBuf,
        context: &CodebaseContext,
        relationship_types: &[RelationshipType],
    ) -> Vec<PathBuf> {
        let mut related_files = HashSet::new();

        if let Some(file_context) = context
            .files
            .iter()
            .find(|f| f.path == *file_path || f.relative_path == *file_path)
        {
            for relationship in &file_context.relationships {
                if relationship_types.contains(&relationship.relationship_type) {
                    related_files.insert(relationship.target_file.clone());
                }
            }
        }

        related_files.into_iter().collect()
    }

    /// Search for symbols in the context
    pub fn search_symbols(
        &self,
        query: &str,
        context: &CodebaseContext,
        symbol_types: Option<&[symbols::SymbolType]>,
    ) -> Vec<symbols::Symbol> {
        context.symbols.search(query, symbol_types)
    }

    /// Update context for changed files
    pub async fn update_context(
        &mut self,
        changed_files: &[PathBuf],
        context: &mut CodebaseContext,
        config: &AnalysisConfig,
    ) -> Result<(), ContextError> {
        let update_start = std::time::Instant::now();
        let changed_files_count = changed_files.len();
        
        println!(
            "Updating context for {} changed files in {}",
            changed_files_count,
            context.root_path.to_string_lossy()
        );
        
        // Re-analyze changed files
        let updated_files = match self
            .analyzer
            .analyze_specific_files(changed_files, config)
            .await {
                Ok(files) => {
                    println!("Successfully re-analyzed {} changed files", files.len());
                    files
                }
                Err(e) => {
                    println!("Failed to re-analyze changed files: {}", e);
                    return Err(e);
                }
            };

        let mut updated_count = 0;
        let mut added_count = 0;
        
        // Update the context
        for updated_file in updated_files {
            if let Some(existing_file) = context
                .files
                .iter_mut()
                .find(|f| f.path == updated_file.path)
            {
                *existing_file = updated_file;
                updated_count += 1;
            } else {
                context.files.push(updated_file);
                added_count += 1;
            }
        }

        // Re-index symbols for updated files
        match self.indexer
            .update_symbols(&context.files, &mut context.symbols)
            .await {
                Ok(()) => {
                    println!("Symbol index updated successfully");
                }
                Err(e) => {
                    println!("Failed to update symbol index: {}", e);
                    return Err(e);
                }
            }

        let update_duration = update_start.elapsed();
        
        println!(
            "Context update completed in {}ms: {} files updated, {} files added",
            update_duration.as_millis(),
            updated_count,
            added_count
        );

        Ok(())
    }

    /// Helper function to count languages in files
    fn count_languages(files: &[FileContext]) -> HashMap<String, usize> {
        let mut language_counts = HashMap::new();
        for file in files {
            *language_counts.entry(file.language.clone()).or_insert(0) += 1;
        }
        language_counts
    }

    /// Get semantic analysis for a codebase if available
    pub fn get_semantic_analysis<'a>(
        &self,
        context: &'a CodebaseContext,
    ) -> Option<&'a SemanticAnalysis> {
        context.semantic_analysis.as_ref()
    }

    /// Perform standalone semantic analysis on existing context
    pub async fn analyze_semantics(
        &mut self,
        context: &CodebaseContext,
    ) -> Result<SemanticAnalysis, ContextError> {
        self.semantic_analyzer
            .analyze(context)
            .await
            .map_err(|e| ContextError::AnalysisFailed(format!("Semantic analysis failed: {}", e)))
    }

    /// Get context suggestions based on semantic analysis
    pub fn get_context_suggestions(
        &self,
        context: &CodebaseContext,
        query: &str,
    ) -> Vec<semantic::ContextSuggestion> {
        if let Some(semantic_analysis) = &context.semantic_analysis {
            // Simple keyword matching for suggestions - could be enhanced with AI
            semantic_analysis
                .context_suggestions
                .iter()
                .filter(|suggestion| {
                    suggestion
                        .description
                        .to_lowercase()
                        .contains(&query.to_lowercase())
                        || suggestion
                            .rationale
                            .to_lowercase()
                            .contains(&query.to_lowercase())
                })
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Clear semantic cache
    pub fn clear_semantic_cache(&mut self) {
        self.semantic_cache.clear();
    }
    
    /// Analyze directory with optional breakdown for profiling
    pub async fn analyze_directory(
        &mut self,
        path: &std::path::Path,
        include_breakdown: bool,
    ) -> Result<CodebaseContext, ContextError> {
        let mut config = AnalysisConfig::default();
        config.deep_analysis = include_breakdown;
        
        self.analyze_codebase(path.to_path_buf(), config).await
    }
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            include_patterns: vec!["**/*".to_string()],
            exclude_patterns: vec![
                // Build directories
                "**/target/**".to_string(),
                "**/build/**".to_string(),
                "**/dist/**".to_string(),
                "**/out/**".to_string(),
                "**/bin/**".to_string(),
                // Package managers and dependencies
                "**/node_modules/**".to_string(),
                "**/__pycache__/**".to_string(),
                "**/venv/**".to_string(),
                "**/env/**".to_string(),
                // Version control and cache
                "**/.git/**".to_string(),
                "**/.svn/**".to_string(),
                "**/.hg/**".to_string(),
                "**/.cache/**".to_string(),
                "**/.tmp/**".to_string(),
                "**/tmp/**".to_string(),
                // IDE and editor files
                "**/.idea/**".to_string(),
                "**/.vscode/**".to_string(),
                "**/.vs/**".to_string(),
                "**/*.swp".to_string(),
                "**/*.swo".to_string(),
                "**/*~".to_string(),
                // OS files
                "**/.DS_Store".to_string(),
                "**/Thumbs.db".to_string(),
                // Log and debug files
                "**/*.log".to_string(),
                "**/logs/**".to_string(),
            ],
            max_file_size_mb: 5, // Reduced from 10MB to 5MB for better performance
            follow_symlinks: false,
            analyze_dependencies: true,
            deep_analysis: false, // Changed to false for better performance
            cache_results: true,
        }
    }
}
