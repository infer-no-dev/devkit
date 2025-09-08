//! Code analysis capabilities for understanding prompts and existing code.

use std::collections::HashMap;
use crate::context::CodebaseContext;
use crate::codegen::CodeModification;

/// Intent detected from a natural language prompt
#[derive(Debug, Clone)]
pub struct PromptIntent {
    pub intent_type: IntentType,
    pub confidence: f64,
    pub parameters: HashMap<String, String>,
    pub suggested_language: Option<String>,
}

/// Types of coding intents that can be detected
#[derive(Debug, Clone, PartialEq)]
pub enum IntentType {
    GenerateFunction,
    GenerateStruct,
    GenerateClass,
    RefactorCode,
    FixBug,
    AddTests,
    OptimizePerformance,
    Documentation,
    Unknown,
}

/// Analyzer for understanding code and prompts
#[derive(Debug, Clone)]
pub struct CodeAnalyzer {
    intent_patterns: HashMap<String, IntentType>,
}

impl CodeAnalyzer {
    /// Create a new code analyzer
    pub fn new() -> Result<Self, crate::codegen::CodeGenError> {
        let mut intent_patterns = HashMap::new();
        
        // Function generation patterns
        intent_patterns.insert("create function".to_string(), IntentType::GenerateFunction);
        intent_patterns.insert("write function".to_string(), IntentType::GenerateFunction);
        intent_patterns.insert("implement function".to_string(), IntentType::GenerateFunction);
        intent_patterns.insert("generate function".to_string(), IntentType::GenerateFunction);
        
        // Struct/Class generation patterns
        intent_patterns.insert("create struct".to_string(), IntentType::GenerateStruct);
        intent_patterns.insert("define struct".to_string(), IntentType::GenerateStruct);
        intent_patterns.insert("create class".to_string(), IntentType::GenerateClass);
        intent_patterns.insert("implement class".to_string(), IntentType::GenerateClass);
        
        // Refactoring patterns
        intent_patterns.insert("refactor".to_string(), IntentType::RefactorCode);
        intent_patterns.insert("improve".to_string(), IntentType::RefactorCode);
        intent_patterns.insert("clean up".to_string(), IntentType::RefactorCode);
        
        // Bug fixing patterns
        intent_patterns.insert("fix".to_string(), IntentType::FixBug);
        intent_patterns.insert("debug".to_string(), IntentType::FixBug);
        intent_patterns.insert("resolve".to_string(), IntentType::FixBug);
        
        // Testing patterns
        intent_patterns.insert("test".to_string(), IntentType::AddTests);
        intent_patterns.insert("unit test".to_string(), IntentType::AddTests);
        intent_patterns.insert("write tests".to_string(), IntentType::AddTests);
        
        // Performance patterns
        intent_patterns.insert("optimize".to_string(), IntentType::OptimizePerformance);
        intent_patterns.insert("performance".to_string(), IntentType::OptimizePerformance);
        intent_patterns.insert("faster".to_string(), IntentType::OptimizePerformance);
        
        // Documentation patterns
        intent_patterns.insert("document".to_string(), IntentType::Documentation);
        intent_patterns.insert("comment".to_string(), IntentType::Documentation);
        intent_patterns.insert("explain".to_string(), IntentType::Documentation);
        
        Ok(Self {
            intent_patterns,
        })
    }
    
    /// Analyze a natural language prompt to determine intent
    pub fn analyze_prompt(&self, prompt: &str) -> Result<PromptIntent, crate::codegen::CodeGenError> {
        let prompt_lower = prompt.to_lowercase();
        let mut best_match = (IntentType::Unknown, 0.0);
        let mut parameters = HashMap::new();
        
        // Check for intent patterns
        for (pattern, intent_type) in &self.intent_patterns {
            if prompt_lower.contains(pattern) {
                let confidence = self.calculate_confidence(pattern, &prompt_lower);
                if confidence > best_match.1 {
                    best_match = (intent_type.clone(), confidence);
                }
            }
        }
        
        // Extract parameters based on intent
        match best_match.0 {
            IntentType::GenerateFunction => {
                if let Some(name) = self.extract_function_name(&prompt_lower) {
                    parameters.insert("function_name".to_string(), name);
                }
            }
            IntentType::GenerateStruct | IntentType::GenerateClass => {
                if let Some(name) = self.extract_type_name(&prompt_lower) {
                    parameters.insert("type_name".to_string(), name);
                }
            }
            _ => {}
        }
        
        // Detect language hints
        let suggested_language = self.detect_language_hint(&prompt_lower);
        
        Ok(PromptIntent {
            intent_type: best_match.0,
            confidence: best_match.1.max(0.3), // Minimum confidence
            parameters,
            suggested_language,
        })
    }
    
    /// Analyze generated code for improvements
    pub fn analyze_generated_code(
        &self,
        code: &str,
        language: &str,
    ) -> Result<Vec<String>, crate::codegen::CodeGenError> {
        let mut suggestions = Vec::new();
        
        match language {
            "rust" => {
                if !code.contains("//") && !code.contains("///") {
                    suggestions.push("Consider adding documentation comments".to_string());
                }
                if code.contains("unwrap()") {
                    suggestions.push("Consider proper error handling instead of unwrap()".to_string());
                }
                if !code.contains("#[derive(") && code.contains("struct") {
                    suggestions.push("Consider adding common derives like Debug, Clone".to_string());
                }
            }
            "python" => {
                if !code.contains("\"\"\"") && !code.contains("def ") {
                    suggestions.push("Consider adding docstrings".to_string());
                }
                if code.contains("except:") {
                    suggestions.push("Use specific exception types instead of bare except".to_string());
                }
            }
            "javascript" | "typescript" => {
                if !code.contains("/**") && code.contains("function") {
                    suggestions.push("Consider adding JSDoc comments".to_string());
                }
                if code.contains("var ") {
                    suggestions.push("Use 'let' or 'const' instead of 'var'".to_string());
                }
            }
            _ => {}
        }
        
        Ok(suggestions)
    }
    
    /// Suggest improvements for existing code
    pub async fn suggest_improvements(
        &self,
        code: &str,
        file_path: &str,
        _context: &CodebaseContext,
    ) -> Result<Vec<CodeModification>, crate::codegen::CodeGenError> {
        let mut modifications = Vec::new();
        
        // Detect language from file extension
        let language = self.detect_language_from_path(file_path);
        
        // Analyze code patterns and suggest improvements
        let lines: Vec<&str> = code.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            match language.as_str() {
                "rust" => {
                    if line.contains("println!(") && !line.contains("debug") {
                        modifications.push(CodeModification {
                            file_path: file_path.to_string(),
                            line_start: line_num + 1,
                            line_end: line_num + 1,
                            original_code: line.to_string(),
                            suggested_code: line.replace("println!", "debug!"),
                            reason: "Use debug logging instead of println for production code".to_string(),
                            confidence: 0.8,
                        });
                    }
                }
                "python" => {
                    if line.contains("print(") && !line.trim().starts_with("#") {
                        modifications.push(CodeModification {
                            file_path: file_path.to_string(),
                            line_start: line_num + 1,
                            line_end: line_num + 1,
                            original_code: line.to_string(),
                            suggested_code: line.replace("print(", "logging.info("),
                            reason: "Use logging instead of print statements".to_string(),
                            confidence: 0.9,
                        });
                    }
                }
                _ => {}
            }
        }
        
        Ok(modifications)
    }
    
    /// Calculate confidence score for pattern matching
    fn calculate_confidence(&self, pattern: &str, prompt: &str) -> f64 {
        let pattern_words: Vec<&str> = pattern.split_whitespace().collect();
        let prompt_words: Vec<&str> = prompt.split_whitespace().collect();
        
        let matches = pattern_words.iter()
            .filter(|word| prompt_words.contains(word))
            .count();
        
        matches as f64 / pattern_words.len() as f64
    }
    
    /// Extract function name from prompt
    fn extract_function_name(&self, prompt: &str) -> Option<String> {
        // Look for patterns like "create function named X" or "function called X"
        let words: Vec<&str> = prompt.split_whitespace().collect();
        
        for i in 0..words.len() {
            if (words[i] == "named" || words[i] == "called") && i + 1 < words.len() {
                return Some(words[i + 1].to_string());
            }
        }
        
        None
    }
    
    /// Extract type name from prompt
    fn extract_type_name(&self, prompt: &str) -> Option<String> {
        // Look for patterns like "create struct named X" or "class called X"
        let words: Vec<&str> = prompt.split_whitespace().collect();
        
        for i in 0..words.len() {
            if (words[i] == "named" || words[i] == "called") && i + 1 < words.len() {
                return Some(words[i + 1].to_string());
            }
        }
        
        None
    }
    
    /// Detect language hint from prompt
    fn detect_language_hint(&self, prompt: &str) -> Option<String> {
        let languages = vec![
            "rust", "python", "javascript", "typescript", "java", "cpp", "c++", "go", "ruby"
        ];
        
        for lang in languages {
            if prompt.contains(lang) {
                return Some(lang.to_string());
            }
        }
        
        None
    }
    
    /// Detect language from file path
    fn detect_language_from_path(&self, file_path: &str) -> String {
        if let Some(extension) = std::path::Path::new(file_path).extension() {
            match extension.to_str().unwrap_or("") {
                "rs" => "rust".to_string(),
                "py" => "python".to_string(),
                "js" => "javascript".to_string(),
                "ts" => "typescript".to_string(),
                "java" => "java".to_string(),
                "cpp" | "cc" | "cxx" => "cpp".to_string(),
                "c" => "c".to_string(),
                "go" => "go".to_string(),
                "rb" => "ruby".to_string(),
                _ => "unknown".to_string(),
            }
        } else {
            "unknown".to_string()
        }
    }
}
