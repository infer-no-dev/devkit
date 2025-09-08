//! Advanced code generation engine for the agentic development environment.
//!
//! This module provides intelligent code generation capabilities, including
//! natural language to code translation, code completion, and refactoring
//! suggestions based on codebase context.

pub mod analyzer;
pub mod generator;
pub mod language_detection;
pub mod templates;

use crate::ai::AIManager;
use crate::context::CodebaseContext;
use std::collections::HashMap;
use std::sync::Arc;

/// Main code generation engine
#[derive(Clone)]
pub struct CodeGenerator {
    analyzer: analyzer::CodeAnalyzer,
    generator: generator::CodeGen,
    language_detector: language_detection::LanguageDetector,
    templates: templates::TemplateManager,
    ai_manager: Option<Arc<AIManager>>,
}

/// Configuration for code generation
#[derive(Debug, Clone)]
pub struct GenerationConfig {
    pub target_language: Option<String>,
    pub style_preferences: StylePreferences,
    pub context_depth: usize,
    pub include_comments: bool,
    pub include_tests: bool,
    // AI-specific settings
    pub temperature: Option<f64>,
    pub max_tokens: Option<usize>,
    pub use_ai: bool,
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GenerationResult {
    pub generated_code: String,
    pub language: String,
    pub confidence_score: f64,
    pub suggestions: Vec<String>,
    pub modifications: Vec<CodeModification>,
    pub metadata: GenerationMetadata,
}

/// Metadata about the generation process
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GenerationMetadata {
    pub generation_time_ms: u64,
    pub tokens_used: usize,
    pub context_files_analyzed: usize,
    pub template_used: Option<String>,
}

/// A code modification suggestion
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    #[error("Language detection failed: {0}")]
    LanguageDetectionFailed(String),
    
    #[error("Code analysis failed: {0}")]
    AnalysisFailed(String),
    
    #[error("Generation failed: {0}")]
    GenerationFailed(String),
    
    #[error("Template error: {0}")]
    TemplateError(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Generation error: {0}")]
    GenerationError(String),
}

impl CodeGenerator {
    /// Create a new code generator
    pub fn new() -> Result<Self, CodeGenError> {
        Ok(Self {
            analyzer: analyzer::CodeAnalyzer::new()?,
            generator: generator::CodeGen::new()?,
            language_detector: language_detection::LanguageDetector::new(),
            templates: templates::TemplateManager::new()?,
            ai_manager: None,
        })
    }
    
    /// Set the AI manager for enhanced code generation
    pub fn set_ai_manager(&mut self, ai_manager: Arc<AIManager>) {
        self.generator.set_ai_manager(ai_manager.clone());
        self.ai_manager = Some(ai_manager);
    }
    
    /// Check if AI is available
    pub fn has_ai(&self) -> bool {
        self.ai_manager.is_some()
    }
    
    /// Generate code from a natural language prompt using AI when available
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
        
        // Use AI-powered generation if available, otherwise fallback to templates
        if self.has_ai() {
            self.generator.generate_with_ai(
                &request.prompt,
                &language,
                &request.context,
                &request.config,
            ).await
        } else {
            // Fallback to template-based generation
            let start_time = std::time::Instant::now();
            let generated_code = self.generate_with_templates(
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
                    tokens_used: 0, // Template-based generation doesn't use tokens
                    context_files_analyzed: request.context.files.len(),
                    template_used: None, // Will be populated by template selection logic
                },
            })
        }
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
    
    /// Generate code from a GenerationRequest - main entry point for CLI
    pub async fn generate_code(&self, request: &GenerationRequest) -> Result<String, CodeGenError> {
        if self.has_ai() {
            self.generate_code_with_ai(request).await
        } else {
            self.generate_code_with_templates(request).await
        }
    }
    
    /// Generate code using AI when available
    async fn generate_code_with_ai(&self, request: &GenerationRequest) -> Result<String, CodeGenError> {
        use crate::ai::{ChatMessage, ChatRequest};
        
        if let Some(ai_manager) = &self.ai_manager {
            // Build comprehensive prompt for AI
            let system_prompt = self.build_ai_system_prompt(&request);
            let user_prompt = self.build_ai_user_prompt(&request);
            
            let messages = vec![
                ChatMessage::system(system_prompt),
                ChatMessage::user(user_prompt),
            ];
            
            let mut parameters = crate::ai::ModelParameters::default();
            if let Some(temp) = request.config.temperature {
                parameters.temperature = Some(temp);
            }
            if let Some(max_tokens) = request.config.max_tokens {
                parameters.max_tokens = Some(max_tokens);
            }
            
            let chat_request = ChatRequest {
                model: "llama3.2:latest".to_string(), // Use default model
                messages,
                parameters: Some(parameters),
                stream: false,
            };
            
            // Generate using AI
            match ai_manager.chat_completion_default(chat_request).await {
                Ok(response) => {
                    let processed_code = self.post_process_ai_response(&response.message.content, request)?;
                    Ok(processed_code)
                },
                Err(e) => Err(CodeGenError::GenerationFailed(format!("AI generation failed: {}", e)))
            }
        } else {
            Err(CodeGenError::GenerationFailed("AI manager not available".to_string()))
        }
    }
    
    /// Generate code using templates as fallback
    async fn generate_code_with_templates(&self, request: &GenerationRequest) -> Result<String, CodeGenError> {
        let language = request.config.target_language.as_deref().unwrap_or("rust");
        let generated = self.generate_basic_code_structure(&request.prompt, language);
        Ok(generated)
    }
    
    fn build_ai_system_prompt(&self, request: &GenerationRequest) -> String {
        let language = request.config.target_language.as_deref().unwrap_or("the most appropriate language");
        
        format!(
            "You are an expert software developer. Generate high-quality, production-ready code in {}. \n\n\
            Requirements:\n\
            - Write clean, readable, well-documented code\n\
            - Follow language best practices and conventions\n\
            - Include appropriate error handling\n\
            - Add helpful comments where necessary\n\
            - Make reasonable assumptions if requirements are unclear\n\n\
            Return ONLY the code without explanations or markdown formatting.",
            language
        )
    }
    
    fn build_ai_user_prompt(&self, request: &GenerationRequest) -> String {
        let mut prompt = format!("Generate code for: {}\n", request.prompt);
        
        if let Some(language) = &request.config.target_language {
            prompt.push_str(&format!("Language: {}\n", language));
        }
        
        if let Some(file_path) = &request.file_path {
            prompt.push_str(&format!("Target file: {}\n", file_path));
        }
        
        if !request.constraints.is_empty() {
            prompt.push_str("\nConstraints:\n");
            for constraint in &request.constraints {
                prompt.push_str(&format!("- {}\n", constraint));
            }
        }
        
        // Add generation config hints
        if let Some(max_tokens) = request.config.max_tokens {
            if max_tokens < 500 {
                prompt.push_str("\nNote: Keep code concise due to length constraints.\n");
            }
        }
        
        prompt
    }
    
    fn post_process_ai_response(&self, response: &str, request: &GenerationRequest) -> Result<String, CodeGenError> {
        let mut code = response.trim().to_string();
        
        // Remove markdown code blocks if present
        if code.starts_with("```") {
            let lines: Vec<&str> = code.lines().collect();
            if lines.len() > 2 && lines.last().unwrap().trim() == "```" {
                code = lines[1..lines.len()-1].join("\n");
            }
        }
        
        // Add file header if requested
        if let Some(file_path) = &request.file_path {
            let header = self.generate_file_header(file_path, &request.prompt);
            code = format!("{}\n\n{}", header, code);
        }
        
        if code.trim().is_empty() {
            return Err(CodeGenError::GenerationFailed("AI generated empty code".to_string()));
        }
        
        Ok(code)
    }
    
    fn generate_file_header(&self, file_path: &str, prompt: &str) -> String {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        let short_prompt = prompt.lines().next().unwrap_or(prompt).chars().take(60).collect::<String>();
        
        format!(
            "// Generated by Agentic Dev Environment\n// File: {}\n// Created: {}\n// Prompt: {}",
            file_path, timestamp, short_prompt
        )
    }
    
    fn generate_basic_code_structure(&self, prompt: &str, language: &str) -> String {
        match language.to_lowercase().as_str() {
            "rust" => self.generate_rust_fallback(prompt),
            "python" => self.generate_python_fallback(prompt),
            "javascript" | "js" => self.generate_js_fallback(prompt),
            "typescript" | "ts" => self.generate_ts_fallback(prompt),
            "go" => self.generate_go_fallback(prompt),
            "java" => self.generate_java_fallback(prompt),
            _ => format!("// Generated code for: {}\n// TODO: Implement functionality", prompt)
        }
    }
    
    fn generate_rust_fallback(&self, prompt: &str) -> String {
        format!(
            "// Generated based on: {}\n\nuse std::{{error::Error, result::Result}};\n\n/// TODO: Implement based on prompt\npub fn generated_function() -> Result<(), Box<dyn Error>> {{\n    todo!(\"Implement: {}\");\n}}\n\n#[cfg(test)]\nmod tests {{\n    use super::*;\n\n    #[test]\n    fn test_generated_function() {{\n        // TODO: Add meaningful tests\n        assert!(true);\n    }}\n}}",
            prompt.lines().next().unwrap_or(prompt),
            prompt.lines().next().unwrap_or(prompt)
        )
    }
    
    fn generate_python_fallback(&self, prompt: &str) -> String {
        format!(
            "\"\"\"\nGenerated based on: {}\n\"\"\"\n\ndef generated_function():\n    \"\"\"TODO: Implement based on prompt\"\"\"\n    raise NotImplementedError(\"Implement: {}\")\n\n\nif __name__ == \"__main__\":\n    generated_function()",
            prompt.lines().next().unwrap_or(prompt),
            prompt.lines().next().unwrap_or(prompt)
        )
    }
    
    fn generate_js_fallback(&self, prompt: &str) -> String {
        format!(
            "/**\n * Generated based on: {}\n */\nfunction generatedFunction() {{\n    // TODO: Implement based on prompt\n    throw new Error('Implement: {}');\n}}\n\nmodule.exports = {{ generatedFunction }};",
            prompt.lines().next().unwrap_or(prompt),
            prompt.lines().next().unwrap_or(prompt)
        )
    }
    
    fn generate_ts_fallback(&self, prompt: &str) -> String {
        format!(
            "/**\n * Generated based on: {}\n */\nexport function generatedFunction(): void {{\n    // TODO: Implement based on prompt\n    throw new Error('Implement: {}');\n}}\n\nexport default generatedFunction;",
            prompt.lines().next().unwrap_or(prompt),
            prompt.lines().next().unwrap_or(prompt)
        )
    }
    
    fn generate_go_fallback(&self, prompt: &str) -> String {
        format!(
            "package main\n\nimport (\n\t\"fmt\"\n\t\"log\"\n)\n\n// GeneratedFunction based on: {}\nfunc GeneratedFunction() {{\n\tlog.Fatal(\"TODO: Implement: {}\")\n}}\n\nfunc main() {{\n\tGeneratedFunction()\n}}",
            prompt.lines().next().unwrap_or(prompt),
            prompt.lines().next().unwrap_or(prompt)
        )
    }
    
    fn generate_java_fallback(&self, prompt: &str) -> String {
        format!(
            "/**\n * Generated based on: {}\n */\npublic class GeneratedClass {{\n    \n    /**\n     * TODO: Implement based on prompt\n     */\n    public void generatedMethod() {{\n        throw new RuntimeException(\"Implement: {}\");\n    }}\n    \n    public static void main(String[] args) {{\n        new GeneratedClass().generatedMethod();\n    }}\n}}",
            prompt.lines().next().unwrap_or(prompt),
            prompt.lines().next().unwrap_or(prompt)
        )
    }
    
    /// Generate code using templates based on prompt analysis
    async fn generate_with_templates(
        &self,
        prompt: &str,
        language: &str,
        _context: &CodebaseContext,
        _config: &GenerationConfig,
    ) -> Result<String, CodeGenError> {
        // Analyze prompt to determine what template to use
        let template_name = self.select_template_from_prompt(prompt, language)?;
        
        // Extract variables from the prompt
        let variables = self.extract_variables_from_prompt(prompt, &template_name)?;
        
        // Apply the template
        let generated_code = self.templates.apply_template(&template_name, &variables)?;
        
        Ok(generated_code)
    }
    
    /// Select appropriate template based on prompt analysis
    fn select_template_from_prompt(&self, prompt: &str, language: &str) -> Result<String, CodeGenError> {
        let prompt_lower = prompt.to_lowercase();
        
        // Simple heuristics to determine template type
        if prompt_lower.contains("function") || prompt_lower.contains("method") || prompt_lower.contains("fn") {
            Ok(format!("{}_function", language))
        } else if prompt_lower.contains("struct") || prompt_lower.contains("class") {
            if language == "rust" && prompt_lower.contains("struct") {
                Ok("rust_struct".to_string())
            } else if language == "python" && prompt_lower.contains("class") {
                Ok("python_class".to_string())
            } else {
                // Default to function for ambiguous cases
                Ok(format!("{}_function", language))
            }
        } else if prompt_lower.contains("test") {
            Ok(format!("{}_test", language))
        } else if language == "rust" && prompt_lower.contains("impl") {
            Ok("rust_impl".to_string())
        } else {
            // Default to function template
            Ok(format!("{}_function", language))
        }
    }
    
    /// Extract template variables from the natural language prompt
    fn extract_variables_from_prompt(&self, prompt: &str, template_name: &str) -> Result<HashMap<String, String>, CodeGenError> {
        let mut variables = HashMap::new();
        
        // Get the template to understand what variables we need
        let template = self.templates.get_template(template_name)
            .ok_or_else(|| CodeGenError::TemplateError(format!("Template '{}' not found", template_name)))?;
        
        // Simple extraction logic - this can be enhanced with NLP
        let words: Vec<&str> = prompt.split_whitespace().collect();
        
        // Try to extract function/class/struct name
        for (i, word) in words.iter().enumerate() {
            if word.to_lowercase().contains("function") || word.to_lowercase().contains("method") ||
               word.to_lowercase().contains("struct") || word.to_lowercase().contains("class") {
                if let Some(next_word) = words.get(i + 1) {
                    // Clean the name (remove punctuation)
                    let name = next_word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
                    if !name.is_empty() {
                        variables.insert("name".to_string(), name.to_string());
                        break;
                    }
                }
            }
        }
        
        // If no name found, try to extract from "create/make/build X" patterns
        if !variables.contains_key("name") {
            for (i, word) in words.iter().enumerate() {
                if ["create", "make", "build", "implement", "write"].contains(&word.to_lowercase().as_str()) {
                    if let Some(next_word) = words.get(i + 1) {
                        let name = next_word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
                        if !name.is_empty() && name.len() > 1 {
                            variables.insert("name".to_string(), name.to_string());
                            break;
                        }
                    }
                }
            }
        }
        
        // Set default name if still not found
        if !variables.contains_key("name") {
            variables.insert("name".to_string(), "generated_item".to_string());
        }
        
        // Use the prompt as description
        variables.insert("description".to_string(), prompt.to_string());
        
        // Set reasonable defaults for other common variables
        if template.variables.iter().any(|v| v.name == "parameters" && !variables.contains_key("parameters")) {
            variables.insert("parameters".to_string(), "".to_string());
        }
        
        if template.variables.iter().any(|v| v.name == "return_type" && !variables.contains_key("return_type")) {
            if template_name.starts_with("rust") {
                variables.insert("return_type".to_string(), "()".to_string());
            }
        }
        
        Ok(variables)
    }
    
    /// Get available templates for a language
    pub fn get_templates_for_language(&self, language: &str) -> Vec<&templates::Template> {
        self.templates.get_templates_for_language(language)
    }
    
    /// List all available template names
    pub fn list_templates(&self) -> Vec<String> {
        self.templates.list_templates()
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
            temperature: Some(0.3),
            max_tokens: Some(2000),
            use_ai: true,
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
