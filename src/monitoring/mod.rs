use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::time::interval;
use crate::error::DevKitError;

pub mod metrics;
pub mod health;
pub mod alerts;

pub use metrics::*;
pub use health::*;
pub use alerts::*;

/// Central monitoring system for devkit
#[derive(Clone)]
pub struct MonitoringSystem {
    metrics: Arc<MetricsCollector>,
    health: Arc<HealthChecker>,
    alerts: Arc<AlertManager>,
}

impl MonitoringSystem {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(MetricsCollector::new()),
            health: Arc::new(HealthChecker::new()),
            alerts: Arc::new(AlertManager::new()),
        }
    }

    /// Start the monitoring system
    pub async fn start(&self) -> Result<(), DevKitError> {
        let metrics = Arc::clone(&self.metrics);
        let health = Arc::clone(&self.health);
        let alerts = Arc::clone(&self.alerts);

        // Start metrics collection
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                if let Err(e) = metrics.collect_system_metrics().await {
                    eprintln!("Failed to collect metrics: {}", e);
                }
            }
        });

        // Start health checks
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                if let Err(e) = health.run_health_checks().await {
                    eprintln!("Health check failed: {}", e);
                }
            }
        });

        // Start alert processing
        tokio::spawn(async move {
            alerts.process_alerts().await;
        });

        Ok(())
    }

    /// Record a custom metric
    pub fn record_metric(&self, name: &str, value: f64, tags: HashMap<String, String>) {
        self.metrics.record(name, value, tags);
    }

    /// Get current system health
    pub async fn get_health_status(&self) -> HealthStatus {
        self.health.get_status().await
    }

    /// Get metrics summary
    pub fn get_metrics_summary(&self) -> MetricsSummary {
        self.metrics.get_summary()
    }
}

/// Metrics collector for performance and usage data
pub struct MetricsCollector {
    metrics: Arc<Mutex<HashMap<String, MetricValue>>>,
    start_time: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    pub value: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub tags: HashMap<String, String>,
    pub count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub uptime: Duration,
    pub metrics_count: usize,
    pub top_metrics: Vec<(String, MetricValue)>,
    pub system_health: String,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }

    pub fn record(&self, name: &str, value: f64, tags: HashMap<String, String>) {
        let mut metrics = self.metrics.lock().unwrap();
        let entry = metrics.entry(name.to_string()).or_insert_with(|| MetricValue {
            value,
            timestamp: chrono::Utc::now(),
            tags: tags.clone(),
            count: 0,
            sum: 0.0,
            min: value,
            max: value,
        });

        entry.count += 1;
        entry.sum += value;
        entry.value = value;
        entry.timestamp = chrono::Utc::now();
        entry.min = entry.min.min(value);
        entry.max = entry.max.max(value);
        entry.tags.extend(tags);
    }

    pub async fn collect_system_metrics(&self) -> Result<(), DevKitError> {
        use std::process::Command;

        // Memory usage
        if let Ok(output) = Command::new("ps")
            .args(&["-o", "rss", "-p", &std::process::id().to_string()])
            .output()
        {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                if let Some(line) = output_str.lines().nth(1) {
                    if let Ok(memory_kb) = line.trim().parse::<f64>() {
                        let memory_mb = memory_kb / 1024.0;
                        self.record("system.memory_usage_mb", memory_mb, HashMap::new());
                    }
                }
            }
        }

        // CPU usage (simplified - in production you'd want a proper CPU monitoring library)
        let cpu_usage = self.get_cpu_usage().await.unwrap_or(0.0);
        self.record("system.cpu_usage_percent", cpu_usage, HashMap::new());

        // Uptime
        let uptime = self.start_time.elapsed().as_secs() as f64;
        self.record("system.uptime_seconds", uptime, HashMap::new());

        Ok(())
    }

    async fn get_cpu_usage(&self) -> Option<f64> {
        // Simplified CPU usage calculation
        // In production, use a proper system monitoring library like `sysinfo`
        Some(0.0) // Placeholder
    }

    pub fn get_summary(&self) -> MetricsSummary {
        let metrics = self.metrics.lock().unwrap();
        let uptime = self.start_time.elapsed();

        let mut top_metrics: Vec<_> = metrics.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        top_metrics.sort_by(|a, b| b.1.count.cmp(&a.1.count));
        top_metrics.truncate(10);

        MetricsSummary {
            uptime,
            metrics_count: metrics.len(),
            top_metrics,
            system_health: "OK".to_string(), // This would be determined by actual health checks
        }
    }
}