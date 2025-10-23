//! Tool Registry
//!
//! Manages available tools, their definitions, metadata, and capabilities.

use super::{ToolError, ToolCapability, ToolCategory, MCPCapability, ToolManifest};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Tool registry for managing available tools
#[derive(Debug)]
pub struct ToolRegistry {
    /// Registered tools indexed by name
    tools: Arc<RwLock<HashMap<String, ToolDefinition>>>,
    /// Tools indexed by category
    categories: Arc<RwLock<HashMap<ToolCategory, Vec<String>>>>,
    /// Search index for capability-based lookups
    capability_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Registry metadata
    metadata: Arc<RwLock<RegistryMetadata>>,
}

/// Complete tool definition with all metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name
    pub name: String,
    /// Tool version
    pub version: String,
    /// Human-readable description
    pub description: String,
    /// Tool category
    pub category: ToolCategory,
    /// Tool provider/source
    pub provider: String,
    /// List of capabilities this tool provides
    pub capabilities: Vec<String>,
    /// Tool operations/commands
    pub operations: Vec<ToolOperation>,
    /// Authentication requirements
    pub auth_requirements: Vec<AuthRequirement>,
    /// Rate limiting information
    pub rate_limits: Option<RateLimit>,
    /// Cost information
    pub cost_info: Option<CostInfo>,
    /// Tool dependencies
    pub dependencies: Vec<String>,
    /// Installation/usage instructions
    pub usage_info: ToolUsageInfo,
    /// Tool metadata
    pub metadata: ToolMetadata,
}

/// Tool operation definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOperation {
    /// Operation name
    pub name: String,
    /// Operation description
    pub description: String,
    /// Input parameters
    pub parameters: Vec<ParameterSpec>,
    /// Return value specification
    pub returns: ReturnSpec,
    /// Examples of usage
    pub examples: Vec<OperationExample>,
    /// Whether this operation has side effects
    pub has_side_effects: bool,
    /// Estimated execution time
    pub estimated_duration: Option<std::time::Duration>,
}

/// Parameter specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSpec {
    /// Parameter name
    pub name: String,
    /// Parameter type (e.g., "string", "number", "boolean")
    pub param_type: String,
    /// Parameter description
    pub description: String,
    /// Whether the parameter is required
    pub required: bool,
    /// Default value if not required
    pub default_value: Option<serde_json::Value>,
    /// Validation constraints
    pub constraints: Vec<ParameterConstraint>,
    /// Examples of valid values
    pub examples: Vec<serde_json::Value>,
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
    Custom { name: String, description: String },
}

/// Return value specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnSpec {
    /// Return type description
    pub return_type: String,
    /// Description of what is returned
    pub description: String,
    /// Schema for structured returns
    pub schema: Option<serde_json::Value>,
    /// Example return values
    pub examples: Vec<serde_json::Value>,
}

/// Operation example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationExample {
    /// Example name
    pub name: String,
    /// Example description
    pub description: String,
    /// Input parameters
    pub input: HashMap<String, serde_json::Value>,
    /// Expected output
    pub output: serde_json::Value,
    /// Use case scenario
    pub scenario: String,
}

/// Authentication requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequirement {
    /// Authentication method
    pub method: AuthMethod,
    /// Required scopes/permissions
    pub scopes: Vec<String>,
    /// Optional endpoint
    pub endpoint: Option<String>,
    /// Additional configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Authentication methods
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthMethod {
    None,
    ApiKey,
    OAuth2,
    BasicAuth,
    BearerToken,
    Certificate,
    Custom(String),
}

/// Rate limiting information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Requests per minute
    pub per_minute: u32,
    /// Requests per hour
    pub per_hour: u32,
    /// Requests per day
    pub per_day: u32,
    /// Burst allowance
    pub burst: u32,
}

/// Cost information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostInfo {
    /// Pricing model
    pub model: PricingModel,
    /// Base cost per operation
    pub base_cost: f64,
    /// Variable costs (e.g., per token, per minute)
    pub variable_costs: HashMap<String, f64>,
    /// Currency
    pub currency: String,
    /// Billing cycle
    pub billing_cycle: Option<String>,
}

/// Pricing models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PricingModel {
    Free,
    PerCall,
    PerToken,
    PerMinute,
    PerResource,
    Subscription,
    Usage,
    Custom(String),
}

/// Tool usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsageInfo {
    /// Installation instructions
    pub installation: Option<String>,
    /// Configuration requirements
    pub configuration: Vec<ConfigRequirement>,
    /// Usage examples
    pub examples: Vec<UsageExample>,
    /// Troubleshooting tips
    pub troubleshooting: Vec<TroubleshootingTip>,
    /// Documentation links
    pub documentation: Vec<DocumentationLink>,
}

/// Configuration requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRequirement {
    /// Configuration key
    pub key: String,
    /// Description
    pub description: String,
    /// Whether required
    pub required: bool,
    /// Default value
    pub default: Option<String>,
    /// Example values
    pub examples: Vec<String>,
}

/// Usage example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageExample {
    /// Example title
    pub title: String,
    /// Example description
    pub description: String,
    /// Command or code
    pub code: String,
    /// Expected result
    pub result: String,
}

/// Troubleshooting tip
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TroubleshootingTip {
    /// Issue description
    pub issue: String,
    /// Solution
    pub solution: String,
    /// Additional resources
    pub resources: Vec<String>,
}

/// Documentation link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationLink {
    /// Link title
    pub title: String,
    /// URL
    pub url: String,
    /// Description
    pub description: String,
    /// Link type (api, guide, reference, etc.)
    pub link_type: String,
}

/// Tool metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// When the tool was registered
    pub registered_at: chrono::DateTime<chrono::Utc>,
    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Usage statistics
    pub usage_stats: UsageStats,
    /// Quality metrics
    pub quality_metrics: QualityMetrics,
    /// Relevance score (for recommendations)
    pub relevance_score: f64,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Maintainer information
    pub maintainer: Option<MaintainerInfo>,
    /// License information
    pub license: Option<String>,
}

/// Usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    /// Total invocations
    pub total_calls: u64,
    /// Successful calls
    pub successful_calls: u64,
    /// Failed calls
    pub failed_calls: u64,
    /// Average execution time
    pub avg_execution_time: std::time::Duration,
    /// Last used timestamp
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    /// Popular operations
    pub popular_operations: HashMap<String, u64>,
}

/// Quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Average response time
    pub avg_response_time: std::time::Duration,
    /// User satisfaction score (0.0 to 5.0)
    pub satisfaction_score: f64,
    /// Reliability score (0.0 to 1.0)
    pub reliability_score: f64,
    /// Performance score (0.0 to 1.0)
    pub performance_score: f64,
}

/// Maintainer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintainerInfo {
    /// Maintainer name
    pub name: String,
    /// Contact email
    pub email: Option<String>,
    /// Organization
    pub organization: Option<String>,
    /// GitHub/repository URL
    pub repository: Option<String>,
}

/// Registry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryMetadata {
    /// Total number of tools
    pub total_tools: usize,
    /// Tools by category count
    pub category_counts: HashMap<ToolCategory, usize>,
    /// Most popular tools
    pub popular_tools: Vec<String>,
    /// Recently added tools
    pub recent_tools: Vec<String>,
    /// Registry health metrics
    pub health_metrics: RegistryHealth,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Registry health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryHealth {
    /// Average tool quality score
    pub avg_quality_score: f64,
    /// Percentage of tools with documentation
    pub documentation_coverage: f64,
    /// Percentage of tools that are actively maintained
    pub maintenance_score: f64,
    /// Percentage of tools with working examples
    pub example_coverage: f64,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            categories: Arc::new(RwLock::new(HashMap::new())),
            capability_index: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(RegistryMetadata {
                total_tools: 0,
                category_counts: HashMap::new(),
                popular_tools: Vec::new(),
                recent_tools: Vec::new(),
                health_metrics: RegistryHealth {
                    avg_quality_score: 0.0,
                    documentation_coverage: 0.0,
                    maintenance_score: 0.0,
                    example_coverage: 0.0,
                },
                last_updated: chrono::Utc::now(),
            })),
        }
    }
    
    /// Register a new tool
    pub async fn register_tool(&self, tool: ToolDefinition) -> Result<(), ToolError> {
        let tool_name = tool.name.clone();
        
        // Add to main tools collection
        {
            let mut tools = self.tools.write().await;
            tools.insert(tool_name.clone(), tool.clone());
        }
        
        // Update category index
        {
            let mut categories = self.categories.write().await;
            let category_tools = categories.entry(tool.category.clone()).or_insert_with(Vec::new);
            if !category_tools.contains(&tool_name) {
                category_tools.push(tool_name.clone());
            }
        }
        
        // Update capability index
        {
            let mut capability_index = self.capability_index.write().await;
            for capability in &tool.capabilities {
                let capability_tools = capability_index.entry(capability.clone()).or_insert_with(Vec::new);
                if !capability_tools.contains(&tool_name) {
                    capability_tools.push(tool_name.clone());
                }
            }
        }
        
        // Update metadata
        self.update_metadata().await?;
        
        tracing::info!("Registered tool: {} v{}", tool.name, tool.version);
        
        Ok(())
    }
    
    /// Get a tool by name
    pub async fn get_tool(&self, name: &str) -> Option<ToolDefinition> {
        let tools = self.tools.read().await;
        tools.get(name).cloned()
    }
    
    /// List all tools
    pub async fn list_tools(&self) -> Vec<ToolDefinition> {
        let tools = self.tools.read().await;
        tools.values().cloned().collect()
    }
    
    /// Get tools by category
    pub async fn get_tools_by_category(&self, category: &ToolCategory) -> Vec<ToolDefinition> {
        let categories = self.categories.read().await;
        let tools = self.tools.read().await;
        
        if let Some(tool_names) = categories.get(category) {
            tool_names.iter()
                .filter_map(|name| tools.get(name).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Search tools by capability
    pub async fn search_by_capability(&self, capability: &str) -> Vec<ToolDefinition> {
        let capability_index = self.capability_index.read().await;
        let tools = self.tools.read().await;
        
        if let Some(tool_names) = capability_index.get(capability) {
            tool_names.iter()
                .filter_map(|name| tools.get(name).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Search tools by query string
    pub async fn search_tools(&self, query: &str) -> Vec<ToolDefinition> {
        let tools = self.tools.read().await;
        let query_lower = query.to_lowercase();
        
        let mut matching_tools: Vec<_> = tools.values()
            .filter(|tool| {
                let name_match = tool.name.to_lowercase().contains(&query_lower);
                let desc_match = tool.description.to_lowercase().contains(&query_lower);
                let capability_match = tool.capabilities.iter()
                    .any(|cap| cap.to_lowercase().contains(&query_lower));
                let tag_match = tool.metadata.tags.iter()
                    .any(|tag| tag.to_lowercase().contains(&query_lower));
                
                name_match || desc_match || capability_match || tag_match
            })
            .cloned()
            .collect();
        
        // Sort by relevance score
        matching_tools.sort_by(|a, b| {
            b.metadata.relevance_score.partial_cmp(&a.metadata.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        matching_tools
    }
    
    /// Get recommended tools based on usage patterns
    pub async fn get_recommendations(&self, limit: usize) -> Vec<ToolDefinition> {
        let tools = self.tools.read().await;
        
        let mut recommendations: Vec<_> = tools.values()
            .cloned()
            .collect();
        
        // Sort by multiple factors: usage, quality, relevance
        recommendations.sort_by(|a, b| {
            let score_a = self.calculate_recommendation_score(a);
            let score_b = self.calculate_recommendation_score(b);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        recommendations.into_iter().take(limit).collect()
    }
    
    /// Calculate recommendation score for a tool
    fn calculate_recommendation_score(&self, tool: &ToolDefinition) -> f64 {
        let usage_weight = 0.3;
        let quality_weight = 0.4;
        let relevance_weight = 0.3;
        
        let usage_score = if tool.metadata.usage_stats.total_calls > 0 {
            (tool.metadata.usage_stats.successful_calls as f64 / 
             tool.metadata.usage_stats.total_calls as f64).min(1.0)
        } else {
            0.0
        };
        
        let quality_score = (tool.metadata.quality_metrics.success_rate +
                           tool.metadata.quality_metrics.reliability_score +
                           tool.metadata.quality_metrics.performance_score) / 3.0;
        
        let relevance_score = tool.metadata.relevance_score;
        
        usage_score * usage_weight +
        quality_score * quality_weight +
        relevance_score * relevance_weight
    }
    
    /// Update tool usage statistics
    pub async fn update_usage_stats(
        &self,
        tool_name: &str,
        operation: &str,
        success: bool,
        execution_time: std::time::Duration,
    ) -> Result<(), ToolError> {
        let mut tools = self.tools.write().await;
        
        if let Some(tool) = tools.get_mut(tool_name) {
            let stats = &mut tool.metadata.usage_stats;
            
            stats.total_calls += 1;
            if success {
                stats.successful_calls += 1;
            } else {
                stats.failed_calls += 1;
            }
            
            // Update average execution time
            let total_time = stats.avg_execution_time * (stats.total_calls - 1) as u32 + execution_time;
            stats.avg_execution_time = total_time / stats.total_calls as u32;
            
            stats.last_used = Some(chrono::Utc::now());
            
            // Update operation popularity
            *stats.popular_operations.entry(operation.to_string()).or_insert(0) += 1;
            
            // Update quality metrics
            let metrics = &mut tool.metadata.quality_metrics;
            metrics.success_rate = stats.successful_calls as f64 / stats.total_calls as f64;
            metrics.avg_response_time = stats.avg_execution_time;
            
            tool.metadata.updated_at = chrono::Utc::now();
        }
        
        Ok(())
    }
    
    /// Remove a tool from the registry
    pub async fn remove_tool(&self, name: &str) -> Result<(), ToolError> {
        // Remove from main tools collection
        let removed_tool = {
            let mut tools = self.tools.write().await;
            tools.remove(name)
        };
        
        if let Some(tool) = removed_tool {
            // Remove from category index
            {
                let mut categories = self.categories.write().await;
                if let Some(category_tools) = categories.get_mut(&tool.category) {
                    category_tools.retain(|n| n != name);
                }
            }
            
            // Remove from capability index
            {
                let mut capability_index = self.capability_index.write().await;
                for capability in &tool.capabilities {
                    if let Some(capability_tools) = capability_index.get_mut(capability) {
                        capability_tools.retain(|n| n != name);
                    }
                }
            }
            
            // Update metadata
            self.update_metadata().await?;
            
            tracing::info!("Removed tool: {}", name);
        }
        
        Ok(())
    }
    
    /// Register tool from MCP capability
    pub async fn register_from_mcp(&self, server_url: &str, capability: MCPCapability) -> Result<(), ToolError> {
        let tool = ToolDefinition {
            name: format!("mcp_{:?}", capability).to_lowercase(),
            version: "1.0.0".to_string(),
            description: format!("MCP capability: {:?}", capability),
            category: ToolCategory::Custom("MCP".to_string()),
            provider: server_url.to_string(),
            capabilities: vec![format!("{:?}", capability)],
            operations: vec![], // Would be populated from MCP introspection
            auth_requirements: vec![],
            rate_limits: None,
            cost_info: None,
            dependencies: vec![],
            usage_info: ToolUsageInfo {
                installation: None,
                configuration: vec![],
                examples: vec![],
                troubleshooting: vec![],
                documentation: vec![],
            },
            metadata: ToolMetadata {
                registered_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                usage_stats: UsageStats {
                    total_calls: 0,
                    successful_calls: 0,
                    failed_calls: 0,
                    avg_execution_time: std::time::Duration::from_millis(0),
                    last_used: None,
                    popular_operations: HashMap::new(),
                },
                quality_metrics: QualityMetrics {
                    success_rate: 0.0,
                    avg_response_time: std::time::Duration::from_millis(0),
                    satisfaction_score: 0.0,
                    reliability_score: 0.0,
                    performance_score: 0.0,
                },
                relevance_score: 0.5,
                tags: vec!["mcp".to_string()],
                maintainer: None,
                license: None,
            },
        };
        
        self.register_tool(tool).await
    }
    
    /// Register tool from manifest
    pub async fn register_from_manifest(&self, manifest: ToolManifest) -> Result<(), ToolError> {
        let tool = ToolDefinition {
            name: manifest.name,
            version: manifest.version,
            description: manifest.description,
            category: manifest.category,
            provider: "filesystem".to_string(),
            capabilities: manifest.capabilities.iter().map(|c| c.name.clone()).collect(),
            operations: manifest.capabilities.into_iter()
                .flat_map(|cap| cap.operations.into_iter().map(|op| ToolOperation {
                    name: op.name,
                    description: op.description,
                    parameters: op.parameters.into_iter().map(|p| ParameterSpec {
                        name: p.name,
                        param_type: p.parameter_type,
                        description: p.description,
                        required: p.required,
                        default_value: p.default_value,
                        constraints: p.constraints.into_iter().map(|c| match c {
                            super::ParameterConstraint::MinLength(n) => ParameterConstraint::MinLength(n),
                            super::ParameterConstraint::MaxLength(n) => ParameterConstraint::MaxLength(n),
                            super::ParameterConstraint::Pattern(p) => ParameterConstraint::Pattern(p),
                            super::ParameterConstraint::MinValue(v) => ParameterConstraint::MinValue(v),
                            super::ParameterConstraint::MaxValue(v) => ParameterConstraint::MaxValue(v),
                            super::ParameterConstraint::OneOf(vals) => ParameterConstraint::OneOf(vals),
                            super::ParameterConstraint::Custom(name) => ParameterConstraint::Custom {
                                name: name.clone(),
                                description: name,
                            },
                        }).collect(),
                        examples: vec![],
                    }).collect(),
                    returns: ReturnSpec {
                        return_type: op.return_type,
                        description: "Operation result".to_string(),
                        schema: None,
                        examples: vec![],
                    },
                    examples: op.examples.into_iter().map(|ex| OperationExample {
                        name: ex.name,
                        description: ex.description,
                        input: ex.parameters,
                        output: ex.expected_result,
                        scenario: "General usage".to_string(),
                    }).collect(),
                    has_side_effects: op.side_effects.len() > 0,
                    estimated_duration: None,
                }))
                .collect(),
            auth_requirements: if manifest.auth_required {
                vec![AuthRequirement {
                    method: AuthMethod::ApiKey,
                    scopes: vec![],
                    endpoint: None,
                    config: HashMap::new(),
                }]
            } else {
                vec![]
            },
            rate_limits: None,
            cost_info: None,
            dependencies: vec![],
            usage_info: ToolUsageInfo {
                installation: Some(format!("Executable: {:?}", manifest.executable)),
                configuration: vec![],
                examples: vec![],
                troubleshooting: vec![],
                documentation: vec![],
            },
            metadata: ToolMetadata {
                registered_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                usage_stats: UsageStats {
                    total_calls: 0,
                    successful_calls: 0,
                    failed_calls: 0,
                    avg_execution_time: std::time::Duration::from_millis(0),
                    last_used: None,
                    popular_operations: HashMap::new(),
                },
                quality_metrics: QualityMetrics {
                    success_rate: 0.0,
                    avg_response_time: std::time::Duration::from_millis(0),
                    satisfaction_score: 0.0,
                    reliability_score: 0.0,
                    performance_score: 0.0,
                },
                relevance_score: 0.5,
                tags: vec!["filesystem".to_string()],
                maintainer: None,
                license: None,
            },
        };
        
        self.register_tool(tool).await
    }
    
    /// Update registry metadata
    async fn update_metadata(&self) -> Result<(), ToolError> {
        let tools = self.tools.read().await;
        let mut metadata = self.metadata.write().await;
        
        metadata.total_tools = tools.len();
        metadata.last_updated = chrono::Utc::now();
        
        // Update category counts
        metadata.category_counts.clear();
        for tool in tools.values() {
            *metadata.category_counts.entry(tool.category.clone()).or_insert(0) += 1;
        }
        
        // Update popular tools (by usage)
        let mut tool_usage: Vec<_> = tools.iter()
            .map(|(name, tool)| (name.clone(), tool.metadata.usage_stats.total_calls))
            .collect();
        tool_usage.sort_by(|a, b| b.1.cmp(&a.1));
        metadata.popular_tools = tool_usage.into_iter()
            .take(10)
            .map(|(name, _)| name)
            .collect();
        
        // Update recent tools
        let mut recent_tools: Vec<_> = tools.iter()
            .map(|(name, tool)| (name.clone(), tool.metadata.registered_at))
            .collect();
        recent_tools.sort_by(|a, b| b.1.cmp(&a.1));
        metadata.recent_tools = recent_tools.into_iter()
            .take(10)
            .map(|(name, _)| name)
            .collect();
        
        // Update health metrics
        if !tools.is_empty() {
            let total_tools = tools.len() as f64;
            
            metadata.health_metrics.avg_quality_score = tools.values()
                .map(|t| (t.metadata.quality_metrics.success_rate +
                         t.metadata.quality_metrics.reliability_score +
                         t.metadata.quality_metrics.performance_score) / 3.0)
                .sum::<f64>() / total_tools;
            
            metadata.health_metrics.documentation_coverage = tools.values()
                .filter(|t| !t.usage_info.documentation.is_empty())
                .count() as f64 / total_tools;
            
            metadata.health_metrics.maintenance_score = tools.values()
                .filter(|t| t.metadata.maintainer.is_some())
                .count() as f64 / total_tools;
            
            metadata.health_metrics.example_coverage = tools.values()
                .filter(|t| t.operations.iter().any(|op| !op.examples.is_empty()))
                .count() as f64 / total_tools;
        }
        
        Ok(())
    }
    
    /// Get registry metadata
    pub async fn get_metadata(&self) -> RegistryMetadata {
        let metadata = self.metadata.read().await;
        metadata.clone()
    }
    
    /// Export tools to JSON
    pub async fn export_tools(&self) -> Result<String, ToolError> {
        let tools = self.tools.read().await;
        serde_json::to_string_pretty(&*tools)
            .map_err(|e| ToolError::SerializationError(e))
    }
    
    /// Import tools from JSON
    pub async fn import_tools(&self, json: &str) -> Result<(), ToolError> {
        let imported_tools: HashMap<String, ToolDefinition> = serde_json::from_str(json)
            .map_err(|e| ToolError::SerializationError(e))?;
        
        for (_, tool) in imported_tools {
            self.register_tool(tool).await?;
        }
        
        Ok(())
    }
    
    /// Get registry statistics
    pub async fn get_stats(&self) -> RegistryStats {
        let tools = self.tools.read().await;
        let metadata = self.metadata.read().await;
        
        RegistryStats {
            total_tools: tools.len(),
            categories: metadata.category_counts.clone(),
            total_operations: tools.values().map(|t| t.operations.len()).sum(),
            total_usage: tools.values().map(|t| t.metadata.usage_stats.total_calls).sum(),
            health_score: (metadata.health_metrics.avg_quality_score +
                          metadata.health_metrics.documentation_coverage +
                          metadata.health_metrics.maintenance_score +
                          metadata.health_metrics.example_coverage) / 4.0,
            last_updated: metadata.last_updated,
        }
    }
}

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_tools: usize,
    pub categories: HashMap<ToolCategory, usize>,
    pub total_operations: usize,
    pub total_usage: u64,
    pub health_score: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}