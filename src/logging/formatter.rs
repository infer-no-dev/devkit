//! Log formatting implementations for different output formats.

use crate::logging::{LogEntry, LogFormat, LoggingError};
use colored::*;
use serde_json;

/// Trait for log formatters
pub trait LogFormatter: std::fmt::Debug {
    /// Format a log entry into a string
    fn format(&self, entry: &LogEntry) -> Result<String, LoggingError>;

    /// Get the preferred format type
    fn format_type(&self) -> LogFormat;

    /// Whether this formatter supports colored output
    fn supports_color(&self) -> bool {
        false
    }
}

/// JSON formatter for structured logging
#[derive(Debug)]
pub struct JsonFormatter {
    pretty: bool,
    include_timestamp: bool,
    include_level: bool,
    include_component: bool,
}

impl JsonFormatter {
    pub fn new() -> Self {
        Self {
            pretty: false,
            include_timestamp: true,
            include_level: true,
            include_component: true,
        }
    }

    pub fn pretty() -> Self {
        Self {
            pretty: true,
            include_timestamp: true,
            include_level: true,
            include_component: true,
        }
    }

    pub fn with_options(
        pretty: bool,
        include_timestamp: bool,
        include_level: bool,
        include_component: bool,
    ) -> Self {
        Self {
            pretty,
            include_timestamp,
            include_level,
            include_component,
        }
    }
}

impl LogFormatter for JsonFormatter {
    fn format(&self, entry: &LogEntry) -> Result<String, LoggingError> {
        let mut json_entry = serde_json::Map::new();

        if self.include_timestamp {
            json_entry.insert(
                "timestamp".to_string(),
                serde_json::Value::String(entry.timestamp.to_rfc3339()),
            );
        }

        if self.include_level {
            json_entry.insert(
                "level".to_string(),
                serde_json::Value::String(entry.level.to_string()),
            );
        }

        json_entry.insert(
            "message".to_string(),
            serde_json::Value::String(entry.message.clone()),
        );

        if self.include_component {
            json_entry.insert(
                "component".to_string(),
                serde_json::Value::String(entry.component.clone()),
            );
        }

        // Add optional fields
        if let Some(module) = &entry.module {
            json_entry.insert(
                "module".to_string(),
                serde_json::Value::String(module.clone()),
            );
        }

        if let Some(location) = &entry.location {
            let mut location_map = serde_json::Map::new();
            location_map.insert(
                "file".to_string(),
                serde_json::Value::String(location.file.clone()),
            );
            location_map.insert(
                "line".to_string(),
                serde_json::Value::Number(location.line.into()),
            );
            if let Some(column) = location.column {
                location_map.insert(
                    "column".to_string(),
                    serde_json::Value::Number(column.into()),
                );
            }
            json_entry.insert(
                "location".to_string(),
                serde_json::Value::Object(location_map),
            );
        }

        if let Some(thread_id) = &entry.thread_id {
            json_entry.insert(
                "thread_id".to_string(),
                serde_json::Value::String(thread_id.clone()),
            );
        }

        if let Some(correlation_id) = &entry.correlation_id {
            json_entry.insert(
                "correlation_id".to_string(),
                serde_json::Value::String(correlation_id.clone()),
            );
        }

        if let Some(user_id) = &entry.user_id {
            json_entry.insert(
                "user_id".to_string(),
                serde_json::Value::String(user_id.clone()),
            );
        }

        if let Some(session_id) = &entry.session_id {
            json_entry.insert(
                "session_id".to_string(),
                serde_json::Value::String(session_id.clone()),
            );
        }

        if let Some(environment) = &entry.environment {
            json_entry.insert(
                "environment".to_string(),
                serde_json::Value::String(environment.clone()),
            );
        }

        // Add custom fields
        for (key, value) in &entry.fields {
            json_entry.insert(key.clone(), value.clone());
        }

        // Add metrics if present
        if let Some(metrics) = &entry.metrics {
            let mut metrics_map = serde_json::Map::new();

            if let Some(duration) = metrics.duration_ms {
                metrics_map.insert(
                    "duration_ms".to_string(),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(duration)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    ),
                );
            }

            if let Some(memory) = metrics.memory_usage_kb {
                metrics_map.insert(
                    "memory_usage_kb".to_string(),
                    serde_json::Value::Number(memory.into()),
                );
            }

            if let Some(cpu) = metrics.cpu_usage_percent {
                metrics_map.insert(
                    "cpu_usage_percent".to_string(),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(cpu)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    ),
                );
            }

            for (key, value) in &metrics.custom_metrics {
                metrics_map.insert(
                    key.clone(),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(*value)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    ),
                );
            }

            if !metrics_map.is_empty() {
                json_entry.insert(
                    "metrics".to_string(),
                    serde_json::Value::Object(metrics_map),
                );
            }
        }

        // Add tags if present
        if !entry.tags.is_empty() {
            json_entry.insert(
                "tags".to_string(),
                serde_json::Value::Array(
                    entry
                        .tags
                        .iter()
                        .map(|tag| serde_json::Value::String(tag.clone()))
                        .collect(),
                ),
            );
        }

        // Add error information if present
        if let Some(error) = &entry.error {
            let mut error_map = serde_json::Map::new();
            error_map.insert(
                "type".to_string(),
                serde_json::Value::String(error.error_type.clone()),
            );
            error_map.insert(
                "message".to_string(),
                serde_json::Value::String(error.error_message.clone()),
            );

            if let Some(code) = &error.error_code {
                error_map.insert("code".to_string(), serde_json::Value::String(code.clone()));
            }

            if let Some(stack_trace) = &error.stack_trace {
                error_map.insert(
                    "stack_trace".to_string(),
                    serde_json::Value::String(stack_trace.clone()),
                );
            }

            if let Some(caused_by) = &error.caused_by {
                let mut caused_by_map = serde_json::Map::new();
                caused_by_map.insert(
                    "type".to_string(),
                    serde_json::Value::String(caused_by.error_type.clone()),
                );
                caused_by_map.insert(
                    "message".to_string(),
                    serde_json::Value::String(caused_by.error_message.clone()),
                );
                error_map.insert(
                    "caused_by".to_string(),
                    serde_json::Value::Object(caused_by_map),
                );
            }

            json_entry.insert("error".to_string(), serde_json::Value::Object(error_map));
        }

        let json_value = serde_json::Value::Object(json_entry);

        if self.pretty {
            Ok(serde_json::to_string_pretty(&json_value)?)
        } else {
            Ok(serde_json::to_string(&json_value)?)
        }
    }

    fn format_type(&self) -> LogFormat {
        LogFormat::Json
    }
}

/// Human-readable text formatter
#[derive(Debug)]
pub struct TextFormatter {
    colored: bool,
    show_timestamp: bool,
    show_level: bool,
    show_component: bool,
    show_location: bool,
    show_thread: bool,
    timestamp_format: String,
}

impl TextFormatter {
    pub fn new() -> Self {
        Self {
            colored: true,
            show_timestamp: true,
            show_level: true,
            show_component: true,
            show_location: false,
            show_thread: false,
            timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        }
    }

    pub fn colored(mut self, colored: bool) -> Self {
        self.colored = colored;
        self
    }

    pub fn show_location(mut self, show: bool) -> Self {
        self.show_location = show;
        self
    }

    pub fn show_thread(mut self, show: bool) -> Self {
        self.show_thread = show;
        self
    }

    pub fn timestamp_format(mut self, format: String) -> Self {
        self.timestamp_format = format;
        self
    }

    fn format_level(&self, level: &crate::logging::LogLevel) -> String {
        if !self.colored {
            return format!("[{}]", level);
        }

        match level {
            crate::logging::LogLevel::Error => format!("[{}]", "ERROR".red().bold()),
            crate::logging::LogLevel::Warn => format!("[{}]", "WARN".yellow().bold()),
            crate::logging::LogLevel::Info => format!("[{}]", "INFO".green()),
            crate::logging::LogLevel::Debug => format!("[{}]", "DEBUG".blue()),
            crate::logging::LogLevel::Trace => format!("[{}]", "TRACE".magenta()),
        }
    }

    fn format_component(&self, component: &str) -> String {
        if self.colored {
            component.cyan().to_string()
        } else {
            component.to_string()
        }
    }

    fn format_location(&self, location: &crate::logging::LogLocation) -> String {
        if let Some(column) = location.column {
            format!("{}:{}:{}", location.file, location.line, column)
        } else {
            format!("{}:{}", location.file, location.line)
        }
    }
}

impl LogFormatter for TextFormatter {
    fn format(&self, entry: &LogEntry) -> Result<String, LoggingError> {
        let mut parts = Vec::new();

        // Timestamp
        if self.show_timestamp {
            let timestamp = entry.timestamp.format(&self.timestamp_format).to_string();
            if self.colored {
                parts.push(timestamp.bright_black().to_string());
            } else {
                parts.push(timestamp);
            }
        }

        // Level
        if self.show_level {
            parts.push(self.format_level(&entry.level));
        }

        // Component
        if self.show_component {
            parts.push(self.format_component(&entry.component));
        }

        // Thread ID
        if self.show_thread {
            if let Some(thread_id) = &entry.thread_id {
                let thread_str = format!("[{}]", thread_id);
                if self.colored {
                    parts.push(thread_str.bright_black().to_string());
                } else {
                    parts.push(thread_str);
                }
            }
        }

        // Message
        let message = if self.colored && entry.level == crate::logging::LogLevel::Error {
            entry.message.red().to_string()
        } else if self.colored && entry.level == crate::logging::LogLevel::Warn {
            entry.message.yellow().to_string()
        } else {
            entry.message.clone()
        };
        parts.push(message);

        // Location
        if self.show_location {
            if let Some(location) = &entry.location {
                let location_str = format!("({})", self.format_location(location));
                if self.colored {
                    parts.push(location_str.bright_black().to_string());
                } else {
                    parts.push(location_str);
                }
            }
        }

        // Correlation ID
        if let Some(correlation_id) = &entry.correlation_id {
            let correlation_str = format!("[corr:{}]", correlation_id);
            if self.colored {
                parts.push(correlation_str.bright_black().to_string());
            } else {
                parts.push(correlation_str);
            }
        }

        // Add structured fields if present
        if !entry.fields.is_empty() {
            let fields_str = entry
                .fields
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(" ");

            if self.colored {
                parts.push(format!("{{{}}}", fields_str.bright_black()));
            } else {
                parts.push(format!("{{{}}}", fields_str));
            }
        }

        // Add metrics if present
        if let Some(metrics) = &entry.metrics {
            let mut metric_parts = Vec::new();

            if let Some(duration) = metrics.duration_ms {
                metric_parts.push(format!("dur:{:.2}ms", duration));
            }

            if let Some(memory) = metrics.memory_usage_kb {
                metric_parts.push(format!("mem:{}kb", memory));
            }

            if let Some(cpu) = metrics.cpu_usage_percent {
                metric_parts.push(format!("cpu:{:.1}%", cpu));
            }

            if !metric_parts.is_empty() {
                let metrics_str = metric_parts.join(" ");
                if self.colored {
                    parts.push(format!("[{}]", metrics_str.bright_green()));
                } else {
                    parts.push(format!("[{}]", metrics_str));
                }
            }
        }

        // Add error information if present
        if let Some(error) = &entry.error {
            let error_str = format!("error({}): {}", error.error_type, error.error_message);
            if self.colored {
                parts.push(error_str.red().bold().to_string());
            } else {
                parts.push(error_str);
            }
        }

        Ok(parts.join(" "))
    }

    fn format_type(&self) -> LogFormat {
        LogFormat::Text
    }

    fn supports_color(&self) -> bool {
        self.colored
    }
}

/// Structured key-value formatter
#[derive(Debug)]
pub struct StructuredFormatter {
    separator: String,
    key_value_separator: String,
    quote_values: bool,
    escape_quotes: bool,
}

impl StructuredFormatter {
    pub fn new() -> Self {
        Self {
            separator: " ".to_string(),
            key_value_separator: "=".to_string(),
            quote_values: true,
            escape_quotes: true,
        }
    }

    pub fn with_separator(mut self, separator: String) -> Self {
        self.separator = separator;
        self
    }

    pub fn with_key_value_separator(mut self, separator: String) -> Self {
        self.key_value_separator = separator;
        self
    }

    pub fn quote_values(mut self, quote: bool) -> Self {
        self.quote_values = quote;
        self
    }

    fn format_value(&self, value: &serde_json::Value) -> String {
        let value_str = match value {
            serde_json::Value::String(s) => {
                if self.escape_quotes {
                    s.replace('"', "\\\"")
                } else {
                    s.clone()
                }
            }
            other => other.to_string(),
        };

        if self.quote_values && matches!(value, serde_json::Value::String(_)) {
            format!("\"{}\"", value_str)
        } else {
            value_str
        }
    }

    fn format_key_value(&self, key: &str, value: &serde_json::Value) -> String {
        format!(
            "{}{}{}",
            key,
            self.key_value_separator,
            self.format_value(value)
        )
    }
}

impl LogFormatter for StructuredFormatter {
    fn format(&self, entry: &LogEntry) -> Result<String, LoggingError> {
        let mut pairs = Vec::new();

        // Core fields
        pairs.push(self.format_key_value(
            "timestamp",
            &serde_json::Value::String(entry.timestamp.to_rfc3339()),
        ));
        pairs.push(
            self.format_key_value("level", &serde_json::Value::String(entry.level.to_string())),
        );
        pairs.push(
            self.format_key_value("message", &serde_json::Value::String(entry.message.clone())),
        );
        pairs.push(self.format_key_value(
            "component",
            &serde_json::Value::String(entry.component.clone()),
        ));

        // Optional fields
        if let Some(module) = &entry.module {
            pairs.push(self.format_key_value("module", &serde_json::Value::String(module.clone())));
        }

        if let Some(thread_id) = &entry.thread_id {
            pairs.push(
                self.format_key_value("thread_id", &serde_json::Value::String(thread_id.clone())),
            );
        }

        if let Some(correlation_id) = &entry.correlation_id {
            pairs.push(self.format_key_value(
                "correlation_id",
                &serde_json::Value::String(correlation_id.clone()),
            ));
        }

        if let Some(user_id) = &entry.user_id {
            pairs.push(
                self.format_key_value("user_id", &serde_json::Value::String(user_id.clone())),
            );
        }

        if let Some(session_id) = &entry.session_id {
            pairs.push(
                self.format_key_value("session_id", &serde_json::Value::String(session_id.clone())),
            );
        }

        if let Some(environment) = &entry.environment {
            pairs.push(self.format_key_value(
                "environment",
                &serde_json::Value::String(environment.clone()),
            ));
        }

        if let Some(location) = &entry.location {
            pairs.push(
                self.format_key_value("file", &serde_json::Value::String(location.file.clone())),
            );
            pairs.push(
                self.format_key_value("line", &serde_json::Value::Number(location.line.into())),
            );
            if let Some(column) = location.column {
                pairs.push(
                    self.format_key_value("column", &serde_json::Value::Number(column.into())),
                );
            }
        }

        // Custom fields
        for (key, value) in &entry.fields {
            pairs.push(self.format_key_value(key, value));
        }

        // Metrics
        if let Some(metrics) = &entry.metrics {
            if let Some(duration) = metrics.duration_ms {
                pairs.push(
                    self.format_key_value(
                        "duration_ms",
                        &serde_json::Value::Number(
                            serde_json::Number::from_f64(duration)
                                .unwrap_or_else(|| serde_json::Number::from(0)),
                        ),
                    ),
                );
            }

            if let Some(memory) = metrics.memory_usage_kb {
                pairs.push(self.format_key_value(
                    "memory_usage_kb",
                    &serde_json::Value::Number(memory.into()),
                ));
            }

            if let Some(cpu) = metrics.cpu_usage_percent {
                pairs.push(
                    self.format_key_value(
                        "cpu_usage_percent",
                        &serde_json::Value::Number(
                            serde_json::Number::from_f64(cpu)
                                .unwrap_or_else(|| serde_json::Number::from(0)),
                        ),
                    ),
                );
            }

            for (key, value) in &metrics.custom_metrics {
                pairs.push(
                    self.format_key_value(
                        &format!("metric_{}", key),
                        &serde_json::Value::Number(
                            serde_json::Number::from_f64(*value)
                                .unwrap_or_else(|| serde_json::Number::from(0)),
                        ),
                    ),
                );
            }
        }

        // Tags
        if !entry.tags.is_empty() {
            pairs.push(
                self.format_key_value("tags", &serde_json::Value::String(entry.tags.join(","))),
            );
        }

        // Error information
        if let Some(error) = &entry.error {
            pairs.push(self.format_key_value(
                "error_type",
                &serde_json::Value::String(error.error_type.clone()),
            ));
            pairs.push(self.format_key_value(
                "error_message",
                &serde_json::Value::String(error.error_message.clone()),
            ));

            if let Some(code) = &error.error_code {
                pairs.push(
                    self.format_key_value("error_code", &serde_json::Value::String(code.clone())),
                );
            }
        }

        Ok(pairs.join(&self.separator))
    }

    fn format_type(&self) -> LogFormat {
        LogFormat::Structured
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::{LogEntry, LogLevel, LogLocation};
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_entry() -> LogEntry {
        LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            message: "Test message".to_string(),
            component: "test_component".to_string(),
            module: Some("test::module".to_string()),
            location: Some(LogLocation {
                file: "test.rs".to_string(),
                line: 42,
                column: Some(10),
            }),
            thread_id: Some("thread-1".to_string()),
            correlation_id: Some("corr-123".to_string()),
            user_id: Some("user-456".to_string()),
            session_id: Some("session-789".to_string()),
            fields: {
                let mut fields = HashMap::new();
                fields.insert(
                    "key1".to_string(),
                    serde_json::Value::String("value1".to_string()),
                );
                fields.insert("key2".to_string(), serde_json::Value::Number(42.into()));
                fields
            },
            metrics: None,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            environment: Some("test".to_string()),
            error: None,
        }
    }

    #[test]
    fn test_json_formatter() {
        let formatter = JsonFormatter::new();
        let entry = create_test_entry();

        let result = formatter.format(&entry).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["level"], "INFO");
        assert_eq!(parsed["message"], "Test message");
        assert_eq!(parsed["component"], "test_component");
    }

    #[test]
    fn test_text_formatter() {
        let formatter = TextFormatter::new().colored(false);
        let entry = create_test_entry();

        let result = formatter.format(&entry).unwrap();

        assert!(result.contains("INFO"));
        assert!(result.contains("Test message"));
        assert!(result.contains("test_component"));
    }

    #[test]
    fn test_structured_formatter() {
        let formatter = StructuredFormatter::new();
        let entry = create_test_entry();

        let result = formatter.format(&entry).unwrap();

        // The structured formatter quotes string values
        assert!(result.contains("level=\"INFO\""));
        assert!(result.contains("message=\"Test message\""));
        assert!(result.contains("component=\"test_component\""));
    }
}
