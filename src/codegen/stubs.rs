//! Stub-based code generation
//!
//! This module provides stub-based code generation for various programming languages.
//! It serves as a reliable fallback or development implementation that can be used
//! when full AI services are not available or needed.

use std::path::PathBuf;

/// Generate code based on a prompt and language
pub fn generate_code_stub(prompt: &str, language: Option<&str>) -> String {
    let lang = language.unwrap_or("generic");
    
    match lang {
        "rust" | "rs" => generate_rust_stub(prompt),
        "python" | "py" => generate_python_stub(prompt),
        "javascript" | "js" => generate_javascript_stub(prompt),
        "typescript" | "ts" => generate_typescript_stub(prompt),
        "go" => generate_go_stub(prompt),
        "java" => generate_java_stub(prompt),
        "cpp" | "c++" => generate_cpp_stub(prompt),
        "c" => generate_c_stub(prompt),
        _ => generate_generic_stub(prompt),
    }
}

/// Infer programming language from a prompt or file path
pub fn infer_language_from_context(prompt: &str, file_path: Option<&PathBuf>) -> Option<String> {
    // First try to infer from file extension
    if let Some(path) = file_path {
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            let lang = match extension {
                "rs" => "rust",
                "py" => "python",
                "js" => "javascript",
                "ts" => "typescript",
                "go" => "go",
                "java" => "java",
                "cpp" | "cc" | "cxx" => "cpp",
                "c" => "c",
                _ => return None,
            };
            return Some(lang.to_string());
        }
    }
    
    // Then try to infer from prompt content
    let prompt_lower = prompt.to_lowercase();
    
    if prompt_lower.contains("rust") || prompt_lower.contains("cargo") {
        Some("rust".to_string())
    } else if prompt_lower.contains("python") || prompt_lower.contains("django") || prompt_lower.contains("flask") {
        Some("python".to_string())
    } else if prompt_lower.contains("javascript") || prompt_lower.contains("node") || prompt_lower.contains("npm") {
        Some("javascript".to_string())
    } else if prompt_lower.contains("typescript") || prompt_lower.contains("tsx") {
        Some("typescript".to_string())
    } else if prompt_lower.contains("golang") || prompt_lower.contains("go ") {
        Some("go".to_string())
    } else if prompt_lower.contains("java") && !prompt_lower.contains("javascript") {
        Some("java".to_string())
    } else if prompt_lower.contains("c++") || prompt_lower.contains("cpp") {
        Some("cpp".to_string())
    } else if prompt_lower.contains(" c ") || prompt_lower.contains("clang") {
        Some("c".to_string())
    } else {
        None
    }
}

/// Generate suggested file name based on prompt and language
pub fn suggest_filename(prompt: &str, language: Option<&str>) -> String {
    let sanitized_prompt = prompt
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == ' ')
        .take(30)
        .collect::<String>()
        .trim()
        .replace(' ', "_")
        .to_lowercase();
    
    let extension = match language {
        Some("rust") => "rs",
        Some("python") => "py",
        Some("javascript") => "js",
        Some("typescript") => "ts",
        Some("go") => "go",
        Some("java") => "java",
        Some("cpp") => "cpp",
        Some("c") => "c",
        _ => "txt",
    };
    
    if sanitized_prompt.is_empty() {
        format!("generated_code.{}", extension)
    } else {
        format!("{}.{}", sanitized_prompt, extension)
    }
}

fn generate_rust_stub(prompt: &str) -> String {
    if prompt.to_lowercase().contains("function") || prompt.to_lowercase().contains("fn") {
        format!(
            "// Generated from: {}\n\n/// TODO: Implement the requested functionality\npub fn generated_function() -> Result<(), Box<dyn std::error::Error>> {{\n    // Implementation goes here\n    println!(\"Hello from generated Rust function!\");\n    Ok(())\n}}\n\n#[cfg(test)]\nmod tests {{\n    use super::*;\n\n    #[test]\n    fn test_generated_function() {{\n        assert!(generated_function().is_ok());\n    }}\n}}",
            prompt
        )
    } else if prompt.to_lowercase().contains("struct") {
        format!(
            "// Generated from: {}\n\n#[derive(Debug, Clone)]\npub struct GeneratedStruct {{\n    // Add fields here\n    pub field1: String,\n    pub field2: i32,\n}}\n\nimpl GeneratedStruct {{\n    pub fn new() -> Self {{\n        Self {{\n            field1: String::new(),\n            field2: 0,\n        }}\n    }}\n}}",
            prompt
        )
    } else {
        format!(
            "// Generated from: {}\n\nfn main() {{\n    println!(\"Generated Rust code!\");\n    // TODO: Implement based on: {}\n}}",
            prompt, prompt
        )
    }
}

fn generate_python_stub(prompt: &str) -> String {
    if prompt.to_lowercase().contains("function") || prompt.to_lowercase().contains("def") {
        format!(
            "# Generated from: {}\n\ndef generated_function():\n    \"\"\"\n    TODO: Implement the requested functionality\n    \"\"\"\n    print(\"Hello from generated Python function!\")\n    return True\n\n\nif __name__ == \"__main__\":\n    result = generated_function()\n    print(f\"Result: {{result}}\")",
            prompt
        )
    } else if prompt.to_lowercase().contains("class") {
        format!(
            "# Generated from: {}\n\nclass GeneratedClass:\n    def __init__(self):\n        self.field1 = \"\"\n        self.field2 = 0\n    \n    def method1(self):\n        # TODO: Implement method\n        pass\n\n\nif __name__ == \"__main__\":\n    instance = GeneratedClass()\n    print(\"Generated Python class!\")",
            prompt
        )
    } else {
        format!(
            "# Generated from: {}\n\ndef main():\n    print(\"Generated Python code!\")\n    # TODO: Implement based on: {}\n\nif __name__ == \"__main__\":\n    main()",
            prompt, prompt
        )
    }
}

fn generate_javascript_stub(prompt: &str) -> String {
    if prompt.to_lowercase().contains("function") {
        format!(
            "// Generated from: {}\n\n/**\n * TODO: Implement the requested functionality\n */\nfunction generatedFunction() {{\n    console.log('Hello from generated JavaScript function!');\n    return true;\n}}\n\n// Example usage\nconsole.log('Result:', generatedFunction());",
            prompt
        )
    } else if prompt.to_lowercase().contains("class") {
        format!(
            "// Generated from: {}\n\nclass GeneratedClass {{\n    constructor() {{\n        this.field1 = '';\n        this.field2 = 0;\n    }}\n\n    method1() {{\n        // TODO: Implement method\n        console.log('Generated method');\n    }}\n}}\n\n// Example usage\nconst instance = new GeneratedClass();\ninstance.method1();",
            prompt
        )
    } else {
        format!(
            "// Generated from: {}\n\nconsole.log('Generated JavaScript code!');\n// TODO: Implement based on: {}",
            prompt, prompt
        )
    }
}

fn generate_typescript_stub(prompt: &str) -> String {
    if prompt.to_lowercase().contains("function") {
        format!(
            "// Generated from: {}\n\n/**\n * TODO: Implement the requested functionality\n */\nfunction generatedFunction(): boolean {{\n    console.log('Hello from generated TypeScript function!');\n    return true;\n}}\n\n// Example usage\nconst result: boolean = generatedFunction();\nconsole.log('Result:', result);",
            prompt
        )
    } else if prompt.to_lowercase().contains("interface") || prompt.to_lowercase().contains("type") {
        format!(
            "// Generated from: {}\n\ninterface GeneratedInterface {{\n    field1: string;\n    field2: number;\n    method1(): void;\n}}\n\nclass GeneratedClass implements GeneratedInterface {{\n    field1: string = '';\n    field2: number = 0;\n\n    method1(): void {{\n        console.log('Generated method');\n    }}\n}}",
            prompt
        )
    } else {
        format!(
            "// Generated from: {}\n\nconsole.log('Generated TypeScript code!');\n// TODO: Implement based on: {}",
            prompt, prompt
        )
    }
}

fn generate_go_stub(prompt: &str) -> String {
    if prompt.to_lowercase().contains("function") || prompt.to_lowercase().contains("func") {
        format!(
            "// Generated from: {}\n\npackage main\n\nimport \"fmt\"\n\n// TODO: Implement the requested functionality\nfunc generatedFunction() error {{\n    fmt.Println(\"Hello from generated Go function!\")\n    return nil\n}}\n\nfunc main() {{\n    if err := generatedFunction(); err != nil {{\n        fmt.Printf(\"Error: %v\\n\", err)\n    }}\n}}",
            prompt
        )
    } else {
        format!(
            "// Generated from: {}\n\npackage main\n\nimport \"fmt\"\n\nfunc main() {{\n    fmt.Println(\"Generated Go code!\")\n    // TODO: Implement based on: {}\n}}",
            prompt, prompt
        )
    }
}

fn generate_java_stub(prompt: &str) -> String {
    if prompt.to_lowercase().contains("class") {
        format!(
            "// Generated from: {}\n\npublic class GeneratedClass {{\n    private String field1;\n    private int field2;\n\n    public GeneratedClass() {{\n        this.field1 = \"\";\n        this.field2 = 0;\n    }}\n\n    public void method1() {{\n        System.out.println(\"Generated method\");\n        // TODO: Implement method\n    }}\n\n    public static void main(String[] args) {{\n        GeneratedClass instance = new GeneratedClass();\n        instance.method1();\n    }}\n}}",
            prompt
        )
    } else {
        format!(
            "// Generated from: {}\n\npublic class Main {{\n    public static void main(String[] args) {{\n        System.out.println(\"Generated Java code!\");\n        // TODO: Implement based on: {}\n    }}\n}}",
            prompt, prompt
        )
    }
}

fn generate_cpp_stub(prompt: &str) -> String {
    if prompt.to_lowercase().contains("class") {
        format!(
            "// Generated from: {}\n\n#include <iostream>\n#include <string>\n\nclass GeneratedClass {{\npublic:\n    GeneratedClass() : field1(\"\"), field2(0) {{}}\n    \n    void method1() {{\n        std::cout << \"Generated method\" << std::endl;\n        // TODO: Implement method\n    }}\n\nprivate:\n    std::string field1;\n    int field2;\n}};\n\nint main() {{\n    GeneratedClass instance;\n    instance.method1();\n    return 0;\n}}",
            prompt
        )
    } else {
        format!(
            "// Generated from: {}\n\n#include <iostream>\n\nint main() {{\n    std::cout << \"Generated C++ code!\" << std::endl;\n    // TODO: Implement based on: {}\n    return 0;\n}}",
            prompt, prompt
        )
    }
}

fn generate_c_stub(prompt: &str) -> String {
    if prompt.to_lowercase().contains("function") {
        format!(
            "// Generated from: {}\n\n#include <stdio.h>\n\n// TODO: Implement the requested functionality\nint generated_function() {{\n    printf(\"Hello from generated C function!\\n\");\n    return 0;\n}}\n\nint main() {{\n    return generated_function();\n}}",
            prompt
        )
    } else {
        format!(
            "// Generated from: {}\n\n#include <stdio.h>\n\nint main() {{\n    printf(\"Generated C code!\\n\");\n    // TODO: Implement based on: {}\n    return 0;\n}}",
            prompt, prompt
        )
    }
}

fn generate_generic_stub(prompt: &str) -> String {
    format!(
        "# Generated from: {}\n\n# TODO: Implement the requested functionality\n# Prompt: {}\n\nprint(\"Generated generic code!\")\n# Add your implementation here",
        prompt, prompt
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_generate_rust_stub() {
        let result = generate_code_stub("create a rust function", Some("rust"));
        assert!(result.contains("fn generated_function"));
        assert!(result.contains("TODO: Implement"));
    }

    #[test]
    fn test_infer_language_from_path() {
        let path = PathBuf::from("test.rs");
        let result = infer_language_from_context("some prompt", Some(&path));
        assert_eq!(result, Some("rust".to_string()));
    }

    #[test]
    fn test_infer_language_from_prompt() {
        let result = infer_language_from_context("create a rust function", None);
        assert_eq!(result, Some("rust".to_string()));
    }

    #[test]
    fn test_suggest_filename() {
        let result = suggest_filename("create hello world function", Some("rust"));
        assert_eq!(result, "create_hello_world_function.rs");
    }
}