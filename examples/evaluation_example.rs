use devkit::{
    EvaluationFramework, EvaluationConfig, EvaluationContext, EvaluationEnvironment,
    BuildProfile, GeneratedCode
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("DevKit Evaluation Framework Example");
    
    // Create evaluation configuration
    let eval_config = EvaluationConfig::default();
    
    // Initialize evaluation framework
    let framework = EvaluationFramework::new(eval_config);
    
    // Create sample generated code
    let generated_code = GeneratedCode {
        content: r#"
fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fibonacci() {
        assert_eq!(fibonacci(0), 0);
        assert_eq!(fibonacci(1), 1);
        assert_eq!(fibonacci(10), 55);
    }
}
        "#.to_string(),
        language: "rust".to_string(),
        file_path: Some("src/fibonacci.rs".to_string()),
        dependencies: vec![],
        metadata: HashMap::new(),
    };
    
    // Create evaluation context
    let evaluation_context = EvaluationContext {
        project_path: ".".to_string(),
        target_files: vec!["src/fibonacci.rs".to_string()],
        baseline_commit: Some("main".to_string()),
        environment: EvaluationEnvironment {
            platform: "Linux".to_string(),
            architecture: "x86_64".to_string(),
            rust_version: "1.70.0".to_string(),
            dependencies: HashMap::new(),
            build_profile: BuildProfile::Debug,
        },
        metadata: HashMap::new(),
    };
    
    // Run evaluation
    println!("Running evaluation...");
    match framework.evaluate_generated_code(&generated_code, &evaluation_context).await {
        Ok(result) => {
            println!("Evaluation completed successfully!");
            println!("Overall Score: {:.1}", result.overall_score);
            println!("Success: {}", result.success);
            println!("Issues Found: {}", result.issues.len());
            
            for issue in &result.issues {
                println!("  - {}: {}", issue.severity, issue.description);
            }
            
            println!("\nRecommendations:");
            for rec in &result.recommendations {
                println!("  - {}", rec);
            }
            
            // Generate HTML report
            println!("\nGenerating HTML report...");
            match framework.generate_report(&[result], devkit::evaluation::reports::ReportFormat::Html).await {
                Ok(html_report) => {
                    println!("Report generated successfully!");
                    println!("Report length: {} characters", html_report.len());
                    
                    // Save report to file
                    std::fs::write("evaluation_report.html", html_report)?;
                    println!("Report saved to evaluation_report.html");
                }
                Err(e) => {
                    println!("Failed to generate report: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Evaluation failed: {}", e);
        }
    }
    
    Ok(())
}