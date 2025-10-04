//! Comprehensive CLI input validation with helpful error messages and suggestions.
//!
//! This module provides validation for all CLI commands and arguments,
//! offering contextual error messages and intelligent suggestions.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use thiserror::Error;

use crate::cli::*;

/// Validation result with detailed error information
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub suggestions: Vec<String>,
}

/// Validation errors with context and suggestions
#[derive(Debug, Clone, Error)]
pub enum ValidationError {
    #[error("Invalid path: '{path}' - {reason}")]
    InvalidPath { path: String, reason: String },

    #[error("Invalid format: '{value}' for {field}. Expected one of: {expected:?}")]
    InvalidFormat {
        field: String,
        value: String,
        expected: Vec<String>,
    },

    #[error("Invalid range: {value} for {field}. Must be between {min} and {max}")]
    InvalidRange {
        field: String,
        value: String,
        min: String,
        max: String,
    },

    #[error("Missing required dependency: {dependency} for {operation}")]
    MissingDependency {
        dependency: String,
        operation: String,
    },

    #[error("Conflicting arguments: {conflicts:?}")]
    ConflictingArguments { conflicts: Vec<String> },

    #[error("Empty value: {field} cannot be empty")]
    EmptyValue { field: String },

    #[error("File not found: '{path}'")]
    FileNotFound { path: String },

    #[error("Directory not found: '{path}'")]
    DirectoryNotFound { path: String },

    #[error("Permission denied: cannot access '{path}'")]
    PermissionDenied { path: String },

    #[error("Invalid language: '{language}'. Supported languages: {supported:?}")]
    InvalidLanguage {
        language: String,
        supported: Vec<String>,
    },

    #[error("Invalid environment: '{env}'. Available environments: {available:?}")]
    InvalidEnvironment { env: String, available: Vec<String> },

    #[error("Custom validation error: {message}")]
    Custom { message: String },
}

/// Validation warnings (non-fatal issues)
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub message: String,
    pub field: Option<String>,
    pub suggestion: Option<String>,
}

/// Comprehensive CLI validator
pub struct CliValidator {
    supported_languages: HashSet<String>,
    supported_formats: HashMap<String, Vec<String>>,
    supported_analysis_depths: HashSet<String>,
    supported_analysis_types: HashSet<String>,
    supported_strategies: HashSet<String>,
    supported_agent_types: HashSet<String>,
    supported_environments: HashSet<String>,
    dependency_checker: DependencyChecker,
}

impl Default for CliValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl CliValidator {
    /// Create a new CLI validator with default configurations
    pub fn new() -> Self {
        let mut supported_languages = HashSet::new();
        supported_languages.extend(
            [
                "rust",
                "python",
                "javascript",
                "typescript",
                "go",
                "java",
                "c",
                "cpp",
                "csharp",
                "php",
                "ruby",
                "swift",
                "kotlin",
                "scala",
                "haskell",
                "erlang",
                "elixir",
                "clojure",
                "lua",
                "dart",
                "r",
                "matlab",
                "shell",
                "bash",
                "zsh",
                "fish",
                "powershell",
                "sql",
                "html",
                "css",
                "scss",
                "sass",
                "less",
                "vue",
                "react",
                "angular",
                "svelte",
                "docker",
                "yaml",
                "toml",
                "json",
                "xml",
                "markdown",
                "tex",
                "latex",
            ]
            .iter()
            .map(|s| s.to_string()),
        );

        let mut supported_formats = HashMap::new();
        supported_formats.insert(
            "general".to_string(),
            vec![
                "text".to_string(),
                "json".to_string(),
                "yaml".to_string(),
                "toml".to_string(),
            ],
        );
        supported_formats.insert(
            "export".to_string(),
            vec!["toml".to_string(), "json".to_string(), "yaml".to_string()],
        );

        let mut supported_analysis_depths = HashSet::new();
        supported_analysis_depths.extend(
            ["shallow", "normal", "deep", "complete"]
                .iter()
                .map(|s| s.to_string()),
        );

        let mut supported_analysis_types = HashSet::new();
        supported_analysis_types.extend(
            [
                "symbols",
                "dependencies",
                "architecture",
                "quality",
                "security",
                "performance",
                "documentation",
                "testing",
                "complexity",
                "patterns",
            ]
            .iter()
            .map(|s| s.to_string()),
        );

        let mut supported_strategies = HashSet::new();
        supported_strategies.extend(
            ["focused", "comprehensive", "iterative", "experimental"]
                .iter()
                .map(|s| s.to_string()),
        );

        let mut supported_agent_types = HashSet::new();
        supported_agent_types.extend(
            [
                "code-generation",
                "analysis",
                "refactoring",
                "review",
                "testing",
                "documentation",
                "optimization",
                "security",
                "custom",
            ]
            .iter()
            .map(|s| s.to_string()),
        );

        let mut supported_environments = HashSet::new();
        supported_environments.extend(
            [
                "development",
                "dev",
                "staging",
                "stage",
                "production",
                "prod",
                "test",
                "default",
            ]
            .iter()
            .map(|s| s.to_string()),
        );

        Self {
            supported_languages,
            supported_formats,
            supported_analysis_depths,
            supported_analysis_types,
            supported_strategies,
            supported_agent_types,
            supported_environments,
            dependency_checker: DependencyChecker::new(),
        }
    }

    /// Validate the entire CLI command structure
    pub fn validate_command(&mut self, cli: &Cli) -> ValidationResult {
        let mut result = ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
        };

        // Validate global arguments
        self.validate_global_args(cli, &mut result);

        // Validate specific commands
        match &cli.command {
            Commands::Init(args) => self.validate_init_args(args, &mut result),
            Commands::Interactive(args) => self.validate_interactive_args(args, &mut result),
            Commands::Analyze(args) => self.validate_analyze_args(args, &mut result),
            Commands::Generate(args) => self.validate_generate_args(args, &mut result),
            Commands::Agent(args) => self.validate_agent_args(args, &mut result),
            Commands::Config(args) => self.validate_config_args(args, &mut result),
            Commands::Inspect(args) => self.validate_inspect_args(args, &mut result),
            Commands::Profile(args) => self.validate_profile_args(args, &mut result),
            Commands::Template(args) => self.validate_template_args(args, &mut result),
            Commands::Status(args) => self.validate_status_args(args, &mut result),
            Commands::Shell(args) => self.validate_shell_args(args, &mut result),
            Commands::Demo(args) => self.validate_demo_args(args, &mut result),
            Commands::Blueprint(args) => self.validate_blueprint_args(args, &mut result),
        }

        result.is_valid = result.errors.is_empty();
        result
    }

    /// Validate global CLI arguments
    fn validate_global_args(&self, cli: &Cli, result: &mut ValidationResult) {
        // Validate config path if provided
        if let Some(config_path) = &cli.config {
            self.validate_file_path(config_path, "config", result);
        }

        // Validate directory path if provided
        if let Some(directory) = &cli.directory {
            self.validate_directory_path(directory, "working directory", result);
        }

        // Validate format
        if let Some(formats) = self.supported_formats.get("general") {
            let format_str = match cli.format {
                OutputFormat::Text => "text",
                OutputFormat::Json => "json",
                OutputFormat::Yaml => "yaml",
                OutputFormat::Table => "table",
            };

            if !formats.contains(&format_str.to_string()) {
                result.errors.push(ValidationError::InvalidFormat {
                    field: "format".to_string(),
                    value: format_str.to_string(),
                    expected: formats.clone(),
                });
            }
        }

        // Check for conflicting flags
        if cli.verbose && cli.quiet {
            result.errors.push(ValidationError::ConflictingArguments {
                conflicts: vec!["verbose".to_string(), "quiet".to_string()],
            });
        }
    }

    /// Validate initialization arguments
    fn validate_init_args(&mut self, args: &InitArgs, result: &mut ValidationResult) {
        // Validate project name
        if args.name.trim().is_empty() {
            result.errors.push(ValidationError::EmptyValue {
                field: "project name".to_string(),
            });
        } else if !self.is_valid_project_name(&args.name) {
            result.errors.push(ValidationError::Custom {
                message: format!(
                    "Invalid project name '{}'. Use only letters, numbers, hyphens, and underscores",
                    args.name
                ),
            });
            result
                .suggestions
                .push("Try using a name like 'my-project' or 'my_project'".to_string());
        }

        // Validate language
        if let Some(language) = &args.language {
            if !self.supported_languages.contains(language) {
                result.errors.push(ValidationError::InvalidLanguage {
                    language: language.clone(),
                    supported: self.get_language_suggestions(language),
                });
            }
        }

        // Check if directory already exists (warn if not using --force)
        let project_path = Path::new(&args.name);
        if project_path.exists() && !args.force {
            result.warnings.push(ValidationWarning {
                message: format!("Directory '{}' already exists", args.name),
                field: Some("name".to_string()),
                suggestion: Some("Use --force to overwrite or choose a different name".to_string()),
            });
        }

        // Check for git dependency if git initialization is requested
        if args.git && !self.dependency_checker.has_git() {
            result.warnings.push(ValidationWarning {
                message: "Git is not available but git initialization is requested".to_string(),
                field: Some("git".to_string()),
                suggestion: Some("Install git or use --no-git flag".to_string()),
            });
        }
    }

    /// Validate interactive mode arguments
    fn validate_interactive_args(&self, args: &InteractiveArgs, result: &mut ValidationResult) {
        if let Some(view) = &args.view {
            let valid_views = ["agents", "context", "config", "logs", "status"];
            if !valid_views.contains(&view.as_str()) {
                result.errors.push(ValidationError::InvalidFormat {
                    field: "view".to_string(),
                    value: view.clone(),
                    expected: valid_views.iter().map(|s| s.to_string()).collect(),
                });
            }
        }

        // Check for conflicting flags
        if args.monitor && args.auto_start {
            result.warnings.push(ValidationWarning {
                message: "Monitor mode conflicts with auto-start agents".to_string(),
                field: None,
                suggestion: Some("Use either --monitor or --auto-start, not both".to_string()),
            });
        }
    }

    /// Validate analysis arguments
    fn validate_analyze_args(&self, args: &AnalyzeArgs, result: &mut ValidationResult) {
        // Validate target paths
        if args.targets.is_empty() {
            result.warnings.push(ValidationWarning {
                message: "No targets specified, will analyze current directory".to_string(),
                field: Some("targets".to_string()),
                suggestion: Some("Specify files or directories to analyze".to_string()),
            });
        } else {
            for target in &args.targets {
                if !target.exists() {
                    result.errors.push(ValidationError::FileNotFound {
                        path: target.to_string_lossy().to_string(),
                    });
                }
            }
        }

        // Validate depth
        if !self.supported_analysis_depths.contains(&args.depth) {
            result.errors.push(ValidationError::InvalidFormat {
                field: "depth".to_string(),
                value: args.depth.clone(),
                expected: self.supported_analysis_depths.iter().cloned().collect(),
            });
        }

        // Validate analysis types
        for analysis_type in &args.analysis_types {
            if !self.supported_analysis_types.contains(analysis_type) {
                result.errors.push(ValidationError::InvalidFormat {
                    field: "analysis-types".to_string(),
                    value: analysis_type.clone(),
                    expected: self.get_analysis_type_suggestions(analysis_type),
                });
            }
        }

        // Validate export path
        if let Some(export_path) = &args.export {
            if let Some(parent) = export_path.parent() {
                if !parent.exists() {
                    result.errors.push(ValidationError::DirectoryNotFound {
                        path: parent.to_string_lossy().to_string(),
                    });
                }
            }
        }
    }

    /// Validate generation arguments
    fn validate_generate_args(&self, args: &GenerateArgs, result: &mut ValidationResult) {
        // Validate prompt
        if args.prompt.trim().is_empty() {
            result.errors.push(ValidationError::EmptyValue {
                field: "prompt".to_string(),
            });
        } else if args.prompt.len() < 10 {
            result.warnings.push(ValidationWarning {
                message: "Very short prompt may lead to poor results".to_string(),
                field: Some("prompt".to_string()),
                suggestion: Some(
                    "Provide a more detailed description of what you want to generate".to_string(),
                ),
            });
        }

        // Validate language
        if let Some(language) = &args.language {
            if !self.supported_languages.contains(language) {
                result.errors.push(ValidationError::InvalidLanguage {
                    language: language.clone(),
                    supported: self.get_language_suggestions(language),
                });
            }
        }

        // Validate strategy
        if !self.supported_strategies.contains(&args.strategy) {
            result.errors.push(ValidationError::InvalidFormat {
                field: "strategy".to_string(),
                value: args.strategy.clone(),
                expected: self.supported_strategies.iter().cloned().collect(),
            });
        }

        // Validate context paths
        for context_path in &args.context {
            if !context_path.exists() {
                result.errors.push(ValidationError::FileNotFound {
                    path: context_path.to_string_lossy().to_string(),
                });
            }
        }

        // Validate numeric parameters
        if let Some(max_tokens) = args.max_tokens {
            if max_tokens == 0 || max_tokens > 100000 {
                result.errors.push(ValidationError::InvalidRange {
                    field: "max-tokens".to_string(),
                    value: max_tokens.to_string(),
                    min: "1".to_string(),
                    max: "100000".to_string(),
                });
            }
        }

        if let Some(temperature) = args.temperature {
            if !(0.0..=2.0).contains(&temperature) {
                result.errors.push(ValidationError::InvalidRange {
                    field: "temperature".to_string(),
                    value: temperature.to_string(),
                    min: "0.0".to_string(),
                    max: "2.0".to_string(),
                });
            }
        }

        // Validate output path
        if let Some(output_path) = &args.output {
            if let Some(parent) = output_path.parent() {
                if !parent.exists() {
                    result.errors.push(ValidationError::DirectoryNotFound {
                        path: parent.to_string_lossy().to_string(),
                    });
                }
            }
        }
    }

    /// Validate agent arguments
    fn validate_agent_args(&self, args: &AgentArgs, result: &mut ValidationResult) {
        match &args.command {
            AgentCommands::Create {
                name,
                agent_type,
                config,
            } => {
                if name.trim().is_empty() {
                    result.errors.push(ValidationError::EmptyValue {
                        field: "agent name".to_string(),
                    });
                }

                if !self.supported_agent_types.contains(agent_type) {
                    result.errors.push(ValidationError::InvalidFormat {
                        field: "agent-type".to_string(),
                        value: agent_type.clone(),
                        expected: self.supported_agent_types.iter().cloned().collect(),
                    });
                }

                if let Some(config_path) = config {
                    self.validate_file_path(config_path, "config", result);
                }
            }
            AgentCommands::Remove { name } => {
                if name.trim().is_empty() {
                    result.errors.push(ValidationError::EmptyValue {
                        field: "agent name".to_string(),
                    });
                }
            }
            AgentCommands::Logs { agent, lines, .. } => {
                if agent.trim().is_empty() {
                    result.errors.push(ValidationError::EmptyValue {
                        field: "agent".to_string(),
                    });
                }

                if *lines == 0 || *lines > 10000 {
                    result.errors.push(ValidationError::InvalidRange {
                        field: "lines".to_string(),
                        value: lines.to_string(),
                        min: "1".to_string(),
                        max: "10000".to_string(),
                    });
                }
            }
            AgentCommands::Start { agents, all } | AgentCommands::Stop { agents, all } => {
                if !all && agents.is_empty() {
                    result.errors.push(ValidationError::Custom {
                        message: "Either specify agent names or use --all flag".to_string(),
                    });
                }
            }
            _ => {} // Other commands have simpler validation
        }
    }

    /// Validate configuration arguments
    fn validate_config_args(&self, args: &ConfigArgs, result: &mut ValidationResult) {
        match &args.command {
            ConfigCommands::Set { path, value } => {
                if path.trim().is_empty() {
                    result.errors.push(ValidationError::EmptyValue {
                        field: "config path".to_string(),
                    });
                }
                if value.trim().is_empty() {
                    result.errors.push(ValidationError::EmptyValue {
                        field: "value".to_string(),
                    });
                }
            }
            ConfigCommands::Get { path } => {
                if path.trim().is_empty() {
                    result.errors.push(ValidationError::EmptyValue {
                        field: "config path".to_string(),
                    });
                }
            }
            ConfigCommands::Environment { env } => {
                if !self.supported_environments.contains(env) {
                    result.errors.push(ValidationError::InvalidEnvironment {
                        env: env.clone(),
                        available: self.supported_environments.iter().cloned().collect(),
                    });
                }
            }
            ConfigCommands::Export { output, format } => {
                if let Some(formats) = self.supported_formats.get("export") {
                    if !formats.contains(format) {
                        result.errors.push(ValidationError::InvalidFormat {
                            field: "format".to_string(),
                            value: format.clone(),
                            expected: formats.clone(),
                        });
                    }
                }

                if let Some(parent) = output.parent() {
                    if !parent.exists() {
                        result.errors.push(ValidationError::DirectoryNotFound {
                            path: parent.to_string_lossy().to_string(),
                        });
                    }
                }
            }
            ConfigCommands::Import { input, .. } => {
                self.validate_file_path(input, "import file", result);
            }
            _ => {} // Other commands have minimal validation needs
        }
    }

    /// Validate inspection arguments
    fn validate_inspect_args(&self, args: &InspectArgs, result: &mut ValidationResult) {
        match &args.command {
            InspectCommands::File { path, .. } => {
                self.validate_file_path(path, "file", result);
            }
            InspectCommands::Dependencies { targets, .. } => {
                if targets.is_empty() {
                    result.warnings.push(ValidationWarning {
                        message: "No targets specified, will analyze current directory".to_string(),
                        field: Some("targets".to_string()),
                        suggestion: None,
                    });
                } else {
                    for target in targets {
                        if !target.exists() {
                            result.errors.push(ValidationError::FileNotFound {
                                path: target.to_string_lossy().to_string(),
                            });
                        }
                    }
                }
            }
            InspectCommands::Relationships { target, depth, .. } => {
                if target.trim().is_empty() {
                    result.errors.push(ValidationError::EmptyValue {
                        field: "target".to_string(),
                    });
                }

                if *depth == 0 || *depth > 10 {
                    result.errors.push(ValidationError::InvalidRange {
                        field: "depth".to_string(),
                        value: depth.to_string(),
                        min: "1".to_string(),
                        max: "10".to_string(),
                    });
                }
            }
            _ => {} // Other commands have minimal validation
        }
    }

    // Placeholder validation methods for remaining command types
    fn validate_profile_args(&self, _args: &ProfileArgs, _result: &mut ValidationResult) {}
    fn validate_template_args(&self, _args: &TemplateArgs, _result: &mut ValidationResult) {}
    fn validate_status_args(&self, _args: &StatusArgs, _result: &mut ValidationResult) {}
    fn validate_shell_args(&self, _args: &ShellArgs, _result: &mut ValidationResult) {}
    fn validate_demo_args(&self, _args: &DemoArgs, _result: &mut ValidationResult) {}

    /// Validate blueprint arguments
    fn validate_blueprint_args(&self, args: &BlueprintArgs, result: &mut ValidationResult) {
        use crate::cli::BlueprintCommands;

        match &args.command {
            BlueprintCommands::Extract { source, output, .. } => {
                self.validate_directory_path(source, "source", result);

                if let Some(parent) = output.parent() {
                    if !parent.exists() {
                        result.errors.push(ValidationError::DirectoryNotFound {
                            path: parent.to_string_lossy().to_string(),
                        });
                    }
                }
            }
            BlueprintCommands::Generate {
                blueprint, output, ..
            } => {
                self.validate_file_path(blueprint, "blueprint", result);

                if let Some(parent) = output.parent() {
                    if !parent.exists() {
                        result.errors.push(ValidationError::DirectoryNotFound {
                            path: parent.to_string_lossy().to_string(),
                        });
                    }
                }
            }
            BlueprintCommands::Replicate { target, .. } => {
                if let Some(parent) = target.parent() {
                    if !parent.exists() {
                        result.errors.push(ValidationError::DirectoryNotFound {
                            path: parent.to_string_lossy().to_string(),
                        });
                    }
                }
            }
            BlueprintCommands::Validate { blueprint } => {
                self.validate_file_path(blueprint, "blueprint", result);
            }
            BlueprintCommands::Info { blueprint, .. } => {
                self.validate_file_path(blueprint, "blueprint", result);
            }
            BlueprintCommands::Compare {
                blueprint1,
                blueprint2,
            } => {
                self.validate_file_path(blueprint1, "first blueprint", result);
                self.validate_file_path(blueprint2, "second blueprint", result);
            }
            BlueprintCommands::Evolution(_) => {
                // Evolution command has its own validation logic within the command handler
                // For now, we don't add additional validation here
            }
        }
    }

    // Helper validation methods

    /// Validate file path exists and is accessible
    fn validate_file_path(&self, path: &Path, field_name: &str, result: &mut ValidationResult) {
        if !path.exists() {
            result.errors.push(ValidationError::FileNotFound {
                path: path.to_string_lossy().to_string(),
            });
        } else if !path.is_file() {
            result.errors.push(ValidationError::Custom {
                message: format!(
                    "{} must be a file, got directory: {}",
                    field_name,
                    path.display()
                ),
            });
        } else if let Err(_) = std::fs::metadata(path) {
            result.errors.push(ValidationError::PermissionDenied {
                path: path.to_string_lossy().to_string(),
            });
        }
    }

    /// Validate directory path exists and is accessible
    fn validate_directory_path(
        &self,
        path: &Path,
        field_name: &str,
        result: &mut ValidationResult,
    ) {
        if !path.exists() {
            result.errors.push(ValidationError::DirectoryNotFound {
                path: path.to_string_lossy().to_string(),
            });
        } else if !path.is_dir() {
            result.errors.push(ValidationError::Custom {
                message: format!(
                    "{} must be a directory, got file: {}",
                    field_name,
                    path.display()
                ),
            });
        } else if let Err(_) = std::fs::read_dir(path) {
            result.errors.push(ValidationError::PermissionDenied {
                path: path.to_string_lossy().to_string(),
            });
        }
    }

    /// Check if project name is valid
    fn is_valid_project_name(&self, name: &str) -> bool {
        !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            && !name.starts_with('-')
            && !name.ends_with('-')
    }

    /// Get language suggestions based on input
    fn get_language_suggestions(&self, input: &str) -> Vec<String> {
        let input_lower = input.to_lowercase();

        // Find exact matches first
        if self.supported_languages.contains(&input_lower) {
            return vec![input_lower];
        }

        let mut suggestions = Vec::new();

        // Add common mappings first (highest priority)
        match input_lower.as_str() {
            "js" => suggestions.push("javascript".to_string()),
            "ts" => suggestions.push("typescript".to_string()),
            "py" => suggestions.push("python".to_string()),
            "rs" => suggestions.push("rust".to_string()),
            "cpp" | "c++" => suggestions.push("cpp".to_string()),
            "c#" => suggestions.push("csharp".to_string()),
            "sh" => suggestions.extend(["shell", "bash"].iter().map(|s| s.to_string())),
            _ => {}
        }

        // Find close matches using simple similarity
        let similarity_matches: Vec<_> = self
            .supported_languages
            .iter()
            .filter(|lang| {
                !suggestions.contains(lang) && // Don't duplicate mappings
                (lang.contains(&input_lower)
                    || input_lower.contains(lang.as_str())
                    || self.levenshtein_distance(lang, &input_lower) <= 2)
            })
            .cloned()
            .collect();

        suggestions.extend(similarity_matches);
        suggestions.into_iter().take(5).collect()
    }

    /// Get analysis type suggestions
    fn get_analysis_type_suggestions(&self, input: &str) -> Vec<String> {
        let input_lower = input.to_lowercase();

        self.supported_analysis_types
            .iter()
            .filter(|analysis_type| {
                analysis_type.contains(&input_lower)
                    || input_lower.contains(analysis_type.as_str())
                    || self.levenshtein_distance(analysis_type, &input_lower) <= 2
            })
            .take(5)
            .cloned()
            .collect()
    }

    /// Simple Levenshtein distance calculation for suggestions
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();

        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                    0
                } else {
                    1
                };
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                    matrix[i - 1][j - 1] + cost,
                );
            }
        }

        matrix[len1][len2]
    }

    /// Format validation results for display
    pub fn format_validation_results(&self, result: &ValidationResult) -> String {
        let mut output = String::new();

        if !result.errors.is_empty() {
            output.push_str("❌ Validation Errors:\n");
            for error in &result.errors {
                output.push_str(&format!("  • {}\n", error));
            }
            output.push('\n');
        }

        if !result.warnings.is_empty() {
            output.push_str("⚠️  Warnings:\n");
            for warning in &result.warnings {
                output.push_str(&format!("  • {}", warning.message));
                if let Some(suggestion) = &warning.suggestion {
                    output.push_str(&format!(" ({})", suggestion));
                }
                output.push('\n');
            }
            output.push('\n');
        }

        if !result.suggestions.is_empty() {
            output.push_str("💡 Suggestions:\n");
            for suggestion in &result.suggestions {
                output.push_str(&format!("  • {}\n", suggestion));
            }
        }

        if output.is_empty() {
            output.push_str("✅ All validations passed!");
        }

        output
    }
}

/// Dependency checker for external tools
struct DependencyChecker {
    checked_dependencies: HashMap<String, bool>,
}

impl DependencyChecker {
    fn new() -> Self {
        Self {
            checked_dependencies: HashMap::new(),
        }
    }

    /// Check if git is available
    fn has_git(&mut self) -> bool {
        self.check_dependency("git")
    }

    /// Check if a command is available in PATH
    fn check_dependency(&mut self, command: &str) -> bool {
        if let Some(&available) = self.checked_dependencies.get(command) {
            return available;
        }

        let available = std::process::Command::new("which")
            .arg(command)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

        self.checked_dependencies
            .insert(command.to_string(), available);
        available
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_name_validation() {
        let validator = CliValidator::new();

        assert!(validator.is_valid_project_name("my-project"));
        assert!(validator.is_valid_project_name("my_project"));
        assert!(validator.is_valid_project_name("project123"));

        assert!(!validator.is_valid_project_name(""));
        assert!(!validator.is_valid_project_name("-project"));
        assert!(!validator.is_valid_project_name("project-"));
        assert!(!validator.is_valid_project_name("my project"));
    }

    #[test]
    fn test_language_suggestions() {
        let validator = CliValidator::new();

        let suggestions = validator.get_language_suggestions("js");
        assert!(suggestions.contains(&"javascript".to_string()));

        let suggestions = validator.get_language_suggestions("py");
        assert!(suggestions.contains(&"python".to_string()));

        let suggestions = validator.get_language_suggestions("rust");
        assert!(suggestions.contains(&"rust".to_string()));
    }

    #[test]
    fn test_levenshtein_distance() {
        let validator = CliValidator::new();

        assert_eq!(validator.levenshtein_distance("rust", "rust"), 0);
        assert_eq!(validator.levenshtein_distance("rust", "rast"), 1);
        assert_eq!(validator.levenshtein_distance("python", "pyton"), 1);
        assert_eq!(validator.levenshtein_distance("javascript", "java"), 6);
    }
}
