//! Policy engine for secrets access control

use crate::secrets::{SecretRequest, PolicyConfig, PolicyDefinition, PolicyCondition, PolicyAction};
use serde::{Deserialize, Serialize};

/// Policy engine for evaluating access requests
#[derive(Debug)]
pub struct PolicyEngine {
    config: PolicyConfig,
}

/// Access policy result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPolicy {
    pub name: String,
    pub allowed: bool,
    pub conditions: Vec<String>,
    pub restrictions: Vec<String>,
}

/// Policy decision result
#[derive(Debug, Clone)]
pub struct PolicyDecision {
    pub action: PolicyAction,
    pub reason: String,
    pub matched_policies: Vec<String>,
    pub restrictions: Vec<String>,
}

impl PolicyEngine {
    /// Create a new policy engine
    pub fn new(config: PolicyConfig) -> Self {
        Self { config }
    }

    /// Evaluate access request against policies
    pub async fn evaluate_access_request(&self, request: &SecretRequest) -> PolicyDecision {
        if !self.config.enabled {
            return PolicyDecision {
                action: PolicyAction::Allow,
                reason: "Policy enforcement disabled".to_string(),
                matched_policies: vec![],
                restrictions: vec![],
            };
        }

        let mut matched_policies = Vec::new();
        let mut restrictions = Vec::new();
        let mut final_action = self.config.default_action.clone();
        let mut reason = "No matching policies found".to_string();

        // Sort policies by priority (higher priority first)
        let mut sorted_policies = self.config.policies.clone();
        sorted_policies.sort_by(|a, b| b.priority.cmp(&a.priority));

        for policy in &sorted_policies {
            if self.policy_matches_request(policy, request) {
                matched_policies.push(policy.name.clone());
                
                // Update action based on policy priority
                match policy.action {
                    PolicyAction::Deny => {
                        final_action = PolicyAction::Deny;
                        reason = format!("Denied by policy: {}", policy.name);
                        break; // Deny takes precedence
                    }
                    PolicyAction::RequireApproval | PolicyAction::RequireMFA => {
                        if final_action == PolicyAction::Allow {
                            final_action = policy.action.clone();
                            reason = format!("Additional verification required by policy: {}", policy.name);
                        }
                    }
                    PolicyAction::Allow => {
                        if final_action == self.config.default_action {
                            final_action = PolicyAction::Allow;
                            reason = format!("Allowed by policy: {}", policy.name);
                        }
                    }
                    PolicyAction::Audit => {
                        // Audit doesn't change the action, just adds restriction
                        restrictions.push("audit_required".to_string());
                    }
                }
            }
        }

        // Apply dry-run mode
        if self.config.dry_run && final_action != PolicyAction::Allow {
            restrictions.push("dry_run_mode".to_string());
            reason = format!("{} (dry-run mode)", reason);
        }

        PolicyDecision {
            action: final_action,
            reason,
            matched_policies,
            restrictions,
        }
    }

    /// Evaluate store request
    pub async fn evaluate_store_request(&self, secret_name: &str, requester: &str) -> PolicyDecision {
        // Create a mock request for policy evaluation
        let request = SecretRequest {
            secret_name: secret_name.to_string(),
            requester: requester.to_string(),
            application: None,
            justification: Some("Store secret".to_string()),
            ttl: None,
            scope: vec![],
        };

        self.evaluate_access_request(&request).await
    }

    /// Evaluate delete request
    pub async fn evaluate_delete_request(&self, secret_name: &str, requester: &str) -> PolicyDecision {
        // Create a mock request for policy evaluation
        let request = SecretRequest {
            secret_name: secret_name.to_string(),
            requester: requester.to_string(),
            application: None,
            justification: Some("Delete secret".to_string()),
            ttl: None,
            scope: vec![],
        };

        // Deletion typically requires higher privileges
        let mut decision = self.evaluate_access_request(&request).await;
        if decision.action == PolicyAction::Allow {
            // Add additional checks for deletion
            if !self.can_delete_secret(secret_name, requester) {
                decision.action = PolicyAction::Deny;
                decision.reason = "Insufficient privileges for deletion".to_string();
            }
        }

        decision
    }

    /// Check if a policy matches a request
    fn policy_matches_request(&self, policy: &PolicyDefinition, request: &SecretRequest) -> bool {
        for condition in &policy.conditions {
            if !self.evaluate_condition(condition, request) {
                return false;
            }
        }
        true
    }

    /// Evaluate a single policy condition
    fn evaluate_condition(&self, condition: &PolicyCondition, request: &SecretRequest) -> bool {
        match condition {
            PolicyCondition::UserGroup(group) => {
                // In a real implementation, this would check user group membership
                self.user_in_group(&request.requester, group)
            }
            PolicyCondition::TimeOfDay { start, end } => {
                self.is_within_time_range(start, end)
            }
            PolicyCondition::IPAddress(ip_pattern) => {
                // In a real implementation, this would check client IP
                self.ip_matches_pattern(ip_pattern)
            }
            PolicyCondition::SecretType(secret_type) => {
                // Check if secret type matches
                request.secret_name.contains(secret_type)
            }
            PolicyCondition::Application(app_name) => {
                if let Some(ref app) = request.application {
                    app == app_name
                } else {
                    false
                }
            }
            PolicyCondition::Environment(env) => {
                // In a real implementation, this would check environment context
                self.is_in_environment(env)
            }
        }
    }

    /// Check if user is in a specific group
    fn user_in_group(&self, _user: &str, _group: &str) -> bool {
        // Placeholder implementation - would integrate with identity provider
        true
    }

    /// Check if current time is within allowed range
    fn is_within_time_range(&self, _start: &str, _end: &str) -> bool {
        // Placeholder implementation - would parse time ranges and check current time
        true
    }

    /// Check if IP address matches pattern
    fn ip_matches_pattern(&self, _pattern: &str) -> bool {
        // Placeholder implementation - would check client IP against pattern
        true
    }

    /// Check if running in specific environment
    fn is_in_environment(&self, _env: &str) -> bool {
        // Placeholder implementation - would check environment variables or context
        true
    }

    /// Check if user can delete a specific secret
    fn can_delete_secret(&self, _secret_name: &str, _requester: &str) -> bool {
        // Placeholder implementation - would check ownership and permissions
        true
    }

    /// Get effective policies for a user
    pub async fn get_effective_policies(&self, requester: &str) -> Vec<AccessPolicy> {
        let mut effective_policies = Vec::new();

        for policy in &self.config.policies {
            // Create a test request to evaluate policy applicability
            let test_request = SecretRequest {
                secret_name: "test".to_string(),
                requester: requester.to_string(),
                application: None,
                justification: None,
                ttl: None,
                scope: vec![],
            };

            if self.policy_matches_request(policy, &test_request) {
                let conditions: Vec<String> = policy.conditions
                    .iter()
                    .map(|c| self.condition_to_string(c))
                    .collect();

                effective_policies.push(AccessPolicy {
                    name: policy.name.clone(),
                    allowed: policy.action == PolicyAction::Allow,
                    conditions,
                    restrictions: vec![], // Would be populated based on policy details
                });
            }
        }

        effective_policies
    }

    /// Convert condition to human-readable string
    fn condition_to_string(&self, condition: &PolicyCondition) -> String {
        match condition {
            PolicyCondition::UserGroup(group) => format!("User in group: {}", group),
            PolicyCondition::TimeOfDay { start, end } => format!("Time between {} and {}", start, end),
            PolicyCondition::IPAddress(ip) => format!("IP address matches: {}", ip),
            PolicyCondition::SecretType(secret_type) => format!("Secret type: {}", secret_type),
            PolicyCondition::Application(app) => format!("Application: {}", app),
            PolicyCondition::Environment(env) => format!("Environment: {}", env),
        }
    }

    /// Update policy configuration
    pub async fn update_config(&mut self, config: PolicyConfig) {
        self.config = config;
    }

    /// Add a new policy
    pub async fn add_policy(&mut self, policy: PolicyDefinition) {
        self.config.policies.push(policy);
    }

    /// Remove a policy by name
    pub async fn remove_policy(&mut self, policy_name: &str) {
        self.config.policies.retain(|p| p.name != policy_name);
    }

    /// Get policy by name
    pub async fn get_policy(&self, policy_name: &str) -> Option<&PolicyDefinition> {
        self.config.policies.iter().find(|p| p.name == policy_name)
    }

    /// Validate policy configuration
    pub fn validate_config(&self) -> Result<(), String> {
        // Check for duplicate policy names
        let mut names = std::collections::HashSet::new();
        for policy in &self.config.policies {
            if !names.insert(&policy.name) {
                return Err(format!("Duplicate policy name: {}", policy.name));
            }
        }

        // Validate policy conditions
        for policy in &self.config.policies {
            for condition in &policy.conditions {
                if let Err(e) = self.validate_condition(condition) {
                    return Err(format!("Invalid condition in policy '{}': {}", policy.name, e));
                }
            }
        }

        Ok(())
    }

    /// Validate a policy condition
    fn validate_condition(&self, condition: &PolicyCondition) -> Result<(), String> {
        match condition {
            PolicyCondition::TimeOfDay { start, end } => {
                // Validate time format
                if start.is_empty() || end.is_empty() {
                    return Err("Time range cannot be empty".to_string());
                }
                // In a real implementation, would parse and validate time formats
            }
            PolicyCondition::IPAddress(ip) => {
                if ip.is_empty() {
                    return Err("IP address pattern cannot be empty".to_string());
                }
                // In a real implementation, would validate IP pattern format
            }
            PolicyCondition::UserGroup(group) => {
                if group.is_empty() {
                    return Err("User group cannot be empty".to_string());
                }
            }
            PolicyCondition::SecretType(secret_type) => {
                if secret_type.is_empty() {
                    return Err("Secret type cannot be empty".to_string());
                }
            }
            PolicyCondition::Application(app) => {
                if app.is_empty() {
                    return Err("Application name cannot be empty".to_string());
                }
            }
            PolicyCondition::Environment(env) => {
                if env.is_empty() {
                    return Err("Environment name cannot be empty".to_string());
                }
            }
        }
        Ok(())
    }
}