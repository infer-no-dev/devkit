use crate::ai::AIManager;
use crate::cli::{CliRunner, GenerateArgs};
use crate::codegen::{CodeGenerator, GenerationRequest};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;

pub async fn run(
    runner: &mut CliRunner,
    args: GenerateArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info(&format!(
        "🤖 Generating code from prompt: \"{}\"",
        args.prompt
    ));

    // Ensure context manager is initialized for context-aware generation
    runner.ensure_context_manager().await?;

    // Initialize AI manager from configuration
    let config = runner.config_manager().config();
    let ai_manager = Arc::new(AIManager::new(config.codegen.ai_model_settings.clone()).await?);

    // Create code generator with AI manager
    let mut code_generator = CodeGenerator::new()?;
    code_generator.set_ai_manager(ai_manager.clone());

    // Build generation request
    let generation_request = build_generation_request(runner, &args).await?;

    if runner.verbose() {
        runner.print_verbose(&format!(
            "Generation request: language={:?}, strategy={}",
            generation_request.config.target_language, args.strategy
        ));
    }

    // Show progress for generation
    if !runner.quiet() {
        print!("🔄 Generating code...");
        io::stdout().flush()?;
    }

    // Generate code using the AI-powered generator
    match code_generator.generate_code(&generation_request).await {
        Ok(generated_code) => {
            if !runner.quiet() {
                println!(" ✅");
            }

            // Handle the generated code based on mode
            if args.preview {
                display_generated_code(runner, &generated_code, &args)?;
            } else {
                save_generated_code(runner, &generated_code, &args).await?;
            }

            runner.print_success("Code generation completed successfully!");
        }
        Err(e) => {
            if !runner.quiet() {
                println!(" ❌");
            }
            runner.print_error(&format!("Code generation failed: {}", e));
            return Err(e.into());
        }
    }

    Ok(())
}

async fn build_generation_request(
    runner: &mut CliRunner,
    args: &GenerateArgs,
) -> Result<GenerationRequest, Box<dyn std::error::Error>> {
    use crate::context::CodebaseContext;

    // Create a basic codebase context or use existing one
    let context = CodebaseContext::default();

    let mut config = crate::codegen::GenerationConfig::default();
    config.target_language = args.language.clone();
    config.temperature = args.temperature.map(|t| t as f64);
    config.max_tokens = args.max_tokens;

    let mut request = GenerationRequest {
        prompt: args.prompt.clone(),
        file_path: args
            .output
            .as_ref()
            .map(|p| p.to_string_lossy().to_string()),
        context,
        config,
        constraints: Vec::new(),
    };

    // Add context from specified files
    if !args.context.is_empty() {
        runner.print_verbose(&format!("Adding context from {} files", args.context.len()));

        let mut context_content = String::new();
        for context_file in &args.context {
            if let Ok(content) = fs::read_to_string(context_file) {
                context_content.push_str(&format!(
                    "\n// Context from {}:\n{}",
                    context_file.display(),
                    content
                ));
            }
        }

        if !context_content.is_empty() {
            request.prompt = format!(
                "{}

Context:
{}",
                request.prompt, context_content
            );
        }
    }

    // Add minimal codebase context information if available
    if runner.context_manager_mut().is_some() {
        // Add a note that context is available for enhanced generation
        request
            .prompt
            .push_str("\n\nNote: This generation is context-aware based on the current codebase.");
    }

    // Detect language from output path if not specified in config
    if request.config.target_language.is_none() {
        if let Some(output_path) = &args.output {
            if let Some(extension) = output_path.extension().and_then(|e| e.to_str()) {
                request.config.target_language = match extension {
                    "rs" => Some("rust".to_string()),
                    "py" => Some("python".to_string()),
                    "js" => Some("javascript".to_string()),
                    "ts" => Some("typescript".to_string()),
                    "go" => Some("go".to_string()),
                    "java" => Some("java".to_string()),
                    "cpp" | "cc" | "cxx" => Some("cpp".to_string()),
                    "c" => Some("c".to_string()),
                    _ => None,
                };
            }
        }
    }

    Ok(request)
}

fn display_generated_code(
    runner: &CliRunner,
    code: &str,
    args: &GenerateArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info("📝 Generated Code (Preview Mode):");
    println!("═══════════════════════════════════");

    // Add syntax highlighting hint if language is known
    if let Some(language) = &args.language {
        println!("```{}", language);
    } else {
        println!("```");
    }

    println!("{}", code);
    println!("```");

    if let Some(output_path) = &args.output {
        runner.print_info(&format!("📁 Would save to: {}", output_path.display()));
    }

    Ok(())
}

async fn save_generated_code(
    runner: &CliRunner,
    code: &str,
    args: &GenerateArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = if let Some(path) = &args.output {
        path.clone()
    } else {
        // Generate a filename based on the prompt and language
        let sanitized_prompt = args
            .prompt
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
            .take(30)
            .collect::<String>();

        let extension = args
            .language
            .as_deref()
            .map(|lang| match lang {
                "rust" => "rs",
                "python" => "py",
                "javascript" => "js",
                "typescript" => "ts",
                "go" => "go",
                "java" => "java",
                "cpp" => "cpp",
                "c" => "c",
                _ => "txt",
            })
            .unwrap_or("txt");

        PathBuf::from(format!(
            "generated_{}_{}.{}",
            sanitized_prompt.to_lowercase(),
            chrono::Utc::now().timestamp(),
            extension
        ))
    };

    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
            runner.print_verbose(&format!("Created directory: {}", parent.display()));
        }
    }

    // Write the generated code to file
    fs::write(&output_path, code)?;

    runner.print_success(&format!("💾 Code saved to: {}", output_path.display()));

    // Show code statistics
    let lines = code.lines().count();
    let chars = code.len();
    runner.print_info(&format!(
        "📊 Generated {} lines ({} characters)",
        lines, chars
    ));

    Ok(())
}
