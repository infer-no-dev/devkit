use crate::cli::{AnalyzeArgs, CliRunner, OutputFormat};
use crate::context::AnalysisConfig;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::time::Instant;

pub async fn run(
    runner: &mut CliRunner,
    args: AnalyzeArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    // Determine targets - default to current directory if none provided
    let targets = if args.targets.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.targets
    };

    // Store flags before borrowing runner mutably
    let verbose = runner.verbose();
    let quiet = runner.quiet();
    let format = runner.format().clone();

    runner.print_info(&format!(
        "🔍 Starting codebase analysis of {} target{}",
        targets.len(),
        if targets.len() == 1 { "" } else { "s" }
    ));

    if verbose {
        for target in &targets {
            runner.print_verbose(&format!("Target: {}", target.display()));
        }
    }

    // Configure analysis based on arguments
    let mut analysis_config = AnalysisConfig::default();

    // Set analysis depth
    analysis_config.deep_analysis = match args.depth.as_str() {
        "shallow" => false,
        "normal" => false,
        "deep" => true,
        _ => {
            runner.print_warning(&format!("Unknown depth '{}', using 'normal'", args.depth));
            false
        }
    };

    // Configure test file inclusion
    if args.include_tests {
        // Remove test exclusions from default patterns
        analysis_config
            .exclude_patterns
            .retain(|pattern| !pattern.contains("test") && !pattern.contains("spec"));
        if verbose {
            runner.print_verbose("Including test files in analysis");
        }
    }

    // Configure analysis types
    if !args.analysis_types.is_empty() && verbose {
        runner.print_verbose(&format!(
            "Specific analysis types requested: {:?}",
            args.analysis_types
        ));
        // For now, we'll note the request but analyze everything
        // In a full implementation, you'd filter the analysis based on types
    }

    // Setup progress indicator if requested
    let progress_bar = if args.progress && !quiet {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::with_template("{spinner:.green} {msg}")
                .unwrap_or_else(|_| ProgressStyle::default_spinner()),
        );
        pb.set_message("Initializing analysis...");
        Some(pb)
    } else {
        None
    };

    // Initialize context manager and analyze targets
    runner.ensure_context_manager().await?;
    let mut all_contexts = Vec::new();

    // Analyze each target
    for (i, target) in targets.iter().enumerate() {
        if let Some(ref pb) = progress_bar {
            pb.set_message(format!(
                "Analyzing target {}/{}: {}",
                i + 1,
                targets.len(),
                target.display()
            ));
        }

        runner.print_info(&format!("📁 Analyzing: {}", target.display()));

        // Perform the analysis (get fresh reference each time to avoid borrowing conflicts)
        let result = {
            let context_manager = runner.context_manager_mut().unwrap();
            context_manager
                .analyze_codebase(target.clone(), analysis_config.clone())
                .await
        };

        match result {
            Ok(context) => {
                runner.print_success(&format!(
                    "✅ Analysis complete: {} files, {} symbols, {} languages",
                    context.metadata.total_files,
                    context.metadata.indexed_symbols,
                    context.metadata.languages.len()
                ));

                all_contexts.push(context);
            }
            Err(e) => {
                runner.print_error(&format!("Failed to analyze {}: {}", target.display(), e));
                return Err(e.into());
            }
        }
    }

    if let Some(ref pb) = progress_bar {
        pb.set_message("Generating output...");
    }

    // Generate output based on format
    let output_content = match format {
        OutputFormat::Text => format_text_output(&all_contexts),
        OutputFormat::Json => format_json_output(&all_contexts)?,
        OutputFormat::Yaml => format_yaml_output(&all_contexts)?,
        OutputFormat::Table => format_table_output(&all_contexts),
    };

    // Export to file if requested
    if let Some(export_path) = args.export {
        runner.print_info(&format!(
            "📄 Exporting results to: {}",
            export_path.display()
        ));
        std::fs::write(&export_path, &output_content)
            .map_err(|e| format!("Failed to write export file: {}", e))?;
        runner.print_success(&format!("Results exported to {}", export_path.display()));
    } else {
        // Print to stdout
        if !quiet {
            println!("\n{}", output_content);
        }
    }

    if let Some(ref pb) = progress_bar {
        pb.finish_with_message("Analysis complete!");
    }

    let duration = start_time.elapsed();
    runner.print_success(&format!(
        "🎉 Analysis completed in {:.2}s",
        duration.as_secs_f64()
    ));

    Ok(())
}

fn format_text_output(contexts: &[crate::context::CodebaseContext]) -> String {
    let mut output = String::new();
    output.push_str("📊 Codebase Analysis Report\n");
    output.push_str("═══════════════════════════\n\n");

    for (i, context) in contexts.iter().enumerate() {
        if contexts.len() > 1 {
            output.push_str(&format!(
                "📁 Target {}: {}\n",
                i + 1,
                context.root_path.display()
            ));
            output.push_str("─────────────────────────\n");
        }

        output.push_str(&format!("📂 Root Path: {}\n", context.root_path.display()));
        output.push_str(&format!(
            "📄 Total Files: {}\n",
            context.metadata.total_files
        ));
        output.push_str(&format!(
            "📏 Total Lines: {}\n",
            context.metadata.total_lines
        ));
        output.push_str(&format!(
            "🔍 Indexed Symbols: {}\n",
            context.metadata.indexed_symbols
        ));
        output.push_str(&format!(
            "⏱️  Analysis Duration: {}ms\n",
            context.metadata.analysis_duration_ms
        ));

        if !context.metadata.languages.is_empty() {
            output.push_str("\n🗣️  Languages:\n");
            let mut lang_vec: Vec<_> = context.metadata.languages.iter().collect();
            lang_vec.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending

            for (lang, count) in lang_vec {
                output.push_str(&format!("  • {}: {} files\n", lang, count));
            }
        }

        if !context.dependencies.is_empty() {
            output.push_str(&format!(
                "\n📦 Dependencies: {}\n",
                context.dependencies.len()
            ));
            for dep in context.dependencies.iter().take(10) {
                // Show first 10
                let version = dep.version.as_deref().unwrap_or("*");
                output.push_str(&format!(
                    "  • {} ({}): {}\n",
                    dep.name,
                    version,
                    format!("{:?}", dep.dependency_type)
                ));
            }
            if context.dependencies.len() > 10 {
                output.push_str(&format!(
                    "  ... and {} more\n",
                    context.dependencies.len() - 10
                ));
            }
        }

        if let Some(ref repo_info) = context.repository_info {
            output.push_str(&format!(
                "\n🌿 Repository: {} (Git)",
                repo_info.root_path.display()
            ));
            if let Some(ref branch) = repo_info.current_branch {
                output.push_str(&format!(" on branch '{}'\n", branch));
            } else {
                output.push('\n');
            }

            if let Some(ref remote) = repo_info.remote_url {
                output.push_str(&format!("  • Remote: {}\n", remote));
            }

            output.push_str(&format!("  • Commits: {}\n", repo_info.commit_count));
            output.push_str(&format!(
                "  • Clean: {}\n",
                if repo_info.status.is_clean {
                    "Yes"
                } else {
                    "No"
                }
            ));
        }

        if let Some(ref semantic) = context.semantic_analysis {
            output.push_str(&format!("\n🧠 Semantic Analysis:\n"));
            output.push_str(&format!(
                "  • Patterns Found: {}\n",
                semantic.patterns.len()
            ));
            output.push_str(&format!(
                "  • Relationships: {}\n",
                semantic.relationships.len()
            ));
            output.push_str(&format!(
                "  • Context Suggestions: {}\n",
                semantic.context_suggestions.len()
            ));
        }

        if contexts.len() > 1 && i < contexts.len() - 1 {
            output.push_str("\n");
        }
    }

    output
}

fn format_json_output(
    contexts: &[crate::context::CodebaseContext],
) -> Result<String, Box<dyn std::error::Error>> {
    Ok(serde_json::to_string_pretty(contexts)?)
}

fn format_yaml_output(
    contexts: &[crate::context::CodebaseContext],
) -> Result<String, Box<dyn std::error::Error>> {
    Ok(serde_yaml::to_string(contexts)?)
}

fn format_table_output(contexts: &[crate::context::CodebaseContext]) -> String {
    let mut output = String::new();
    output.push_str("┌─────────────────────┬──────────┬──────────┬─────────┬───────────┐\n");
    output.push_str("│ Path                │ Files    │ Lines    │ Symbols │ Languages │\n");
    output.push_str("├─────────────────────┼──────────┼──────────┼─────────┼───────────┤\n");

    for context in contexts {
        let path = context
            .root_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(".")
            .chars()
            .take(19)
            .collect::<String>();

        output.push_str(&format!(
            "│ {:<19} │ {:<8} │ {:<8} │ {:<7} │ {:<9} │\n",
            path,
            context.metadata.total_files,
            context.metadata.total_lines,
            context.metadata.indexed_symbols,
            context.metadata.languages.len()
        ));
    }

    output.push_str("└─────────────────────┴──────────┴──────────┴─────────┴───────────┘\n");
    output
}
