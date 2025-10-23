//! Span management for distributed tracing
//!
//! This module provides span tracking and management for agent operations,
//! supporting parent-child relationships and performance measurement.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Span manager handles the lifecycle of distributed tracing spans
#[derive(Debug)]
pub struct SpanManager {
    spans: RwLock<HashMap<Uuid, Span>>,
    max_spans: usize,
}

/// A span represents a unit of work in a distributed trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    /// Unique identifier for this span
    pub span_id: Uuid,
    /// ID of the parent span, if any
    pub parent_id: Option<Uuid>,
    /// Name of the operation
    pub operation_name: String,
    /// Kind of span (client, server, producer, consumer, internal)
    pub kind: SpanKind,
    /// When the span started
    pub start_time: DateTime<Utc>,
    /// When the span finished (None if still active)
    pub end_time: Option<DateTime<Utc>>,
    /// Duration of the span (None if still active)
    pub duration: Option<Duration>,
    /// Whether the operation was successful
    pub success: Option<bool>,
    /// Error message if the operation failed
    pub error: Option<String>,
    /// Key-value attributes
    pub attributes: HashMap<String, String>,
    /// Span status
    pub status: SpanStatus,
    /// Performance timing information
    pub start_instant: Option<chrono::DateTime<chrono::Utc>>,
}

/// Context information for a span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanContext {
    pub span_id: Uuid,
    pub trace_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub baggage: HashMap<String, String>,
}

/// Type of span operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanKind {
    /// Represents a request to a remote service
    Client,
    /// Represents a server handling a request
    Server,
    /// Represents a producer sending a message
    Producer,
    /// Represents a consumer receiving a message
    Consumer,
    /// Represents an internal operation
    Internal,
}

/// Status of a span
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanStatus {
    /// Span is currently active
    Active,
    /// Span completed successfully
    Ok,
    /// Span completed with an error
    Error,
    /// Span was cancelled
    Cancelled,
}

impl SpanManager {
    /// Create a new span manager
    pub fn new(max_spans: usize) -> Self {
        Self {
            spans: RwLock::new(HashMap::new()),
            max_spans,
        }
    }
    
    /// Start a new span
    pub async fn start_span(
        &self,
        operation_name: String,
        kind: SpanKind,
        parent_id: Option<Uuid>,
        attributes: HashMap<String, String>,
    ) -> Uuid {
        let span_id = Uuid::new_v4();
        let now = Utc::now();
        let instant = Instant::now();
        
        let span = Span {
            span_id,
            parent_id,
            operation_name,
            kind,
            start_time: now,
            end_time: None,
            duration: None,
            success: None,
            error: None,
            attributes,
            status: SpanStatus::Active,
            start_instant: Some(chrono::Utc::now()),
        };
        
        let mut spans = self.spans.write().await;
        
        // Clean up old spans if we exceed the limit
        if spans.len() >= self.max_spans {
            self.cleanup_old_spans(&mut spans).await;
        }
        
        spans.insert(span_id, span);
        span_id
    }
    
    /// Finish a span with success/failure information
    pub async fn finish_span(
        &self,
        span_id: Uuid,
        success: bool,
        error: Option<String>,
        additional_attributes: HashMap<String, String>,
    ) {
        let mut spans = self.spans.write().await;
        if let Some(span) = spans.get_mut(&span_id) {
            let end_time = Utc::now();
            span.end_time = Some(end_time);
            span.success = Some(success);
            span.error = error;
            span.status = if success { SpanStatus::Ok } else { SpanStatus::Error };
            span.attributes.extend(additional_attributes);
            
            // Calculate duration
            if let Some(start_instant) = span.start_instant {
                span.duration = Some((chrono::Utc::now() - start_instant).to_std().unwrap_or_default());
            }
        }
    }
    
    /// Add attributes to an active span
    pub async fn add_span_attributes(
        &self,
        span_id: Uuid,
        attributes: HashMap<String, String>,
    ) {
        let mut spans = self.spans.write().await;
        if let Some(span) = spans.get_mut(&span_id) {
            span.attributes.extend(attributes);
        }
    }
    
    /// Add an event to a span
    pub async fn add_span_event(
        &self,
        span_id: Uuid,
        name: &str,
        attributes: HashMap<String, String>,
    ) {
        let mut attrs = attributes;
        attrs.insert("event.name".to_string(), name.to_string());
        attrs.insert("event.timestamp".to_string(), Utc::now().to_rfc3339());
        
        self.add_span_attributes(span_id, attrs).await;
    }
    
    /// Cancel a span
    pub async fn cancel_span(&self, span_id: Uuid) {
        let mut spans = self.spans.write().await;
        if let Some(span) = spans.get_mut(&span_id) {
            span.end_time = Some(Utc::now());
            span.status = SpanStatus::Cancelled;
            
            if let Some(start_instant) = span.start_instant {
                span.duration = Some((chrono::Utc::now() - start_instant).to_std().unwrap_or_default());
            }
        }
    }
    
    /// Get a span by ID
    pub async fn get_span(&self, span_id: Uuid) -> Option<Span> {
        let spans = self.spans.read().await;
        spans.get(&span_id).cloned()
    }
    
    /// Get all spans for a specific agent
    pub async fn get_agent_spans(&self, agent_id: &str) -> Vec<Span> {
        let spans = self.spans.read().await;
        spans.values()
            .filter(|span| {
                span.attributes.get("agent.id")
                    .map(|id| id == agent_id)
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }
    
    /// Get completed spans for export
    pub async fn get_completed_spans(&self, limit: usize) -> Vec<Span> {
        let spans = self.spans.read().await;
        spans.values()
            .filter(|span| span.status != SpanStatus::Active)
            .take(limit)
            .cloned()
            .collect()
    }
    
    /// Get all spans (for shutdown export)
    pub async fn get_all_spans(&self) -> Vec<Span> {
        let spans = self.spans.read().await;
        spans.values().cloned().collect()
    }
    
    /// Get count of active spans for an agent
    pub async fn get_active_span_count(&self, agent_id: &str) -> usize {
        let spans = self.spans.read().await;
        spans.values()
            .filter(|span| {
                span.status == SpanStatus::Active &&
                span.attributes.get("agent.id")
                    .map(|id| id == agent_id)
                    .unwrap_or(false)
            })
            .count()
    }
    
    /// Get total span count
    pub async fn get_total_spans(&self) -> usize {
        let spans = self.spans.read().await;
        spans.len()
    }
    
    /// Get all unique agent IDs
    pub async fn get_all_agent_ids(&self) -> Vec<String> {
        let spans = self.spans.read().await;
        let mut agent_ids: Vec<String> = spans.values()
            .filter_map(|span| span.attributes.get("agent.id"))
            .cloned()
            .collect();
        
        agent_ids.sort();
        agent_ids.dedup();
        agent_ids
    }
    
    /// Get span hierarchy for a trace
    pub async fn get_trace_spans(&self, trace_id: Uuid) -> Vec<Span> {
        let spans = self.spans.read().await;
        let mut trace_spans: Vec<Span> = spans.values()
            .filter(|span| {
                span.span_id == trace_id || 
                self.is_descendant_of(span, trace_id, &spans)
            })
            .cloned()
            .collect();
        
        // Sort by start time to maintain chronological order
        trace_spans.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        trace_spans
    }
    
    /// Check if a span is a descendant of another span
    fn is_descendant_of(&self, span: &Span, ancestor_id: Uuid, spans: &HashMap<Uuid, Span>) -> bool {
        let mut current_parent = span.parent_id;
        while let Some(parent_id) = current_parent {
            if parent_id == ancestor_id {
                return true;
            }
            current_parent = spans.get(&parent_id).and_then(|s| s.parent_id);
        }
        false
    }
    
    /// Clean up old completed spans to make room for new ones
    async fn cleanup_old_spans(&self, spans: &mut HashMap<Uuid, Span>) {
        let completed_spans: Vec<(Uuid, DateTime<Utc>)> = spans.iter()
            .filter_map(|(id, span)| {
                if span.status != SpanStatus::Active {
                    span.end_time.map(|end| (*id, end))
                } else {
                    None
                }
            })
            .collect();
        
        if completed_spans.len() > self.max_spans / 2 {
            // Remove the oldest completed spans
            let mut sorted_spans = completed_spans;
            sorted_spans.sort_by(|a, b| a.1.cmp(&b.1));
            
            let to_remove = sorted_spans.len() - (self.max_spans / 4);
            for (span_id, _) in sorted_spans.iter().take(to_remove) {
                spans.remove(span_id);
            }
        }
    }
    
    /// Get performance metrics for spans
    pub async fn get_span_metrics(&self) -> SpanMetrics {
        let spans = self.spans.read().await;
        
        let total_spans = spans.len();
        let active_spans = spans.values().filter(|s| s.status == SpanStatus::Active).count();
        let completed_spans = total_spans - active_spans;
        let successful_spans = spans.values()
            .filter(|s| s.success == Some(true))
            .count();
        let failed_spans = spans.values()
            .filter(|s| s.success == Some(false))
            .count();
        
        let durations: Vec<Duration> = spans.values()
            .filter_map(|s| s.duration)
            .collect();
        
        let avg_duration = if !durations.is_empty() {
            durations.iter().sum::<Duration>() / durations.len() as u32
        } else {
            Duration::from_millis(0)
        };
        
        let min_duration = durations.iter().min().cloned()
            .unwrap_or(Duration::from_millis(0));
        let max_duration = durations.iter().max().cloned()
            .unwrap_or(Duration::from_millis(0));
        
        // Calculate percentiles
        let mut sorted_durations = durations;
        sorted_durations.sort();
        
        let p50 = percentile(&sorted_durations, 50);
        let p90 = percentile(&sorted_durations, 90);
        let p99 = percentile(&sorted_durations, 99);
        
        SpanMetrics {
            total_spans,
            active_spans,
            completed_spans,
            successful_spans,
            failed_spans,
            success_rate: if completed_spans > 0 {
                successful_spans as f64 / (successful_spans + failed_spans) as f64
            } else {
                0.0
            },
            avg_duration,
            min_duration,
            max_duration,
            p50_duration: p50,
            p90_duration: p90,
            p99_duration: p99,
        }
    }
}

/// Performance metrics for spans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanMetrics {
    pub total_spans: usize,
    pub active_spans: usize,
    pub completed_spans: usize,
    pub successful_spans: usize,
    pub failed_spans: usize,
    pub success_rate: f64,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub p50_duration: Duration,
    pub p90_duration: Duration,
    pub p99_duration: Duration,
}

/// Calculate percentile from sorted durations
fn percentile(sorted_durations: &[Duration], percentile: u8) -> Duration {
    if sorted_durations.is_empty() {
        return Duration::from_millis(0);
    }
    
    let index = (sorted_durations.len() as f64 * (percentile as f64 / 100.0)) as usize;
    let bounded_index = index.min(sorted_durations.len() - 1);
    sorted_durations[bounded_index]
}

impl Default for SpanKind {
    fn default() -> Self {
        SpanKind::Internal
    }
}

impl std::fmt::Display for SpanKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpanKind::Client => write!(f, "CLIENT"),
            SpanKind::Server => write!(f, "SERVER"),
            SpanKind::Producer => write!(f, "PRODUCER"),
            SpanKind::Consumer => write!(f, "CONSUMER"),
            SpanKind::Internal => write!(f, "INTERNAL"),
        }
    }
}

impl std::fmt::Display for SpanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpanStatus::Active => write!(f, "ACTIVE"),
            SpanStatus::Ok => write!(f, "OK"),
            SpanStatus::Error => write!(f, "ERROR"),
            SpanStatus::Cancelled => write!(f, "CANCELLED"),
        }
    }
}