//! Advanced code generation engine for the agentic development environment.
//!
//! This module provides intelligent code generation capabilities, including
//! natural language to code translation, code completion, and refactoring
//! suggestions based on codebase context.

pub mod analyzer;
pub mod generator;
pub mod language_detection;
pub mod templates;

use crate::context::CodebaseContext;
use std::collections::HashMap;

/// Main code generation engine
#[derive(Debug)]
pub struct CodeGenerator {
    analyzer: analyzer::CodeAnalyzer,
    generator: generator::CodeGen,
    language_detector: language_detection::LanguageDetector,
    templates: templates::TemplateManager,
}

/// Configuration for code generation
#[derive(Debug, Clone)]
pub struct GenerationConfig {
    pub target_language: Option<String>,
    pub style_preferences: StylePreferences,
    pub context_depth: usize,
    pub include_comments: bool,
    pub include_tests: bool,
}

/// Code style preferences
#[derive(Debug, Clone)]
pub struct StylePreferences {
    pub indentation: IndentationStyle,
    pub line_length: usize,
    pub naming_convention: NamingConvention,
    pub formatting_rules: HashMap<String, String>,
}

/// Indentation style options
#[derive(Debug, Clone)]
pub enum IndentationStyle {
    Spaces(usize),
    Tabs,
}

/// Naming convention options
#[derive(Debug, Clone)]
pub enum NamingConvention {
    CamelCase,
    PascalCase,
    SnakeCase,
    KebabCase,
    Auto, // Detect from existing codebase
}

/// Request for code generation
#[derive(Debug, Clone)]
pub struct GenerationRequest {
    pub prompt: String,
    pub file_path: Option<String>,
    pub context: CodebaseContext,
    pub config: GenerationConfig,
    pub constraints: Vec<String>,
}

/// Result of code generation
#[derive(Debug, Clone)]
pub struct GenerationResult {
    pub generated_code: String,
    pub language: String,
    pub confidence_score: f64,
    pub suggestions: Vec<String>,
    pub modifications: Vec<CodeModification>,
    pub metadata: GenerationMetadata,
}

/// Metadata about the generation process
#[derive(Debug, Clone)]
pub struct GenerationMetadata {
    pub generation_time_ms: u64,
    pub tokens_used: usize,
    pub context_files_analyzed: usize,
    pub template_used: Option<String>,
}

/// A code modification suggestion
#[derive(Debug, Clone)]
pub struct CodeModification {
    pub file_path: String,
    pub line_start: usize,
    pub line_end: usize,
    pub original_code: String,
    pub suggested_code: String,
    pub reason: String,
    pub confidence: f64,
}

/// Errors that can occur during code generation
#[derive(Debug, thiserror::Error)]
pub enum CodeGenError {
    #[error(\"Language detection failed: {0}\")]
    LanguageDetectionFailed(String),
    
    #[error(\"Code analysis failed: {0}\")]
    AnalysisFailed(String),
    
    #[error(\"Generation failed: {0}\")]
    GenerationFailed(String),
    
    #[error(\"Template error: {0}\")]
    TemplateError(String),
    
    #[error(\"Invalid configuration: {0}\")]
    InvalidConfig(String),
}

impl CodeGenerator {
    /// Create a new code generator
    pub fn new() -> Result<Self, CodeGenError> {
        Ok(Self {
            analyzer: analyzer::CodeAnalyzer::new()?,
            generator: generator::CodeGen::new()?,
            language_detector: language_detection::LanguageDetector::new(),
            templates: templates::TemplateManager::new()?,
        })
    }
    
    /// Generate code from a natural language prompt
    pub async fn generate_from_prompt(
        &self,
        request: GenerationRequest,
    ) -> Result<GenerationResult, CodeGenError> {
        // Analyze the prompt and determine intent
        let intent = self.analyzer.analyze_prompt(&request.prompt)?;
        
        // Detect or confirm the target language
        let language = match &request.config.target_language {
            Some(lang) => lang.clone(),
            None => self.language_detector.detect_from_context(&request.context)?,
        };
        
        // Generate the code
        let start_time = std::time::Instant::now();
        let generated_code = self.generator.generate(
            &request.prompt,
            &language,
            &request.context,
            &request.config,
        ).await?;
        
        let generation_time = start_time.elapsed();
        
        // Analyze the generated code for improvements
        let suggestions = self.analyzer.analyze_generated_code(&generated_code, &language)?;
        
        Ok(GenerationResult {
            generated_code,
            language,
            confidence_score: intent.confidence,
            suggestions,
            modifications: Vec::new(), // Will be populated by analyzer
            metadata: GenerationMetadata {
                generation_time_ms: generation_time.as_millis() as u64,
                tokens_used: 0, // Will be populated by generator
                context_files_analyzed: request.context.files.len(),
                template_used: None, // Will be populated by generator
            },
        })
    }
    
    /// Suggest code improvements for existing code
    pub async fn suggest_improvements(
        &self,
        code: &str,
        file_path: &str,
        context: &CodebaseContext,
    ) -> Result<Vec<CodeModification>, CodeGenError> {
        self.analyzer.suggest_improvements(code, file_path, context).await
    }
    
    /// Complete code based on partial input
    pub async fn complete_code(
        &self,
        partial_code: &str,
        cursor_position: usize,
        context: &CodebaseContext,
    ) -> Result<Vec<String>, CodeGenError> {
        self.generator.complete_code(partial_code, cursor_position, context).await
    }
    
    /// Refactor code according to specified rules
    pub async fn refactor_code(
        &self,
        code: &str,
        refactor_type: RefactorType,
        context: &CodebaseContext,
    ) -> Result<GenerationResult, CodeGenError> {
        self.generator.refactor(code, refactor_type, context).await
    }
}

/// Types of refactoring operations
#[derive(Debug, Clone)]
pub enum RefactorType {
    ExtractFunction {
        start_line: usize,
        end_line: usize,
        function_name: String,
    },
    RenameSymbol {
        old_name: String,
        new_name: String,
    },
    SimplifyLogic,
    OptimizePerformance,
    ImproveReadability,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            target_language: None,
            style_preferences: StylePreferences::default(),
            context_depth: 3,
            include_comments: true,
            include_tests: false,
        }
    }
}

impl Default for StylePreferences {
    fn default() -> Self {
        Self {
            indentation: IndentationStyle::Spaces(4),
            line_length: 100,
            naming_convention: NamingConvention::Auto,
            formatting_rules: HashMap::new(),
        }
    }
}
