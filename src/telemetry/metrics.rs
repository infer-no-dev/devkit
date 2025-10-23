//! Metrics collection for telemetry

use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct MetricsCollector {
    metrics: RwLock<Vec<TelemetryMetrics>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryMetrics {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub tags: HashMap<String, String>,
    pub span_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    pub value: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: RwLock::new(Vec::new()),
        }
    }
    
    pub async fn record_metric(
        &self,
        name: &str,
        value: f64,
        unit: &str,
        tags: HashMap<String, String>,
        span_id: Option<Uuid>,
    ) {
        let metric = TelemetryMetrics {
            name: name.to_string(),
            value,
            unit: unit.to_string(),
            tags,
            span_id,
        };
        
        let mut metrics = self.metrics.write().await;
        metrics.push(metric);
    }
    
    pub async fn get_agent_metrics(&self, agent_id: &str) -> Vec<TelemetryMetrics> {
        let metrics = self.metrics.read().await;
        metrics.iter()
            .filter(|m| m.tags.get("agent.id").map(|id| id == agent_id).unwrap_or(false))
            .cloned()
            .collect()
    }
    
    pub async fn get_metrics(&self, limit: usize) -> Vec<TelemetryMetrics> {
        let metrics = self.metrics.read().await;
        metrics.iter().rev().take(limit).cloned().collect()
    }
    
    pub async fn get_all_metrics(&self) -> Vec<TelemetryMetrics> {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }
    
    pub async fn get_total_metrics(&self) -> usize {
        let metrics = self.metrics.read().await;
        metrics.len()
    }
}