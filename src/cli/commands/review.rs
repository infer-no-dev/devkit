use crate::agents::review::{ReviewCategory, ReviewConfig, ReviewSeverity};
use crate::cli::CliRunner;
use clap::{Args, ValueEnum};
use std::path::PathBuf;

/// Arguments for the review command
#[derive(Args, Clone)]
pub struct ReviewArgs {
    /// Path(s) to review (files or directories)
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,

    /// Focus areas for the review (can be specified multiple times)
    #[arg(short, long, value_enum)]
    pub focus: Vec<ReviewFocus>,

    /// Minimum severity level to report
    #[arg(long, value_enum, default_value = "low")]
    pub severity: ReviewSeverityArg,

    /// Output format
    #[arg(short, long, value_enum, default_value = "text")]
    pub output: OutputFormat,

    /// Save results to file
    #[arg(short = 'o', long)]
    pub save_to: Option<PathBuf>,

    /// Enable auto-fix for fixable issues
    #[arg(long)]
    pub auto_fix: bool,

    /// Include style issues in the review
    #[arg(long, default_value = "true")]
    pub include_style: bool,

    /// Maximum number of issues to report per file
    #[arg(long, default_value = "50")]
    pub max_issues_per_file: usize,

    /// Exclude files matching these patterns
    #[arg(long)]
    pub exclude: Vec<String>,

    /// Run review in watch mode (re-run on file changes)
    #[arg(short, long)]
    pub watch: bool,

    /// Show detailed progress information
    #[arg(short, long)]
    pub verbose: bool,
}

/// Review focus areas that can be specified via CLI
#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum ReviewFocus {
    Security,
    Performance,
    CodeSmells,
    Documentation,
    Testing,
    Maintainability,
    Style,
    Bugs,
    Vulnerabilities,
    All,
}

impl From<ReviewFocus> for ReviewCategory {
    fn from(focus: ReviewFocus) -> Self {
        match focus {
            ReviewFocus::Security => ReviewCategory::Security,
            ReviewFocus::Performance => ReviewCategory::Performance,
            ReviewFocus::CodeSmells => ReviewCategory::CodeSmell,
            ReviewFocus::Documentation => ReviewCategory::Documentation,
            ReviewFocus::Testing => ReviewCategory::Testing,
            ReviewFocus::Maintainability => ReviewCategory::Maintainability,
            ReviewFocus::Style => ReviewCategory::Style,
            ReviewFocus::Bugs => ReviewCategory::Bug,
            ReviewFocus::Vulnerabilities => ReviewCategory::Vulnerability,
            ReviewFocus::All => ReviewCategory::Security, // Default, will be handled specially
        }
    }
}

/// CLI severity argument
#[derive(ValueEnum, Clone, Debug)]
pub enum ReviewSeverityArg {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl From<ReviewSeverityArg> for ReviewSeverity {
    fn from(severity: ReviewSeverityArg) -> Self {
        match severity {
            ReviewSeverityArg::Info => ReviewSeverity::Info,
            ReviewSeverityArg::Low => ReviewSeverity::Low,
            ReviewSeverityArg::Medium => ReviewSeverity::Medium,
            ReviewSeverityArg::High => ReviewSeverity::High,
            ReviewSeverityArg::Critical => ReviewSeverity::Critical,
        }
    }
}

/// Output format options
#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Text,
    Json,
    Yaml,
    Csv,
    Html,
}

/// Main entry point for the review command
pub async fn run(
    runner: &mut CliRunner,
    args: ReviewArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    if args.verbose {
        runner.print_info(&format!(
            "Starting code review of {} paths",
            args.paths.len()
        ));
    }

    // Default to current directory if no paths specified
    let review_paths = if args.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.paths.clone()
    };

    // Validate paths exist
    for path in &review_paths {
        if !path.exists() {
            return Err(format!("Path does not exist: {}", path.display()).into());
        }
    }

    // Build review configuration from CLI args
    let review_config = build_review_config(&args)?;

    if args.watch {
        // Run in watch mode
        run_watch_mode(runner, &review_paths, &review_config, &args).await
    } else {
        // Run single review
        run_single_review(runner, &review_paths, &review_config, &args).await
    }
}

/// Build ReviewConfig from CLI arguments
fn build_review_config(args: &ReviewArgs) -> Result<ReviewConfig, Box<dyn std::error::Error>> {
    let mut config = ReviewConfig::default();

    // Set focus areas
    if !args.focus.is_empty() {
        config.focus_areas = if args.focus.contains(&ReviewFocus::All) {
            vec![
                ReviewCategory::Security,
                ReviewCategory::Performance,
                ReviewCategory::CodeSmell,
                ReviewCategory::Documentation,
                ReviewCategory::Testing,
                ReviewCategory::Maintainability,
                ReviewCategory::Style,
                ReviewCategory::Bug,
                ReviewCategory::Vulnerability,
            ]
        } else {
            args.focus.iter().map(|f| f.clone().into()).collect()
        };
    }

    // Set other options
    config.severity_threshold = args.severity.clone().into();
    config.enable_auto_fix = args.auto_fix;
    config.include_style_issues = args.include_style;
    config.max_issues_per_file = args.max_issues_per_file;

    // Add exclude patterns
    if !args.exclude.is_empty() {
        config.exclude_files.extend(args.exclude.iter().cloned());
    }

    Ok(config)
}

/// Run a single code review
async fn run_single_review(
    _runner: &mut CliRunner,
    paths: &[PathBuf],
    config: &ReviewConfig,
    args: &ReviewArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    if args.verbose {
        println!(
            "Reviewing {} paths with {} focus areas",
            paths.len(),
            config.focus_areas.len()
        );
    }

    // For now, create a mock result since the CLI integration needs more work
    // TODO: Integrate with actual agent system once CLI runner is properly set up

    let mock_result = crate::agents::review::ReviewResult {
        issues: vec![crate::agents::review::ReviewIssue {
            category: crate::agents::review::ReviewCategory::CodeSmell,
            severity: crate::agents::review::ReviewSeverity::Medium,
            title: "Example code smell detected".to_string(),
            description: "This is a demonstration of the code review functionality.".to_string(),
            file_path: paths
                .get(0)
                .unwrap_or(&std::path::PathBuf::from("."))
                .clone(),
            line_start: Some(1),
            line_end: Some(1),
            suggestion: Some("Consider refactoring for better maintainability.".to_string()),
            auto_fixable: false,
            code_snippet: None,
        }],
        summary: crate::agents::review::ReviewSummary {
            total_issues: 1,
            issues_by_severity: std::collections::HashMap::from([("Medium".to_string(), 1)]),
            issues_by_category: std::collections::HashMap::from([("CodeSmell".to_string(), 1)]),
            auto_fixable_issues: 0,
            most_common_issue: Some("CodeSmell".to_string()),
        },
        files_reviewed: paths.len(),
        total_lines: 100, // Mock value
        review_duration: std::time::Duration::from_secs(1),
    };

    if args.verbose {
        println!(
            "Review completed in {:.2}s",
            mock_result.review_duration.as_secs_f64()
        );
    }

    display_review_results(&mock_result, args)?;

    // Save to file if requested
    if let Some(output_path) = &args.save_to {
        save_review_results(&mock_result, output_path, &args.output)?;
        println!("✅ Review results saved to: {}", output_path.display());
    }

    // Show summary
    display_review_summary(&mock_result);

    Ok(())
}

/// Run code review in watch mode
async fn run_watch_mode(
    _runner: &mut CliRunner,
    _paths: &[PathBuf],
    _config: &ReviewConfig,
    _args: &ReviewArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement file watching and re-running reviews
    println!("Watch mode not yet implemented");
    Ok(())
}

/// Display review results in the specified format
fn display_review_results(
    result: &crate::agents::review::ReviewResult,
    args: &ReviewArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    match args.output {
        OutputFormat::Text => display_text_results(result),
        OutputFormat::Json => display_json_results(result)?,
        OutputFormat::Yaml => display_yaml_results(result)?,
        OutputFormat::Csv => display_csv_results(result)?,
        OutputFormat::Html => display_html_results(result)?,
    }
    Ok(())
}

/// Display results in text format
fn display_text_results(result: &crate::agents::review::ReviewResult) {
    if result.issues.is_empty() {
        println!("✅ No issues found!");
        return;
    }

    println!("🔍 Found {} issues:", result.issues.len());
    println!();

    for issue in &result.issues {
        let severity_icon = match issue.severity {
            crate::agents::review::ReviewSeverity::Critical => "🚨",
            crate::agents::review::ReviewSeverity::High => "⚠️ ",
            crate::agents::review::ReviewSeverity::Medium => "⚡",
            crate::agents::review::ReviewSeverity::Low => "💡",
            crate::agents::review::ReviewSeverity::Info => "ℹ️ ",
        };

        let category_label = format!("{:?}", issue.category);

        println!(
            "{} {} [{}] {}",
            severity_icon,
            category_label,
            format!("{:?}", issue.severity),
            issue.title
        );

        println!("   📄 {}", issue.file_path.display());

        if let Some(line) = issue.line_start {
            println!("   📍 Line {}", line);
        }

        println!("   📝 {}", issue.description);

        if let Some(suggestion) = &issue.suggestion {
            println!("   💭 Suggestion: {}", suggestion);
        }

        if issue.auto_fixable {
            println!("   🔧 Auto-fixable");
        }

        println!();
    }
}

/// Display results in JSON format
fn display_json_results(
    result: &crate::agents::review::ReviewResult,
) -> Result<(), Box<dyn std::error::Error>> {
    let json_output = serde_json::to_string_pretty(result)?;
    println!("{}", json_output);
    Ok(())
}

/// Display results in YAML format
fn display_yaml_results(
    result: &crate::agents::review::ReviewResult,
) -> Result<(), Box<dyn std::error::Error>> {
    let yaml_output = serde_yaml::to_string(result)?;
    println!("{}", yaml_output);
    Ok(())
}

/// Display results in CSV format
fn display_csv_results(
    result: &crate::agents::review::ReviewResult,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("File,Line,Severity,Category,Title,Description,Suggestion,AutoFixable");

    for issue in &result.issues {
        println!(
            "{},{},{:?},{:?},{},{},{},{}",
            issue.file_path.display(),
            issue.line_start.map_or("".to_string(), |l| l.to_string()),
            issue.severity,
            issue.category,
            escape_csv_field(&issue.title),
            escape_csv_field(&issue.description),
            issue.suggestion.as_deref().unwrap_or(""),
            issue.auto_fixable
        );
    }
    Ok(())
}

/// Display results in HTML format
fn display_html_results(
    result: &crate::agents::review::ReviewResult,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("<!DOCTYPE html>");
    println!("<html><head><title>Code Review Report</title>");
    println!("<style>");
    println!("body {{ font-family: Arial, sans-serif; margin: 40px; }}");
    println!(".issue {{ margin: 20px 0; padding: 15px; border-left: 4px solid #ddd; }}");
    println!(".critical {{ border-color: #d73a49; }}");
    println!(".high {{ border-color: #f66a0a; }}");
    println!(".medium {{ border-color: #ffd33d; }}");
    println!(".low {{ border-color: #28a745; }}");
    println!(".info {{ border-color: #0366d6; }}");
    println!("</style>");
    println!("</head><body>");

    println!("<h1>Code Review Report</h1>");
    println!(
        "<p>Found {} issues in {} files</p>",
        result.issues.len(),
        result.files_reviewed
    );

    for issue in &result.issues {
        let severity_class = format!("{:?}", issue.severity).to_lowercase();
        println!("<div class=\"issue {}\">", severity_class);
        println!("<h3>{}</h3>", html_escape(&issue.title));
        println!(
            "<p><strong>File:</strong> {}</p>",
            html_escape(&issue.file_path.display().to_string())
        );

        if let Some(line) = issue.line_start {
            println!("<p><strong>Line:</strong> {}</p>", line);
        }

        println!("<p><strong>Category:</strong> {:?}</p>", issue.category);
        println!("<p><strong>Severity:</strong> {:?}</p>", issue.severity);
        println!(
            "<p><strong>Description:</strong> {}</p>",
            html_escape(&issue.description)
        );

        if let Some(suggestion) = &issue.suggestion {
            println!(
                "<p><strong>Suggestion:</strong> {}</p>",
                html_escape(suggestion)
            );
        }

        if issue.auto_fixable {
            println!("<p><em>This issue can be automatically fixed.</em></p>");
        }

        println!("</div>");
    }

    println!("</body></html>");
    Ok(())
}

/// Display review summary
fn display_review_summary(result: &crate::agents::review::ReviewResult) {
    println!();
    println!("📊 Review Summary:");
    println!("   Files reviewed: {}", result.files_reviewed);
    println!("   Total lines: {}", result.total_lines);
    println!("   Total issues: {}", result.summary.total_issues);
    println!(
        "   Auto-fixable issues: {}",
        result.summary.auto_fixable_issues
    );
    println!(
        "   Review duration: {:.2}s",
        result.review_duration.as_secs_f64()
    );

    if !result.summary.issues_by_severity.is_empty() {
        println!();
        println!("Issues by severity:");
        for (severity, count) in &result.summary.issues_by_severity {
            println!("   {}: {}", severity, count);
        }
    }

    if !result.summary.issues_by_category.is_empty() {
        println!();
        println!("Issues by category:");
        for (category, count) in &result.summary.issues_by_category {
            println!("   {}: {}", category, count);
        }
    }

    if let Some(most_common) = &result.summary.most_common_issue {
        println!();
        println!("Most common issue type: {}", most_common);
    }
}

/// Save review results to file
fn save_review_results(
    result: &crate::agents::review::ReviewResult,
    output_path: &PathBuf,
    format: &OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = match format {
        OutputFormat::Json => serde_json::to_string_pretty(result)?,
        OutputFormat::Yaml => serde_yaml::to_string(result)?,
        OutputFormat::Text => format!(
            "Code Review Report\\n\\nTotal Issues: {}\\n\\n{:#?}",
            result.summary.total_issues, result
        ),
        OutputFormat::Csv => {
            let mut csv_content =
                "File,Line,Severity,Category,Title,Description,Suggestion,AutoFixable\\n"
                    .to_string();
            for issue in &result.issues {
                csv_content.push_str(&format!(
                    "{},{},{:?},{:?},{},{},{},{}\\n",
                    issue.file_path.display(),
                    issue.line_start.map_or("".to_string(), |l| l.to_string()),
                    issue.severity,
                    issue.category,
                    escape_csv_field(&issue.title),
                    escape_csv_field(&issue.description),
                    issue.suggestion.as_deref().unwrap_or(""),
                    issue.auto_fixable
                ));
            }
            csv_content
        }
        OutputFormat::Html => {
            // Generate HTML content similar to display_html_results
            format!("<!DOCTYPE html><html><head><title>Code Review Report</title></head><body><h1>Review Results</h1><p>Total Issues: {}</p></body></html>", result.summary.total_issues)
        }
    };

    std::fs::write(output_path, content)?;
    Ok(())
}

/// Escape CSV field content
fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

/// Escape HTML content
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
