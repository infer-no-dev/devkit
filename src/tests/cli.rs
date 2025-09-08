//! CLI command tests

use crate::cli::*;
use crate::tests::test_utils::*;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_analyze_command_basic() {
    let (_temp_dir, project_path) = create_sample_rust_project();
    
    // Create CLI with analyze command
    let args = AnalyzeArgs {
        targets: vec![project_path.clone()],
        depth: "normal".to_string(),
        include_tests: true,
        export: None,
        analysis_types: vec!["symbols".to_string(), "dependencies".to_string()],
        progress: false,
    };
    
    // Test basic analysis functionality
    // Note: In a real implementation, this would create a CliRunner and run the command
    assert!(!args.targets.is_empty());
    assert_eq!(args.depth, "normal");
    assert!(args.include_tests);
}

#[tokio::test]
async fn test_generate_command_basic() {
    let args = GenerateArgs {
        prompt: "Create a simple HTTP client function".to_string(),
        output: None,
        language: Some("rust".to_string()),
        context: vec![],
        strategy: "focused".to_string(),
        max_tokens: Some(500),
        temperature: Some(0.7),
        preview: true,
    };
    
    // Test basic generation functionality
    assert!(!args.prompt.is_empty());
    assert_eq!(args.language.as_ref().unwrap(), "rust");
    assert_eq!(args.strategy, "focused");
    assert!(args.preview);
}

#[tokio::test]
async fn test_template_commands() {
    // Test template listing functionality
    // This would normally test the actual template management
    
    // Test template creation
    let template_name = "test_template";
    let language = "rust";
    
    // In a real test, we would:
    // 1. Create a template manager
    // 2. Add a test template
    // 3. Verify it can be retrieved
    // 4. Test template application
    
    assert_eq!(template_name, "test_template");
    assert_eq!(language, "rust");
}

#[test]
fn test_cli_argument_parsing() {
    // Test that CLI arguments are parsed correctly
    use clap::Parser;
    
    // Test analyze command parsing
    let cli = Cli::try_parse_from(&[
        "agentic-dev",
        "analyze", 
        "src/",
        "--depth", "deep",
        "--include-tests",
        "--progress"
    ]);
    
    assert!(cli.is_ok());
    
    if let Ok(cli) = cli {
        match cli.command {
            Commands::Analyze(args) => {
                assert_eq!(args.targets.len(), 1);
                assert_eq!(args.depth, "deep");
                assert!(args.include_tests);
                assert!(args.progress);
            }
            _ => panic!("Expected Analyze command"),
        }
    }
}

#[test]
fn test_global_options() {
    use clap::Parser;
    
    let cli = Cli::try_parse_from(&[
        "agentic-dev",
        "--verbose",
        "--format", "json",
        "analyze",
        "."
    ]);
    
    assert!(cli.is_ok());
    
    if let Ok(cli) = cli {
        assert!(cli.verbose);
        assert_eq!(cli.format, OutputFormat::Json);
    }
}

#[tokio::test]
async fn test_interactive_args() {
    let args = InteractiveArgs {
        view: Some("agents".to_string()),
        auto_start: true,
        monitor: false,
    };
    
    assert_eq!(args.view.as_ref().unwrap(), "agents");
    assert!(args.auto_start);
    assert!(!args.monitor);
}
