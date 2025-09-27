//! Comprehensive help system for the CLI with detailed descriptions and examples.
//!
//! This module provides rich help content, including command examples, common use cases,
//! and interactive help features.

use crossterm::style::Stylize;
use std::collections::HashMap;

/// Help content structure with examples and related commands
#[derive(Debug, Clone)]
pub struct HelpContent {
    pub description: String,
    pub long_description: Option<String>,
    pub examples: Vec<HelpExample>,
    pub related_commands: Vec<String>,
    pub common_options: Vec<CommonOption>,
    pub troubleshooting: Vec<TroubleshootingTip>,
}

/// Example with description and command
#[derive(Debug, Clone)]
pub struct HelpExample {
    pub description: String,
    pub command: String,
    pub expected_output: Option<String>,
}

/// Common option descriptions
#[derive(Debug, Clone)]
pub struct CommonOption {
    pub flag: String,
    pub description: String,
    pub example: Option<String>,
}

/// Troubleshooting tips
#[derive(Debug, Clone)]
pub struct TroubleshootingTip {
    pub problem: String,
    pub solution: String,
    pub additional_info: Option<String>,
}

/// Comprehensive help system
pub struct HelpSystem {
    command_help: HashMap<String, HelpContent>,
    global_options: Vec<CommonOption>,
    getting_started_guide: Vec<String>,
}

impl Default for HelpSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl HelpSystem {
    /// Create a new help system with all content
    pub fn new() -> Self {
        let mut help_system = Self {
            command_help: HashMap::new(),
            global_options: Vec::new(),
            getting_started_guide: Vec::new(),
        };

        help_system.populate_help_content();
        help_system
    }

    /// Get help content for a specific command
    pub fn get_command_help(&self, command: &str) -> Option<&HelpContent> {
        self.command_help.get(command)
    }

    /// Get formatted help for a command
    pub fn format_command_help(&self, command: &str) -> String {
        if let Some(help_content) = self.get_command_help(command) {
            self.format_help_content(command, help_content)
        } else {
            format!("No help available for command '{}'", command)
        }
    }

    /// Get list of all available commands
    pub fn get_available_commands(&self) -> Vec<String> {
        let mut commands: Vec<_> = self.command_help.keys().cloned().collect();
        commands.sort();
        commands
    }

    /// Get getting started guide
    pub fn get_getting_started_guide(&self) -> String {
        let mut guide = String::new();
        guide.push_str(
            &"🚀 Getting Started with Agentic Development Environment\n\n"
                .bold()
                .to_string(),
        );

        for (i, step) in self.getting_started_guide.iter().enumerate() {
            guide.push_str(&format!("{}. {}\n\n", i + 1, step));
        }

        guide.push_str("For more detailed help on any command, use:\n");
        guide.push_str(&"  agentic-dev <command> --help\n\n".italic().to_string());
        guide.push_str("Visit our documentation at: https://agentic-dev.dev/docs\n");

        guide
    }

    /// Search help content by keyword
    pub fn search_help(&self, keyword: &str) -> Vec<String> {
        let keyword_lower = keyword.to_lowercase();
        let mut results = Vec::new();

        for (command, help_content) in &self.command_help {
            // Search in descriptions
            if help_content
                .description
                .to_lowercase()
                .contains(&keyword_lower)
            {
                results.push(format!("{} - {}", command, help_content.description));
            }

            // Search in examples
            for example in &help_content.examples {
                if example.description.to_lowercase().contains(&keyword_lower)
                    || example.command.to_lowercase().contains(&keyword_lower)
                {
                    results.push(format!("{} (example) - {}", command, example.description));
                }
            }
        }

        results.sort();
        results
    }

    /// Get interactive help prompts
    pub fn get_interactive_prompts(&self) -> Vec<String> {
        vec![
            "What would you like to do?".to_string(),
            "• Initialize a new project (init)".to_string(),
            "• Analyze existing code (analyze)".to_string(),
            "• Generate new code (generate)".to_string(),
            "• Start interactive mode (interactive)".to_string(),
            "• Manage agents (agent)".to_string(),
            "• Configure settings (config)".to_string(),
            "\nType a command name for detailed help, or 'exit' to quit.".to_string(),
        ]
    }

    /// Format help content with colors and styling
    fn format_help_content(&self, command: &str, content: &HelpContent) -> String {
        let mut help = String::new();

        // Title
        help.push_str(&format!("{}\n", command.to_uppercase().bold()));
        help.push_str(&format!("{}\n\n", content.description.clone().italic()));

        // Long description if available
        if let Some(long_desc) = &content.long_description {
            help.push_str(&format!("{}\n\n", long_desc));
        }

        // Examples
        if !content.examples.is_empty() {
            help.push_str(&"EXAMPLES\n".bold().to_string());
            for (i, example) in content.examples.iter().enumerate() {
                help.push_str(&format!(
                    "  {}. {}\n",
                    i + 1,
                    example.description.clone().italic()
                ));
                help.push_str(&format!("     {}\n", example.command.clone().dark_green()));

                if let Some(output) = &example.expected_output {
                    help.push_str(&format!("     → {}\n", output.clone().dark_grey()));
                }
                help.push('\n');
            }
        }

        // Common options
        if !content.common_options.is_empty() {
            help.push_str(&"COMMON OPTIONS\n".bold().to_string());
            for option in &content.common_options {
                help.push_str(&format!(
                    "  {} - {}\n",
                    option.flag.clone().yellow(),
                    option.description
                ));
                if let Some(example) = &option.example {
                    help.push_str(&format!("    Example: {}\n", example.clone().dark_green()));
                }
            }
            help.push('\n');
        }

        // Related commands
        if !content.related_commands.is_empty() {
            help.push_str(&"RELATED COMMANDS\n".bold().to_string());
            for related in &content.related_commands {
                help.push_str(&format!("  {}\n", related.clone().cyan()));
            }
            help.push('\n');
        }

        // Troubleshooting
        if !content.troubleshooting.is_empty() {
            help.push_str(&"TROUBLESHOOTING\n".bold().to_string());
            for tip in &content.troubleshooting {
                help.push_str(&format!("  Problem: {}\n", tip.problem.clone().red()));
                help.push_str(&format!("  Solution: {}\n", tip.solution.clone().green()));
                if let Some(info) = &tip.additional_info {
                    help.push_str(&format!("  Info: {}\n", info.clone().dark_grey()));
                }
                help.push('\n');
            }
        }

        help
    }

    /// Populate all help content
    fn populate_help_content(&mut self) {
        self.populate_global_options();
        self.populate_getting_started_guide();
        self.populate_command_help();
    }

    /// Populate global options
    fn populate_global_options(&mut self) {
        self.global_options = vec![
            CommonOption {
                flag: "-v, --verbose".to_string(),
                description: "Enable verbose output".to_string(),
                example: Some("agentic-dev --verbose analyze .".to_string()),
            },
            CommonOption {
                flag: "-q, --quiet".to_string(),
                description: "Suppress non-essential output".to_string(),
                example: Some("agentic-dev --quiet generate 'create function'".to_string()),
            },
            CommonOption {
                flag: "-c, --config <PATH>".to_string(),
                description: "Specify custom configuration file".to_string(),
                example: Some("agentic-dev --config custom.toml analyze".to_string()),
            },
            CommonOption {
                flag: "--format <FORMAT>".to_string(),
                description: "Output format (text, json, yaml)".to_string(),
                example: Some("agentic-dev --format json status".to_string()),
            },
        ];
    }

    /// Populate getting started guide
    fn populate_getting_started_guide(&mut self) {
        self.getting_started_guide = vec![
            "Install the agentic development environment and ensure prerequisites are met".to_string(),
            "Initialize a new project: `agentic-dev init my-project --language rust`".to_string(),
            "Navigate to your project and analyze the codebase: `agentic-dev analyze .`".to_string(),
            "Start the interactive mode for guided development: `agentic-dev interactive`".to_string(),
            "Generate code with natural language: `agentic-dev generate 'create a REST API handler'`".to_string(),
            "Configure your preferences: `agentic-dev config show` and `agentic-dev config set`".to_string(),
            "Explore advanced features like custom agents and templates".to_string(),
        ];
    }

    /// Populate command-specific help content
    fn populate_command_help(&mut self) {
        // Init command
        self.command_help.insert("init".to_string(), HelpContent {
            description: "Initialize a new agentic development project".to_string(),
            long_description: Some(
                "Creates a new project directory with proper structure, configuration files, \
                 and language-specific templates. Optionally initializes git repository and \
                 sets up development environment.".to_string()
            ),
            examples: vec![
                HelpExample {
                    description: "Create a new Rust project".to_string(),
                    command: "agentic-dev init my-rust-app --language rust".to_string(),
                    expected_output: Some("✅ Project 'my-rust-app' created successfully".to_string()),
                },
                HelpExample {
                    description: "Create Python project with specific template".to_string(),
                    command: "agentic-dev init ml-project --language python --template machine-learning".to_string(),
                    expected_output: None,
                },
                HelpExample {
                    description: "Create project without git initialization".to_string(),
                    command: "agentic-dev init simple-project --no-git".to_string(),
                    expected_output: None,
                },
            ],
            common_options: vec![
                CommonOption {
                    flag: "--language, -l".to_string(),
                    description: "Programming language for the project".to_string(),
                    example: Some("--language rust".to_string()),
                },
                CommonOption {
                    flag: "--template, -t".to_string(),
                    description: "Project template to use".to_string(),
                    example: Some("--template web-service".to_string()),
                },
                CommonOption {
                    flag: "--force".to_string(),
                    description: "Overwrite existing directory".to_string(),
                    example: None,
                },
            ],
            related_commands: vec![
                "template list".to_string(),
                "config show".to_string(),
                "status".to_string(),
            ],
            troubleshooting: vec![
                TroubleshootingTip {
                    problem: "Directory already exists error".to_string(),
                    solution: "Use --force flag to overwrite or choose a different name".to_string(),
                    additional_info: None,
                },
                TroubleshootingTip {
                    problem: "Git initialization fails".to_string(),
                    solution: "Install git or use --no-git flag".to_string(),
                    additional_info: Some("Check 'git --version' to verify installation".to_string()),
                },
            ],
        });

        // Generate command
        self.command_help.insert("generate".to_string(), HelpContent {
            description: "Generate code using AI agents with natural language prompts".to_string(),
            long_description: Some(
                "Uses advanced AI agents to generate code based on natural language descriptions. \
                 Considers existing codebase context and follows project conventions. Supports \
                 multiple programming languages and generation strategies.".to_string()
            ),
            examples: vec![
                HelpExample {
                    description: "Generate a simple function".to_string(),
                    command: "agentic-dev generate 'create a function to calculate fibonacci numbers'".to_string(),
                    expected_output: Some("Generated fibonacci.rs with recursive implementation".to_string()),
                },
                HelpExample {
                    description: "Generate with specific context files".to_string(),
                    command: "agentic-dev generate 'add error handling' --context src/main.rs --context src/lib.rs".to_string(),
                    expected_output: None,
                },
                HelpExample {
                    description: "Preview without writing files".to_string(),
                    command: "agentic-dev generate 'create REST API endpoints' --preview".to_string(),
                    expected_output: None,
                },
            ],
            common_options: vec![
                CommonOption {
                    flag: "--language, -l".to_string(),
                    description: "Target programming language".to_string(),
                    example: Some("--language typescript".to_string()),
                },
                CommonOption {
                    flag: "--context".to_string(),
                    description: "Include specific files as context".to_string(),
                    example: Some("--context src/models.rs".to_string()),
                },
                CommonOption {
                    flag: "--strategy".to_string(),
                    description: "Generation strategy (focused, comprehensive, iterative)".to_string(),
                    example: Some("--strategy comprehensive".to_string()),
                },
            ],
            related_commands: vec![
                "analyze".to_string(),
                "agent status".to_string(),
                "config show".to_string(),
            ],
            troubleshooting: vec![
                TroubleshootingTip {
                    problem: "Generated code doesn't compile".to_string(),
                    solution: "Use --strategy iterative for better results or provide more context".to_string(),
                    additional_info: Some("Try running 'analyze' first to improve context understanding".to_string()),
                },
            ],
        });

        // Analyze command
        self.command_help.insert("analyze".to_string(), HelpContent {
            description: "Analyze codebase structure, dependencies, and patterns".to_string(),
            long_description: Some(
                "Performs comprehensive codebase analysis including symbol extraction, dependency \
                 mapping, architectural pattern detection, and code quality assessment. Results \
                 are cached for faster subsequent operations.".to_string()
            ),
            examples: vec![
                HelpExample {
                    description: "Analyze current directory".to_string(),
                    command: "agentic-dev analyze".to_string(),
                    expected_output: Some("Found 25 files, 150 symbols, 12 dependencies".to_string()),
                },
                HelpExample {
                    description: "Deep analysis with specific types".to_string(),
                    command: "agentic-dev analyze --depth deep --analysis-types symbols,dependencies,quality".to_string(),
                    expected_output: None,
                },
                HelpExample {
                    description: "Export analysis results".to_string(),
                    command: "agentic-dev analyze --export analysis.json --format json".to_string(),
                    expected_output: None,
                },
            ],
            common_options: vec![
                CommonOption {
                    flag: "--depth".to_string(),
                    description: "Analysis depth (shallow, normal, deep)".to_string(),
                    example: Some("--depth deep".to_string()),
                },
                CommonOption {
                    flag: "--analysis-types".to_string(),
                    description: "Specific analysis types to run".to_string(),
                    example: Some("--analysis-types symbols,dependencies".to_string()),
                },
                CommonOption {
                    flag: "--export, -e".to_string(),
                    description: "Export results to file".to_string(),
                    example: Some("--export results.yaml".to_string()),
                },
            ],
            related_commands: vec![
                "inspect symbols".to_string(),
                "inspect dependencies".to_string(),
                "generate".to_string(),
            ],
            troubleshooting: vec![
                TroubleshootingTip {
                    problem: "Analysis is very slow".to_string(),
                    solution: "Use --depth shallow for faster results or exclude test files".to_string(),
                    additional_info: None,
                },
            ],
        });

        // Interactive command
        self.command_help.insert("interactive".to_string(), HelpContent {
            description: "Start interactive development mode with real-time agent monitoring".to_string(),
            long_description: Some(
                "Launches a rich terminal interface for interactive development. Monitor agent \
                 activities, view real-time logs, execute commands, and get contextual assistance. \
                 Ideal for exploratory development and debugging.".to_string()
            ),
            examples: vec![
                HelpExample {
                    description: "Start interactive mode".to_string(),
                    command: "agentic-dev interactive".to_string(),
                    expected_output: Some("Launching interactive mode...".to_string()),
                },
                HelpExample {
                    description: "Start with specific view".to_string(),
                    command: "agentic-dev interactive --view agents".to_string(),
                    expected_output: None,
                },
                HelpExample {
                    description: "Monitor mode (read-only)".to_string(),
                    command: "agentic-dev interactive --monitor".to_string(),
                    expected_output: None,
                },
            ],
            common_options: vec![
                CommonOption {
                    flag: "--view, -V".to_string(),
                    description: "Starting view (agents, context, config, logs)".to_string(),
                    example: Some("--view logs".to_string()),
                },
                CommonOption {
                    flag: "--monitor, -m".to_string(),
                    description: "Read-only monitoring mode".to_string(),
                    example: None,
                },
            ],
            related_commands: vec![
                "agent status".to_string(),
                "status".to_string(),
                "config show".to_string(),
            ],
            troubleshooting: vec![
                TroubleshootingTip {
                    problem: "Interface appears corrupted".to_string(),
                    solution: "Ensure terminal supports colors and try resizing".to_string(),
                    additional_info: Some("Some terminals may not fully support TUI features".to_string()),
                },
            ],
        });

        // Agent command
        self.command_help.insert("agent".to_string(), HelpContent {
            description: "Manage AI agents for code generation, analysis, and automation".to_string(),
            long_description: Some(
                "Control the lifecycle and configuration of AI agents. Agents can be specialized \
                 for different tasks like code generation, analysis, refactoring, and review. \
                 Monitor their status, view logs, and create custom agents.".to_string()
            ),
            examples: vec![
                HelpExample {
                    description: "List all available agents".to_string(),
                    command: "agentic-dev agent list".to_string(),
                    expected_output: Some("code-gen: running, analysis: idle, review: stopped".to_string()),
                },
                HelpExample {
                    description: "Start specific agents".to_string(),
                    command: "agentic-dev agent start code-gen analysis".to_string(),
                    expected_output: None,
                },
                HelpExample {
                    description: "Create custom agent".to_string(),
                    command: "agentic-dev agent create my-agent --agent-type refactoring".to_string(),
                    expected_output: None,
                },
            ],
            common_options: vec![
                CommonOption {
                    flag: "--all".to_string(),
                    description: "Apply action to all agents".to_string(),
                    example: Some("agent start --all".to_string()),
                },
                CommonOption {
                    flag: "--agent-type".to_string(),
                    description: "Type of agent to create".to_string(),
                    example: Some("--agent-type review".to_string()),
                },
            ],
            related_commands: vec![
                "interactive".to_string(),
                "generate".to_string(),
                "status".to_string(),
            ],
            troubleshooting: vec![
                TroubleshootingTip {
                    problem: "Agent fails to start".to_string(),
                    solution: "Check agent logs and ensure AI provider is configured".to_string(),
                    additional_info: Some("Use 'agent logs <agent-name>' to see error details".to_string()),
                },
            ],
        });

        // Config command
        self.command_help.insert("config".to_string(), HelpContent {
            description: "Manage configuration settings and environments".to_string(),
            long_description: Some(
                "Configure all aspects of the agentic development environment including AI \
                 providers, agent settings, UI preferences, and environment-specific configurations. \
                 Supports multiple environments and configuration validation.".to_string()
            ),
            examples: vec![
                HelpExample {
                    description: "Show current configuration".to_string(),
                    command: "agentic-dev config show".to_string(),
                    expected_output: None,
                },
                HelpExample {
                    description: "Set specific configuration value".to_string(),
                    command: "agentic-dev config set agents.max_concurrent 8".to_string(),
                    expected_output: Some("✅ Configuration updated".to_string()),
                },
                HelpExample {
                    description: "Switch to development environment".to_string(),
                    command: "agentic-dev config environment dev".to_string(),
                    expected_output: None,
                },
            ],
            common_options: vec![
                CommonOption {
                    flag: "--format, -f".to_string(),
                    description: "Output format for show/export commands".to_string(),
                    example: Some("--format yaml".to_string()),
                },
                CommonOption {
                    flag: "--merge, -m".to_string(),
                    description: "Merge imported configuration with existing".to_string(),
                    example: Some("config import settings.toml --merge".to_string()),
                },
            ],
            related_commands: vec![
                "status".to_string(),
                "agent list".to_string(),
                "init".to_string(),
            ],
            troubleshooting: vec![
                TroubleshootingTip {
                    problem: "Invalid configuration value".to_string(),
                    solution: "Use 'config validate' to check for errors and see valid options".to_string(),
                    additional_info: None,
                },
            ],
        });
    }
}

/// Interactive help session
pub struct InteractiveHelp {
    help_system: HelpSystem,
    session_active: bool,
}

impl InteractiveHelp {
    pub fn new() -> Self {
        Self {
            help_system: HelpSystem::new(),
            session_active: false,
        }
    }

    /// Start interactive help session
    pub fn start_session(&mut self) -> String {
        self.session_active = true;
        let mut output = String::new();

        output.push_str(
            &"🤖 Agentic Development Environment - Interactive Help\n\n"
                .bold()
                .cyan()
                .to_string(),
        );
        output.push_str("Welcome to the interactive help system!\n\n");

        for prompt in self.help_system.get_interactive_prompts() {
            output.push_str(&format!("{}\n", prompt));
        }

        output
    }

    /// Process help query
    pub fn process_query(&self, query: &str) -> String {
        let query_lower = query.trim().to_lowercase();

        match query_lower.as_str() {
            "exit" | "quit" | "q" => {
                "Thanks for using the agentic development environment! 👋".to_string()
            }
            "help" | "?" => self.help_system.get_getting_started_guide(),
            "commands" | "list" => {
                let commands = self.help_system.get_available_commands();
                let mut output = String::from("Available commands:\n\n");
                for command in commands {
                    if let Some(help) = self.help_system.get_command_help(&command) {
                        output.push_str(&format!("  {} - {}\n", command.bold(), help.description));
                    }
                }
                output
            }
            _ => {
                // Check if it's a specific command
                if let Some(_help) = self.help_system.get_command_help(&query_lower) {
                    self.help_system.format_command_help(&query_lower)
                } else {
                    // Search for the query
                    let search_results = self.help_system.search_help(&query_lower);
                    if search_results.is_empty() {
                        format!(
                            "No help found for '{}'. Try 'commands' to see all available commands, or 'help' for the getting started guide.",
                            query
                        )
                    } else {
                        let mut output = format!("Search results for '{}':\n\n", query);
                        for result in search_results.iter().take(5) {
                            output.push_str(&format!("  • {}\n", result));
                        }
                        if search_results.len() > 5 {
                            output.push_str(&format!(
                                "  ... and {} more results\n",
                                search_results.len() - 5
                            ));
                        }
                        output
                    }
                }
            }
        }
    }

    /// End help session
    pub fn end_session(&mut self) {
        self.session_active = false;
    }

    /// Check if session is active
    pub fn is_active(&self) -> bool {
        self.session_active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_system_creation() {
        let help_system = HelpSystem::new();
        assert!(!help_system.get_available_commands().is_empty());
    }

    #[test]
    fn test_command_help_retrieval() {
        let help_system = HelpSystem::new();
        assert!(help_system.get_command_help("init").is_some());
        assert!(help_system.get_command_help("nonexistent").is_none());
    }

    #[test]
    fn test_help_search() {
        let help_system = HelpSystem::new();
        let results = help_system.search_help("generate");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_interactive_help() {
        let mut interactive_help = InteractiveHelp::new();
        let welcome = interactive_help.start_session();
        assert!(welcome.contains("Interactive Help"));

        let response = interactive_help.process_query("init");
        assert!(response.contains("Initialize"));

        interactive_help.end_session();
        assert!(!interactive_help.is_active());
    }
}
