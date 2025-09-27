//! Zsh shell integration.

/// Zsh-specific shell operations
pub struct ZshShell;

impl ZshShell {
    pub fn new() -> Self {
        Self
    }

    pub fn get_prompt_command() -> String {
        "PS1='%% '".to_string()
    }

    pub fn get_history_command() -> String {
        "history".to_string()
    }
}
