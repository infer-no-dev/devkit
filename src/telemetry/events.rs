//! Event management for telemetry

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::correlation::CorrelationContext;

#[derive(Debug)]
pub struct EventManager {
    events: RwLock<Vec<TelemetryEvent>>,
    max_events: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub event_id: Uuid,
    pub event_type: EventType,
    pub agent_id: String,
    pub message: String,
    pub data: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub correlation_context: Option<CorrelationContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    AgentStarted,
    AgentFinished,
    TaskStarted,
    TaskFinished,
    Error,
    Warning,
    Info,
    Custom(String),
}

impl EventManager {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: RwLock::new(Vec::new()),
            max_events,
        }
    }
    
    pub async fn record_event(
        &self,
        event_type: EventType,
        agent_id: String,
        message: String,
        data: HashMap<String, serde_json::Value>,
        correlation_context: Option<CorrelationContext>,
    ) {
        let event = TelemetryEvent {
            event_id: Uuid::new_v4(),
            event_type,
            agent_id,
            message,
            data,
            timestamp: Utc::now(),
            correlation_context,
        };
        
        let mut events = self.events.write().await;
        events.push(event);
        
        // Simple cleanup
        if events.len() > self.max_events {
            events.remove(0);
        }
    }
    
    pub async fn get_agent_events(&self, agent_id: &str) -> Vec<TelemetryEvent> {
        let events = self.events.read().await;
        events.iter()
            .filter(|e| e.agent_id == agent_id)
            .cloned()
            .collect()
    }
    
    pub async fn get_events(&self, limit: usize) -> Vec<TelemetryEvent> {
        let events = self.events.read().await;
        events.iter().rev().take(limit).cloned().collect()
    }
    
    pub async fn get_all_events(&self) -> Vec<TelemetryEvent> {
        let events = self.events.read().await;
        events.clone()
    }
    
    pub async fn get_total_events(&self) -> usize {
        let events = self.events.read().await;
        events.len()
    }
}