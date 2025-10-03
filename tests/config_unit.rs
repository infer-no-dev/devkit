//! Unit tests for the configuration system using the comprehensive testing framework
//!
//! This module tests configuration functionality including:
//! - Configuration loading and validation
//! - Environment-specific configurations
//! - Hot-reloading capabilities
//! - Configuration merging and inheritance
//! - Backup and restore functionality

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use devkit_env::{
    config::{Config, ConfigError, ConfigManager},
    testing::{
        MockDataFactory, PropertyGenerator, TestAssertions, TestContext, TestEnvironment,
        TestRunner, TestSuiteConfig,
    },
};

/// Test suite configuration for config unit tests
fn create_test_config() -> TestSuiteConfig {
    TestSuiteConfig {
        parallel_execution: true,
        max_parallel_tests: 2,
        timeout: Duration::from_secs(10),
        retry_count: 0,
        capture_output: true,
        performance_benchmarks: false,
    }
}

/// Test configuration loading from file
async fn test_config_loading(ctx: TestContext) -> Result<(), String> {
    let temp_dir = ctx.create_temp_dir().map_err(|e| e.to_string())?;

    // Create test configuration file
    let config_content = r#"
[general]
log_level = "info"
auto_save = true
backup_enabled = true
telemetry_enabled = false

[agents]
max_concurrent_agents = 3
agent_timeout_seconds = 30
default_agent_priority = "normal"

[agents.notification_settings]
enabled = true
sound_enabled = false

[codegen]
[codegen.default_style]
indentation = "spaces"
indent_size = 2

[shell]
command_timeout = 15

[ui]
theme = "light"
"#;

    let config_path = temp_dir.join("test_config.toml");
    std::fs::write(&config_path, config_content).map_err(|e| e.to_string())?;

    // Load configuration
    let mut config_manager = ConfigManager::new();
    let config = config_manager
        .load_from_path(&config_path)
        .map_err(|e| e.to_string())?;

    // Verify loaded values
    assert_eq!(config.general.log_level, "info");
    assert_eq!(config.agents.max_concurrent_agents, 3);
    assert_eq!(config.codegen.default_style.indent_size, 2);
    assert_eq!(config.shell.command_timeout, 15);
    assert_eq!(config.ui.theme, "light");

    ctx.add_metadata("config_loaded", "true");
    ctx.add_metadata("config_path", config_path.to_string_lossy().as_ref());

    Ok(())
}

/// Test configuration validation
async fn test_config_validation(ctx: TestContext) -> Result<(), String> {
    let mock_config = MockDataFactory::create_mock_config();

    // Test valid configuration
    let mut config_manager = ConfigManager::new();
    config_manager.config = Some(mock_config.clone());

    let validation_result = config_manager.validate();
    if validation_result.is_err() {
        return Err(format!(
            "Valid configuration failed validation: {:?}",
            validation_result
        ));
    }

    // Test invalid configuration
    let mut invalid_config = mock_config.clone();
    invalid_config.agents.max_concurrent_agents = 0; // Invalid value
    invalid_config.shell.command_timeout = 0; // Invalid timeout

    config_manager.config = Some(invalid_config);
    let invalid_validation_result = config_manager.validate();

    if invalid_validation_result.is_ok() {
        return Err(
            "Invalid configuration passed validation when it should have failed".to_string(),
        );
    }

    ctx.add_metadata("valid_config_passed", "true");
    ctx.add_metadata("invalid_config_failed", "true");

    Ok(())
}

/// Test environment-specific configuration loading
async fn test_environment_config(ctx: TestContext) -> Result<(), String> {
    let temp_dir = ctx.create_temp_dir().map_err(|e| e.to_string())?;

    // Create base configuration
    let base_config = r#"
[general]
log_level = "info"
auto_save = true

[agents]
max_concurrent_agents = 3
"#;

    // Create development environment config
    let dev_config = r#"
[general]
log_level = "debug"

[agents]
max_concurrent_agents = 5
"#;

    // Create production environment config
    let prod_config = r#"
[general]
log_level = "warn"
telemetry_enabled = true

[agents]
max_concurrent_agents = 10
"#;

    let base_path = temp_dir.join("config.toml");
    let dev_path = temp_dir.join("config.development.toml");
    let prod_path = temp_dir.join("config.production.toml");

    std::fs::write(&base_path, base_config).map_err(|e| e.to_string())?;
    std::fs::write(&dev_path, dev_config).map_err(|e| e.to_string())?;
    std::fs::write(&prod_path, prod_config).map_err(|e| e.to_string())?;

    // Test development environment loading
    let mut config_manager = ConfigManager::new();
    config_manager.set_config_dir(temp_dir.clone());

    config_manager
        .switch_environment("development")
        .map_err(|e| e.to_string())?;
    let dev_loaded_config = config_manager
        .get_config()
        .ok_or("Failed to get dev config")?;

    assert_eq!(dev_loaded_config.general.log_level, "debug");
    assert_eq!(dev_loaded_config.agents.max_concurrent_agents, 5);

    // Test production environment loading
    config_manager
        .switch_environment("production")
        .map_err(|e| e.to_string())?;
    let prod_loaded_config = config_manager
        .get_config()
        .ok_or("Failed to get prod config")?;

    assert_eq!(prod_loaded_config.general.log_level, "warn");
    assert_eq!(prod_loaded_config.agents.max_concurrent_agents, 10);
    assert_eq!(prod_loaded_config.general.telemetry_enabled, true);

    ctx.add_metadata("environments_tested", "2");
    ctx.add_metadata("dev_log_level", "debug");
    ctx.add_metadata("prod_log_level", "warn");
    ctx.add_metadata("environment_switching", "successful");

    Ok(())
}

/// Test configuration hot-reloading
async fn test_config_hot_reload(ctx: TestContext) -> Result<(), String> {
    let temp_dir = ctx.create_temp_dir().map_err(|e| e.to_string())?;

    // Create initial configuration
    let initial_config = r#"
[general]
log_level = "info"
auto_save = false
"#;

    let config_path = temp_dir.join("hot_reload_test.toml");
    std::fs::write(&config_path, initial_config).map_err(|e| e.to_string())?;

    // Load configuration with hot reload enabled
    let mut config_manager = ConfigManager::new();
    config_manager
        .load_from_path(&config_path)
        .map_err(|e| e.to_string())?;
    config_manager.enable_hot_reload();

    let initial_loaded = config_manager
        .get_config()
        .ok_or("Failed to get initial config")?;
    assert_eq!(initial_loaded.general.log_level, "info");
    assert!(!initial_loaded.general.auto_save);

    // Modify configuration file
    let updated_config = r#"
[general]
log_level = "debug"
auto_save = true
"#;

    std::fs::write(&config_path, updated_config).map_err(|e| e.to_string())?;

    // Wait a bit for file system change detection
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Check if hot reload detects changes
    let hot_reload_detected = config_manager
        .check_and_reload()
        .map_err(|e| e.to_string())?;
    if !hot_reload_detected {
        return Err("Hot reload did not detect configuration changes".to_string());
    }

    let reloaded_config = config_manager
        .get_config()
        .ok_or("Failed to get reloaded config")?;
    assert_eq!(reloaded_config.general.log_level, "debug");
    assert!(reloaded_config.general.auto_save);

    ctx.add_metadata("hot_reload_enabled", "true");
    ctx.add_metadata("changes_detected", "true");
    ctx.add_metadata("config_reloaded", "true");

    Ok(())
}

/// Test configuration backup and restore
async fn test_config_backup_restore(ctx: TestContext) -> Result<(), String> {
    let temp_dir = ctx.create_temp_dir().map_err(|e| e.to_string())?;

    let original_config = MockDataFactory::create_mock_config();

    // Create configuration manager and set up initial config
    let mut config_manager = ConfigManager::new();
    config_manager.config = Some(original_config.clone());

    // Create backup
    let backup_result = config_manager.create_backup();
    if backup_result.is_err() {
        return Err(format!("Failed to create backup: {:?}", backup_result));
    }

    // Verify backup exists
    let backup = config_manager.get_backup();
    if backup.is_none() {
        return Err("Backup was not created properly".to_string());
    }

    // Modify current configuration
    if let Some(ref mut config) = config_manager.config {
        config.general.log_level = "trace".to_string();
        config.agents.max_concurrent_agents = 999;
    }

    // Verify configuration was changed
    let modified_config = config_manager
        .get_config()
        .ok_or("Failed to get modified config")?;
    assert_eq!(modified_config.general.log_level, "trace");
    assert_eq!(modified_config.agents.max_concurrent_agents, 999);

    // Restore from backup
    let restore_result = config_manager.restore_from_backup();
    if restore_result.is_err() {
        return Err(format!(
            "Failed to restore from backup: {:?}",
            restore_result
        ));
    }

    // Verify restoration
    let restored_config = config_manager
        .get_config()
        .ok_or("Failed to get restored config")?;
    assert_eq!(
        restored_config.general.log_level,
        original_config.general.log_level
    );
    assert_eq!(
        restored_config.agents.max_concurrent_agents,
        original_config.agents.max_concurrent_agents
    );

    ctx.add_metadata("backup_created", "true");
    ctx.add_metadata("config_modified", "true");
    ctx.add_metadata("restore_successful", "true");

    Ok(())
}

/// Test configuration value getting and setting by path
async fn test_config_path_operations(ctx: TestContext) -> Result<(), String> {
    let mut config_manager = ConfigManager::new();
    let original_config = MockDataFactory::create_mock_config();
    config_manager.config = Some(original_config.clone());

    // Test getting values by path
    let log_level = config_manager.get_value("general.log_level");
    if log_level != Some(serde_json::Value::String("debug".to_string())) {
        return Err("Failed to get value by path".to_string());
    }

    let max_agents = config_manager.get_value("agents.max_concurrent_agents");
    if max_agents != Some(serde_json::Value::Number(serde_json::Number::from(5))) {
        return Err("Failed to get nested value by path".to_string());
    }

    // Test setting values by path
    config_manager
        .set_value(
            "general.log_level",
            serde_json::Value::String("error".to_string()),
        )
        .map_err(|e| e.to_string())?;

    config_manager
        .set_value(
            "agents.max_concurrent_agents",
            serde_json::Value::Number(serde_json::Number::from(15)),
        )
        .map_err(|e| e.to_string())?;

    // Verify changes
    let updated_log_level = config_manager.get_value("general.log_level");
    if updated_log_level != Some(serde_json::Value::String("error".to_string())) {
        return Err("Failed to set value by path".to_string());
    }

    let updated_max_agents = config_manager.get_value("agents.max_concurrent_agents");
    if updated_max_agents != Some(serde_json::Value::Number(serde_json::Number::from(15))) {
        return Err("Failed to set nested value by path".to_string());
    }

    ctx.add_metadata("path_operations_tested", "4");
    ctx.add_metadata("get_operations", "2");
    ctx.add_metadata("set_operations", "2");

    Ok(())
}

/// Test property-based configuration generation and validation
async fn test_property_based_config_validation(ctx: TestContext) -> Result<(), String> {
    let generator = PropertyGenerator::with_seed(123);

    // Generate random configuration values within valid ranges
    let log_levels = vec!["trace", "debug", "info", "warn", "error"];
    let themes = vec!["dark", "light", "auto"];
    let shell_types = vec!["bash", "zsh", "fish"];

    let agent_counts = generator.generate_numbers(10, 1, 20);
    let timeouts = generator.generate_numbers(10, 5, 300);

    // Test multiple random configurations
    for i in 0..5 {
        let mut test_config = MockDataFactory::create_mock_config();

        // Apply random but valid values
        test_config.agents.max_concurrent_agents = agent_counts[i % agent_counts.len()] as usize;
        test_config.shell.command_timeout = timeouts[i % timeouts.len()] as u64;
        test_config.general.log_level = log_levels[i % log_levels.len()].to_string();
        test_config.ui.theme = themes[i % themes.len()].to_string();

        // Validate each generated configuration
        let mut config_manager = ConfigManager::new();
        config_manager.config = Some(test_config);

        let validation_result = config_manager.validate();
        if validation_result.is_err() {
            return Err(format!(
                "Generated configuration {} failed validation: {:?}",
                i, validation_result
            ));
        }
    }

    ctx.add_metadata("property_configs_tested", "5");
    ctx.add_metadata("agent_count_range", "1-20");
    ctx.add_metadata("timeout_range", "5-300");
    ctx.add_metadata("property_validation_passed", "true");

    Ok(())
}

/// Test configuration performance with large configurations
async fn test_config_performance(ctx: TestContext) -> Result<(), String> {
    let start_time = std::time::Instant::now();

    // Create a large configuration with many entries
    let mut large_config = MockDataFactory::create_mock_config();

    // Add many custom agents
    for i in 0..100 {
        large_config
            .agents
            .custom_agents
            .push(format!("custom_agent_{}", i));
    }

    // Add many environment variables
    for i in 0..50 {
        large_config
            .shell
            .environment_variables
            .insert(format!("ENV_VAR_{}", i), format!("value_{}", i));
    }

    // Add many keybindings
    for i in 0..200 {
        large_config
            .keybindings
            .insert(format!("keybinding_{}", i), format!("action_{}", i));
    }

    let setup_time = start_time.elapsed();

    // Test loading performance
    let load_start = std::time::Instant::now();
    let mut config_manager = ConfigManager::new();
    config_manager.config = Some(large_config.clone());
    let load_time = load_start.elapsed();

    // Test validation performance
    let validation_start = std::time::Instant::now();
    let validation_result = config_manager.validate();
    let validation_time = validation_start.elapsed();

    if validation_result.is_err() {
        return Err(format!(
            "Large configuration validation failed: {:?}",
            validation_result
        ));
    }

    // Test serialization performance
    let serialization_start = std::time::Instant::now();
    let _serialized = serde_json::to_string(&large_config).map_err(|e| e.to_string())?;
    let serialization_time = serialization_start.elapsed();

    // Assert performance is within reasonable bounds
    TestAssertions::assert_execution_time_within(
        load_time,
        Duration::from_nanos(1),
        Duration::from_millis(50),
    )?;

    TestAssertions::assert_execution_time_within(
        validation_time,
        Duration::from_nanos(1),
        Duration::from_millis(100),
    )?;

    ctx.add_metadata("setup_time_ms", &setup_time.as_millis().to_string());
    ctx.add_metadata("load_time_ms", &load_time.as_millis().to_string());
    ctx.add_metadata(
        "validation_time_ms",
        &validation_time.as_millis().to_string(),
    );
    ctx.add_metadata(
        "serialization_time_ms",
        &serialization_time.as_millis().to_string(),
    );
    ctx.add_metadata("custom_agents_count", "100");
    ctx.add_metadata("env_vars_count", "50");
    ctx.add_metadata("keybindings_count", "200");

    Ok(())
}

/// Main unit test runner
#[tokio::test]
async fn run_config_unit_tests() {
    let config = create_test_config();
    let runner = TestRunner::new(config);

    let tests = vec![
        ("config_loading", test_config_loading),
        ("config_validation", test_config_validation),
        ("environment_config", test_environment_config),
        ("config_hot_reload", test_config_hot_reload),
        ("config_backup_restore", test_config_backup_restore),
        ("config_path_operations", test_config_path_operations),
        (
            "property_based_config_validation",
            test_property_based_config_validation,
        ),
        ("config_performance", test_config_performance),
    ];

    let report = runner.run_test_suite(tests).await;

    // Print comprehensive test report
    report.print_summary();

    // Export detailed results
    if let Ok(json_report) = report.export_json() {
        let report_path = std::path::PathBuf::from("/tmp/config_unit_test_report.json");
        if let Err(e) = std::fs::write(&report_path, json_report) {
            eprintln!(
                "Failed to write test report to {}: {}",
                report_path.display(),
                e
            );
        }
    }

    // Assert all tests passed
    assert!(report.all_passed(), "Some configuration unit tests failed");
    assert!(report.total_tests >= 8, "Expected at least 8 tests to run");
}

/// Test configuration edge cases
#[tokio::test]
async fn test_config_edge_cases() {
    let config = create_test_config();
    let runner = TestRunner::new(config);

    let edge_case_test = |ctx: TestContext| async move {
        // Test empty configuration file
        let temp_dir = ctx.create_temp_dir().map_err(|e| e.to_string())?;
        let empty_config_path = temp_dir.join("empty.toml");
        std::fs::write(&empty_config_path, "").map_err(|e| e.to_string())?;

        let mut config_manager = ConfigManager::new();
        let empty_result = config_manager.load_from_path(&empty_config_path);

        // Should handle empty files gracefully
        if empty_result.is_ok() {
            ctx.add_metadata("empty_config_handled", "gracefully");
        }

        // Test non-existent configuration file
        let nonexistent_path = temp_dir.join("does_not_exist.toml");
        let missing_result = config_manager.load_from_path(&nonexistent_path);

        if missing_result.is_err() {
            ctx.add_metadata("missing_config_handled", "with_error");
        }

        // Test malformed configuration
        let malformed_config = "this is not valid TOML [[[";
        let malformed_path = temp_dir.join("malformed.toml");
        std::fs::write(&malformed_path, malformed_config).map_err(|e| e.to_string())?;

        let malformed_result = config_manager.load_from_path(&malformed_path);
        if malformed_result.is_err() {
            ctx.add_metadata("malformed_config_handled", "with_error");
        }

        ctx.add_metadata("edge_cases_tested", "3");
        Ok(())
    };

    let result = runner.run_test("config_edge_cases", edge_case_test).await;
    assert!(
        result.success,
        "Configuration edge case test failed: {:?}",
        result.error_message
    );
}
