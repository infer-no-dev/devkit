use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::codegen::{CodeGenerator, GenerationConfig, GenerationRequest, GenerationResult};
use crate::context::CodebaseContext;

/// Interactive session state
#[derive(Debug, Serialize, Deserialize)]
pub struct InteractiveSession {
    /// Session ID for tracking
    pub session_id: String,
    /// Session creation timestamp
    pub created_at: std::time::SystemTime,
    /// Current project context
    pub project_path: Option<PathBuf>,
    /// Conversation history
    pub history: Vec<ConversationEntry>,
    /// Current code artifacts
    pub artifacts: HashMap<String, CodeArtifact>,
    /// Session configuration
    pub config: SessionConfig,
}

/// Single conversation entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationEntry {
    /// Entry timestamp
    pub timestamp: std::time::SystemTime,
    /// User input or system message
    pub role: ConversationRole,
    /// The content of the message
    pub content: String,
    /// Associated code generation result (if any)
    pub result: Option<GenerationResult>,
    /// Entry type for better organization
    pub entry_type: EntryType,
}

/// Role in conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversationRole {
    User,
    Assistant,
    System,
}

/// Type of conversation entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryType {
    /// Initial code generation request
    Generate,
    /// Code refinement request
    Refine,
    /// Explanation request
    Explain,
    /// Optimization request
    Optimize,
    /// Test generation request
    AddTests,
    /// Debug assistance request
    Debug,
    /// General conversation
    Chat,
    /// System status message
    Status,
}

/// Code artifact stored in session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeArtifact {
    /// Unique identifier
    pub id: String,
    /// Artifact name/title
    pub name: String,
    /// Programming language
    pub language: String,
    /// Current code content
    pub code: String,
    /// File path (if saved)
    pub file_path: Option<PathBuf>,
    /// Creation timestamp
    pub created_at: std::time::SystemTime,
    /// Last modified timestamp
    pub modified_at: std::time::SystemTime,
    /// Confidence score from AI
    pub confidence: f64,
    /// Version history
    pub versions: Vec<CodeVersion>,
}

/// Code version for artifact history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeVersion {
    /// Version number
    pub version: u32,
    /// Code content at this version
    pub code: String,
    /// Change description
    pub description: String,
    /// Timestamp of this version
    pub timestamp: std::time::SystemTime,
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Auto-save artifacts to files
    pub auto_save: bool,
    /// Default language for generation
    pub default_language: Option<String>,
    /// Show confidence scores
    pub show_confidence: bool,
    /// Enable verbose output
    pub verbose: bool,
    /// Maximum history entries to keep
    pub max_history: usize,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            auto_save: false,
            default_language: None,
            show_confidence: true,
            verbose: false,
            max_history: 100,
        }
    }
}

impl InteractiveSession {
    /// Create new interactive session
    pub fn new(project_path: Option<PathBuf>) -> Self {
        let session_id = uuid::Uuid::new_v4().to_string();
        
        Self {
            session_id,
            created_at: std::time::SystemTime::now(),
            project_path,
            history: Vec::new(),
            artifacts: HashMap::new(),
            config: SessionConfig::default(),
        }
    }
    
    /// Load session from file
    pub fn load_from_file(path: PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let session: InteractiveSession = serde_json::from_str(&content)?;
        Ok(session)
    }
    
    /// Save session to file
    pub fn save_to_file(&self, path: PathBuf) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// Add entry to conversation history
    pub fn add_history_entry(&mut self, entry: ConversationEntry) {
        self.history.push(entry);
        
        // Trim history if it exceeds max size
        if self.history.len() > self.config.max_history {
            self.history.drain(0..self.history.len() - self.config.max_history);
        }
    }
    
    /// Add or update code artifact
    pub fn add_artifact(&mut self, mut artifact: CodeArtifact) {
        // Update modified timestamp
        artifact.modified_at = std::time::SystemTime::now();
        
        // If updating existing artifact, preserve version history
        if let Some(existing) = self.artifacts.get(&artifact.id) {
            artifact.versions = existing.versions.clone();
            // Add current version to history
            let version = CodeVersion {
                version: existing.versions.len() as u32 + 1,
                code: existing.code.clone(),
                description: "Previous version".to_string(),
                timestamp: existing.modified_at,
            };
            artifact.versions.push(version);
        }
        
        self.artifacts.insert(artifact.id.clone(), artifact);
    }
    
    /// Get artifact by name or ID
    pub fn get_artifact(&self, identifier: &str) -> Option<&CodeArtifact> {
        // Try by ID first
        if let Some(artifact) = self.artifacts.get(identifier) {
            return Some(artifact);
        }
        
        // Then try by name
        self.artifacts.values().find(|a| a.name == identifier)
    }
    
    /// List all artifacts
    pub fn list_artifacts(&self) -> Vec<&CodeArtifact> {
        self.artifacts.values().collect()
    }
    
    /// Get recent conversation context for AI
    pub fn get_recent_context(&self, max_entries: usize) -> Vec<&ConversationEntry> {
        let start_idx = if self.history.len() > max_entries {
            self.history.len() - max_entries
        } else {
            0
        };
        
        self.history[start_idx..].iter().collect()
    }
}

/// Interactive mode manager
pub struct InteractiveManager {
    session: InteractiveSession,
    code_generator: CodeGenerator,
    agent_system: Option<std::sync::Arc<crate::agents::AgentSystem>>,
    context: Option<CodebaseContext>,
}

impl InteractiveManager {
    /// Create new interactive manager
    pub fn new(
        session: InteractiveSession,
        code_generator: CodeGenerator,
        agent_system: Option<std::sync::Arc<crate::agents::AgentSystem>>,
        context: Option<CodebaseContext>,
    ) -> Self {
        Self {
            session,
            code_generator,
            agent_system,
            context,
        }
    }
    
    /// Start interactive mode
    pub async fn start(&mut self) -> Result<()> {
        println!("🚀 Interactive Code Generation Mode Started!");
        println!("Session ID: {}", self.session.session_id);
        
        if let Some(project_path) = &self.session.project_path {
            println!("Project: {:?}", project_path);
        }
        
        self.show_help();
        
        // Main interaction loop
        loop {
            print!("\n> ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();
            
            if input.is_empty() {
                continue;
            }
            
            match self.process_input(input).await {
                Ok(should_continue) => {
                    if !should_continue {
                        break;
                    }
                },
                Err(e) => {
                    println!("❌ Error: {}", e);
                }
            }
        }
        
        println!("👋 Interactive session ended.");
        Ok(())
    }
    
    /// Process user input
    async fn process_input(&mut self, input: &str) -> Result<bool> {
        // Handle commands
        if input.starts_with('/') {
            return self.handle_command(input).await;
        }
        
        // Parse refinement requests
        let (action, content) = self.parse_user_input(input);
        
        // Add user input to history
        let entry = ConversationEntry {
            timestamp: std::time::SystemTime::now(),
            role: ConversationRole::User,
            content: input.to_string(),
            result: None,
            entry_type: action.clone(),
        };
        self.session.add_history_entry(entry);
        
        // Process the request
        match action {
            EntryType::Generate => {
                self.handle_generate_request(content).await?;
            },
            EntryType::Refine => {
                self.handle_refine_request(content).await?;
            },
            EntryType::Explain => {
                self.handle_explain_request(content).await?;
            },
            EntryType::Optimize => {
                self.handle_optimize_request(content).await?;
            },
            EntryType::AddTests => {
                self.handle_add_tests_request(content).await?;
            },
            EntryType::Debug => {
                self.handle_debug_request(content).await?;
            },
            EntryType::Chat => {
                self.handle_chat_request(content).await?;
            },
            _ => {
                println!("🤔 I'm not sure how to handle that. Try /help for available commands.");
            }
        }
        
        Ok(true)
    }
    
    /// Parse user input to determine action and content
    fn parse_user_input(&self, input: &str) -> (EntryType, String) {
        let lower = input.to_lowercase();
        
        if lower.starts_with("refine") || lower.starts_with("improve") || lower.starts_with("make it") {
            (EntryType::Refine, input.to_string())
        } else if lower.starts_with("explain") || lower.starts_with("how does") || lower.starts_with("what does") {
            (EntryType::Explain, input.to_string())
        } else if lower.starts_with("optimize") || lower.starts_with("make faster") || lower.contains("performance") {
            (EntryType::Optimize, input.to_string())
        } else if lower.contains("test") && (lower.contains("add") || lower.contains("write") || lower.contains("create")) {
            (EntryType::AddTests, input.to_string())
        } else if lower.contains("debug") || lower.contains("fix") || lower.contains("error") {
            (EntryType::Debug, input.to_string())
        } else if lower.starts_with("create") || lower.starts_with("generate") || lower.starts_with("write") {
            (EntryType::Generate, input.to_string())
        } else {
            (EntryType::Chat, input.to_string())
        }
    }
    
    /// Handle code generation request
    async fn handle_generate_request(&mut self, content: String) -> Result<()> {
        println!("🔧 Generating code...");
        
        // Try to use agent system first, fall back to direct code generator
        if let Some(_agent_system) = &self.agent_system {
            match self.handle_generate_with_agent(content.clone()).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    println!("⚠️  Agent generation failed, trying direct generator: {}", e);
                }
            }
        }
        
        // Fallback to direct code generator
        let mut config = GenerationConfig::default();
        config.target_language = self.session.config.default_language.clone();
        
        let request = GenerationRequest {
            prompt: content.clone(),
            file_path: None,
            context: self.context.clone().unwrap_or_default(),
            config,
            constraints: Vec::new(),
        };
        
        match self.code_generator.generate_from_prompt(request).await {
            Ok(result) => {
                println!("✅ Code generated successfully!");
                
                if self.session.config.show_confidence {
                    println!("Confidence: {:.2}", result.confidence_score);
                }
                
                println!("\n--- Generated Code ---");
                println!("{}", result.generated_code);
                
                if !result.suggestions.is_empty() {
                    println!("\n💡 Suggestions:");
                    for suggestion in &result.suggestions {
                        println!("  • {}", suggestion);
                    }
                }
                
                // Create artifact
                let artifact_id = format!("artifact_{}", self.session.artifacts.len() + 1);
                let artifact = CodeArtifact {
                    id: artifact_id.clone(),
                    name: self.extract_name_from_prompt(&content, &result.language),
                    language: result.language.clone(),
                    code: result.generated_code.clone(),
                    file_path: None,
                    created_at: std::time::SystemTime::now(),
                    modified_at: std::time::SystemTime::now(),
                    confidence: result.confidence_score,
                    versions: Vec::new(),
                };
                
                self.session.add_artifact(artifact);
                
                println!("\n📦 Artifact created: {}", artifact_id);
                
                // Add to history
                let entry = ConversationEntry {
                    timestamp: std::time::SystemTime::now(),
                    role: ConversationRole::Assistant,
                    content: "Code generated successfully".to_string(),
                    result: Some(result.clone()),
                    entry_type: EntryType::Generate,
                };
                self.session.add_history_entry(entry);
            },
            Err(e) => {
                println!("❌ Failed to generate code: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Handle code refinement request
    async fn handle_refine_request(&mut self, content: String) -> Result<()> {
        // Find the most recent artifact to refine
        let artifact_id = if let Some(artifact) = self.session.artifacts.values().last() {
            artifact.id.clone()
        } else {
            println!("❌ No code artifact found to refine. Generate some code first!");
            return Ok(());
        };
        
        println!("🔧 Refining code (artifact: {})...", artifact_id);
        
        let artifact = self.session.artifacts.get(&artifact_id).unwrap();
        
        // Create refinement prompt
        let refinement_prompt = format!(
            "Refine the following {} code based on this request: {}\n\n```{}\n{}\n```",
            artifact.language,
            content,
            artifact.language,
            artifact.code
        );
        
        let mut config = GenerationConfig::default();
        config.target_language = Some(artifact.language.clone());
        
        let request = GenerationRequest {
            prompt: refinement_prompt,
            file_path: None,
            context: self.context.clone().unwrap_or_default(),
            config,
            constraints: Vec::new(),
        };
        
        match self.code_generator.generate_from_prompt(request).await {
            Ok(result) => {
                println!("✅ Code refined successfully!");
                
                if self.session.config.show_confidence {
                    println!("Confidence: {:.2}", result.confidence_score);
                }
                
                println!("\n--- Refined Code ---");
                println!("{}", result.generated_code);
                
                // Update artifact
                let mut updated_artifact = artifact.clone();
                updated_artifact.code = result.generated_code.clone();
                updated_artifact.confidence = result.confidence_score;
                
                self.session.add_artifact(updated_artifact);
                
                println!("\n📦 Artifact updated: {}", artifact_id);
                
                // Add to history
                let entry = ConversationEntry {
                    timestamp: std::time::SystemTime::now(),
                    role: ConversationRole::Assistant,
                    content: "Code refined successfully".to_string(),
                    result: Some(result),
                    entry_type: EntryType::Refine,
                };
                self.session.add_history_entry(entry);
            },
            Err(e) => {
                println!("❌ Failed to refine code: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Handle explanation request
    async fn handle_explain_request(&mut self, content: String) -> Result<()> {
        // Find the most recent artifact to explain
        let artifact = if let Some(artifact) = self.session.artifacts.values().last() {
            artifact
        } else {
            println!("❌ No code artifact found to explain. Generate some code first!");
            return Ok(());
        };
        
        println!("🔍 Explaining code...");
        
        // Create explanation prompt
        let explanation_prompt = format!(
            "Explain the following {} code in detail. Focus on: {}\n\n```{}\n{}\n```",
            artifact.language,
            content,
            artifact.language,
            artifact.code
        );
        
        let mut config = GenerationConfig::default();
        config.target_language = Some("markdown".to_string()); // Request explanation in markdown
        
        let request = GenerationRequest {
            prompt: explanation_prompt,
            file_path: None,
            context: self.context.clone().unwrap_or_default(),
            config,
            constraints: Vec::new(),
        };
        
        match self.code_generator.generate_from_prompt(request).await {
            Ok(result) => {
                println!("✅ Explanation generated:");
                println!("\n{}", result.generated_code);
                
                // Add to history
                let entry = ConversationEntry {
                    timestamp: std::time::SystemTime::now(),
                    role: ConversationRole::Assistant,
                    content: result.generated_code.clone(),
                    result: Some(result),
                    entry_type: EntryType::Explain,
                };
                self.session.add_history_entry(entry);
            },
            Err(e) => {
                println!("❌ Failed to generate explanation: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Handle optimization request
    async fn handle_optimize_request(&mut self, content: String) -> Result<()> {
        // Try to use agent system first
        if let Some(_agent_system) = &self.agent_system {
            match self.handle_optimize_with_agent(content.clone()).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    println!("⚠️  Agent optimization failed: {}", e);
                }
            }
        }
        
        println!("⚡ Optimization feature will be enhanced with specialized agents!");
        Ok(())
    }
    
    /// Handle test generation request
    async fn handle_add_tests_request(&mut self, _content: String) -> Result<()> {
        println!("🧪 This feature will be implemented with the testing & validation enhancement!");
        Ok(())
    }
    
    /// Handle debug request
    async fn handle_debug_request(&mut self, content: String) -> Result<()> {
        // Try to use agent system first
        if let Some(_agent_system) = &self.agent_system {
            match self.handle_debug_with_agent(content.clone()).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    println!("⚠️  Agent debugging failed: {}", e);
                }
            }
        }
        
        println!("🐛 Debug assistance will be enhanced with specialized agents!");
        Ok(())
    }
    
    /// Handle general chat request
    async fn handle_chat_request(&mut self, _content: String) -> Result<()> {
        println!("💬 Chat functionality will be enhanced with better context understanding!");
        Ok(())
    }
    
    /// Handle command
    async fn handle_command(&mut self, input: &str) -> Result<bool> {
        let parts: Vec<&str> = input[1..].split_whitespace().collect();
        if parts.is_empty() {
            return Ok(true);
        }
        
        match parts[0] {
            "help" => {
                self.show_help();
            },
            "list" | "ls" => {
                self.show_artifacts();
            },
            "show" => {
                if parts.len() > 1 {
                    self.show_artifact(parts[1]);
                } else {
                    println!("Usage: /show <artifact_id>");
                }
            },
            "save" => {
                if parts.len() > 2 {
                    self.save_artifact(parts[1], parts[2]).await?;
                } else {
                    println!("Usage: /save <artifact_id> <file_path>");
                }
            },
            "session" => {
                self.show_session_info();
            },
            "config" => {
                if parts.len() > 2 {
                    self.update_config(parts[1], parts[2]);
                } else {
                    self.show_config();
                }
            },
            "clear" => {
                print!("\x1B[2J\x1B[1;1H"); // Clear screen
                io::stdout().flush()?;
            },
            "exit" | "quit" => {
                return Ok(false);
            },
            _ => {
                println!("Unknown command: {}. Type /help for available commands.", parts[0]);
            }
        }
        
        Ok(true)
    }
    
    /// Show help information
    fn show_help(&self) {
        println!("\n📖 Interactive Mode Help");
        println!("─────────────────────────");
        println!("Natural language commands:");
        println!("  'Create a function that...'  - Generate new code");
        println!("  'Refine this code to...'     - Improve existing code");
        println!("  'Explain how this works'     - Get code explanation");
        println!("  'Optimize for performance'   - Optimize code (coming soon)");
        println!("  'Add tests for this'         - Generate tests (coming soon)");
        println!("  'Fix this bug...'            - Debug assistance (coming soon)");
        println!();
        println!("Commands:");
        println!("  /help                        - Show this help");
        println!("  /list, /ls                   - List code artifacts");
        println!("  /show <id>                   - Show specific artifact");
        println!("  /save <id> <path>            - Save artifact to file");
        println!("  /session                     - Show session info");
        println!("  /config [key] [value]        - View/update configuration");
        println!("  /clear                       - Clear screen");
        println!("  /exit, /quit                 - Exit interactive mode");
        println!();
    }
    
    /// Show artifacts
    fn show_artifacts(&self) {
        if self.session.artifacts.is_empty() {
            println!("📦 No code artifacts created yet.");
            return;
        }
        
        println!("📦 Code Artifacts:");
        println!("─────────────────");
        for artifact in self.session.artifacts.values() {
            println!("  {} ({}) - {} [confidence: {:.2}]", 
                artifact.id, artifact.language, artifact.name, artifact.confidence);
        }
    }
    
    /// Show specific artifact
    fn show_artifact(&self, id: &str) {
        if let Some(artifact) = self.session.get_artifact(id) {
            println!("📄 Artifact: {}", artifact.id);
            println!("Name: {}", artifact.name);
            println!("Language: {}", artifact.language);
            println!("Confidence: {:.2}", artifact.confidence);
            println!("Created: {:?}", artifact.created_at);
            println!("Modified: {:?}", artifact.modified_at);
            if let Some(path) = &artifact.file_path {
                println!("File: {:?}", path);
            }
            println!("\n--- Code ---");
            println!("{}", artifact.code);
        } else {
            println!("❌ Artifact '{}' not found.", id);
        }
    }
    
    /// Save artifact to file
    async fn save_artifact(&mut self, id: &str, file_path: &str) -> Result<()> {
        if let Some(artifact) = self.session.artifacts.get_mut(id) {
            let path = PathBuf::from(file_path);
            std::fs::write(&path, &artifact.code)?;
            artifact.file_path = Some(path.clone());
            println!("✅ Artifact saved to: {:?}", path);
        } else {
            println!("❌ Artifact '{}' not found.", id);
        }
        Ok(())
    }
    
    /// Show session information
    fn show_session_info(&self) {
        println!("📊 Session Information");
        println!("─────────────────────");
        println!("Session ID: {}", self.session.session_id);
        println!("Created: {:?}", self.session.created_at);
        println!("Project: {:?}", self.session.project_path.as_ref().unwrap_or(&PathBuf::from("None")));
        println!("Artifacts: {}", self.session.artifacts.len());
        println!("History entries: {}", self.session.history.len());
    }
    
    /// Show configuration
    fn show_config(&self) {
        println!("⚙️  Configuration");
        println!("─────────────────");
        println!("Auto-save: {}", self.session.config.auto_save);
        println!("Default language: {:?}", self.session.config.default_language);
        println!("Show confidence: {}", self.session.config.show_confidence);
        println!("Verbose: {}", self.session.config.verbose);
        println!("Max history: {}", self.session.config.max_history);
    }
    
    /// Update configuration
    fn update_config(&mut self, key: &str, value: &str) {
        match key {
            "auto_save" => {
                self.session.config.auto_save = value.parse().unwrap_or(false);
                println!("✅ Auto-save set to: {}", self.session.config.auto_save);
            },
            "default_language" => {
                self.session.config.default_language = if value == "none" { 
                    None 
                } else { 
                    Some(value.to_string()) 
                };
                println!("✅ Default language set to: {:?}", self.session.config.default_language);
            },
            "show_confidence" => {
                self.session.config.show_confidence = value.parse().unwrap_or(true);
                println!("✅ Show confidence set to: {}", self.session.config.show_confidence);
            },
            "verbose" => {
                self.session.config.verbose = value.parse().unwrap_or(false);
                println!("✅ Verbose set to: {}", self.session.config.verbose);
            },
            "max_history" => {
                self.session.config.max_history = value.parse().unwrap_or(100);
                println!("✅ Max history set to: {}", self.session.config.max_history);
            },
            _ => {
                println!("❌ Unknown config key: {}", key);
                println!("Available keys: auto_save, default_language, show_confidence, verbose, max_history");
            }
        }
    }
    
    /// Extract name from prompt for artifact naming
    fn extract_name_from_prompt(&self, prompt: &str, language: &str) -> String {
        // Simple heuristic to extract a meaningful name
        let words: Vec<&str> = prompt.split_whitespace().collect();
        
        // Look for function/class/struct keywords
        for (i, word) in words.iter().enumerate() {
            if matches!(word.to_lowercase().as_str(), "function" | "class" | "struct" | "interface" | "component") {
                if i + 1 < words.len() {
                    return words[i + 1].trim_matches(|c: char| !c.is_alphanumeric() && c != '_').to_string();
                }
            }
        }
        
        // Fallback to first few meaningful words
        let meaningful_words: Vec<&str> = words.iter()
            .filter(|w| w.len() > 2 && !matches!(w.to_lowercase().as_str(), "that" | "the" | "and" | "for" | "with"))
            .take(3)
            .cloned()
            .collect();
        
        if !meaningful_words.is_empty() {
            meaningful_words.join("_").to_lowercase()
        } else {
            format!("{}_code", language.to_lowercase())
        }
    }
    
    /// Get mutable session reference
    pub fn session_mut(&mut self) -> &mut InteractiveSession {
        &mut self.session
    }
    
    /// Get session reference
    pub fn session(&self) -> &InteractiveSession {
        &self.session
    }
    
    /// Handle code generation using agent system
    async fn handle_generate_with_agent(&mut self, content: String) -> Result<()> {
        let agent_system = self.agent_system.as_ref().unwrap();
        
        // Create agent task for code generation
        let task = crate::agents::AgentTask {
            id: uuid::Uuid::new_v4().to_string(),
            task_type: "generate_code".to_string(),
            description: content.clone(),
            context: serde_json::json!({
                "language": self.session.config.default_language,
                "existing_code": "",
                "file_path": "",
                "requirements": []
            }),
            priority: crate::agents::TaskPriority::High,
        };
        
        println!("🤖 Using AI agent for code generation...");
        
        // Submit task to agent system
        match agent_system.submit_task(task).await {
            Ok(result) => {
                if result.success {
                    println!("✅ Agent generated code successfully!");
                    println!("\n--- Generated Code ---");
                    
                    // Extract code from the first artifact if available
                    if let Some(artifact) = result.artifacts.first() {
                        println!("{}", artifact.content);
                        
                        // Create interactive session artifact
                        let artifact_id = format!("artifact_{}", self.session.artifacts.len() + 1);
                        let interactive_artifact = CodeArtifact {
                            id: artifact_id.clone(),
                            name: self.extract_name_from_prompt(&content, "rust"), // Default to rust for now
                            language: artifact.metadata.get("language")
                                .and_then(|v| v.as_str())
                                .unwrap_or("rust")
                                .to_string(),
                            code: artifact.content.clone(),
                            file_path: None,
                            created_at: std::time::SystemTime::now(),
                            modified_at: std::time::SystemTime::now(),
                            confidence: artifact.metadata.get("confidence")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.8),
                            versions: Vec::new(),
                        };
                        
                        self.session.add_artifact(interactive_artifact);
                        println!("\n📦 Artifact created: {}", artifact_id);
                        
                        // Add to history
                        let entry = ConversationEntry {
                            timestamp: std::time::SystemTime::now(),
                            role: ConversationRole::Assistant,
                            content: "Agent-generated code successfully".to_string(),
                            result: None, // We don't have GenerationResult from agent
                            entry_type: EntryType::Generate,
                        };
                        self.session.add_history_entry(entry);
                        
                        Ok(())
                    } else {
                        println!("⚠️  Agent completed task but no artifacts were generated");
                        Err(anyhow::anyhow!("No artifacts generated by agent"))
                    }
                } else {
                    println!("❌ Agent task failed: {}", result.output);
                    Err(anyhow::anyhow!("Agent task failed: {}", result.output))
                }
            },
            Err(e) => {
                println!("❌ Agent system error: {}", e);
                Err(anyhow::anyhow!("Agent system error: {}", e))
            }
        }
    }
    
    /// Handle code optimization using analysis agent
    async fn handle_optimize_with_agent(&mut self, content: String) -> Result<()> {
        let agent_system = self.agent_system.as_ref().unwrap();
        
        // Find the most recent artifact to optimize
        let artifact = if let Some(artifact) = self.session.artifacts.values().last() {
            artifact
        } else {
            return Err(anyhow::anyhow!("No code artifact found to optimize"));
        };
        
        // Create agent task for performance analysis
        let task = crate::agents::AgentTask {
            id: uuid::Uuid::new_v4().to_string(),
            task_type: "performance_analysis".to_string(),
            description: format!("Analyze and suggest optimizations for: {}", content),
            context: serde_json::json!({
                "language": artifact.language,
                "existing_code": artifact.code,
                "optimization_focus": content
            }),
            priority: crate::agents::TaskPriority::Normal,
        };
        
        println!("🔍 Using analysis agent for code optimization...");
        
        // Submit task to agent system
        match agent_system.submit_task(task).await {
            Ok(result) => {
                if result.success {
                    println!("✅ Agent analysis completed!");
                    println!("\n--- Optimization Suggestions ---");
                    println!("{}", result.output);
                    
                    // Add to history
                    let entry = ConversationEntry {
                        timestamp: std::time::SystemTime::now(),
                        role: ConversationRole::Assistant,
                        content: result.output,
                        result: None,
                        entry_type: EntryType::Optimize,
                    };
                    self.session.add_history_entry(entry);
                    
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Agent analysis failed: {}", result.output))
                }
            },
            Err(e) => {
                Err(anyhow::anyhow!("Agent system error: {}", e))
            }
        }
    }
    
    /// Handle debugging using debugging agent
    async fn handle_debug_with_agent(&mut self, content: String) -> Result<()> {
        let agent_system = self.agent_system.as_ref().unwrap();
        
        // Find the most recent artifact to debug
        let artifact = if let Some(artifact) = self.session.artifacts.values().last() {
            artifact
        } else {
            return Err(anyhow::anyhow!("No code artifact found to debug"));
        };
        
        // Create agent task for debugging
        let task = crate::agents::AgentTask {
            id: uuid::Uuid::new_v4().to_string(),
            task_type: "debug_issue".to_string(),
            description: format!("Debug issue: {}", content),
            context: serde_json::json!({
                "language": artifact.language,
                "existing_code": artifact.code,
                "issue_description": content
            }),
            priority: crate::agents::TaskPriority::High,
        };
        
        println!("🔧 Using debugging agent for issue analysis...");
        
        // Submit task to agent system
        match agent_system.submit_task(task).await {
            Ok(result) => {
                if result.success {
                    println!("✅ Agent debugging completed!");
                    println!("\n--- Debug Analysis ---");
                    println!("{}", result.output);
                    
                    // Add to history
                    let entry = ConversationEntry {
                        timestamp: std::time::SystemTime::now(),
                        role: ConversationRole::Assistant,
                        content: result.output,
                        result: None,
                        entry_type: EntryType::Debug,
                    };
                    self.session.add_history_entry(entry);
                    
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Agent debugging failed: {}", result.output))
                }
            },
            Err(e) => {
                Err(anyhow::anyhow!("Agent system error: {}", e))
            }
        }
    }
}

