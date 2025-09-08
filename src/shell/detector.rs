//! Shell detection utilities.

use crate::shell::{ShellType, ShellError};
use std::env;

/// Detector for identifying the current shell
pub struct ShellDetector;

impl ShellDetector {
    pub fn new() -> Self {
        Self
    }
    
    /// Detect the current shell from environment variables
    pub fn detect_current_shell(&self) -> Result<ShellType, ShellError> {
        // Try to detect from SHELL environment variable
        if let Ok(shell_path) = env::var("SHELL") {
            if let Some(shell_name) = std::path::Path::new(&shell_path).file_name() {
                if let Some(name) = shell_name.to_str() {
                    return Ok(match name {
                        "bash" => ShellType::Bash,
                        "zsh" => ShellType::Zsh,
                        "fish" => ShellType::Fish,
                        _ => ShellType::Unknown(name.to_string()),
                    });
                }
            }
        }
        
        // Try to detect from parent process on Unix-like systems
        #[cfg(unix)]
        {
            if let Ok(ppid) = self.get_parent_process_name() {
                return Ok(match ppid.as_str() {
                    "bash" => ShellType::Bash,
                    "zsh" => ShellType::Zsh,
                    "fish" => ShellType::Fish,
                    _ => ShellType::Unknown(ppid),
                });
            }
        }
        
        // Try to detect PowerShell on Windows
        #[cfg(windows)]
        {
            if env::var("PSModulePath").is_ok() {
                return Ok(ShellType::PowerShell);
            }
        }
        
        // Fallback to bash as default on Unix-like systems
        #[cfg(unix)]
        {
            Ok(ShellType::Bash)
        }
        
        // Fallback to PowerShell on Windows
        #[cfg(windows)]
        {
            Ok(ShellType::PowerShell)
        }
    }
    
    #[cfg(unix)]
    fn get_parent_process_name(&self) -> Result<String, ShellError> {
        use std::process::Command;
        
        let output = Command::new("ps")
            .args(&["-o", "comm=", "-p", &std::process::id().to_string()])
            .output()
            .map_err(|e| ShellError::DetectionFailed(format!("Failed to execute ps: {}", e)))?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let process_name = stdout.trim().to_string();
            if !process_name.is_empty() {
                return Ok(process_name);
            }
        }
        
        Err(ShellError::DetectionFailed("Could not determine parent process".to_string()))
    }
}
