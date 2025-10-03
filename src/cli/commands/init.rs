//! Project initialization command implementation
//!
//! This module handles creating new agentic development projects with:
//! - Template-based project scaffolding
//! - Language-specific configurations
//! - Interactive setup wizards
//! - Git repository initialization
//! - Best-practice project structures

use super::utils::*;
use crate::cli::{CliRunner, InitArgs};
use chrono::Datelike;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Project template structure
#[derive(Debug, Clone)]
pub struct ProjectTemplate {
    pub name: String,
    pub language: String,
    pub description: String,
    pub files: HashMap<PathBuf, String>,
    pub directories: Vec<PathBuf>,
    pub config: TemplateConfig,
}

/// Template configuration
#[derive(Debug, Clone)]
pub struct TemplateConfig {
    pub dependencies: Vec<String>,
    pub dev_dependencies: Vec<String>,
    pub scripts: HashMap<String, String>,
    pub git_ignore_patterns: Vec<String>,
    pub recommended_extensions: Vec<String>,
}

/// Available project templates
pub fn get_available_templates() -> HashMap<String, ProjectTemplate> {
    let mut templates = HashMap::new();

    // Rust template
    templates.insert(
        "rust".to_string(),
        ProjectTemplate {
            name: "Rust Project".to_string(),
            language: "rust".to_string(),
            description: "A modern Rust project with best practices".to_string(),
            files: rust_template_files(),
            directories: vec![
                PathBuf::from("src"),
                PathBuf::from("tests"),
                PathBuf::from("examples"),
                PathBuf::from("benches"),
            ],
            config: TemplateConfig {
                dependencies: vec![
                    "serde = { version = \"1.0\", features = [\"derive\"] }".to_string(),
                    "tokio = { version = \"1.0\", features = [\"full\"] }".to_string(),
                    "clap = { version = \"4.0\", features = [\"derive\"] }".to_string(),
                ],
                dev_dependencies: vec![
                    "criterion = \"0.5\"".to_string(),
                    "tempfile = \"3.0\"".to_string(),
                ],
                scripts: HashMap::new(),
                git_ignore_patterns: vec![
                    "/target/".to_string(),
                    "Cargo.lock".to_string(),
                    "*.orig".to_string(),
                    ".DS_Store".to_string(),
                ],
                recommended_extensions: vec!["rust-analyzer".to_string(), "crates".to_string()],
            },
        },
    );

    // Python template
    templates.insert(
        "python".to_string(),
        ProjectTemplate {
            name: "Python Project".to_string(),
            language: "python".to_string(),
            description: "A Python project with modern tooling".to_string(),
            files: python_template_files(),
            directories: vec![
                PathBuf::from("src"),
                PathBuf::from("tests"),
                PathBuf::from("docs"),
            ],
            config: TemplateConfig {
                dependencies: vec!["click>=8.0.0".to_string(), "requests>=2.28.0".to_string()],
                dev_dependencies: vec![
                    "pytest>=7.0.0".to_string(),
                    "black>=22.0.0".to_string(),
                    "flake8>=5.0.0".to_string(),
                    "mypy>=0.991".to_string(),
                ],
                scripts: HashMap::from([
                    ("test".to_string(), "pytest".to_string()),
                    ("format".to_string(), "black src tests".to_string()),
                    ("lint".to_string(), "flake8 src tests".to_string()),
                    ("typecheck".to_string(), "mypy src".to_string()),
                ]),
                git_ignore_patterns: vec![
                    "__pycache__/".to_string(),
                    "*.py[cod]".to_string(),
                    "*.egg-info/".to_string(),
                    "build/".to_string(),
                    "dist/".to_string(),
                    ".pytest_cache/".to_string(),
                    ".coverage".to_string(),
                ],
                recommended_extensions: vec!["python".to_string(), "pylance".to_string()],
            },
        },
    );

    // JavaScript/TypeScript template
    templates.insert(
        "typescript".to_string(),
        ProjectTemplate {
            name: "TypeScript Project".to_string(),
            language: "typescript".to_string(),
            description: "A TypeScript project with modern tooling".to_string(),
            files: typescript_template_files(),
            directories: vec![
                PathBuf::from("src"),
                PathBuf::from("tests"),
                PathBuf::from("dist"),
            ],
            config: TemplateConfig {
                dependencies: vec!["typescript".to_string(), "@types/node".to_string()],
                dev_dependencies: vec![
                    "jest".to_string(),
                    "@types/jest".to_string(),
                    "eslint".to_string(),
                    "prettier".to_string(),
                ],
                scripts: HashMap::from([
                    ("build".to_string(), "tsc".to_string()),
                    ("test".to_string(), "jest".to_string()),
                    ("format".to_string(), "prettier --write src".to_string()),
                    ("lint".to_string(), "eslint src --ext .ts".to_string()),
                ]),
                git_ignore_patterns: vec![
                    "node_modules/".to_string(),
                    "dist/".to_string(),
                    "*.log".to_string(),
                    ".env".to_string(),
                ],
                recommended_extensions: vec![
                    "typescript".to_string(),
                    "eslint".to_string(),
                    "prettier".to_string(),
                ],
            },
        },
    );

    templates
}

/// Run the init command
pub async fn run(runner: &mut CliRunner, args: InitArgs) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info(&format!(
        "🚀 Initializing new agentic development project: {}",
        args.name
    ));

    // Validate project name
    if args.name.is_empty() {
        return Err("Project name cannot be empty".into());
    }

    let project_path = PathBuf::from(&args.name);

    // Check if directory already exists
    if project_path.exists() && !args.force {
        if project_path.is_dir() && fs::read_dir(&project_path)?.next().is_some() {
            return Err(format!(
                "Directory '{}' already exists and is not empty. Use --force to overwrite.",
                args.name
            )
            .into());
        }
    }

    // Create project directory
    ensure_directory(&project_path)?;

    // Determine template and language
    let templates = get_available_templates();
    let (template_name, language) = if let Some(template) = &args.template {
        if !templates.contains_key(template) {
            runner.print_error(&format!("Template '{}' not found", template));
            runner.print_info("Available templates:");
            for (name, template) in &templates {
                runner.print_info(&format!("  {} - {}", name, template.description));
            }
            return Err("Invalid template".into());
        }
        (template.clone(), templates[template].language.clone())
    } else if let Some(lang) = &args.language {
        let template_name = templates
            .iter()
            .find(|(_, t)| t.language == *lang)
            .map(|(name, _)| name.clone())
            .unwrap_or_else(|| lang.clone());
        (template_name, lang.clone())
    } else if !args.no_interactive {
        // Interactive template selection
        runner.print_info("🤖 Let's set up your agentic development project!\n");

        // Detect language from current directory if possible
        let detected_lang = detect_project_language(&std::env::current_dir()?);

        let language = if let Some(detected) = detected_lang {
            runner.print_info(&format!("Detected language: {}", detected));
            if confirm_action("Use detected language?", true)? {
                detected
            } else {
                select_language_interactive(&templates)?
            }
        } else {
            select_language_interactive(&templates)?
        };

        let template_name = templates
            .iter()
            .find(|(_, t)| t.language == language)
            .map(|(name, _)| name.clone())
            .unwrap_or_else(|| language.clone());

        (template_name, language)
    } else {
        // Default to rust template
        ("rust".to_string(), "rust".to_string())
    };

    // Get template
    let template = templates
        .get(&template_name)
        .ok_or(format!("Template '{}' not found", template_name))?;

    runner.print_info(&format!(
        "📋 Using template: {} ({})",
        template.name, template.description
    ));

    // Create project structure
    create_project_structure(&project_path, template, &args, runner).await?;

    // Initialize git repository if requested
    if args.git {
        initialize_git_repository(&project_path, runner).await?;
    }

    // Create agentic development configuration
    create_agentic_config(&project_path, &language, runner).await?;

    runner.print_success(&format!(
        "✨ Project '{}' initialized successfully!",
        args.name
    ));
    runner.print_info("\nNext steps:");
    runner.print_info(&format!("  cd {}", args.name));

    match language.as_str() {
        "rust" => {
            runner.print_info("  cargo build");
            runner.print_info("  agentic-dev analyze");
        }
        "python" => {
            runner.print_info("  python -m venv venv");
            runner.print_info(
                "  source venv/bin/activate  # or .\\venv\\Scripts\\activate on Windows",
            );
            runner.print_info("  pip install -e .");
            runner.print_info("  agentic-dev analyze");
        }
        "typescript" => {
            runner.print_info("  npm install");
            runner.print_info("  npm run build");
            runner.print_info("  agentic-dev analyze");
        }
        _ => {
            runner.print_info("  agentic-dev analyze");
        }
    }

    runner.print_info("  agentic-dev interactive  # Start interactive development mode");

    Ok(())
}

/// Interactive language selection
fn select_language_interactive(
    templates: &HashMap<String, ProjectTemplate>,
) -> Result<String, Box<dyn std::error::Error>> {
    println!("Available languages:");
    let languages: Vec<_> = templates.values().map(|t| t.language.as_str()).collect();

    for (i, template) in templates.values().enumerate() {
        println!(
            "  {}. {} - {}",
            i + 1,
            template.language,
            template.description
        );
    }

    loop {
        let input = get_user_input("Select language (enter number)", None)?;

        if let Ok(num) = input.parse::<usize>() {
            if num > 0 && num <= languages.len() {
                return Ok(languages[num - 1].to_string());
            }
        }

        // Try direct language name
        if templates.values().any(|t| t.language == input) {
            return Ok(input);
        }

        println!(
            "Invalid selection. Please enter a number from 1 to {} or a language name.",
            languages.len()
        );
    }
}

/// Create the project structure from template
async fn create_project_structure(
    project_path: &Path,
    template: &ProjectTemplate,
    args: &InitArgs,
    runner: &CliRunner,
) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info("📁 Creating project structure...");

    // Create directories
    for dir in &template.directories {
        let dir_path = project_path.join(dir);
        ensure_directory(&dir_path)?;
        runner.print_verbose(&format!("Created directory: {}", dir.display()));
    }

    // Create template variables
    let mut template_vars = HashMap::new();
    template_vars.insert("project_name".to_string(), args.name.clone());
    template_vars.insert("language".to_string(), template.language.clone());
    template_vars.insert("year".to_string(), chrono::Utc::now().year().to_string());

    // Create files from templates
    for (file_path, content_template) in &template.files {
        // Process template variables in file path
        let file_path_str = file_path.to_string_lossy();
        let processed_file_path_str = process_template_variables(&file_path_str, &template_vars);
        let processed_file_path = PathBuf::from(processed_file_path_str);
        let full_path = project_path.join(&processed_file_path);

        // Ensure parent directory exists
        if let Some(parent) = full_path.parent() {
            ensure_directory(parent)?;
        }

        // Process template variables in content
        let content = process_template_variables(content_template, &template_vars);

        fs::write(&full_path, content)?;
        runner.print_verbose(&format!("Created file: {}", processed_file_path.display()));
    }

    // Create language-specific configuration files
    create_language_config(project_path, template, args, runner).await?;

    Ok(())
}

/// Create language-specific configuration files
async fn create_language_config(
    project_path: &Path,
    template: &ProjectTemplate,
    args: &InitArgs,
    _runner: &CliRunner,
) -> Result<(), Box<dyn std::error::Error>> {
    match template.language.as_str() {
        "rust" => {
            let cargo_toml = project_path.join("Cargo.toml");
            // Sanitize project name for Cargo package/bin names
            let crate_name = sanitize_crate_name(&args.name);
            let cargo_content = format!(
                r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
authors = ["TODO: Your Name <your.email@example.com>"]
description = "TODO: Add description"
license = "MIT OR Apache-2.0"

[dependencies]
{}

[dev-dependencies]
{}

[[bin]]
name = "{}"
path = "src/main.rs"
"#,
                crate_name,
                template.config.dependencies.join("\n"),
                template.config.dev_dependencies.join("\n"),
                crate_name
            );
            fs::write(&cargo_toml, cargo_content)?;
        }

        "python" => {
            let setup_py = project_path.join("setup.py");
            let setup_content = format!(
                r#"from setuptools import setup, find_packages

setup(
    name="{}",
    version="0.1.0",
    packages=find_packages(where="src"),
    package_dir={{"": "src"}},
    python_requires=">=3.8",
    install_requires=[
        {}
    ],
    extras_require={{
        "dev": [
            {}
        ]
    }},
    entry_points={{
        "console_scripts": [
            "{}={}:main",
        ],
    }},
)
"#,
                args.name,
                template
                    .config
                    .dependencies
                    .iter()
                    .map(|d| format!("\"{}\"", d))
                    .collect::<Vec<_>>()
                    .join(",\n        "),
                template
                    .config
                    .dev_dependencies
                    .iter()
                    .map(|d| format!("\"{}\"", d))
                    .collect::<Vec<_>>()
                    .join(",\n            "),
                args.name,
                args.name.replace("-", "_")
            );
            fs::write(&setup_py, setup_content)?;

            // Create requirements files
            let requirements_txt = project_path.join("requirements.txt");
            fs::write(&requirements_txt, template.config.dependencies.join("\n"))?;

            let requirements_dev_txt = project_path.join("requirements-dev.txt");
            fs::write(
                &requirements_dev_txt,
                format!(
                    "-r requirements.txt\n{}",
                    template.config.dev_dependencies.join("\n")
                ),
            )?;
        }

        "typescript" => {
            let package_json = project_path.join("package.json");
            let package_content = serde_json::json!({
                "name": args.name,
                "version": "0.1.0",
                "description": "TODO: Add description",
                "main": "dist/index.js",
                "types": "dist/index.d.ts",
                "scripts": template.config.scripts,
                "dependencies": {
                    "typescript": "^4.9.0",
                    "@types/node": "^18.0.0"
                },
                "devDependencies": {
                    "jest": "^29.0.0",
                    "@types/jest": "^29.0.0",
                    "eslint": "^8.0.0",
                    "prettier": "^2.8.0"
                },
                "engines": {
                    "node": ">=16.0.0"
                }
            });
            fs::write(
                &package_json,
                serde_json::to_string_pretty(&package_content)?,
            )?;

            // Create tsconfig.json
            let tsconfig_json = project_path.join("tsconfig.json");
            let tsconfig_content = serde_json::json!({
                "compilerOptions": {
                    "target": "ES2020",
                    "module": "commonjs",
                    "outDir": "./dist",
                    "rootDir": "./src",
                    "strict": true,
                    "esModuleInterop": true,
                    "skipLibCheck": true,
                    "forceConsistentCasingInFileNames": true,
                    "declaration": true,
                    "declarationMap": true,
                    "sourceMap": true
                },
                "include": ["src/**/*"],
                "exclude": ["node_modules", "dist", "tests"]
            });
            fs::write(
                &tsconfig_json,
                serde_json::to_string_pretty(&tsconfig_content)?,
            )?;
        }

        _ => {}
    }

    // Create .gitignore
    let gitignore_path = project_path.join(".gitignore");
    fs::write(
        &gitignore_path,
        template.config.git_ignore_patterns.join("\n"),
    )?;

    Ok(())
}

/// Initialize git repository
async fn initialize_git_repository(
    project_path: &Path,
    runner: &CliRunner,
) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info("🌱 Initializing git repository...");

    let output = std::process::Command::new("git")
        .arg("init")
        .current_dir(project_path)
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "Failed to initialize git repository: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    // Create initial commit
    std::process::Command::new("git")
        .args(&["add", "."])
        .current_dir(project_path)
        .output()?;

    std::process::Command::new("git")
        .args(&["commit", "-m", "Initial commit from agentic-dev init"])
        .current_dir(project_path)
        .output()?;

    runner.print_success("Git repository initialized with initial commit");

    Ok(())
}

/// Create agentic development configuration
async fn create_agentic_config(
    project_path: &Path,
    language: &str,
    runner: &CliRunner,
) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info("⚙️  Creating agentic development configuration...");

    let config_path = project_path.join(".agentic-config.toml");
    let config_content = format!(
        r#"# Agentic Development Environment Configuration
# This file was generated by `agentic-dev init`

[general]
project_name = "{}"
language = "{}"
log_level = "info"
auto_save = true

[agents]
max_concurrent_agents = 3
agent_timeout_seconds = 300
default_agent_priority = "normal"

[codegen]
[codegen.default_style]
indentation = "spaces"
indent_size = {}
line_length = {}
naming_convention = "{}"
include_comments = true
include_type_hints = true

[context]
analysis_depth = "normal"
include_tests = true
cache_results = true

[shell]
command_timeout = 30
history_enabled = true

[ui]
theme = "dark"
show_line_numbers = true
show_timestamps = true
"#,
        project_path.file_name().unwrap().to_string_lossy(),
        language,
        match language {
            "python" => 4,
            "rust" => 4,
            "typescript" | "javascript" => 2,
            _ => 4,
        },
        match language {
            "python" => 88,
            "rust" => 100,
            "typescript" | "javascript" => 80,
            _ => 80,
        },
        match language {
            "python" => "snake_case",
            "rust" => "snake_case",
            "typescript" | "javascript" => "camelCase",
            "java" => "camelCase",
            _ => "snake_case",
        }
    );

    fs::write(&config_path, config_content)?;
    runner.print_success("Created .agentic-config.toml");

    Ok(())
}

/// Process template variables in content
fn process_template_variables(content: &str, vars: &HashMap<String, String>) -> String {
    let mut result = content.to_string();

    for (key, value) in vars {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }

    result
}

/// Convert a provided project identifier (which may be a path like "./my-app")
/// into a valid Cargo crate name. This:
/// - uses only the final path segment
/// - lowercases the name
/// - replaces invalid characters with underscores
/// - ensures the first character is a letter or underscore
fn sanitize_crate_name(input: &str) -> String {
    use std::path::Path;
    let base = Path::new(input)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(input);

    let mut out: String = base
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'a'..='z' | '0'..='9' | '_' | '-' => if c == '-' { '_' } else { c },
            _ => '_',
        })
        .collect();

    if out.is_empty() {
        out = "project".to_string();
    }

    let first = out.chars().next().unwrap();
    if !matches!(first, 'a'..='z' | '_') {
        out.insert(0, '_');
    }

    out
}

// Template file contents
fn rust_template_files() -> HashMap<PathBuf, String> {
    let mut files = HashMap::new();

    files.insert(
        PathBuf::from("src/main.rs"),
        r#"//! {{project_name}} - An agentic development project
//!
//! This project was generated by the Agentic Development Environment.

use std::env;
use std::process;

fn main() {
    println!("🤖 Welcome to {{project_name}}!");
    println!("This project was created with agentic-dev.");
    
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                print_help();
            }
            "--version" | "-V" => {
                println!("{{project_name}} version 0.1.0");
            }
            _ => {
                println!("Unknown argument: {}", args[1]);
                print_help();
                process::exit(1);
            }
        }
    } else {
        println!("Run with --help for usage information.");
    }
}

fn print_help() {
    println!("Usage: {{project_name}} [OPTIONS]");
    println!("");
    println!("Options:");
    println!("  -h, --help     Print this help message");
    println!("  -V, --version  Print version information");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // Add your tests here
        assert_eq!(2 + 2, 4);
    }
}
"#
        .to_string(),
    );

    files.insert(
        PathBuf::from("src/lib.rs"),
        r#"//! {{project_name}} library
//!
//! This library was generated by the Agentic Development Environment.

/// A sample function to demonstrate the project structure
pub fn hello_world() -> String {
    "Hello from {{project_name}}!".to_string()
}

/// Add your library functions here
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world() {
        assert_eq!(hello_world(), "Hello from {{project_name}}!");
    }

    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }
}
"#
        .to_string(),
    );

    files.insert(
        PathBuf::from("README.md"),
        r#"# {{project_name}}

A Rust project created with the Agentic Development Environment.

## Getting Started

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Agentic Development Environment

### Building

```bash
cargo build
```

### Running

```bash
cargo run
```

### Testing

```bash
cargo test
```

## Development with Agentic Environment

This project is set up to work with the Agentic Development Environment:

```bash
# Analyze the codebase
agentic-dev analyze

# Start interactive development mode
agentic-dev interactive

# Generate code with AI assistance
agentic-dev generate "create a function that processes user input"
```

## Project Structure

- `src/main.rs` - Main application entry point
- `src/lib.rs` - Library code
- `tests/` - Integration tests
- `examples/` - Example usage
- `benches/` - Benchmarks

## License

This project is licensed under the MIT OR Apache-2.0 license.
"#
        .to_string(),
    );

    files
}

fn python_template_files() -> HashMap<PathBuf, String> {
    let mut files = HashMap::new();

    files.insert(
        PathBuf::from("src/{{project_name}}/__init__.py"),
        r#"""{{project_name}} - An agentic development project

This project was generated by the Agentic Development Environment.
"""

__version__ = "0.1.0"
__author__ = "TODO: Your Name"
__email__ = "TODO: your.email@example.com"

from .main import main

__all__ = ["main"]
"#
        .to_string(),
    );

    files.insert(
        PathBuf::from("src/{{project_name}}/main.py"),
        r#"""Main module for {{project_name}}."""

import sys
import argparse
from typing import List, Optional


def hello_world() -> str:
    """Return a greeting message."""
    return "Hello from {{project_name}}!"


def add(a: int, b: int) -> int:
    """Add two integers."""
    return a + b


def main(argv: Optional[List[str]] = None) -> int:
    """Main entry point."""
    if argv is None:
        argv = sys.argv[1:]
    
    parser = argparse.ArgumentParser(
        description="{{project_name}} - An agentic development project"
    )
    parser.add_argument(
        "--version", 
        action="version", 
        version="{{project_name}} 0.1.0"
    )
    
    args = parser.parse_args(argv)
    
    print("🤖 Welcome to {{project_name}}!")
    print("This project was created with agentic-dev.")
    print(hello_world())
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
"#
        .to_string(),
    );

    files.insert(
        PathBuf::from("tests/test_main.py"),
        r#"""Tests for the main module."""

import pytest
from {{project_name}}.main import hello_world, add


def test_hello_world():
    """Test the hello_world function."""
    result = hello_world()
    assert "{{project_name}}" in result


def test_add():
    """Test the add function."""
    assert add(2, 2) == 4
    assert add(-1, 1) == 0
    assert add(0, 0) == 0


def test_add_type_error():
    """Test that add raises TypeError for invalid inputs."""
    with pytest.raises(TypeError):
        add("a", "b")  # type: ignore
"#
        .to_string(),
    );

    files.insert(
        PathBuf::from("README.md"),
        r#"# {{project_name}}

A Python project created with the Agentic Development Environment.

## Getting Started

### Prerequisites

- Python 3.8+
- pip or conda
- Agentic Development Environment

### Installation

```bash
# Create virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install the package
pip install -e .

# Install development dependencies
pip install -e ".[dev]"
```

### Running

```bash
python -m {{project_name}}
# or
{{project_name}}
```

### Testing

```bash
pytest
```

### Code Quality

```bash
# Format code
black src tests

# Lint code
flake8 src tests

# Type checking
mypy src
```

## Development with Agentic Environment

This project is set up to work with the Agentic Development Environment:

```bash
# Analyze the codebase
agentic-dev analyze

# Start interactive development mode
agentic-dev interactive

# Generate code with AI assistance
agentic-dev generate "create a function that processes user input"
```

## Project Structure

- `src/{{project_name}}/` - Main package
- `tests/` - Test files
- `docs/` - Documentation

## License

This project is licensed under the MIT license.
"#
        .to_string(),
    );

    files
}

fn typescript_template_files() -> HashMap<PathBuf, String> {
    let mut files = HashMap::new();

    files.insert(
        PathBuf::from("src/index.ts"),
        r#"/**
 * {{project_name}} - An agentic development project
 * 
 * This project was generated by the Agentic Development Environment.
 */

export function helloWorld(): string {
    return "Hello from {{project_name}}!";
}

export function add(a: number, b: number): number {
    return a + b;
}

export function main(): void {
    console.log("🤖 Welcome to {{project_name}}!");
    console.log("This project was created with agentic-dev.");
    console.log(helloWorld());
}

// Run main if this file is executed directly
if (require.main === module) {
    main();
}
"#
        .to_string(),
    );

    files.insert(
        PathBuf::from("tests/index.test.ts"),
        r#"import { helloWorld, add } from '../src/index';

describe('{{project_name}}', () => {
    test('helloWorld returns greeting', () => {
        const result = helloWorld();
        expect(result).toContain('{{project_name}}');
    });

    test('add function works correctly', () => {
        expect(add(2, 2)).toBe(4);
        expect(add(-1, 1)).toBe(0);
        expect(add(0, 0)).toBe(0);
    });
});
"#
        .to_string(),
    );

    files.insert(
        PathBuf::from("README.md"),
        r#"# {{project_name}}

A TypeScript project created with the Agentic Development Environment.

## Getting Started

### Prerequisites

- Node.js 16+
- npm or yarn
- Agentic Development Environment

### Installation

```bash
npm install
```

### Building

```bash
npm run build
```

### Running

```bash
node dist/index.js
```

### Testing

```bash
npm test
```

### Development

```bash
# Run in watch mode
npm run dev

# Lint code
npm run lint

# Format code
npm run format
```

## Development with Agentic Environment

This project is set up to work with the Agentic Development Environment:

```bash
# Analyze the codebase
agentic-dev analyze

# Start interactive development mode
agentic-dev interactive

# Generate code with AI assistance
agentic-dev generate "create a function that processes user input"
```

## Project Structure

- `src/` - Source TypeScript files
- `dist/` - Compiled JavaScript files
- `tests/` - Test files

## License

This project is licensed under the MIT license.
"#
        .to_string(),
    );

    files
}
