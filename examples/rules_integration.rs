//! Example showing how WARP.md rules integrate with agent tasks
//!
//! This example demonstrates the hierarchical rules precedence system:
//! - Global rules from ~/.devkit/global_rules.md
//! - Project rules from ./WARP.md  
//! - Directory rules from subdirectory WARP.md files
//!
//! Run with: cargo run --example rules_integration

use devkit::config::{ConfigManager, rules::{RuleContext, RulePriority}};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 DevKit Rules Integration Example");
    println!("====================================\n");
    
    // Create a config manager
    let mut config = ConfigManager::new_with_smart_defaults(None)?;
    
    // Simulate a project directory
    let project_dir = std::env::current_dir()?;
    
    // Load rules hierarchy for the current project
    println!("📚 Loading rules hierarchy for: {}", project_dir.display());
    match config.load_rules(&project_dir).await {
        Ok(_) => println!("✅ Rules loaded successfully"),
        Err(e) => println!("⚠️  No rules found or error loading: {}", e),
    }
    
    // Show rules summary
    let summary = config.get_rules_summary();
    println!("\n📊 Rules Summary:");
    println!("  Total rule sets: {}", summary.total_rule_sets);
    println!("  Total rules: {}", summary.total_rules);
    
    for (priority, count) in &summary.by_priority {
        println!("  {:?} priority: {} rules", priority, count);
    }
    
    // Create different contexts to show precedence
    let contexts = vec![
        (
            "Rust file in project root",
            config.create_rule_context(
                project_dir.clone(),
                Some(project_dir.join("main.rs")),
                Some("rust".to_string()),
                Some("codegen".to_string()),
                Some("generate_code".to_string()),
            )
        ),
        (
            "TypeScript file in src/ subdirectory", 
            config.create_rule_context(
                project_dir.join("src"),
                Some(project_dir.join("src/index.ts")),
                Some("typescript".to_string()),
                Some("codegen".to_string()),
                Some("generate_code".to_string()),
            )
        ),
        (
            "General context without specific files",
            config.create_rule_context(
                project_dir.clone(),
                None,
                None,
                Some("chat".to_string()),
                Some("general_query".to_string()),
            )
        ),
    ];
    
    // Show effective rules for each context
    for (context_name, context) in contexts {
        println!("\n🎯 Context: {}", context_name);
        println!("   Directory: {}", context.current_directory.display());
        println!("   File: {:?}", context.file_path);
        println!("   Language: {:?}", context.language);
        
        let effective_rules = config.get_effective_rules(&context);
        
        if effective_rules.is_empty() {
            println!("   📝 No specific rules apply");
        } else {
            println!("   📝 Effective rules ({} total):", effective_rules.len());
            for (i, rule) in effective_rules.iter().enumerate() {
                println!("     {}. {} (Priority: {:?})", 
                    i + 1, 
                    rule.name, 
                    rule.priority
                );
                println!("        Source: {}", 
                    rule.metadata.source_file.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                );
            }
        }
        
        // Show how this would be formatted for AI
        let formatted = config.format_rules_for_ai(&context);
        if !formatted.contains("No specific rules") {
            println!("   🤖 Sample AI formatting (first 200 chars):");
            println!("      {}", 
                formatted.chars().take(200).collect::<String>()
                    .replace('\n', "\\n")
                + "..."
            );
        }
    }
    
    // Example of creating a sample WARP.md structure
    println!("\n📖 Example WARP.md Structure:");
    println!("==============================");
    
    let example_warp = r#"
# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

This is an intelligent development environment that assists with code generation and analysis.

## Rule: Code Style
- Always use 4 spaces for indentation
- Prefer explicit types over implicit ones
- Include comprehensive documentation

## Rule: Testing
- Write unit tests for all public functions
- Use descriptive test names that explain the behavior being tested
- Include edge cases and error conditions

## Rule: Documentation
- All public APIs must have documentation comments
- Include usage examples in documentation
- Keep README.md updated with new features
"#;
    
    println!("{}", example_warp);
    
    // Show how precedence works
    println!("\n🏆 Precedence Rules:");
    println!("=====================");
    println!("1. Session overrides    (Priority: 500) - Temporary session-specific rules");
    println!("2. Directory rules      (Priority: 400) - WARP.md in current/parent directories");
    println!("3. Project rules        (Priority: 300) - WARP.md in project root");
    println!("4. User rules          (Priority: 200) - ~/.devkit/global_rules.md");
    println!("5. Global defaults     (Priority: 100) - Built-in system defaults");
    println!("\nWhen multiple rules apply, higher priority rules override lower priority ones.");
    println!("This allows project-specific customization while maintaining global consistency.");
    
    Ok(())
}
