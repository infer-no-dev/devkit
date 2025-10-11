#!/usr/bin/env rust-script
//! This script demonstrates devkit's self-replication capabilities
//! 
//! ```cargo
//! [dependencies]
//! devkit = { path = "." }
//! tokio = { version = "1", features = ["full"] }
//! anyhow = "1"
//! ```

use devkit::blueprint::{
    replicator::{SystemReplicator, ReplicationConfig}, 
    SystemBlueprint
};
use std::path::PathBuf;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔄 DevKit Self-Replication Experiment");
    println!("=====================================");
    
    // Current devkit directory
    let source_path = PathBuf::from(".");
    
    // Target directory for the replica
    let target_path = PathBuf::from("../devkit-replica");
    
    println!("📋 Source: {:?}", source_path.canonicalize()?);
    println!("🎯 Target: {:?}", target_path);
    
    // Create replication configuration
    let config = ReplicationConfig {
        source_path: source_path.clone(),
        target_path: target_path.clone(),
        preserve_git: true,
        validate_generated: true,
        dry_run: false,  // Set to true for a safe test run
        include_tests: true,
        include_documentation: true,
        include_ci: true,
    };
    
    // Create the replicator
    let replicator = SystemReplicator::with_config(config);
    
    println!("\n🚀 Starting self-replication process...");
    
    // Execute the replication
    match replicator.replicate().await {
        Ok(result) => {
            println!("\n✅ Self-replication completed!");
            println!("   Success: {}", result.success);
            println!("   Files generated: {}", result.generated_files.len());
            println!("   Execution time: {:?}", result.execution_time);
            
            if !result.warnings.is_empty() {
                println!("\n⚠️  Warnings:");
                for warning in &result.warnings {
                    println!("   - {}", warning);
                }
            }
            
            if !result.errors.is_empty() {
                println!("\n❌ Errors:");
                for error in &result.errors {
                    println!("   - {}", error);
                }
            }
            
            // Generate a detailed report
            replicator.generate_report(&result).await?;
            
            println!("\n🎉 DevKit has successfully replicated itself!");
            println!("📊 Check the REPLICATION_REPORT.md in the target directory for details.");
        }
        Err(e) => {
            println!("❌ Self-replication failed: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}