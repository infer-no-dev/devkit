//! Context management for logging system.
//!
//! This module provides structures and utilities for managing logging context
//! such as correlation IDs, user IDs, session IDs, and tags that should be
//! included with log entries.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Logging context that is attached to log entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogContext {
    /// Correlation ID for tracking related operations
    pub correlation_id: Option<String>,

    /// User ID if available
    pub user_id: Option<String>,

    /// Session ID for user sessions
    pub session_id: Option<String>,

    /// Request ID for HTTP requests or similar
    pub request_id: Option<String>,

    /// Transaction ID for database transactions
    pub transaction_id: Option<String>,

    /// Operation name or identifier
    pub operation: Option<String>,

    /// Service name or component
    pub service: Option<String>,

    /// Version of the service/application
    pub version: Option<String>,

    /// Environment (dev, staging, prod, etc.)
    pub environment: String,

    /// Tags for categorizing log entries
    pub tags: Vec<String>,

    /// Custom key-value pairs
    pub custom_fields: HashMap<String, String>,

    /// Trace ID for distributed tracing
    pub trace_id: Option<String>,

    /// Span ID for distributed tracing
    pub span_id: Option<String>,

    /// Parent span ID for distributed tracing
    pub parent_span_id: Option<String>,
}

impl Default for LogContext {
    fn default() -> Self {
        Self::new()
    }
}

impl LogContext {
    /// Create a new empty logging context
    pub fn new() -> Self {
        Self {
            correlation_id: None,
            user_id: None,
            session_id: None,
            request_id: None,
            transaction_id: None,
            operation: None,
            service: None,
            version: None,
            environment: "development".to_string(),
            tags: Vec::new(),
            custom_fields: HashMap::new(),
            trace_id: None,
            span_id: None,
            parent_span_id: None,
        }
    }

    /// Create a new context with basic information
    pub fn with_basic_info(
        service: Option<String>,
        version: Option<String>,
        environment: String,
    ) -> Self {
        Self {
            service,
            version,
            environment,
            ..Default::default()
        }
    }

    /// Generate and set a new correlation ID
    pub fn generate_correlation_id(&mut self) -> &str {
        let id = Uuid::new_v4().to_string();
        self.correlation_id = Some(id);
        self.correlation_id.as_ref().unwrap()
    }

    /// Generate and set a new request ID
    pub fn generate_request_id(&mut self) -> &str {
        let id = Uuid::new_v4().to_string();
        self.request_id = Some(id);
        self.request_id.as_ref().unwrap()
    }

    /// Generate and set a new trace ID
    pub fn generate_trace_id(&mut self) -> &str {
        let id = Uuid::new_v4().simple().to_string();
        self.trace_id = Some(id);
        self.trace_id.as_ref().unwrap()
    }

    /// Generate and set a new span ID
    pub fn generate_span_id(&mut self) -> &str {
        let id = Uuid::new_v4().simple().to_string()[..16].to_string();
        self.span_id = Some(id);
        self.span_id.as_ref().unwrap()
    }

    /// Set correlation ID
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    /// Set user ID
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set session ID
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Set request ID
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    /// Set transaction ID
    pub fn with_transaction_id(mut self, transaction_id: String) -> Self {
        self.transaction_id = Some(transaction_id);
        self
    }

    /// Set operation name
    pub fn with_operation(mut self, operation: String) -> Self {
        self.operation = Some(operation);
        self
    }

    /// Set service name
    pub fn with_service(mut self, service: String) -> Self {
        self.service = Some(service);
        self
    }

    /// Set version
    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    /// Set environment
    pub fn with_environment(mut self, environment: String) -> Self {
        self.environment = environment;
        self
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: String) -> &mut Self {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
        self
    }

    /// Add multiple tags
    pub fn add_tags(&mut self, tags: Vec<String>) -> &mut Self {
        for tag in tags {
            self.add_tag(tag);
        }
        self
    }

    /// Remove a tag
    pub fn remove_tag(&mut self, tag: &str) -> &mut Self {
        self.tags.retain(|t| t != tag);
        self
    }

    /// Clear all tags
    pub fn clear_tags(&mut self) -> &mut Self {
        self.tags.clear();
        self
    }

    /// Add a custom field
    pub fn add_custom_field(&mut self, key: String, value: String) -> &mut Self {
        self.custom_fields.insert(key, value);
        self
    }

    /// Remove a custom field
    pub fn remove_custom_field(&mut self, key: &str) -> &mut Self {
        self.custom_fields.remove(key);
        self
    }

    /// Set trace information
    pub fn with_trace_info(
        mut self,
        trace_id: String,
        span_id: String,
        parent_span_id: Option<String>,
    ) -> Self {
        self.trace_id = Some(trace_id);
        self.span_id = Some(span_id);
        self.parent_span_id = parent_span_id;
        self
    }

    /// Clone context and generate a new span ID for child operations
    pub fn create_child_span(&self) -> Self {
        let mut child = self.clone();
        let current_span = child.span_id.clone();
        child.generate_span_id();
        child.parent_span_id = current_span;
        child
    }

    /// Merge another context into this one
    pub fn merge(&mut self, other: &LogContext) -> &mut Self {
        // Keep existing values but add missing ones
        if self.correlation_id.is_none() && other.correlation_id.is_some() {
            self.correlation_id = other.correlation_id.clone();
        }
        if self.user_id.is_none() && other.user_id.is_some() {
            self.user_id = other.user_id.clone();
        }
        if self.session_id.is_none() && other.session_id.is_some() {
            self.session_id = other.session_id.clone();
        }
        if self.request_id.is_none() && other.request_id.is_some() {
            self.request_id = other.request_id.clone();
        }
        if self.transaction_id.is_none() && other.transaction_id.is_some() {
            self.transaction_id = other.transaction_id.clone();
        }
        if self.operation.is_none() && other.operation.is_some() {
            self.operation = other.operation.clone();
        }
        if self.service.is_none() && other.service.is_some() {
            self.service = other.service.clone();
        }
        if self.version.is_none() && other.version.is_some() {
            self.version = other.version.clone();
        }
        if self.trace_id.is_none() && other.trace_id.is_some() {
            self.trace_id = other.trace_id.clone();
        }
        if self.span_id.is_none() && other.span_id.is_some() {
            self.span_id = other.span_id.clone();
        }
        if self.parent_span_id.is_none() && other.parent_span_id.is_some() {
            self.parent_span_id = other.parent_span_id.clone();
        }

        // Merge tags
        for tag in &other.tags {
            self.add_tag(tag.clone());
        }

        // Merge custom fields (other takes precedence)
        for (key, value) in &other.custom_fields {
            self.custom_fields.insert(key.clone(), value.clone());
        }

        self
    }

    /// Clear all context information
    pub fn clear(&mut self) -> &mut Self {
        *self = LogContext::new();
        self
    }

    /// Check if context has any meaningful information
    pub fn is_empty(&self) -> bool {
        self.correlation_id.is_none()
            && self.user_id.is_none()
            && self.session_id.is_none()
            && self.request_id.is_none()
            && self.transaction_id.is_none()
            && self.operation.is_none()
            && self.service.is_none()
            && self.version.is_none()
            && self.tags.is_empty()
            && self.custom_fields.is_empty()
            && self.trace_id.is_none()
            && self.span_id.is_none()
            && self.parent_span_id.is_none()
    }

    /// Convert to key-value pairs for structured logging
    pub fn to_fields(&self) -> HashMap<String, serde_json::Value> {
        let mut fields = HashMap::new();

        if let Some(ref correlation_id) = self.correlation_id {
            fields.insert(
                "correlation_id".to_string(),
                serde_json::Value::String(correlation_id.clone()),
            );
        }
        if let Some(ref user_id) = self.user_id {
            fields.insert(
                "user_id".to_string(),
                serde_json::Value::String(user_id.clone()),
            );
        }
        if let Some(ref session_id) = self.session_id {
            fields.insert(
                "session_id".to_string(),
                serde_json::Value::String(session_id.clone()),
            );
        }
        if let Some(ref request_id) = self.request_id {
            fields.insert(
                "request_id".to_string(),
                serde_json::Value::String(request_id.clone()),
            );
        }
        if let Some(ref transaction_id) = self.transaction_id {
            fields.insert(
                "transaction_id".to_string(),
                serde_json::Value::String(transaction_id.clone()),
            );
        }
        if let Some(ref operation) = self.operation {
            fields.insert(
                "operation".to_string(),
                serde_json::Value::String(operation.clone()),
            );
        }
        if let Some(ref service) = self.service {
            fields.insert(
                "service".to_string(),
                serde_json::Value::String(service.clone()),
            );
        }
        if let Some(ref version) = self.version {
            fields.insert(
                "version".to_string(),
                serde_json::Value::String(version.clone()),
            );
        }
        if let Some(ref trace_id) = self.trace_id {
            fields.insert(
                "trace_id".to_string(),
                serde_json::Value::String(trace_id.clone()),
            );
        }
        if let Some(ref span_id) = self.span_id {
            fields.insert(
                "span_id".to_string(),
                serde_json::Value::String(span_id.clone()),
            );
        }
        if let Some(ref parent_span_id) = self.parent_span_id {
            fields.insert(
                "parent_span_id".to_string(),
                serde_json::Value::String(parent_span_id.clone()),
            );
        }

        fields.insert(
            "environment".to_string(),
            serde_json::Value::String(self.environment.clone()),
        );

        if !self.tags.is_empty() {
            fields.insert(
                "tags".to_string(),
                serde_json::Value::Array(
                    self.tags
                        .iter()
                        .map(|tag| serde_json::Value::String(tag.clone()))
                        .collect(),
                ),
            );
        }

        for (key, value) in &self.custom_fields {
            fields.insert(key.clone(), serde_json::Value::String(value.clone()));
        }

        fields
    }
}

/// Builder for creating logging contexts
#[derive(Debug, Default)]
pub struct ContextBuilder {
    context: LogContext,
}

impl ContextBuilder {
    /// Create a new context builder
    pub fn new() -> Self {
        Self {
            context: LogContext::new(),
        }
    }

    /// Set correlation ID
    pub fn correlation_id(mut self, correlation_id: String) -> Self {
        self.context.correlation_id = Some(correlation_id);
        self
    }

    /// Generate a new correlation ID
    pub fn generate_correlation_id(mut self) -> Self {
        self.context.generate_correlation_id();
        self
    }

    /// Set user ID
    pub fn user_id(mut self, user_id: String) -> Self {
        self.context.user_id = Some(user_id);
        self
    }

    /// Set session ID
    pub fn session_id(mut self, session_id: String) -> Self {
        self.context.session_id = Some(session_id);
        self
    }

    /// Set request ID
    pub fn request_id(mut self, request_id: String) -> Self {
        self.context.request_id = Some(request_id);
        self
    }

    /// Generate a new request ID
    pub fn generate_request_id(mut self) -> Self {
        self.context.generate_request_id();
        self
    }

    /// Set transaction ID
    pub fn transaction_id(mut self, transaction_id: String) -> Self {
        self.context.transaction_id = Some(transaction_id);
        self
    }

    /// Set operation name
    pub fn operation(mut self, operation: String) -> Self {
        self.context.operation = Some(operation);
        self
    }

    /// Set service name
    pub fn service(mut self, service: String) -> Self {
        self.context.service = Some(service);
        self
    }

    /// Set version
    pub fn version(mut self, version: String) -> Self {
        self.context.version = Some(version);
        self
    }

    /// Set environment
    pub fn environment(mut self, environment: String) -> Self {
        self.context.environment = environment;
        self
    }

    /// Add a tag
    pub fn tag(mut self, tag: String) -> Self {
        self.context.add_tag(tag);
        self
    }

    /// Add multiple tags
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.context.add_tags(tags);
        self
    }

    /// Add a custom field
    pub fn custom_field(mut self, key: String, value: String) -> Self {
        self.context.add_custom_field(key, value);
        self
    }

    /// Set trace information
    pub fn trace_info(
        mut self,
        trace_id: String,
        span_id: String,
        parent_span_id: Option<String>,
    ) -> Self {
        self.context = self
            .context
            .with_trace_info(trace_id, span_id, parent_span_id);
        self
    }

    /// Generate trace information
    pub fn generate_trace_info(mut self) -> Self {
        self.context.generate_trace_id();
        self.context.generate_span_id();
        self
    }

    /// Build the context
    pub fn build(self) -> LogContext {
        self.context
    }
}

/// Thread-local storage for logging context
thread_local! {
    static CURRENT_CONTEXT: std::cell::RefCell<LogContext> = std::cell::RefCell::new(LogContext::new());
}

/// Get the current thread-local logging context
pub fn current_context() -> LogContext {
    CURRENT_CONTEXT.with(|ctx| ctx.borrow().clone())
}

/// Set the current thread-local logging context
pub fn set_current_context(context: LogContext) {
    CURRENT_CONTEXT.with(|ctx| *ctx.borrow_mut() = context);
}

/// Update the current thread-local logging context
pub fn update_current_context<F>(updater: F)
where
    F: FnOnce(&mut LogContext),
{
    CURRENT_CONTEXT.with(|ctx| updater(&mut ctx.borrow_mut()));
}

/// Clear the current thread-local logging context
pub fn clear_current_context() {
    CURRENT_CONTEXT.with(|ctx| {
        ctx.borrow_mut().clear();
    });
}

/// Execute a closure with a temporary logging context
pub fn with_context<F, R>(context: LogContext, f: F) -> R
where
    F: FnOnce() -> R,
{
    let previous = current_context();
    set_current_context(context);
    let result = f();
    set_current_context(previous);
    result
}

/// Context guard that automatically restores the previous context when dropped
pub struct ContextGuard {
    previous_context: LogContext,
}

impl ContextGuard {
    /// Create a new context guard with the given context
    pub fn new(context: LogContext) -> Self {
        let previous = current_context();
        set_current_context(context);
        Self {
            previous_context: previous,
        }
    }
}

impl Drop for ContextGuard {
    fn drop(&mut self) {
        set_current_context(self.previous_context.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let context = LogContext::new();
        assert!(context.is_empty());
        assert_eq!(context.environment, "development");
    }

    #[test]
    fn test_context_builder() {
        let context = ContextBuilder::new()
            .correlation_id("test-123".to_string())
            .user_id("user-456".to_string())
            .operation("test_operation".to_string())
            .tag("test".to_string())
            .custom_field("key".to_string(), "value".to_string())
            .build();

        assert_eq!(context.correlation_id, Some("test-123".to_string()));
        assert_eq!(context.user_id, Some("user-456".to_string()));
        assert_eq!(context.operation, Some("test_operation".to_string()));
        assert!(context.tags.contains(&"test".to_string()));
        assert_eq!(context.custom_fields.get("key"), Some(&"value".to_string()));
        assert!(!context.is_empty());
    }

    #[test]
    fn test_context_merge() {
        let mut ctx1 = LogContext::new().with_correlation_id("corr-1".to_string());
        let ctx2 = LogContext::new()
            .with_user_id("user-2".to_string())
            .with_correlation_id("corr-2".to_string()); // This shouldn't override

        ctx1.merge(&ctx2);

        assert_eq!(ctx1.correlation_id, Some("corr-1".to_string())); // Should keep original
        assert_eq!(ctx1.user_id, Some("user-2".to_string())); // Should get from ctx2
    }

    #[test]
    fn test_child_span_creation() {
        let mut parent_context = LogContext::new();
        parent_context.generate_trace_id();
        parent_context.generate_span_id();
        let parent_span_id = parent_context.span_id.clone();

        let child_context = parent_context.create_child_span();

        assert_eq!(child_context.trace_id, parent_context.trace_id);
        assert_ne!(child_context.span_id, parent_context.span_id);
        assert_eq!(child_context.parent_span_id, parent_span_id);
    }

    #[test]
    fn test_thread_local_context() {
        let context = ContextBuilder::new()
            .correlation_id("thread-test".to_string())
            .build();

        set_current_context(context.clone());
        let retrieved = current_context();

        assert_eq!(retrieved.correlation_id, context.correlation_id);

        clear_current_context();
        let cleared = current_context();
        assert!(cleared.is_empty());
    }

    #[test]
    fn test_with_context() {
        let original_context = ContextBuilder::new()
            .correlation_id("original".to_string())
            .build();
        set_current_context(original_context.clone());

        let temp_context = ContextBuilder::new()
            .correlation_id("temporary".to_string())
            .build();

        let result = with_context(temp_context, || {
            let current = current_context();
            current.correlation_id.unwrap_or_default()
        });

        assert_eq!(result, "temporary");

        let restored = current_context();
        assert_eq!(restored.correlation_id, Some("original".to_string()));
    }

    #[test]
    fn test_context_guard() {
        let original_context = ContextBuilder::new()
            .correlation_id("original".to_string())
            .build();
        set_current_context(original_context.clone());

        {
            let temp_context = ContextBuilder::new()
                .correlation_id("guard_test".to_string())
                .build();
            let _guard = ContextGuard::new(temp_context);

            let current = current_context();
            assert_eq!(current.correlation_id, Some("guard_test".to_string()));
        }

        let restored = current_context();
        assert_eq!(restored.correlation_id, Some("original".to_string()));
    }

    #[test]
    fn test_context_to_fields() {
        let context = ContextBuilder::new()
            .correlation_id("test-123".to_string())
            .user_id("user-456".to_string())
            .tag("test".to_string())
            .custom_field("custom_key".to_string(), "custom_value".to_string())
            .build();

        let fields = context.to_fields();

        assert_eq!(
            fields.get("correlation_id").unwrap(),
            &serde_json::Value::String("test-123".to_string())
        );
        assert_eq!(
            fields.get("user_id").unwrap(),
            &serde_json::Value::String("user-456".to_string())
        );
        assert_eq!(
            fields.get("custom_key").unwrap(),
            &serde_json::Value::String("custom_value".to_string())
        );
        assert!(fields.contains_key("tags"));
    }
}
