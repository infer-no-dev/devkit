//! Enhanced CLI system for the Agentic Development Environment
//!
//! This module provides a comprehensive command-line interface with:
//! - Rich subcommand structure for all system operations
//! - Interactive modes and wizards
//! - Auto-completion support
//! - Progress indicators and rich output formatting
//! - Integration with all system components

use clap::{Args, Parser, Subcommand};
use crossterm::{
    style::{Color, Print, ResetColor, SetForegroundColor},
    ExecutableCommand,
};
use is_terminal::IsTerminal;
use std::io::{self, Write};
use std::path::PathBuf;

pub mod commands;
pub mod completion;
pub mod formatting;
pub mod help;
pub mod interactive;
pub mod progress;
pub mod validation;

use crate::agents::AgentSystem;
use crate::config::ConfigManager;
use crate::context::ContextManager;

/// Simple color support detection
fn supports_color() -> bool {
    // Check common environment variables and terminal capabilities
    std::env::var("NO_COLOR").is_err()
        && (std::env::var("FORCE_COLOR").is_ok()
            || std::env::var("TERM")
                .map(|term| !term.is_empty() && term != "dumb")
                .unwrap_or(false))
}

/// Main CLI application structure
#[derive(Parser)]
#[command(
    name = "agentic-dev",
    about = "Agentic Development Environment - AI-powered code generation and analysis",
    version = env!("CARGO_PKG_VERSION"),
    long_about = "
🤖 Agentic Development Environment

An intelligent, multi-agent development environment built for AI-assisted 
code generation on large existing codebases. The system leverages multiple 
concurrent AI agents, advanced code analysis, and natural language programming.

Features:
• Multi-agent coordination for complex tasks
• Advanced code analysis with semantic understanding
• Context-aware code generation
• Cross-platform shell integration
• Rich terminal-based interface
• Comprehensive configuration management
"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Quiet mode - minimal output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Configuration file path
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    /// Working directory
    #[arg(short = 'C', long, global = true)]
    pub directory: Option<PathBuf>,

    /// Output format (text, json, yaml)
    #[arg(long, global = true, default_value = "text")]
    pub format: OutputFormat,

    /// Enable colored output
    #[arg(long, global = true, default_value = "auto")]
    pub color: ColorMode,
}

/// Available CLI commands
#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new agentic development project
    #[command(alias = "new")]
    Init(InitArgs),

    /// Start the interactive development mode
    #[command(alias = "dev")]
    Interactive(InteractiveArgs),

    /// Analyze codebase and generate context
    Analyze(AnalyzeArgs),

    /// Generate code using AI agents
    Generate(GenerateArgs),

    /// Manage AI agents
    Agent(AgentArgs),

    /// Configuration management
    Config(ConfigArgs),

    /// Context and symbol inspection
    Inspect(InspectArgs),

    /// Performance profiling and diagnostics
    Profile(ProfileArgs),

    /// Template management
    Template(TemplateArgs),

    /// Project status and health check
    Status(StatusArgs),

    /// Shell integration and completion
    Shell(ShellArgs),

    /// Run end-to-end demo workflow
    Demo(DemoArgs),
}

/// Project initialization arguments
#[derive(Args)]
pub struct InitArgs {
    /// Project directory name
    pub name: String,

    /// Project template to use
    #[arg(short, long)]
    pub template: Option<String>,

    /// Programming language
    #[arg(short, long)]
    pub language: Option<String>,

    /// Skip interactive prompts
    #[arg(long)]
    pub no_interactive: bool,

    /// Force overwrite existing directory
    #[arg(long)]
    pub force: bool,

    /// Initialize git repository
    #[arg(long, default_value = "true")]
    pub git: bool,
}

/// Interactive mode arguments
#[derive(Args)]
pub struct InteractiveArgs {
    /// Start with specific view (agents, context, config, logs)
    #[arg(short = 'V', long)]
    pub view: Option<String>,

    /// Auto-start agents
    #[arg(long)]
    pub auto_start: bool,

    /// Monitor mode (read-only)
    #[arg(short, long)]
    pub monitor: bool,
}

/// Code analysis arguments
#[derive(Args)]
pub struct AnalyzeArgs {
    /// Target files or directories
    pub targets: Vec<PathBuf>,

    /// Analysis depth (shallow, normal, deep)
    #[arg(short, long, default_value = "normal")]
    pub depth: String,

    /// Include test files
    #[arg(long)]
    pub include_tests: bool,

    /// Export results to file
    #[arg(short, long)]
    pub export: Option<PathBuf>,

    /// Specific analysis types (symbols, dependencies, architecture, quality)
    #[arg(long)]
    pub analysis_types: Vec<String>,

    /// Show progress
    #[arg(short, long)]
    pub progress: bool,
}

/// Code generation arguments
#[derive(Args)]
pub struct GenerateArgs {
    /// Natural language prompt
    pub prompt: String,

    /// Target file or directory
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Programming language
    #[arg(short, long)]
    pub language: Option<String>,

    /// Include context files
    #[arg(long)]
    pub context: Vec<PathBuf>,

    /// Generation strategy (focused, comprehensive, iterative)
    #[arg(long, default_value = "focused")]
    pub strategy: String,

    /// Maximum tokens to generate
    #[arg(long)]
    pub max_tokens: Option<usize>,

    /// Temperature for generation (0.0-1.0)
    #[arg(long)]
    pub temperature: Option<f32>,

    /// Preview mode (don't write files)
    #[arg(short, long)]
    pub preview: bool,
}

/// Agent management arguments
#[derive(Args)]
pub struct AgentArgs {
    #[command(subcommand)]
    pub command: AgentCommands,
}

#[derive(Subcommand)]
pub enum AgentCommands {
    /// List available agents
    List,
    /// Show agent status
    Status {
        /// Agent ID or name
        agent: Option<String>,
    },
    /// Start specific agents
    Start {
        /// Agent IDs or names
        agents: Vec<String>,
        /// Start all agents
        #[arg(long)]
        all: bool,
    },
    /// Stop specific agents
    Stop {
        /// Agent IDs or names
        agents: Vec<String>,
        /// Stop all agents
        #[arg(long)]
        all: bool,
    },
    /// Create custom agent
    Create {
        /// Agent name
        name: String,
        /// Agent type
        #[arg(short, long)]
        agent_type: String,
        /// Configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    /// Remove custom agent
    Remove {
        /// Agent name
        name: String,
    },
    /// Show agent logs
    Logs {
        /// Agent ID or name
        agent: String,
        /// Number of lines to show
        #[arg(short, long, default_value = "50")]
        lines: usize,
        /// Follow logs
        #[arg(short, long)]
        follow: bool,
    },
}

/// Configuration management arguments
#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommands,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    Show {
        /// Specific config path (e.g., agents.max_concurrent)
        path: Option<String>,
    },
    /// Set configuration value
    Set {
        /// Configuration path
        path: String,
        /// Value to set
        value: String,
    },
    /// Get configuration value
    Get {
        /// Configuration path
        path: String,
    },
    /// Validate configuration
    Validate,
    /// Switch environment
    Environment {
        /// Environment name (dev, staging, prod)
        env: String,
    },
    /// List available environments
    Environments,
    /// Edit configuration interactively
    Edit,
    /// Reset to defaults
    Reset {
        /// Specific section to reset
        section: Option<String>,
    },
    /// Export configuration
    Export {
        /// Output file
        output: PathBuf,
        /// Format (toml, json, yaml)
        #[arg(short, long, default_value = "toml")]
        format: String,
    },
    /// Import configuration
    Import {
        /// Input file
        input: PathBuf,
        /// Merge with existing config
        #[arg(short, long)]
        merge: bool,
    },
}

/// Code inspection arguments
#[derive(Args)]
pub struct InspectArgs {
    #[command(subcommand)]
    pub command: InspectCommands,
}

#[derive(Subcommand)]
pub enum InspectCommands {
    /// Inspect symbols in codebase
    Symbols {
        /// Symbol name pattern
        pattern: Option<String>,
        /// Symbol type filter
        #[arg(short, long)]
        symbol_type: Option<String>,
        /// File path filter
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Show file context information
    File {
        /// File path
        path: PathBuf,
        /// Show detailed analysis
        #[arg(short, long)]
        detailed: bool,
    },
    /// Analyze dependencies
    Dependencies {
        /// Target files or directories
        targets: Vec<PathBuf>,
        /// Show external dependencies only
        #[arg(long)]
        external_only: bool,
        /// Include dev dependencies
        #[arg(long)]
        include_dev: bool,
    },
    /// Show code relationships
    Relationships {
        /// Starting symbol or file
        target: String,
        /// Maximum depth
        #[arg(short, long, default_value = "3")]
        depth: usize,
        /// Relationship types to follow
        #[arg(short, long)]
        types: Vec<String>,
    },
    /// Code quality metrics
    Quality {
        /// Target files or directories
        targets: Vec<PathBuf>,
        /// Include detailed metrics
        #[arg(short, long)]
        detailed: bool,
    },
}

/// Profiling and diagnostics arguments
#[derive(Args)]
pub struct ProfileArgs {
    #[command(subcommand)]
    pub command: ProfileCommands,
}

#[derive(Subcommand)]
pub enum ProfileCommands {
    /// Profile system performance
    System {
        /// Duration to profile (seconds)
        #[arg(short, long, default_value = "30")]
        duration: u64,
        /// Include memory profiling
        #[arg(short, long)]
        memory: bool,
    },
    /// Profile agent performance
    Agents {
        /// Specific agent to profile
        agent: Option<String>,
        /// Duration to profile (seconds)
        #[arg(short, long, default_value = "60")]
        duration: u64,
    },
    /// Profile context analysis
    Context {
        /// Target directory
        target: PathBuf,
        /// Include timing breakdown
        #[arg(long)]
        breakdown: bool,
    },
    /// Show system diagnostics
    Diagnostics,
    /// Memory usage analysis
    Memory {
        /// Show detailed breakdown
        #[arg(short, long)]
        detailed: bool,
    },
}

/// Template management arguments
#[derive(Args)]
pub struct TemplateArgs {
    #[command(subcommand)]
    pub command: TemplateCommands,
}

#[derive(Subcommand)]
pub enum TemplateCommands {
    /// List available templates
    List {
        /// Language filter
        #[arg(short, long)]
        language: Option<String>,
    },
    /// Show template details
    Show {
        /// Template name
        name: String,
    },
    /// Create new template
    Create {
        /// Template name
        name: String,
        /// Language
        #[arg(short, long)]
        language: String,
        /// Template source directory
        #[arg(short, long)]
        source: PathBuf,
    },
    /// Remove template
    Remove {
        /// Template name
        name: String,
    },
    /// Update template
    Update {
        /// Template name
        name: String,
        /// New source directory
        #[arg(short, long)]
        source: Option<PathBuf>,
    },
}

/// Status and health check arguments
#[derive(Args)]
pub struct StatusArgs {
    /// Show detailed status
    #[arg(short, long)]
    pub detailed: bool,

    /// Check specific components
    #[arg(short, long)]
    pub components: Vec<String>,

    /// Include performance metrics
    #[arg(short, long)]
    pub performance: bool,

    /// Check external dependencies
    #[arg(long)]
    pub external: bool,
}

/// Shell integration arguments
#[derive(Args)]
pub struct ShellArgs {
    #[command(subcommand)]
    pub command: ShellCommands,
}

/// Demo workflow arguments
#[derive(Args)]
pub struct DemoArgs {
    /// Run specific demo step (analyze, generate, interactive, all)
    #[arg(short, long)]
    pub step: Option<String>,

    /// Skip confirmation prompts
    #[arg(long)]
    pub yes: bool,

    /// Clean up demo artifacts after completion
    #[arg(long)]
    pub cleanup: bool,
}

#[derive(Subcommand)]
pub enum ShellCommands {
    /// Generate shell completion scripts
    Completion {
        /// Shell type (bash, zsh, fish, powershell)
        shell: String,
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Install shell integration
    Install {
        /// Shell type
        shell: Option<String>,
    },
    /// Show shell integration status
    Status,
}

/// Output format options
#[derive(Clone, Debug)]
pub enum OutputFormat {
    Text,
    Json,
    Yaml,
    Table,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            "yaml" => Ok(OutputFormat::Yaml),
            "table" => Ok(OutputFormat::Table),
            _ => Err(format!("Invalid output format: {}", s)),
        }
    }
}

/// Color mode options
#[derive(Clone, Debug)]
pub enum ColorMode {
    Always,
    Never,
    Auto,
}

impl std::str::FromStr for ColorMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "always" => Ok(ColorMode::Always),
            "never" => Ok(ColorMode::Never),
            "auto" => Ok(ColorMode::Auto),
            _ => Err(format!("Invalid color mode: {}", s)),
        }
    }
}

/// Main CLI application runner
pub struct CliRunner {
    config_manager: ConfigManager,
    context_manager: Option<ContextManager>,
    agent_system: Option<AgentSystem>,
    verbose: bool,
    quiet: bool,
    format: OutputFormat,
    color_enabled: bool,
}

impl CliRunner {
    /// Create a new CLI runner
    pub fn new(cli: &Cli) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize configuration manager
        let config_manager = ConfigManager::new(cli.config.clone())?;

        // Configuration is already loaded by the constructor

        // Determine color mode
        let color_enabled = match cli.color {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => supports_color(),
        };

        // Change directory if specified
        if let Some(dir) = &cli.directory {
            std::env::set_current_dir(dir)?;
        }

        Ok(Self {
            config_manager,
            context_manager: None,
            agent_system: None,
            verbose: cli.verbose,
            quiet: cli.quiet,
            format: cli.format.clone(),
            color_enabled,
        })
    }

    /// Run the CLI command
    pub async fn run(&mut self, command: Commands) -> Result<(), Box<dyn std::error::Error>> {
        match command {
            Commands::Init(args) => self.run_init(args).await,
            Commands::Interactive(args) => self.run_interactive(args).await,
            Commands::Analyze(args) => self.run_analyze(args).await,
            Commands::Generate(args) => self.run_generate(args).await,
            Commands::Agent(args) => self.run_agent(args.command).await,
            Commands::Config(args) => self.run_config(args.command).await,
            Commands::Inspect(args) => self.run_inspect(args.command).await,
            Commands::Profile(args) => self.run_profile(args.command).await,
            Commands::Template(args) => self.run_template(args.command).await,
            Commands::Status(args) => self.run_status(args).await,
            Commands::Shell(args) => self.run_shell(args.command).await,
            Commands::Demo(args) => self.run_demo(args).await,
        }
    }

    /// Print formatted output with color support
    pub fn print_output(&self, content: &str, color: Option<Color>) {
        if !self.quiet {
            let mut stdout = io::stdout();

            if self.color_enabled && color.is_some() {
                let _ = stdout.execute(SetForegroundColor(color.unwrap()));
                let _ = stdout.execute(Print(content));
                let _ = stdout.execute(ResetColor);
            } else {
                print!("{}", content);
            }

            let _ = stdout.flush();
        }
    }

    /// Print success message
    pub fn print_success(&self, message: &str) {
        self.print_output(&format!("✅ {}\n", message), Some(Color::Green));
    }

    /// Print error message
    pub fn print_error(&self, message: &str) {
        self.print_output(&format!("❌ {}\n", message), Some(Color::Red));
    }

    /// Print warning message
    pub fn print_warning(&self, message: &str) {
        self.print_output(&format!("⚠️  {}\n", message), Some(Color::Yellow));
    }

    /// Print info message
    pub fn print_info(&self, message: &str) {
        self.print_output(&format!("ℹ️  {}\n", message), Some(Color::Blue));
    }

    /// Print verbose message
    pub fn print_verbose(&self, message: &str) {
        if self.verbose {
            self.print_output(&format!("🔍 {}\n", message), Some(Color::Cyan));
        }
    }

    /// Print command example
    pub fn print_command(&self, command: &str) {
        self.print_output(&format!("  $ {}\n", command), Some(Color::Cyan));
    }

    /// Print code block
    pub fn print_code(&self, code: &str) {
        if self.color_enabled {
            io::stdout()
                .execute(SetForegroundColor(Color::DarkGrey))
                .ok();
        }
        print!(
            "┌─────────────────────────────────────────────────────────────────────────────────┐\n"
        );

        for line in code.lines() {
            println!("│ {:<79} │", line);
        }

        print!(
            "└─────────────────────────────────────────────────────────────────────────────────┘\n"
        );
        if self.color_enabled {
            io::stdout().execute(ResetColor).ok();
        }
    }

    /// Check if running in interactive terminal
    pub fn is_interactive(&self) -> bool {
        std::io::stdin().is_terminal()
    }

    /// Initialize context manager if not already done
    pub async fn ensure_context_manager(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.context_manager.is_none() {
            let _config = self.config_manager.config();

            self.context_manager = Some(ContextManager::new()?);
            self.print_verbose("Context manager initialized");
        }
        Ok(())
    }

    /// Get mutable reference to context manager
    pub fn context_manager_mut(&mut self) -> Option<&mut ContextManager> {
        self.context_manager.as_mut()
    }

    /// Get verbose flag
    pub fn verbose(&self) -> bool {
        self.verbose
    }

    /// Get quiet flag  
    pub fn quiet(&self) -> bool {
        self.quiet
    }

    /// Get output format
    pub fn format(&self) -> &OutputFormat {
        &self.format
    }

    /// Get config manager
    pub fn config_manager(&self) -> &ConfigManager {
        &self.config_manager
    }

    /// Get mutable config manager
    pub fn config_manager_mut(&mut self) -> &mut ConfigManager {
        &mut self.config_manager
    }

    /// Initialize agent system if not already done
    pub async fn ensure_agent_system(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.agent_system.is_none() {
            self.ensure_context_manager().await?;

            let _config = self.config_manager.config();

            // For now, just initialize with default AgentSystem
            self.agent_system = Some(AgentSystem::new());
            self.print_verbose("Agent system initialized");
        }
        Ok(())
    }

    // Command implementations will be added in separate files
    async fn run_init(&mut self, args: InitArgs) -> Result<(), Box<dyn std::error::Error>> {
        commands::init::run(self, args).await
    }

    async fn run_interactive(
        &mut self,
        args: InteractiveArgs,
    ) -> Result<(), Box<dyn std::error::Error>> {
        commands::interactive::run(self, args).await
    }

    async fn run_analyze(&mut self, args: AnalyzeArgs) -> Result<(), Box<dyn std::error::Error>> {
        commands::analyze::run(self, args).await
    }

    async fn run_generate(&mut self, args: GenerateArgs) -> Result<(), Box<dyn std::error::Error>> {
        commands::generate::run(self, args).await
    }

    async fn run_agent(
        &mut self,
        command: AgentCommands,
    ) -> Result<(), Box<dyn std::error::Error>> {
        commands::agent::run(self, command).await
    }

    async fn run_config(
        &mut self,
        command: ConfigCommands,
    ) -> Result<(), Box<dyn std::error::Error>> {
        commands::config::run(self, command).await
    }

    async fn run_inspect(
        &mut self,
        command: InspectCommands,
    ) -> Result<(), Box<dyn std::error::Error>> {
        commands::inspect::run(self, command).await
    }

    async fn run_profile(
        &mut self,
        command: ProfileCommands,
    ) -> Result<(), Box<dyn std::error::Error>> {
        commands::profile::run(self, command).await
    }

    async fn run_template(
        &mut self,
        command: TemplateCommands,
    ) -> Result<(), Box<dyn std::error::Error>> {
        commands::template::run(self, command).await
    }

    async fn run_status(&mut self, args: StatusArgs) -> Result<(), Box<dyn std::error::Error>> {
        commands::status::run(self, args).await
    }

    async fn run_shell(
        &mut self,
        command: ShellCommands,
    ) -> Result<(), Box<dyn std::error::Error>> {
        commands::shell::run(self, command).await
    }

    async fn run_demo(&mut self, args: DemoArgs) -> Result<(), Box<dyn std::error::Error>> {
        commands::demo::run(self, args).await
    }
}
