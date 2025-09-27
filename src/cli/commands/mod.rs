//! CLI command implementations
//!
//! This module contains the implementation of all CLI commands for the
//! agentic development environment. Each command is implemented in a separate
//! module for better organization and maintainability.

pub mod agent;
pub mod analyze;
pub mod config;
pub mod demo;
pub mod generate;
pub mod init;
pub mod inspect;
pub mod interactive;
pub mod profile;
pub mod review;
pub mod shell;
pub mod status;
pub mod template;

/// Common utilities for command implementations
pub mod utils {
    // Module re-exports handled by individual command files
    use std::fs;
    use std::path::{Path, PathBuf};

    /// Check if a path exists and is accessible
    pub fn validate_path(path: &Path) -> Result<(), String> {
        if !path.exists() {
            return Err(format!("Path does not exist: {}", path.display()));
        }

        if path.is_dir() && fs::read_dir(path).is_err() {
            return Err(format!("Cannot access directory: {}", path.display()));
        }

        if path.is_file() && fs::read(path).is_err() {
            return Err(format!("Cannot read file: {}", path.display()));
        }

        Ok(())
    }

    /// Create directory if it doesn't exist
    pub fn ensure_directory(path: &Path) -> Result<(), String> {
        if !path.exists() {
            fs::create_dir_all(path)
                .map_err(|e| format!("Failed to create directory {}: {}", path.display(), e))?;
        } else if !path.is_dir() {
            return Err(format!(
                "Path exists but is not a directory: {}",
                path.display()
            ));
        }
        Ok(())
    }

    /// Get relative path from current directory
    pub fn get_relative_path(path: &Path) -> Result<PathBuf, String> {
        let current_dir = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        path.strip_prefix(&current_dir)
            .map(|p| p.to_path_buf())
            .or_else(|_| Ok(path.to_path_buf()))
    }

    /// Format file size in human readable format
    pub fn format_file_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

        if size == 0 {
            return "0 B".to_string();
        }

        let mut size = size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", size as u64, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }

    /// Format duration in human readable format
    pub fn format_duration(duration: std::time::Duration) -> String {
        let secs = duration.as_secs();

        if secs < 60 {
            format!("{}s", secs)
        } else if secs < 3600 {
            format!("{}m {}s", secs / 60, secs % 60)
        } else {
            format!("{}h {}m {}s", secs / 3600, (secs % 3600) / 60, secs % 60)
        }
    }

    /// Pretty print JSON with syntax highlighting
    pub fn format_json(value: &serde_json::Value) -> String {
        serde_json::to_string_pretty(value).unwrap_or_else(|_| "Invalid JSON".to_string())
    }

    /// Create a progress bar for long-running operations
    pub fn create_progress_bar(total: u64, message: &str) -> indicatif::ProgressBar {
        use indicatif::{ProgressBar, ProgressStyle};

        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-")
        );
        pb.set_message(message.to_string());
        pb
    }

    /// Check if user wants to continue (interactive prompt)
    pub fn confirm_action(message: &str, default: bool) -> Result<bool, String> {
        use std::io::{self, Write};

        let default_str = if default { "Y/n" } else { "y/N" };
        print!("{} [{}]: ", message, default_str);
        io::stdout()
            .flush()
            .map_err(|e| format!("Failed to flush stdout: {}", e))?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("Failed to read input: {}", e))?;

        let input = input.trim().to_lowercase();

        Ok(match input.as_str() {
            "y" | "yes" => true,
            "n" | "no" => false,
            "" => default,
            _ => {
                println!("Please enter 'y' or 'n'");
                confirm_action(message, default)?
            }
        })
    }

    /// Get user input with optional default value
    pub fn get_user_input(prompt: &str, default: Option<&str>) -> Result<String, String> {
        use std::io::{self, Write};

        if let Some(default) = default {
            print!("{} [{}]: ", prompt, default);
        } else {
            print!("{}: ", prompt);
        }

        io::stdout()
            .flush()
            .map_err(|e| format!("Failed to flush stdout: {}", e))?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("Failed to read input: {}", e))?;

        let input = input.trim().to_string();

        if input.is_empty() {
            if let Some(default) = default {
                Ok(default.to_string())
            } else {
                get_user_input(prompt, default)
            }
        } else {
            Ok(input)
        }
    }

    /// Detect project language from directory contents
    pub fn detect_project_language(dir: &Path) -> Option<String> {
        let entries = fs::read_dir(dir).ok()?;

        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();

            match file_name.as_ref() {
                "Cargo.toml" => return Some("rust".to_string()),
                "package.json" => return Some("javascript".to_string()),
                "requirements.txt" | "setup.py" | "pyproject.toml" => {
                    return Some("python".to_string())
                }
                "go.mod" => return Some("go".to_string()),
                "pom.xml" | "build.gradle" => return Some("java".to_string()),
                "Gemfile" => return Some("ruby".to_string()),
                "composer.json" => return Some("php".to_string()),
                "CMakeLists.txt" | "Makefile" => return Some("cpp".to_string()),
                _ => continue,
            }
        }

        None
    }
}
