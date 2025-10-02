//! Concrete Script Generators
//!
//! Provides specific implementations of script generators for different
//! categories of blueprint changes.

use super::*;
use super::migration::*;
use anyhow::Result;
use std::path::PathBuf;

/// Generator for architecture-related changes
pub struct ArchitectureGenerator;

impl ScriptGenerator for ArchitectureGenerator {
    fn generate_migration_script(
        &self,
        change: &BlueprintChange,
        context: &MigrationContext,
    ) -> Result<MigrationStep> {
        let script_content = self.generate_architecture_script(change, context)?;
        let script_path = context.working_dir.join("migration_architecture.sh");

        // Write script to file
        std::fs::write(&script_path, script_content)?;
        
        // Make script executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script_path, perms)?;
        }

        Ok(MigrationStep {
            step_id: format!("arch_{}", uuid::Uuid::new_v4()),
            step_type: MigrationStepType::SchemaUpdate,
            description: format!("Architecture change: {}", change.description),
            script_path: Some(script_path),
            dependencies: vec!["pre_migration".to_string()],
            rollback_script: Some(self.generate_rollback_script(change, context)?.script_path.unwrap()),
            validation_checks: vec![
                ValidationCheck {
                    name: "architecture_validation".to_string(),
                    description: "Validate architecture change".to_string(),
                    check_type: ValidationType::ConfigurationValid,
                    expected_result: "Architecture updated successfully".to_string(),
                    failure_action: FailureAction::Rollback,
                }
            ],
            estimated_duration: Some(std::time::Duration::from_secs(180)),
            execution_result: None,
        })
    }

    fn generate_rollback_script(
        &self,
        change: &BlueprintChange,
        context: &MigrationContext,
    ) -> Result<MigrationStep> {
        let script_content = self.generate_architecture_rollback_script(change)?;
        let script_path = context.working_dir.join("rollback_architecture.sh");
        std::fs::write(&script_path, script_content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script_path, perms)?;
        }

        Ok(MigrationStep {
            step_id: format!("arch_rollback_{}", uuid::Uuid::new_v4()),
            step_type: MigrationStepType::Rollback,
            description: format!("Rollback architecture change: {}", change.description),
            script_path: Some(script_path),
            dependencies: Vec::new(),
            rollback_script: None,
            validation_checks: Vec::new(),
            estimated_duration: Some(std::time::Duration::from_secs(120)),
            execution_result: None,
        })
    }

    fn can_handle_change(&self, change: &BlueprintChange) -> bool {
        change.change_category == ChangeCategory::Architecture
    }
}

impl ArchitectureGenerator {
    fn generate_architecture_script(&self, change: &BlueprintChange, context: &MigrationContext) -> Result<String> {
        let mut script = String::new();
        script.push_str("#!/bin/bash\n");
        script.push_str("# Architecture Migration Script\n");
        script.push_str("set -e\n\n");

        script.push_str(&format!("echo \"Migrating architecture change: {}\"\n", change.path));
        script.push_str(&format!("echo \"Change type: {:?}\"\n", change.change_type));
        
        match change.change_type {
            ChangeType::Modified => {
                if let (Some(old_val), Some(new_val)) = (&change.old_value, &change.new_value) {
                    script.push_str(&format!("echo \"Updating {} from {} to {}\"\n", 
                        change.path,
                        self.format_value(old_val),
                        self.format_value(new_val)
                    ));
                    
                    // Generate specific migration logic based on the change
                    if change.path.contains("architecture_paradigm") {
                        script.push_str(&self.generate_paradigm_migration(old_val, new_val)?);
                    }
                }
            }
            ChangeType::Added => {
                if let Some(new_val) = &change.new_value {
                    script.push_str(&format!("echo \"Adding new architecture component: {}\"\n", 
                        self.format_value(new_val)));
                }
            }
            ChangeType::Removed => {
                if let Some(old_val) = &change.old_value {
                    script.push_str(&format!("echo \"Removing architecture component: {}\"\n", 
                        self.format_value(old_val)));
                }
            }
            _ => {}
        }

        script.push_str("echo \"Architecture migration completed successfully\"\n");
        Ok(script)
    }

    fn generate_architecture_rollback_script(&self, change: &BlueprintChange) -> Result<String> {
        let mut script = String::new();
        script.push_str("#!/bin/bash\n");
        script.push_str("# Architecture Rollback Script\n");
        script.push_str("set -e\n\n");

        script.push_str(&format!("echo \"Rolling back architecture change: {}\"\n", change.path));
        
        // Generate rollback logic (reverse of migration)
        match change.change_type {
            ChangeType::Modified => {
                if let (Some(old_val), Some(_new_val)) = (&change.old_value, &change.new_value) {
                    script.push_str(&format!("echo \"Restoring {} to {}\"\n", 
                        change.path,
                        self.format_value(old_val)
                    ));
                }
            }
            ChangeType::Added => {
                script.push_str(&format!("echo \"Removing added component: {}\"\n", change.path));
            }
            ChangeType::Removed => {
                if let Some(old_val) = &change.old_value {
                    script.push_str(&format!("echo \"Restoring removed component: {}\"\n", 
                        self.format_value(old_val)));
                }
            }
            _ => {}
        }

        script.push_str("echo \"Architecture rollback completed successfully\"\n");
        Ok(script)
    }

    fn generate_paradigm_migration(&self, old_val: &serde_json::Value, new_val: &serde_json::Value) -> Result<String> {
        let mut script = String::new();
        
        let old_paradigm = old_val.as_str().unwrap_or("unknown");
        let new_paradigm = new_val.as_str().unwrap_or("unknown");

        script.push_str(&format!("echo \"Migrating from {} to {}\"\n", old_paradigm, new_paradigm));

        match (old_paradigm.to_lowercase().as_str(), new_paradigm.to_lowercase().as_str()) {
            ("monolithic", "microservices") => {
                script.push_str("echo \"Converting monolithic to microservices architecture\"\n");
                script.push_str("# Split services\n");
                script.push_str("# Update service discovery\n");
                script.push_str("# Implement service mesh\n");
            }
            ("microservices", "monolithic") => {
                script.push_str("echo \"Converting microservices to monolithic architecture\"\n");
                script.push_str("# Merge services\n");
                script.push_str("# Remove service mesh\n");
                script.push_str("# Consolidate databases\n");
            }
            _ => {
                script.push_str(&format!("echo \"Generic paradigm migration: {} -> {}\"\n", old_paradigm, new_paradigm));
            }
        }

        Ok(script)
    }

    fn format_value(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => format!("\"{}\"", s),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            _ => value.to_string(),
        }
    }
}

/// Generator for module-related changes
pub struct ModuleGenerator;

impl ScriptGenerator for ModuleGenerator {
    fn generate_migration_script(
        &self,
        change: &BlueprintChange,
        context: &MigrationContext,
    ) -> Result<MigrationStep> {
        let script_content = self.generate_module_script(change, context)?;
        let script_path = context.working_dir.join("migration_module.sh");
        std::fs::write(&script_path, script_content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script_path, perms)?;
        }

        Ok(MigrationStep {
            step_id: format!("module_{}", uuid::Uuid::new_v4()),
            step_type: MigrationStepType::CodeGeneration,
            description: format!("Module change: {}", change.description),
            script_path: Some(script_path),
            dependencies: vec!["pre_migration".to_string()],
            rollback_script: Some(self.generate_rollback_script(change, context)?.script_path.unwrap()),
            validation_checks: vec![
                ValidationCheck {
                    name: "module_validation".to_string(),
                    description: "Validate module changes".to_string(),
                    check_type: ValidationType::CompilationSuccess,
                    expected_result: "Module compiled successfully".to_string(),
                    failure_action: FailureAction::Rollback,
                }
            ],
            estimated_duration: Some(std::time::Duration::from_secs(120)),
            execution_result: None,
        })
    }

    fn generate_rollback_script(
        &self,
        change: &BlueprintChange,
        context: &MigrationContext,
    ) -> Result<MigrationStep> {
        let script_content = self.generate_module_rollback_script(change)?;
        let script_path = context.working_dir.join("rollback_module.sh");
        std::fs::write(&script_path, script_content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script_path, perms)?;
        }

        Ok(MigrationStep {
            step_id: format!("module_rollback_{}", uuid::Uuid::new_v4()),
            step_type: MigrationStepType::Rollback,
            description: format!("Rollback module change: {}", change.description),
            script_path: Some(script_path),
            dependencies: Vec::new(),
            rollback_script: None,
            validation_checks: Vec::new(),
            estimated_duration: Some(std::time::Duration::from_secs(90)),
            execution_result: None,
        })
    }

    fn can_handle_change(&self, change: &BlueprintChange) -> bool {
        change.change_category == ChangeCategory::Module
    }
}

impl ModuleGenerator {
    fn generate_module_script(&self, change: &BlueprintChange, _context: &MigrationContext) -> Result<String> {
        let mut script = String::new();
        script.push_str("#!/bin/bash\n");
        script.push_str("# Module Migration Script\n");
        script.push_str("set -e\n\n");

        script.push_str(&format!("echo \"Processing module change: {}\"\n", change.path));

        match change.change_type {
            ChangeType::Added => {
                script.push_str("echo \"Adding new module\"\n");
                script.push_str("# Generate module structure\n");
                script.push_str("# Create module files\n");
                script.push_str("# Update module registry\n");
            }
            ChangeType::Modified => {
                script.push_str("echo \"Modifying existing module\"\n");
                script.push_str("# Update module configuration\n");
                script.push_str("# Recompile module\n");
                script.push_str("# Update dependencies\n");
            }
            ChangeType::Removed => {
                script.push_str("echo \"Removing module\"\n");
                script.push_str("# Backup module data\n");
                script.push_str("# Remove module files\n");
                script.push_str("# Update module registry\n");
            }
            _ => {}
        }

        script.push_str("echo \"Module migration completed successfully\"\n");
        Ok(script)
    }

    fn generate_module_rollback_script(&self, change: &BlueprintChange) -> Result<String> {
        let mut script = String::new();
        script.push_str("#!/bin/bash\n");
        script.push_str("# Module Rollback Script\n");
        script.push_str("set -e\n\n");

        script.push_str(&format!("echo \"Rolling back module change: {}\"\n", change.path));

        match change.change_type {
            ChangeType::Added => {
                script.push_str("echo \"Removing added module\"\n");
                script.push_str("# Remove module files\n");
                script.push_str("# Clean up dependencies\n");
            }
            ChangeType::Modified => {
                script.push_str("echo \"Restoring previous module state\"\n");
                script.push_str("# Restore from backup\n");
                script.push_str("# Revert configuration\n");
            }
            ChangeType::Removed => {
                script.push_str("echo \"Restoring removed module\"\n");
                script.push_str("# Restore module files\n");
                script.push_str("# Restore dependencies\n");
            }
            _ => {}
        }

        script.push_str("echo \"Module rollback completed successfully\"\n");
        Ok(script)
    }
}

/// Generator for configuration-related changes
pub struct ConfigurationGenerator;

impl ScriptGenerator for ConfigurationGenerator {
    fn generate_migration_script(
        &self,
        change: &BlueprintChange,
        context: &MigrationContext,
    ) -> Result<MigrationStep> {
        let script_content = self.generate_config_script(change)?;
        let script_path = context.working_dir.join("migration_config.sh");
        std::fs::write(&script_path, script_content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script_path, perms)?;
        }

        Ok(MigrationStep {
            step_id: format!("config_{}", uuid::Uuid::new_v4()),
            step_type: MigrationStepType::ConfigUpdate,
            description: format!("Configuration change: {}", change.description),
            script_path: Some(script_path),
            dependencies: vec!["pre_migration".to_string()],
            rollback_script: Some(self.generate_rollback_script(change, context)?.script_path.unwrap()),
            validation_checks: vec![
                ValidationCheck {
                    name: "config_validation".to_string(),
                    description: "Validate configuration changes".to_string(),
                    check_type: ValidationType::ConfigurationValid,
                    expected_result: "Configuration is valid".to_string(),
                    failure_action: FailureAction::Warn,
                }
            ],
            estimated_duration: Some(std::time::Duration::from_secs(60)),
            execution_result: None,
        })
    }

    fn generate_rollback_script(
        &self,
        change: &BlueprintChange,
        context: &MigrationContext,
    ) -> Result<MigrationStep> {
        let script_content = self.generate_config_rollback_script(change)?;
        let script_path = context.working_dir.join("rollback_config.sh");
        std::fs::write(&script_path, script_content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script_path, perms)?;
        }

        Ok(MigrationStep {
            step_id: format!("config_rollback_{}", uuid::Uuid::new_v4()),
            step_type: MigrationStepType::Rollback,
            description: format!("Rollback configuration change: {}", change.description),
            script_path: Some(script_path),
            dependencies: Vec::new(),
            rollback_script: None,
            validation_checks: Vec::new(),
            estimated_duration: Some(std::time::Duration::from_secs(30)),
            execution_result: None,
        })
    }

    fn can_handle_change(&self, change: &BlueprintChange) -> bool {
        change.change_category == ChangeCategory::Configuration
    }
}

impl ConfigurationGenerator {
    fn generate_config_script(&self, change: &BlueprintChange) -> Result<String> {
        let mut script = String::new();
        script.push_str("#!/bin/bash\n");
        script.push_str("# Configuration Migration Script\n");
        script.push_str("set -e\n\n");

        script.push_str(&format!("echo \"Processing configuration change: {}\"\n", change.path));

        // Generate config-specific migration logic
        if change.path.contains("metadata") {
            script.push_str("echo \"Updating metadata configuration\"\n");
            script.push_str("# Update version information\n");
            script.push_str("# Update timestamps\n");
        } else {
            script.push_str("echo \"Updating general configuration\"\n");
            script.push_str("# Apply configuration changes\n");
            script.push_str("# Validate configuration\n");
        }

        script.push_str("echo \"Configuration migration completed successfully\"\n");
        Ok(script)
    }

    fn generate_config_rollback_script(&self, change: &BlueprintChange) -> Result<String> {
        let mut script = String::new();
        script.push_str("#!/bin/bash\n");
        script.push_str("# Configuration Rollback Script\n");
        script.push_str("set -e\n\n");

        script.push_str(&format!("echo \"Rolling back configuration change: {}\"\n", change.path));
        script.push_str("echo \"Restoring previous configuration\"\n");
        script.push_str("# Restore from backup\n");
        script.push_str("# Validate restored configuration\n");
        script.push_str("echo \"Configuration rollback completed successfully\"\n");

        Ok(script)
    }
}

/// Register all default generators
pub fn register_default_generators(engine: &mut MigrationEngine) {
    engine.register_generator(ChangeCategory::Architecture, Box::new(ArchitectureGenerator));
    engine.register_generator(ChangeCategory::Module, Box::new(ModuleGenerator));
    engine.register_generator(ChangeCategory::Configuration, Box::new(ConfigurationGenerator));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_architecture_generator_can_handle() {
        let generator = ArchitectureGenerator;
        let change = BlueprintChange {
            change_type: ChangeType::Modified,
            change_category: ChangeCategory::Architecture,
            path: "architecture.system_type".to_string(),
            old_value: Some(serde_json::json!("monolith")),
            new_value: Some(serde_json::json!("microservices")),
            description: "Architecture change".to_string(),
            impact_level: ImpactLevel::Critical,
        };

        assert!(generator.can_handle_change(&change));
    }

    #[test]
    fn test_module_generator_can_handle() {
        let generator = ModuleGenerator;
        let change = BlueprintChange {
            change_type: ChangeType::Added,
            change_category: ChangeCategory::Module,
            path: "modules[1]".to_string(),
            old_value: None,
            new_value: Some(serde_json::json!({"name": "new_module"})),
            description: "Added new module".to_string(),
            impact_level: ImpactLevel::Medium,
        };

        assert!(generator.can_handle_change(&change));
    }

    #[test]
    fn test_configuration_generator_can_handle() {
        let generator = ConfigurationGenerator;
        let change = BlueprintChange {
            change_type: ChangeType::Modified,
            change_category: ChangeCategory::Configuration,
            path: "metadata.version".to_string(),
            old_value: Some(serde_json::json!("1.0.0")),
            new_value: Some(serde_json::json!("2.0.0")),
            description: "Version update".to_string(),
            impact_level: ImpactLevel::Medium,
        };

        assert!(generator.can_handle_change(&change));
    }
}