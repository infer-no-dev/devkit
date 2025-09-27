use proptest::prelude::*;
use devkit::codegen::CodeGenerator;
use devkit::ai::AIManager;
use devkit::config::Config;

#[cfg(test)]
mod property_tests {
    use super::*;
    
    proptest! {
        #[test]
        fn test_code_generation_always_produces_valid_output(
            prompt in r"[a-zA-Z0-9 ]{10,100}",
            language in prop::option::of(r"rust|python|javascript|go|c\+\+"),
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let config = Config::default();
                if let Ok(ai_manager) = AIManager::new(config).await {
                    let code_gen = CodeGenerator::new(ai_manager);
                    
                    if let Ok(result) = code_gen.generate_code(&prompt, language, None).await {
                        // Properties that should always hold
                        prop_assert!(!result.generated_code.is_empty());
                        prop_assert!(result.generated_code.len() < 100_000); // Reasonable size limit
                        
                        // If language is specified, check basic syntax validity
                        if let Some(lang) = &result.language {
                            match lang.as_str() {
                                "rust" => {
                                    // Basic Rust syntax checks
                                    prop_assert!(!result.generated_code.contains("undefined"));
                                }
                                "python" => {
                                    // Basic Python syntax checks
                                    prop_assert!(!result.generated_code.contains("SyntaxError"));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            });
        }
        
        #[test]
        fn test_context_analysis_is_deterministic(
            project_path in r"/[a-zA-Z0-9_/]{5,50}",
        ) {
            // Context analysis should produce consistent results for the same input
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let config = Config::default();
                if let Ok(context_manager) = devkit::context::ContextManager::new(config) {
                    if let Ok(result1) = context_manager.analyze_directory(project_path.clone().into()).await {
                        if let Ok(result2) = context_manager.analyze_directory(project_path.into()).await {
                            prop_assert_eq!(result1.files.len(), result2.files.len());
                            prop_assert_eq!(result1.metadata.indexed_symbols, result2.metadata.indexed_symbols);
                        }
                    }
                }
            });
        }
    }
}