//! Shell completion utilities

use crate::cli::commands::shell::{
    generate_bash_completion,
    generate_zsh_completion, 
    generate_fish_completion,
    generate_powershell_completion
};

/// Generate shell completion scripts
pub fn generate_completions(shell: &str) -> Result<String, String> {
    match shell.to_lowercase().as_str() {
        "bash" => Ok(generate_bash_completion()),
        "zsh" => Ok(generate_zsh_completion()),
        "fish" => Ok(generate_fish_completion()),
        "powershell" => Ok(generate_powershell_completion()),
        _ => Err(format!("Unsupported shell: {}. Supported: bash, zsh, fish, powershell", shell))
    }
}

/// List supported shells
pub fn supported_shells() -> Vec<&'static str> {
    vec!["bash", "zsh", "fish", "powershell"]
}
