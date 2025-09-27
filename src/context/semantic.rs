//! Semantic analysis and pattern detection for advanced context understanding.

use crate::context::symbols::SymbolType;
use crate::context::{CodebaseContext, RelationshipType};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Semantic analyzer for understanding code patterns and relationships
#[derive(Debug)]
pub struct SemanticAnalyzer {
    pattern_cache: HashMap<String, CodePattern>,
    naming_analysis: NamingAnalyzer,
    architectural_analyzer: ArchitecturalAnalyzer,
}

/// Code pattern detection and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePattern {
    pub pattern_type: PatternType,
    pub confidence: f64,
    pub occurrences: usize,
    pub examples: Vec<PatternExample>,
    pub files_affected: HashSet<PathBuf>,
}

/// Types of detectable code patterns
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PatternType {
    NamingConvention(String),
    ErrorHandling(String),
    AsyncPattern(String),
    TestingPattern(String),
    ImportOrganization(String),
    ArchitecturalPattern(String),
}

/// Example of a pattern occurrence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternExample {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub symbol_name: String,
    pub context: String,
}

/// Enhanced semantic relationship information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticRelationship {
    pub source_file: PathBuf,
    pub target_file: PathBuf,
    pub relationship_type: RelationshipType,
    pub semantic_strength: f64,
    pub dependency_direction: DependencyDirection,
    pub coupling_type: CouplingType,
}

/// Direction of dependency relationship
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DependencyDirection {
    OneWay,
    Bidirectional,
    Circular,
}

/// Type of coupling between modules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CouplingType {
    Tight,    // High interdependence
    Loose,    // Low interdependence
    Cohesive, // Related functionality
    Utility,  // Helper/utility usage
}

/// Comprehensive semantic analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAnalysis {
    pub patterns: HashMap<PatternType, CodePattern>,
    pub relationships: Vec<SemanticRelationship>,
    pub naming_insights: NamingInsights,
    pub architectural_insights: ArchitecturalInsights,
    pub context_suggestions: Vec<ContextSuggestion>,
}

/// Insights about naming conventions and patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingInsights {
    pub dominant_convention: Option<String>,
    pub consistency_score: f64,
    pub convention_by_type: HashMap<SymbolType, String>,
    pub inconsistencies: Vec<NamingInconsistency>,
}

/// Architectural insights and recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalInsights {
    pub detected_patterns: Vec<String>,
    pub module_organization: String,
    pub dependency_health: DependencyHealth,
    pub coupling_analysis: CouplingAnalysis,
}

/// Dependency health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyHealth {
    pub circular_dependencies: Vec<Vec<String>>,
    pub high_coupling_pairs: Vec<(String, String)>,
    pub isolated_modules: Vec<String>,
    pub dependency_depth: HashMap<String, usize>,
}

/// Coupling analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouplingAnalysis {
    pub overall_coupling: f64,
    pub cohesion_score: f64,
    pub hotspots: Vec<String>,
}

/// Naming inconsistency detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingInconsistency {
    pub file_path: PathBuf,
    pub symbol_name: String,
    pub expected_convention: String,
    pub actual_convention: String,
    pub suggestion: String,
}

/// Context-based suggestions for code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSuggestion {
    pub suggestion_type: SuggestionType,
    pub description: String,
    pub rationale: String,
    pub confidence: f64,
    pub applicable_files: Vec<PathBuf>,
}

/// Types of context-based suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    NamingConvention,
    ErrorHandling,
    ModuleOrganization,
    TestingStrategy,
    DocumentationImprovement,
    RefactoringOpportunity,
}

/// Naming convention analyzer
#[derive(Debug)]
pub struct NamingAnalyzer {
    convention_patterns: HashMap<String, regex::Regex>,
}

/// Architectural pattern analyzer
#[derive(Debug)]
pub struct ArchitecturalAnalyzer {
    known_patterns: HashMap<String, ArchitecturalPattern>,
}

/// Definition of an architectural pattern
#[derive(Debug, Clone)]
pub struct ArchitecturalPattern {
    pub name: String,
    pub indicators: Vec<String>,
    pub file_patterns: Vec<String>,
    pub symbol_patterns: Vec<String>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            pattern_cache: HashMap::new(),
            naming_analysis: NamingAnalyzer::new(),
            architectural_analyzer: ArchitecturalAnalyzer::new(),
        }
    }

    /// Perform comprehensive semantic analysis of a codebase
    pub async fn analyze(
        &mut self,
        context: &CodebaseContext,
    ) -> Result<SemanticAnalysis, crate::context::ContextError> {
        // Detect code patterns
        let patterns = self.detect_patterns(context).await?;

        // Analyze semantic relationships
        let relationships = self.analyze_relationships(context).await?;

        // Analyze naming conventions
        let naming_insights = self.naming_analysis.analyze(context).await?;

        // Analyze architectural patterns
        let architectural_insights = self
            .architectural_analyzer
            .analyze(context, &relationships)
            .await?;

        // Generate context-based suggestions
        let context_suggestions = self
            .generate_suggestions(context, &patterns, &naming_insights)
            .await?;

        Ok(SemanticAnalysis {
            patterns,
            relationships,
            naming_insights,
            architectural_insights,
            context_suggestions,
        })
    }

    /// Detect various code patterns in the codebase
    async fn detect_patterns(
        &mut self,
        context: &CodebaseContext,
    ) -> Result<HashMap<PatternType, CodePattern>, crate::context::ContextError> {
        let mut patterns = HashMap::new();

        // Detect naming patterns
        self.detect_naming_patterns(context, &mut patterns).await?;

        // Detect error handling patterns
        self.detect_error_handling_patterns(context, &mut patterns)
            .await?;

        // Detect async patterns
        self.detect_async_patterns(context, &mut patterns).await?;

        // Detect testing patterns
        self.detect_testing_patterns(context, &mut patterns).await?;

        // Detect import organization patterns
        self.detect_import_patterns(context, &mut patterns).await?;

        Ok(patterns)
    }

    /// Analyze semantic relationships between files
    async fn analyze_relationships(
        &self,
        context: &CodebaseContext,
    ) -> Result<Vec<SemanticRelationship>, crate::context::ContextError> {
        let mut relationships = Vec::new();

        for file in &context.files {
            for relationship in &file.relationships {
                let semantic_rel = self.create_semantic_relationship(
                    &file.path,
                    &relationship.target_file,
                    &relationship.relationship_type,
                    context,
                )?;
                relationships.push(semantic_rel);
            }
        }

        Ok(relationships)
    }

    /// Generate context-based suggestions for code improvement
    async fn generate_suggestions(
        &self,
        context: &CodebaseContext,
        patterns: &HashMap<PatternType, CodePattern>,
        naming_insights: &NamingInsights,
    ) -> Result<Vec<ContextSuggestion>, crate::context::ContextError> {
        let mut suggestions = Vec::new();

        // Generate naming convention suggestions
        if naming_insights.consistency_score < 0.8 {
            suggestions.push(ContextSuggestion {
                suggestion_type: SuggestionType::NamingConvention,
                description: "Improve naming consistency across the codebase".to_string(),
                rationale: format!(
                    "Current consistency score: {:.2}",
                    naming_insights.consistency_score
                ),
                confidence: 1.0 - naming_insights.consistency_score,
                applicable_files: context.files.iter().map(|f| f.path.clone()).collect(),
            });
        }

        // Generate error handling suggestions
        if let Some(error_pattern) =
            patterns.get(&PatternType::ErrorHandling("rust_result".to_string()))
        {
            if error_pattern.confidence < 0.6 {
                suggestions.push(ContextSuggestion {
                    suggestion_type: SuggestionType::ErrorHandling,
                    description: "Consider standardizing error handling patterns".to_string(),
                    rationale: "Inconsistent error handling detected across modules".to_string(),
                    confidence: 0.8,
                    applicable_files: error_pattern.files_affected.iter().cloned().collect(),
                });
            }
        }

        // Generate testing suggestions
        let test_coverage = self.calculate_test_coverage(context);
        if test_coverage < 0.5 {
            suggestions.push(ContextSuggestion {
                suggestion_type: SuggestionType::TestingStrategy,
                description: "Increase test coverage for better code reliability".to_string(),
                rationale: format!("Current test coverage: {:.2}%", test_coverage * 100.0),
                confidence: 0.9,
                applicable_files: context
                    .files
                    .iter()
                    .filter(|f| !f.path.to_string_lossy().contains("test"))
                    .map(|f| f.path.clone())
                    .collect(),
            });
        }

        Ok(suggestions)
    }

    // Pattern detection methods

    async fn detect_naming_patterns(
        &self,
        context: &CodebaseContext,
        patterns: &mut HashMap<PatternType, CodePattern>,
    ) -> Result<(), crate::context::ContextError> {
        let snake_case_pattern = self.detect_snake_case_usage(context)?;
        if let Some(pattern) = snake_case_pattern {
            patterns.insert(
                PatternType::NamingConvention("snake_case".to_string()),
                pattern,
            );
        }

        let camel_case_pattern = self.detect_camel_case_usage(context)?;
        if let Some(pattern) = camel_case_pattern {
            patterns.insert(
                PatternType::NamingConvention("camelCase".to_string()),
                pattern,
            );
        }

        Ok(())
    }

    async fn detect_error_handling_patterns(
        &self,
        context: &CodebaseContext,
        patterns: &mut HashMap<PatternType, CodePattern>,
    ) -> Result<(), crate::context::ContextError> {
        let mut result_usage = 0;
        let mut total_functions = 0;
        let mut examples = Vec::new();
        let mut files_affected = HashSet::new();

        for file in &context.files {
            if file.language == "rust" {
                for symbol in &file.symbols {
                    if symbol.symbol_type == SymbolType::Function {
                        total_functions += 1;
                        if let Some(signature) = &symbol.signature {
                            if signature.contains("Result<") {
                                result_usage += 1;
                                files_affected.insert(file.path.clone());
                                if examples.len() < 5 {
                                    examples.push(PatternExample {
                                        file_path: file.path.clone(),
                                        line_number: symbol.line_number,
                                        symbol_name: symbol.name.clone(),
                                        context: "Result return type".to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        if total_functions > 0 {
            let confidence = result_usage as f64 / total_functions as f64;
            patterns.insert(
                PatternType::ErrorHandling("rust_result".to_string()),
                CodePattern {
                    pattern_type: PatternType::ErrorHandling("rust_result".to_string()),
                    confidence,
                    occurrences: result_usage,
                    examples,
                    files_affected,
                },
            );
        }

        Ok(())
    }

    async fn detect_async_patterns(
        &self,
        context: &CodebaseContext,
        patterns: &mut HashMap<PatternType, CodePattern>,
    ) -> Result<(), crate::context::ContextError> {
        let mut async_usage = 0;
        let mut total_functions = 0;
        let mut examples = Vec::new();
        let mut files_affected = HashSet::new();

        for file in &context.files {
            if file.language == "rust" {
                for symbol in &file.symbols {
                    if symbol.symbol_type == SymbolType::Function {
                        total_functions += 1;
                        if let Some(signature) = &symbol.signature {
                            if signature.contains("async fn") {
                                async_usage += 1;
                                files_affected.insert(file.path.clone());
                                if examples.len() < 5 {
                                    examples.push(PatternExample {
                                        file_path: file.path.clone(),
                                        line_number: symbol.line_number,
                                        symbol_name: symbol.name.clone(),
                                        context: "Async function".to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        if total_functions > 0 {
            let confidence = async_usage as f64 / total_functions as f64;
            if confidence > 0.1 {
                // At least 10% async usage
                patterns.insert(
                    PatternType::AsyncPattern("rust_async".to_string()),
                    CodePattern {
                        pattern_type: PatternType::AsyncPattern("rust_async".to_string()),
                        confidence,
                        occurrences: async_usage,
                        examples,
                        files_affected,
                    },
                );
            }
        }

        Ok(())
    }

    async fn detect_testing_patterns(
        &self,
        context: &CodebaseContext,
        patterns: &mut HashMap<PatternType, CodePattern>,
    ) -> Result<(), crate::context::ContextError> {
        let mut test_functions = 0;
        let mut examples = Vec::new();
        let mut files_affected = HashSet::new();

        for file in &context.files {
            let path_str = file.path.to_string_lossy();
            if path_str.contains("test") || path_str.ends_with("_test.rs") {
                files_affected.insert(file.path.clone());
                for symbol in &file.symbols {
                    if symbol.symbol_type == SymbolType::Function {
                        if let Some(signature) = &symbol.signature {
                            if signature.contains("#[test]") || symbol.name.starts_with("test_") {
                                test_functions += 1;
                                if examples.len() < 5 {
                                    examples.push(PatternExample {
                                        file_path: file.path.clone(),
                                        line_number: symbol.line_number,
                                        symbol_name: symbol.name.clone(),
                                        context: "Test function".to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        if test_functions > 0 {
            patterns.insert(
                PatternType::TestingPattern("rust_unit_tests".to_string()),
                CodePattern {
                    pattern_type: PatternType::TestingPattern("rust_unit_tests".to_string()),
                    confidence: 1.0,
                    occurrences: test_functions,
                    examples,
                    files_affected,
                },
            );
        }

        Ok(())
    }

    async fn detect_import_patterns(
        &self,
        context: &CodebaseContext,
        patterns: &mut HashMap<PatternType, CodePattern>,
    ) -> Result<(), crate::context::ContextError> {
        let mut organized_imports = 0;
        let mut total_files_with_imports = 0;
        let mut examples = Vec::new();
        let mut files_affected = HashSet::new();

        for file in &context.files {
            if !file.imports.is_empty() {
                total_files_with_imports += 1;

                // Check if imports are organized (grouped by source)
                let std_imports = file
                    .imports
                    .iter()
                    .filter(|imp| imp.starts_with("std::"))
                    .count();
                let external_imports = file
                    .imports
                    .iter()
                    .filter(|imp| !imp.starts_with("std::") && !imp.starts_with("crate::"))
                    .count();
                let internal_imports = file
                    .imports
                    .iter()
                    .filter(|imp| imp.starts_with("crate::"))
                    .count();

                // If imports are grouped (std, external, internal), consider it organized
                if std_imports > 0 || external_imports > 0 || internal_imports > 0 {
                    organized_imports += 1;
                    files_affected.insert(file.path.clone());

                    if examples.len() < 5 {
                        examples.push(PatternExample {
                            file_path: file.path.clone(),
                            line_number: 1, // Imports are typically at the top
                            symbol_name: "imports".to_string(),
                            context: format!(
                                "Organized imports: std={}, ext={}, internal={}",
                                std_imports, external_imports, internal_imports
                            ),
                        });
                    }
                }
            }
        }

        if total_files_with_imports > 0 {
            let confidence = organized_imports as f64 / total_files_with_imports as f64;
            patterns.insert(
                PatternType::ImportOrganization("grouped_imports".to_string()),
                CodePattern {
                    pattern_type: PatternType::ImportOrganization("grouped_imports".to_string()),
                    confidence,
                    occurrences: organized_imports,
                    examples,
                    files_affected,
                },
            );
        }

        Ok(())
    }

    // Helper methods

    fn detect_snake_case_usage(
        &self,
        context: &CodebaseContext,
    ) -> Result<Option<CodePattern>, crate::context::ContextError> {
        let snake_case_regex = regex::Regex::new(r"^[a-z][a-z0-9_]*$").unwrap();
        let mut matching_symbols = 0;
        let mut total_symbols = 0;
        let mut examples = Vec::new();
        let mut files_affected = HashSet::new();

        for file in &context.files {
            for symbol in &file.symbols {
                if matches!(
                    symbol.symbol_type,
                    SymbolType::Function | SymbolType::Variable
                ) {
                    total_symbols += 1;
                    if snake_case_regex.is_match(&symbol.name) {
                        matching_symbols += 1;
                        files_affected.insert(file.path.clone());
                        if examples.len() < 5 {
                            examples.push(PatternExample {
                                file_path: file.path.clone(),
                                line_number: symbol.line_number,
                                symbol_name: symbol.name.clone(),
                                context: format!("{:?} symbol", symbol.symbol_type),
                            });
                        }
                    }
                }
            }
        }

        if total_symbols > 0 {
            let confidence = matching_symbols as f64 / total_symbols as f64;
            if confidence > 0.5 {
                return Ok(Some(CodePattern {
                    pattern_type: PatternType::NamingConvention("snake_case".to_string()),
                    confidence,
                    occurrences: matching_symbols,
                    examples,
                    files_affected,
                }));
            }
        }

        Ok(None)
    }

    fn detect_camel_case_usage(
        &self,
        context: &CodebaseContext,
    ) -> Result<Option<CodePattern>, crate::context::ContextError> {
        let camel_case_regex = regex::Regex::new(r"^[a-z][a-zA-Z0-9]*$").unwrap();
        let mut matching_symbols = 0;
        let mut total_symbols = 0;
        let mut examples = Vec::new();
        let mut files_affected = HashSet::new();

        for file in &context.files {
            for symbol in &file.symbols {
                if matches!(
                    symbol.symbol_type,
                    SymbolType::Function | SymbolType::Variable
                ) {
                    total_symbols += 1;
                    if camel_case_regex.is_match(&symbol.name) {
                        matching_symbols += 1;
                        files_affected.insert(file.path.clone());
                        if examples.len() < 5 {
                            examples.push(PatternExample {
                                file_path: file.path.clone(),
                                line_number: symbol.line_number,
                                symbol_name: symbol.name.clone(),
                                context: format!("{:?} symbol", symbol.symbol_type),
                            });
                        }
                    }
                }
            }
        }

        if total_symbols > 0 {
            let confidence = matching_symbols as f64 / total_symbols as f64;
            if confidence > 0.5 {
                return Ok(Some(CodePattern {
                    pattern_type: PatternType::NamingConvention("camelCase".to_string()),
                    confidence,
                    occurrences: matching_symbols,
                    examples,
                    files_affected,
                }));
            }
        }

        Ok(None)
    }

    fn create_semantic_relationship(
        &self,
        source: &PathBuf,
        target: &PathBuf,
        relationship_type: &RelationshipType,
        _context: &CodebaseContext,
    ) -> Result<SemanticRelationship, crate::context::ContextError> {
        let semantic_strength = match relationship_type {
            RelationshipType::Imports => 0.8,
            RelationshipType::Extends => 0.9,
            RelationshipType::Implements => 0.9,
            RelationshipType::References => 0.6,
            RelationshipType::Tests => 0.7,
            RelationshipType::Documentation => 0.3,
        };

        let coupling_type = match relationship_type {
            RelationshipType::Imports => CouplingType::Utility,
            RelationshipType::Extends | RelationshipType::Implements => CouplingType::Tight,
            RelationshipType::References => CouplingType::Cohesive,
            RelationshipType::Tests => CouplingType::Loose,
            RelationshipType::Documentation => CouplingType::Loose,
        };

        Ok(SemanticRelationship {
            source_file: source.clone(),
            target_file: target.clone(),
            relationship_type: relationship_type.clone(),
            semantic_strength,
            dependency_direction: DependencyDirection::OneWay,
            coupling_type,
        })
    }

    fn calculate_test_coverage(&self, context: &CodebaseContext) -> f64 {
        let total_files = context.files.len();
        let test_files = context
            .files
            .iter()
            .filter(|f| f.path.to_string_lossy().contains("test"))
            .count();

        if total_files > 0 {
            test_files as f64 / total_files as f64
        } else {
            0.0
        }
    }
}

impl NamingAnalyzer {
    pub fn new() -> Self {
        let mut convention_patterns = HashMap::new();

        convention_patterns.insert(
            "snake_case".to_string(),
            regex::Regex::new(r"^[a-z][a-z0-9_]*$").unwrap(),
        );
        convention_patterns.insert(
            "camelCase".to_string(),
            regex::Regex::new(r"^[a-z][a-zA-Z0-9]*$").unwrap(),
        );
        convention_patterns.insert(
            "PascalCase".to_string(),
            regex::Regex::new(r"^[A-Z][a-zA-Z0-9]*$").unwrap(),
        );
        convention_patterns.insert(
            "SCREAMING_SNAKE_CASE".to_string(),
            regex::Regex::new(r"^[A-Z][A-Z0-9_]*$").unwrap(),
        );

        Self {
            convention_patterns,
        }
    }

    pub async fn analyze(
        &self,
        context: &CodebaseContext,
    ) -> Result<NamingInsights, crate::context::ContextError> {
        let mut convention_counts = HashMap::new();
        let mut convention_by_type = HashMap::new();
        let mut inconsistencies = Vec::new();
        let mut total_symbols = 0;

        // Analyze each symbol type separately
        for symbol_type in [
            SymbolType::Function,
            SymbolType::Struct,
            SymbolType::Variable,
            SymbolType::Constant,
        ] {
            let mut type_convention_counts = HashMap::new();
            let mut type_symbols = 0;

            for file in &context.files {
                for symbol in &file.symbols {
                    if symbol.symbol_type == symbol_type {
                        type_symbols += 1;
                        total_symbols += 1;

                        let detected_convention = self.detect_naming_convention(&symbol.name);
                        *type_convention_counts
                            .entry(detected_convention.clone())
                            .or_insert(0) += 1;
                        *convention_counts
                            .entry(detected_convention.clone())
                            .or_insert(0) += 1;
                    }
                }
            }

            // Determine dominant convention for this symbol type
            if let Some((dominant_convention, _)) = type_convention_counts
                .iter()
                .max_by_key(|(_, count)| *count)
            {
                convention_by_type.insert(symbol_type, dominant_convention.clone());
            }
        }

        // Find dominant overall convention
        let dominant_convention = convention_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(convention, _)| convention.clone());

        // Calculate consistency score
        let consistency_score = if total_symbols > 0 {
            let max_count = convention_counts.values().max().unwrap_or(&0);
            *max_count as f64 / total_symbols as f64
        } else {
            1.0
        };

        // Find inconsistencies (symbols not following the dominant convention for their type)
        for file in &context.files {
            for symbol in &file.symbols {
                if let Some(expected_convention) = convention_by_type.get(&symbol.symbol_type) {
                    let actual_convention = self.detect_naming_convention(&symbol.name);
                    if actual_convention != *expected_convention {
                        inconsistencies.push(NamingInconsistency {
                            file_path: file.path.clone(),
                            symbol_name: symbol.name.clone(),
                            expected_convention: expected_convention.clone(),
                            actual_convention,
                            suggestion: self
                                .generate_naming_suggestion(&symbol.name, expected_convention),
                        });
                    }
                }
            }
        }

        Ok(NamingInsights {
            dominant_convention,
            consistency_score,
            convention_by_type,
            inconsistencies,
        })
    }

    fn detect_naming_convention(&self, symbol_name: &str) -> String {
        for (convention, pattern) in &self.convention_patterns {
            if pattern.is_match(symbol_name) {
                return convention.clone();
            }
        }
        "unknown".to_string()
    }

    fn generate_naming_suggestion(&self, symbol_name: &str, target_convention: &str) -> String {
        match target_convention {
            "snake_case" => self.to_snake_case(symbol_name),
            "camelCase" => self.to_camel_case(symbol_name),
            "PascalCase" => self.to_pascal_case(symbol_name),
            "SCREAMING_SNAKE_CASE" => self.to_screaming_snake_case(symbol_name),
            _ => symbol_name.to_string(),
        }
    }

    fn to_snake_case(&self, s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch.is_uppercase() && !result.is_empty() {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap_or(ch));
        }

        result
    }

    fn to_camel_case(&self, s: &str) -> String {
        let parts: Vec<&str> = s.split('_').collect();
        let mut result = String::new();

        for (i, part) in parts.iter().enumerate() {
            if i == 0 {
                result.push_str(&part.to_lowercase());
            } else {
                result.push_str(&self.capitalize_first(part));
            }
        }

        result
    }

    fn to_pascal_case(&self, s: &str) -> String {
        let parts: Vec<&str> = s.split('_').collect();
        parts
            .iter()
            .map(|part| self.capitalize_first(part))
            .collect()
    }

    fn to_screaming_snake_case(&self, s: &str) -> String {
        self.to_snake_case(s).to_uppercase()
    }

    fn capitalize_first(&self, s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => {
                first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
            }
        }
    }
}

impl ArchitecturalAnalyzer {
    pub fn new() -> Self {
        let mut known_patterns = HashMap::new();

        // Define known architectural patterns
        known_patterns.insert(
            "MVC".to_string(),
            ArchitecturalPattern {
                name: "MVC".to_string(),
                indicators: vec![
                    "model".to_string(),
                    "view".to_string(),
                    "controller".to_string(),
                ],
                file_patterns: vec![
                    "**/models/**".to_string(),
                    "**/views/**".to_string(),
                    "**/controllers/**".to_string(),
                ],
                symbol_patterns: vec![
                    "Controller".to_string(),
                    "Model".to_string(),
                    "View".to_string(),
                ],
            },
        );

        Self { known_patterns }
    }

    pub async fn analyze(
        &self,
        context: &CodebaseContext,
        relationships: &[SemanticRelationship],
    ) -> Result<ArchitecturalInsights, crate::context::ContextError> {
        let detected_patterns = self.detect_architectural_patterns(context);
        let module_organization = self.analyze_module_organization(context);
        let dependency_health = self.analyze_dependency_health(relationships);
        let coupling_analysis = self.analyze_coupling(relationships);

        Ok(ArchitecturalInsights {
            detected_patterns,
            module_organization,
            dependency_health,
            coupling_analysis,
        })
    }

    fn detect_architectural_patterns(&self, context: &CodebaseContext) -> Vec<String> {
        let mut detected = Vec::new();

        for (pattern_name, pattern) in &self.known_patterns {
            let mut indicator_count = 0;
            let total_indicators = pattern.indicators.len();

            for file in &context.files {
                let path_str = file.path.to_string_lossy().to_lowercase();
                for indicator in &pattern.indicators {
                    if path_str.contains(indicator) {
                        indicator_count += 1;
                        break;
                    }
                }
            }

            if indicator_count >= total_indicators / 2 {
                detected.push(pattern_name.clone());
            }
        }

        detected
    }

    fn analyze_module_organization(&self, context: &CodebaseContext) -> String {
        let mut lib_modules = 0;
        let mut bin_modules = 0;
        let mut test_modules = 0;
        let mut nested_depth = 0;

        for file in &context.files {
            let path_str = file.path.to_string_lossy();
            let depth = path_str.matches('/').count();
            nested_depth = nested_depth.max(depth);

            if path_str.contains("lib.rs") || path_str.contains("src/") {
                lib_modules += 1;
            } else if path_str.contains("main.rs") || path_str.contains("bin/") {
                bin_modules += 1;
            } else if path_str.contains("test") {
                test_modules += 1;
            }
        }

        if nested_depth > 4 {
            "hierarchical".to_string()
        } else if lib_modules > bin_modules {
            "library-centric".to_string()
        } else if bin_modules > 0 {
            "application-centric".to_string()
        } else {
            "flat".to_string()
        }
    }

    fn analyze_dependency_health(
        &self,
        relationships: &[SemanticRelationship],
    ) -> DependencyHealth {
        let mut file_dependencies = HashMap::new();
        let mut high_coupling_pairs = Vec::new();

        // Build dependency graph
        for rel in relationships {
            file_dependencies
                .entry(rel.source_file.clone())
                .or_insert(Vec::new())
                .push(rel.target_file.clone());

            if rel.semantic_strength > 0.8 && rel.coupling_type == CouplingType::Tight {
                let source_name = rel
                    .source_file
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let target_name = rel
                    .target_file
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                high_coupling_pairs.push((source_name, target_name));
            }
        }

        // Find isolated modules
        let mut isolated_modules = Vec::new();
        for file in file_dependencies.keys() {
            let deps = file_dependencies.get(file).unwrap();
            if deps.is_empty() {
                if let Some(file_name) = file.file_stem().and_then(|s| s.to_str()) {
                    isolated_modules.push(file_name.to_string());
                }
            }
        }

        // Calculate dependency depth (simplified)
        let dependency_depth = file_dependencies
            .iter()
            .map(|(file, deps)| {
                let file_name = file
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                (file_name, deps.len())
            })
            .collect();

        DependencyHealth {
            circular_dependencies: Vec::new(), // TODO: Implement circular dependency detection
            high_coupling_pairs,
            isolated_modules,
            dependency_depth,
        }
    }

    fn analyze_coupling(&self, relationships: &[SemanticRelationship]) -> CouplingAnalysis {
        let total_relationships = relationships.len() as f64;
        let high_coupling_count = relationships
            .iter()
            .filter(|r| r.semantic_strength > 0.7)
            .count() as f64;

        let overall_coupling = if total_relationships > 0.0 {
            high_coupling_count / total_relationships
        } else {
            0.0
        };

        // Calculate cohesion (simplified)
        let cohesive_relationships = relationships
            .iter()
            .filter(|r| r.coupling_type == CouplingType::Cohesive)
            .count() as f64;

        let cohesion_score = if total_relationships > 0.0 {
            cohesive_relationships / total_relationships
        } else {
            0.0
        };

        // Identify hotspots (files involved in many high-coupling relationships)
        let mut file_coupling_count = HashMap::new();
        for rel in relationships {
            if rel.semantic_strength > 0.7 {
                let source_name = rel
                    .source_file
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");
                *file_coupling_count
                    .entry(source_name.to_string())
                    .or_insert(0) += 1;
            }
        }

        let mut hotspots: Vec<_> = file_coupling_count
            .iter()
            .filter(|(_, count)| **count > 3)
            .map(|(file, _)| file.clone())
            .collect();
        hotspots.sort();

        CouplingAnalysis {
            overall_coupling,
            cohesion_score,
            hotspots,
        }
    }
}
