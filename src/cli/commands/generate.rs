use crate::cli::{CliRunner, GenerateArgs};
use crate::codegen::stubs;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

pub async fn run(
    runner: &mut CliRunner,
    args: GenerateArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info(&format!(
        "🤖 Generating code from prompt: \"{}\"",
        args.prompt
    ));

    // Analyze the prompt and infer language
    let language = determine_language(&args);
    
    if runner.verbose() {
        runner.print_verbose(&format!(
            "Language: {:?}, Strategy: {}", 
            language, args.strategy
        ));
    }

    // Show progress for generation  
    if !runner.quiet() {
        print!("🔄 Generating code...");
        io::stdout().flush()?;
    }

    // Generate code using the stub system
    let generated_code = generate_code_with_stubs(&args, language.as_deref());
    
    if !runner.quiet() {
        println!(" ✅");
    }

    // Handle the generated code based on mode
    if args.preview {
        display_generated_code(runner, &generated_code, &args, language.as_deref())?;
    } else {
        save_generated_code(runner, &generated_code, &args, language.as_deref()).await?;
    }

    runner.print_success("Code generation completed successfully!");
    Ok(())
}

/// Determine the language to use for code generation
fn determine_language(args: &GenerateArgs) -> Option<String> {
    // First, check if language was explicitly specified
    if let Some(lang) = &args.language {
        return Some(lang.clone());
    }

    // Then, try to infer from output path
    if let Some(output_path) = &args.output {
        return stubs::infer_language_from_context(&args.prompt, Some(output_path));
    }

    // Finally, try to infer from prompt
    stubs::infer_language_from_context(&args.prompt, None)
}

/// Generate code using our stub system
fn generate_code_with_stubs(args: &GenerateArgs, language: Option<&str>) -> String {
    // Enhance prompt with context if provided
    let enhanced_prompt = if !args.context.is_empty() {
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
            format!(
                "{}\n\nContext:\n{}",
                args.prompt, context_content
            )
        } else {
            args.prompt.clone()
        }
    } else {
        args.prompt.clone()
    };

    stubs::generate_code_stub(&enhanced_prompt, language)
}

fn display_generated_code(
    runner: &CliRunner,
    code: &str,
    args: &GenerateArgs,
    language: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info("📝 Generated Code (Preview Mode):");
    
    // Use the enhanced code display from chat command
    runner.print_code(code);

    // Show where it would be saved
    if let Some(output_path) = &args.output {
        runner.print_info(&format!("📁 Would save to: {}", output_path.display()));
    } else {
        let filename = stubs::suggest_filename(&args.prompt, language);
        runner.print_info(&format!("📁 Would save to: {}", filename));
    }

    Ok(())
}

async fn save_generated_code(
    runner: &CliRunner,
    code: &str,
    args: &GenerateArgs,
    language: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = if let Some(path) = &args.output {
        path.clone()
    } else {
        // Use the shared filename suggestion from stubs
        PathBuf::from(stubs::suggest_filename(&args.prompt, language))
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
