//! Fish shell integration.

/// Fish-specific shell operations
pub struct FishShell;

impl FishShell {
    pub fn new() -> Self {
        Self
    }
    
    pub fn get_prompt_command() -> String {
        "function fish_prompt; echo '$ '; end".to_string()
    }
    
    pub fn get_history_command() -> String {
        "history".to_string()
    }
}
