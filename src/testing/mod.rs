//! Testing utilities and common helpers for the agentic development environment.

use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use tempfile::TempDir;
use tokio::sync::mpsc;

pub mod mocks;
pub mod fixtures;
pub mod test_utils;

// Re-export test utilities for easier access
pub use test_utils::*;

/// Test utilities for creating temporary test environments
pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub workspace_path: PathBuf,
    pub config_path: PathBuf,
}

impl TestEnvironment {
    /// Create a new test environment with temporary directories
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let workspace_path = temp_dir.path().join("workspace");
        let config_path = temp_dir.path().join("config");
        
        fs::create_dir_all(&workspace_path)?;
        fs::create_dir_all(&config_path)?;
        
        Ok(Self {
            temp_dir,
            workspace_path,
            config_path,
        })
    }
    
    /// Create a sample project structure
    pub fn create_sample_project(&self) -> Result<(), Box<dyn std::error::Error>> {
        let src_dir = self.workspace_path.join("src");
        fs::create_dir_all(&src_dir)?;
        
        // Create main.rs
        fs::write(
            src_dir.join("main.rs"),
            r#"fn main() {
    println!("Hello, world!");
}"#,
        )?;
        
        // Create lib.rs
        fs::write(
            src_dir.join("lib.rs"),
            r#"pub mod utils;

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}"#,
        )?;
        
        // Create utils.rs
        fs::write(
            src_dir.join("utils.rs"),
            r#"use std::collections::HashMap;

pub fn process_data(data: &str) -> HashMap<String, i32> {
    let mut map = HashMap::new();
    for line in data.lines() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() == 2 {
            if let Ok(value) = parts[1].parse::<i32>() {
                map.insert(parts[0].to_string(), value);
            }
        }
    }
    map
}

pub fn format_output(data: &HashMap<String, i32>) -> String {
    let mut items: Vec<_> = data.iter().collect();
    items.sort_by_key(|&(k, _)| k);
    
    let mut output = String::new();
    for (key, value) in items {
        output.push_str(&format!("{}: {}\n", key, value));
    }
    output
}"#,
        )?;
        
        // Create Cargo.toml
        fs::write(
            self.workspace_path.join("Cargo.toml"),
            r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
"#,
        )?;
        
        // Create README.md
        fs::write(
            self.workspace_path.join("README.md"),
            r#"# Test Project

This is a test project for the agentic development environment.

## Features

- Basic arithmetic operations
- Data processing utilities
- Command-line interface

## Usage

```bash
cargo run
```
"#,
        )?;
        
        Ok(())
    }
    
    /// Create a test git repository
    pub fn create_git_repo(&self) -> Result<(), Box<dyn std::error::Error>> {
        use std::process::Command;
        
        let output = Command::new("git")
            .args(&["init"])
            .current_dir(&self.workspace_path)
            .output()?;
        
        if !output.status.success() {
            return Err("Failed to initialize git repository".into());
        }
        
        // Configure git user for testing
        Command::new("git")
            .args(&["config", "user.name", "Test User"])
            .current_dir(&self.workspace_path)
            .output()?;
            
        Command::new("git")
            .args(&["config", "user.email", "test@example.com"])
            .current_dir(&self.workspace_path)
            .output()?;
        
        // Create initial commit
        Command::new("git")
            .args(&["add", "."])
            .current_dir(&self.workspace_path)
            .output()?;
            
        Command::new("git")
            .args(&["commit", "-m", "Initial commit"])
            .current_dir(&self.workspace_path)
            .output()?;
        
        Ok(())
    }
    
    /// Get the workspace path
    pub fn workspace_path(&self) -> &Path {
        &self.workspace_path
    }
    
    /// Get the config path
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }
    
    /// Create a test configuration file
    pub fn create_test_config(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_file = self.config_path.join("test_config.toml");
        let config_content = r#"
[general]
log_level = "debug"
auto_save = true
backup_enabled = false
telemetry_enabled = false

[agents]
max_concurrent_agents = 3
agent_timeout_seconds = 30
default_agent_priority = "normal"

[agents.notification_settings]
enabled = true
sound_enabled = false
desktop_notifications = false
auto_dismiss_timeout = 3000

[codegen]
[codegen.default_style]
indentation = "spaces"
indent_size = 4
line_length = 80
naming_convention = "snake_case"
include_comments = true
include_type_hints = true

[codegen.ai_model_settings]
default_model = "test-model"
context_window_size = 1024
temperature = 0.5
max_tokens = 500

[shell]
command_timeout = 10
history_enabled = true

[ui]
theme = "dark"
color_scheme = "dark"
font_size = 12
show_line_numbers = true
show_timestamps = true
auto_scroll = true

[ui.panel_layout]
output_panel_percentage = 60
agent_panel_percentage = 40
notification_panel_height = 3
input_panel_height = 2
"#;
        
        fs::write(&config_file, config_content)?;
        Ok(config_file)
    }
}

/// Test assertion helpers
pub struct TestAssertions;

impl TestAssertions {
    /// Assert that two strings match after normalizing whitespace
    pub fn assert_strings_match_normalized(expected: &str, actual: &str) {
        let normalize = |s: &str| {
            s.lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join("\n")
        };
        
        assert_eq!(normalize(expected), normalize(actual));
    }
    
    /// Assert that a file exists and has expected content
    pub fn assert_file_content(path: &Path, expected_content: &str) -> Result<(), Box<dyn std::error::Error>> {
        assert!(path.exists(), "File does not exist: {}", path.display());
        let actual_content = fs::read_to_string(path)?;
        Self::assert_strings_match_normalized(expected_content, &actual_content);
        Ok(())
    }
    
    /// Assert that a directory contains expected files
    pub fn assert_directory_contains(dir: &Path, expected_files: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        assert!(dir.is_dir(), "Path is not a directory: {}", dir.display());
        
        let mut actual_files = Vec::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    actual_files.push(name.to_string());
                }
            }
        }
        
        for expected_file in expected_files {
            assert!(
                actual_files.iter().any(|f| f == expected_file),
                "Expected file not found: {}",
                expected_file
            );
        }
        
        Ok(())
    }
}

/// Channel utilities for testing async components
pub struct TestChannels;

impl TestChannels {
    /// Create a test channel pair for agent communication
    pub fn create_agent_channels<T>() -> (mpsc::UnboundedSender<T>, mpsc::UnboundedReceiver<T>) {
        mpsc::unbounded_channel()
    }
    
    /// Create bounded channels for testing backpressure
    pub fn create_bounded_channels<T>(capacity: usize) -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
        mpsc::channel(capacity)
    }
}

/// Time utilities for testing
pub struct TestTime;

impl TestTime {
    /// Create a mock system time for testing
    pub fn mock_system_time() -> std::time::SystemTime {
        std::time::UNIX_EPOCH + std::time::Duration::from_secs(1234567890)
    }
    
    /// Wait for a condition to be true within a timeout
    pub async fn wait_for_condition<F>(mut condition: F, timeout: std::time::Duration) -> bool
    where
        F: FnMut() -> bool,
    {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            if condition() {
                return true;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        false
    }
}

/// Logging utilities for tests
pub struct TestLogging;

impl TestLogging {
    /// Initialize test logging
    pub fn init() {
        // Simple test logging initialization
        let _ = env_logger::try_init();
    }
    
    /// Initialize tracing for async tests
    pub fn init_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_test_writer()
            .try_init();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_environment_creation() {
        let env = TestEnvironment::new().expect("Failed to create test environment");
        assert!(env.workspace_path.exists());
        assert!(env.config_path.exists());
    }
    
    #[tokio::test]
    async fn test_sample_project_creation() {
        let env = TestEnvironment::new().expect("Failed to create test environment");
        env.create_sample_project().expect("Failed to create sample project");
        
        TestAssertions::assert_directory_contains(
            &env.workspace_path.join("src"),
            &["main.rs", "lib.rs", "utils.rs"]
        ).expect("Sample project files not created");
    }
    
    #[test]
    fn test_string_normalization() {
        let expected = "line1\nline2\nline3";
        let actual = "  line1  \n\n  line2  \n  line3  \n";
        TestAssertions::assert_strings_match_normalized(expected, actual);
    }
    
    #[tokio::test]
    async fn test_channel_creation() {
        let (tx, mut rx) = TestChannels::create_agent_channels::<String>();
        
        tx.send("test message".to_string()).expect("Failed to send message");
        let received = rx.recv().await.expect("Failed to receive message");
        assert_eq!(received, "test message");
    }
    
    #[tokio::test]
    async fn test_time_utilities() {
        let mut counter = 0;
        let condition = || {
            counter += 1;
            counter >= 3
        };
        
        let result = TestTime::wait_for_condition(
            condition,
            std::time::Duration::from_millis(100)
        ).await;
        
        assert!(result);
        assert_eq!(counter, 3);
    }
}
