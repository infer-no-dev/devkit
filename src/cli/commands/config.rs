use crate::cli::{CliRunner, ConfigCommands};
use serde_json::Value;
use std::path::PathBuf;

pub async fn run(runner: &mut CliRunner, command: ConfigCommands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ConfigCommands::Show { path } => {
            show_config(runner, path.as_deref()).await
        }
        ConfigCommands::Get { path } => {
            get_config_value(runner, &path).await
        }
        ConfigCommands::Set { path, value } => {
            set_config_value(runner, &path, &value).await
        }
        ConfigCommands::Validate => {
            validate_config(runner).await
        }
        ConfigCommands::Environment { env } => {
            switch_environment(runner, &env).await
        }
        ConfigCommands::Environments => {
            list_environments(runner).await
        }
        ConfigCommands::Edit => {
            edit_config(runner).await
        }
        ConfigCommands::Reset { section } => {
            reset_config(runner, section.as_deref()).await
        }
        ConfigCommands::Export { output, format } => {
            export_config(runner, &output, &format).await
        }
        ConfigCommands::Import { input, merge } => {
            import_config(runner, &input, merge).await
        }
    }
}

async fn show_config(runner: &CliRunner, path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let config = runner.config_manager().config();
    
    if let Some(path) = path {
        if let Some(value) = runner.config_manager().get_value(path) {
            match runner.format() {
                crate::cli::OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&value)?);
                }
                crate::cli::OutputFormat::Yaml => {
                    println!("{}", serde_yaml::to_string(&value)?);
                }
                _ => {
                    println!("{}: {}", path, format_value(&value));
                }
            }
        } else {
            runner.print_error(&format!("Configuration path '{}' not found", path));
            return Err("Configuration path not found".into());
        }
    } else {
        match runner.format() {
            crate::cli::OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(config)?);
            }
            crate::cli::OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(config)?);
            }
            crate::cli::OutputFormat::Table => {
                print_config_table(runner, config);
            }
            _ => {
                print_config_text(runner, config);
            }
        }
    }
    
    Ok(())
}

async fn get_config_value(runner: &CliRunner, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(value) = runner.config_manager().get_value(path) {
        println!("{}", format_value(&value));
    } else {
        runner.print_error(&format!("Configuration path '{}' not found", path));
        return Err("Configuration path not found".into());
    }
    Ok(())
}

async fn set_config_value(runner: &mut CliRunner, path: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Try to parse the value as JSON first, then fall back to string
    let parsed_value = if let Ok(json_value) = serde_json::from_str::<Value>(value) {
        json_value
    } else {
        Value::String(value.to_string())
    };
    
    if let Err(e) = runner.config_manager_mut().set_value(path, parsed_value) {
        runner.print_error(&format!("Failed to set configuration: {}", e));
        return Err(e.into());
    }
    
    // Save the updated configuration
    if let Err(e) = runner.config_manager_mut().save() {
        runner.print_error(&format!("Failed to save configuration: {}", e));
        return Err(e.into());
    }
    
    runner.print_success(&format!("Set {} = {}", path, value));
    Ok(())
}

async fn validate_config(runner: &CliRunner) -> Result<(), Box<dyn std::error::Error>> {
    match runner.config_manager().validate() {
        Ok(()) => {
            runner.print_success("✅ Configuration is valid");
        }
        Err(e) => {
            runner.print_error(&format!("❌ Configuration validation failed: {}", e));
            return Err(e.into());
        }
    }
    Ok(())
}

async fn switch_environment(runner: &mut CliRunner, env: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Err(e) = runner.config_manager_mut().switch_environment(env) {
        runner.print_error(&format!("Failed to switch environment: {}", e));
        return Err(e.into());
    }
    
    runner.print_success(&format!("Switched to environment: {}", env));
    Ok(())
}

async fn list_environments(runner: &CliRunner) -> Result<(), Box<dyn std::error::Error>> {
    match runner.config_manager().available_environments() {
        Ok(environments) => {
            let current_env = runner.config_manager().environment();
            
            runner.print_info("Available environments:");
            for env in environments {
                let marker = if env == current_env { " (current)" } else { "" };
                println!("  • {}{}", env, marker);
            }
        }
        Err(e) => {
            runner.print_error(&format!("Failed to list environments: {}", e));
            return Err(e.into());
        }
    }
    Ok(())
}

async fn edit_config(_runner: &CliRunner) -> Result<(), Box<dyn std::error::Error>> {
    // For now, just provide instructions
    println!("To edit configuration manually, edit your config file:");
    println!("  ~/.config/agentic-dev-env/config.toml");
    println!("");
    println!("Or use the 'set' command to modify specific values:");
    println!("  agentic-dev config set codegen.ai_model_settings.default_model 'llama3.2:latest'");
    println!("  agentic-dev config set codegen.ai_model_settings.temperature 0.8");
    Ok(())
}

async fn reset_config(runner: &mut CliRunner, section: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(_section) = section {
        runner.print_warning("Section-specific reset not yet implemented");
        println!("Please use 'agentic-dev config reset' to reset entire configuration");
    } else {
        runner.config_manager_mut().reset_to_default();
        if let Err(e) = runner.config_manager_mut().save() {
            runner.print_error(&format!("Failed to save reset configuration: {}", e));
            return Err(e.into());
        }
        runner.print_success("Configuration reset to defaults");
    }
    Ok(())
}

async fn export_config(runner: &CliRunner, output: &PathBuf, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = runner.config_manager().config();
    
    let content = match format {
        "json" => serde_json::to_string_pretty(config)?,
        "yaml" => serde_yaml::to_string(config)?,
        "toml" => toml::to_string_pretty(config)?,
        _ => {
            runner.print_error(&format!("Unsupported export format: {}", format));
            return Err("Unsupported format".into());
        }
    };
    
    std::fs::write(output, content)?;
    runner.print_success(&format!("Configuration exported to {}", output.display()));
    Ok(())
}

async fn import_config(runner: &mut CliRunner, input: &PathBuf, merge: bool) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(input)?;
    
    if merge {
        runner.print_warning("Merge functionality not yet implemented");
        println!("Configuration will be completely replaced");
    }
    
    if let Err(e) = runner.config_manager_mut().import_from_json(&content) {
        runner.print_error(&format!("Failed to import configuration: {}", e));
        return Err(e.into());
    }
    
    if let Err(e) = runner.config_manager_mut().save() {
        runner.print_error(&format!("Failed to save imported configuration: {}", e));
        return Err(e.into());
    }
    
    runner.print_success(&format!("Configuration imported from {}", input.display()));
    Ok(())
}

fn format_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(format_value).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Object(_) => "<object>".to_string(),
        Value::Null => "null".to_string(),
    }
}

fn print_config_text(runner: &CliRunner, config: &crate::config::Config) {
    runner.print_info("📋 Current Configuration");
    println!("═══════════════════════════\n");
    
    println!("🔧 General Settings:");
    if let Some(workspace_path) = &config.general.workspace_path {
        println!("  • Workspace Path: {}", workspace_path.display());
    }
    println!("  • Log Level: {}", config.general.log_level);
    println!("  • Auto Save: {}", config.general.auto_save);
    println!("  • Backup Enabled: {}", config.general.backup_enabled);
    
    println!("\n🤖 Agent Settings:");
    println!("  • Max Concurrent Agents: {}", config.agents.max_concurrent_agents);
    println!("  • Agent Timeout: {}s", config.agents.agent_timeout_seconds);
    println!("  • Default Priority: {}", config.agents.default_agent_priority);
    
    println!("\n🎨 Code Generation:");
    println!("  • Indentation: {}", config.codegen.default_style.indentation);
    println!("  • Indent Size: {}", config.codegen.default_style.indent_size);
    println!("  • Line Length: {}", config.codegen.default_style.line_length);
    println!("  • Naming Convention: {}", config.codegen.default_style.naming_convention);
    
    println!("\n🧠 AI Model Settings:");
    println!("  • Default Provider: {}", config.codegen.ai_model_settings.default_provider);
    println!("  • Default Model: {}", config.codegen.ai_model_settings.default_model);
    println!("  • Temperature: {}", config.codegen.ai_model_settings.temperature);
    println!("  • Max Tokens: {}", config.codegen.ai_model_settings.max_tokens);
    println!("  • Context Window: {}", config.codegen.ai_model_settings.context_window_size);
    
    println!("\n🔗 Ollama Settings:");
    println!("  • Endpoint: {}", config.codegen.ai_model_settings.ollama.endpoint);
    println!("  • Timeout: {}s", config.codegen.ai_model_settings.ollama.timeout_seconds);
    println!("  • Max Retries: {}", config.codegen.ai_model_settings.ollama.max_retries);
    if let Some(default_model) = &config.codegen.ai_model_settings.ollama.default_model {
        println!("  • Default Model: {}", default_model);
    }
    
    println!("\n🖥️  Shell Settings:");
    println!("  • Command Timeout: {}s", config.shell.command_timeout);
    println!("  • History Enabled: {}", config.shell.history_enabled);
    
    println!("\n🎨 UI Settings:");
    println!("  • Theme: {}", config.ui.theme);
    println!("  • Color Scheme: {}", config.ui.color_scheme);
    println!("  • Show Line Numbers: {}", config.ui.show_line_numbers);
    println!("  • Show Timestamps: {}", config.ui.show_timestamps);
}

fn print_config_table(runner: &CliRunner, config: &crate::config::Config) {
    runner.print_info("📋 Configuration Summary");
    println!(
        "┌─────────────────────────────────┬────────────────────────────────────┐\n\
         │ Setting                         │ Value                              │\n\
         ├─────────────────────────────────┼────────────────────────────────────┤"
    );
    
    println!("│ {:<31} │ {:<34} │", "AI Provider", config.codegen.ai_model_settings.default_provider);
    println!("│ {:<31} │ {:<34} │", "AI Model", config.codegen.ai_model_settings.default_model);
    println!("│ {:<31} │ {:<34} │", "Ollama Endpoint", config.codegen.ai_model_settings.ollama.endpoint);
    println!("│ {:<31} │ {:<34} │", "Temperature", config.codegen.ai_model_settings.temperature);
    println!("│ {:<31} │ {:<34} │", "Max Tokens", config.codegen.ai_model_settings.max_tokens);
    println!("│ {:<31} │ {:<34} │", "Max Concurrent Agents", config.agents.max_concurrent_agents);
    println!("│ {:<31} │ {:<34} │", "Agent Timeout (s)", config.agents.agent_timeout_seconds);
    println!("│ {:<31} │ {:<34} │", "Log Level", config.general.log_level);
    println!("│ {:<31} │ {:<34} │", "Theme", config.ui.theme);
    
    println!(
        "└─────────────────────────────────┴────────────────────────────────────┘"
    );
}
