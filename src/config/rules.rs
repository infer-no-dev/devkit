//! Rules system with hierarchical precedence for project configuration
//!
//! This module implements a comprehensive rules system that allows:
//! - Global rules (user-wide configuration)
//! - Project rules (project-specific WARP.md files)  
//! - Directory rules (subdirectory-specific overrides)
//! - Proper precedence handling (more specific rules override general ones)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use thiserror::Error;

/// A single rule that can be applied to agent behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub content: String,
    pub priority: RulePriority,
    pub scope: RuleScope,
    pub conditions: Vec<RuleCondition>,
    pub metadata: RuleMetadata,
}

/// Priority level for rules (higher priority rules override lower priority ones)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RulePriority {
    /// Global defaults (lowest priority)
    Global = 100,
    /// User-wide settings
    User = 200,
    /// Project-wide settings (WARP.md in project root)
    Project = 300,
    /// Directory-specific settings (WARP.md in subdirectories)
    Directory = 400,
    /// Temporary/session overrides (highest priority)
    Session = 500,
}

/// Scope where the rule applies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleScope {
    /// Applies to all operations
    Global,
    /// Applies to specific file types/patterns
    FilePattern(Vec<String>),
    /// Applies to specific languages
    Language(Vec<String>),
    /// Applies to specific agent types
    Agent(Vec<String>),
    /// Applies to specific tasks/operations
    Task(Vec<String>),
}

/// Conditions under which a rule should be applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    /// Apply when working with files matching patterns
    FileMatches(String),
    /// Apply when using specific programming language
    LanguageIs(String),
    /// Apply when in specific directory
    DirectoryMatches(String),
    /// Apply when specific agent is active
    AgentIs(String),
    /// Apply when performing specific task type
    TaskIs(String),
    /// Apply based on environment variable
    EnvironmentVar(String, String),
}

/// Metadata associated with a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleMetadata {
    pub source_file: PathBuf,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub tags: Vec<String>,
}

/// Collection of rules from a specific source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSet {
    pub source_path: PathBuf,
    pub priority: RulePriority,
    pub rules: Vec<Rule>,
    pub metadata: RuleSetMetadata,
}

/// Metadata for a rule set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSetMetadata {
    pub loaded_at: chrono::DateTime<chrono::Utc>,
    pub file_hash: String,
    pub file_size: u64,
    pub is_enabled: bool,
}

/// Context for rule evaluation
#[derive(Debug, Clone)]
pub struct RuleContext {
    pub current_directory: PathBuf,
    pub file_path: Option<PathBuf>,
    pub language: Option<String>,
    pub agent_type: Option<String>,
    pub task_type: Option<String>,
    pub environment_vars: HashMap<String, String>,
}

/// Manager for loading, evaluating, and applying rules
#[derive(Debug)]
pub struct RulesManager {
    rule_sets: Vec<RuleSet>,
    global_rules: Vec<Rule>,
    cache: HashMap<PathBuf, RuleSet>,
    watch_enabled: bool,
}

/// Errors that can occur in the rules system
#[derive(Debug, Error)]
pub enum RuleError {
    #[error("Failed to load rules file: {path}")]
    LoadError { path: PathBuf },
    
    #[error("Failed to parse rules file: {path}: {error}")]
    ParseError { path: PathBuf, error: String },
    
    #[error("Rule validation failed: {rule_id}: {error}")]
    ValidationError { rule_id: String, error: String },
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl RulesManager {
    /// Create a new rules manager
    pub fn new() -> Self {
        Self {
            rule_sets: Vec::new(),
            global_rules: Vec::new(),
            cache: HashMap::new(),
            watch_enabled: false,
        }
    }
    
    /// Load rules from a directory hierarchy
    pub async fn load_rules_hierarchy(&mut self, root_path: &Path) -> Result<(), RuleError> {
        self.rule_sets.clear();
        
        // Load global/user rules first
        self.load_user_rules().await?;
        
        // Walk up the directory tree to find WARP.md files
        let mut current_path = root_path.to_path_buf();
        let mut rule_files = Vec::new();
        
        loop {
            let warp_file = current_path.join("WARP.md");
            if warp_file.exists() {
                rule_files.push((warp_file, current_path.clone()));
            }
            
            if let Some(parent) = current_path.parent() {
                current_path = parent.to_path_buf();
            } else {
                break;
            }
        }
        
        // Reverse to get proper precedence (root first, then deeper)
        rule_files.reverse();
        
        // Load each rule file with appropriate priority
        for (rule_file, dir_path) in rule_files {
            let priority = if dir_path == root_path {
                RulePriority::Project
            } else {
                RulePriority::Directory
            };
            
            if let Ok(rule_set) = self.load_warp_file(&rule_file, priority).await {
                self.rule_sets.push(rule_set);
            }
        }
        
        // Sort rule sets by priority
        self.rule_sets.sort_by_key(|rs| rs.priority);
        
        Ok(())
    }
    
    /// Load user-wide global rules
    async fn load_user_rules(&mut self) -> Result<(), RuleError> {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let global_rules_file = home_dir.join(".devkit").join("global_rules.md");
        
        if global_rules_file.exists() {
            let rule_set = self.load_warp_file(&global_rules_file, RulePriority::User).await?;
            self.rule_sets.push(rule_set);
        }
        
        Ok(())
    }
    
    /// Load a WARP.md file and parse it into rules
    async fn load_warp_file(&self, file_path: &Path, priority: RulePriority) -> Result<RuleSet, RuleError> {
        let content = fs::read_to_string(file_path)
            .map_err(|_| RuleError::LoadError { path: file_path.to_path_buf() })?;
            
        let metadata = fs::metadata(file_path)?;
        let file_hash = md5::compute(content.as_bytes());
        
        // Parse WARP.md content into rules
        let rules = self.parse_warp_content(&content, file_path)?;
        
        Ok(RuleSet {
            source_path: file_path.to_path_buf(),
            priority,
            rules,
            metadata: RuleSetMetadata {
                loaded_at: chrono::Utc::now(),
                file_hash: format!("{:x}", file_hash),
                file_size: metadata.len(),
                is_enabled: true,
            },
        })
    }
    
    /// Parse WARP.md content into structured rules
    fn parse_warp_content(&self, content: &str, source_file: &Path) -> Result<Vec<Rule>, RuleError> {
        let mut rules = Vec::new();
        let mut current_rule: Option<Rule> = None;
        let mut current_content = String::new();
        let mut rule_counter = 0;
        
        for line in content.lines() {
            let trimmed = line.trim();
            
            // Check for rule headers (## Rule: or # Rule: or similar patterns)
            if trimmed.starts_with("#") && (trimmed.contains("Rule:") || trimmed.contains("rule:")) {
                // Save previous rule if it exists
                if let Some(mut rule) = current_rule.take() {
                    rule.content = current_content.trim().to_string();
                    rules.push(rule);
                    current_content.clear();
                }
                
                // Start new rule
                rule_counter += 1;
                let rule_name = trimmed
                    .split(":")
                    .nth(1)
                    .unwrap_or(&format!("Rule {}", rule_counter))
                    .trim()
                    .to_string();
                    
                current_rule = Some(Rule {
                    id: format!("{}:{}", source_file.file_stem().unwrap_or_default().to_string_lossy(), rule_counter),
                    name: rule_name.clone(),
                    description: rule_name, // Will be updated if description is found
                    content: String::new(),
                    priority: RulePriority::Project, // Will be set by caller
                    scope: RuleScope::Global,
                    conditions: Vec::new(),
                    metadata: RuleMetadata {
                        source_file: source_file.to_path_buf(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        author: None,
                        version: None,
                        tags: Vec::new(),
                    },
                });
            } else if trimmed.starts_with("#") && current_rule.is_some() {
                // Section headers within a rule
                current_content.push_str(line);
                current_content.push('\n');
            } else if !trimmed.is_empty() {
                // Regular content
                current_content.push_str(line);
                current_content.push('\n');
                
                // If no explicit rule was found, create an implicit global rule
                if current_rule.is_none() {
                    rule_counter += 1;
                    current_rule = Some(Rule {
                        id: format!("{}:global", source_file.file_stem().unwrap_or_default().to_string_lossy()),
                        name: "Global Configuration".to_string(),
                        description: "Global project configuration from WARP.md".to_string(),
                        content: String::new(),
                        priority: RulePriority::Project,
                        scope: RuleScope::Global,
                        conditions: Vec::new(),
                        metadata: RuleMetadata {
                            source_file: source_file.to_path_buf(),
                            created_at: chrono::Utc::now(),
                            updated_at: chrono::Utc::now(),
                            author: None,
                            version: None,
                            tags: Vec::new(),
                        },
                    });
                }
            }
        }
        
        // Save final rule
        if let Some(mut rule) = current_rule.take() {
            rule.content = current_content.trim().to_string();
            rules.push(rule);
        }
        
        Ok(rules)
    }
    
    /// Get effective rules for a given context (with precedence applied)
    pub fn get_effective_rules(&self, context: &RuleContext) -> Vec<Rule> {
        let mut applicable_rules = Vec::new();
        
        // Collect all applicable rules from all rule sets
        for rule_set in &self.rule_sets {
            if !rule_set.metadata.is_enabled {
                continue;
            }
            
            for rule in &rule_set.rules {
                if self.rule_applies(rule, context) {
                    applicable_rules.push(rule.clone());
                }
            }
        }
        
        // Sort by priority (highest priority first)
        applicable_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        // Remove duplicates by ID, keeping highest priority version
        let mut seen_ids = std::collections::HashSet::new();
        applicable_rules.retain(|rule| seen_ids.insert(rule.id.clone()));
        
        applicable_rules
    }
    
    /// Check if a rule applies to the given context
    fn rule_applies(&self, rule: &Rule, context: &RuleContext) -> bool {
        // Check scope
        let scope_matches = match &rule.scope {
            RuleScope::Global => true,
            RuleScope::FilePattern(patterns) => {
                if let Some(file_path) = &context.file_path {
                    patterns.iter().any(|pattern| {
                        // Simple glob-like matching
                        file_path.to_string_lossy().contains(pattern) ||
                        glob_match(pattern, &file_path.to_string_lossy())
                    })
                } else {
                    false
                }
            },
            RuleScope::Language(languages) => {
                if let Some(lang) = &context.language {
                    languages.contains(lang)
                } else {
                    false
                }
            },
            RuleScope::Agent(agents) => {
                if let Some(agent) = &context.agent_type {
                    agents.contains(agent)
                } else {
                    false
                }
            },
            RuleScope::Task(tasks) => {
                if let Some(task) = &context.task_type {
                    tasks.contains(task)
                } else {
                    false
                }
            },
        };
        
        scope_matches && self.conditions_match(&rule.conditions, context)
    }
    
    /// Check if all conditions for a rule are met
    fn conditions_match(&self, conditions: &[RuleCondition], context: &RuleContext) -> bool {
        if conditions.is_empty() {
            return true;
        }
        
        conditions.iter().all(|condition| {
            match condition {
                RuleCondition::FileMatches(pattern) => {
                    if let Some(file_path) = &context.file_path {
                        glob_match(pattern, &file_path.to_string_lossy())
                    } else {
                        false
                    }
                },
                RuleCondition::LanguageIs(lang) => {
                    context.language.as_ref() == Some(lang)
                },
                RuleCondition::DirectoryMatches(pattern) => {
                    glob_match(pattern, &context.current_directory.to_string_lossy())
                },
                RuleCondition::AgentIs(agent) => {
                    context.agent_type.as_ref() == Some(agent)
                },
                RuleCondition::TaskIs(task) => {
                    context.task_type.as_ref() == Some(task)
                },
                RuleCondition::EnvironmentVar(key, value) => {
                    context.environment_vars.get(key) == Some(value)
                },
            }
        })
    }
    
    /// Get rules as a formatted string for AI consumption
    pub fn format_rules_for_ai(&self, context: &RuleContext) -> String {
        let effective_rules = self.get_effective_rules(context);
        
        if effective_rules.is_empty() {
            return "No specific rules apply to the current context.".to_string();
        }
        
        let mut formatted = String::new();
        formatted.push_str("# Applicable Rules\n\n");
        formatted.push_str(&format!("Rules are listed in order of precedence (highest priority first).\n"));
        formatted.push_str(&format!("Context: directory={}, file={:?}, language={:?}\n\n", 
            context.current_directory.display(),
            context.file_path,
            context.language
        ));
        
        for (i, rule) in effective_rules.iter().enumerate() {
            formatted.push_str(&format!("## Rule {}: {} (Priority: {:?})\n\n", 
                i + 1, 
                rule.name, 
                rule.priority
            ));
            formatted.push_str(&rule.content);
            formatted.push_str("\n\n");
            formatted.push_str(&format!("*Source: {}*\n\n", 
                rule.metadata.source_file.display()
            ));
        }
        
        formatted
    }
    
    /// Reload rules if any source files have changed
    pub async fn reload_if_changed(&mut self, root_path: &Path) -> Result<bool, RuleError> {
        let mut needs_reload = false;
        
        // Check if any source files have changed
        for rule_set in &self.rule_sets {
            if let Ok(metadata) = fs::metadata(&rule_set.source_path) {
                let content = fs::read_to_string(&rule_set.source_path)?;
                let current_hash = format!("{:x}", md5::compute(content.as_bytes()));
                
                if current_hash != rule_set.metadata.file_hash {
                    needs_reload = true;
                    break;
                }
            }
        }
        
        if needs_reload {
            self.load_rules_hierarchy(root_path).await?;
        }
        
        Ok(needs_reload)
    }
    
    /// Get summary of loaded rules
    pub fn get_rules_summary(&self) -> RulesSummary {
        let mut summary = RulesSummary {
            total_rule_sets: self.rule_sets.len(),
            total_rules: 0,
            by_priority: HashMap::new(),
            by_source: Vec::new(),
        };
        
        for rule_set in &self.rule_sets {
            summary.total_rules += rule_set.rules.len();
            *summary.by_priority.entry(rule_set.priority).or_insert(0) += rule_set.rules.len();
            summary.by_source.push(RuleSetSummary {
                path: rule_set.source_path.clone(),
                priority: rule_set.priority,
                rule_count: rule_set.rules.len(),
                enabled: rule_set.metadata.is_enabled,
                last_loaded: rule_set.metadata.loaded_at,
            });
        }
        
        summary
    }
}

/// Summary of loaded rules for diagnostics
#[derive(Debug, Clone)]
pub struct RulesSummary {
    pub total_rule_sets: usize,
    pub total_rules: usize,
    pub by_priority: HashMap<RulePriority, usize>,
    pub by_source: Vec<RuleSetSummary>,
}

/// Summary of a specific rule set
#[derive(Debug, Clone)]
pub struct RuleSetSummary {
    pub path: PathBuf,
    pub priority: RulePriority,
    pub rule_count: usize,
    pub enabled: bool,
    pub last_loaded: chrono::DateTime<chrono::Utc>,
}

/// Simple glob matching for patterns
fn glob_match(pattern: &str, text: &str) -> bool {
    // Simple implementation - in production you'd use a proper glob library
    if pattern == "*" {
        return true;
    }
    
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            // Simple prefix/suffix matching
            return text.starts_with(parts[0]) && text.ends_with(parts[1]);
        }
    }
    
    pattern == text
}

impl Default for RulesManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[tokio::test]
    async fn test_rule_precedence() {
        let temp_dir = tempdir().unwrap();
        let root_path = temp_dir.path();
        
        // Create a project-level WARP.md
        let project_warp = root_path.join("WARP.md");
        fs::write(&project_warp, "# Project Rules\n\nThis is a project-level rule.").unwrap();
        
        // Create a subdirectory with its own WARP.md
        let sub_dir = root_path.join("src");
        fs::create_dir_all(&sub_dir).unwrap();
        let sub_warp = sub_dir.join("WARP.md");
        fs::write(&sub_warp, "# Directory Rules\n\nThis is a directory-level rule.").unwrap();
        
        let mut manager = RulesManager::new();
        manager.load_rules_hierarchy(&sub_dir).await.unwrap();
        
        let context = RuleContext {
            current_directory: sub_dir.clone(),
            file_path: Some(sub_dir.join("test.rs")),
            language: Some("rust".to_string()),
            agent_type: None,
            task_type: None,
            environment_vars: HashMap::new(),
        };
        
        let rules = manager.get_effective_rules(&context);
        
        // Directory rules should have higher priority than project rules
        assert!(!rules.is_empty());
        assert_eq!(rules[0].priority, RulePriority::Directory);
    }
    
    #[test]
    fn test_glob_matching() {
        assert!(glob_match("*.rs", "main.rs"));
        assert!(glob_match("test_*", "test_example"));
        assert!(!glob_match("*.py", "main.rs"));
        assert!(glob_match("*", "anything"));
    }
    
    #[tokio::test]
    async fn test_warp_content_parsing() {
        let manager = RulesManager::new();
        let temp_file = PathBuf::from("test.md");
        
        let content = r#"# Project Overview
This is a test project.

## Rule: Code Style
Always use 4 spaces for indentation.

## Rule: Testing
Write comprehensive tests for all functions.
"#;
        
        let rules = manager.parse_warp_content(content, &temp_file).unwrap();
        
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].name, "Code Style");
        assert_eq!(rules[1].name, "Testing");
        assert!(rules[0].content.contains("4 spaces"));
        assert!(rules[1].content.contains("comprehensive tests"));
    }
}