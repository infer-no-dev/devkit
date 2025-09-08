//! Code generation and template tests

use crate::codegen::templates::*;
use std::collections::HashMap;

#[test]
fn test_template_manager_creation() {
    let template_manager = TemplateManager::new().expect("Failed to create template manager");
    
    // Test that built-in templates are loaded
    let templates = template_manager.list_templates();
    assert!(!templates.is_empty());
    
    // Test that common templates exist
    assert!(template_manager.has_template("rust_function"));
    assert!(template_manager.has_template("python_function"));
    assert!(template_manager.has_template("javascript_function"));
}

#[test]
fn test_template_retrieval() {
    let template_manager = TemplateManager::new().expect("Failed to create template manager");
    
    let rust_template = template_manager.get_template("rust_function");
    assert!(rust_template.is_some());
    
    if let Some(template) = rust_template {
        assert_eq!(template.name, "rust_function");
        assert_eq!(template.language, "rust");
        assert!(!template.content.is_empty());
        assert!(!template.variables.is_empty());
    }
}

#[test]
fn test_template_variable_system() {
    let template_manager = TemplateManager::new().expect("Failed to create template manager");
    
    if let Some(template) = template_manager.get_template("rust_function") {
        // Check that required variables exist
        let required_vars = template.required_variables();
        let optional_vars = template.optional_variables();
        
        assert!(!required_vars.is_empty(), "Rust function template should have required variables");
        
        // Test that we can find the name variable (which should be required)
        let name_var = template.variables.iter().find(|v| v.name == "name");
        assert!(name_var.is_some(), "Template should have a 'name' variable");
        
        if let Some(var) = name_var {
            assert!(var.required, "Name variable should be required");
        }
    }
}

#[test]
fn test_template_application() {
    let template_manager = TemplateManager::new().expect("Failed to create template manager");
    
    if let Some(template) = template_manager.get_template("rust_function") {
        let mut variables = HashMap::new();
        variables.insert("name".to_string(), "hello_world".to_string());
        variables.insert("description".to_string(), "A simple hello world function".to_string());
        variables.insert("parameters".to_string(), "".to_string());
        variables.insert("return_type".to_string(), "()".to_string());
        variables.insert("body".to_string(), "println!(\"Hello, World!\");".to_string());
        
        let result = template.apply(&variables);
        assert!(result.is_ok(), "Template application should succeed");
        
        if let Ok(generated_code) = result {
            assert!(generated_code.contains("hello_world"), "Generated code should contain function name");
            assert!(generated_code.contains("Hello, World!"), "Generated code should contain function body");
            assert!(generated_code.contains("pub fn"), "Generated code should be a public function");
        }
    }
}

#[test]
fn test_template_application_missing_required_variable() {
    let template_manager = TemplateManager::new().expect("Failed to create template manager");
    
    if let Some(template) = template_manager.get_template("rust_function") {
        let variables = HashMap::new(); // Empty variables map
        
        let result = template.apply(&variables);
        assert!(result.is_err(), "Template application should fail with missing required variables");
    }
}

#[test]
fn test_template_variables_with_defaults() {
    let template_manager = TemplateManager::new().expect("Failed to create template manager");
    
    if let Some(template) = template_manager.get_template("rust_function") {
        let mut variables = HashMap::new();
        variables.insert("name".to_string(), "test_function".to_string());
        // Don't provide optional variables - they should use defaults
        
        let result = template.apply(&variables);
        assert!(result.is_ok(), "Template application should succeed with defaults");
        
        if let Ok(generated_code) = result {
            assert!(generated_code.contains("test_function"), "Generated code should contain function name");
            // Should contain default values
            assert!(generated_code.contains("Generated function") || generated_code.contains("todo!"), 
                    "Generated code should contain default values");
        }
    }
}

#[test]
fn test_language_filtering() {
    let template_manager = TemplateManager::new().expect("Failed to create template manager");
    
    let rust_templates = template_manager.get_templates_for_language("rust");
    let python_templates = template_manager.get_templates_for_language("python");
    let js_templates = template_manager.get_templates_for_language("javascript");
    
    assert!(!rust_templates.is_empty(), "Should have Rust templates");
    assert!(!python_templates.is_empty(), "Should have Python templates");  
    assert!(!js_templates.is_empty(), "Should have JavaScript templates");
    
    // Test that templates are correctly filtered by language
    for template in rust_templates {
        assert_eq!(template.language, "rust");
    }
    
    for template in python_templates {
        assert_eq!(template.language, "python");
    }
}

#[test]
fn test_custom_template_creation() {
    let mut template_manager = TemplateManager::new_empty();
    
    let custom_template = Template {
        name: "test_template".to_string(),
        language: "rust".to_string(),
        description: "A test template".to_string(),
        content: "// Test template\nfn {{name}}() {\n    {{body}}\n}".to_string(),
        variables: vec![
            TemplateVariable::required("name", "Function name"),
            TemplateVariable::optional("body", "Function body", "// TODO: implement"),
        ],
    };
    
    template_manager.add_template(custom_template);
    
    // Test that the template was added
    assert!(template_manager.has_template("test_template"));
    
    // Test that we can retrieve and apply it
    if let Some(template) = template_manager.get_template("test_template") {
        let mut variables = HashMap::new();
        variables.insert("name".to_string(), "my_function".to_string());
        
        let result = template.apply(&variables);
        assert!(result.is_ok());
        
        if let Ok(code) = result {
            assert!(code.contains("my_function"));
            assert!(code.contains("// TODO: implement"));
        }
    }
}

#[test]
fn test_template_removal() {
    let mut template_manager = TemplateManager::new().expect("Failed to create template manager");
    
    // Ensure template exists
    assert!(template_manager.has_template("rust_function"));
    
    // Remove it
    let removed = template_manager.remove_template("rust_function");
    assert!(removed.is_some());
    
    // Verify it's gone
    assert!(!template_manager.has_template("rust_function"));
}

#[test] 
fn test_template_variable_types() {
    // Test creating different types of template variables
    
    let required_var = TemplateVariable::required("name", "The name");
    assert!(required_var.required);
    assert!(required_var.default_value.is_none());
    assert_eq!(required_var.name, "name");
    
    let optional_var = TemplateVariable::optional("description", "The description", "Default desc");
    assert!(!optional_var.required);
    assert!(optional_var.default_value.is_some());
    assert_eq!(optional_var.default_value.as_ref().unwrap(), "Default desc");
    
    let optional_no_default = TemplateVariable::optional_no_default("optional", "Optional field");
    assert!(!optional_no_default.required);
    assert!(optional_no_default.default_value.is_none());
}
