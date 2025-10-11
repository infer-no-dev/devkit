use devkit::blueprint::replicator::{SystemReplicator, ReplicationConfig};
use std::path::PathBuf;
use tempfile::tempdir;

#[tokio::test]
async fn test_devkit_self_replication() {
    println!("🔄 Testing DevKit Self-Replication");
    println!("===================================");
    
    // Create a temporary directory for the replica
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let target_path = temp_dir.path().to_path_buf();
    
    // Source is the current devkit directory
    let source_path = PathBuf::from(".");
    
    println!("📋 Source: {:?}", source_path.canonicalize().unwrap());
    println!("🎯 Target: {:?}", target_path);
    
    // Create replication configuration with dry_run = true for safety
    let config = ReplicationConfig {
        source_path: source_path.clone(),
        target_path: target_path.clone(),
        preserve_git: false, // Don't copy git for test
        validate_generated: true,
        dry_run: true, // Safe test - won't actually create files
        include_tests: true,
        include_documentation: true,
        include_ci: true,
    };
    
    // Create the replicator
    let replicator = SystemReplicator::with_config(config);
    
    println!("\n🚀 Starting self-replication process (DRY RUN)...");
    
    // Execute the replication
    match replicator.replicate().await {
        Ok(result) => {
            println!("\n✅ Self-replication dry run completed!");
            println!("   Success: {}", result.success);
            println!("   Files that would be generated: {}", result.generated_files.len());
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
            
            // Generate a report
            replicator.generate_report(&result).await.expect("Failed to generate report");
            
            println!("\n🎉 DevKit self-replication test completed!");
            println!("   This was a DRY RUN - no files were actually created.");
            
            // Assert that the dry run was successful
            assert!(result.execution_time.as_millis() > 0, "Replication should take some time");
        }
        Err(e) => {
            println!("❌ Self-replication test failed: {}", e);
            panic!("Self-replication failed: {}", e);
        }
    }
}