//! Context management system for analyzing and indexing codebases.
//!
//! This module provides comprehensive context analysis including file structure,
//! symbol definitions, dependencies, and semantic relationships within codebases.

pub mod analyzer;
pub mod indexer;
pub mod repository;
pub mod symbols;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Complete context information about a codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseContext {
    pub root_path: PathBuf,
    pub files: Vec<FileContext>,
    pub symbols: symbols::SymbolIndex,
    pub dependencies: Vec<Dependency>,
    pub repository_info: Option<repository::RepositoryInfo>,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

/// Context manager for analyzing and maintaining codebase context
#[derive(Debug)]
pub struct ContextManager {
    analyzer: analyzer::CodebaseAnalyzer,
    indexer: indexer::SymbolIndexer,
    repository: repository::RepositoryAnalyzer,
    cache: HashMap<PathBuf, CodebaseContext>,
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
    #[error(\"Path not found: {0}\")]
    PathNotFound(PathBuf),
    
    #[error(\"Permission denied: {0}\")]
    PermissionDenied(String),
    
    #[error(\"Analysis failed: {0}\")]
    AnalysisFailed(String),
    
    #[error(\"Indexing failed: {0}\")]
    IndexingFailed(String),
    
    #[error(\"Repository analysis failed: {0}\")]
    RepositoryAnalysisFailed(String),
    
    #[error(\"Cache error: {0}\")]
    CacheError(String),
}

impl ContextManager {
    /// Create a new context manager
    pub fn new() -> Result<Self, ContextError> {
        Ok(Self {
            analyzer: analyzer::CodebaseAnalyzer::new()?,
            indexer: indexer::SymbolIndexer::new(),
            repository: repository::RepositoryAnalyzer::new()?,
            cache: HashMap::new(),
        })
    }
    
    /// Analyze a codebase and build comprehensive context
    pub async fn analyze_codebase(
        &mut self,
        path: PathBuf,
        config: AnalysisConfig,
    ) -> Result<CodebaseContext, ContextError> {
        // Check cache first
        if config.cache_results {
            if let Some(cached_context) = self.cache.get(&path) {
                return Ok(cached_context.clone());
            }
        }
        
        let start_time = std::time::Instant::now();
        
        // Analyze the file structure and content
        let files = self.analyzer.analyze_files(&path, &config).await?;
        
        // Build symbol index
        let symbols = self.indexer.index_symbols(&files).await?;
        
        // Analyze dependencies
        let dependencies = if config.analyze_dependencies {
            self.analyzer.analyze_dependencies(&path, &files).await?
        } else {
            Vec::new()
        };
        
        // Get repository information if available
        let repository_info = self.repository.analyze(&path).await.ok();
        
        let analysis_duration = start_time.elapsed();
        
        // Build metadata
        let metadata = ContextMetadata {
            analysis_timestamp: std::time::SystemTime::now(),
            total_files: files.len(),
            total_lines: files.iter().map(|f| f.line_count).sum(),
            languages: Self::count_languages(&files),
            analysis_duration_ms: analysis_duration.as_millis() as u64,
            indexed_symbols: symbols.total_symbols(),
        };
        
        let context = CodebaseContext {
            root_path: path.clone(),
            files,
            symbols,
            dependencies,
            repository_info,
            metadata,
        };
        
        // Cache the result
        if config.cache_results {
            self.cache.insert(path, context.clone());
        }
        
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
            if let Some(file_context) = context.files.iter()
                .find(|f| f.path == *path || f.relative_path == *path) {
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
        
        if let Some(file_context) = context.files.iter()
            .find(|f| f.path == *file_path || f.relative_path == *file_path) {
            
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
        // Re-analyze changed files
        let updated_files = self.analyzer.analyze_specific_files(changed_files, config).await?;
        
        // Update the context
        for updated_file in updated_files {
            if let Some(existing_file) = context.files.iter_mut()
                .find(|f| f.path == updated_file.path) {
                *existing_file = updated_file;
            } else {
                context.files.push(updated_file);
            }
        }
        
        // Re-index symbols for updated files
        self.indexer.update_symbols(&context.files, &mut context.symbols).await?;
        
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
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            include_patterns: vec![\"**/*\".to_string()],
            exclude_patterns: vec![
                \"**/target/**\".to_string(),
                \"**/node_modules/**\".to_string(),
                \"**/.git/**\".to_string(),
                \"**/.cache/**\".to_string(),
            ],
            max_file_size_mb: 10,
            follow_symlinks: false,
            analyze_dependencies: true,
            deep_analysis: true,
            cache_results: true,
        }
    }
}
