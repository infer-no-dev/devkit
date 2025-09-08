//! Language detection for code generation.

use std::collections::HashMap;
use crate::context::CodebaseContext;
use crate::codegen::CodeGenError;

/// Language detector that identifies programming languages from various sources
#[derive(Debug, Clone)]
pub struct LanguageDetector {
    extension_mappings: HashMap<String, String>,
    content_patterns: HashMap<String, Vec<String>>,
}

impl LanguageDetector {
    /// Create a new language detector
    pub fn new() -> Self {
        let mut extension_mappings = HashMap::new();
        let mut content_patterns = HashMap::new();
        
        // File extension mappings
        extension_mappings.insert("rs".to_string(), "rust".to_string());
        extension_mappings.insert("py".to_string(), "python".to_string());
        extension_mappings.insert("js".to_string(), "javascript".to_string());
        extension_mappings.insert("ts".to_string(), "typescript".to_string());
        extension_mappings.insert("java".to_string(), "java".to_string());
        extension_mappings.insert("cpp".to_string(), "cpp".to_string());
        extension_mappings.insert("cc".to_string(), "cpp".to_string());
        extension_mappings.insert("cxx".to_string(), "cpp".to_string());
        extension_mappings.insert("c".to_string(), "c".to_string());
        extension_mappings.insert("go".to_string(), "go".to_string());
        extension_mappings.insert("rb".to_string(), "ruby".to_string());
        extension_mappings.insert("php".to_string(), "php".to_string());
        extension_mappings.insert("sh".to_string(), "bash".to_string());
        extension_mappings.insert("yml".to_string(), "yaml".to_string());
        extension_mappings.insert("yaml".to_string(), "yaml".to_string());
        extension_mappings.insert("json".to_string(), "json".to_string());
        extension_mappings.insert("xml".to_string(), "xml".to_string());
        extension_mappings.insert("html".to_string(), "html".to_string());
        extension_mappings.insert("css".to_string(), "css".to_string());
        extension_mappings.insert("sql".to_string(), "sql".to_string());
        
        // Content pattern mappings for language detection
        content_patterns.insert("rust".to_string(), vec![
            "fn ".to_string(),
            "struct ".to_string(),
            "impl ".to_string(),
            "use ".to_string(),
            "pub ".to_string(),
            "let ".to_string(),
            "match ".to_string(),
            "Result<".to_string(),
            "Option<".to_string(),
        ]);
        
        content_patterns.insert("python".to_string(), vec![
            "def ".to_string(),
            "class ".to_string(),
            "import ".to_string(),
            "from ".to_string(),
            "if __name__".to_string(),
            "self.".to_string(),
            "elif ".to_string(),
        ]);
        
        content_patterns.insert("javascript".to_string(), vec![
            "function ".to_string(),
            "var ".to_string(),
            "let ".to_string(),
            "const ".to_string(),
            "=> ".to_string(),
            "console.log".to_string(),
            "require(".to_string(),
            "module.exports".to_string(),
        ]);
        
        content_patterns.insert("typescript".to_string(), vec![
            "interface ".to_string(),
            "type ".to_string(),
            ": string".to_string(),
            ": number".to_string(),
            ": boolean".to_string(),
            "extends ".to_string(),
            "implements ".to_string(),
        ]);
        
        content_patterns.insert("java".to_string(), vec![
            "public class ".to_string(),
            "private ".to_string(),
            "protected ".to_string(),
            "public static void main".to_string(),
            "import ".to_string(),
            "package ".to_string(),
            "extends ".to_string(),
            "implements ".to_string(),
        ]);
        
        content_patterns.insert("cpp".to_string(), vec![
            "#include ".to_string(),
            "using namespace ".to_string(),
            "std::".to_string(),
            "class ".to_string(),
            "template<".to_string(),
            "cout <<".to_string(),
            "cin >>".to_string(),
        ]);
        
        content_patterns.insert("c".to_string(), vec![
            "#include <".to_string(),
            "int main(".to_string(),
            "printf(".to_string(),
            "scanf(".to_string(),
            "malloc(".to_string(),
            "free(".to_string(),
        ]);
        
        content_patterns.insert("go".to_string(), vec![
            "package ".to_string(),
            "import ".to_string(),
            "func ".to_string(),
            "type ".to_string(),
            "var ".to_string(),
            "fmt.".to_string(),
            "go ".to_string(),
            "defer ".to_string(),
        ]);
        
        Self {
            extension_mappings,
            content_patterns,
        }
    }
    
    /// Detect language from file extension
    pub fn detect_from_extension(&self, filename: &str) -> Option<String> {
        if let Some(extension) = std::path::Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str()) {
            self.extension_mappings.get(extension).cloned()
        } else {
            None
        }
    }
    
    /// Detect language from file content
    pub fn detect_from_content(&self, content: &str) -> Result<String, CodeGenError> {
        let content_lower = content.to_lowercase();
        let mut language_scores: HashMap<String, usize> = HashMap::new();
        
        // Score each language based on pattern matches
        for (language, patterns) in &self.content_patterns {
            let mut score = 0;
            for pattern in patterns {
                if content_lower.contains(pattern) {
                    score += 1;
                }
            }
            if score > 0 {
                language_scores.insert(language.clone(), score);
            }
        }
        
        // Return the language with the highest score
        if let Some((best_language, _)) = language_scores.iter()
            .max_by_key(|(_, score)| *score) {
            Ok(best_language.clone())
        } else {
            Ok("unknown".to_string())
        }
    }
    
    /// Detect language from codebase context
    pub fn detect_from_context(&self, context: &CodebaseContext) -> Result<String, CodeGenError> {
        // Find the most common language in the codebase
        let mut language_counts: HashMap<String, usize> = HashMap::new();
        
        for file in &context.files {
            if let Some(language) = self.detect_from_extension(&file.path.to_string_lossy()) {
                *language_counts.entry(language).or_insert(0) += 1;
            }
        }
        
        if let Some((most_common_language, _)) = language_counts.iter()
            .max_by_key(|(_, count)| *count) {
            Ok(most_common_language.clone())
        } else {
            // Fallback to analyzing metadata
            if let Some((language, _)) = context.metadata.languages.iter()
                .max_by_key(|(_, count)| *count) {
                Ok(language.clone())
            } else {
                Ok("unknown".to_string())
            }
        }
    }
    
    /// Detect language from multiple sources with confidence scoring
    pub fn detect_comprehensive(
        &self, 
        filename: Option<&str>,
        content: Option<&str>,
        context: Option<&CodebaseContext>,
    ) -> Result<(String, f64), CodeGenError> {
        let mut candidates: HashMap<String, f64> = HashMap::new();
        
        // Check file extension
        if let Some(filename) = filename {
            if let Some(lang) = self.detect_from_extension(filename) {
                candidates.insert(lang, 0.8); // High confidence for file extensions
            }
        }
        
        // Check content patterns
        if let Some(content) = content {
            let detected_lang = self.detect_from_content(content)?;
            if detected_lang != "unknown" {
                *candidates.entry(detected_lang).or_insert(0.0) += 0.6;
            }
        }
        
        // Check context
        if let Some(context) = context {
            let context_lang = self.detect_from_context(context)?;
            if context_lang != "unknown" {
                *candidates.entry(context_lang).or_insert(0.0) += 0.4;
            }
        }
        
        // Return the language with highest combined confidence
        if let Some((best_language, confidence)) = candidates.iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)) {
            Ok((best_language.clone(), *confidence))
        } else {
            Ok(("unknown".to_string(), 0.0))
        }
    }
    
    /// Get all supported languages
    pub fn supported_languages(&self) -> Vec<String> {
        let mut languages: Vec<String> = self.extension_mappings.values().cloned().collect();
        languages.sort();
        languages.dedup();
        languages
    }
    
    /// Check if a language is supported
    pub fn is_supported(&self, language: &str) -> bool {
        self.content_patterns.contains_key(language) || 
        self.extension_mappings.values().any(|lang| lang == language)
    }
}
