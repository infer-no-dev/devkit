//! Configuration loading and saving utilities.

use crate::config::{Config, ConfigError};
use std::path::PathBuf;

/// Configuration loader/saver
#[derive(Debug)]
pub struct ConfigLoader;

impl ConfigLoader {
    pub fn new() -> Self {
        Self
    }
    
    pub fn load_from_file(&self, path: &PathBuf) -> Result<Config, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
    
    pub fn save_to_file(&self, config: &Config, path: &PathBuf) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(config)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
