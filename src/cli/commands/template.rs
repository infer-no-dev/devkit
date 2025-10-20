use crate::cli::{CliRunner, TemplateCommands};
use crate::codegen::templates::{Template, TemplateManager, TemplateVariable};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub async fn run(
    runner: &mut CliRunner,
    command: TemplateCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut template_manager = TemplateManager::new()?;

    match command {
        TemplateCommands::List { language } => {
            list_templates(runner, &template_manager, language).await?
        }
        TemplateCommands::Show { name } => show_template(runner, &template_manager, &name).await?,
        TemplateCommands::Apply { name, vars, output, force } => {
            apply_template(runner, &template_manager, &name, vars, output, force).await?
        }
        TemplateCommands::Create {
            name,
            language,
            source,
        } => create_template(runner, &mut template_manager, name, language, source).await?,
        TemplateCommands::Remove { name } => {
            remove_template(runner, &mut template_manager, name).await?
        }
        TemplateCommands::Update { name, source } => {
            update_template(runner, &mut template_manager, name, source).await?
        }
    }

    Ok(())
}

async fn list_templates(
    runner: &CliRunner,
    template_manager: &TemplateManager,
    language_filter: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let all_templates = template_manager.list_templates();

    let filtered_templates: Vec<String> = if let Some(lang) = &language_filter {
        all_templates
            .into_iter()
            .filter(|name| {
                if let Some(template) = template_manager.get_template(name) {
                    template.language.to_lowercase() == lang.to_lowercase()
                } else {
                    false
                }
            })
            .collect()
    } else {
        all_templates
    };

    match runner.format() {
        crate::cli::OutputFormat::Json => {
            let template_data: Vec<_> = filtered_templates
                .iter()
                .filter_map(|name| template_manager.get_template(name))
                .map(|template| {
                    json!({
                        "name": template.name,
                        "language": template.language,
                        "description": template.description,
                        "variables": template.variables.len(),
                        "has_template": !template.content.is_empty()
                    })
                })
                .collect();

            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "templates": template_data,
                    "count": template_data.len(),
                    "filter": language_filter
                }))?
            );
        }
        _ => {
            if filtered_templates.is_empty() {
                if let Some(lang) = &language_filter {
                    runner.print_info(&format!("No templates found for language: {}", lang));
                } else {
                    runner.print_info("No templates available");
                }
                return Ok(());
            }

            let count = filtered_templates.len();
            if let Some(lang) = &language_filter {
                runner.print_info(&format!("Found {} templates for {}:", count, lang));
            } else {
                runner.print_info(&format!("Found {} templates:", count));
            }
            println!();

            for template_name in filtered_templates {
                if let Some(template) = template_manager.get_template(&template_name) {
                    print_template_summary(runner, template);
                }
            }
        }
    }

    Ok(())
}

async fn show_template(
    runner: &CliRunner,
    template_manager: &TemplateManager,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(template) = template_manager.get_template(name) {
        match runner.format() {
            crate::cli::OutputFormat::Json => {
                let template_data = json!({
                    "name": template.name,
                    "language": template.language,
                    "description": template.description,
                    "template": template.content,
                    "variables": template.variables.iter().map(|v| {
                        json!({
                            "name": v.name,
                            "description": v.description,
                            "default_value": v.default_value,
                            "required": v.default_value.is_none()
                        })
                    }).collect::<Vec<_>>()
                });

                println!("{}", serde_json::to_string_pretty(&template_data)?);
            }
            _ => {
                println!("📄 Template: {}", template.name);
                println!("🗣️  Language: {}", template.language);
                if !template.description.is_empty() {
                    println!("📝 Description: {}", template.description);
                }
                println!();

                if !template.variables.is_empty() {
                    println!("🔧 Variables:");
                    for var in &template.variables {
                        if let Some(default) = &var.default_value {
                            println!("  • {} = \"{}\" - {}", var.name, default, var.description);
                        } else {
                            println!("  • {} (required) - {}", var.name, var.description);
                        }
                    }
                    println!();
                }

                if !template.content.is_empty() {
                    println!("📃 Template Content:");
                    println!("{}", "─".repeat(50));
                    println!("{}", template.content);
                    println!("{}", "─".repeat(50));
                } else {
                    runner.print_warning("Template content is empty");
                }
            }
        }
    } else {
        runner.print_error(&format!("Template '{}' not found", name));
        return Err(format!("Template '{}' not found", name).into());
    }

    Ok(())
}

async fn create_template(
    runner: &CliRunner,
    template_manager: &mut TemplateManager,
    name: String,
    language: String,
    source: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if template already exists
    if template_manager.get_template(&name).is_some() {
        runner.print_error(&format!("Template '{}' already exists", name));
        return Err(format!("Template '{}' already exists", name).into());
    }

    // Check if source path exists
    if !source.exists() {
        runner.print_error(&format!("Source path does not exist: {}", source.display()));
        return Err(format!("Source path does not exist: {}", source.display()).into());
    }

    runner.print_info(&format!(
        "Creating template '{}' for {} from {}",
        name,
        language,
        source.display()
    ));

    // Read template content from source
    let template_content = if source.is_file() {
        fs::read_to_string(&source)?
    } else {
        // If it's a directory, look for template files
        let template_file = source.join("template.txt");
        if template_file.exists() {
            fs::read_to_string(&template_file)?
        } else {
            runner.print_error(&format!(
                "No template.txt found in directory: {}",
                source.display()
            ));
            return Err("Template file not found".into());
        }
    };

    // Parse variables from template content (look for {{variable}} patterns)
    let variables = extract_template_variables(&template_content);

    let template = Template {
        name: name.clone(),
        language: language.clone(),
        description: format!("Custom {} template", language),
        content: template_content,
        variables,
    };

    template_manager.add_template(template);
    runner.print_success(&format!("Template '{}' created successfully", name));

    Ok(())
}

async fn remove_template(
    runner: &CliRunner,
    template_manager: &mut TemplateManager,
    name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    if template_manager.remove_template(&name).is_some() {
        runner.print_success(&format!("Template '{}' removed successfully", name));
    } else {
        runner.print_error(&format!("Template '{}' not found", name));
        return Err(format!("Template '{}' not found", name).into());
    }

    Ok(())
}

async fn update_template(
    runner: &CliRunner,
    template_manager: &mut TemplateManager,
    name: String,
    source: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if template exists
    let existing_template = template_manager
        .get_template(&name)
        .ok_or_else(|| format!("Template '{}' not found", name))?;

    if let Some(source_path) = source {
        // Update template content from source
        if !source_path.exists() {
            runner.print_error(&format!(
                "Source path does not exist: {}",
                source_path.display()
            ));
            return Err(format!("Source path does not exist: {}", source_path.display()).into());
        }

        let new_content = if source_path.is_file() {
            fs::read_to_string(&source_path)?
        } else {
            let template_file = source_path.join("template.txt");
            if template_file.exists() {
                fs::read_to_string(&template_file)?
            } else {
                return Err("Template file not found".into());
            }
        };

        let new_variables = extract_template_variables(&new_content);

        // Create updated template
        let updated_template = Template {
            name: existing_template.name.clone(),
            language: existing_template.language.clone(),
            description: existing_template.description.clone(),
            content: new_content,
            variables: new_variables,
        };

        // Remove old and add updated
        template_manager.remove_template(&name);
        template_manager.add_template(updated_template);

        runner.print_success(&format!("Template '{}' updated successfully", name));
    } else {
        runner.print_info(&format!(
            "Template '{}' exists but no updates specified",
            name
        ));
    }

    Ok(())
}

fn print_template_summary(_runner: &CliRunner, template: &Template) {
    let var_count = template.variables.len();
    let var_info = if var_count > 0 {
        format!(" ({} variables)", var_count)
    } else {
        String::new()
    };

    println!("  📄 {} ({}){}", template.name, template.language, var_info);
    if !template.description.is_empty()
        && template.description != format!("Custom {} template", template.language)
    {
        println!("     {}", template.description);
    }
}

fn extract_template_variables(template_content: &str) -> Vec<TemplateVariable> {
    use regex::Regex;
    use std::collections::HashSet;

    let mut variables = Vec::new();
    let mut seen = HashSet::new();

    // Match {{variable}} patterns
    if let Ok(re) = Regex::new(r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\}\}") {
        for cap in re.captures_iter(template_content) {
            if let Some(var_name) = cap.get(1) {
                let name = var_name.as_str().to_string();
                if !seen.contains(&name) {
                    seen.insert(name.clone());
                    variables.push(TemplateVariable::required(
                        &name,
                        &format!("Template variable for {}", name),
                    ));
                }
            }
        }
    }

    variables
}

async fn apply_template(
    runner: &CliRunner,
    template_manager: &TemplateManager,
    name: &str,
    vars: Vec<String>,
    output: Option<PathBuf>,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Fetch template
    let template = template_manager
        .get_template(name)
        .ok_or_else(|| format!("Template '{}' not found", name))?;

    // Parse provided vars into a map
    let mut var_map: HashMap<String, String> = HashMap::new();
    for kv in vars {
        let mut parts = kv.splitn(2, '=');
        let k = parts
            .next()
            .ok_or("Invalid --var format; expected key=value")?
            .trim();
        let v = parts
            .next()
            .ok_or("Invalid --var format; expected key=value")?
            .to_string();
        if k.is_empty() {
            return Err("Variable key cannot be empty".into());
        }
        var_map.insert(k.to_string(), v);
    }

    // Check required variables
    for req in template.required_variables() {
        if !var_map.contains_key(&req.name) && req.default_value.is_none() {
            return Err(format!("Missing required variable: {}", req.name).into());
        }
    }

    // Apply the template
    let rendered = template_manager.apply_template(name, &var_map)?;

    // Output handling
    if let Some(path) = output {
        if path.exists() && !force {
            return Err(format!(
                "Output file exists: {} (use --force to overwrite)",
                path.display()
            )
            .into());
        }
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(&path, rendered)?;
        runner.print_success(&format!("Template '{}' applied to {}", name, path.display()));
    } else {
        runner.print_info(&format!("📝 Rendered output for template '{}':", name));
        runner.print_code(&rendered);
    }

    Ok(())
}
