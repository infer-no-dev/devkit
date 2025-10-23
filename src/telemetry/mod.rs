//! Structured telemetry system with per-agent spans and distributed tracing
//!
//! This module provides comprehensive telemetry for the agentic development environment,
//! including distributed tracing, per-agent performance monitoring, and structured
//! event correlation for debugging and performance optimization.

pub mod spans;
pub mod events;
pub mod metrics;
pub mod correlation;
pub mod exporters;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

pub use spans::{SpanManager, Span, SpanContext, SpanKind};
pub use events::{TelemetryEvent, EventManager, EventType};
pub use metrics::{TelemetryMetrics, MetricsCollector, MetricValue};
pub use correlation::{CorrelationManager, CorrelationContext};
pub use exporters::{TelemetryExporter, JsonExporter, OtelExporter};

/// Main telemetry system that coordinates all telemetry activities
pub struct TelemetrySystem {
    span_manager: Arc<SpanManager>,
    event_manager: Arc<EventManager>,
    metrics_collector: Arc<MetricsCollector>,
    correlation_manager: Arc<CorrelationManager>,
    exporters: Vec<Arc<dyn TelemetryExporter + Send + Sync>>,
    config: TelemetryConfig,
    start_time: Instant,
}

/// Configuration for the telemetry system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Whether telemetry is enabled
    pub enabled: bool,
    /// Sampling rate for traces (0.0 to 1.0)
    pub trace_sample_rate: f64,
    /// Whether to export to OpenTelemetry
    pub enable_otel_export: bool,
    /// Whether to export to JSON files
    pub enable_json_export: bool,
    /// JSON export file path
    pub json_export_path: Option<String>,
    /// Maximum number of spans to keep in memory
    pub max_spans: usize,
    /// Maximum number of events to keep in memory
    pub max_events: usize,
    /// Batch size for exports
    pub export_batch_size: usize,
    /// Export interval in seconds
    pub export_interval_seconds: u64,
    /// Agent-specific configurations
    pub agent_configs: HashMap<String, AgentTelemetryConfig>,
}

/// Agent-specific telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTelemetryConfig {
    /// Whether to enable detailed tracing for this agent
    pub detailed_tracing: bool,
    /// Custom sample rate for this agent
    pub sample_rate: Option<f64>,
    /// Additional tags to apply to all spans from this agent
    pub default_tags: HashMap<String, String>,
    /// Whether to track performance metrics
    pub track_performance: bool,
    /// Whether to track memory usage
    pub track_memory: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            trace_sample_rate: 1.0, // 100% sampling for development
            enable_otel_export: false,
            enable_json_export: true,
            json_export_path: Some(".devkit/telemetry.json".to_string()),
            max_spans: 10_000,
            max_events: 50_000,
            export_batch_size: 100,
            export_interval_seconds: 30,
            agent_configs: HashMap::new(),
        }
    }
}

impl Default for AgentTelemetryConfig {
    fn default() -> Self {
        Self {
            detailed_tracing: true,
            sample_rate: None, // Use global sample rate
            default_tags: HashMap::new(),
            track_performance: true,
            track_memory: true,
        }
    }
}

impl TelemetrySystem {
    /// Create a new telemetry system
    pub fn new(config: TelemetryConfig) -> Result<Self, TelemetryError> {
        let span_manager = Arc::new(SpanManager::new(config.max_spans));
        let event_manager = Arc::new(EventManager::new(config.max_events));
        let metrics_collector = Arc::new(MetricsCollector::new());
        let correlation_manager = Arc::new(CorrelationManager::new());
        
        let mut exporters: Vec<Arc<dyn TelemetryExporter + Send + Sync>> = Vec::new();
        
        if config.enable_json_export {
            let path = config.json_export_path
                .as_ref()
                .unwrap_or(&".devkit/telemetry.json".to_string())
                .clone();
            exporters.push(Arc::new(JsonExporter::new(path)?));
        }
        
        if config.enable_otel_export {
            exporters.push(Arc::new(OtelExporter::new()?));
        }
        
        Ok(Self {
            span_manager,
            event_manager,
            metrics_collector,
            correlation_manager,
            exporters,
            config,
            start_time: Instant::now(),
        })
    }
    
    /// Start the telemetry system
    pub async fn start(&self) -> Result<(), TelemetryError> {
        if !self.config.enabled {
            return Ok(());
        }
        
        // Start periodic export task
        let exporters = self.exporters.clone();
        let span_manager = Arc::clone(&self.span_manager);
        let event_manager = Arc::clone(&self.event_manager);
        let metrics_collector = Arc::clone(&self.metrics_collector);
        let export_interval = Duration::from_secs(self.config.export_interval_seconds);
        let batch_size = self.config.export_batch_size;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(export_interval);
            
            loop {
                interval.tick().await;
                
                // Export spans
                let spans = span_manager.get_completed_spans(batch_size).await;
                if !spans.is_empty() {
                    for exporter in &exporters {
                        if let Err(e) = exporter.export_spans(&spans).await {
                            eprintln!("Failed to export spans: {}", e);
                        }
                    }
                }
                
                // Export events
                let events = event_manager.get_events(batch_size).await;
                if !events.is_empty() {
                    for exporter in &exporters {
                        if let Err(e) = exporter.export_events(&events).await {
                            eprintln!("Failed to export events: {}", e);
                        }
                    }
                }
                
                // Export metrics
                let metrics = metrics_collector.get_metrics(batch_size).await;
                if !metrics.is_empty() {
                    for exporter in &exporters {
                        if let Err(e) = exporter.export_metrics(&metrics).await {
                            eprintln!("Failed to export metrics: {}", e);
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Create a new span for an agent operation
    pub async fn start_agent_span(
        &self,
        agent_id: &str,
        operation: &str,
        kind: SpanKind,
        parent_span_id: Option<Uuid>,
    ) -> Result<Uuid, TelemetryError> {
        if !self.config.enabled || !self.should_sample(agent_id) {
            return Ok(Uuid::new_v4()); // Return dummy span ID
        }
        
        let mut tags = HashMap::new();
        tags.insert("agent.id".to_string(), agent_id.to_string());
        tags.insert("agent.operation".to_string(), operation.to_string());
        
        // Add agent-specific tags
        if let Some(agent_config) = self.config.agent_configs.get(agent_id) {
            tags.extend(agent_config.default_tags.clone());
        }
        
        let span_id = self.span_manager.start_span(
            format!("{}.{}", agent_id, operation),
            kind,
            parent_span_id,
            tags,
        ).await;
        
        // Create correlation context
        let correlation_ctx = CorrelationContext {
            trace_id: span_id, // For simplicity, using span_id as trace_id
            span_id,
            agent_id: agent_id.to_string(),
            operation: operation.to_string(),
            created_at: Utc::now(),
        };
        
        self.correlation_manager.set_context(span_id, correlation_ctx).await;
        
        Ok(span_id)
    }
    
    /// Finish a span with result information
    pub async fn finish_span(
        &self,
        span_id: Uuid,
        success: bool,
        error: Option<String>,
        attributes: HashMap<String, String>,
    ) -> Result<(), TelemetryError> {
        if !self.config.enabled {
            return Ok(());
        }
        
        self.span_manager.finish_span(span_id, success, error, attributes).await;
        Ok(())
    }
    
    /// Record a telemetry event
    pub async fn record_event(
        &self,
        event_type: EventType,
        agent_id: &str,
        message: String,
        data: HashMap<String, serde_json::Value>,
        span_id: Option<Uuid>,
    ) -> Result<(), TelemetryError> {
        if !self.config.enabled {
            return Ok(());
        }
        
        let correlation_ctx = if let Some(span_id) = span_id {
            self.correlation_manager.get_context(span_id).await
        } else {
            None
        };
        
        self.event_manager.record_event(
            event_type,
            agent_id.to_string(),
            message,
            data,
            correlation_ctx,
        ).await;
        
        Ok(())
    }
    
    /// Record a performance metric
    pub async fn record_metric(
        &self,
        name: &str,
        value: f64,
        unit: &str,
        agent_id: Option<&str>,
        span_id: Option<Uuid>,
        tags: HashMap<String, String>,
    ) -> Result<(), TelemetryError> {
        if !self.config.enabled {
            return Ok(());
        }
        
        let mut metric_tags = tags;
        if let Some(agent_id) = agent_id {
            metric_tags.insert("agent.id".to_string(), agent_id.to_string());
        }
        
        self.metrics_collector.record_metric(
            name,
            value,
            unit,
            metric_tags,
            span_id,
        ).await;
        
        Ok(())
    }
    
    /// Get telemetry summary for an agent
    pub async fn get_agent_summary(&self, agent_id: &str) -> AgentTelemetrySummary {
        let spans = self.span_manager.get_agent_spans(agent_id).await;
        let events = self.event_manager.get_agent_events(agent_id).await;
        let metrics = self.metrics_collector.get_agent_metrics(agent_id).await;
        
        let total_operations = spans.len();
        let successful_operations = spans.iter().filter(|s| s.success.unwrap_or(false)).count();
        let failed_operations = total_operations - successful_operations;
        
        let avg_duration = if !spans.is_empty() {
            spans.iter()
                .filter_map(|s| s.duration)
                .map(|d| d.as_millis() as f64)
                .sum::<f64>() / spans.len() as f64
        } else {
            0.0
        };
        
        AgentTelemetrySummary {
            agent_id: agent_id.to_string(),
            total_operations,
            successful_operations,
            failed_operations,
            success_rate: if total_operations > 0 {
                successful_operations as f64 / total_operations as f64
            } else {
                0.0
            },
            average_duration_ms: avg_duration,
            total_events: events.len(),
            active_spans: self.span_manager.get_active_span_count(agent_id).await,
            last_activity: spans.iter()
                .map(|s| s.start_time)
                .max(),
        }
    }
    
    /// Get overall telemetry summary
    pub async fn get_system_summary(&self) -> SystemTelemetrySummary {
        let total_spans = self.span_manager.get_total_spans().await;
        let total_events = self.event_manager.get_total_events().await;
        let total_metrics = self.metrics_collector.get_total_metrics().await;
        let uptime = self.start_time.elapsed();
        
        let agent_ids = self.span_manager.get_all_agent_ids().await;
        let mut agent_summaries = Vec::new();
        for agent_id in &agent_ids {
            agent_summaries.push(self.get_agent_summary(agent_id));
        }
        
        // Simplified to avoid futures dependency
        let mut agent_summaries = Vec::new();
        for agent_id in self.span_manager.get_all_agent_ids().await {
            agent_summaries.push(self.get_agent_summary(&agent_id).await);
        }
        
        SystemTelemetrySummary {
            uptime,
            total_spans,
            total_events,
            total_metrics,
            active_agents: agent_summaries.len(),
            agent_summaries: agent_summaries.clone(),
            system_health: if agent_summaries.iter().any(|s| s.success_rate < 0.8) {
                "Warning".to_string()
            } else {
                "Healthy".to_string()
            },
        }
    }
    
    /// Check if we should sample telemetry for an agent
    fn should_sample(&self, agent_id: &str) -> bool {
        let sample_rate = self.config.agent_configs
            .get(agent_id)
            .and_then(|config| config.sample_rate)
            .unwrap_or(self.config.trace_sample_rate);
        
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen::<f64>() < sample_rate
    }
    
    /// Shutdown the telemetry system
    pub async fn shutdown(&self) -> Result<(), TelemetryError> {
        // Export any remaining data
        for exporter in &self.exporters {
            let spans = self.span_manager.get_all_spans().await;
            let events = self.event_manager.get_all_events().await;
            let metrics = self.metrics_collector.get_all_metrics().await;
            
            if !spans.is_empty() {
                exporter.export_spans(&spans).await?;
            }
            if !events.is_empty() {
                exporter.export_events(&events).await?;
            }
            if !metrics.is_empty() {
                exporter.export_metrics(&metrics).await?;
            }
        }
        
        Ok(())
    }
}

/// Summary of telemetry data for a specific agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTelemetrySummary {
    pub agent_id: String,
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub success_rate: f64,
    pub average_duration_ms: f64,
    pub total_events: usize,
    pub active_spans: usize,
    pub last_activity: Option<DateTime<Utc>>,
}

/// Summary of overall system telemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemTelemetrySummary {
    pub uptime: Duration,
    pub total_spans: usize,
    pub total_events: usize,
    pub total_metrics: usize,
    pub active_agents: usize,
    pub agent_summaries: Vec<AgentTelemetrySummary>,
    pub system_health: String,
}

/// Telemetry system errors
#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Export error: {0}")]
    ExportError(String),
    
    #[error("Span error: {0}")]
    SpanError(String),
    
    #[error("Event error: {0}")]
    EventError(String),
    
    #[error("Metrics error: {0}")]
    MetricsError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("System error: {0}")]
    SystemError(String),
    
    #[error("Export error: {0}")]
    BoxError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// Convenience macros for telemetry
#[macro_export]
macro_rules! telemetry_span {
    ($system:expr, $agent_id:expr, $operation:expr, $kind:expr) => {
        $system.start_agent_span($agent_id, $operation, $kind, None).await
    };
    ($system:expr, $agent_id:expr, $operation:expr, $kind:expr, $parent:expr) => {
        $system.start_agent_span($agent_id, $operation, $kind, Some($parent)).await
    };
}

#[macro_export]
macro_rules! telemetry_event {
    ($system:expr, $type:expr, $agent_id:expr, $message:expr) => {
        $system.record_event($type, $agent_id, $message.to_string(), HashMap::new(), None).await
    };
    ($system:expr, $type:expr, $agent_id:expr, $message:expr, $data:expr) => {
        $system.record_event($type, $agent_id, $message.to_string(), $data, None).await
    };
    ($system:expr, $type:expr, $agent_id:expr, $message:expr, $data:expr, $span_id:expr) => {
        $system.record_event($type, $agent_id, $message.to_string(), $data, Some($span_id)).await
    };
}

#[macro_export]
macro_rules! telemetry_metric {
    ($system:expr, $name:expr, $value:expr, $unit:expr) => {
        $system.record_metric($name, $value, $unit, None, None, HashMap::new()).await
    };
    ($system:expr, $name:expr, $value:expr, $unit:expr, $agent_id:expr) => {
        $system.record_metric($name, $value, $unit, Some($agent_id), None, HashMap::new()).await
    };
    ($system:expr, $name:expr, $value:expr, $unit:expr, $agent_id:expr, $tags:expr) => {
        $system.record_metric($name, $value, $unit, Some($agent_id), None, $tags).await
    };
}

/// Global telemetry system instance
static GLOBAL_TELEMETRY: once_cell::sync::OnceCell<Arc<TelemetrySystem>> = 
    once_cell::sync::OnceCell::new();

/// Initialize the global telemetry system
pub fn init_global_telemetry(config: TelemetryConfig) -> Result<(), TelemetryError> {
    let telemetry = Arc::new(TelemetrySystem::new(config)?);
    GLOBAL_TELEMETRY
        .set(telemetry)
        .map_err(|_| TelemetryError::SystemError("Global telemetry already initialized".to_string()))?;
    Ok(())
}

/// Get the global telemetry system
pub fn global_telemetry() -> Option<Arc<TelemetrySystem>> {
    GLOBAL_TELEMETRY.get().cloned()
}