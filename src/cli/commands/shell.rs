use crate::cli::{CliRunner, ShellCommands};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::env;

pub async fn run(
    runner: &mut CliRunner,
    command: ShellCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ShellCommands::Completion { shell, output } => {
            generate_completion(&shell, output).await
        },
        ShellCommands::Install { shell } => {
            install_shell_integration(shell).await
        },
        ShellCommands::Status => {
            show_shell_status(runner).await
        }
    }
}

/// Generate shell completion scripts
async fn generate_completion(
    shell: &str,
    output: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let completion_content = match shell.to_lowercase().as_str() {
        "bash" => generate_bash_completion(),
        "zsh" => generate_zsh_completion(),
        "fish" => generate_fish_completion(),
        "powershell" => generate_powershell_completion(),
        _ => {
            eprintln!("❌ Unsupported shell: {}", shell);
            eprintln!("Supported shells: bash, zsh, fish, powershell");
            return Err("Unsupported shell".into());
        }
    };

    if let Some(output_path) = output {
        // Write to specified file
        fs::write(&output_path, completion_content)?;
        println!("✅ Completion script written to: {}", output_path.display());
    } else {
        // Write to stdout
        print!("{}", completion_content);
    }

    Ok(())
}

/// Install shell integration (completions and aliases)
async fn install_shell_integration(
    shell: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let detected_shell = match shell {
        Some(s) => s,
        None => detect_current_shell()?
    };

    println!("🔧 Installing shell integration for: {}", detected_shell);
    
    match detected_shell.to_lowercase().as_str() {
        "bash" => install_bash_integration().await?,
        "zsh" => install_zsh_integration().await?,
        "fish" => install_fish_integration().await?,
        _ => {
            eprintln!("❌ Unsupported shell: {}", detected_shell);
            return Err("Unsupported shell".into());
        }
    }

    println!("✅ Shell integration installed successfully!");
    println!("📝 Restart your terminal or run 'source ~/.{}rc' to activate", detected_shell);
    
    Ok(())
}

/// Show shell integration status
async fn show_shell_status(
    _runner: &CliRunner,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🐚 Shell Integration Status");
    println!("═══════════════════════════");
    
    let current_shell = detect_current_shell().unwrap_or_else(|_| "unknown".to_string());
    println!("📱 Current shell: {}", current_shell);
    
    let binary_path = env::current_exe()?;
    println!("📍 Binary location: {}", binary_path.display());
    
    // Check if binary is in PATH
    let in_path = env::var("PATH")
        .map(|path| {
            path.split(':')
                .any(|dir| binary_path.starts_with(dir))
        })
        .unwrap_or(false);
    
    if in_path {
        println!("✅ Binary is in PATH");
    } else {
        println!("⚠️ Binary is not in PATH");
    }
    
    // Check for existing shell integration
    check_shell_integration_status(&current_shell);
    
    println!();
    println!("💡 To install shell integration, run:");
    println!("   devkit shell install");
    
    Ok(())
}

/// Detect the current shell
fn detect_current_shell() -> Result<String, Box<dyn std::error::Error>> {
    if let Ok(shell_env) = env::var("SHELL") {
        let shell_name = shell_env
            .split('/')
            .last()
            .unwrap_or("unknown")
            .to_string();
        return Ok(shell_name);
    }
    
    // Fallback detection methods
    if env::var("ZSH_VERSION").is_ok() {
        return Ok("zsh".to_string());
    }
    if env::var("BASH_VERSION").is_ok() {
        return Ok("bash".to_string());
    }
    if env::var("FISH_VERSION").is_ok() {
        return Ok("fish".to_string());
    }
    
    Err("Could not detect current shell".into())
}

/// Check if shell integration is already installed
fn check_shell_integration_status(shell: &str) {
    let config_file = get_shell_config_file(shell);
    let completion_file = get_completion_file_path(shell);
    
    match config_file {
        Some(config_path) => {
            if config_path.exists() {
                let has_devkit_alias = fs::read_to_string(&config_path)
                    .map(|content| content.contains("devkit") || content.contains("ade"))
                    .unwrap_or(false);
                
                if has_devkit_alias {
                    println!("✅ Shell aliases found in {}", config_path.display());
                } else {
                    println!("⚠️ No devkit aliases found in {}", config_path.display());
                }
            } else {
                println!("⚠️ Shell config file not found: {}", config_path.display());
            }
        },
        None => {
            println!("⚠️ Unknown shell config file location");
        }
    }
    
    match completion_file {
        Some(comp_path) => {
            if comp_path.exists() {
                println!("✅ Completion script found at {}", comp_path.display());
            } else {
                println!("⚠️ Completion script not found: {}", comp_path.display());
            }
        },
        None => {
            println!("⚠️ Unknown completion file location");
        }
    }
}

/// Get shell config file path
fn get_shell_config_file(shell: &str) -> Option<PathBuf> {
    let home = env::var("HOME").ok()?;
    
    match shell {
        "bash" => Some(PathBuf::from(home).join(".bashrc")),
        "zsh" => Some(PathBuf::from(home).join(".zshrc")),
        "fish" => Some(PathBuf::from(home).join(".config/fish/config.fish")),
        _ => None,
    }
}

/// Get completion file path
fn get_completion_file_path(shell: &str) -> Option<PathBuf> {
    let home = env::var("HOME").ok()?;
    
    match shell {
        "bash" => Some(PathBuf::from(home).join(".local/share/bash-completion/completions/devkit")),
        "zsh" => Some(PathBuf::from(home).join(".local/share/zsh/site-functions/_devkit")),
        "fish" => Some(PathBuf::from(home).join(".config/fish/completions/devkit.fish")),
        _ => None,
    }
}

/// Install Bash integration
async fn install_bash_integration() -> Result<(), Box<dyn std::error::Error>> {
    let home = env::var("HOME")?;
    let bashrc_path = PathBuf::from(&home).join(".bashrc");
    let completion_dir = PathBuf::from(&home).join(".local/share/bash-completion/completions");
    
    // Create completion directory
    fs::create_dir_all(&completion_dir)?;
    
    // Install completion script
    let completion_path = completion_dir.join("devkit");
    fs::write(&completion_path, generate_bash_completion())?;
    println!("📝 Installed Bash completion to: {}", completion_path.display());
    
    // Install aliases
    install_shell_aliases(&bashrc_path, "bash").await?;
    
    Ok(())
}

/// Install Zsh integration
async fn install_zsh_integration() -> Result<(), Box<dyn std::error::Error>> {
    let home = env::var("HOME")?;
    let zshrc_path = PathBuf::from(&home).join(".zshrc");
    let completion_dir = PathBuf::from(&home).join(".local/share/zsh/site-functions");
    
    // Create completion directory
    fs::create_dir_all(&completion_dir)?;
    
    // Install completion script
    let completion_path = completion_dir.join("_devkit");
    fs::write(&completion_path, generate_zsh_completion())?;
    println!("📝 Installed Zsh completion to: {}", completion_path.display());
    
    // Install aliases
    install_shell_aliases(&zshrc_path, "zsh").await?;
    
    Ok(())
}

/// Install Fish integration
async fn install_fish_integration() -> Result<(), Box<dyn std::error::Error>> {
    let home = env::var("HOME")?;
    let fish_config_path = PathBuf::from(&home).join(".config/fish/config.fish");
    let completion_dir = PathBuf::from(&home).join(".config/fish/completions");
    
    // Create directories
    fs::create_dir_all(fish_config_path.parent().unwrap())?;
    fs::create_dir_all(&completion_dir)?;
    
    // Install completion script
    let completion_path = completion_dir.join("devkit.fish");
    fs::write(&completion_path, generate_fish_completion())?;
    println!("📝 Installed Fish completion to: {}", completion_path.display());
    
    // Install aliases
    install_shell_aliases(&fish_config_path, "fish").await?;
    
    Ok(())
}

/// Install shell aliases to config file
async fn install_shell_aliases(
    config_path: &PathBuf,
    shell: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let binary_path = env::current_exe()?;
    
    // Check if aliases are already installed
    let existing_content = if config_path.exists() {
        fs::read_to_string(config_path)?
    } else {
        String::new()
    };
    
    if existing_content.contains("# devkit shell integration") {
        println!("📝 Shell aliases already installed in: {}", config_path.display());
        return Ok(());
    }
    
    let aliases = generate_shell_aliases(&binary_path, shell);
    
    // Append aliases to config file
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(config_path)?;
    
    writeln!(file, "\n# devkit shell integration")?;
    writeln!(file, "{}", aliases)?;
    
    println!("📝 Installed shell aliases to: {}", config_path.display());
    
    Ok(())
}

/// Generate shell aliases
fn generate_shell_aliases(binary_path: &PathBuf, shell: &str) -> String {
    let binary_str = binary_path.to_string_lossy();
    
    match shell {
        "fish" => format!(
            "alias devkit '{}'
alias dk '{}'

function dk-analyze
    {} analyze $argv
end

function dk-generate
    {} generate $argv
end

function dk-status
    {} status $argv
end",
            binary_str, binary_str, binary_str, binary_str, binary_str
        ),
        _ => format!(
            "alias devkit='{}'
alias dk='{}'

dk-analyze() {{
    '{}' analyze \"$@\"
}}

dk-generate() {{
    '{}' generate \"$@\"
}}

dk-status() {{
    '{}' status \"$@\"
}}",
            binary_str, binary_str, binary_str, binary_str, binary_str
        ),
    }
}

/// Generate Bash completion script
pub fn generate_bash_completion() -> String {
    r#"#!/bin/bash

_devkit_completions() {
    local cur prev
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    
    case ${COMP_CWORD} in
        1)
            COMPREPLY=($(compgen -W "init analyze generate agent config profile template status shell demo interactive" -- ${cur}))
            ;;
        2)
            case ${prev} in
                analyze)
                    COMPREPLY=($(compgen -W "--format --output --quiet --include-tests --include-docs" -- ${cur}))
                    ;;
                generate)
                    COMPREPLY=($(compgen -W "--output --language --file --template --no-ai" -- ${cur}))
                    ;;
                shell)
                    COMPREPLY=($(compgen -W "completion install status" -- ${cur}))
                    ;;
                config)
                    COMPREPLY=($(compgen -W "--show --edit --reset --list" -- ${cur}))
                    ;;
            esac
            ;;
        *)
            case ${prev} in
                --format)
                    COMPREPLY=($(compgen -W "json yaml text table" -- ${cur}))
                    ;;
                --language)
                    COMPREPLY=($(compgen -W "rust python javascript typescript go java c cpp" -- ${cur}))
                    ;;
                --output|--file|--config)
                    COMPREPLY=($(compgen -f -- ${cur}))
                    ;;
                completion)
                    COMPREPLY=($(compgen -W "bash zsh fish powershell" -- ${cur}))
                    ;;
            esac
            ;;
    esac
}

complete -F _devkit_completions devkit
complete -F _devkit_completions dk
"#.to_string()
}

/// Generate Zsh completion script
pub fn generate_zsh_completion() -> String {
    r#"#compdef devkit dk

_devkit() {
    local state line
    typeset -A opt_args
    
    _arguments -C \
        '1: :->command' \
        '*: :->args' \
        && return 0
    
    case $state in
        command)
            _values 'command' \
                'init[Initialize a new project]' \
                'analyze[Analyze a codebase]' \
                'generate[Generate code from prompt]' \
                'agent[Manage agents]' \
                'config[Manage configuration]' \
                'profile[Profile performance]' \
                'template[Manage templates]' \
                'status[Show system status]' \
                'shell[Shell integration]' \
                'demo[Run demo workflows]' \
                'interactive[Start interactive mode]'
            ;;
        args)
            case $line[1] in
                analyze)
                    _arguments \
                        '--format[Output format]:format:(json yaml text table)' \
                        '--output[Output file]:path:_files' \
                        '--quiet[Suppress verbose output]' \
                        '--include-tests[Include test files]' \
                        '--include-docs[Include documentation]' \
                        '1:path:_files -/'
                    ;;
                generate)
                    _arguments \
                        '--output[Output file]:path:_files' \
                        '--language[Target language]:language:(rust python javascript typescript go java c cpp)' \
                        '--file[Target file]:path:_files' \
                        '--template[Use template]:template:()' \
                        '--no-ai[Disable AI generation]' \
                        '1:prompt:'
                    ;;
                shell)
                    _arguments \
                        '1: :(completion install status)'
                    ;;
                config)
                    _arguments \
                        '--show[Show configuration]' \
                        '--edit[Edit configuration]' \
                        '--reset[Reset configuration]' \
                        '--list[List configurations]'
                    ;;
            esac
            ;;
    esac
}

_devkit "$@"
"#.to_string()
}

/// Generate Fish completion script
pub fn generate_fish_completion() -> String {
    r#"# Completions for devkit

complete -c devkit -f
complete -c dk -f

# Subcommands
complete -c devkit -n "__fish_use_subcommand" -a "init" -d "Initialize a new project"
complete -c devkit -n "__fish_use_subcommand" -a "analyze" -d "Analyze a codebase"
complete -c devkit -n "__fish_use_subcommand" -a "generate" -d "Generate code from prompt"
complete -c devkit -n "__fish_use_subcommand" -a "agent" -d "Manage agents"
complete -c devkit -n "__fish_use_subcommand" -a "config" -d "Manage configuration"
complete -c devkit -n "__fish_use_subcommand" -a "profile" -d "Profile performance"
complete -c devkit -n "__fish_use_subcommand" -a "template" -d "Manage templates"
complete -c devkit -n "__fish_use_subcommand" -a "status" -d "Show system status"
complete -c devkit -n "__fish_use_subcommand" -a "shell" -d "Shell integration"
complete -c devkit -n "__fish_use_subcommand" -a "demo" -d "Run demo workflows"
complete -c devkit -n "__fish_use_subcommand" -a "interactive" -d "Start interactive mode"

# Options for analyze subcommand
complete -c devkit -n "__fish_seen_subcommand_from analyze" -l format -d "Output format" -xa "json yaml text table"
complete -c devkit -n "__fish_seen_subcommand_from analyze" -l output -d "Output file" -r
complete -c devkit -n "__fish_seen_subcommand_from analyze" -l quiet -d "Suppress verbose output"
complete -c devkit -n "__fish_seen_subcommand_from analyze" -l include-tests -d "Include test files"
complete -c devkit -n "__fish_seen_subcommand_from analyze" -l include-docs -d "Include documentation"

# Options for generate subcommand
complete -c devkit -n "__fish_seen_subcommand_from generate" -l output -d "Output file" -r
complete -c devkit -n "__fish_seen_subcommand_from generate" -l language -d "Target language" -xa "rust python javascript typescript go java c cpp"
complete -c devkit -n "__fish_seen_subcommand_from generate" -l file -d "Target file" -r
complete -c devkit -n "__fish_seen_subcommand_from generate" -l template -d "Use template"
complete -c devkit -n "__fish_seen_subcommand_from generate" -l no-ai -d "Disable AI generation"

# Options for shell subcommand
complete -c devkit -n "__fish_seen_subcommand_from shell" -n "__fish_use_subcommand" -a "completion" -d "Generate completion scripts"
complete -c devkit -n "__fish_seen_subcommand_from shell" -n "__fish_use_subcommand" -a "install" -d "Install shell integration"
complete -c devkit -n "__fish_seen_subcommand_from shell" -n "__fish_use_subcommand" -a "status" -d "Show integration status"

# Options for config subcommand
complete -c devkit -n "__fish_seen_subcommand_from config" -l show -d "Show configuration"
complete -c devkit -n "__fish_seen_subcommand_from config" -l edit -d "Edit configuration"
complete -c devkit -n "__fish_seen_subcommand_from config" -l reset -d "Reset configuration"
complete -c devkit -n "__fish_seen_subcommand_from config" -l list -d "List configurations"

# Copy completions for dk alias
complete -c dk -w devkit
"#.to_string()
}

/// Generate PowerShell completion script
pub fn generate_powershell_completion() -> String {
    r#"# PowerShell completion for devkit

Register-ArgumentCompleter -Native -CommandName devkit -ScriptBlock {
    param($commandName, $wordToComplete, $cursorPosition)
    
    $commandElements = $commandElements = $line.Split(' ', [StringSplitOptions]::RemoveEmptyEntries)
    $command = @()
    for ($i = 1; $i -lt $commandElements.Count; $i++) {
        $element = $commandElements[$i]
        if ($element -notlike '-*') {
            $command += $element
        }
    }
    
    $completions = @()
    
    if ($command.Count -eq 0) {
        $completions += 'init', 'analyze', 'generate', 'agent', 'config', 'profile', 'template', 'status', 'shell', 'demo', 'interactive'
    }
    elseif ($command.Count -eq 1) {
        switch ($command[0]) {
            'analyze' {
                $completions += '--format', '--output', '--quiet', '--include-tests', '--include-docs'
            }
            'generate' {
                $completions += '--output', '--language', '--file', '--template', '--no-ai'
            }
            'shell' {
                $completions += 'completion', 'install', 'status'
            }
            'config' {
                $completions += '--show', '--edit', '--reset', '--list'
            }
        }
    }
    
    $completions | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
    }
}

# Also register for dk alias
Register-ArgumentCompleter -Native -CommandName dk -ScriptBlock {
    param($commandName, $wordToComplete, $cursorPosition)
    # Reuse devkit completion logic
    $line = $line -replace '^dk', 'devkit'
    # ... (same logic as above)
}
"#.to_string()
}
