//! Language Analyzers Module
//!
//! Provides concrete implementations of language analyzers for
//! different programming languages.

mod python;
mod javascript;
mod rust;  // Assuming we already have a Rust analyzer

use super::LanguageAnalyzer;
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

pub use python::PythonAnalyzer;
pub use javascript::JavaScriptAnalyzer;
pub use rust::RustAnalyzer;

/// Creates a language analyzer for a specific language
pub fn create_analyzer_for_language(language: &super::Language) -> Arc<dyn LanguageAnalyzer> {
    match language {
        super::Language::Rust => Arc::new(RustAnalyzer::new()),
        super::Language::Python => Arc::new(PythonAnalyzer::new()),
        super::Language::JavaScript | super::Language::TypeScript => Arc::new(JavaScriptAnalyzer::new()),
        _ => Arc::new(RustAnalyzer::new()), // Default fallback
    }
}

/// Creates a language analyzer based on detected language
pub async fn create_analyzer_for_path(path: &Path) -> Result<Arc<dyn LanguageAnalyzer>> {
    // Detect language from path
    let language = detect_language(path).await?;
    
    // Create appropriate analyzer
    let analyzer: Arc<dyn LanguageAnalyzer> = match language.as_str() {
        "rust" => Arc::new(RustAnalyzer::new()),
        "python" => Arc::new(PythonAnalyzer::new()),
        "javascript" | "typescript" => Arc::new(JavaScriptAnalyzer::new()),
        _ => Arc::new(RustAnalyzer::new()), // Default to Rust for now
    };
    
    Ok(analyzer)
}

/// Detect language from project path
async fn detect_language(path: &Path) -> Result<String> {
    // Check for language-specific files
    if path.join("Cargo.toml").exists() {
        return Ok("rust".to_string());
    }
    
    if path.join("requirements.txt").exists() || 
       path.join("setup.py").exists() || 
       path.join("pyproject.toml").exists() {
        return Ok("python".to_string());
    }
    
    if path.join("package.json").exists() {
        // Determine if JavaScript or TypeScript
        if path.join("tsconfig.json").exists() {
            return Ok("typescript".to_string());
        }
        
        // Check for .ts files
        let walker = walkdir::WalkDir::new(path)
            .max_depth(3)  // Don't go too deep
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file());
            
        for entry in walker {
            if let Some(ext) = entry.path().extension() {
                if ext == "ts" || ext == "tsx" {
                    return Ok("typescript".to_string());
                }
            }
        }
        
        return Ok("javascript".to_string());
    }
    
    // Fallback: Check file extensions
    let walker = walkdir::WalkDir::new(path)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file());
        
    let mut rust_files = 0;
    let mut python_files = 0;
    let mut js_files = 0;
    let mut ts_files = 0;
    
    for entry in walker {
        if let Some(ext) = entry.path().extension() {
            match ext.to_str() {
                Some("rs") => rust_files += 1,
                Some("py") => python_files += 1,
                Some("js") | Some("jsx") | Some("mjs") => js_files += 1,
                Some("ts") | Some("tsx") => ts_files += 1,
                _ => {}
            }
        }
    }
    
    // Return the language with most files
    if rust_files > python_files && rust_files > js_files && rust_files > ts_files {
        return Ok("rust".to_string());
    } else if python_files > rust_files && python_files > js_files && python_files > ts_files {
        return Ok("python".to_string());
    } else if ts_files > 0 && ts_files >= js_files {
        return Ok("typescript".to_string());
    } else if js_files > 0 {
        return Ok("javascript".to_string());
    }
    
    // Default to Rust as this is primarily a Rust tool
    Ok("rust".to_string())
}

/// Get all available analyzers
pub fn get_all_analyzers() -> Vec<Arc<dyn LanguageAnalyzer>> {
    vec![
        Arc::new(RustAnalyzer::new()),
        Arc::new(PythonAnalyzer::new()),
        Arc::new(JavaScriptAnalyzer::new()),
    ]
}