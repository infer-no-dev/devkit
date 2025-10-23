//! Secret vault implementation with encrypted storage

use crate::secrets::{SecretsError, VaultConfig, EncryptionConfig, EncryptionAlgorithm};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::fs;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Secret vault for secure storage
#[derive(Debug)]
pub struct SecretVault {
    config: VaultConfig,
    storage: Arc<RwLock<HashMap<String, SecretEntry>>>,
    encryption: EncryptionService,
}

/// A secret entry in the vault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretEntry {
    pub id: String,
    pub value: String,
    pub metadata: SecretMetadata,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub expires_at: Option<SystemTime>,
    pub access_count: u64,
    pub last_accessed: Option<SystemTime>,
}

/// Metadata associated with a secret
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub owner: String,
    pub secret_type: SecretType,
    pub rotation_policy: Option<RotationPolicy>,
    pub access_policy: Option<String>,
    pub custom_fields: HashMap<String, String>,
}

/// Types of secrets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecretType {
    ApiKey,
    Password,
    Certificate,
    PrivateKey,
    DatabaseConnection,
    OAuthToken,
    Custom(String),
}

/// Rotation policy for secrets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationPolicy {
    pub interval: Duration,
    pub auto_rotate: bool,
    pub notification_days: u32,
    pub rotation_script: Option<String>,
}

/// Encryption service for vault operations
#[derive(Debug)]
struct EncryptionService {
    config: EncryptionConfig,
    master_key: Vec<u8>,
}

/// Vault statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultStats {
    pub total_secrets: usize,
    pub expired_secrets: usize,
    pub secrets_by_type: HashMap<String, usize>,
    pub last_backup: Option<SystemTime>,
    pub storage_size_bytes: u64,
}

impl SecretVault {
    /// Create a new secret vault
    pub async fn new(config: VaultConfig) -> Result<Self, SecretsError> {
        // Ensure storage directory exists
        if let Some(parent) = config.storage_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let encryption = EncryptionService::new(Default::default())?;
        let storage = Arc::new(RwLock::new(HashMap::new()));

        let vault = Self {
            config,
            storage,
            encryption,
        };

        // Load existing secrets from disk
        vault.load_from_disk().await?;

        Ok(vault)
    }

    /// Store a secret in the vault
    pub async fn store_secret(
        &self,
        name: &str,
        value: &str,
        metadata: SecretMetadata,
    ) -> Result<(), SecretsError> {
        let mut storage = self.storage.write().await;

        // Check capacity
        if storage.len() >= self.config.max_secrets {
            return Err(SecretsError::ConfigError("Vault at maximum capacity".to_string()));
        }

        // Encrypt the secret value
        let encrypted_value = self.encryption.encrypt(value.as_bytes())?;

        let now = SystemTime::now();
        let expires_at = if let Some(ref rotation_policy) = metadata.rotation_policy {
            if rotation_policy.auto_rotate {
                Some(now + rotation_policy.interval)
            } else {
                None
            }
        } else {
            Some(now + self.config.default_ttl)
        };

        let entry = SecretEntry {
            id: Uuid::new_v4().to_string(),
            value: base64::encode(&encrypted_value),
            metadata,
            created_at: now,
            updated_at: now,
            expires_at,
            access_count: 0,
            last_accessed: None,
        };

        storage.insert(name.to_string(), entry);

        // Persist to disk
        self.save_to_disk().await?;

        Ok(())
    }

    /// Retrieve a secret from the vault
    pub async fn get_secret(&self, name: &str) -> Result<SecretEntry, SecretsError> {
        let mut storage = self.storage.write().await;

        match storage.get_mut(name) {
            Some(entry) => {
                // Check if secret has expired
                if let Some(expires_at) = entry.expires_at {
                    if SystemTime::now() > expires_at {
                        return Err(SecretsError::SecretNotFound(format!("{} (expired)", name)));
                    }
                }

                // Decrypt the value
                let encrypted_data = base64::decode(&entry.value)
                    .map_err(|e| SecretsError::EncryptionError(e.to_string()))?;
                let decrypted_value = self.encryption.decrypt(&encrypted_data)?;
                let value = String::from_utf8(decrypted_value)
                    .map_err(|e| SecretsError::EncryptionError(e.to_string()))?;

                // Update access tracking
                entry.access_count += 1;
                entry.last_accessed = Some(SystemTime::now());

                let mut result = entry.clone();
                result.value = value;

                // Persist access tracking updates
                self.save_to_disk().await?;

                Ok(result)
            }
            None => Err(SecretsError::SecretNotFound(name.to_string())),
        }
    }

    /// Update an existing secret
    pub async fn update_secret(
        &self,
        name: &str,
        new_value: &str,
        metadata: Option<SecretMetadata>,
    ) -> Result<(), SecretsError> {
        let mut storage = self.storage.write().await;

        match storage.get_mut(name) {
            Some(entry) => {
                // Encrypt new value
                let encrypted_value = self.encryption.encrypt(new_value.as_bytes())?;
                entry.value = base64::encode(&encrypted_value);
                entry.updated_at = SystemTime::now();

                if let Some(new_metadata) = metadata {
                    entry.metadata = new_metadata;
                }

                // Update expiration if rotation policy exists
                if let Some(ref rotation_policy) = entry.metadata.rotation_policy {
                    if rotation_policy.auto_rotate {
                        entry.expires_at = Some(SystemTime::now() + rotation_policy.interval);
                    }
                }

                self.save_to_disk().await?;
                Ok(())
            }
            None => Err(SecretsError::SecretNotFound(name.to_string())),
        }
    }

    /// Rotate a secret (generate new value)
    pub async fn rotate_secret(&self, name: &str, new_value: &str) -> Result<(), SecretsError> {
        self.update_secret(name, new_value, None).await
    }

    /// Delete a secret from the vault
    pub async fn delete_secret(&self, name: &str) -> Result<(), SecretsError> {
        let mut storage = self.storage.write().await;
        
        match storage.remove(name) {
            Some(_) => {
                self.save_to_disk().await?;
                Ok(())
            }
            None => Err(SecretsError::SecretNotFound(name.to_string())),
        }
    }

    /// List all secret names
    pub async fn list_secrets(&self) -> Result<Vec<String>, SecretsError> {
        let storage = self.storage.read().await;
        let now = SystemTime::now();

        let active_secrets = storage
            .iter()
            .filter_map(|(name, entry)| {
                // Filter out expired secrets
                if let Some(expires_at) = entry.expires_at {
                    if now > expires_at {
                        return None;
                    }
                }
                Some(name.clone())
            })
            .collect();

        Ok(active_secrets)
    }

    /// Get vault statistics
    pub async fn get_stats(&self) -> Result<VaultStats, SecretsError> {
        let storage = self.storage.read().await;
        let now = SystemTime::now();

        let mut expired_count = 0;
        let mut secrets_by_type = HashMap::new();

        for entry in storage.values() {
            // Count expired secrets
            if let Some(expires_at) = entry.expires_at {
                if now > expires_at {
                    expired_count += 1;
                }
            }

            // Count by type
            let type_name = match &entry.metadata.secret_type {
                SecretType::ApiKey => "api_key",
                SecretType::Password => "password",
                SecretType::Certificate => "certificate",
                SecretType::PrivateKey => "private_key",
                SecretType::DatabaseConnection => "database_connection",
                SecretType::OAuthToken => "oauth_token",
                SecretType::Custom(name) => name,
            };
            
            *secrets_by_type.entry(type_name.to_string()).or_insert(0) += 1;
        }

        // Calculate storage size (approximate)
        let storage_size = storage
            .values()
            .map(|entry| entry.value.len() as u64)
            .sum();

        Ok(VaultStats {
            total_secrets: storage.len(),
            expired_secrets: expired_count,
            secrets_by_type,
            last_backup: self.get_last_backup_time().await?,
            storage_size_bytes: storage_size,
        })
    }

    /// Clean up expired secrets
    pub async fn cleanup_expired(&self) -> Result<usize, SecretsError> {
        let mut storage = self.storage.write().await;
        let now = SystemTime::now();
        let mut removed_count = 0;

        let expired_keys: Vec<String> = storage
            .iter()
            .filter_map(|(name, entry)| {
                if let Some(expires_at) = entry.expires_at {
                    if now > expires_at {
                        return Some(name.clone());
                    }
                }
                None
            })
            .collect();

        for key in expired_keys {
            storage.remove(&key);
            removed_count += 1;
        }

        if removed_count > 0 {
            self.save_to_disk().await?;
        }

        Ok(removed_count)
    }

    /// Backup vault to a file
    pub async fn backup(&self, backup_path: &Path) -> Result<(), SecretsError> {
        let storage = self.storage.read().await;
        let backup_data = serde_json::to_string_pretty(&*storage)?;
        
        // Encrypt backup data
        let encrypted_backup = self.encryption.encrypt(backup_data.as_bytes())?;
        
        fs::write(backup_path, &encrypted_backup).await?;
        Ok(())
    }

    /// Restore vault from a backup file
    pub async fn restore(&self, backup_path: &Path) -> Result<(), SecretsError> {
        let encrypted_data = fs::read(backup_path).await?;
        let decrypted_data = self.encryption.decrypt(&encrypted_data)?;
        let backup_json = String::from_utf8(decrypted_data)
            .map_err(|e| SecretsError::EncryptionError(e.to_string()))?;
            
        let backup_storage: HashMap<String, SecretEntry> = serde_json::from_str(&backup_json)?;
        
        let mut storage = self.storage.write().await;
        *storage = backup_storage;
        
        self.save_to_disk().await?;
        Ok(())
    }

    /// Load secrets from persistent storage
    async fn load_from_disk(&self) -> Result<(), SecretsError> {
        if !self.config.storage_path.exists() {
            return Ok(());
        }

        match fs::read(&self.config.storage_path).await {
            Ok(encrypted_data) => {
                let decrypted_data = self.encryption.decrypt(&encrypted_data)?;
                let json_data = String::from_utf8(decrypted_data)
                    .map_err(|e| SecretsError::EncryptionError(e.to_string()))?;
                
                let stored_secrets: HashMap<String, SecretEntry> = serde_json::from_str(&json_data)
                    .unwrap_or_default();
                
                let mut storage = self.storage.write().await;
                *storage = stored_secrets;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // File doesn't exist yet, that's fine
            }
            Err(e) => return Err(SecretsError::IoError(e)),
        }

        Ok(())
    }

    /// Save secrets to persistent storage
    async fn save_to_disk(&self) -> Result<(), SecretsError> {
        let storage = self.storage.read().await;
        let json_data = serde_json::to_string(&*storage)?;
        
        let encrypted_data = self.encryption.encrypt(json_data.as_bytes())?;
        
        fs::write(&self.config.storage_path, &encrypted_data).await?;
        Ok(())
    }

    /// Get the last backup time
    async fn get_last_backup_time(&self) -> Result<Option<SystemTime>, SecretsError> {
        // In a real implementation, this would track backup metadata
        Ok(None)
    }
}

impl EncryptionService {
    /// Create a new encryption service
    fn new(config: EncryptionConfig) -> Result<Self, SecretsError> {
        // Generate a master key (in production, this would come from a secure source)
        let master_key = Self::derive_master_key(&config)?;
        
        Ok(Self {
            config,
            master_key,
        })
    }

    /// Encrypt data
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecretsError> {
        match self.config.algorithm {
            EncryptionAlgorithm::AES256GCM => self.encrypt_aes_gcm(data),
            EncryptionAlgorithm::ChaCha20Poly1305 => self.encrypt_chacha20_poly1305(data),
            EncryptionAlgorithm::AES256CBC => self.encrypt_aes_cbc(data),
        }
    }

    /// Decrypt data
    fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, SecretsError> {
        match self.config.algorithm {
            EncryptionAlgorithm::AES256GCM => self.decrypt_aes_gcm(encrypted_data),
            EncryptionAlgorithm::ChaCha20Poly1305 => self.decrypt_chacha20_poly1305(encrypted_data),
            EncryptionAlgorithm::AES256CBC => self.decrypt_aes_cbc(encrypted_data),
        }
    }

    /// Derive master key from configuration
    fn derive_master_key(config: &EncryptionConfig) -> Result<Vec<u8>, SecretsError> {
        // In production, this would use proper key derivation functions
        // For now, we'll generate a simple key
        use rand::RngCore;
        let mut key = vec![0u8; 32]; // 256-bit key
        rand::thread_rng().fill_bytes(&mut key);
        Ok(key)
    }

    /// Encrypt using AES-256-GCM
    fn encrypt_aes_gcm(&self, data: &[u8]) -> Result<Vec<u8>, SecretsError> {
        // Placeholder implementation - in production use a proper crypto library
        let mut result = self.master_key.clone();
        result.extend_from_slice(data);
        Ok(result)
    }

    /// Decrypt using AES-256-GCM
    fn decrypt_aes_gcm(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, SecretsError> {
        // Placeholder implementation
        if encrypted_data.len() < self.master_key.len() {
            return Err(SecretsError::EncryptionError("Invalid encrypted data".to_string()));
        }
        Ok(encrypted_data[self.master_key.len()..].to_vec())
    }

    /// Encrypt using ChaCha20-Poly1305
    fn encrypt_chacha20_poly1305(&self, data: &[u8]) -> Result<Vec<u8>, SecretsError> {
        // Placeholder - would use proper ChaCha20-Poly1305 implementation
        self.encrypt_aes_gcm(data)
    }

    /// Decrypt using ChaCha20-Poly1305
    fn decrypt_chacha20_poly1305(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, SecretsError> {
        // Placeholder
        self.decrypt_aes_gcm(encrypted_data)
    }

    /// Encrypt using AES-256-CBC
    fn encrypt_aes_cbc(&self, data: &[u8]) -> Result<Vec<u8>, SecretsError> {
        // Placeholder - would use proper AES-CBC implementation
        self.encrypt_aes_gcm(data)
    }

    /// Decrypt using AES-256-CBC
    fn decrypt_aes_cbc(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, SecretsError> {
        // Placeholder
        self.decrypt_aes_gcm(encrypted_data)
    }
}

impl Default for SecretMetadata {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: None,
            tags: Vec::new(),
            owner: "system".to_string(),
            secret_type: SecretType::Custom("generic".to_string()),
            rotation_policy: None,
            access_policy: None,
            custom_fields: HashMap::new(),
        }
    }
}

impl std::fmt::Display for SecretType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecretType::ApiKey => write!(f, "API Key"),
            SecretType::Password => write!(f, "Password"),
            SecretType::Certificate => write!(f, "Certificate"),
            SecretType::PrivateKey => write!(f, "Private Key"),
            SecretType::DatabaseConnection => write!(f, "Database Connection"),
            SecretType::OAuthToken => write!(f, "OAuth Token"),
            SecretType::Custom(name) => write!(f, "{}", name),
        }
    }
}