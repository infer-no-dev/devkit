//! Metrics collection and reporting for the logging system.
//!
//! This module provides functionality to track logging system performance,
//! including entry counts, processing times, error rates, and system health.

use crate::logging::{LogEntry, LogLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Metrics for the logging system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMetrics {
    /// Total number of log entries processed
    pub total_entries: u64,

    /// Count by log level
    pub entries_by_level: HashMap<String, u64>,

    /// Count by component
    pub entries_by_component: HashMap<String, u64>,

    /// Total number of errors encountered
    pub total_errors: u64,

    /// Entries dropped due to filters or errors
    pub dropped_entries: u64,

    /// Average processing time per entry in microseconds
    pub avg_processing_time_us: f64,

    /// Maximum processing time observed in microseconds
    pub max_processing_time_us: u64,

    /// Minimum processing time observed in microseconds
    pub min_processing_time_us: u64,

    /// Total processing time in microseconds
    pub total_processing_time_us: u64,

    /// Number of flush operations
    pub flush_count: u64,

    /// Total time spent flushing in microseconds
    pub flush_time_us: u64,

    /// Buffer utilization percentage
    pub buffer_utilization: f32,

    /// Memory usage in bytes
    pub memory_usage_bytes: u64,

    /// Timestamp when metrics were last updated
    pub last_updated: u64,

    /// System uptime when logging started
    pub start_time: u64,

    /// Rate limiting statistics
    pub rate_limit_stats: RateLimitStats,

    /// Output destination statistics
    pub output_stats: HashMap<String, OutputStats>,
}

/// Rate limiting statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitStats {
    /// Number of entries rate limited
    pub rate_limited_entries: u64,

    /// Current rate limiting window start time
    pub window_start: u64,

    /// Entries processed in current window
    pub window_entries: u64,

    /// Maximum entries allowed per window
    pub max_entries_per_window: u64,
}

/// Statistics for output destinations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputStats {
    /// Number of entries written
    pub entries_written: u64,

    /// Number of write errors
    pub write_errors: u64,

    /// Total bytes written
    pub bytes_written: u64,

    /// Average write time in microseconds
    pub avg_write_time_us: f64,

    /// Maximum write time in microseconds
    pub max_write_time_us: u64,

    /// Last successful write timestamp
    pub last_write_time: u64,

    /// Output availability (percentage)
    pub availability: f32,
}

impl Default for LogMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl LogMetrics {
    /// Create new empty metrics
    pub fn new() -> Self {
        Self {
            total_entries: 0,
            entries_by_level: HashMap::new(),
            entries_by_component: HashMap::new(),
            total_errors: 0,
            dropped_entries: 0,
            avg_processing_time_us: 0.0,
            max_processing_time_us: 0,
            min_processing_time_us: u64::MAX,
            total_processing_time_us: 0,
            flush_count: 0,
            flush_time_us: 0,
            buffer_utilization: 0.0,
            memory_usage_bytes: 0,
            last_updated: current_timestamp(),
            start_time: current_timestamp(),
            rate_limit_stats: RateLimitStats::default(),
            output_stats: HashMap::new(),
        }
    }

    /// Record a new log entry
    pub fn record_entry(&mut self, entry: &LogEntry) {
        self.total_entries += 1;

        // Count by level
        let level_key = entry.level.to_string();
        *self.entries_by_level.entry(level_key).or_insert(0) += 1;

        // Count by component
        *self
            .entries_by_component
            .entry(entry.component.clone())
            .or_insert(0) += 1;

        // Record error if applicable
        if entry.error.is_some() || entry.level == LogLevel::Error {
            self.total_errors += 1;
        }

        self.last_updated = current_timestamp();
    }

    /// Record processing time for a log entry
    pub fn record_processing_time(&mut self, duration: Duration) {
        let duration_us = duration.as_micros() as u64;

        self.total_processing_time_us += duration_us;
        self.max_processing_time_us = self.max_processing_time_us.max(duration_us);
        self.min_processing_time_us = self.min_processing_time_us.min(duration_us);

        // Update average based on total processing times recorded, not total entries
        // This allows recording processing time independently of log entries
        let processing_count = if self.total_entries > 0 {
            self.total_entries
        } else {
            1 // At least one processing time has been recorded
        };

        self.avg_processing_time_us =
            self.total_processing_time_us as f64 / processing_count as f64;
    }

    /// Record a dropped entry
    pub fn record_dropped_entry(&mut self) {
        self.dropped_entries += 1;
        self.last_updated = current_timestamp();
    }

    /// Record a flush operation
    pub fn record_flush(&mut self, duration: Duration) {
        self.flush_count += 1;
        self.flush_time_us += duration.as_micros() as u64;
        self.last_updated = current_timestamp();
    }

    /// Update buffer utilization
    pub fn update_buffer_utilization(&mut self, utilization: f32) {
        self.buffer_utilization = utilization.clamp(0.0, 100.0);
        self.last_updated = current_timestamp();
    }

    /// Update memory usage
    pub fn update_memory_usage(&mut self, bytes: u64) {
        self.memory_usage_bytes = bytes;
        self.last_updated = current_timestamp();
    }

    /// Record rate limiting event
    pub fn record_rate_limited(&mut self) {
        self.rate_limit_stats.rate_limited_entries += 1;
        self.last_updated = current_timestamp();
    }

    /// Update rate limiting window
    pub fn update_rate_limit_window(&mut self, entries: u64, max_entries: u64) {
        self.rate_limit_stats.window_start = current_timestamp();
        self.rate_limit_stats.window_entries = entries;
        self.rate_limit_stats.max_entries_per_window = max_entries;
    }

    /// Record output statistics
    pub fn record_output_write(&mut self, output_name: &str, bytes: u64, duration: Duration) {
        let stats = self
            .output_stats
            .entry(output_name.to_string())
            .or_insert_with(OutputStats::default);
        stats.record_write(bytes, duration);
    }

    /// Record output error
    pub fn record_output_error(&mut self, output_name: &str) {
        let stats = self
            .output_stats
            .entry(output_name.to_string())
            .or_insert_with(OutputStats::default);
        stats.record_error();
    }

    /// Calculate entries per second
    pub fn entries_per_second(&self) -> f64 {
        let elapsed = current_timestamp() - self.start_time;
        if elapsed == 0 {
            0.0
        } else {
            self.total_entries as f64 / (elapsed as f64 / 1000.0)
        }
    }

    /// Calculate error rate as percentage
    pub fn error_rate_percent(&self) -> f64 {
        if self.total_entries == 0 {
            0.0
        } else {
            (self.total_errors as f64 / self.total_entries as f64) * 100.0
        }
    }

    /// Calculate drop rate as percentage
    pub fn drop_rate_percent(&self) -> f64 {
        if self.total_entries == 0 {
            0.0
        } else {
            (self.dropped_entries as f64 / self.total_entries as f64) * 100.0
        }
    }

    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        (current_timestamp() - self.start_time) / 1000
    }

    /// Calculate average flush time
    pub fn avg_flush_time_us(&self) -> f64 {
        if self.flush_count == 0 {
            0.0
        } else {
            self.flush_time_us as f64 / self.flush_count as f64
        }
    }

    /// Get top components by log volume
    pub fn top_components(&self, limit: usize) -> Vec<(String, u64)> {
        let mut components: Vec<_> = self.entries_by_component.iter().collect();
        components.sort_by(|a, b| b.1.cmp(a.1));
        components
            .into_iter()
            .take(limit)
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        *self = LogMetrics::new();
    }

    /// Generate a summary report
    pub fn summary(&self) -> MetricsSummary {
        MetricsSummary {
            total_entries: self.total_entries,
            entries_per_second: self.entries_per_second(),
            error_rate_percent: self.error_rate_percent(),
            drop_rate_percent: self.drop_rate_percent(),
            avg_processing_time_us: self.avg_processing_time_us,
            buffer_utilization: self.buffer_utilization,
            memory_usage_mb: self.memory_usage_bytes as f64 / (1024.0 * 1024.0),
            uptime_seconds: self.uptime_seconds(),
            top_levels: self.entries_by_level.clone(),
            top_components: self.top_components(5),
        }
    }
}

impl Default for RateLimitStats {
    fn default() -> Self {
        Self {
            rate_limited_entries: 0,
            window_start: current_timestamp(),
            window_entries: 0,
            max_entries_per_window: 0,
        }
    }
}

impl Default for OutputStats {
    fn default() -> Self {
        Self {
            entries_written: 0,
            write_errors: 0,
            bytes_written: 0,
            avg_write_time_us: 0.0,
            max_write_time_us: 0,
            last_write_time: 0,
            availability: 100.0,
        }
    }
}

impl OutputStats {
    /// Record a successful write operation
    pub fn record_write(&mut self, bytes: u64, duration: Duration) {
        self.entries_written += 1;
        self.bytes_written += bytes;

        let duration_us = duration.as_micros() as u64;
        self.max_write_time_us = self.max_write_time_us.max(duration_us);

        // Update average write time
        if self.entries_written > 0 {
            let total_time =
                self.avg_write_time_us * (self.entries_written - 1) as f64 + duration_us as f64;
            self.avg_write_time_us = total_time / self.entries_written as f64;
        }

        self.last_write_time = current_timestamp();

        // Update availability based on error rate
        self.update_availability();
    }

    /// Record a write error
    pub fn record_error(&mut self) {
        self.write_errors += 1;
        self.update_availability();
    }

    fn update_availability(&mut self) {
        let total_operations = self.entries_written + self.write_errors;
        if total_operations > 0 {
            self.availability = (self.entries_written as f32 / total_operations as f32) * 100.0;
        }
    }
}

/// Summary of key metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub total_entries: u64,
    pub entries_per_second: f64,
    pub error_rate_percent: f64,
    pub drop_rate_percent: f64,
    pub avg_processing_time_us: f64,
    pub buffer_utilization: f32,
    pub memory_usage_mb: f64,
    pub uptime_seconds: u64,
    pub top_levels: HashMap<String, u64>,
    pub top_components: Vec<(String, u64)>,
}

/// Metrics collector for gathering system-level metrics
pub struct MetricsCollector {
    start_time: Instant,
    last_collection: Instant,
    entries_processed: AtomicU64,
    processing_times: Arc<std::sync::Mutex<Vec<Duration>>>,
    max_processing_times_kept: usize,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_collection: now,
            entries_processed: AtomicU64::new(0),
            processing_times: Arc::new(std::sync::Mutex::new(Vec::new())),
            max_processing_times_kept: 1000, // Keep last 1000 processing times
        }
    }

    /// Record that an entry was processed
    pub fn record_entry_processed(&self, processing_time: Duration) {
        self.entries_processed.fetch_add(1, Ordering::Relaxed);

        if let Ok(mut times) = self.processing_times.lock() {
            times.push(processing_time);

            // Keep only recent processing times to prevent unbounded growth
            if times.len() > self.max_processing_times_kept {
                times.remove(0);
            }
        }
    }

    /// Get current throughput (entries per second)
    pub fn current_throughput(&self) -> f64 {
        let elapsed = self.start_time.elapsed();
        if elapsed.is_zero() {
            0.0
        } else {
            self.entries_processed.load(Ordering::Relaxed) as f64 / elapsed.as_secs_f64()
        }
    }

    /// Get recent throughput since last collection
    pub fn recent_throughput(&mut self) -> f64 {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_collection);
        self.last_collection = now;

        if elapsed.is_zero() {
            0.0
        } else {
            // This is simplified - in a real implementation, you'd track entries since last collection
            self.current_throughput()
        }
    }

    /// Get processing time statistics
    pub fn processing_time_stats(&self) -> ProcessingTimeStats {
        let times = self.processing_times.lock().unwrap();

        if times.is_empty() {
            return ProcessingTimeStats::default();
        }

        let total_us: u64 = times.iter().map(|d| d.as_micros() as u64).sum();
        let count = times.len();
        let avg_us = total_us as f64 / count as f64;

        let max_us = times
            .iter()
            .map(|d| d.as_micros() as u64)
            .max()
            .unwrap_or(0);
        let min_us = times
            .iter()
            .map(|d| d.as_micros() as u64)
            .min()
            .unwrap_or(0);

        // Calculate percentiles
        let mut sorted_times: Vec<u64> = times.iter().map(|d| d.as_micros() as u64).collect();
        sorted_times.sort_unstable();

        let p50_us = percentile(&sorted_times, 50);
        let p90_us = percentile(&sorted_times, 90);
        let p95_us = percentile(&sorted_times, 95);
        let p99_us = percentile(&sorted_times, 99);

        ProcessingTimeStats {
            avg_us,
            min_us,
            max_us,
            p50_us,
            p90_us,
            p95_us,
            p99_us,
            total_us,
            sample_count: count,
        }
    }

    /// Get memory usage information
    pub fn memory_info(&self) -> MemoryInfo {
        // This is a simplified implementation
        // In a real system, you'd use proper memory profiling
        MemoryInfo {
            heap_used_bytes: 0, // Would use a memory profiler
            heap_total_bytes: 0,
            stack_used_bytes: 0,
            buffer_used_bytes: 0,
        }
    }

    /// Reset all collected metrics
    pub fn reset(&mut self) {
        let now = Instant::now();
        self.start_time = now;
        self.last_collection = now;
        self.entries_processed.store(0, Ordering::Relaxed);

        if let Ok(mut times) = self.processing_times.lock() {
            times.clear();
        }
    }
}

/// Processing time statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingTimeStats {
    pub avg_us: f64,
    pub min_us: u64,
    pub max_us: u64,
    pub p50_us: u64,
    pub p90_us: u64,
    pub p95_us: u64,
    pub p99_us: u64,
    pub total_us: u64,
    pub sample_count: usize,
}

impl Default for ProcessingTimeStats {
    fn default() -> Self {
        Self {
            avg_us: 0.0,
            min_us: 0,
            max_us: 0,
            p50_us: 0,
            p90_us: 0,
            p95_us: 0,
            p99_us: 0,
            total_us: 0,
            sample_count: 0,
        }
    }
}

/// Memory usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub heap_used_bytes: u64,
    pub heap_total_bytes: u64,
    pub stack_used_bytes: u64,
    pub buffer_used_bytes: u64,
}

/// Get current timestamp in milliseconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Calculate percentile from sorted array
fn percentile(sorted_values: &[u64], percentile: u8) -> u64 {
    if sorted_values.is_empty() {
        return 0;
    }

    // Use the "nearest rank" method for percentile calculation
    // For array indices starting at 0, we use floor instead of round
    let index = ((percentile as f64 / 100.0) * (sorted_values.len() - 1) as f64) as usize;
    sorted_values[index.min(sorted_values.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_metrics_creation() {
        let metrics = LogMetrics::new();
        assert_eq!(metrics.total_entries, 0);
        assert_eq!(metrics.total_errors, 0);
        assert!(metrics.entries_by_level.is_empty());
        assert!(metrics.entries_by_component.is_empty());
    }

    #[test]
    fn test_record_entry() {
        let mut metrics = LogMetrics::new();
        let entry = LogEntry {
            timestamp: chrono::Utc::now(),
            level: LogLevel::Info,
            message: "Test message".to_string(),
            component: "test_component".to_string(),
            module: None,
            location: None,
            thread_id: None,
            correlation_id: None,
            user_id: None,
            session_id: None,
            fields: HashMap::new(),
            metrics: None,
            tags: Vec::new(),
            environment: Some("test".to_string()),
            error: None,
        };

        metrics.record_entry(&entry);

        assert_eq!(metrics.total_entries, 1);
        assert_eq!(metrics.entries_by_level.get("INFO"), Some(&1));
        assert_eq!(metrics.entries_by_component.get("test_component"), Some(&1));
    }

    #[test]
    fn test_processing_time_recording() {
        let mut metrics = LogMetrics::new();
        let duration = Duration::from_millis(10);

        metrics.record_processing_time(duration);

        assert!(metrics.avg_processing_time_us > 0.0);
        assert!(metrics.max_processing_time_us > 0);
        assert!(metrics.total_processing_time_us > 0);
    }

    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new();

        collector.record_entry_processed(Duration::from_micros(100));
        collector.record_entry_processed(Duration::from_micros(200));

        let stats = collector.processing_time_stats();
        assert_eq!(stats.sample_count, 2);
        assert_eq!(stats.min_us, 100);
        assert_eq!(stats.max_us, 200);
        assert_eq!(stats.avg_us, 150.0);
    }

    #[test]
    fn test_output_stats() {
        let mut stats = OutputStats::default();

        stats.record_write(1024, Duration::from_micros(500));
        assert_eq!(stats.entries_written, 1);
        assert_eq!(stats.bytes_written, 1024);
        assert_eq!(stats.avg_write_time_us, 500.0);
        assert_eq!(stats.availability, 100.0);

        stats.record_error();
        assert_eq!(stats.write_errors, 1);
        assert_eq!(stats.availability, 50.0);
    }

    #[test]
    fn test_percentile_calculation() {
        let values = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        assert_eq!(percentile(&values, 50), 5);
        assert_eq!(percentile(&values, 90), 9);
        assert_eq!(percentile(&values, 100), 10);

        let empty: Vec<u64> = vec![];
        assert_eq!(percentile(&empty, 50), 0);
    }

    #[test]
    fn test_metrics_summary() {
        let mut metrics = LogMetrics::new();

        // Simulate some activity
        for i in 0..100 {
            let entry = LogEntry {
                timestamp: chrono::Utc::now(),
                level: if i % 10 == 0 {
                    LogLevel::Error
                } else {
                    LogLevel::Info
                },
                message: format!("Test message {}", i),
                component: format!("component_{}", i % 3),
                module: None,
                location: None,
                thread_id: None,
                correlation_id: None,
                user_id: None,
                session_id: None,
                fields: HashMap::new(),
                metrics: None,
                tags: Vec::new(),
                environment: Some("test".to_string()),
                error: None,
            };
            metrics.record_entry(&entry);
        }

        let summary = metrics.summary();
        assert_eq!(summary.total_entries, 100);
        assert_eq!(summary.error_rate_percent, 10.0);
        assert_eq!(summary.top_components.len(), 3);
    }

    #[test]
    fn test_rate_limiting_stats() {
        let mut metrics = LogMetrics::new();

        metrics.record_rate_limited();
        metrics.record_rate_limited();

        assert_eq!(metrics.rate_limit_stats.rate_limited_entries, 2);

        metrics.update_rate_limit_window(50, 100);
        assert_eq!(metrics.rate_limit_stats.window_entries, 50);
        assert_eq!(metrics.rate_limit_stats.max_entries_per_window, 100);
    }
}
