//! PowerShell integration.

/// PowerShell-specific shell operations
pub struct PowerShell;

impl PowerShell {
    pub fn new() -> Self {
        Self
    }

    pub fn get_prompt_command() -> String {
        "function prompt { 'PS> ' }".to_string()
    }

    pub fn get_history_command() -> String {
        "Get-History".to_string()
    }
}
