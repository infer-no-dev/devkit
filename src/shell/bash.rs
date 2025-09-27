//! Bash shell integration.

/// Bash-specific shell operations
pub struct BashShell;

impl BashShell {
    pub fn new() -> Self {
        Self
    }

    pub fn get_prompt_command() -> String {
        "PS1='$ '".to_string()
    }

    pub fn get_history_command() -> String {
        "history".to_string()
    }
}
