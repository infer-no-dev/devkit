//! Tool Providers
//!
//! Different providers for various types of tools and services.

use super::{ToolError, ToolCapability, ToolCategory};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Tool provider trait
#[async_trait]
pub trait ToolProvider {
    /// Get provider name
    fn name(&self) -> &str;
    
    /// Get provider capabilities
    async fn get_capabilities(&self) -> Result<Vec<ProviderCapability>, ToolError>;
    
    /// List available tools
    async fn list_tools(&self) -> Result<Vec<ToolCapability>, ToolError>;
    
    /// Get specific tool information
    async fn get_tool(&self, name: &str) -> Result<Option<ToolCapability>, ToolError>;
    
    /// Execute a tool operation
    async fn execute(
        &self,
        tool_name: &str,
        operation: &str,
        parameters: HashMap<String, serde_json::Value>,
        context: ExecutionContext,
    ) -> Result<ExecutionResult, ToolError>;
    
    /// Check if the provider is healthy/available
    async fn health_check(&self) -> Result<ProviderHealth, ToolError>;
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider type
    pub provider_type: ProviderType,
    /// Provider name/identifier
    pub name: String,
    /// Configuration parameters
    pub config: HashMap<String, serde_json::Value>,
    /// Whether the provider is enabled
    pub enabled: bool,
    /// Priority for tool resolution
    pub priority: u32,
}

/// Provider capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCapability {
    /// Capability name
    pub name: String,
    /// Capability description
    pub description: String,
    /// Supported operations
    pub operations: Vec<String>,
    /// Tool categories this capability supports
    pub supported_categories: Vec<ToolCategory>,
    /// Configuration requirements
    pub requirements: Vec<ConfigRequirement>,
}

/// Configuration requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRequirement {
    /// Configuration key
    pub key: String,
    /// Whether it's required
    pub required: bool,
    /// Description
    pub description: String,
    /// Default value
    pub default_value: Option<serde_json::Value>,
}

/// Provider health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    /// Whether the provider is healthy
    pub healthy: bool,
    /// Status message
    pub status: String,
    /// Response time
    pub response_time: std::time::Duration,
    /// Additional metrics
    pub metrics: HashMap<String, serde_json::Value>,
}

/// Provider types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderType {
    /// Local shell/executable tools
    Shell,
    /// HTTP/REST API tools
    Http,
    /// Docker container tools
    Docker,
    /// Cloud service tools (AWS, GCP, Azure)
    Cloud,
    /// Version control tools (git, hg)
    VersionControl,
    /// Database tools
    Database,
    /// AI/ML model tools
    AiModel,
    /// Custom provider type
    Custom(String),
}

/// Execution context for tool operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Tool name
    pub tool_name: String,
    /// Operation name
    pub operation: String,
    /// Operation parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Execution context metadata
    pub context: HashMap<String, serde_json::Value>,
    /// Authentication credentials
    pub credentials: Option<super::auth::Credential>,
    /// Execution timeout
    pub timeout: Option<std::time::Duration>,
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Whether execution succeeded
    pub success: bool,
    /// Result output
    pub output: serde_json::Value,
    /// Execution duration
    pub duration: std::time::Duration,
    /// Generated artifacts
    pub artifacts: Vec<ExecutionArtifact>,
    /// Warnings or informational messages
    pub warnings: Vec<String>,
    /// Debug information
    pub debug_info: HashMap<String, serde_json::Value>,
    /// Cost information if applicable
    pub cost: Option<f64>,
    /// Rate limit information
    pub rate_limit_remaining: Option<u32>,
}

/// Execution artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionArtifact {
    /// Artifact type
    pub artifact_type: super::ArtifactType,
    /// Artifact name
    pub name: String,
    /// Artifact content
    pub content: serde_json::Value,
    /// MIME type
    pub mime_type: String,
    /// Size in bytes
    pub size_bytes: u64,
    /// Checksum
    pub checksum: String,
}

/// Shell tool provider
#[derive(Debug)]
pub struct ShellProvider {
    name: String,
    config: HashMap<String, serde_json::Value>,
}

/// HTTP tool provider
#[derive(Debug)]
pub struct HttpProvider {
    name: String,
    config: HashMap<String, serde_json::Value>,
    client: reqwest::Client,
}

/// Docker tool provider
#[derive(Debug)]
pub struct DockerProvider {
    name: String,
    config: HashMap<String, serde_json::Value>,
}

/// Cloud provider (AWS, GCP, Azure)
#[derive(Debug)]
pub struct CloudProvider {
    name: String,
    provider_type: CloudProviderType,
    config: HashMap<String, serde_json::Value>,
}

/// Cloud provider types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloudProviderType {
    Aws,
    Gcp,
    Azure,
    Custom(String),
}

impl ShellProvider {
    pub fn new(name: String, config: HashMap<String, serde_json::Value>) -> Self {
        Self { name, config }
    }
}

#[async_trait]
impl ToolProvider for ShellProvider {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn get_capabilities(&self) -> Result<Vec<ProviderCapability>, ToolError> {
        Ok(vec![
            ProviderCapability {
                name: "shell_execution".to_string(),
                description: "Execute shell commands and scripts".to_string(),
                operations: vec!["execute".to_string(), "which".to_string(), "help".to_string()],
                supported_categories: vec![
                    ToolCategory::VersionControl,
                    ToolCategory::BuildTools,
                    ToolCategory::Analysis,
                    ToolCategory::Custom("System".to_string()),
                ],
                requirements: vec![
                    ConfigRequirement {
                        key: "shell".to_string(),
                        required: false,
                        description: "Shell to use (bash, zsh, fish, etc.)".to_string(),
                        default_value: Some(serde_json::Value::String("bash".to_string())),
                    },
                ],
            }
        ])
    }
    
    async fn list_tools(&self) -> Result<Vec<ToolCapability>, ToolError> {
        // Discover available shell tools
        let mut tools = Vec::new();
        
        // Common shell tools
        let common_tools = vec![
            ("git", "Version control system"),
            ("npm", "Node.js package manager"),
            ("cargo", "Rust package manager"),
            ("python", "Python interpreter"),
            ("docker", "Container platform"),
            ("kubectl", "Kubernetes CLI"),
            ("terraform", "Infrastructure as code"),
        ];
        
        for (tool_name, description) in common_tools {
            // Check if tool exists
            if let Ok(output) = tokio::process::Command::new("which")
                .arg(tool_name)
                .output()
                .await
            {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    tools.push(ToolCapability {
                        name: tool_name.to_string(),
                        description: description.to_string(),
                        operations: vec![
                            super::ToolOperation {
                                name: "execute".to_string(),
                                description: format!("Execute {} command", tool_name),
                                parameters: vec![],
                                return_type: "object".to_string(),
                                examples: vec![],
                                side_effects: vec![],
                            }
                        ],
                        required_auth: vec![],
                        rate_limits: None,
                        cost_model: None,
                        dependencies: vec![],
                    });
                }
            }
        }
        
        Ok(tools)
    }
    
    async fn get_tool(&self, name: &str) -> Result<Option<ToolCapability>, ToolError> {
        let tools = self.list_tools().await?;
        Ok(tools.into_iter().find(|t| t.name == name))
    }
    
    async fn execute(
        &self,
        tool_name: &str,
        operation: &str,
        parameters: HashMap<String, serde_json::Value>,
        context: ExecutionContext,
    ) -> Result<ExecutionResult, ToolError> {
        let start_time = std::time::Instant::now();
        
        match operation {
            "execute" => {
                let args = parameters.get("args")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
                    .unwrap_or_else(Vec::new);
                
                let mut command = tokio::process::Command::new(tool_name);
                command.args(&args);
                
                // Set working directory if provided
                if let Some(workdir) = context.context.get("working_directory") {
                    if let Some(dir) = workdir.as_str() {
                        command.current_dir(dir);
                    }
                }
                
                // Set environment variables
                if let Some(env) = context.context.get("environment") {
                    if let Some(env_obj) = env.as_object() {
                        for (key, value) in env_obj {
                            if let Some(val_str) = value.as_str() {
                                command.env(key, val_str);
                            }
                        }
                    }
                }
                
                let output = command.output().await
                    .map_err(|e| ToolError::ExecutionTimeout(format!("Failed to execute {}: {}", tool_name, e)))?;
                
                let duration = start_time.elapsed();
                let success = output.status.success();
                
                let result_output = if success {
                    serde_json::json!({
                        "stdout": String::from_utf8_lossy(&output.stdout),
                        "stderr": String::from_utf8_lossy(&output.stderr),
                        "exit_code": output.status.code().unwrap_or(-1)
                    })
                } else {
                    serde_json::json!({
                        "error": String::from_utf8_lossy(&output.stderr),
                        "stdout": String::from_utf8_lossy(&output.stdout),
                        "exit_code": output.status.code().unwrap_or(-1)
                    })
                };
                
                Ok(ExecutionResult {
                    success,
                    output: result_output,
                    duration,
                    artifacts: vec![],
                    warnings: if !success { vec!["Command execution failed".to_string()] } else { vec![] },
                    debug_info: HashMap::new(),
                    cost: None,
                    rate_limit_remaining: None,
                })
            },
            "which" => {
                let output = tokio::process::Command::new("which")
                    .arg(tool_name)
                    .output()
                    .await
                    .map_err(|e| ToolError::ExecutionTimeout(format!("Failed to execute which: {}", e)))?;
                
                let duration = start_time.elapsed();
                let success = output.status.success();
                
                let path_str = if success { 
                    String::from_utf8_lossy(&output.stdout).trim().to_string()
                } else { 
                    String::new()
                };
                
                let result_output = serde_json::json!({
                    "path": path_str,
                    "found": success
                });
                
                Ok(ExecutionResult {
                    success,
                    output: result_output,
                    duration,
                    artifacts: vec![],
                    warnings: vec![],
                    debug_info: HashMap::new(),
                    cost: None,
                    rate_limit_remaining: None,
                })
            },
            _ => Err(ToolError::OperationNotSupported {
                tool: tool_name.to_string(),
                operation: operation.to_string(),
            }),
        }
    }
    
    async fn health_check(&self) -> Result<ProviderHealth, ToolError> {
        let start_time = std::time::Instant::now();
        
        // Test basic shell availability
        let result = tokio::process::Command::new("echo")
            .arg("health_check")
            .output()
            .await;
        
        let response_time = start_time.elapsed();
        
        match result {
            Ok(output) if output.status.success() => {
                Ok(ProviderHealth {
                    healthy: true,
                    status: "Shell provider is healthy".to_string(),
                    response_time,
                    metrics: HashMap::from([
                        ("shell_available".to_string(), serde_json::Value::Bool(true)),
                    ]),
                })
            },
            _ => {
                Ok(ProviderHealth {
                    healthy: false,
                    status: "Shell provider is not available".to_string(),
                    response_time,
                    metrics: HashMap::from([
                        ("shell_available".to_string(), serde_json::Value::Bool(false)),
                    ]),
                })
            }
        }
    }
}

impl HttpProvider {
    pub fn new(name: String, config: HashMap<String, serde_json::Value>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        
        Self { name, config, client }
    }
}

#[async_trait]
impl ToolProvider for HttpProvider {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn get_capabilities(&self) -> Result<Vec<ProviderCapability>, ToolError> {
        Ok(vec![
            ProviderCapability {
                name: "http_requests".to_string(),
                description: "Make HTTP requests to external APIs".to_string(),
                operations: vec!["get".to_string(), "post".to_string(), "put".to_string(), "delete".to_string()],
                supported_categories: vec![
                    ToolCategory::Communication,
                    ToolCategory::ProjectManagement,
                    ToolCategory::Infrastructure,
                    ToolCategory::Custom("API".to_string()),
                ],
                requirements: vec![
                    ConfigRequirement {
                        key: "base_url".to_string(),
                        required: false,
                        description: "Base URL for API requests".to_string(),
                        default_value: None,
                    },
                ],
            }
        ])
    }
    
    async fn list_tools(&self) -> Result<Vec<ToolCapability>, ToolError> {
        // This would typically query a registry or configuration to find available HTTP tools
        Ok(vec![])
    }
    
    async fn get_tool(&self, name: &str) -> Result<Option<ToolCapability>, ToolError> {
        let tools = self.list_tools().await?;
        Ok(tools.into_iter().find(|t| t.name == name))
    }
    
    async fn execute(
        &self,
        tool_name: &str,
        operation: &str,
        parameters: HashMap<String, serde_json::Value>,
        context: ExecutionContext,
    ) -> Result<ExecutionResult, ToolError> {
        let start_time = std::time::Instant::now();
        
        let url = parameters.get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing URL parameter".to_string()))?;
        
        let result = match operation {
            "get" => self.client.get(url).send().await,
            "post" => {
                let mut request = self.client.post(url);
                
                if let Some(body) = parameters.get("body") {
                    request = request.json(body);
                }
                
                request.send().await
            },
            "put" => {
                let mut request = self.client.put(url);
                
                if let Some(body) = parameters.get("body") {
                    request = request.json(body);
                }
                
                request.send().await
            },
            "delete" => self.client.delete(url).send().await,
            _ => return Err(ToolError::OperationNotSupported {
                tool: tool_name.to_string(),
                operation: operation.to_string(),
            }),
        };
        
        let duration = start_time.elapsed();
        
        match result {
            Ok(response) => {
                let status = response.status();
                let headers: HashMap<String, String> = response.headers()
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                    .collect();
                
                let body = response.text().await.unwrap_or_default();
                let body_json: serde_json::Value = serde_json::from_str(&body)
                    .unwrap_or_else(|_| serde_json::Value::String(body));
                
                let success = status.is_success();
                
                Ok(ExecutionResult {
                    success,
                    output: serde_json::json!({
                        "status": status.as_u16(),
                        "headers": headers,
                        "body": body_json
                    }),
                    duration,
                    artifacts: vec![],
                    warnings: if !success { vec![format!("HTTP request failed with status {}", status)] } else { vec![] },
                    debug_info: HashMap::new(),
                    cost: None,
                    rate_limit_remaining: None,
                })
            },
            Err(e) => {
                Ok(ExecutionResult {
                    success: false,
                    output: serde_json::json!({
                        "error": e.to_string()
                    }),
                    duration,
                    artifacts: vec![],
                    warnings: vec!["HTTP request failed".to_string()],
                    debug_info: HashMap::new(),
                    cost: None,
                    rate_limit_remaining: None,
                })
            }
        }
    }
    
    async fn health_check(&self) -> Result<ProviderHealth, ToolError> {
        let start_time = std::time::Instant::now();
        
        // Test basic HTTP connectivity
        let result = self.client.get("https://httpbin.org/get").send().await;
        let response_time = start_time.elapsed();
        
        match result {
            Ok(response) if response.status().is_success() => {
                Ok(ProviderHealth {
                    healthy: true,
                    status: "HTTP provider is healthy".to_string(),
                    response_time,
                    metrics: HashMap::from([
                        ("http_available".to_string(), serde_json::Value::Bool(true)),
                    ]),
                })
            },
            _ => {
                Ok(ProviderHealth {
                    healthy: false,
                    status: "HTTP provider is not available".to_string(),
                    response_time,
                    metrics: HashMap::from([
                        ("http_available".to_string(), serde_json::Value::Bool(false)),
                    ]),
                })
            }
        }
    }
}

/// Create a provider based on configuration
pub fn create_provider(config: ProviderConfig) -> Result<Arc<dyn ToolProvider + Send + Sync>, ToolError> {
    match config.provider_type {
        ProviderType::Shell => {
            Ok(Arc::new(ShellProvider::new(config.name, config.config)))
        },
        ProviderType::Http => {
            Ok(Arc::new(HttpProvider::new(config.name, config.config)))
        },
        ProviderType::Docker => {
            Ok(Arc::new(DockerProvider::new(config.name, config.config)))
        },
        ProviderType::Cloud => {
            // Determine cloud provider type
            let cloud_type = config.config.get("cloud_type")
                .and_then(|v| v.as_str())
                .map(|s| match s {
                    "aws" => CloudProviderType::Aws,
                    "gcp" => CloudProviderType::Gcp,
                    "azure" => CloudProviderType::Azure,
                    other => CloudProviderType::Custom(other.to_string()),
                })
                .unwrap_or(CloudProviderType::Aws);
            
            Ok(Arc::new(CloudProvider::new(config.name, cloud_type, config.config)))
        },
        _ => Err(ToolError::ConfigurationError(format!(
            "Unsupported provider type: {:?}", config.provider_type
        ))),
    }
}

impl DockerProvider {
    pub fn new(name: String, config: HashMap<String, serde_json::Value>) -> Self {
        Self { name, config }
    }
}

#[async_trait]
impl ToolProvider for DockerProvider {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn get_capabilities(&self) -> Result<Vec<ProviderCapability>, ToolError> {
        Ok(vec![
            ProviderCapability {
                name: "container_execution".to_string(),
                description: "Execute tools in Docker containers".to_string(),
                operations: vec!["run".to_string(), "exec".to_string(), "build".to_string()],
                supported_categories: vec![
                    ToolCategory::BuildTools,
                    ToolCategory::Testing,
                    ToolCategory::Analysis,
                    ToolCategory::Infrastructure,
                ],
                requirements: vec![],
            }
        ])
    }
    
    async fn list_tools(&self) -> Result<Vec<ToolCapability>, ToolError> {
        // This would query available Docker images/containers
        Ok(vec![])
    }
    
    async fn get_tool(&self, _name: &str) -> Result<Option<ToolCapability>, ToolError> {
        Ok(None)
    }
    
    async fn execute(
        &self,
        _tool_name: &str,
        _operation: &str,
        _parameters: HashMap<String, serde_json::Value>,
        _context: ExecutionContext,
    ) -> Result<ExecutionResult, ToolError> {
        // Docker execution would be implemented here
        Err(ToolError::ProviderError("Docker provider not implemented yet".to_string()))
    }
    
    async fn health_check(&self) -> Result<ProviderHealth, ToolError> {
        Ok(ProviderHealth {
            healthy: false,
            status: "Docker provider not implemented".to_string(),
            response_time: std::time::Duration::from_millis(0),
            metrics: HashMap::new(),
        })
    }
}

impl CloudProvider {
    pub fn new(name: String, provider_type: CloudProviderType, config: HashMap<String, serde_json::Value>) -> Self {
        Self { name, provider_type, config }
    }
}

#[async_trait]
impl ToolProvider for CloudProvider {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn get_capabilities(&self) -> Result<Vec<ProviderCapability>, ToolError> {
        let capabilities = match self.provider_type {
            CloudProviderType::Aws => vec!["s3", "ec2", "lambda", "rds"],
            CloudProviderType::Gcp => vec!["compute", "storage", "functions"],
            CloudProviderType::Azure => vec!["storage", "compute", "functions"],
            CloudProviderType::Custom(_) => vec!["custom"],
        };
        
        Ok(capabilities.into_iter().map(|cap| {
            ProviderCapability {
                name: cap.to_string(),
                description: format!("{} service capability", cap),
                operations: vec!["list".to_string(), "create".to_string(), "delete".to_string()],
                supported_categories: vec![ToolCategory::Infrastructure],
                requirements: vec![],
            }
        }).collect())
    }
    
    async fn list_tools(&self) -> Result<Vec<ToolCapability>, ToolError> {
        // This would query available cloud services/tools
        Ok(vec![])
    }
    
    async fn get_tool(&self, _name: &str) -> Result<Option<ToolCapability>, ToolError> {
        Ok(None)
    }
    
    async fn execute(
        &self,
        _tool_name: &str,
        _operation: &str,
        _parameters: HashMap<String, serde_json::Value>,
        _context: ExecutionContext,
    ) -> Result<ExecutionResult, ToolError> {
        // Cloud provider execution would be implemented here
        Err(ToolError::ProviderError("Cloud provider not implemented yet".to_string()))
    }
    
    async fn health_check(&self) -> Result<ProviderHealth, ToolError> {
        Ok(ProviderHealth {
            healthy: false,
            status: "Cloud provider not implemented".to_string(),
            response_time: std::time::Duration::from_millis(0),
            metrics: HashMap::new(),
        })
    }
}