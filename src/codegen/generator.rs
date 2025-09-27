//! Core code generation engine.

use crate::ai::{AIManager, ChatMessage, ChatRequest, ModelParameters};
use crate::codegen::{CodeGenError, GenerationConfig, GenerationResult, RefactorType};
use crate::context::CodebaseContext;
use std::collections::HashMap;
use std::sync::Arc;

/// Additional error types for code generation
#[derive(Debug, thiserror::Error)]
pub enum GenerationError {
    #[error("AI generation failed: {0}")]
    AIGenerationFailed(String),

    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Invalid prompt: {0}")]
    InvalidPrompt(String),

    #[error("Context error: {0}")]
    ContextError(String),
}

/// Convert GenerationError to CodeGenError
impl From<GenerationError> for CodeGenError {
    fn from(err: GenerationError) -> Self {
        match err {
            GenerationError::AIGenerationFailed(msg) => CodeGenError::GenerationFailed(msg),
            GenerationError::TemplateNotFound(msg) => CodeGenError::TemplateError(msg),
            GenerationError::InvalidPrompt(msg) => CodeGenError::InvalidConfig(msg),
            GenerationError::ContextError(msg) => CodeGenError::AnalysisFailed(msg),
        }
    }
}

/// Main code generation engine
#[derive(Debug, Clone)]
pub struct CodeGen {
    templates: HashMap<String, CodeTemplate>,
    ai_manager: Option<Arc<AIManager>>,
}

/// Template for generating code snippets
#[derive(Debug, Clone)]
pub struct CodeTemplate {
    pub name: String,
    pub language: String,
    pub template: String,
    pub variables: Vec<String>,
}

impl CodeGen {
    /// Create a new code generator
    pub fn new() -> Result<Self, CodeGenError> {
        let mut templates = HashMap::new();

        // Add default templates
        templates.insert(
            "rust_function".to_string(),
            CodeTemplate {
                name: "rust_function".to_string(),
                language: "rust".to_string(),
                template: r#"/// {description}
pub fn {name}({params}) -> {return_type} {{
    {body}
}}"#
                .to_string(),
                variables: vec![
                    "description".to_string(),
                    "name".to_string(),
                    "params".to_string(),
                    "return_type".to_string(),
                    "body".to_string(),
                ],
            },
        );

        templates.insert(
            "rust_struct".to_string(),
            CodeTemplate {
                name: "rust_struct".to_string(),
                language: "rust".to_string(),
                template: r#"/// {description}
#[derive(Debug, Clone)]
pub struct {name} {{
    {fields}
}}"#
                .to_string(),
                variables: vec![
                    "description".to_string(),
                    "name".to_string(),
                    "fields".to_string(),
                ],
            },
        );

        templates.insert(
            "python_function".to_string(),
            CodeTemplate {
                name: "python_function".to_string(),
                language: "python".to_string(),
                template: r#"def {name}({params}):
    """{description}"""
    {body}"#
                    .to_string(),
                variables: vec![
                    "description".to_string(),
                    "name".to_string(),
                    "params".to_string(),
                    "body".to_string(),
                ],
            },
        );

        templates.insert(
            "javascript_function".to_string(),
            CodeTemplate {
                name: "javascript_function".to_string(),
                language: "javascript".to_string(),
                template: r#"/**
 * {description}
 */
function {name}({params}) {{
    {body}
}}"#
                .to_string(),
                variables: vec![
                    "description".to_string(),
                    "name".to_string(),
                    "params".to_string(),
                    "body".to_string(),
                ],
            },
        );

        Ok(Self {
            templates,
            ai_manager: None,
        })
    }

    /// Set the AI manager for enhanced code generation
    pub fn set_ai_manager(&mut self, ai_manager: Arc<AIManager>) {
        self.ai_manager = Some(ai_manager);
    }

    /// Get a reference to the AI manager if available
    pub fn ai_manager(&self) -> Option<&Arc<AIManager>> {
        self.ai_manager.as_ref()
    }

    /// Generate code using AI if available, fallback to templates
    pub async fn generate_with_ai(
        &self,
        prompt: &str,
        language: &str,
        context: &CodebaseContext,
        config: &GenerationConfig,
    ) -> Result<GenerationResult, CodeGenError> {
        if let Some(ai_manager) = &self.ai_manager {
            self.generate_with_ai_backend(prompt, language, context, config, ai_manager)
                .await
        } else {
            // Fallback to template-based generation
            let code = self.generate(prompt, language, context, config).await?;
            Ok(GenerationResult {
                generated_code: code,
                language: language.to_string(),
                confidence_score: 0.6, // Lower confidence for template-based generation
                suggestions: vec![
                    "Consider setting up AI integration for better code generation".to_string(),
                ],
                modifications: Vec::new(),
                metadata: crate::codegen::GenerationMetadata {
                    generation_time_ms: 100,
                    tokens_used: 0,
                    context_files_analyzed: 1,
                    template_used: Some("template_fallback".to_string()),
                },
            })
        }
    }

    /// Internal method for AI-powered code generation
    async fn generate_with_ai_backend(
        &self,
        prompt: &str,
        language: &str,
        context: &CodebaseContext,
        config: &GenerationConfig,
        ai_manager: &AIManager,
    ) -> Result<GenerationResult, CodeGenError> {
        let start_time = std::time::Instant::now();

        // Build context-aware prompt
        let enhanced_prompt = self.build_context_prompt(prompt, language, context);

        // Prepare AI request
        let parameters = ModelParameters {
            temperature: Some(config.temperature.unwrap_or(0.3)), // Lower temperature for code
            max_tokens: Some(config.max_tokens.unwrap_or(2000)),
            top_p: Some(0.9),
            stop: Some(vec!["```".to_string(), "\n\n---".to_string()]),
            ..Default::default()
        };

        let messages = vec![
            ChatMessage::system(
                "You are an expert software engineer. Generate clean, well-documented code based on the user's request. \
                Follow the existing codebase patterns and conventions. Provide only the code without additional explanations unless requested."
            ),
            ChatMessage::user(enhanced_prompt),
        ];

        let request = ChatRequest {
            model: String::new(), // Use default model
            messages,
            parameters: Some(parameters),
            stream: false,
        };

        // Make AI request
        let response = ai_manager
            .chat_completion_default(request)
            .await
            .map_err(|e| CodeGenError::GenerationError(format!("AI generation failed: {}", e)))?;

        let generation_time_ms = start_time.elapsed().as_millis() as u64;

        // Extract and clean the generated code
        let generated_code = self.clean_generated_code(&response.message.content, language);

        // Calculate confidence based on response quality
        let confidence = self.calculate_confidence(&generated_code, language);

        // Generate suggestions
        let suggestions = self.generate_suggestions(&generated_code, language, context);

        Ok(GenerationResult {
            generated_code,
            language: language.to_string(),
            confidence_score: confidence,
            suggestions,
            modifications: Vec::new(),
            metadata: crate::codegen::GenerationMetadata {
                generation_time_ms,
                tokens_used: response.usage.map(|u| u.total_tokens).unwrap_or(0),
                context_files_analyzed: context.files.len(),
                template_used: Some("ai_generation".to_string()),
            },
        })
    }

    /// Build a context-aware prompt for AI generation
    fn build_context_prompt(
        &self,
        prompt: &str,
        language: &str,
        context: &CodebaseContext,
    ) -> String {
        let mut enhanced_prompt = String::new();

        // Add language context
        enhanced_prompt.push_str(&format!("Programming Language: {}\n\n", language));

        // Add relevant context from codebase
        if !context.files.is_empty() {
            enhanced_prompt.push_str("Relevant codebase context:\n");
            for file_context in context.files.iter().take(3) {
                // Limit to first 3 files
                enhanced_prompt.push_str(&format!("File: {}\n", file_context.path.display()));
                if !file_context.imports.is_empty() {
                    enhanced_prompt.push_str("Imports: ");
                    enhanced_prompt.push_str(&file_context.imports.join(", "));
                    enhanced_prompt.push_str("\n");
                }
                if !file_context.symbols.is_empty() {
                    enhanced_prompt.push_str("Symbols: ");
                    enhanced_prompt.push_str(
                        &file_context
                            .symbols
                            .iter()
                            .take(5)
                            .map(|s| s.name.clone())
                            .collect::<Vec<_>>()
                            .join(", "),
                    );
                    enhanced_prompt.push_str("\n");
                }
                enhanced_prompt.push_str("\n");
            }
        }

        // Add the actual prompt
        enhanced_prompt.push_str("Task: ");
        enhanced_prompt.push_str(prompt);
        enhanced_prompt.push_str("\n\nPlease generate the requested code following the existing patterns and conventions.");

        enhanced_prompt
    }

    /// Clean and format the generated code
    fn clean_generated_code(&self, raw_code: &str, language: &str) -> String {
        let mut cleaned = raw_code.trim().to_string();

        // Remove markdown code blocks if present
        if cleaned.starts_with("```") {
            let lines: Vec<&str> = cleaned.lines().collect();
            if lines.len() > 2 {
                // Remove first and last lines if they are markdown code block markers
                cleaned = lines[1..lines.len() - 1].join("\n");
            }
        }

        // Remove language specification line if present
        if let Some(_lang) = language.to_lowercase().chars().next() {
            let lang_marker = format!("```{}", language.to_lowercase());
            if cleaned.starts_with(&lang_marker) {
                cleaned = cleaned
                    .strip_prefix(&lang_marker)
                    .unwrap_or(&cleaned)
                    .trim()
                    .to_string();
            }
        }

        // Basic cleanup
        cleaned = cleaned.trim().to_string();

        cleaned
    }

    /// Calculate confidence score based on generated code quality
    fn calculate_confidence(&self, code: &str, language: &str) -> f64 {
        let mut score: f64 = 0.5; // Base score

        // Check if code is not empty
        if !code.trim().is_empty() {
            score += 0.2;
        }

        // Check for language-specific syntax
        match language.to_lowercase().as_str() {
            "rust" => {
                if code.contains("fn ") || code.contains("struct ") || code.contains("impl ") {
                    score += 0.15;
                }
                if code.contains("pub ") {
                    score += 0.05;
                }
                if code.contains("//") || code.contains("///") {
                    score += 0.1;
                }
            }
            "python" => {
                if code.contains("def ") || code.contains("class ") {
                    score += 0.15;
                }
                if code.contains(&"\"\"\""[..]) {
                    score += 0.1;
                }
            }
            "javascript" | "typescript" => {
                if code.contains("function ") || code.contains(" => ") {
                    score += 0.15;
                }
                if code.contains("/**") {
                    score += 0.1;
                }
            }
            _ => {}
        }

        // Ensure score is within valid range
        score.min(1.0).max(0.0)
    }

    /// Generate suggestions for the code
    fn generate_suggestions(
        &self,
        code: &str,
        language: &str,
        _context: &CodebaseContext,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Basic suggestions based on code analysis
        if code.len() > 1000 {
            suggestions.push("Consider breaking this into smaller functions".to_string());
        }

        if !code.contains("//") && !code.contains("#") && !code.contains("/*") {
            suggestions.push("Add comments to explain complex logic".to_string());
        }

        match language.to_lowercase().as_str() {
            "rust" => {
                if !code.contains("Result<") && code.contains("unwrap()") {
                    suggestions
                        .push("Consider proper error handling instead of unwrap()".to_string());
                }
                if code.contains("clone()") {
                    suggestions
                        .push("Review clone() usage for performance implications".to_string());
                }
            }
            "python" => {
                if !code.contains("try:") && code.contains("Exception") {
                    suggestions.push("Consider adding proper exception handling".to_string());
                }
            }
            _ => {}
        }

        if suggestions.is_empty() {
            suggestions.push("Code looks good! Consider adding tests.".to_string());
        }

        suggestions
    }

    /// Generate code from a prompt
    pub async fn generate(
        &self,
        prompt: &str,
        language: &str,
        _context: &CodebaseContext,
        config: &GenerationConfig,
    ) -> Result<String, CodeGenError> {
        // For this implementation, we'll use simple template-based generation
        // In a real implementation, this would interface with an AI service

        let template_key = format!("{}_{}", language, self.infer_template_type(prompt));

        if let Some(template) = self.templates.get(&template_key) {
            self.apply_template(template, prompt, config).await
        } else {
            // Fallback to basic code generation
            self.generate_basic_code(prompt, language, config).await
        }
    }

    /// Complete partial code
    pub async fn complete_code(
        &self,
        partial_code: &str,
        cursor_position: usize,
        _context: &CodebaseContext,
    ) -> Result<Vec<String>, CodeGenError> {
        // Simple completion suggestions
        let mut suggestions = Vec::new();

        let code_before_cursor = &partial_code[..cursor_position.min(partial_code.len())];

        if code_before_cursor.ends_with("fn ") {
            suggestions.push("new_function() -> Result<(), Error> {\n    todo!()\n}".to_string());
        } else if code_before_cursor.ends_with("struct ") {
            suggestions.push("NewStruct {\n    field: String,\n}".to_string());
        } else if code_before_cursor.ends_with("impl ") {
            suggestions
                .push("SomeType {\n    fn new() -> Self {\n        Self {}\n    }\n}".to_string());
        } else if code_before_cursor.ends_with("let ") {
            suggestions.push("variable = value;".to_string());
        }

        if suggestions.is_empty() {
            suggestions.push("// Continue implementation here".to_string());
        }

        Ok(suggestions)
    }

    /// Refactor existing code
    pub async fn refactor(
        &self,
        code: &str,
        refactor_type: RefactorType,
        _context: &CodebaseContext,
    ) -> Result<GenerationResult, CodeGenError> {
        match refactor_type {
            RefactorType::ExtractFunction {
                start_line,
                end_line,
                function_name,
            } => {
                let lines: Vec<&str> = code.lines().collect();
                if start_line > 0 && end_line <= lines.len() && start_line <= end_line {
                    let extracted_lines = &lines[start_line - 1..end_line];
                    let extracted_code = extracted_lines.join("\n");

                    let new_function = format!("fn {}() {{\n{}\n}}", function_name, extracted_code);

                    Ok(GenerationResult {
                        generated_code: new_function,
                        language: "rust".to_string(),
                        confidence_score: 0.8,
                        suggestions: vec!["Consider adding parameters if the extracted code uses external variables".to_string()],
                        modifications: Vec::new(),
                        metadata: crate::codegen::GenerationMetadata {
                            generation_time_ms: 50,
                            tokens_used: 100,
                            context_files_analyzed: 1,
                            template_used: Some("extract_function".to_string()),
                        },
                    })
                } else {
                    Err(CodeGenError::InvalidConfig(
                        "Invalid line range for extraction".to_string(),
                    ))
                }
            }
            RefactorType::RenameSymbol { old_name, new_name } => {
                let refactored_code = code.replace(&old_name, &new_name);
                Ok(GenerationResult {
                    generated_code: refactored_code,
                    language: "rust".to_string(),
                    confidence_score: 0.9,
                    suggestions: vec![
                        "Verify that all references have been updated correctly".to_string()
                    ],
                    modifications: Vec::new(),
                    metadata: crate::codegen::GenerationMetadata {
                        generation_time_ms: 25,
                        tokens_used: 50,
                        context_files_analyzed: 1,
                        template_used: Some("rename_symbol".to_string()),
                    },
                })
            }
            RefactorType::SimplifyLogic => {
                // Basic logic simplification
                let simplified = code
                    .replace("if true {", "// Simplified: ")
                    .replace("if false {", "// Removed unreachable code: ");

                Ok(GenerationResult {
                    generated_code: simplified,
                    language: "rust".to_string(),
                    confidence_score: 0.6,
                    suggestions: vec![
                        "Review the simplified logic to ensure correctness".to_string()
                    ],
                    modifications: Vec::new(),
                    metadata: crate::codegen::GenerationMetadata {
                        generation_time_ms: 75,
                        tokens_used: 150,
                        context_files_analyzed: 1,
                        template_used: Some("simplify_logic".to_string()),
                    },
                })
            }
            RefactorType::OptimizePerformance => {
                // Basic performance optimizations
                let optimized = code
                    .replace("Vec::new()", "Vec::with_capacity(10)")
                    .replace("String::new()", "String::with_capacity(50)");

                Ok(GenerationResult {
                    generated_code: optimized,
                    language: "rust".to_string(),
                    confidence_score: 0.7,
                    suggestions: vec![
                        "Consider profiling to measure actual performance impact".to_string()
                    ],
                    modifications: Vec::new(),
                    metadata: crate::codegen::GenerationMetadata {
                        generation_time_ms: 100,
                        tokens_used: 200,
                        context_files_analyzed: 1,
                        template_used: Some("optimize_performance".to_string()),
                    },
                })
            }
            RefactorType::ImproveReadability => {
                // Basic readability improvements
                let readable = code
                    .lines()
                    .map(|line| {
                        if line.trim().is_empty() {
                            line.to_string()
                        } else {
                            format!("    {}", line.trim()) // Add consistent indentation
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                Ok(GenerationResult {
                    generated_code: readable,
                    language: "rust".to_string(),
                    confidence_score: 0.8,
                    suggestions: vec!["Consider adding comments for complex logic".to_string()],
                    modifications: Vec::new(),
                    metadata: crate::codegen::GenerationMetadata {
                        generation_time_ms: 60,
                        tokens_used: 120,
                        context_files_analyzed: 1,
                        template_used: Some("improve_readability".to_string()),
                    },
                })
            }
        }
    }

    /// Infer template type from prompt
    fn infer_template_type(&self, prompt: &str) -> &str {
        let prompt_lower = prompt.to_lowercase();

        if prompt_lower.contains("function") || prompt_lower.contains("method") {
            "function"
        } else if prompt_lower.contains("struct") || prompt_lower.contains("class") {
            "struct"
        } else {
            "function" // Default to function
        }
    }

    /// Apply template to generate code
    async fn apply_template(
        &self,
        template: &CodeTemplate,
        prompt: &str,
        _config: &GenerationConfig,
    ) -> Result<String, CodeGenError> {
        let mut code = template.template.clone();

        // Extract information from prompt for template variables
        let name = self
            .extract_name(prompt)
            .unwrap_or_else(|| "generated_item".to_string());
        let description = prompt.to_string();

        // Replace template variables
        code = code.replace("{name}", &name);
        code = code.replace("{description}", &description);
        code = code.replace("{params}", "");
        code = code.replace("{return_type}", "()");
        code = code.replace("{body}", "    todo!(\"Implement this\")");
        code = code.replace("{fields}", "    // Add fields here");

        Ok(code)
    }

    /// Generate basic code without templates
    async fn generate_basic_code(
        &self,
        prompt: &str,
        language: &str,
        _config: &GenerationConfig,
    ) -> Result<String, CodeGenError> {
        match language {
            "rust" => {
                Ok(format!(
                    "// Generated code for: {}\n// TODO: Implement the following:\n// {}\n\npub fn generated_function() {{\n    todo!(\"Implement based on prompt\")\n}}",
                    prompt, prompt
                ))
            }
            "python" => {
                Ok(format!(
                    "# Generated code for: {}\n# TODO: Implement the following:\n# {}\n\ndef generated_function():\n    \"\"\"Generated function based on prompt.\"\"\"\n    pass",
                    prompt, prompt
                ))
            }
            "javascript" => {
                Ok(format!(
                    "// Generated code for: {}\n// TODO: Implement the following:\n// {}\n\nfunction generatedFunction() {{\n    // Implement based on prompt\n    return null;\n}}",
                    prompt, prompt
                ))
            }
            _ => {
                Ok(format!(
                    "// Generated code for: {}\n// Language: {}\n// TODO: Implement the following:\n// {}",
                    prompt, language, prompt
                ))
            }
        }
    }

    /// Extract name from prompt
    fn extract_name(&self, prompt: &str) -> Option<String> {
        let words: Vec<&str> = prompt.split_whitespace().collect();

        for i in 0..words.len() {
            if (words[i] == "named" || words[i] == "called") && i + 1 < words.len() {
                return Some(words[i + 1].to_string());
            }
        }

        // Look for capitalized words that might be names
        for word in words {
            if word.chars().next().unwrap_or('a').is_uppercase() && word.len() > 2 {
                return Some(word.to_string());
            }
        }

        None
    }
}
