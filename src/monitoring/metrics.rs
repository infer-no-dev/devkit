// Stub metrics module
pub struct MetricsCollector;
impl MetricsCollector {
    pub fn new() -> Self { Self }
    pub async fn collect_system_metrics(&self) -> Result<(), crate::error::DevKitError> { Ok(()) }
    pub fn record(&self, _name: &str, _value: f64, _tags: std::collections::HashMap<String, String>) {}
    pub fn get_summary(&self) -> super::MetricsSummary {
        super::MetricsSummary {
            uptime: std::time::Duration::from_secs(0),
            metrics_count: 0,
            top_metrics: vec![],
            system_health: "OK".to_string(),
        }
    }
}