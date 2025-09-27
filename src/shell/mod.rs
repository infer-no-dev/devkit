//! Cross-platform shell integration for zsh, bash, fish, and PowerShell.
//!
//! This module provides seamless integration with different shell environments,
//! allowing the agentic development environment to execute commands and
//! interact with the user's preferred shell.

pub mod bash;
pub mod detector;
pub mod executor;
pub mod fish;
pub mod powershell;
pub mod zsh;

use std::collections::HashMap;

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
    #[error("Shell detection failed: {0}")]
    DetectionFailed(String),

    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Unsupported shell: {0}")]
    UnsupportedShell(String),

    #[error("Timeout occurred while executing command")]
    Timeout,

    #[error("Permission denied: {0}")]
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

        let output = self
            .executor
            .execute(command, &self.current_shell, &config, &self.environment)
            .await?;

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
            ShellType::PowerShell => {
                format!("Get-Command {} -ErrorAction SilentlyContinue", command)
            }
            _ => format!("command -v {}", command),
        };

        self.execute_command(&check_command, None)
            .await
            .map(|result| result.exit_code == 0)
            .unwrap_or(false)
    }

    /// Execute a command operation
    pub async fn execute_operation(
        &self,
        operation: CommandOperation,
        config: Option<ShellConfig>,
    ) -> Result<CommandResult, ShellError> {
        let command = self.get_command_syntax(operation);
        self.execute_command(&command, config).await
    }

    // Project-specific high-level operations

    /// Initialize a git repository
    pub async fn git_init(&self, path: Option<&str>) -> Result<CommandResult, ShellError> {
        let command = match path {
            Some(p) => format!("git init '{}'", p),
            None => "git init".to_string(),
        };
        self.execute_command(&command, None).await
    }

    /// Add files to git staging
    pub async fn git_add(&self, files: &[&str]) -> Result<CommandResult, ShellError> {
        let files_str = if files.is_empty() {
            ".".to_string()
        } else {
            files.join(" ")
        };
        let command = format!("git add {}", files_str);
        self.execute_command(&command, None).await
    }

    /// Check git repository status
    pub async fn git_status(&self) -> Result<CommandResult, ShellError> {
        self.execute_operation(CommandOperation::GitStatus, None)
            .await
    }

    /// Initialize a new Cargo project
    pub async fn cargo_new(&self, name: &str, is_lib: bool) -> Result<CommandResult, ShellError> {
        let command = if is_lib {
            format!("cargo new --lib '{}'", name)
        } else {
            format!("cargo new '{}'", name)
        };
        self.execute_command(&command, None).await
    }

    /// Add a dependency to Cargo.toml
    pub async fn cargo_add(&self, dependency: &str) -> Result<CommandResult, ShellError> {
        let command = format!("cargo add '{}'", dependency);
        self.execute_command(&command, None).await
    }

    /// Create a directory structure
    pub async fn create_directory_structure(
        &self,
        paths: &[&str],
    ) -> Result<Vec<CommandResult>, ShellError> {
        let mut results = Vec::new();

        for path in paths {
            let operation = CommandOperation::CreateDirectory {
                path: path.to_string(),
                recursive: true,
            };
            let result = self.execute_operation(operation, None).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Write content to a file
    pub async fn write_file_content(
        &self,
        path: &str,
        content: &str,
    ) -> Result<CommandResult, ShellError> {
        let operation = CommandOperation::WriteFile {
            path: path.to_string(),
            content: content.to_string(),
        };
        self.execute_operation(operation, None).await
    }

    /// Read content from a file
    pub async fn read_file_content(&self, path: &str) -> Result<String, ShellError> {
        let operation = CommandOperation::ReadFile {
            path: path.to_string(),
        };
        let result = self.execute_operation(operation, None).await?;

        if result.exit_code == 0 {
            Ok(result.stdout)
        } else {
            Err(ShellError::ExecutionFailed(format!(
                "Failed to read file '{}': {}",
                path, result.stderr
            )))
        }
    }

    /// Setup a new project with basic structure
    pub async fn setup_project(
        &self,
        project_type: &str,
        name: &str,
    ) -> Result<Vec<CommandResult>, ShellError> {
        let mut results = Vec::new();

        match project_type {
            "rust" => {
                // Create new Cargo project
                let result = self.cargo_new(name, false).await?;
                results.push(result);

                // Initialize git if not already initialized
                if self.command_exists("git").await {
                    if let Ok(git_result) = self.git_init(Some(name)).await {
                        results.push(git_result);
                    }
                }
            }
            "node" | "npm" => {
                // Create directory
                let mkdir_op = CommandOperation::CreateDirectory {
                    path: name.to_string(),
                    recursive: true,
                };
                let result = self.execute_operation(mkdir_op, None).await?;
                results.push(result);

                // Change to directory and run npm init
                let mut config = ShellConfig::default();
                config.working_directory = Some(std::path::PathBuf::from(name));

                let npm_result = self.execute_command("npm init -y", Some(config)).await?;
                results.push(npm_result);
            }
            _ => {
                return Err(ShellError::UnsupportedShell(format!(
                    "Unknown project type: {}",
                    project_type
                )));
            }
        }

        Ok(results)
    }

    /// Get shell-specific command syntax
    pub fn get_command_syntax(&self, operation: CommandOperation) -> String {
        match (&self.current_shell, operation) {
            // Basic operations
            (ShellType::PowerShell, CommandOperation::ListFiles) => "Get-ChildItem".to_string(),
            (_, CommandOperation::ListFiles) => "ls".to_string(),
            (ShellType::PowerShell, CommandOperation::ChangeDirectory(path)) => {
                format!("Set-Location '{}'", path)
            }
            (_, CommandOperation::ChangeDirectory(path)) => format!("cd '{}'", path),
            (ShellType::PowerShell, CommandOperation::Echo(text)) => {
                format!("Write-Output '{}'", text)
            }
            (_, CommandOperation::Echo(text)) => format!("echo '{}'", text),

            // Git operations
            (_, CommandOperation::GitClone { url, directory }) => match directory {
                Some(dir) => format!("git clone '{}' '{}'", url, dir),
                None => format!("git clone '{}'", url),
            },
            (_, CommandOperation::GitCommit { message }) => {
                format!("git commit -m '{}'", message)
            }
            (_, CommandOperation::GitPush) => "git push".to_string(),
            (_, CommandOperation::GitPull) => "git pull".to_string(),
            (_, CommandOperation::GitStatus) => "git status".to_string(),
            (_, CommandOperation::GitBranch { name }) => match name {
                Some(branch) => format!("git checkout -b '{}'", branch),
                None => "git branch".to_string(),
            },

            // Cargo operations
            (_, CommandOperation::CargoBuild { release }) => {
                if release {
                    "cargo build --release".to_string()
                } else {
                    "cargo build".to_string()
                }
            }
            (_, CommandOperation::CargoTest { package }) => match package {
                Some(pkg) => format!("cargo test -p '{}'", pkg),
                None => "cargo test".to_string(),
            },
            (_, CommandOperation::CargoRun { args }) => {
                if args.is_empty() {
                    "cargo run".to_string()
                } else {
                    format!("cargo run -- {}", args.join(" "))
                }
            }
            (_, CommandOperation::CargoCheck) => "cargo check".to_string(),

            // NPM operations
            (_, CommandOperation::NpmInstall) => "npm install".to_string(),
            (_, CommandOperation::NpmRun { script }) => format!("npm run {}", script),
            (_, CommandOperation::NpmTest) => "npm test".to_string(),

            // File operations
            (ShellType::PowerShell, CommandOperation::CreateDirectory { path, recursive }) => {
                if recursive {
                    format!("New-Item -ItemType Directory -Path '{}' -Force", path)
                } else {
                    format!("New-Item -ItemType Directory -Path '{}'", path)
                }
            }
            (_, CommandOperation::CreateDirectory { path, recursive }) => {
                if recursive {
                    format!("mkdir -p '{}'", path)
                } else {
                    format!("mkdir '{}'", path)
                }
            }
            (ShellType::PowerShell, CommandOperation::CreateFile { path, content }) => {
                format!("Set-Content -Path '{}' -Value '{}'", path, content)
            }
            (_, CommandOperation::CreateFile { path, content }) => {
                format!("echo '{}' > '{}'", content, path)
            }
            (ShellType::PowerShell, CommandOperation::ReadFile { path }) => {
                format!("Get-Content '{}'", path)
            }
            (_, CommandOperation::ReadFile { path }) => format!("cat '{}'", path),
            (ShellType::PowerShell, CommandOperation::WriteFile { path, content }) => {
                format!("Set-Content -Path '{}' -Value '{}'", path, content)
            }
            (_, CommandOperation::WriteFile { path, content }) => {
                format!("echo '{}' > '{}'", content, path)
            }
            (ShellType::PowerShell, CommandOperation::DeleteFile { path }) => {
                format!("Remove-Item '{}'", path)
            }
            (_, CommandOperation::DeleteFile { path }) => format!("rm '{}'", path),
            (ShellType::PowerShell, CommandOperation::CopyFile { from, to }) => {
                format!("Copy-Item '{}' '{}'", from, to)
            }
            (_, CommandOperation::CopyFile { from, to }) => format!("cp '{}' '{}'", from, to),
            (ShellType::PowerShell, CommandOperation::MoveFile { from, to }) => {
                format!("Move-Item '{}' '{}'", from, to)
            }
            (_, CommandOperation::MoveFile { from, to }) => format!("mv '{}' '{}'", from, to),

            // System operations
            (ShellType::PowerShell, CommandOperation::Which { command }) => {
                format!("Get-Command '{}'", command)
            }
            (_, CommandOperation::Which { command }) => format!("which '{}'", command),
            (ShellType::PowerShell, CommandOperation::Ps) => "Get-Process".to_string(),
            (_, CommandOperation::Ps) => "ps aux".to_string(),
            (ShellType::PowerShell, CommandOperation::Kill { pid }) => {
                format!("Stop-Process -Id {}", pid)
            }
            (_, CommandOperation::Kill { pid }) => format!("kill {}", pid),
            (ShellType::PowerShell, CommandOperation::Env) => "Get-ChildItem Env:".to_string(),
            (_, CommandOperation::Env) => "env".to_string(),
            (ShellType::PowerShell, CommandOperation::Export { key, value }) => {
                format!("$env:{} = '{}'", key, value)
            }
            (_, CommandOperation::Export { key, value }) => {
                format!("export {}='{}'", key, value)
            }
        }
    }
}

/// Common shell operations
#[derive(Debug, Clone)]
pub enum CommandOperation {
    ListFiles,
    ChangeDirectory(String),
    Echo(String),
    // Project management operations
    GitClone {
        url: String,
        directory: Option<String>,
    },
    GitCommit {
        message: String,
    },
    GitPush,
    GitPull,
    GitStatus,
    GitBranch {
        name: Option<String>,
    },
    // Build system operations
    CargoBuild {
        release: bool,
    },
    CargoTest {
        package: Option<String>,
    },
    CargoRun {
        args: Vec<String>,
    },
    CargoCheck,
    NpmInstall,
    NpmRun {
        script: String,
    },
    NpmTest,
    // File operations
    CreateDirectory {
        path: String,
        recursive: bool,
    },
    CreateFile {
        path: String,
        content: String,
    },
    ReadFile {
        path: String,
    },
    WriteFile {
        path: String,
        content: String,
    },
    DeleteFile {
        path: String,
    },
    CopyFile {
        from: String,
        to: String,
    },
    MoveFile {
        from: String,
        to: String,
    },
    // System operations
    Which {
        command: String,
    },
    Ps,
    Kill {
        pid: u32,
    },
    Env,
    Export {
        key: String,
        value: String,
    },
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
            ShellType::Bash => write!(f, "bash"),
            ShellType::Zsh => write!(f, "zsh"),
            ShellType::Fish => write!(f, "fish"),
            ShellType::PowerShell => write!(f, "powershell"),
            ShellType::Unknown(name) => write!(f, "unknown({})", name),
        }
    }
}
