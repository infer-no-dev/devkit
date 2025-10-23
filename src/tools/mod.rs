//! Comprehensive Tool Ecosystem with MCP Protocol Support
//!
//! This module provides Model Context Protocol (MCP) support, tool registry,
//! auth brokering, and integration with external development tools and services.

pub mod mcp;
pub mod registry;
pub mod auth;
pub mod providers;
pub mod execution;

use crate::agents::AgentError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

pub use mcp::{MCPClient, MCPServer, MCPMessage, MCPCapability};
pub use registry::{ToolRegistry, ToolDefinition, ToolMetadata};
pub use auth::{AuthBroker, Credential, AuthMethod};
pub use providers::{ToolProvider, ProviderConfig, ProviderCapability};
pub use execution::{ToolExecutor, ExecutionStats};
pub use providers::{ExecutionContext, ExecutionResult};

/// Comprehensive tool ecosystem manager
pub struct ToolEcosystem {
    /// Tool registry
    registry: Arc<ToolRegistry>,
    /// Authentication broker
    auth_broker: Arc<AuthBroker>,
    /// MCP clients for external integrations
    mcp_clients: Arc<RwLock<HashMap<String, Arc<MCPClient>>>>,
    /// MCP server for exposing our capabilities
    mcp_server: Arc<MCPServer>,
    /// Tool providers
    providers: Arc<RwLock<HashMap<String, Arc<dyn ToolProvider + Send + Sync>>>>,
    /// Tool executor
    executor: Arc<ToolExecutor>,
    /// Configuration
    config: ToolEcosystemConfig,
}

impl std::fmt::Debug for ToolEcosystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolEcosystem")
            .field("registry", &self.registry)
            .field("auth_broker", &self.auth_broker)
            .field("mcp_clients", &self.mcp_clients)
            .field("mcp_server", &self.mcp_server)
            .field("providers", &"[trait objects]")
            .field("executor", &self.executor)
            .field("config", &self.config)
            .finish()
    }
}

/// Configuration for the tool ecosystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEcosystemConfig {
    /// Maximum number of concurrent tool executions
    pub max_concurrent_executions: usize,
    /// Default timeout for tool executions
    pub default_timeout: Duration,
    /// Whether to enable MCP server
    pub enable_mcp_server: bool,
    /// MCP server port
    pub mcp_server_port: u16,
    /// Tool discovery settings
    pub discovery: ToolDiscoveryConfig,
    /// Security settings
    pub security: ToolSecurityConfig,
    /// Provider configurations
    pub providers: HashMap<String, ProviderConfig>,
    /// Auth configurations
    pub auth: HashMap<String, AuthConfig>,
}

/// Tool discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDiscoveryConfig {
    /// Whether to enable automatic tool discovery
    pub auto_discovery: bool,
    /// Paths to scan for tools
    pub scan_paths: Vec<std::path::PathBuf>,
    /// MCP server URLs to connect to
    pub mcp_servers: Vec<String>,
    /// Discovery interval
    pub discovery_interval: Duration,
    /// Tool manifests to load
    pub manifests: Vec<std::path::PathBuf>,
}

/// Tool security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSecurityConfig {
    /// Whether to enable sandboxing for tool execution
    pub enable_sandboxing: bool,
    /// Allowed tool categories
    pub allowed_categories: Vec<ToolCategory>,
    /// Blocked tool patterns
    pub blocked_patterns: Vec<String>,
    /// Whether to require approval for new tools
    pub require_approval: bool,
    /// Maximum execution time
    pub max_execution_time: Duration,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub auth_method: AuthMethod,
    pub endpoint: Option<String>,
    pub scopes: Vec<String>,
    pub expires_in: Option<Duration>,
    pub refresh_threshold: Option<Duration>,
}

/// Categories of tools
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolCategory {
    /// Version control tools (git, hg, etc.)
    VersionControl,
    /// Build and CI/CD tools
    BuildTools,
    /// Testing frameworks
    Testing,
    /// Code analysis and linting
    Analysis,
    /// Documentation tools
    Documentation,
    /// Database tools
    Database,
    /// Cloud and infrastructure
    Infrastructure,
    /// Communication tools (Slack, Discord, etc.)
    Communication,
    /// Project management
    ProjectManagement,
    /// AI and ML tools
    MachineLearning,
    /// Security tools
    Security,
    /// Custom tools
    Custom(String),
}

/// Tool invocation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocation {
    pub tool_name: String,
    pub operation: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub context: ExecutionContext,
    pub timeout: Option<Duration>,
    pub auth_required: bool,
}

/// Result of tool invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: serde_json::Value,
    pub metadata: ToolResultMetadata,
    pub artifacts: Vec<ToolArtifact>,
    pub next_actions: Vec<SuggestedAction>,
}

/// Metadata about tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultMetadata {
    pub execution_time: Duration,
    pub tool_version: String,
    pub provider: String,
    pub cost: Option<f64>,
    pub rate_limit_remaining: Option<u32>,
    pub warnings: Vec<String>,
    pub debug_info: HashMap<String, serde_json::Value>,
}

/// Artifacts produced by tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolArtifact {
    pub artifact_type: ArtifactType,
    pub name: String,
    pub content: serde_json::Value,
    pub mime_type: String,
    pub size_bytes: u64,
    pub checksum: String,
}

/// Types of artifacts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    File,
    Directory,
    Report,
    Image,
    Data,
    Log,
    Configuration,
    Custom(String),
}

/// Suggested follow-up actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedAction {
    pub action_type: ActionType,
    pub description: String,
    pub tool_name: Option<String>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub priority: ActionPriority,
}

/// Types of suggested actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    InvokeTool,
    ReviewResult,
    ApproveChange,
    RunTest,
    CreatePullRequest,
    NotifyTeam,
    Custom(String),
}

/// Priority levels for actions
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ActionPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Tool capability description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCapability {
    pub name: String,
    pub description: String,
    pub operations: Vec<ToolOperation>,
    pub required_auth: Vec<AuthMethod>,
    pub rate_limits: Option<RateLimit>,
    pub cost_model: Option<CostModel>,
    pub dependencies: Vec<String>,
}

/// Tool operation description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOperation {
    pub name: String,
    pub description: String,
    pub parameters: Vec<OperationParameter>,
    pub return_type: String,
    pub examples: Vec<OperationExample>,
    pub side_effects: Vec<SideEffect>,
}

/// Operation parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationParameter {
    pub name: String,
    pub parameter_type: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
    pub constraints: Vec<ParameterConstraint>,
}

/// Parameter constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterConstraint {
    MinLength(usize),
    MaxLength(usize),
    Pattern(String),
    MinValue(f64),
    MaxValue(f64),
    OneOf(Vec<serde_json::Value>),
    Custom(String),
}

/// Operation example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationExample {
    pub name: String,
    pub description: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub expected_result: serde_json::Value,
}

/// Side effects of operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SideEffect {
    FileSystemWrite,
    FileSystemRead,
    NetworkRequest,
    StateModification,
    ExternalServiceCall,
    UserInteraction,
    Custom(String),
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub requests_per_day: u32,
    pub burst_limit: u32,
    pub reset_time: chrono::DateTime<chrono::Utc>,
}

/// Cost model for tool usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostModel {
    pub pricing_type: PricingType,
    pub base_cost: f64,
    pub variable_costs: HashMap<String, f64>,
    pub currency: String,
}

/// Pricing types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PricingType {
    Free,
    PerCall,
    PerMinute,
    PerToken,
    PerResource,
    Subscription,
    Custom(String),
}

/// Tool discovery service
#[derive(Debug)]
pub struct ToolDiscovery {
    config: ToolDiscoveryConfig,
    discovered_tools: Arc<RwLock<HashMap<String, DiscoveredTool>>>,
    discovery_tasks: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
}

/// Information about a discovered tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredTool {
    pub name: String,
    pub version: String,
    pub source: DiscoverySource,
    pub capabilities: Vec<ToolCapability>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub status: ToolStatus,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Source of tool discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoverySource {
    MCPServer(String),
    FileSystem(std::path::PathBuf),
    Registry(String),
    API(String),
    Manual,
}

/// Status of discovered tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolStatus {
    Available,
    Unavailable,
    Deprecated,
    RequiresAuth,
    RateLimited,
    Error(String),
}

/// Tool ecosystem errors
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    
    #[error("Operation not supported: {operation} on {tool}")]
    OperationNotSupported { tool: String, operation: String },
    
    #[error("Authentication failed for tool: {0}")]
    AuthenticationFailed(String),
    
    #[error("Tool execution timeout: {0}")]
    ExecutionTimeout(String),
    
    #[error("Rate limit exceeded for tool: {0}")]
    RateLimitExceeded(String),
    
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    
    #[error("Tool provider error: {0}")]
    ProviderError(String),
    
    #[error("MCP protocol error: {0}")]
    MCPError(String),
    
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl ToolEcosystem {
    /// Create a new tool ecosystem
    pub async fn new(config: ToolEcosystemConfig) -> Result<Self, ToolError> {
        let registry = Arc::new(ToolRegistry::new());
        let auth_broker = Arc::new(AuthBroker::new(config.auth.clone()).await?);
        let executor = Arc::new(ToolExecutor::new(config.max_concurrent_executions));
        
        let mcp_server = if config.enable_mcp_server {
            Arc::new(MCPServer::new(config.mcp_server_port).await?)
        } else {
            Arc::new(MCPServer::disabled())
        };
        
        let ecosystem = Self {
            registry: registry.clone(),
            auth_broker,
            mcp_clients: Arc::new(RwLock::new(HashMap::new())),
            mcp_server,
            providers: Arc::new(RwLock::new(HashMap::new())),
            executor,
            config,
        };
        
        // Initialize discovery if enabled
        if ecosystem.config.discovery.auto_discovery {
            ecosystem.start_discovery().await?;
        }
        
        // Initialize providers
        ecosystem.initialize_providers().await?;
        
        Ok(ecosystem)
    }
    
    /// Invoke a tool with the given parameters
    pub async fn invoke_tool(&self, invocation: ToolInvocation) -> Result<ToolResult, ToolError> {
        // Validate tool exists
        let tool_def = self.registry.get_tool(&invocation.tool_name).await
            .ok_or_else(|| ToolError::ToolNotFound(invocation.tool_name.clone()))?;
        
        // Check security policies
        self.validate_security(&invocation, &tool_def).await?;
        
        // Handle authentication if required
        let credentials = if invocation.auth_required {
            Some(self.auth_broker.get_credentials(&invocation.tool_name).await?)
        } else {
            None
        };
        
        // Execute the tool
        let execution_context = ExecutionContext {
            tool_name: invocation.tool_name.clone(),
            operation: invocation.operation.clone(),
            parameters: invocation.parameters,
            context: invocation.context.context.clone(),
            credentials,
            timeout: invocation.timeout.or(Some(self.config.default_timeout)),
        };
        
        let execution_result = self.executor.execute(execution_context).await?;
        
        // Generate next actions before moving execution_result
        let next_actions = self.generate_next_actions(&execution_result).await;
        
        // Convert to tool result
        let tool_result = ToolResult {
            success: execution_result.success,
            output: execution_result.output,
            metadata: ToolResultMetadata {
                execution_time: execution_result.duration,
                tool_version: tool_def.version.clone(),
                provider: tool_def.provider.clone(),
                cost: execution_result.cost,
                rate_limit_remaining: execution_result.rate_limit_remaining,
                warnings: execution_result.warnings,
                debug_info: execution_result.debug_info,
            },
            artifacts: execution_result.artifacts.into_iter()
                .map(|a| ToolArtifact {
                    artifact_type: a.artifact_type,
                    name: a.name,
                    content: a.content,
                    mime_type: a.mime_type,
                    size_bytes: a.size_bytes,
                    checksum: a.checksum,
                })
                .collect(),
            next_actions,
        };
        
        Ok(tool_result)
    }
    
    /// Register a new tool provider
    pub async fn register_provider(
        &self, 
        name: String, 
        provider: Arc<dyn ToolProvider + Send + Sync>
    ) -> Result<(), ToolError> {
        let mut providers = self.providers.write().await;
        providers.insert(name, provider);
        Ok(())
    }
    
    /// Connect to an MCP server
    pub async fn connect_mcp_server(&self, url: String) -> Result<(), ToolError> {
        let client = MCPClient::connect(&url).await?;
        let capabilities = client.get_capabilities().await?;
        
        // Register tools from this MCP server
        for capability in capabilities {
            self.registry.register_from_mcp(&url, capability).await?;
        }
        
        let mut clients = self.mcp_clients.write().await;
        clients.insert(url, Arc::new(client));
        
        Ok(())
    }
    
    /// List available tools
    pub async fn list_tools(&self) -> Vec<ToolDefinition> {
        self.registry.list_tools().await
    }
    
    /// Get tool definition
    pub async fn get_tool_definition(&self, name: &str) -> Option<ToolDefinition> {
        self.registry.get_tool(name).await
    }
    
    /// Search for tools by capability
    pub async fn search_tools(&self, query: &str) -> Vec<ToolDefinition> {
        self.registry.search_tools(query).await
    }
    
    /// Get tool recommendations based on context
    pub async fn recommend_tools(&self, context: &ExecutionContext) -> Vec<ToolDefinition> {
        // Analyze context and recommend appropriate tools
        let mut recommendations = Vec::new();
        
        // This would use ML/heuristics to suggest relevant tools
        // For now, return tools that match the context keywords
        let tools = self.list_tools().await;
        
        for tool in tools {
            if self.matches_context(&tool, context) {
                recommendations.push(tool);
            }
        }
        
        // Sort by relevance score
        recommendations.sort_by(|a, b| b.metadata.relevance_score.partial_cmp(&a.metadata.relevance_score).unwrap());
        
        recommendations
    }
    
    /// Start tool discovery process
    async fn start_discovery(&self) -> Result<(), ToolError> {
        let discovery = ToolDiscovery::new(self.config.discovery.clone());
        discovery.start_continuous_discovery(self.registry.clone()).await?;
        Ok(())
    }
    
    /// Initialize configured providers
    async fn initialize_providers(&self) -> Result<(), ToolError> {
        for (name, provider_config) in &self.config.providers {
            let provider = providers::create_provider(provider_config.clone())?;
            self.register_provider(name.clone(), provider).await?;
        }
        Ok(())
    }
    
    /// Validate security policies for tool invocation
    async fn validate_security(
        &self, 
        invocation: &ToolInvocation, 
        tool_def: &ToolDefinition
    ) -> Result<(), ToolError> {
        let security_config = &self.config.security;
        
        // Check if tool category is allowed
        if !security_config.allowed_categories.contains(&tool_def.category) {
            return Err(ToolError::SecurityViolation(format!(
                "Tool category {:?} is not allowed", tool_def.category
            )));
        }
        
        // Check blocked patterns
        for pattern in &security_config.blocked_patterns {
            if invocation.tool_name.contains(pattern) {
                return Err(ToolError::SecurityViolation(format!(
                    "Tool name matches blocked pattern: {}", pattern
                )));
            }
        }
        
        // Check execution time limits
        let timeout = invocation.timeout.unwrap_or(self.config.default_timeout);
        if timeout > security_config.max_execution_time {
            return Err(ToolError::SecurityViolation(format!(
                "Execution timeout exceeds maximum allowed: {:?} > {:?}",
                timeout, security_config.max_execution_time
            )));
        }
        
        Ok(())
    }
    
    /// Generate suggested next actions based on execution result
    async fn generate_next_actions(&self, result: &ExecutionResult) -> Vec<SuggestedAction> {
        let mut actions = Vec::new();
        
        // Analyze the result and suggest follow-up actions
        if result.success {
            if result.artifacts.len() > 0 {
                actions.push(SuggestedAction {
                    action_type: ActionType::ReviewResult,
                    description: "Review generated artifacts".to_string(),
                    tool_name: None,
                    parameters: HashMap::new(),
                    priority: ActionPriority::Medium,
                });
            }
            
            // Suggest related tools based on the output
            // This would be more sophisticated in a real implementation
            
        } else {
            actions.push(SuggestedAction {
                action_type: ActionType::ReviewResult,
                description: "Review error details and consider alternative approaches".to_string(),
                tool_name: None,
                parameters: HashMap::new(),
                priority: ActionPriority::High,
            });
        }
        
        actions
    }
    
    /// Check if a tool matches the given context
    fn matches_context(&self, tool: &ToolDefinition, context: &ExecutionContext) -> bool {
        // Simple keyword matching - would be more sophisticated in practice
        let context_str = format!("{} {}", context.operation, 
            context.parameters.values()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        ).to_lowercase();
        
        let tool_str = format!("{} {} {}", 
            tool.name, 
            tool.description,
            tool.capabilities.join(" ")
        ).to_lowercase();
        
        // Check for keyword overlaps
        let context_words: std::collections::HashSet<&str> = context_str.split_whitespace().collect();
        let tool_words: std::collections::HashSet<&str> = tool_str.split_whitespace().collect();
        
        let intersection_count = context_words.intersection(&tool_words).count();
        intersection_count > 0
    }
    
    /// Get ecosystem statistics
    pub async fn get_stats(&self) -> ToolEcosystemStats {
        let tools = self.list_tools().await;
        let providers = self.providers.read().await;
        let mcp_clients = self.mcp_clients.read().await;
        
        ToolEcosystemStats {
            total_tools: tools.len(),
            active_providers: providers.len(),
            mcp_connections: mcp_clients.len(),
            tools_by_category: self.group_tools_by_category(&tools),
            execution_stats: self.executor.get_stats().await,
        }
    }
    
    /// Group tools by category for statistics
    fn group_tools_by_category(&self, tools: &[ToolDefinition]) -> HashMap<String, usize> {
        let mut category_counts = HashMap::new();
        
        for tool in tools {
            let category_name = match &tool.category {
                ToolCategory::Custom(name) => name.clone(),
                other => format!("{:?}", other),
            };
            *category_counts.entry(category_name).or_insert(0) += 1;
        }
        
        category_counts
    }
}

/// Statistics about the tool ecosystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEcosystemStats {
    pub total_tools: usize,
    pub active_providers: usize,
    pub mcp_connections: usize,
    pub tools_by_category: HashMap<String, usize>,
    pub execution_stats: execution::ExecutionStats,
}

impl ToolDiscovery {
    fn new(config: ToolDiscoveryConfig) -> Self {
        Self {
            config,
            discovered_tools: Arc::new(RwLock::new(HashMap::new())),
            discovery_tasks: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    async fn start_continuous_discovery(
        &self, 
        registry: Arc<ToolRegistry>
    ) -> Result<(), ToolError> {
        // Start discovery tasks for different sources
        
        // MCP server discovery
        for server_url in &self.config.mcp_servers {
            let url = server_url.clone();
            let registry_clone = registry.clone();
            let discovered_tools = self.discovered_tools.clone();
            
            let task = tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
                
                loop {
                    interval.tick().await;
                    
                    match MCPClient::connect(&url).await {
                        Ok(client) => {
                            match client.get_capabilities().await {
                                Ok(capabilities) => {
                                    for capability in capabilities {
                                        let _ = registry_clone.register_from_mcp(&url, capability).await;
                                    }
                                }
                                Err(e) => tracing::warn!("Failed to get MCP capabilities from {}: {}", url, e),
                            }
                        }
                        Err(e) => tracing::warn!("Failed to connect to MCP server {}: {}", url, e),
                    }
                }
            });
            
            let mut tasks = self.discovery_tasks.write().await;
            tasks.push(task);
        }
        
        // File system discovery
        if !self.config.scan_paths.is_empty() {
            let paths = self.config.scan_paths.clone();
            let registry_clone = registry.clone();
            
            let task = tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(600)); // 10 minutes
                
                loop {
                    interval.tick().await;
                    
                    for path in &paths {
                        if let Err(e) = Self::discover_filesystem_tools(path, &registry_clone).await {
                            tracing::warn!("Failed to discover tools in {:?}: {}", path, e);
                        }
                    }
                }
            });
            
            let mut tasks = self.discovery_tasks.write().await;
            tasks.push(task);
        }
        
        Ok(())
    }
    
    async fn discover_filesystem_tools(
        path: &std::path::Path,
        registry: &Arc<ToolRegistry>
    ) -> Result<(), ToolError> {
        // Scan for tool manifests, executables, etc.
        // This is a simplified implementation
        
        if path.is_dir() {
            let mut entries = tokio::fs::read_dir(path).await?;
            
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                
                if path.is_file() && path.extension() == Some(std::ffi::OsStr::new("json")) {
                    // Try to load as tool manifest
                    if let Ok(content) = tokio::fs::read_to_string(&path).await {
                        if let Ok(manifest) = serde_json::from_str::<ToolManifest>(&content) {
                            let _ = registry.register_from_manifest(manifest).await;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}

/// Tool manifest structure for file system discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub category: ToolCategory,
    pub capabilities: Vec<ToolCapability>,
    pub executable: std::path::PathBuf,
    pub auth_required: bool,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for ToolEcosystemConfig {
    fn default() -> Self {
        Self {
            max_concurrent_executions: 10,
            default_timeout: Duration::from_secs(30),
            enable_mcp_server: false,
            mcp_server_port: 8080,
            discovery: ToolDiscoveryConfig::default(),
            security: ToolSecurityConfig::default(),
            providers: HashMap::new(),
            auth: HashMap::new(),
        }
    }
}

impl Default for ToolDiscoveryConfig {
    fn default() -> Self {
        Self {
            auto_discovery: false,
            scan_paths: vec![],
            mcp_servers: vec![],
            discovery_interval: Duration::from_secs(300),
            manifests: vec![],
        }
    }
}

impl Default for ToolSecurityConfig {
    fn default() -> Self {
        Self {
            enable_sandboxing: true,
            allowed_categories: vec![
                ToolCategory::VersionControl,
                ToolCategory::BuildTools,
                ToolCategory::Testing,
                ToolCategory::Analysis,
                ToolCategory::Documentation,
            ],
            blocked_patterns: vec![
                "rm".to_string(),
                "delete".to_string(),
                "destroy".to_string(),
            ],
            require_approval: false,
            max_execution_time: Duration::from_secs(300),
        }
    }
}