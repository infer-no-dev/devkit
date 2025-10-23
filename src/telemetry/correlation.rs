//! Correlation context management

use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct CorrelationManager {
    contexts: RwLock<HashMap<Uuid, CorrelationContext>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationContext {
    pub trace_id: Uuid,
    pub span_id: Uuid,
    pub agent_id: String,
    pub operation: String,
    pub created_at: DateTime<Utc>,
}

impl CorrelationManager {
    pub fn new() -> Self {
        Self {
            contexts: RwLock::new(HashMap::new()),
        }
    }
    
    pub async fn set_context(&self, span_id: Uuid, context: CorrelationContext) {
        let mut contexts = self.contexts.write().await;
        contexts.insert(span_id, context);
    }
    
    pub async fn get_context(&self, span_id: Uuid) -> Option<CorrelationContext> {
        let contexts = self.contexts.read().await;
        contexts.get(&span_id).cloned()
    }
}