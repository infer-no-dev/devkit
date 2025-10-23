//! Audit logging system for secrets management

use crate::secrets::{SecretsError, AuditConfig, AuditEventType, AuditFilters};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Audit logger for security events
#[derive(Debug)]
pub struct AuditLogger {
    config: AuditConfig,
    events: Arc<RwLock<Vec<AuditEvent>>>,
}

/// Audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: AuditEventType,
    pub level: AuditLevel,
    pub actor: String,
    pub resource: String,
    pub action: String,
    pub outcome: AuditOutcome,
    pub details: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub session_id: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Audit levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditLevel {
    Info,
    Warning,
    Error,
    Critical,
}

/// Audit outcomes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditOutcome {
    Success,
    Failure,
    Denied,
    Error,
}

impl AuditLogger {
    /// Create a new audit logger
    pub async fn new(config: AuditConfig) -> Result<Self, SecretsError> {
        let logger = Self {
            config,
            events: Arc::new(RwLock::new(Vec::new())),
        };

        // Ensure log directory exists
        if let Some(parent) = logger.config.log_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        Ok(logger)
    }

    /// Log an audit event
    pub async fn log_event(&self, event: AuditEvent) {
        if !self.config.enabled {
            return;
        }

        // Check if this event type should be audited
        if !self.config.events_to_audit.contains(&event.event_type) {
            return;
        }

        // Store in memory
        {
            let mut events = self.events.write().await;
            events.push(event.clone());
        }

        // Write to file
        if let Err(e) = self.write_to_file(&event).await {
            eprintln!("Failed to write audit log: {}", e);
        }

        // Send to syslog if configured
        if self.config.syslog {
            self.send_to_syslog(&event).await;
        }

        // Send to external audit system if configured
        if let Some(ref external_config) = self.config.external_audit {
            self.send_to_external(&event, external_config).await;
        }
    }

    /// Get audit events with filters
    pub async fn get_events(&self, filters: AuditFilters) -> Result<Vec<AuditEvent>, SecretsError> {
        let events = self.events.read().await;
        let mut filtered_events: Vec<AuditEvent> = events
            .iter()
            .filter(|event| {
                // Filter by requester
                if let Some(ref requester) = filters.requester {
                    if event.actor != *requester {
                        return false;
                    }
                }

                // Filter by secret name
                if let Some(ref secret_name) = filters.secret_name {
                    if event.resource != *secret_name {
                        return false;
                    }
                }

                // Filter by event type
                if let Some(ref event_type) = filters.event_type {
                    if event.event_type != *event_type {
                        return false;
                    }
                }

                // Filter by time range
                if let Some(start_time) = filters.start_time {
                    if event.timestamp < start_time {
                        return false;
                    }
                }

                if let Some(end_time) = filters.end_time {
                    if event.timestamp > end_time {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // Sort by timestamp (newest first)
        filtered_events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply limit
        if let Some(limit) = filters.limit {
            filtered_events.truncate(limit);
        }

        Ok(filtered_events)
    }

    /// Flush pending audit events
    pub async fn flush(&self) -> Result<(), SecretsError> {
        // In a real implementation, this would flush any pending writes
        Ok(())
    }

    /// Write audit event to file
    async fn write_to_file(&self, event: &AuditEvent) -> Result<(), SecretsError> {
        let log_entry = serde_json::to_string(event)?;
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.config.log_path)
            .await?;

        file.write_all(log_entry.as_bytes()).await?;
        file.write_all(b"\n").await?;
        file.flush().await?;

        Ok(())
    }

    /// Send to syslog
    async fn send_to_syslog(&self, event: &AuditEvent) {
        // In a real implementation, this would send to syslog
        tracing::info!("AUDIT: {}", serde_json::to_string(event).unwrap_or_default());
    }

    /// Send to external audit system
    async fn send_to_external(&self, _event: &AuditEvent, _config: &crate::secrets::ExternalAuditConfig) {
        // In a real implementation, this would send to external systems
        // like Splunk, ELK stack, etc.
    }
}

impl AuditEvent {
    /// Create a secret access event
    pub fn secret_accessed(actor: &str, resource: &str, details: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type: AuditEventType::SecretAccess,
            level: AuditLevel::Info,
            actor: actor.to_string(),
            resource: resource.to_string(),
            action: "access".to_string(),
            outcome: AuditOutcome::Success,
            details: details.to_string(),
            ip_address: None,
            user_agent: None,
            session_id: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create a secret creation event
    pub fn secret_created(actor: &str, resource: &str, details: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type: AuditEventType::SecretCreation,
            level: AuditLevel::Info,
            actor: actor.to_string(),
            resource: resource.to_string(),
            action: "create".to_string(),
            outcome: AuditOutcome::Success,
            details: details.to_string(),
            ip_address: None,
            user_agent: None,
            session_id: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create a secret modification event
    pub fn secret_modified(actor: &str, resource: &str, details: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type: AuditEventType::SecretModification,
            level: AuditLevel::Info,
            actor: actor.to_string(),
            resource: resource.to_string(),
            action: "modify".to_string(),
            outcome: AuditOutcome::Success,
            details: details.to_string(),
            ip_address: None,
            user_agent: None,
            session_id: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create a secret deletion event
    pub fn secret_deleted(actor: &str, resource: &str, details: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type: AuditEventType::SecretDeletion,
            level: AuditLevel::Warning,
            actor: actor.to_string(),
            resource: resource.to_string(),
            action: "delete".to_string(),
            outcome: AuditOutcome::Success,
            details: details.to_string(),
            ip_address: None,
            user_agent: None,
            session_id: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create a policy violation event
    pub fn policy_violation(actor: &str, details: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type: AuditEventType::PolicyViolation,
            level: AuditLevel::Warning,
            actor: actor.to_string(),
            resource: "policy".to_string(),
            action: "violation".to_string(),
            outcome: AuditOutcome::Denied,
            details,
            ip_address: None,
            user_agent: None,
            session_id: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create an unauthorized access event
    pub fn unauthorized_access(actor: &str, resource: &str, details: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type: AuditEventType::UnauthorizedAccess,
            level: AuditLevel::Error,
            actor: actor.to_string(),
            resource: resource.to_string(),
            action: "access_denied".to_string(),
            outcome: AuditOutcome::Denied,
            details: details.to_string(),
            ip_address: None,
            user_agent: None,
            session_id: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create a secret access failure event
    pub fn secret_access_failure(actor: &str, resource: &str, details: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type: AuditEventType::SecretAccess,
            level: AuditLevel::Warning,
            actor: actor.to_string(),
            resource: resource.to_string(),
            action: "access_failed".to_string(),
            outcome: AuditOutcome::Failure,
            details: details.to_string(),
            ip_address: None,
            user_agent: None,
            session_id: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create a security scan event
    pub fn security_scan(actor: &str, resource: &str, details: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type: AuditEventType::SecretAccess, // Using closest available type
            level: AuditLevel::Info,
            actor: actor.to_string(),
            resource: resource.to_string(),
            action: "security_scan".to_string(),
            outcome: AuditOutcome::Success,
            details: details.to_string(),
            ip_address: None,
            user_agent: None,
            session_id: None,
            metadata: std::collections::HashMap::new(),
        }
    }
}