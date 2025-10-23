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
pub mod session_manager;
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

/// Version string with build metadata
const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (git:",
    env!("BUILD_GIT_HASH"),
    ", built:",
    env!("BUILD_TIMESTAMP"),
    ")"
);

/// Main CLI application structure
#[derive(Parser)]
#[command(
    name = "devkit",
    about = "Agentic Development Environment - AI-powered code generation and analysis",
    version = VERSION,
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

    // Global orchestrator defaults
    /// Task timeout in seconds (orchestrator)
    #[arg(long = "orchestrator-task-timeout-seconds", global = true)]
    pub orchestrator_task_timeout_seconds: Option<u64>,

    /// Enable retries (orchestrator)
    #[arg(long = "orchestrator-retry-failed-tasks", global = true)]
    pub orchestrator_retry_failed_tasks: Option<bool>,

    /// Max retry attempts (orchestrator)
    #[arg(long = "orchestrator-max-retry-attempts", global = true)]
    pub orchestrator_max_retry_attempts: Option<usize>,

    /// Backoff strategy: fixed|exponential (orchestrator)
    #[arg(long = "orchestrator-backoff", global = true)]
    pub orchestrator_backoff: Option<String>,

    /// Base/backoff seconds (orchestrator)
    #[arg(long = "orchestrator-backoff-base-secs", global = true)]
    pub orchestrator_backoff_base_secs: Option<u64>,

    /// Backoff factor (orchestrator, exponential only)
    #[arg(long = "orchestrator-backoff-factor", global = true)]
    pub orchestrator_backoff_factor: Option<u32>,

    /// Backoff max seconds (orchestrator, exponential only)
    #[arg(long = "orchestrator-backoff-max-secs", global = true)]
    pub orchestrator_backoff_max_secs: Option<u64>,
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
    #[command(
        about = "Generate code or scaffold a project from a prompt",
        long_about = "Generate code or scaffold a multi-file project from a natural language prompt.\n\nExamples:\n  devkit generate \"todo api\" --language rust --stack rust-axum --root ./api\n  devkit generate \"marketing site\" --language typescript --stack nextjs --root ./web --dry-run\n  devkit generate --list-stacks\n  devkit generate --apply-plan plan.json --force\n\nFlags:\n  --stack, --dry-run, --force, --no-scaffold, --single-file, --root, --export-plan, --apply-plan, --list-stacks"
    )]
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

    /// System blueprint operations
    Blueprint(BlueprintArgs),

    /// Plugin marketplace operations
    Plugin(PluginArgs),

    /// AI-powered project manager agent
    #[command(alias = "c")]
    Chat(ChatArgs),

    /// Session management (list, create, switch, branch, analytics)
    Session(SessionArgs),

    /// Open coordination visualizer
    Visualize(VisualizeArgs),

    /// View system dashboard
    Dashboard(DashboardArgs),

    /// Generate analytics reports
    Analytics(AnalyticsArgs),

    /// Monitor agent performance and system metrics
    Monitor(MonitorArgs),

    /// Export session data and reports
    Export(ExportArgs),

    /// Agent behavior customization
    Behavior(BehaviorArgs),

    /// Run project diagnostics
    Diagnose(DiagnoseArgs),
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

    /// Enable web dashboard
    #[arg(short, long)]
    pub web: bool,

    /// Web dashboard port (default: 8080)
    #[arg(long)]
    pub web_port: Option<u16>,

    /// Web dashboard host (default: 127.0.0.1)
    #[arg(long)]
    pub web_host: Option<String>,
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

    /// Target file or directory (if directory and scaffolding is enabled, a project is created here)
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

    /// Enable automatic project scaffolding (multi-file). Disabled if --single-file or --no-scaffold is set.
    #[arg(long, default_value_t = true)]
    pub scaffold: bool,

    /// Disable scaffolding (alias for --scaffold=false)
    #[arg(long)]
    pub no_scaffold: bool,

    /// Force single-file output (disables scaffolding)
    #[arg(long)]
    pub single_file: bool,

    /// Root directory to scaffold into (overrides detection from --output)
    #[arg(long)]
    pub root: Option<PathBuf>,

    /// Stack preset (e.g. rust-axum, rust-actix, rust-axum-sqlx, node-express, node-nest, nextjs, python-fastapi, python-fastapi-sqlalchemy)
    #[arg(long)]
    pub stack: Option<String>,

    /// Dry run scaffolding (print plan, do not write)
    #[arg(long)]
    pub dry_run: bool,

    /// Overwrite existing files/directories during scaffolding
    #[arg(long)]
    pub force: bool,

    /// List available --stack presets and exit
    #[arg(long)]
    pub list_stacks: bool,

    /// Export planned file map to JSON (planning only)
    #[arg(long)]
    pub export_plan: Option<PathBuf>,

    /// Apply a previously exported plan JSON instead of generating
    #[arg(long)]
    pub apply_plan: Option<PathBuf>,
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
    /// Cancel a running or queued task by ID
    CancelTask {
        /// Task ID to cancel
        task_id: String,
    },
    /// Resume pending/running tasks from snapshots
    Resume,

    /// Configure or view orchestrator settings (timeouts, retries, backoff)
    Orchestrator(OrchestratorArgs),
}

/// Orchestrator settings arguments
#[derive(Args, Clone, Debug)]
pub struct OrchestratorArgs {
    /// Task timeout in seconds
    #[arg(long)]
    pub task_timeout_seconds: Option<u64>,

    /// Enable/disable retries for failed tasks
    #[arg(long)]
    pub retry_failed_tasks: Option<bool>,

    /// Maximum retry attempts
    #[arg(long)]
    pub max_retry_attempts: Option<usize>,

    /// Backoff strategy: fixed or exponential
    #[arg(long, default_value = "exponential")]
    pub backoff: String,

    /// For fixed/exponential: base delay seconds (fixed uses this as the constant delay)
    #[arg(long, default_value_t = 10)]
    pub backoff_base_secs: u64,

    /// For exponential strategy: growth factor
    #[arg(long, default_value_t = 2)]
    pub backoff_factor: u32,

    /// For exponential: maximum delay seconds
    #[arg(long, default_value_t = 300)]
    pub backoff_max_secs: u64,
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
    /// Apply a template with variables
    Apply {
        /// Template name
        name: String,
        /// Variables as key=value (repeatable)
        #[arg(short = 'v', long = "var")]
        vars: Vec<String>,
        /// Output file path (prints to stdout if omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Overwrite output file if it exists
        #[arg(long, default_value_t = false)]
        force: bool,
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
    #[arg(long)]
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

/// System blueprint operations
#[derive(Args)]
pub struct BlueprintArgs {
    #[command(subcommand)]
    pub command: BlueprintCommands,
}

/// Plugin marketplace operations
#[derive(Args)]
pub struct PluginArgs {
    #[command(subcommand)]
    pub command: PluginCommands,
}

/// AI-powered project manager arguments
#[derive(Args)]
pub struct ChatArgs {
    /// Project root directory
    #[arg(short, long)]
    pub project: Option<PathBuf>,

    /// Configuration file
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Enable debug output
    #[arg(long)]
    pub debug: bool,

    /// Initial message or question to start the conversation
    #[arg(short, long)]
    pub message: Option<String>,

    /// Keep conversation history persistent across sessions
    #[arg(long)]
    pub persist: bool,

    /// Continue from previous conversation session
    #[arg(long)]
    pub resume: bool,

    /// Show onboarding greeting (disable with --no-onboarding)
    #[arg(long, default_value_t = true)]
    pub onboarding: bool,

    /// Maximum number of conversation turns
    #[arg(long, default_value = "50")]
    pub max_turns: usize,
}

#[derive(Subcommand)]
pub enum BlueprintCommands {
    /// Extract system blueprint from codebase
    Extract {
        /// Source codebase path
        #[arg(short, long, default_value = ".")]
        source: PathBuf,
        /// Output blueprint file
        #[arg(short, long, default_value = "system_blueprint.toml")]
        output: PathBuf,
        /// Include detailed analysis
        #[arg(long)]
        detailed: bool,
    },
    /// Generate project from blueprint
    Generate {
        /// Blueprint file path
        blueprint: PathBuf,
        /// Output directory
        #[arg(short, long, default_value = "./generated_project")]
        output: PathBuf,
        /// Preview mode (don't create files)
        #[arg(short, long)]
        preview: bool,
    },
    /// Replicate current system
    Replicate {
        /// Target directory for replication
        #[arg(short, long, default_value = "./replicated_system")]
        target: PathBuf,
        /// Preserve git history
        #[arg(long)]
        preserve_git: bool,
        /// Skip validation of generated code
        #[arg(long)]
        skip_validation: bool,
        /// Dry run (show what would be done)
        #[arg(long)]
        dry_run: bool,
    },
    /// Validate blueprint file
    Validate {
        /// Blueprint file path
        blueprint: PathBuf,
    },
    /// Show blueprint information
    Info {
        /// Blueprint file path
        blueprint: PathBuf,
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },
    /// Compare blueprints
    Compare {
        /// First blueprint file
        blueprint1: PathBuf,
        /// Second blueprint file
        blueprint2: PathBuf,
    },
    /// Blueprint evolution and versioning
    #[command(subcommand)]
    Evolution(commands::evolution::EvolutionCommand),
}

#[derive(Subcommand)]
pub enum PluginCommands {
    /// Search for plugins in the marketplace
    Search {
        /// Search query
        query: Option<String>,
        
        /// Plugin category to filter by
        #[arg(long)]
        category: Option<String>,
        
        /// Show only free plugins
        #[arg(long)]
        free_only: bool,
        
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },

    /// Show detailed information about a plugin
    Info {
        /// Plugin ID to show info for
        plugin_id: String,
    },

    /// Install a plugin from the marketplace
    Install {
        /// Plugin ID to install
        plugin_id: String,
        
        /// Specific version to install
        #[arg(long)]
        version: Option<String>,
        
        /// License key for paid plugins
        #[arg(long)]
        license_key: Option<String>,
    },

    /// List installed plugins
    List {
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },

    /// Update plugins
    Update {
        /// Specific plugin to update (updates all if not specified)
        plugin_id: Option<String>,
    },

    /// Show plugin system status
    Status,
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

/// Session management arguments
#[derive(Args)]
pub struct SessionArgs {
    #[command(subcommand)]
    pub command: SessionCommands,
}

#[derive(Subcommand)]
pub enum SessionCommands {
    /// List all sessions
    List {
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },
    /// Create a new session
    Create {
        /// Session name
        name: String,
        /// Session description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Switch to a session
    Switch {
        /// Session name
        name: String,
    },
    /// Create session branch for experimentation
    Branch {
        #[command(subcommand)]
        command: SessionBranchCommands,
    },
    /// View session analytics
    Analytics {
        /// Session name
        #[arg(short, long)]
        session: Option<String>,
        /// Output format
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,
    },
}

#[derive(Subcommand)]
pub enum SessionBranchCommands {
    /// Create a new branch
    Create {
        /// Branch name
        name: String,
    },
    /// List branches
    List,
    /// Switch to a branch
    Switch {
        /// Branch name
        name: String,
    },
    /// Merge a branch
    Merge {
        /// Branch name
        name: String,
    },
}

/// Visualizer arguments
#[derive(Args)]
pub struct VisualizeArgs {
    /// Visualization type (network, timeline, resource, overview)
    #[arg(short = 'V', long, default_value = "network")]
    pub view: String,
    /// Auto-refresh interval in seconds
    #[arg(long, default_value = "5")]
    pub refresh: u64,
}

/// Dashboard arguments
#[derive(Args)]
pub struct DashboardArgs {
    /// Dashboard port
    #[arg(short, long, default_value = "8080")]
    pub port: u16,
    /// Dashboard host
    #[arg(long, default_value = "localhost")]
    pub host: String,
    /// Open browser automatically
    #[arg(long)]
    pub open: bool,
}

/// Analytics arguments
#[derive(Args)]
pub struct AnalyticsArgs {
    #[command(subcommand)]
    pub command: AnalyticsCommands,
}

#[derive(Subcommand)]
pub enum AnalyticsCommands {
    /// Generate analytics report
    Report {
        /// Output format
        #[arg(long, value_enum, default_value = "json")]
        format: OutputFormat,
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

/// Monitor arguments
#[derive(Args)]
pub struct MonitorArgs {
    /// Monitor target (agents, system, performance)
    #[arg(short, long, default_value = "agents")]
    pub target: String,
    /// Real-time monitoring
    #[arg(long)]
    pub real_time: bool,
    /// Refresh interval in seconds
    #[arg(long, default_value = "2")]
    pub interval: u64,
}

/// Export arguments
#[derive(Args)]
pub struct ExportArgs {
    /// Session name to export
    #[arg(short, long)]
    pub session: Option<String>,
    /// Export format
    #[arg(long, value_enum, default_value = "json")]
    pub format: OutputFormat,
    /// Output file
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

/// Behavior customization arguments
#[derive(Args)]
pub struct BehaviorArgs {
    #[command(subcommand)]
    pub command: BehaviorCommands,
}

#[derive(Subcommand)]
pub enum BehaviorCommands {
    /// Open behavior editor
    Edit,
    /// Load behavior profile
    Load {
        /// Profile name
        #[arg(long)]
        profile: String,
    },
    /// Create custom behavior profile
    Create {
        /// Profile name
        #[arg(long)]
        name: String,
        /// Interactive mode
        #[arg(long)]
        interactive: bool,
    },
    /// List available profiles
    List,
}

/// Diagnostics arguments
#[derive(Args)]
pub struct DiagnoseArgs {
    /// Run specific diagnostic (config, agents, context, all)
    #[arg(long, default_value = "all")]
    pub check: String,
    /// Fix issues automatically where possible
    #[arg(long)]
    pub fix: bool,
    /// Show detailed diagnostic information
    #[arg(short, long)]
    pub verbose: bool,
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
#[derive(Clone, Debug, clap::ValueEnum)]
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
    pub(crate) context_manager: Option<ContextManager>,
    agent_system: Option<AgentSystem>,
    verbose: bool,
    quiet: bool,
    format: OutputFormat,
    color_enabled: bool,
    orchestrator_defaults: OrchestratorArgs,
}

impl CliRunner {
    /// Create a new CLI runner
pub fn new(cli: &Cli) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize configuration manager with fallback support
        let config_manager = ConfigManager::new_with_smart_defaults(cli.config.clone())?;

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

let oc_snapshot = config_manager.config().orchestrator.clone();
        Ok(Self {
            config_manager,
            context_manager: None,
            agent_system: None,
            verbose: cli.verbose,
            quiet: cli.quiet,
            format: cli.format.clone(),
            color_enabled,
            orchestrator_defaults: {
                // Start with config values (snapshot taken before moving config_manager)
                let mut args = OrchestratorArgs {
                    task_timeout_seconds: Some(oc_snapshot.task_timeout_seconds),
                    retry_failed_tasks: Some(oc_snapshot.retry_failed_tasks),
                    max_retry_attempts: Some(oc_snapshot.max_retry_attempts),
                    backoff: oc_snapshot.backoff,
                    backoff_base_secs: oc_snapshot.backoff_base_secs,
                    backoff_factor: oc_snapshot.backoff_factor,
                    backoff_max_secs: oc_snapshot.backoff_max_secs,
                };
                // Override with CLI when provided
                if let Some(v) = cli.orchestrator_task_timeout_seconds { args.task_timeout_seconds = Some(v); }
                if let Some(v) = cli.orchestrator_retry_failed_tasks { args.retry_failed_tasks = Some(v); }
                if let Some(v) = cli.orchestrator_max_retry_attempts { args.max_retry_attempts = Some(v); }
                if let Some(v) = &cli.orchestrator_backoff { args.backoff = v.clone(); }
                if let Some(v) = cli.orchestrator_backoff_base_secs { args.backoff_base_secs = v; }
                if let Some(v) = cli.orchestrator_backoff_factor { args.backoff_factor = v; }
                if let Some(v) = cli.orchestrator_backoff_max_secs { args.backoff_max_secs = v; }
                args
            },
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
            Commands::Blueprint(args) => self.run_blueprint(args.command).await,
            Commands::Plugin(args) => self.run_plugin(args.command).await,
            Commands::Chat(args) => self.run_chat(args).await,
            Commands::Session(args) => self.run_session(args.command).await,
            Commands::Visualize(args) => self.run_visualize(args).await,
            Commands::Dashboard(args) => self.run_dashboard(args).await,
            Commands::Analytics(args) => self.run_analytics(args.command).await,
            Commands::Monitor(args) => self.run_monitor(args).await,
            Commands::Export(args) => self.run_export(args).await,
            Commands::Behavior(args) => self.run_behavior(args.command).await,
            Commands::Diagnose(args) => self.run_diagnose(args).await,
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

    /// Get orchestrator defaults
    pub fn orchestrator_defaults(&self) -> &OrchestratorArgs {
        &self.orchestrator_defaults
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

            // Build AgentSystem from global orchestrator defaults
            let mut sys_cfg = crate::agents::system::AgentSystemConfig::default();
            if let Some(t) = self.orchestrator_defaults.task_timeout_seconds { sys_cfg.task_timeout_seconds = t; }
            if let Some(r) = self.orchestrator_defaults.retry_failed_tasks { sys_cfg.retry_failed_tasks = r; }
            if let Some(m) = self.orchestrator_defaults.max_retry_attempts { sys_cfg.max_retry_attempts = m; }

            let retry_policy = match self.orchestrator_defaults.backoff.to_lowercase().as_str() {
                "fixed" => crate::agents::orchestrator::RetryPolicy { max_retries: sys_cfg.max_retry_attempts, strategy: crate::agents::orchestrator::BackoffStrategy::Fixed { delay_secs: self.orchestrator_defaults.backoff_base_secs } },
                _ => crate::agents::orchestrator::RetryPolicy { max_retries: sys_cfg.max_retry_attempts, strategy: crate::agents::orchestrator::BackoffStrategy::Exponential { base_secs: self.orchestrator_defaults.backoff_base_secs, factor: self.orchestrator_defaults.backoff_factor, max_secs: self.orchestrator_defaults.backoff_max_secs } },
            };

            let system = AgentSystem::with_config_and_policy(sys_cfg, retry_policy);
            system.initialize().await?;
            system.start().await?;
            self.agent_system = Some(system);
            self.print_verbose("Agent system initialized and started with orchestrator defaults");
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

    async fn run_blueprint(
        &mut self,
        command: BlueprintCommands,
    ) -> Result<(), Box<dyn std::error::Error>> {
        commands::blueprint::run(self, command).await
    }

    async fn run_plugin(
        &mut self,
        command: PluginCommands,
    ) -> Result<(), Box<dyn std::error::Error>> {
        commands::plugin::run(self, command).await
    }

    async fn run_chat(&mut self, args: ChatArgs) -> Result<(), Box<dyn std::error::Error>> {
        commands::chat::run(self, args).await
    }

    // New command implementations (stubs for now)
    async fn run_session(&mut self, command: SessionCommands) -> Result<(), Box<dyn std::error::Error>> {
        self.print_info("Session management functionality coming soon!");
        match command {
            SessionCommands::List { .. } => {
                self.print_output("📋 Available Sessions:\n", None);
                self.print_output("  (No sessions found - feature in development)\n", None);
            }
            SessionCommands::Create { name, description } => {
                self.print_info(&format!("Would create session '{}'", name));
                if let Some(desc) = description {
                    self.print_info(&format!("Description: {}", desc));
                }
            }
            SessionCommands::Switch { name } => {
                self.print_info(&format!("Would switch to session '{}'", name));
            }
            SessionCommands::Branch { .. } => {
                self.print_info("Session branching feature in development");
            }
            SessionCommands::Analytics { .. } => {
                self.print_info("Session analytics feature in development");
            }
        }
        Ok(())
    }

    async fn run_visualize(&mut self, args: VisualizeArgs) -> Result<(), Box<dyn std::error::Error>> {
        self.print_info(&format!("🎨 Opening {} visualization...", args.view));
        self.print_info("Visualization dashboard feature coming soon!");
        self.print_info(&format!("Would refresh every {} seconds", args.refresh));
        Ok(())
    }

    async fn run_dashboard(&mut self, args: DashboardArgs) -> Result<(), Box<dyn std::error::Error>> {
        self.print_info(&format!("🖥️ Starting dashboard on {}:{}", args.host, args.port));
        self.print_info("Web dashboard feature coming soon!");
        if args.open {
            self.print_info("Would open browser automatically");
        }
        Ok(())
    }

    async fn run_analytics(&mut self, command: AnalyticsCommands) -> Result<(), Box<dyn std::error::Error>> {
        self.print_info("📊 Analytics system functionality coming soon!");
        match command {
            AnalyticsCommands::Report { format, output } => {
                self.print_info(&format!("Would generate report in {:?} format", format));
                if let Some(path) = output {
                    self.print_info(&format!("Would save to: {}", path.display()));
                }
            }
        }
        Ok(())
    }

    async fn run_monitor(&mut self, args: MonitorArgs) -> Result<(), Box<dyn std::error::Error>> {
        self.print_info(&format!("🔍 Monitoring {} system...", args.target));
        self.print_info("Real-time monitoring feature coming soon!");
        if args.real_time {
            self.print_info(&format!("Would refresh every {} seconds", args.interval));
        }
        Ok(())
    }

    async fn run_export(&mut self, args: ExportArgs) -> Result<(), Box<dyn std::error::Error>> {
        self.print_info("📤 Export functionality coming soon!");
        if let Some(session) = args.session {
            self.print_info(&format!("Would export session: {}", session));
        }
        self.print_info(&format!("Format: {:?}", args.format));
        if let Some(output) = args.output {
            self.print_info(&format!("Would save to: {}", output.display()));
        }
        Ok(())
    }

    async fn run_behavior(&mut self, command: BehaviorCommands) -> Result<(), Box<dyn std::error::Error>> {
        self.print_info("🎭 Agent behavior customization coming soon!");
        match command {
            BehaviorCommands::Edit => {
                self.print_info("Would open behavior editor");
            }
            BehaviorCommands::Load { profile } => {
                self.print_info(&format!("Would load behavior profile: {}", profile));
            }
            BehaviorCommands::Create { name, interactive } => {
                self.print_info(&format!("Would create behavior profile: {}", name));
                if interactive {
                    self.print_info("Would use interactive mode");
                }
            }
            BehaviorCommands::List => {
                self.print_output("📋 Available Behavior Profiles:\n", None);
                self.print_output("  - default (built-in)\n", None);
                self.print_output("  - conservative-coder (placeholder)\n", None);
                self.print_output("  (Custom profiles feature in development)\n", None);
            }
        }
        Ok(())
    }

    async fn run_diagnose(&mut self, args: DiagnoseArgs) -> Result<(), Box<dyn std::error::Error>> {
        self.print_info(&format!("🔧 Running {} diagnostics...", args.check));
        
        // Run basic diagnostics as a placeholder
        self.print_output("\n🔍 System Diagnostics:\n", None);
        self.print_output("═══════════════════════\n", None);
        
        // Check configuration
        self.print_success("Configuration system is working");
        
        // Check build info
        let version = env!("CARGO_PKG_VERSION");
        let git_hash = env!("BUILD_GIT_HASH");
        self.print_info(&format!("Version: {} (git:{})", version, git_hash));
        
        if args.check == "all" || args.check == "config" {
            let config_path = self.config_manager.config_path();
            self.print_info(&format!("Config loaded from: {}", config_path.display()));
        }
        
        if args.fix {
            self.print_info("Auto-fix functionality coming soon!");
        }
        
        if args.verbose {
            self.print_info("Detailed diagnostic information coming soon!");
        }
        
        self.print_output("\n✅ Basic diagnostics completed\n", None);
        Ok(())
    }
}
