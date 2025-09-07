//! Cross-platform shell integration for zsh, bash, fish, and PowerShell.
//!
//! This module provides seamless integration with different shell environments,
//! allowing the agentic development environment to execute commands and
//! interact with the user's preferred shell.

pub mod bash;
pub mod fish;
pub mod powershell;
pub mod zsh;
pub mod detector;
pub mod executor;

use std::collections::HashMap;
use std::process::Output;

/// Supported shell types
#[derive(Debug, Clone, PartialEq)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Unknown(String),
}

/// Shell integration manager
#[derive(Debug)]
pub struct ShellManager {
    current_shell: ShellType,
    executor: executor::CommandExecutor,
    environment: HashMap<String, String>,
}

/// Configuration for shell operations
#[derive(Debug, Clone)]
pub struct ShellConfig {
    pub shell_type: Option<ShellType>,
    pub working_directory: Option<std::path::PathBuf>,
    pub environment_variables: HashMap<String, String>,
    pub timeout_seconds: u64,
    pub capture_output: bool,
}

/// Result of a shell command execution
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time_ms: u64,
    pub command: String,
}

/// Errors that can occur during shell operations
#[derive(Debug, thiserror::Error)]
pub enum ShellError {
    #[error(\"Shell detection failed: {0}\")]
    DetectionFailed(String),
    
    #[error(\"Command execution failed: {0}\")]
    ExecutionFailed(String),
    
    #[error(\"Unsupported shell: {0}\")]
    UnsupportedShell(String),
    
    #[error(\"Timeout occurred while executing command\")]
    Timeout,
    
    #[error(\"Permission denied: {0}\")]
    PermissionDenied(String),
}

impl ShellManager {
    /// Create a new shell manager with automatic shell detection
    pub fn new() -> Result<Self, ShellError> {
        let detector = detector::ShellDetector::new();
        let current_shell = detector.detect_current_shell()?;
        
        Ok(Self {
            current_shell,
            executor: executor::CommandExecutor::new(),
            environment: std::env::vars().collect(),
        })
    }
    
    /// Create a shell manager for a specific shell type
    pub fn with_shell(shell_type: ShellType) -> Result<Self, ShellError> {
        Ok(Self {
            current_shell: shell_type,
            executor: executor::CommandExecutor::new(),
            environment: std::env::vars().collect(),
        })
    }
    
    /// Execute a command in the current shell
    pub async fn execute_command(
        &self,
        command: &str,
        config: Option<ShellConfig>,
    ) -> Result<CommandResult, ShellError> {
        let config = config.unwrap_or_default();
        let start_time = std::time::Instant::now();
        
        let output = self.executor.execute(
            command,
            &self.current_shell,
            &config,
            &self.environment,
        ).await?;
        
        let execution_time = start_time.elapsed();
        
        Ok(CommandResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            execution_time_ms: execution_time.as_millis() as u64,
            command: command.to_string(),
        })
    }
    
    /// Execute multiple commands in sequence
    pub async fn execute_commands(
        &self,
        commands: &[&str],
        config: Option<ShellConfig>,
    ) -> Result<Vec<CommandResult>, ShellError> {
        let mut results = Vec::new();
        
        for command in commands {
            let result = self.execute_command(command, config.clone()).await?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Get the current shell type
    pub fn current_shell(&self) -> &ShellType {
        &self.current_shell
    }
    
    /// Set environment variables for future commands
    pub fn set_environment_variable(&mut self, key: String, value: String) {
        self.environment.insert(key, value);
    }
    
    /// Get environment variables
    pub fn get_environment(&self) -> &HashMap<String, String> {
        &self.environment
    }
    
    /// Check if a command exists in the current environment
    pub async fn command_exists(&self, command: &str) -> bool {
        let check_command = match self.current_shell {
            ShellType::PowerShell => format!(\"Get-Command {} -ErrorAction SilentlyContinue\", command),
            _ => format!(\"command -v {}\", command),
        };
        
        self.execute_command(&check_command, None)
            .await
            .map(|result| result.exit_code == 0)
            .unwrap_or(false)
    }
    
    /// Get shell-specific command syntax
    pub fn get_command_syntax(&self, operation: CommandOperation) -> String {
        match (&self.current_shell, operation) {
            (ShellType::PowerShell, CommandOperation::ListFiles) => \"Get-ChildItem\".to_string(),
            (_, CommandOperation::ListFiles) => \"ls\".to_string(),
            (ShellType::PowerShell, CommandOperation::ChangeDirectory(path)) => {
                format!(\"Set-Location '{}'\", path)
            },
            (_, CommandOperation::ChangeDirectory(path)) => format!(\"cd '{}'\", path),
            (ShellType::PowerShell, CommandOperation::Echo(text)) => {
                format!(\"Write-Output '{}'\", text)
            },
            (_, CommandOperation::Echo(text)) => format!(\"echo '{}'\", text),
        }
    }
}

/// Common shell operations
#[derive(Debug, Clone)]
pub enum CommandOperation {
    ListFiles,
    ChangeDirectory(String),
    Echo(String),
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            shell_type: None,
            working_directory: None,
            environment_variables: HashMap::new(),
            timeout_seconds: 30,
            capture_output: true,
        }
    }
}

impl std::fmt::Display for ShellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellType::Bash => write!(f, \"bash\"),
            ShellType::Zsh => write!(f, \"zsh\"),
            ShellType::Fish => write!(f, \"fish\"),
            ShellType::PowerShell => write!(f, \"powershell\"),
            ShellType::Unknown(name) => write!(f, \"unknown({})\", name),
        }
    }
}
