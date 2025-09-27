//! Code generation templates and template management.

use crate::codegen::CodeGenError;
use std::collections::HashMap;

/// Template manager for organizing and applying code templates
#[derive(Debug, Clone)]
pub struct TemplateManager {
    templates: HashMap<String, Template>,
}

/// A code template with variables that can be substituted
#[derive(Debug, Clone)]
pub struct Template {
    pub name: String,
    pub language: String,
    pub description: String,
    pub content: String,
    pub variables: Vec<TemplateVariable>,
}

/// A variable that can be substituted in a template
#[derive(Debug, Clone)]
pub struct TemplateVariable {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<String>,
}

impl TemplateManager {
    /// Create a new template manager with built-in templates
    pub fn new() -> Result<Self, CodeGenError> {
        let mut templates = HashMap::new();

        // Add built-in templates
        templates.insert(
            "rust_function".to_string(),
            Template {
                name: "rust_function".to_string(),
                language: "rust".to_string(),
                description: "Basic Rust function template".to_string(),
                content: r#"/// {{description}}
pub fn {{name}}({{parameters}}) -> {{return_type}} {{
    {{body}}
}}"#
                .to_string(),
                variables: vec![
                    TemplateVariable::required("name", "Function name"),
                    TemplateVariable::optional(
                        "description",
                        "Function description",
                        "Generated function",
                    ),
                    TemplateVariable::optional("parameters", "Function parameters", ""),
                    TemplateVariable::optional("return_type", "Return type", "()"),
                    TemplateVariable::optional(
                        "body",
                        "Function body",
                        r#"todo!("Implement this function")"#,
                    ),
                ],
            },
        );

        templates.insert(
            "rust_struct".to_string(),
            Template {
                name: "rust_struct".to_string(),
                language: "rust".to_string(),
                description: "Basic Rust struct template".to_string(),
                content: r#"/// {{description}}
#[derive(Debug, Clone)]
pub struct {{name}} {{
    {{fields}}
}}"#
                .to_string(),
                variables: vec![
                    TemplateVariable::required("name", "Struct name"),
                    TemplateVariable::optional(
                        "description",
                        "Struct description",
                        "Generated struct",
                    ),
                    TemplateVariable::optional("fields", "Struct fields", "// Add fields here"),
                ],
            },
        );

        templates.insert(
            "rust_impl".to_string(),
            Template {
                name: "rust_impl".to_string(),
                language: "rust".to_string(),
                description: "Basic Rust impl block template".to_string(),
                content: r#"impl {{name}} {{
    /// Create a new instance
    pub fn new() -> Self {{
        Self {{
            {{initialization}}
        }}
    }}
    
    {{methods}}
}}"#
                .to_string(),
                variables: vec![
                    TemplateVariable::required("name", "Type name"),
                    TemplateVariable::optional("initialization", "Field initialization", ""),
                    TemplateVariable::optional("methods", "Additional methods", ""),
                ],
            },
        );

        templates.insert(
            "rust_test".to_string(),
            Template {
                name: "rust_test".to_string(),
                language: "rust".to_string(),
                description: "Basic Rust test template".to_string(),
                content: r#"#[cfg(test)]
mod tests {{
    use super::*;
    
    #[test]
    fn {{test_name}}() {{
        {{test_body}}
    }}
}}"#
                .to_string(),
                variables: vec![
                    TemplateVariable::required("test_name", "Test function name"),
                    TemplateVariable::optional(
                        "test_body",
                        "Test body",
                        r#"assert!(true, "Test not implemented");"#,
                    ),
                ],
            },
        );

        templates.insert(
            "python_function".to_string(),
            Template {
                name: "python_function".to_string(),
                language: "python".to_string(),
                description: "Basic Python function template".to_string(),
                content: r#"def {{name}}({{parameters}}):
    """{{description}}"""
    {{body}}"#
                    .to_string(),
                variables: vec![
                    TemplateVariable::required("name", "Function name"),
                    TemplateVariable::optional(
                        "description",
                        "Function description",
                        "Generated function",
                    ),
                    TemplateVariable::optional("parameters", "Function parameters", ""),
                    TemplateVariable::optional("body", "Function body", "pass"),
                ],
            },
        );

        templates.insert(
            "python_class".to_string(),
            Template {
                name: "python_class".to_string(),
                language: "python".to_string(),
                description: "Basic Python class template".to_string(),
                content: r#"class {{name}}:
    """{{description}}"""
    
    def __init__(self{{init_parameters}}):
        """Initialize the class."""
        {{init_body}}
        
    {{methods}}"#
                    .to_string(),
                variables: vec![
                    TemplateVariable::required("name", "Class name"),
                    TemplateVariable::optional(
                        "description",
                        "Class description",
                        "Generated class",
                    ),
                    TemplateVariable::optional("init_parameters", "Constructor parameters", ""),
                    TemplateVariable::optional("init_body", "Constructor body", "pass"),
                    TemplateVariable::optional("methods", "Additional methods", ""),
                ],
            },
        );

        templates.insert(
            "javascript_function".to_string(),
            Template {
                name: "javascript_function".to_string(),
                language: "javascript".to_string(),
                description: "Basic JavaScript function template".to_string(),
                content: r#"/**
 * {{description}}
 * {{param_docs}}
 * @returns {{return_description}}
 */
function {{name}}({{parameters}}) {{
    {{body}}
}}"#
                .to_string(),
                variables: vec![
                    TemplateVariable::required("name", "Function name"),
                    TemplateVariable::optional(
                        "description",
                        "Function description",
                        "Generated function",
                    ),
                    TemplateVariable::optional("parameters", "Function parameters", ""),
                    TemplateVariable::optional("param_docs", "Parameter documentation", ""),
                    TemplateVariable::optional(
                        "return_description",
                        "Return value description",
                        "void",
                    ),
                    TemplateVariable::optional("body", "Function body", "// TODO: Implement"),
                ],
            },
        );

        Ok(Self { templates })
    }

    /// Create an empty template manager without built-in templates
    pub fn new_empty() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Get a template by name
    pub fn get_template(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }

    /// Get all templates for a specific language
    pub fn get_templates_for_language(&self, language: &str) -> Vec<&Template> {
        self.templates
            .values()
            .filter(|template| template.language == language)
            .collect()
    }

    /// Get all available template names
    pub fn list_templates(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// Apply a template with the given variables
    pub fn apply_template(
        &self,
        template_name: &str,
        variables: &HashMap<String, String>,
    ) -> Result<String, CodeGenError> {
        if let Some(template) = self.get_template(template_name) {
            template.apply(variables)
        } else {
            Err(CodeGenError::TemplateError(format!(
                "Template '{}' not found",
                template_name
            )))
        }
    }

    /// Add a custom template
    pub fn add_template(&mut self, template: Template) {
        self.templates.insert(template.name.clone(), template);
    }

    /// Remove a template
    pub fn remove_template(&mut self, name: &str) -> Option<Template> {
        self.templates.remove(name)
    }

    /// Check if a template exists
    pub fn has_template(&self, name: &str) -> bool {
        self.templates.contains_key(name)
    }
}

impl Template {
    /// Apply this template with the given variables
    pub fn apply(&self, variables: &HashMap<String, String>) -> Result<String, CodeGenError> {
        let mut result = self.content.clone();

        // Check for required variables
        for var in &self.variables {
            if var.required && !variables.contains_key(&var.name) {
                return Err(CodeGenError::TemplateError(format!(
                    "Required variable '{}' not provided for template '{}'",
                    var.name, self.name
                )));
            }
        }

        // Replace template variables
        for var in &self.variables {
            let value = if let Some(provided_value) = variables.get(&var.name) {
                provided_value.clone()
            } else if let Some(default_value) = &var.default_value {
                default_value.clone()
            } else {
                continue; // Skip optional variables without defaults
            };

            let placeholder = format!("{{{{{}}}}}", var.name);
            result = result.replace(&placeholder, &value);
        }

        Ok(result)
    }

    /// Get required variables for this template
    pub fn required_variables(&self) -> Vec<&TemplateVariable> {
        self.variables.iter().filter(|var| var.required).collect()
    }

    /// Get optional variables for this template
    pub fn optional_variables(&self) -> Vec<&TemplateVariable> {
        self.variables.iter().filter(|var| !var.required).collect()
    }
}

impl TemplateVariable {
    /// Create a required template variable
    pub fn required(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            required: true,
            default_value: None,
        }
    }

    /// Create an optional template variable with a default value
    pub fn optional(name: &str, description: &str, default_value: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            required: false,
            default_value: Some(default_value.to_string()),
        }
    }

    /// Create an optional template variable without a default value
    pub fn optional_no_default(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            required: false,
            default_value: None,
        }
    }
}
