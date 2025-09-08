use std::collections::HashMap;
use std::path::PathBuf;

use agentic_dev_env::codegen::*;
use agentic_dev_env::context::CodebaseContext;
use agentic_dev_env::testing::{
    TestEnvironment, 
    mocks::{MockCodeGenerator, MockLanguageDetector, MockTemplateManager}, 
    fixtures::{CodegenFixtures, ContextFixtures}
};

/// Tests for code generation functionality
mod generator_tests {
    use super::*;

    #[test]
    fn test_code_generator_creation() {
        let generator = MockCodeGenerator::new();
        assert!(!generator.languages.is_empty());
        assert!(generator.languages.contains(&"rust".to_string()));
    }

    #[test]
    fn test_simple_code_generation() {
        let mut generator = MockCodeGenerator::new();
        let context = ContextFixtures::create_codebase_context("/test/project");
        let config = CodegenFixtures::create_generation_config();
        
        let result = generator.generate_code("Create a hello world function", &context, &config);
        
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("Generated code for: Create a hello world function"));
        assert!(code.contains("pub fn mock_function"));
        
        let generated = generator.get_generated_code();
        assert_eq!(generated.len(), 1);
    }

    #[test]
    fn test_code_generation_with_context() {
        let mut generator = MockCodeGenerator::new();
        let mut context = ContextFixtures::create_codebase_context("/test/project");
        
        // Add some context-specific information
        context.metadata.insert("framework".to_string(), "tokio".to_string());
        context.metadata.insert("language_version".to_string(), "rust-2021".to_string());
        
        let config = CodegenFixtures::create_generation_config();
        
        let result = generator.generate_code(
            "Create an async function that handles HTTP requests", 
            &context, 
            &config
        );
        
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("async function") || code.contains("HTTP requests"));
    }

    #[test]
    fn test_code_generation_failure() {
        let mut generator = MockCodeGenerator::new().with_failure(true);
        let context = ContextFixtures::create_codebase_context("/test/project");
        let config = CodegenFixtures::create_generation_config();
        
        let result = generator.generate_code("This will fail", &context, &config);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CodegenError::GenerationFailed(_)));
        
        let generated = generator.get_generated_code();
        assert!(generated.is_empty());
    }

    #[test]
    fn test_code_validation() {
        let generator = MockCodeGenerator::new();
        
        let valid_rust_code = "pub fn test() { println!(\"Hello\"); }";
        let result = generator.validate_code(valid_rust_code, "rust");
        
        assert!(result.is_ok());
        let messages = result.unwrap();
        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("validation passed"));
    }

    #[test]
    fn test_multiple_generations() {
        let mut generator = MockCodeGenerator::new();
        let context = ContextFixtures::create_codebase_context("/test/project");
        let config = CodegenFixtures::create_generation_config();
        
        let prompts = CodegenFixtures::create_sample_prompts();
        
        for prompt in &prompts {
            let result = generator.generate_code(prompt, &context, &config);
            assert!(result.is_ok());
        }
        
        let generated = generator.get_generated_code();
        assert_eq!(generated.len(), prompts.len());
    }
}

/// Tests for language detection
mod language_detection_tests {
    use super::*;

    #[test]
    fn test_rust_detection() {
        let detector = MockLanguageDetector::new();
        
        let rust_code = "fn main() { println!(\"Hello, world!\"); }";
        let result = detector.detect_language(rust_code);
        
        assert_eq!(result, Some("rust".to_string()));
    }

    #[test]
    fn test_python_detection() {
        let detector = MockLanguageDetector::new();
        
        let python_code = "def main():\n    print('Hello, world!')";
        let result = detector.detect_language(python_code);
        
        assert_eq!(result, Some("python".to_string()));
    }

    #[test]
    fn test_javascript_detection() {
        let detector = MockLanguageDetector::new();
        
        let js_code = "function main() { console.log('Hello, world!'); }";
        let result = detector.detect_language(js_code);
        
        assert_eq!(result, Some("javascript".to_string()));
    }

    #[test]
    fn test_unknown_language() {
        let detector = MockLanguageDetector::new();
        
        let unknown_code = "SOME UNKNOWN LANGUAGE CODE";
        let result = detector.detect_language(unknown_code);
        
        assert_eq!(result, None);
    }

    #[test]
    fn test_supported_languages() {
        let detector = MockLanguageDetector::new();
        let supported = detector.supported_languages();
        
        assert!(supported.contains(&"rust".to_string()));
        assert!(supported.contains(&"python".to_string()));
        assert!(supported.contains(&"javascript".to_string()));
    }

    #[test]
    fn test_code_with_multiple_patterns() {
        let detector = MockLanguageDetector::new();
        
        // Code that might contain multiple language patterns
        let mixed_code = "fn main() { /* This is Rust, but mentions def main() in comment */ }";
        let result = detector.detect_language(mixed_code);
        
        // Should detect Rust first (depends on implementation order)
        assert_eq!(result, Some("rust".to_string()));
    }
}

/// Tests for template management
mod template_tests {
    use super::*;

    #[test]
    fn test_template_retrieval() {
        let manager = MockTemplateManager::new();
        
        let rust_template = manager.get_template("rust_function");
        assert!(rust_template.is_some());
        
        let template = rust_template.unwrap();
        assert!(template.contains("{{name}}"));
        assert!(template.contains("{{params}}"));
        assert!(template.contains("{{return_type}}"));
        assert!(template.contains("{{body}}"));
    }

    #[test]
    fn test_nonexistent_template() {
        let manager = MockTemplateManager::new();
        
        let result = manager.get_template("nonexistent_template");
        assert!(result.is_none());
    }

    #[test]
    fn test_list_templates() {
        let manager = MockTemplateManager::new();
        let templates = manager.list_templates();
        
        assert!(templates.contains(&"rust_function".to_string()));
        assert!(templates.contains(&"rust_struct".to_string()));
        assert!(templates.contains(&"python_function".to_string()));
        assert!(templates.contains(&"python_class".to_string()));
    }

    #[test]
    fn test_template_rendering() {
        let manager = MockTemplateManager::new();
        let template = manager.get_template("rust_function").unwrap();
        
        let mut variables = HashMap::new();
        variables.insert("name".to_string(), "calculate_sum".to_string());
        variables.insert("params".to_string(), "a: i32, b: i32".to_string());
        variables.insert("return_type".to_string(), "i32".to_string());
        variables.insert("body".to_string(), "a + b".to_string());
        
        let result = manager.render_template(&template, &variables);
        
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.contains("pub fn calculate_sum(a: i32, b: i32) -> i32"));
        assert!(rendered.contains("a + b"));
    }

    #[test]
    fn test_template_rendering_with_missing_variables() {
        let manager = MockTemplateManager::new();
        let template = manager.get_template("rust_function").unwrap();
        
        let mut variables = HashMap::new();
        variables.insert("name".to_string(), "incomplete_function".to_string());
        // Missing other required variables
        
        let result = manager.render_template(&template, &variables);
        
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.contains("incomplete_function"));
        // Template placeholders should remain for missing variables
        assert!(rendered.contains("{{params}}"));
        assert!(rendered.contains("{{return_type}}"));
        assert!(rendered.contains("{{body}}"));
    }

    #[test]
    fn test_complex_template_rendering() {
        let manager = MockTemplateManager::new();
        let template = manager.get_template("rust_struct").unwrap();
        
        let mut variables = HashMap::new();
        variables.insert("name".to_string(), "Person".to_string());
        variables.insert("description".to_string(), "Represents a person with basic information".to_string());
        variables.insert("fields".to_string(), "name: String,\n    age: u32,".to_string());
        variables.insert("constructor_params".to_string(), "name: String, age: u32".to_string());
        variables.insert("field_assignments".to_string(), "name,\n            age,".to_string());
        
        let result = manager.render_template(&template, &variables);
        
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.contains("pub struct Person"));
        assert!(rendered.contains("name: String"));
        assert!(rendered.contains("age: u32"));
        assert!(rendered.contains("pub fn new(name: String, age: u32)"));
    }
}

/// Tests for code analysis functionality
mod analyzer_tests {
    use super::*;

    #[test]
    fn test_code_quality_analysis() {
        let analyzer = CodeAnalyzer::new();
        let code = r#"
            fn calculate_factorial(n: u32) -> u32 {
                if n == 0 {
                    1
                } else {
                    n * calculate_factorial(n - 1)
                }
            }
        "#;
        
        let result = analyzer.analyze_code_quality(code, "rust");
        
        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(analysis.complexity_score > 0.0);
        assert!(!analysis.suggestions.is_empty());
    }

    #[test]
    fn test_code_complexity_analysis() {
        let analyzer = CodeAnalyzer::new();
        
        // Simple function
        let simple_code = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let simple_result = analyzer.analyze_complexity(simple_code);
        assert!(simple_result.is_ok());
        let simple_complexity = simple_result.unwrap();
        
        // Complex function with multiple branches
        let complex_code = r#"
            fn complex_function(x: i32, y: i32, z: i32) -> i32 {
                if x > 0 {
                    if y > 0 {
                        if z > 0 {
                            x + y + z
                        } else {
                            x + y - z
                        }
                    } else {
                        if z > 0 {
                            x - y + z
                        } else {
                            x - y - z
                        }
                    }
                } else {
                    0
                }
            }
        "#;
        let complex_result = analyzer.analyze_complexity(complex_code);
        assert!(complex_result.is_ok());
        let complex_complexity = complex_result.unwrap();
        
        // Complex function should have higher complexity score
        assert!(complex_complexity > simple_complexity);
    }

    #[test]
    fn test_code_pattern_detection() {
        let analyzer = CodeAnalyzer::new();
        
        let code_with_patterns = r#"
            use std::sync::Arc;
            use std::sync::Mutex;
            
            fn process_data(data: Arc<Mutex<Vec<i32>>>) -> Result<i32, String> {
                let guard = data.lock().unwrap();
                let sum = guard.iter().sum();
                Ok(sum)
            }
        "#;
        
        let result = analyzer.detect_patterns(code_with_patterns, "rust");
        
        assert!(result.is_ok());
        let patterns = result.unwrap();
        
        // Should detect common Rust patterns
        assert!(patterns.iter().any(|p| p.pattern_type == "concurrency"));
        assert!(patterns.iter().any(|p| p.pattern_type == "error_handling"));
    }

    #[test]
    fn test_code_improvement_suggestions() {
        let analyzer = CodeAnalyzer::new();
        
        let problematic_code = r#"
            fn bad_function() {
                let mut x = 5;
                x = x + 1;
                x = x + 1;
                x = x + 1;
                println!("Result: {}", x);
                
                // Unused variable
                let _unused = "this is not used";
                
                // Magic numbers
                if x > 10 {
                    println!("Large number");
                }
            }
        "#;
        
        let result = analyzer.suggest_improvements(problematic_code, "rust");
        
        assert!(result.is_ok());
        let suggestions = result.unwrap();
        
        assert!(!suggestions.is_empty());
        // Should suggest improvements for repetitive code, magic numbers, etc.
        assert!(suggestions.iter().any(|s| s.category == "code_duplication" || s.category == "magic_numbers"));
    }
}

/// Tests for generation configuration
mod config_tests {
    use super::*;

    #[test]
    fn test_generation_config_creation() {
        let config = CodegenFixtures::create_generation_config();
        
        assert_eq!(config.style.indent_size, 4);
        assert_eq!(config.style.indentation, "spaces");
        assert_eq!(config.style.line_length, 100);
        assert_eq!(config.style.naming_convention, "snake_case");
        assert!(config.style.include_comments);
        assert!(config.style.include_type_hints);
    }

    #[test]
    fn test_ai_model_config() {
        let config = CodegenFixtures::create_generation_config();
        
        assert_eq!(config.ai_model.default_model, "test-model");
        assert_eq!(config.ai_model.context_window_size, 2048);
        assert_eq!(config.ai_model.temperature, 0.7);
        assert_eq!(config.ai_model.max_tokens, 1000);
    }

    #[test]
    fn test_style_config_validation() {
        let style = StyleConfig {
            indentation: "tabs".to_string(),
            indent_size: 2,
            line_length: 120,
            naming_convention: "camelCase".to_string(),
            include_comments: false,
            include_type_hints: true,
        };
        
        let result = style.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_style_config() {
        let invalid_style = StyleConfig {
            indentation: "invalid".to_string(), // Invalid indentation type
            indent_size: 0, // Invalid indent size
            line_length: 0, // Invalid line length
            naming_convention: "unknown".to_string(), // Invalid naming convention
            include_comments: true,
            include_type_hints: true,
        };
        
        let result = invalid_style.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_language_preferences() {
        let config = CodegenFixtures::create_generation_config();
        
        assert!(config.language_preferences.contains_key("rust"));
        assert!(config.language_preferences.contains_key("python"));
        
        let rust_prefs = &config.language_preferences["rust"];
        assert!(rust_prefs.get("formatter").is_some());
        assert!(rust_prefs.get("linter").is_some());
    }
}

/// Integration tests for complete code generation workflows
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_end_to_end_code_generation() {
        let env = TestEnvironment::new().expect("Failed to create test environment");
        env.create_sample_project().expect("Failed to create sample project");
        
        let mut generator = MockCodeGenerator::new();
        let detector = MockLanguageDetector::new();
        let template_manager = MockTemplateManager::new();
        
        // Create a generation pipeline
        let pipeline = GenerationPipeline::new(generator, detector, template_manager);
        
        let context = ContextFixtures::create_codebase_context(env.workspace_path().to_str().unwrap());
        let config = CodegenFixtures::create_generation_config();
        
        let request = GenerationRequest {
            prompt: "Create a function to handle user authentication".to_string(),
            target_language: Some("rust".to_string()),
            target_file: Some(PathBuf::from("src/auth.rs")),
            context: HashMap::new(),
        };
        
        let result = pipeline.generate(request, &context, &config).await;
        
        assert!(result.is_ok());
        let generation_result = result.unwrap();
        assert!(!generation_result.generated_code.is_empty());
        assert!(generation_result.target_file.is_some());
        assert_eq!(generation_result.language, "rust");
    }

    #[test]
    fn test_template_based_generation() {
        let template_manager = MockTemplateManager::new();
        
        // Test generating a Rust function using template
        let template = template_manager.get_template("rust_function").unwrap();
        
        let mut variables = HashMap::new();
        variables.insert("name".to_string(), "authenticate_user".to_string());
        variables.insert("params".to_string(), "username: &str, password: &str".to_string());
        variables.insert("return_type".to_string(), "Result<bool, AuthError>".to_string());
        variables.insert("body".to_string(), "// TODO: Implement authentication logic\nOk(true)".to_string());
        
        let result = template_manager.render_template(&template, &variables);
        
        assert!(result.is_ok());
        let code = result.unwrap();
        
        assert!(code.contains("pub fn authenticate_user"));
        assert!(code.contains("username: &str, password: &str"));
        assert!(code.contains("Result<bool, AuthError>"));
        assert!(code.contains("// TODO: Implement authentication logic"));
    }

    #[test]
    fn test_multi_language_generation() {
        let mut generator = MockCodeGenerator::new();
        let context = ContextFixtures::create_codebase_context("/test/project");
        let config = CodegenFixtures::create_generation_config();
        
        let languages = vec!["rust", "python", "javascript"];
        let base_prompt = "Create a simple calculator function";
        
        for language in languages {
            let language_specific_prompt = format!("{} in {}", base_prompt, language);
            let result = generator.generate_code(&language_specific_prompt, &context, &config);
            
            assert!(result.is_ok());
            let code = result.unwrap();
            assert!(code.contains(&language_specific_prompt));
        }
        
        let all_generated = generator.get_generated_code();
        assert_eq!(all_generated.len(), 3);
    }

    #[test]
    fn test_context_aware_generation() {
        let mut generator = MockCodeGenerator::new();
        let mut context = ContextFixtures::create_codebase_context("/test/project");
        
        // Add specific context that should influence generation
        context.metadata.insert("async_runtime".to_string(), "tokio".to_string());
        context.metadata.insert("database".to_string(), "postgresql".to_string());
        context.metadata.insert("web_framework".to_string(), "axum".to_string());
        
        let config = CodegenFixtures::create_generation_config();
        
        let result = generator.generate_code(
            "Create a web endpoint that fetches user data from database",
            &context,
            &config
        );
        
        assert!(result.is_ok());
        let code = result.unwrap();
        
        // Code should reference the context (in a real implementation)
        assert!(code.contains("web endpoint") || code.contains("database"));
    }

    #[test] 
    fn test_generation_with_validation() {
        let mut generator = MockCodeGenerator::new();
        let context = ContextFixtures::create_codebase_context("/test/project");
        let config = CodegenFixtures::create_generation_config();
        
        let result = generator.generate_code("Create a validated function", &context, &config);
        
        assert!(result.is_ok());
        let code = result.unwrap();
        
        // Validate the generated code
        let validation_result = generator.validate_code(&code, "rust");
        
        assert!(validation_result.is_ok());
        let validation_messages = validation_result.unwrap();
        assert!(!validation_messages.is_empty());
        assert!(validation_messages.iter().any(|msg| msg.contains("validation passed")));
    }

    #[test]
    fn test_incremental_code_generation() {
        let mut generator = MockCodeGenerator::new();
        let context = ContextFixtures::create_codebase_context("/test/project");
        let config = CodegenFixtures::create_generation_config();
        
        // Generate base structure
        let base_result = generator.generate_code("Create a basic user struct", &context, &config);
        assert!(base_result.is_ok());
        
        // Generate additional methods
        let methods_result = generator.generate_code("Add validation methods to user struct", &context, &config);
        assert!(methods_result.is_ok());
        
        // Generate tests
        let tests_result = generator.generate_code("Create unit tests for user struct", &context, &config);
        assert!(tests_result.is_ok());
        
        let all_generated = generator.get_generated_code();
        assert_eq!(all_generated.len(), 3);
        
        // Each generation should be different
        let base_code = &all_generated[0];
        let methods_code = &all_generated[1];
        let tests_code = &all_generated[2];
        
        assert_ne!(base_code, methods_code);
        assert_ne!(methods_code, tests_code);
        assert_ne!(base_code, tests_code);
    }
}
