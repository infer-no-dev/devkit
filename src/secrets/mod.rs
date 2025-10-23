//! Enterprise secrets management with secure credential handling
//!
//! This module provides secure credential handling with scoped access, secret scanning,
//! audit trails, and integration with enterprise secret stores and policy enforcement.

pub mod vault;
pub mod scanner;
pub mod audit;
pub mod providers;
pub mod policies;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

pub use vault::{SecretVault, SecretEntry, SecretMetadata};
pub use scanner::{SecretScanner, ScanResult, ScanRule};
pub use audit::{AuditLogger, AuditEvent, AuditLevel};
pub use providers::{SecretProvider, ProviderType, ProviderConfig};
pub use policies::{PolicyEngine, AccessPolicy, PolicyDecision};

/// Secrets management system
pub struct SecretsManager {
    config: SecretsConfig,
    vault: Arc<SecretVault>,
    scanner: Arc<SecretScanner>,
    audit_logger: Arc<AuditLogger>,
    policy_engine: Arc<PolicyEngine>,
    providers: RwLock<HashMap<String, Box<dyn SecretProvider + Send + Sync>>>,
}

/// Configuration for secrets management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsConfig {
    /// Default vault configuration
    pub vault: VaultConfig,
    /// Secret scanning settings
    pub scanning: ScanConfig,
    /// Audit configuration
    pub audit: AuditConfig,
    /// Policy enforcement settings
    pub policies: PolicyConfig,
    /// Provider configurations
    pub providers: HashMap<String, ProviderConfig>,
    /// Encryption settings
    pub encryption: EncryptionConfig,
}

/// Vault configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    /// Local vault storage path
    pub storage_path: PathBuf,
    /// Whether to use external vault providers
    pub use_external_providers: bool,
    /// Default TTL for secrets
    pub default_ttl: Duration,
    /// Maximum number of secrets to store
    pub max_secrets: usize,
    /// Whether to enable automatic rotation
    pub auto_rotate: bool,
    /// Rotation interval
    pub rotation_interval: Duration,
}

/// Secret scanning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    /// Whether scanning is enabled
    pub enabled: bool,
    /// File patterns to scan
    pub file_patterns: Vec<String>,
    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,
    /// Maximum file size to scan (bytes)
    pub max_file_size: usize,
    /// Whether to scan recursively
    pub recursive: bool,
    /// Custom scan rules
    pub custom_rules: Vec<CustomScanRule>,
    /// Whether to auto-remediate found secrets
    pub auto_remediate: bool,
}

/// Custom scan rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomScanRule {
    pub name: String,
    pub pattern: String,
    pub description: String,
    pub severity: SeverityLevel,
    pub confidence: f64,
}

/// Severity levels for security issues
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeverityLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Audit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Whether auditing is enabled
    pub enabled: bool,
    /// Audit log file path
    pub log_path: PathBuf,
    /// Log rotation settings
    pub rotation: LogRotationConfig,
    /// Whether to log to syslog
    pub syslog: bool,
    /// Whether to send to external audit system
    pub external_audit: Option<ExternalAuditConfig>,
    /// Events to audit
    pub events_to_audit: Vec<AuditEventType>,
}

/// Log rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationConfig {
    pub max_size_mb: usize,
    pub max_files: usize,
    pub compress: bool,
}

/// External audit system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalAuditConfig {
    pub endpoint: String,
    pub auth_token_name: String,
    pub batch_size: usize,
    pub flush_interval: Duration,
}

/// Types of audit events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventType {
    SecretAccess,
    SecretCreation,
    SecretModification,
    SecretDeletion,
    PolicyViolation,
    AuthenticationFailure,
    UnauthorizedAccess,
}

/// Policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    /// Whether policy enforcement is enabled
    pub enabled: bool,
    /// Policy definitions
    pub policies: Vec<PolicyDefinition>,
    /// Default action for undefined scenarios
    pub default_action: PolicyAction,
    /// Whether to enforce policies in dry-run mode
    pub dry_run: bool,
}

/// Policy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDefinition {
    pub name: String,
    pub description: String,
    pub conditions: Vec<PolicyCondition>,
    pub action: PolicyAction,
    pub priority: u32,
}

/// Policy condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyCondition {
    UserGroup(String),
    TimeOfDay { start: String, end: String },
    IPAddress(String),
    SecretType(String),
    Application(String),
    Environment(String),
}

/// Policy action
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyAction {
    Allow,
    Deny,
    RequireApproval,
    RequireMFA,
    Audit,
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Encryption algorithm to use
    pub algorithm: EncryptionAlgorithm,
    /// Key derivation settings
    pub key_derivation: KeyDerivationConfig,
    /// Whether to use hardware security modules
    pub use_hsm: bool,
    /// HSM configuration
    pub hsm_config: Option<HsmConfig>,
}

/// Supported encryption algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    AES256GCM,
    ChaCha20Poly1305,
    AES256CBC,
}

/// Key derivation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationConfig {
    pub algorithm: String,
    pub iterations: u32,
    pub salt_length: usize,
}

/// HSM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsmConfig {
    pub provider: String,
    pub slot: u32,
    pub pin_env_var: String,
}

/// Secret access request
#[derive(Debug, Clone)]
pub struct SecretRequest {
    pub secret_name: String,
    pub requester: String,
    pub application: Option<String>,
    pub justification: Option<String>,
    pub ttl: Option<Duration>,
    pub scope: Vec<String>,
}

/// Result of secret access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretResponse {
    pub secret_value: Option<String>,
    pub metadata: SecretMetadata,
    pub access_token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub restrictions: Vec<String>,
}

/// Secret rotation request
#[derive(Debug, Clone)]
pub struct RotationRequest {
    pub secret_name: String,
    pub new_value: Option<String>,
    pub rotation_reason: RotationReason,
    pub notify_applications: Vec<String>,
}

/// Reason for secret rotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RotationReason {
    Scheduled,
    Compromised,
    PolicyRequired,
    Manual,
}

/// Secrets management errors
#[derive(Debug, thiserror::Error)]
pub enum SecretsError {
    #[error("Secret not found: {0}")]
    SecretNotFound(String),
    
    #[error("Access denied: {0}")]
    AccessDenied(String),
    
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    
    #[error("Provider error: {0}")]
    ProviderError(String),
    
    #[error("Policy violation: {0}")]
    PolicyViolation(String),
    
    #[error("Audit error: {0}")]
    AuditError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl SecretsManager {
    /// Create a new secrets manager
    pub async fn new(config: SecretsConfig) -> Result<Self, SecretsError> {
        let vault = Arc::new(SecretVault::new(config.vault.clone()).await?);
        let scanner = Arc::new(SecretScanner::new(config.scanning.clone()));
        let audit_logger = Arc::new(AuditLogger::new(config.audit.clone()).await?);
        let policy_engine = Arc::new(PolicyEngine::new(config.policies.clone()));
        
        let mut manager = Self {
            config: config.clone(),
            vault,
            scanner,
            audit_logger,
            policy_engine,
            providers: RwLock::new(HashMap::new()),
        };
        
        // Initialize providers
        manager.initialize_providers().await?;
        
        Ok(manager)
    }
    
    /// Initialize secret providers
    async fn initialize_providers(&mut self) -> Result<(), SecretsError> {
        let mut providers = self.providers.write().await;
        
        for (name, provider_config) in &self.config.providers {
            match providers::create_provider(provider_config.clone()) {
                Ok(provider) => {
                    providers.insert(name.clone(), provider);
                }
                Err(e) => {
                    tracing::warn!("Failed to initialize provider {}: {}", name, e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Store a secret securely
    pub async fn store_secret(
        &self,
        name: &str,
        value: &str,
        metadata: SecretMetadata,
        requester: &str,
    ) -> Result<(), SecretsError> {
        // Check policy
        let policy_decision = self.policy_engine.evaluate_store_request(name, requester).await;
        if policy_decision.action != PolicyAction::Allow {
            self.audit_logger.log_event(AuditEvent::policy_violation(
                requester,
                format!("Secret store denied for {}: {:?}", name, policy_decision),
            )).await;
            return Err(SecretsError::PolicyViolation(policy_decision.reason));
        }
        
        // Store in vault
        self.vault.store_secret(name, value, metadata).await?;
        
        // Audit the operation
        self.audit_logger.log_event(AuditEvent::secret_created(
            requester,
            name,
            "Secret stored successfully",
        )).await;
        
        Ok(())
    }
    
    /// Retrieve a secret
    pub async fn get_secret(&self, request: SecretRequest) -> Result<SecretResponse, SecretsError> {
        // Check policy
        let policy_decision = self.policy_engine.evaluate_access_request(&request).await;
        match policy_decision.action {
            PolicyAction::Deny => {
                self.audit_logger.log_event(AuditEvent::unauthorized_access(
                    &request.requester,
                    &request.secret_name,
                    &policy_decision.reason,
                )).await;
                return Err(SecretsError::AccessDenied(policy_decision.reason));
            }
            PolicyAction::RequireApproval | PolicyAction::RequireMFA => {
                // In a real implementation, this would trigger approval workflows
                return Err(SecretsError::AccessDenied("Additional approval required".to_string()));
            }
            PolicyAction::Allow | PolicyAction::Audit => {
                // Continue with access
            }
        }
        
        // Try to retrieve from vault
        match self.vault.get_secret(&request.secret_name).await {
            Ok(secret_entry) => {
                // Generate access token
                let access_token = self.generate_access_token(&request).await?;
                
                // Calculate expiration
                let ttl = request.ttl.unwrap_or(self.config.vault.default_ttl);
                let expires_at = chrono::Utc::now() + chrono::Duration::from_std(ttl).unwrap();
                
                // Audit the access
                self.audit_logger.log_event(AuditEvent::secret_accessed(
                    &request.requester,
                    &request.secret_name,
                    "Secret retrieved successfully",
                )).await;
                
                Ok(SecretResponse {
                    secret_value: Some(secret_entry.value),
                    metadata: secret_entry.metadata,
                    access_token,
                    expires_at,
                    restrictions: policy_decision.restrictions,
                })
            }
            Err(_) => {
                // Try external providers
                self.try_external_providers(&request).await
            }
        }
    }
    
    /// Try to retrieve secret from external providers
    async fn try_external_providers(&self, request: &SecretRequest) -> Result<SecretResponse, SecretsError> {
        let providers = self.providers.read().await;
        
        for (provider_name, provider) in providers.iter() {
            match provider.get_secret(&request.secret_name).await {
                Ok(value) => {
                    // Found in external provider
                    let access_token = self.generate_access_token(request).await?;
                    let ttl = request.ttl.unwrap_or(self.config.vault.default_ttl);
                    let expires_at = chrono::Utc::now() + chrono::Duration::from_std(ttl).unwrap();
                    
                    // Audit the external access
                    self.audit_logger.log_event(AuditEvent::secret_accessed(
                        &request.requester,
                        &request.secret_name,
                        &format!("Secret retrieved from provider: {}", provider_name),
                    )).await;
                    
                    return Ok(SecretResponse {
                        secret_value: Some(value),
                        metadata: SecretMetadata::default(),
                        access_token,
                        expires_at,
                        restrictions: vec![],
                    });
                }
                Err(_) => continue,
            }
        }
        
        // Secret not found anywhere
        self.audit_logger.log_event(AuditEvent::secret_access_failure(
            &request.requester,
            &request.secret_name,
            "Secret not found in any provider",
        )).await;
        
        Err(SecretsError::SecretNotFound(request.secret_name.clone()))
    }
    
    /// Scan files for potential secrets
    pub async fn scan_for_secrets(&self, path: &Path) -> Result<Vec<ScanResult>, SecretsError> {
        let results = self.scanner.scan_path(path).await?;
        
        // Audit the scan
        self.audit_logger.log_event(AuditEvent::security_scan(
            "system",
            path.to_string_lossy().as_ref(),
            &format!("Found {} potential secrets", results.len()),
        )).await;
        
        // Auto-remediate if configured
        if self.config.scanning.auto_remediate {
            for result in &results {
                if result.severity == SeverityLevel::Critical || result.severity == SeverityLevel::High {
                    self.remediate_secret_exposure(result).await?;
                }
            }
        }
        
        Ok(results)
    }
    
    /// Rotate a secret
    pub async fn rotate_secret(&self, request: RotationRequest) -> Result<(), SecretsError> {
        // Generate new value if not provided
        let new_value = if let Some(value) = request.new_value {
            value
        } else {
            self.generate_secret_value(&request.secret_name).await?
        };
        
        // Update in vault
        self.vault.rotate_secret(&request.secret_name, &new_value).await?;
        
        // Notify applications if specified
        for app in &request.notify_applications {
            self.notify_application_of_rotation(app, &request.secret_name, &new_value).await?;
        }
        
        // Audit the rotation
        self.audit_logger.log_event(AuditEvent::secret_modified(
            "system",
            &request.secret_name,
            &format!("Secret rotated: {:?}", request.rotation_reason),
        )).await;
        
        Ok(())
    }
    
    /// Delete a secret
    pub async fn delete_secret(&self, name: &str, requester: &str) -> Result<(), SecretsError> {
        // Check policy
        let policy_decision = self.policy_engine.evaluate_delete_request(name, requester).await;
        if policy_decision.action != PolicyAction::Allow {
            return Err(SecretsError::PolicyViolation(policy_decision.reason));
        }
        
        // Delete from vault
        self.vault.delete_secret(name).await?;
        
        // Audit the deletion
        self.audit_logger.log_event(AuditEvent::secret_deleted(
            requester,
            name,
            "Secret deleted successfully",
        )).await;
        
        Ok(())
    }
    
    /// List accessible secrets for a user
    pub async fn list_secrets(&self, requester: &str) -> Result<Vec<String>, SecretsError> {
        let all_secrets = self.vault.list_secrets().await?;
        
        // Filter based on access policies
        let mut accessible_secrets = Vec::new();
        for secret_name in all_secrets {
            let request = SecretRequest {
                secret_name: secret_name.clone(),
                requester: requester.to_string(),
                application: None,
                justification: None,
                ttl: None,
                scope: vec![],
            };
            
            let policy_decision = self.policy_engine.evaluate_access_request(&request).await;
            if policy_decision.action == PolicyAction::Allow {
                accessible_secrets.push(secret_name);
            }
        }
        
        Ok(accessible_secrets)
    }
    
    /// Get audit events
    pub async fn get_audit_events(&self, filters: AuditFilters) -> Result<Vec<AuditEvent>, SecretsError> {
        self.audit_logger.get_events(filters).await
    }
    
    /// Generate access token for secret access
    async fn generate_access_token(&self, request: &SecretRequest) -> Result<String, SecretsError> {
        // In a real implementation, this would generate a JWT or similar token
        let token_data = format!("{}:{}:{}", 
            request.requester, 
            request.secret_name, 
            chrono::Utc::now().timestamp()
        );
        
        // Simple base64 encoding for demonstration
        Ok(base64::encode(token_data))
    }
    
    /// Generate new secret value
    async fn generate_secret_value(&self, _secret_name: &str) -> Result<String, SecretsError> {
        // Generate cryptographically secure random value
        use rand::{Rng, distributions::Alphanumeric};
        let random_string: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
            
        Ok(random_string)
    }
    
    /// Remediate secret exposure
    async fn remediate_secret_exposure(&self, result: &ScanResult) -> Result<(), SecretsError> {
        // Log the exposure
        tracing::warn!("Remediating secret exposure: {} in {}", result.rule_name, result.file_path);
        
        // In a real implementation, this might:
        // - Rotate the exposed secret
        // - Notify security team
        // - Update the file to remove the secret
        // - Create incident ticket
        
        Ok(())
    }
    
    /// Notify application of secret rotation
    async fn notify_application_of_rotation(
        &self, 
        _app: &str, 
        _secret_name: &str, 
        _new_value: &str
    ) -> Result<(), SecretsError> {
        // In a real implementation, this would send notifications to applications
        // via webhooks, message queues, or other mechanisms
        Ok(())
    }
    
    /// Shutdown the secrets manager
    pub async fn shutdown(&self) -> Result<(), SecretsError> {
        self.audit_logger.flush().await?;
        Ok(())
    }
}

/// Filters for audit events
#[derive(Debug, Clone, Default)]
pub struct AuditFilters {
    pub requester: Option<String>,
    pub secret_name: Option<String>,
    pub event_type: Option<AuditEventType>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<usize>,
}

impl Default for SecretsConfig {
    fn default() -> Self {
        Self {
            vault: VaultConfig::default(),
            scanning: ScanConfig::default(),
            audit: AuditConfig::default(),
            policies: PolicyConfig::default(),
            providers: HashMap::new(),
            encryption: EncryptionConfig::default(),
        }
    }
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            storage_path: PathBuf::from(".devkit/secrets"),
            use_external_providers: false,
            default_ttl: Duration::from_secs(3600), // 1 hour
            max_secrets: 1000,
            auto_rotate: false,
            rotation_interval: Duration::from_secs(7 * 24 * 3600), // 7 days
        }
    }
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            file_patterns: vec![
                "*.rs".to_string(),
                "*.py".to_string(),
                "*.js".to_string(),
                "*.ts".to_string(),
                "*.go".to_string(),
                "*.java".to_string(),
                "*.yaml".to_string(),
                "*.yml".to_string(),
                "*.json".to_string(),
                "*.toml".to_string(),
                "*.env".to_string(),
                ".env*".to_string(),
            ],
            exclude_patterns: vec![
                "node_modules/**".to_string(),
                "target/**".to_string(),
                ".git/**".to_string(),
                "*.lock".to_string(),
            ],
            max_file_size: 10 * 1024 * 1024, // 10MB
            recursive: true,
            custom_rules: vec![],
            auto_remediate: false,
        }
    }
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_path: PathBuf::from(".devkit/audit.log"),
            rotation: LogRotationConfig {
                max_size_mb: 100,
                max_files: 10,
                compress: true,
            },
            syslog: false,
            external_audit: None,
            events_to_audit: vec![
                AuditEventType::SecretAccess,
                AuditEventType::SecretCreation,
                AuditEventType::SecretModification,
                AuditEventType::SecretDeletion,
                AuditEventType::PolicyViolation,
                AuditEventType::UnauthorizedAccess,
            ],
        }
    }
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            policies: vec![
                PolicyDefinition {
                    name: "default_allow".to_string(),
                    description: "Default allow policy for authenticated users".to_string(),
                    conditions: vec![],
                    action: PolicyAction::Allow,
                    priority: 0,
                }
            ],
            default_action: PolicyAction::Deny,
            dry_run: false,
        }
    }
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            algorithm: EncryptionAlgorithm::AES256GCM,
            key_derivation: KeyDerivationConfig {
                algorithm: "PBKDF2".to_string(),
                iterations: 100_000,
                salt_length: 32,
            },
            use_hsm: false,
            hsm_config: None,
        }
    }
}

// Re-export for convenience
pub use base64;