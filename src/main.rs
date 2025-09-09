//! devkit - AI-powered development toolkit
//!
//! From Infer No Dev - Just describe what you want, no manual coding needed.
//! Built in Rust for developers who are too lazy to write code manually.

// Module declarations
mod agents;
mod ai;
mod codegen;
mod context;
mod shell;
mod ui;
mod config;
mod interactive;
mod cli;

#[cfg(test)]
mod testing;

// External crate imports
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tokio;
use tracing::{info, error};
use anyhow::Result;

// Internal imports
use crate::config::{Config, ConfigManager};
use crate::agents::AgentSystem;
use crate::context::{ContextManager, AnalysisConfig};
use crate::shell::ShellManager;
use crate::ui::{Application, UIConfig};
use crate::codegen::CodeGenerator;
use crate::ai::AIManager;
use std::sync::Arc;

/// devkit - AI-powered development toolkit CLI
#[derive(Parser)]
#[command(name = "devkit")]
#[command(about = "AI-powered development toolkit - just describe what you want, no manual coding needed")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,
    
    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,
    
    /// Working directory
    #[arg(short, long)]
    working_dir: Option<PathBuf>,
    
    /// Subcommands
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Available subcommands
#[derive(Subcommand)]
enum Commands {
    /// Launch multi-agent development dashboard with visual UI
    Start {
        /// Project path to analyze
        #[arg(short, long)]
        project: Option<PathBuf>,
    },
    
    /// Initialize configuration for a new project
    Init {
        /// Target directory for initialization
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    
    /// Configure the development environment
    Config {
        /// Show current configuration
        #[arg(long)]
        show: bool,
        
        /// Reset to default configuration
        #[arg(long)]
        reset: bool,
        
        /// Export configuration to JSON
        #[arg(long)]
        export: Option<PathBuf>,
        
        /// Import configuration from JSON
        #[arg(long)]
        import: Option<PathBuf>,
    },
    
    /// Analyze a codebase and generate context
    Analyze {
        /// Path to analyze
        path: PathBuf,
        
        /// Output file for analysis results
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Include dependency analysis
        #[arg(long)]
        dependencies: bool,
    },
    
    /// Generate code from a natural language prompt
    Generate {
        /// Natural language prompt
        prompt: String,
        
        /// Target file path (for output)
        #[arg(short, long)]
        file: Option<PathBuf>,
        
        /// Target programming language
        #[arg(short, long)]
        language: Option<String>,
        
        /// Write generated code to project (auto-detect filename)
        #[arg(long)]
        write_to_project: bool,
        
        /// Force overwrite existing files
        #[arg(long)]
        force: bool,
        
        /// Create directory structure if needed
        #[arg(long)]
        create_dirs: bool,
        
        /// Apply code modifications to existing file instead of creating new
        #[arg(long)]
        modify: bool,
    },
    
    /// Start conversational AI chat mode for code generation
    Interactive {
        /// Project path to analyze for context
        #[arg(short, long)]
        project: Option<PathBuf>,
        
        /// Load existing session file
        #[arg(long)]
        session: Option<PathBuf>,
        
        /// Save session to file on exit
        #[arg(long)]
        save_session: Option<PathBuf>,
    },
    
    /// Show version information
    Version,
}

/// Main application state
struct DevKit {
    config_manager: ConfigManager,
    agent_system: std::sync::Arc<AgentSystem>,
    context_manager: ContextManager,
    shell_manager: ShellManager,
    code_generator: CodeGenerator,
}

impl DevKit {
    /// Initialize the devkit development environment
    async fn new(config_path: Option<PathBuf>) -> Result<Self> {
        // Initialize configuration
        let mut config_manager = ConfigManager::new(config_path)
            .map_err(|e| anyhow::anyhow!("Failed to initialize config: {}", e))?;
        config_manager.load()
            .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;
        
        // Initialize AI manager first
        let ai_manager = match AIManager::from_config(config_manager.config()).await {
            Ok(manager) => {
                info!("AI manager initialized successfully");
                manager
            },
            Err(e) => {
                info!("AI manager initialization failed, some features may be unavailable: {}", e);
                // Create a minimal AI manager with default config for fallback
                AIManager::new(crate::config::AIModelConfig::default()).await
                    .map_err(|e| anyhow::anyhow!("Failed to initialize fallback AI manager: {}", e))?
            }
        };
        let ai_manager = Arc::new(ai_manager);
        
        // Initialize core systems
        let agent_system = AgentSystem::new();
        
        // Initialize specialized agents with AI capabilities
        let code_gen_agent = crate::agents::agent_types::CodeGenerationAgent::with_ai_manager(ai_manager.clone());
        let analysis_agent = crate::agents::agent_types::AnalysisAgent::with_ai_manager(ai_manager.clone());
        let refactoring_agent = crate::agents::agent_types::RefactoringAgent::with_ai_manager(ai_manager.clone());
        
        // Register agents with the system
        agent_system.register_agent(Box::new(code_gen_agent)).await;
        agent_system.register_agent(Box::new(analysis_agent)).await;
        agent_system.register_agent(Box::new(refactoring_agent)).await;
        info!("Agent system initialized with 3 specialized agents");
        
        let context_manager = ContextManager::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize context manager: {}", e))?;
        let shell_manager = ShellManager::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize shell manager: {}", e))?;
        let mut code_generator = CodeGenerator::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize code generator: {}", e))?;
        
        // Connect AI manager to code generator
        code_generator.set_ai_manager(ai_manager.clone());
        
        info!("Agentic development environment initialized successfully");
        info!("Current shell: {}", shell_manager.current_shell());
        info!("AI integration: {}", if code_generator.has_ai() { "Enabled" } else { "Disabled" });
        
        Ok(Self {
            config_manager,
            agent_system: std::sync::Arc::new(agent_system),
            context_manager,
            shell_manager,
            code_generator,
        })
    }
    
    /// Start the interactive UI
    async fn start_interactive(&mut self, project_path: Option<PathBuf>) -> Result<()> {
        info!("Starting interactive UI dashboard");
        
        // Analyze project if path provided
        let project_context = if let Some(ref path) = project_path {
            info!("Analyzing project at: {:?}", path);
            let analysis_config = AnalysisConfig::default();
            match self.context_manager
                .analyze_codebase(path.clone(), analysis_config)
                .await
            {
                Ok(context) => {
                    info!("Project analysis complete: {} files, {} symbols", 
                          context.files.len(), context.metadata.indexed_symbols);
                    Some(context)
                },
                Err(e) => {
                    info!("Could not analyze project context: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        // Create UI configuration from devkit config
        let ui_config = UIConfig::default(); // TODO: Load from self.config_manager
        let mut app = Application::new(ui_config)
            .map_err(|e| anyhow::anyhow!("Failed to initialize UI: {}", e))?;
        
        // Initialize agent system for UI integration
        self.agent_system.initialize().await;
        
        // Create communication channels
        let (ui_tx, ui_rx) = tokio::sync::mpsc::unbounded_channel::<crate::ui::UIEvent>();
        let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        
        // Connect command sender to UI
        app.set_command_sender(command_tx.clone());
        
        // Add welcome notification with longer duration
        let welcome_notification = crate::ui::notifications::Notification::info(
            "DevKit Dashboard Started".to_string(),
            "Multi-agent development environment is now active. Press '?' or F1 for help, 'i' for input mode, ':' for commands.".to_string(),
        ).with_ttl(std::time::Duration::from_secs(30)); // 30 seconds
        app.add_notification(welcome_notification);
        
        // Add a persistent help reminder
        let help_notification = crate::ui::notifications::Notification::system_message(
            "Press '?' or F1 to show keybindings help. Press 'q' or Ctrl+C to quit.".to_string(),
        ).sticky(); // This won't auto-dismiss
        app.add_notification(help_notification);
        
        // Add initial agent status
        app.update_agent_status(
            "CodeGeneration".to_string(),
            crate::agents::AgentStatus::Idle,
            None,
            None,
            None,
        );
        app.update_agent_status(
            "Analysis".to_string(),
            crate::agents::AgentStatus::Idle,
            None,
            None,
            None,
        );
        app.update_agent_status(
            "Refactoring".to_string(),
            crate::agents::AgentStatus::Idle,
            None,
            None,
            None,
        );
        
        // Create interactive session for command processing
        let session = crate::interactive::InteractiveSession::new(project_path.clone());
        let interactive_manager = crate::interactive::InteractiveManager::new(
            session,
            self.code_generator.clone(),
            Some(self.agent_system.clone()),
            project_context,
        );
        
        // Spawn background tasks
        let agent_monitor = Self::spawn_agent_monitor(
            self.agent_system.clone(),
            ui_tx.clone(),
        );
        
        let command_processor = Self::spawn_command_processor(
            interactive_manager,
            command_rx,
            ui_tx.clone(),
        );
        
        let ui_event_handler = Self::spawn_ui_event_handler(ui_rx);
        
        info!("Starting UI dashboard with multi-agent system");
        
        // Run the main UI event loop
        let ui_result = tokio::select! {
            result = app.run() => result,
            _ = tokio::signal::ctrl_c() => {
                info!("Received shutdown signal");
                Ok(())
            }
        };
        
        // Cleanup background tasks
        agent_monitor.abort();
        command_processor.abort();
        ui_event_handler.abort();
        
        match ui_result {
            Ok(_) => {
                info!("UI dashboard ended successfully");
                Ok(())
            }
            Err(e) => {
                error!("UI dashboard error: {}", e);
                Err(anyhow::anyhow!("UI dashboard failed: {}", e))
            }
        }
    }
    
    /// Initialize a new project
    async fn init_project(&self, path: PathBuf) -> Result<()> {
        info!("Initializing project at: {:?}", path);
        
        // Create directory if it doesn't exist
        std::fs::create_dir_all(&path)?;
        
        // Create default project configuration
        let config_path = path.join(".agentic-config.toml");
        let default_config = Config::default();
        
        let toml_content = toml::to_string_pretty(&default_config)?;
        std::fs::write(&config_path, toml_content)?;
        
        println!("✅ Project initialized successfully!");
        println!("Configuration file created at: {:?}", config_path);
        
        Ok(())
    }
    
    /// Handle configuration commands
    async fn handle_config_command(
        &mut self,
        show: bool,
        reset: bool,
        export: Option<PathBuf>,
        import: Option<PathBuf>,
    ) -> Result<()> {
        if show {
            let config = self.config_manager.config();
            let json = serde_json::to_string_pretty(config)?;
            println!("{}", json);
            return Ok(());
        }
        
        if reset {
            self.config_manager.reset_to_default();
            self.config_manager.save()
                .map_err(|e| anyhow::anyhow!("Failed to save config: {}", e))?;
            println!("✅ Configuration reset to defaults");
            return Ok(());
        }
        
        if let Some(export_path) = export {
            let json = self.config_manager.export_as_json()
                .map_err(|e| anyhow::anyhow!("Failed to export config: {}", e))?;
            std::fs::write(&export_path, json)?;
            println!("✅ Configuration exported to: {:?}", export_path);
            return Ok(());
        }
        
        if let Some(import_path) = import {
            let json = std::fs::read_to_string(&import_path)?;
            self.config_manager.import_from_json(&json)
                .map_err(|e| anyhow::anyhow!("Failed to import config: {}", e))?;
            self.config_manager.save()
                .map_err(|e| anyhow::anyhow!("Failed to save config: {}", e))?;
            println!("✅ Configuration imported from: {:?}", import_path);
            return Ok(());
        }
        
        println!("Use --show, --reset, --export <path>, or --import <path>");
        Ok(())
    }
    
    /// Analyze a codebase
    async fn analyze_codebase(
        &mut self,
        path: PathBuf,
        output: Option<PathBuf>,
        include_dependencies: bool,
    ) -> Result<()> {
        info!("Analyzing codebase at: {:?}", path);
        
        let mut analysis_config = AnalysisConfig::default();
        analysis_config.analyze_dependencies = include_dependencies;
        
        let context = self.context_manager
            .analyze_codebase(path, analysis_config)
            .await
            .map_err(|e| anyhow::anyhow!("Analysis failed: {}", e))?;
        
        let analysis_json = serde_json::to_string_pretty(&context)?;
        
        if let Some(output_path) = output {
            std::fs::write(&output_path, &analysis_json)?;
            println!("✅ Analysis results written to: {:?}", output_path);
        } else {
            println!("{}", analysis_json);
        }
        
        println!("\n📊 Analysis Summary:");
        println!("  Files: {}", context.metadata.total_files);
        println!("  Lines of code: {}", context.metadata.total_lines);
        println!("  Languages: {:?}", context.metadata.languages.keys().collect::<Vec<_>>());
        println!("  Dependencies: {}", context.dependencies.len());
        println!("  Analysis time: {}ms", context.metadata.analysis_duration_ms);
        
        Ok(())
    }
    
    /// Generate code from a prompt
    async fn generate_code(
        &mut self,
        prompt: String,
        file_path: Option<PathBuf>,
        language: Option<String>,
        write_to_project: bool,
        force: bool,
        create_dirs: bool,
        modify: bool,
    ) -> Result<()> {
        info!("Generating code from prompt: {}", prompt);
        
        // Analyze current project for context (lightweight analysis)
        let current_dir = std::env::current_dir()?;
        info!("Analyzing project context at: {:?}", current_dir);
        
        let mut analysis_config = AnalysisConfig::default();
        analysis_config.analyze_dependencies = false; // Keep it fast for code generation
        
        let context = match self.context_manager.analyze_codebase(current_dir.clone(), analysis_config).await {
            Ok(ctx) => {
                info!("Context analysis completed: {} files, {} symbols", ctx.files.len(), ctx.metadata.indexed_symbols);
                ctx
            },
            Err(e) => {
                info!("Could not analyze project context, using minimal context: {}", e);
                // Fallback to minimal context
                context::CodebaseContext {
                    root_path: current_dir,
                    files: Vec::new(),
                    symbols: Default::default(),
                    dependencies: Vec::new(),
                    repository_info: None,
                    semantic_analysis: None,
                    metadata: context::ContextMetadata {
                        analysis_timestamp: std::time::SystemTime::now(),
                        total_files: 0,
                        total_lines: 0,
                        languages: std::collections::HashMap::new(),
                        analysis_duration_ms: 0,
                        indexed_symbols: 0,
                        semantic_patterns_found: 0,
                        semantic_relationships: 0,
                    },
                }
            }
        };
        
        let mut generation_config = codegen::GenerationConfig::default();
        generation_config.target_language = language;
        
        // Clone file_path to avoid move issues
        let output_file = file_path.clone();
        
        let request = codegen::GenerationRequest {
            prompt,
            file_path: file_path.map(|p| p.to_string_lossy().to_string()),
            context,
            config: generation_config,
            constraints: Vec::new(),
        };
        
        match self.code_generator.generate_from_prompt(request).await {
            Ok(result) => {
                println!("✅ Code generation completed!");
                println!("Language: {}", result.language);
                println!("Confidence: {:.2}", result.confidence_score);
                println!("Generation time: {}ms", result.metadata.generation_time_ms);
                
                if result.metadata.tokens_used > 0 {
                    println!("Tokens used: {}", result.metadata.tokens_used);
                }
                
                println!("\n--- Generated Code ---");
                println!("{}", result.generated_code);
                
                if !result.suggestions.is_empty() {
                    println!("\n💡 Suggestions:");
                    for suggestion in result.suggestions {
                        println!("  • {}", suggestion);
                    }
                }
                
                // Handle file output with enhanced options
                self.handle_file_output(
                    &result.generated_code,
                    &result.language,
                    output_file,
                    write_to_project,
                    force,
                    create_dirs,
                    modify,
                ).await?;
            },
            Err(e) => {
                error!("Code generation failed: {}", e);
                return Err(anyhow::anyhow!("Code generation failed: {}", e));
            }
        }
        
        Ok(())
    }
    
    /// Handle file output with various options
    async fn handle_file_output(
        &self,
        generated_code: &str,
        language: &str,
        file_path: Option<PathBuf>,
        write_to_project: bool,
        force: bool,
        create_dirs: bool,
        modify: bool,
    ) -> Result<()> {
        // If neither file path nor write_to_project is specified, just print to console
        if file_path.is_none() && !write_to_project {
            return Ok(());
        }
        
        let output_path = if let Some(path) = file_path {
            path
        } else if write_to_project {
            // Auto-generate filename based on language
            self.auto_generate_filename(language, generated_code)?
        } else {
            return Ok(());
        };
        
        // Check if file exists and handle accordingly
        if output_path.exists() && !force {
            if modify {
                // Modify existing file (append or intelligent merge)
                return self.modify_existing_file(&output_path, generated_code, language).await;
            } else {
                println!("❌ File already exists: {:?}", output_path);
                println!("   Use --force to overwrite or --modify to append/merge");
                return Ok(());
            }
        }
        
        // Create directory structure if needed
        if create_dirs {
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
                info!("Created directory structure: {:?}", parent);
            }
        }
        
        // Write the file
        match std::fs::write(&output_path, generated_code) {
            Ok(()) => {
                println!("\n📁 Code saved to: {:?}", output_path);
                println!("   Language: {}", language);
                println!("   Size: {} bytes", generated_code.len());
                
                // Attempt to validate the generated code
                if let Err(e) = self.validate_generated_code(&output_path, language).await {
                    println!("   ⚠️  Validation warning: {}", e);
                } else {
                    println!("   ✅ Code appears valid");
                }
            },
            Err(e) => {
                if !create_dirs && e.kind() == std::io::ErrorKind::NotFound {
                    println!("❌ Directory doesn't exist. Use --create-dirs to create it automatically.");
                } else {
                    println!("❌ Failed to write file: {}", e);
                }
                return Err(anyhow::anyhow!("Failed to write file: {}", e));
            }
        }
        
        Ok(())
    }
    
    /// Auto-generate filename based on language and code content
    fn auto_generate_filename(&self, language: &str, code: &str) -> Result<PathBuf> {
        let extension = match language.to_lowercase().as_str() {
            "rust" => "rs",
            "python" => "py",
            "javascript" => "js",
            "typescript" => "ts",
            "go" => "go",
            "java" => "java",
            "cpp" | "c++" => "cpp",
            "c" => "c",
            "csharp" | "c#" => "cs",
            "php" => "php",
            "ruby" => "rb",
            "swift" => "swift",
            "kotlin" => "kt",
            "dart" => "dart",
            "scala" => "scala",
            "shell" | "bash" => "sh",
            "sql" => "sql",
            "html" => "html",
            "css" => "css",
            "yaml" | "yml" => "yml",
            "json" => "json",
            "toml" => "toml",
            "xml" => "xml",
            "markdown" => "md",
            _ => "txt",
        };
        
        // Try to extract a function/class name for the filename
        let base_name = self.extract_name_from_code(code, language)
            .unwrap_or_else(|| "generated_code".to_string());
        
        // Ensure filename is valid
        let safe_name = base_name
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
            .collect::<String>();
        
        let filename = format!("{}.{}", safe_name, extension);
        Ok(PathBuf::from(filename))
    }
    
    /// Extract function/class name from generated code
    fn extract_name_from_code(&self, code: &str, language: &str) -> Option<String> {
        match language.to_lowercase().as_str() {
            "rust" => {
                // Look for 'fn function_name' or 'struct StructName'
                for line in code.lines() {
                    if let Some(name) = self.extract_rust_name(line) {
                        return Some(name);
                    }
                }
            },
            "python" => {
                // Look for 'def function_name' or 'class ClassName'
                for line in code.lines() {
                    if let Some(name) = self.extract_python_name(line) {
                        return Some(name);
                    }
                }
            },
            "javascript" | "typescript" => {
                // Look for 'function functionName' or 'class ClassName'
                for line in code.lines() {
                    if let Some(name) = self.extract_js_name(line) {
                        return Some(name);
                    }
                }
            },
            _ => {}
        }
        None
    }
    
    fn extract_rust_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") {
            if let Some(start) = trimmed.find("fn ") {
                let after_fn = &trimmed[start + 3..].trim();
                if let Some(end) = after_fn.find('(') {
                    return Some(after_fn[..end].trim().to_string());
                }
            }
        } else if trimmed.starts_with("struct ") || trimmed.starts_with("pub struct ") {
            if let Some(start) = trimmed.find("struct ") {
                let after_struct = &trimmed[start + 7..].trim();
                if let Some(end) = after_struct.find(|c: char| c.is_whitespace() || c == '{') {
                    return Some(after_struct[..end].trim().to_string());
                }
            }
        }
        None
    }
    
    fn extract_python_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        if trimmed.starts_with("def ") {
            let after_def = &trimmed[4..].trim();
            if let Some(end) = after_def.find('(') {
                return Some(after_def[..end].trim().to_string());
            }
        } else if trimmed.starts_with("class ") {
            let after_class = &trimmed[6..].trim();
            if let Some(end) = after_class.find(|c: char| c.is_whitespace() || c == '(' || c == ':') {
                return Some(after_class[..end].trim().to_string());
            }
        }
        None
    }
    
    fn extract_js_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        if trimmed.starts_with("function ") {
            let after_fn = &trimmed[9..].trim();
            if let Some(end) = after_fn.find('(') {
                return Some(after_fn[..end].trim().to_string());
            }
        } else if trimmed.starts_with("class ") {
            let after_class = &trimmed[6..].trim();
            if let Some(end) = after_class.find(|c: char| c.is_whitespace() || c == '{') {
                return Some(after_class[..end].trim().to_string());
            }
        }
        None
    }
    
    /// Modify existing file by appending or intelligent merging
    async fn modify_existing_file(
        &self,
        file_path: &PathBuf,
        generated_code: &str,
        language: &str,
    ) -> Result<()> {
        let existing_content = std::fs::read_to_string(file_path)?;
        
        // Simple append for now - could be made more intelligent
        let separator = match language.to_lowercase().as_str() {
            "rust" | "javascript" | "typescript" | "java" | "c" | "cpp" | "csharp" => "\n\n// --- Generated Code ---\n",
            "python" => "\n\n# --- Generated Code ---\n",
            "shell" | "bash" => "\n\n# --- Generated Code ---\n",
            _ => "\n\n<!-- Generated Code -->\n",
        };
        
        let modified_content = format!("{}{}{}", existing_content, separator, generated_code);
        
        std::fs::write(file_path, modified_content)?;
        println!("\n📁 Code appended to: {:?}", file_path);
        println!("   Added {} bytes", generated_code.len());
        
        Ok(())
    }
    
    /// Validate generated code (basic syntax check)
    async fn validate_generated_code(
        &self,
        file_path: &PathBuf,
        language: &str,
    ) -> Result<()> {
        match language.to_lowercase().as_str() {
            "rust" => {
                // Try to check with rustc --check
                let output = tokio::process::Command::new("rustc")
                    .args(["--emit=metadata", "--crate-type=lib", file_path.to_str().unwrap()])
                    .output()
                    .await;
                
                match output {
                    Ok(result) => {
                        if !result.status.success() {
                            return Err(anyhow::anyhow!("Syntax errors detected"));
                        }
                    },
                    Err(_) => {
                        // rustc not available, skip validation
                        return Ok(());
                    }
                }
            },
            "python" => {
                // Try to compile with python -m py_compile
                let output = tokio::process::Command::new("python3")
                    .args(["-m", "py_compile", file_path.to_str().unwrap()])
                    .output()
                    .await;
                
                match output {
                    Ok(result) => {
                        if !result.status.success() {
                            return Err(anyhow::anyhow!("Syntax errors detected"));
                        }
                    },
                    Err(_) => {
                        // python not available, skip validation
                        return Ok(());
                    }
                }
            },
            _ => {
                // For other languages, we'll skip validation for now
                return Ok(());
            }
        }
        
        Ok(())
    }
    
    /// Start interactive conversational code generation mode
    async fn start_interactive_mode(
        &mut self,
        project_path: Option<PathBuf>,
        session_file: Option<PathBuf>,
        save_session_file: Option<PathBuf>,
    ) -> Result<()> {
        info!("Starting interactive mode");
        
        // Load or create session
        let session = if let Some(session_path) = session_file {
            info!("Loading session from: {:?}", session_path);
            match interactive::InteractiveSession::load_from_file(session_path) {
                Ok(session) => {
                    println!("✅ Session loaded successfully!");
                    session
                },
                Err(e) => {
                    println!("❌ Failed to load session: {}", e);
                    println!("Creating new session instead...");
                    interactive::InteractiveSession::new(project_path.clone())
                }
            }
        } else {
            interactive::InteractiveSession::new(project_path.clone())
        };
        
        // Analyze project context if needed
        let context = if let Some(path) = project_path {
            info!("Analyzing project context at: {:?}", path);
            let analysis_config = AnalysisConfig::default();
            match self.context_manager.analyze_codebase(path, analysis_config).await {
                Ok(ctx) => {
                    info!("Context analysis completed: {} files, {} symbols", ctx.files.len(), ctx.metadata.indexed_symbols);
                    Some(ctx)
                },
                Err(e) => {
                    info!("Could not analyze project context: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        // Create interactive manager and start
        let mut interactive_manager = interactive::InteractiveManager::new(
            session,
            self.code_generator.clone(),
            Some(self.agent_system.clone()),
            context,
        );
        
        // Start interactive mode
        interactive_manager.start().await?;
        
        // Save session if requested
        if let Some(save_path) = save_session_file {
            info!("Saving session to: {:?}", save_path);
            match interactive_manager.session().save_to_file(save_path.clone()) {
                Ok(()) => {
                    println!("✅ Session saved to: {:?}", save_path);
                },
                Err(e) => {
                    println!("❌ Failed to save session: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Spawn agent monitoring task
    fn spawn_agent_monitor(
        agent_system: Arc<AgentSystem>,
        ui_sender: tokio::sync::mpsc::UnboundedSender<crate::ui::UIEvent>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));
            
            loop {
                interval.tick().await;
                
                // Get real-time agent status updates
                let agents_info = agent_system.get_agents_info().await;
                
                for agent_info in agents_info {
                    let _ = ui_sender.send(crate::ui::UIEvent::AgentStatusUpdate {
                        agent_name: agent_info.name.clone(),
                        status: agent_info.status.clone(),
                        task: None, // TODO: Add current task info when available
                        priority: None, // TODO: Add priority info when available
                        progress: None, // TODO: Add progress tracking when available
                    });
                }
                
                // Send periodic system status
                let agent_statuses = agent_system.get_agent_statuses().await;
                if !agent_statuses.is_empty() {
                    let status_count = agent_statuses.len();
                    let active_count = agent_statuses.iter()
                        .filter(|(_, status)| !matches!(status, crate::agents::AgentStatus::Idle))
                        .count();
                    
                    if active_count > 0 {
                        let status_notification = crate::ui::notifications::Notification::info(
                            "Agent Activity".to_string(),
                            format!("{} of {} agents active", active_count, status_count),
                        );
                        let _ = ui_sender.send(crate::ui::UIEvent::Notification(status_notification));
                    }
                }
            }
        })
    }
    
    /// Spawn command processing task
    fn spawn_command_processor(
        interactive_manager: crate::interactive::InteractiveManager,
        mut command_rx: tokio::sync::mpsc::UnboundedReceiver<String>,
        ui_sender: tokio::sync::mpsc::UnboundedSender<crate::ui::UIEvent>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(command) = command_rx.recv().await {
                // Send user input to UI
                let _ = ui_sender.send(crate::ui::UIEvent::Output {
                    content: command.clone(),
                    block_type: "user".to_string(),
                });
                
                // Process command based on type
                let response = if command.starts_with("/") {
                    Self::process_system_command(&command[1..], &ui_sender).await
                } else {
                    Self::process_natural_language_command(&command, &interactive_manager, &ui_sender).await
                };
                
                // Send response to UI
                let _ = ui_sender.send(crate::ui::UIEvent::Output {
                    content: response,
                    block_type: "agent".to_string(),
                });
            }
        })
    }
    
    /// Spawn UI event handler task
    fn spawn_ui_event_handler(
        mut ui_rx: tokio::sync::mpsc::UnboundedReceiver<crate::ui::UIEvent>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(_event) = ui_rx.recv().await {
                // UI events are handled directly by the Application instance
                // This task is mainly for future extension of event handling
                // For now, we just consume events to prevent channel blocking
            }
        })
    }
    
    /// Process system commands (starting with /)
    async fn process_system_command(
        command: &str,
        ui_sender: &tokio::sync::mpsc::UnboundedSender<crate::ui::UIEvent>,
    ) -> String {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd = parts.get(0).unwrap_or(&"");
        
        match *cmd {
            "help" => {
                r#"Available Commands:
  /help     - Show this help message
  /status   - Show system status
  /clear    - Clear output
  /quit     - Exit dashboard

Natural Language Commands:
  - "generate a function to..."
  - "explain this code"
  - "optimize this algorithm"
  - "add tests for..."
  - "debug this issue"

Press Ctrl+C to exit at any time."#.to_string()
            }
            "status" => {
                "DevKit Dashboard Status: Active\nAgent System: Running\nUI: Connected".to_string()
            }
            "clear" => {
                // Send multiple newlines to clear output
                let _ = ui_sender.send(crate::ui::UIEvent::Output {
                    content: "\n\n\n\n\n\n\n\n\n\n".to_string(),
                    block_type: "system".to_string(),
                });
                "Output cleared".to_string()
            }
            "quit" => {
                let _ = ui_sender.send(crate::ui::UIEvent::Quit);
                "Shutting down dashboard...".to_string()
            }
            _ => format!("Unknown command: /{}. Type /help for available commands.", cmd),
        }
    }
    
    /// Process natural language commands through agents
    async fn process_natural_language_command(
        command: &str,
        _interactive_manager: &crate::interactive::InteractiveManager,
        ui_sender: &tokio::sync::mpsc::UnboundedSender<crate::ui::UIEvent>,
    ) -> String {
        // For now, we'll provide a simple response indicating the command was received
        // In a full implementation, this would route to the appropriate agent
        
        let _ = ui_sender.send(crate::ui::UIEvent::Notification(
            crate::ui::notifications::Notification::info(
                "Command Processing".to_string(),
                format!("Processing: {}", command),
            )
        ));
        
        // Simulate processing time
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        // Classify and route command (simplified version)
        if command.to_lowercase().contains("generate") || command.to_lowercase().contains("create") {
            "🔧 Code generation request received. This would be processed by the CodeGeneration agent.".to_string()
        } else if command.to_lowercase().contains("explain") || command.to_lowercase().contains("what") {
            "📚 Code explanation request received. This would be processed by the Analysis agent.".to_string()
        } else if command.to_lowercase().contains("debug") || command.to_lowercase().contains("fix") {
            "🐛 Debug request received. This would be processed by the Debugging agent.".to_string()
        } else if command.to_lowercase().contains("optimize") {
            "⚡ Optimization request received. This would be processed by the Analysis agent.".to_string()
        } else {
            "💬 General query received. Processing with available agents...".to_string()
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Initialize logging
    let log_level = match cli.log_level.as_str() {
        "trace" => tracing::Level::TRACE,
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };
    
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();
    
    // Change working directory if specified
    if let Some(working_dir) = cli.working_dir {
        std::env::set_current_dir(&working_dir)
            .map_err(|e| anyhow::anyhow!("Failed to change directory to {:?}: {}", working_dir, e))?;
        info!("Changed working directory to: {:?}", working_dir);
    }
    
    // Initialize the devkit development environment
    let mut env = DevKit::new(cli.config).await?;
    
    // Handle commands
    match cli.command {
        Some(Commands::Start { project }) => {
            env.start_interactive(project).await?
        },
        Some(Commands::Init { path }) => {
            env.init_project(path).await?
        },
        Some(Commands::Config { show, reset, export, import }) => {
            env.handle_config_command(show, reset, export, import).await?
        },
        Some(Commands::Analyze { path, output, dependencies }) => {
            env.analyze_codebase(path, output, dependencies).await?
        },
        Some(Commands::Generate { prompt, file, language, write_to_project, force, create_dirs, modify }) => {
            env.generate_code(prompt, file, language, write_to_project, force, create_dirs, modify).await?
        },
        Some(Commands::Interactive { project, session, save_session }) => {
            env.start_interactive_mode(project, session, save_session).await?
        },
        Some(Commands::Version) => {
            println!("devkit v{}", env!("CARGO_PKG_VERSION"));
            println!("From Infer No Dev - AI-powered development toolkit");
        },
        None => {
            // Default to starting interactive mode
            env.start_interactive(None).await?
        }
    }
    
    Ok(())
}
