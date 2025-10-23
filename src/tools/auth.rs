//! Authentication Broker
//!
//! Manages authentication credentials and methods for tool access.

use super::{ToolError, AuthConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::secrets::SecretsManager;
use crate::secrets::vault::{SecretType, SecretMetadata};

pub use crate::tools::registry::AuthMethod;

/// Authentication broker
pub struct AuthBroker {
    /// Secret manager for secure credential storage
    secret_manager: Arc<SecretsManager>,
    /// Active credentials cache
    credentials: Arc<RwLock<HashMap<String, Credential>>>,
    /// Authentication configurations
    auth_configs: HashMap<String, AuthConfig>,
    /// Token refresh handlers
    refresh_handlers: Arc<RwLock<HashMap<String, Box<dyn RefreshHandler + Send + Sync>>>>,
}

impl std::fmt::Debug for AuthBroker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthBroker")
            .field("secret_manager", &"SecretsManager")
            .field("credentials", &self.credentials)
            .field("auth_configs", &self.auth_configs)
            .field("refresh_handlers", &"[trait objects]")
            .finish()
    }
}

/// Credential information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    /// Credential type
    pub auth_method: AuthMethod,
    /// The actual credential data
    pub data: CredentialData,
    /// When the credential expires
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Associated scopes/permissions
    pub scopes: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Credential data variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CredentialData {
    ApiKey {
        key: String,
        header_name: Option<String>,
    },
    Bearer {
        token: String,
    },
    Basic {
        username: String,
        password: String,
    },
    OAuth2 {
        access_token: String,
        refresh_token: Option<String>,
        token_type: String,
    },
    Certificate {
        cert_path: String,
        key_path: String,
        passphrase: Option<String>,
    },
    Custom {
        auth_type: String,
        data: HashMap<String, String>,
    },
}

/// Token refresh handler trait
#[async_trait::async_trait]
pub trait RefreshHandler {
    /// Refresh the credential
    async fn refresh(&self, credential: &Credential) -> Result<Credential, ToolError>;
    
    /// Check if the credential needs refresh
    fn needs_refresh(&self, credential: &Credential) -> bool;
}

/// OAuth2 refresh handler
#[derive(Debug)]
pub struct OAuth2RefreshHandler {
    client_id: String,
    client_secret: String,
    token_endpoint: String,
}

/// Authentication request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    /// Tool or service name
    pub service: String,
    /// Required scopes
    pub scopes: Vec<String>,
    /// Authentication method preference
    pub preferred_method: Option<AuthMethod>,
    /// Additional parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Authentication response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    /// Whether authentication succeeded
    pub success: bool,
    /// Credential if successful
    pub credential: Option<Credential>,
    /// Error message if failed
    pub error: Option<String>,
    /// Additional response data
    pub metadata: HashMap<String, serde_json::Value>,
}

impl AuthBroker {
    /// Create a new authentication broker
    pub async fn new(auth_configs: HashMap<String, AuthConfig>) -> Result<Self, ToolError> {
        let secrets_config = crate::secrets::SecretsConfig::default();
        let secret_manager = SecretsManager::new(secrets_config).await
            .map_err(|e| ToolError::ConfigurationError(format!("Failed to initialize secrets manager: {}", e)))?;
        
        Ok(Self {
            secret_manager: Arc::new(secret_manager),
            credentials: Arc::new(RwLock::new(HashMap::new())),
            auth_configs,
            refresh_handlers: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Get credentials for a service
    pub async fn get_credentials(&self, service: &str) -> Result<Credential, ToolError> {
        // Check if we have cached credentials
        {
            let credentials = self.credentials.read().await;
            if let Some(credential) = credentials.get(service) {
                // Check if credential is still valid
                if !self.is_expired(credential) {
                    return Ok(credential.clone());
                }
            }
        }
        
        // Try to refresh if we have a refresh handler
        if let Some(credential) = self.try_refresh_credential(service).await? {
            return Ok(credential);
        }
        
        // Load from secret manager
        if let Some(credential) = self.load_credential(service).await? {
            // Cache the credential
            {
                let mut credentials = self.credentials.write().await;
                credentials.insert(service.to_string(), credential.clone());
            }
            return Ok(credential);
        }
        
        // No credential available
        Err(ToolError::AuthenticationFailed(format!(
            "No credentials found for service: {}", service
        )))
    }
    
    /// Store credentials for a service
    pub async fn store_credentials(&self, service: &str, credential: Credential) -> Result<(), ToolError> {
        // Store in secret manager
        self.save_credential(service, &credential).await?;
        
        // Cache the credential
        {
            let mut credentials = self.credentials.write().await;
            credentials.insert(service.to_string(), credential);
        }
        
        Ok(())
    }
    
    /// Authenticate with a service
    pub async fn authenticate(&self, request: AuthRequest) -> Result<AuthResponse, ToolError> {
        let service = &request.service;
        
        // Get auth configuration for the service
        let auth_config = self.auth_configs.get(service)
            .ok_or_else(|| ToolError::ConfigurationError(format!(
                "No auth configuration found for service: {}", service
            )))?;
        
        let credential = match &auth_config.auth_method {
            AuthMethod::None => {
                return Ok(AuthResponse {
                    success: true,
                    credential: None,
                    error: None,
                    metadata: HashMap::new(),
                });
            },
            AuthMethod::ApiKey => {
                self.authenticate_api_key(service, &request).await?
            },
            AuthMethod::OAuth2 => {
                self.authenticate_oauth2(service, auth_config, &request).await?
            },
            AuthMethod::BasicAuth => {
                self.authenticate_basic(service, &request).await?
            },
            AuthMethod::BearerToken => {
                self.authenticate_bearer(service, &request).await?
            },
            AuthMethod::Certificate => {
                self.authenticate_certificate(service, &request).await?
            },
            AuthMethod::Custom(method) => {
                self.authenticate_custom(service, method, &request).await?
            },
        };
        
        // Store the credential
        self.store_credentials(service, credential.clone()).await?;
        
        Ok(AuthResponse {
            success: true,
            credential: Some(credential),
            error: None,
            metadata: HashMap::new(),
        })
    }
    
    /// Check if a credential is expired
    fn is_expired(&self, credential: &Credential) -> bool {
        if let Some(expires_at) = credential.expires_at {
            chrono::Utc::now() > expires_at
        } else {
            false
        }
    }
    
    /// Try to refresh a credential
    async fn try_refresh_credential(&self, service: &str) -> Result<Option<Credential>, ToolError> {
        let handlers = self.refresh_handlers.read().await;
        
        if let Some(handler) = handlers.get(service) {
            // Get current credential
            let current_credential = {
                let credentials = self.credentials.read().await;
                credentials.get(service).cloned()
            };
            
            if let Some(credential) = current_credential {
                if handler.needs_refresh(&credential) {
                    let refreshed = handler.refresh(&credential).await?;
                    
                    // Update cache
                    {
                        let mut credentials = self.credentials.write().await;
                        credentials.insert(service.to_string(), refreshed.clone());
                    }
                    
                    return Ok(Some(refreshed));
                }
            }
        }
        
        Ok(None)
    }
    
    /// Load credential from secret manager
    async fn load_credential(&self, service: &str) -> Result<Option<Credential>, ToolError> {
        let secret_key = format!("tool_auth_{}", service);
        
        let request = crate::secrets::SecretRequest {
            secret_name: secret_key,
            requester: "auth_broker".to_string(),
            application: Some("tool_authentication".to_string()),
            justification: Some("Tool authentication credential".to_string()),
            ttl: None,
            scope: vec![],
        };
        
        if let Ok(secret_response) = self.secret_manager.get_secret(request).await {
            if let Some(secret_value) = secret_response.secret_value {
                let credential: Credential = serde_json::from_str(&secret_value)
                    .map_err(|e| ToolError::SerializationError(e))?;
                Ok(Some(credential))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
    
    /// Save credential to secret manager
    async fn save_credential(&self, service: &str, credential: &Credential) -> Result<(), ToolError> {
        let secret_key = format!("tool_auth_{}", service);
        let secret_value = serde_json::to_string(credential)
            .map_err(|e| ToolError::SerializationError(e))?;
        
        let metadata = SecretMetadata {
            name: service.to_string(),
            description: Some("Tool authentication credential".to_string()),
            tags: vec!["tool".to_string(), "auth".to_string()],
            owner: "system".to_string(),
            secret_type: SecretType::Custom("tool_credential".to_string()),
            rotation_policy: None,
            access_policy: None,
            custom_fields: HashMap::new(),
        };
        
        self.secret_manager.store_secret(&secret_key, &secret_value, metadata, "auth_broker").await
            .map_err(|e| ToolError::ProviderError(format!("Failed to store credential: {}", e)))?;
        
        Ok(())
    }
    
    /// Authenticate using API key
    async fn authenticate_api_key(&self, service: &str, request: &AuthRequest) -> Result<Credential, ToolError> {
        // Try to get API key from parameters
        let api_key = request.parameters.get("api_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing api_key parameter".to_string()))?;
        
        let header_name = request.parameters.get("header_name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        Ok(Credential {
            auth_method: AuthMethod::ApiKey,
            data: CredentialData::ApiKey {
                key: api_key.to_string(),
                header_name,
            },
            expires_at: None, // API keys typically don't expire
            scopes: request.scopes.clone(),
            metadata: HashMap::new(),
        })
    }
    
    /// Authenticate using OAuth2
    async fn authenticate_oauth2(&self, service: &str, auth_config: &AuthConfig, request: &AuthRequest) -> Result<Credential, ToolError> {
        // This would typically involve redirecting to OAuth2 provider
        // For now, assume we have the tokens in parameters
        
        let access_token = request.parameters.get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing access_token parameter".to_string()))?;
        
        let refresh_token = request.parameters.get("refresh_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let token_type = request.parameters.get("token_type")
            .and_then(|v| v.as_str())
            .unwrap_or("Bearer");
        
        let expires_in = request.parameters.get("expires_in")
            .and_then(|v| v.as_u64());
        
        let expires_at = expires_in.map(|seconds| {
            chrono::Utc::now() + chrono::Duration::seconds(seconds as i64)
        });
        
        // Set up refresh handler if we have refresh token
        if let Some(ref refresh_token) = refresh_token {
            let handler = OAuth2RefreshHandler {
                client_id: request.parameters.get("client_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                client_secret: request.parameters.get("client_secret")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                token_endpoint: auth_config.endpoint.clone()
                    .unwrap_or_else(|| "".to_string()),
            };
            
            let mut handlers = self.refresh_handlers.write().await;
            handlers.insert(service.to_string(), Box::new(handler));
        }
        
        Ok(Credential {
            auth_method: AuthMethod::OAuth2,
            data: CredentialData::OAuth2 {
                access_token: access_token.to_string(),
                refresh_token,
                token_type: token_type.to_string(),
            },
            expires_at,
            scopes: request.scopes.clone(),
            metadata: HashMap::new(),
        })
    }
    
    /// Authenticate using basic auth
    async fn authenticate_basic(&self, service: &str, request: &AuthRequest) -> Result<Credential, ToolError> {
        let username = request.parameters.get("username")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing username parameter".to_string()))?;
        
        let password = request.parameters.get("password")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing password parameter".to_string()))?;
        
        Ok(Credential {
            auth_method: AuthMethod::BasicAuth,
            data: CredentialData::Basic {
                username: username.to_string(),
                password: password.to_string(),
            },
            expires_at: None,
            scopes: request.scopes.clone(),
            metadata: HashMap::new(),
        })
    }
    
    /// Authenticate using bearer token
    async fn authenticate_bearer(&self, service: &str, request: &AuthRequest) -> Result<Credential, ToolError> {
        let token = request.parameters.get("token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing token parameter".to_string()))?;
        
        let expires_in = request.parameters.get("expires_in")
            .and_then(|v| v.as_u64());
        
        let expires_at = expires_in.map(|seconds| {
            chrono::Utc::now() + chrono::Duration::seconds(seconds as i64)
        });
        
        Ok(Credential {
            auth_method: AuthMethod::BearerToken,
            data: CredentialData::Bearer {
                token: token.to_string(),
            },
            expires_at,
            scopes: request.scopes.clone(),
            metadata: HashMap::new(),
        })
    }
    
    /// Authenticate using certificates
    async fn authenticate_certificate(&self, service: &str, request: &AuthRequest) -> Result<Credential, ToolError> {
        let cert_path = request.parameters.get("cert_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing cert_path parameter".to_string()))?;
        
        let key_path = request.parameters.get("key_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing key_path parameter".to_string()))?;
        
        let passphrase = request.parameters.get("passphrase")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        Ok(Credential {
            auth_method: AuthMethod::Certificate,
            data: CredentialData::Certificate {
                cert_path: cert_path.to_string(),
                key_path: key_path.to_string(),
                passphrase,
            },
            expires_at: None,
            scopes: request.scopes.clone(),
            metadata: HashMap::new(),
        })
    }
    
    /// Authenticate using custom method
    async fn authenticate_custom(&self, service: &str, method: &str, request: &AuthRequest) -> Result<Credential, ToolError> {
        let mut data = HashMap::new();
        
        // Copy all parameters as custom auth data
        for (key, value) in &request.parameters {
            if let Some(str_value) = value.as_str() {
                data.insert(key.clone(), str_value.to_string());
            }
        }
        
        Ok(Credential {
            auth_method: AuthMethod::Custom(method.to_string()),
            data: CredentialData::Custom {
                auth_type: method.to_string(),
                data,
            },
            expires_at: None,
            scopes: request.scopes.clone(),
            metadata: HashMap::new(),
        })
    }
    
    /// Remove credentials for a service
    pub async fn remove_credentials(&self, service: &str) -> Result<(), ToolError> {
        // Remove from cache
        {
            let mut credentials = self.credentials.write().await;
            credentials.remove(service);
        }
        
        // Remove from secret manager
        let secret_key = format!("tool_auth_{}", service);
        let _ = self.secret_manager.delete_secret(&secret_key, "auth_broker").await;
        
        Ok(())
    }
    
    /// List services with stored credentials
    pub async fn list_authenticated_services(&self) -> Vec<String> {
        let credentials = self.credentials.read().await;
        credentials.keys().cloned().collect()
    }
    
    /// Get authentication statistics
    pub async fn get_stats(&self) -> AuthBrokerStats {
        let credentials = self.credentials.read().await;
        let handlers = self.refresh_handlers.read().await;
        
        let mut auth_methods = HashMap::new();
        let mut expired_count = 0;
        
        for credential in credentials.values() {
            let method_name = match &credential.auth_method {
                AuthMethod::Custom(name) => name.clone(),
                other => format!("{:?}", other),
            };
            *auth_methods.entry(method_name).or_insert(0) += 1;
            
            if self.is_expired(credential) {
                expired_count += 1;
            }
        }
        
        AuthBrokerStats {
            total_credentials: credentials.len(),
            auth_methods,
            expired_credentials: expired_count,
            refresh_handlers: handlers.len(),
            configured_services: self.auth_configs.len(),
        }
    }
}

#[async_trait::async_trait]
impl RefreshHandler for OAuth2RefreshHandler {
    async fn refresh(&self, credential: &Credential) -> Result<Credential, ToolError> {
        if let CredentialData::OAuth2 { refresh_token: Some(refresh_token), .. } = &credential.data {
            // Make HTTP request to refresh token
            let client = reqwest::Client::new();
            
            let mut params = HashMap::new();
            params.insert("grant_type", "refresh_token");
            params.insert("refresh_token", refresh_token);
            params.insert("client_id", &self.client_id);
            params.insert("client_secret", &self.client_secret);
            
            let response = client
                .post(&self.token_endpoint)
                .form(&params)
                .send()
                .await
                .map_err(|e| ToolError::MCPError(format!("Token refresh failed: {}", e)))?;
            
            if response.status().is_success() {
                let token_response: serde_json::Value = response.json().await
                    .map_err(|e| ToolError::MCPError(format!("Failed to parse token response: {}", e)))?;
                
                let new_access_token = token_response["access_token"]
                    .as_str()
                    .ok_or_else(|| ToolError::MCPError("Missing access_token in refresh response".to_string()))?;
                
                let new_refresh_token = token_response["refresh_token"]
                    .as_str()
                    .map(|s| s.to_string())
                    .or_else(|| Some(refresh_token.clone()));
                
                let expires_in = token_response["expires_in"]
                    .as_u64();
                
                let expires_at = expires_in.map(|seconds| {
                    chrono::Utc::now() + chrono::Duration::seconds(seconds as i64)
                });
                
                let token_type = token_response["token_type"]
                    .as_str()
                    .unwrap_or("Bearer");
                
                Ok(Credential {
                    auth_method: AuthMethod::OAuth2,
                    data: CredentialData::OAuth2 {
                        access_token: new_access_token.to_string(),
                        refresh_token: new_refresh_token,
                        token_type: token_type.to_string(),
                    },
                    expires_at,
                    scopes: credential.scopes.clone(),
                    metadata: credential.metadata.clone(),
                })
            } else {
                Err(ToolError::AuthenticationFailed(format!(
                    "Token refresh failed with status: {}", response.status()
                )))
            }
        } else {
            Err(ToolError::InvalidParameters("No refresh token available".to_string()))
        }
    }
    
    fn needs_refresh(&self, credential: &Credential) -> bool {
        if let Some(expires_at) = credential.expires_at {
            // Refresh if expires within 5 minutes
            let threshold = chrono::Utc::now() + chrono::Duration::minutes(5);
            expires_at < threshold
        } else {
            false
        }
    }
}

/// Authentication broker statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthBrokerStats {
    pub total_credentials: usize,
    pub auth_methods: HashMap<String, usize>,
    pub expired_credentials: usize,
    pub refresh_handlers: usize,
    pub configured_services: usize,
}