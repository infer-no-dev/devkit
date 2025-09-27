//! Command execution utilities for different shells.

use crate::shell::{ShellConfig, ShellError, ShellType};
use std::collections::HashMap;
use std::process::{Command, Output};
use tokio::time::{timeout, Duration};

/// Command executor that handles different shell types
#[derive(Debug)]
pub struct CommandExecutor;

impl CommandExecutor {
    pub fn new() -> Self {
        Self
    }

    /// Execute a command in the specified shell
    pub async fn execute(
        &self,
        command: &str,
        shell_type: &ShellType,
        config: &ShellConfig,
        environment: &HashMap<String, String>,
    ) -> Result<Output, ShellError> {
        let mut cmd = self.build_command(command, shell_type, config)?;

        // Set environment variables
        for (key, value) in environment {
            cmd.env(key, value);
        }

        // Set environment variables from config
        for (key, value) in &config.environment_variables {
            cmd.env(key, value);
        }

        // Set working directory if specified
        if let Some(working_dir) = &config.working_directory {
            cmd.current_dir(working_dir);
        }

        // Execute with timeout
        let timeout_duration = Duration::from_secs(config.timeout_seconds);

        let output = timeout(timeout_duration, async {
            tokio::task::spawn_blocking(move || cmd.output())
                .await
                .map_err(|e| ShellError::ExecutionFailed(format!("Task join error: {}", e)))?
                .map_err(|e| {
                    ShellError::ExecutionFailed(format!("Command execution failed: {}", e))
                })
        })
        .await
        .map_err(|_| ShellError::Timeout)??;

        Ok(output)
    }

    /// Build the appropriate command for the shell type
    fn build_command(
        &self,
        command: &str,
        shell_type: &ShellType,
        _config: &ShellConfig,
    ) -> Result<Command, ShellError> {
        match shell_type {
            ShellType::Bash => {
                let mut cmd = Command::new("bash");
                cmd.arg("-c").arg(command);
                Ok(cmd)
            }
            ShellType::Zsh => {
                let mut cmd = Command::new("zsh");
                cmd.arg("-c").arg(command);
                Ok(cmd)
            }
            ShellType::Fish => {
                let mut cmd = Command::new("fish");
                cmd.arg("-c").arg(command);
                Ok(cmd)
            }
            ShellType::PowerShell => {
                #[cfg(windows)]
                {
                    let mut cmd = Command::new("powershell");
                    cmd.arg("-Command").arg(command);
                    Ok(cmd)
                }
                #[cfg(not(windows))]
                {
                    // Try pwsh (cross-platform PowerShell) on non-Windows
                    let mut cmd = Command::new("pwsh");
                    cmd.arg("-Command").arg(command);
                    Ok(cmd)
                }
            }
            ShellType::Unknown(name) => Err(ShellError::UnsupportedShell(format!(
                "Unknown shell: {}",
                name
            ))),
        }
    }
}
