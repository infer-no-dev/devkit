//! Enhanced UI error handling system with graceful degradation and user-friendly error display.

// Temporarily disable import until error module is properly configured
// use crate::error::{DevKitError, DevKitResult, ErrorHandler, RecoveryStrategy};
use crate::agents::AgentError as DevKitError; // Temporary stub
 // Temporary stub

// Temporary stub types
#[derive(Debug, Default)]
pub struct ErrorHandler {
    placeholder: String,
}

#[derive(Debug)]
pub enum RecoveryStrategy {
    Ignore,
    Retry,
    Exit,
}

impl ErrorHandler {
    pub async fn handle_error(&self, _error: &DevKitError) -> RecoveryStrategy {
        RecoveryStrategy::Ignore
    }
}
use crate::ui::notifications::{Notification, NotificationType};
use crate::ui::themes::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{error, warn, info};

/// UI Error handler with enhanced user experience
#[derive(Debug)]
pub struct UIErrorHandler {
    error_handler: ErrorHandler,
    error_display_buffer: Vec<UIError>,
    last_error: Option<(UIError, Instant)>,
    error_notification_sender: mpsc::UnboundedSender<Notification>,
    max_buffer_size: usize,
    auto_dismiss_duration: Duration,
}

/// Enhanced UI error with display information
#[derive(Debug, Clone)]
pub struct UIError {
    pub error_message: String,
    pub severity: ErrorSeverity,
    pub display_message: String,
    pub technical_details: Option<String>,
    pub recovery_suggestion: Option<String>,
    pub timestamp: Instant,
    pub correlation_id: Option<String>,
}

/// Error severity levels for UI display
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorSeverity {
    /// Critical system failure - requires immediate attention
    Critical,
    /// Error that prevents operation completion
    Error,
    /// Warning that may impact functionality
    Warning,
    /// Informational message about recoverable issues
    Info,
}

impl UIErrorHandler {
    /// Create a new UI error handler
    pub fn new(notification_sender: mpsc::UnboundedSender<Notification>) -> Self {
        Self {
            error_handler: ErrorHandler::default(),
            error_display_buffer: Vec::new(),
            last_error: None,
            error_notification_sender: notification_sender,
            max_buffer_size: 50,
            auto_dismiss_duration: Duration::from_secs(10),
        }
    }

    /// Handle an error with appropriate UI response
    pub async fn handle_error(&mut self, error: DevKitError) -> RecoveryStrategy {
        let ui_error = self.convert_to_ui_error(&error);
        let severity = ui_error.severity;
        
        // Log the error appropriately
        match severity {
            ErrorSeverity::Critical => error!("Critical UI error: {}", ui_error.display_message),
            ErrorSeverity::Error => error!("UI error: {}", ui_error.display_message),
            ErrorSeverity::Warning => warn!("UI warning: {}", ui_error.display_message),
            ErrorSeverity::Info => info!("UI info: {}", ui_error.display_message),
        }

        // Add to error buffer
        self.add_to_buffer(ui_error.clone());

        // Update last error for immediate display
        self.last_error = Some((ui_error.clone(), Instant::now()));

        // Send notification
        self.send_error_notification(&ui_error).await;

        // Get recovery strategy
        self.error_handler.handle_error(&error).await
    }

    /// Convert DevKitError to UIError with enhanced display information
    fn convert_to_ui_error(&self, error: &DevKitError) -> UIError {
        // Simple stub implementation - in a real scenario, this would handle different error types
        let (severity, display_message, recovery_suggestion) = (
            ErrorSeverity::Error,
            "An error occurred in the agent system".to_string(),
            Some("Try restarting or using alternative commands".to_string()),
        );

        UIError {
            error_message: error.to_string(),
            severity,
            display_message,
            technical_details: Some(format!("{:?}", error)),
            recovery_suggestion,
            timestamp: Instant::now(),
            correlation_id: None,
        }
    }

    /// Add error to display buffer
    fn add_to_buffer(&mut self, error: UIError) {
        self.error_display_buffer.push(error);
        
        // Keep buffer size manageable
        if self.error_display_buffer.len() > self.max_buffer_size {
            self.error_display_buffer.remove(0);
        }
    }

    /// Send error notification
    async fn send_error_notification(&self, ui_error: &UIError) {
        let notification_type = match ui_error.severity {
            ErrorSeverity::Critical => NotificationType::Error,
            ErrorSeverity::Error => NotificationType::Error,
            ErrorSeverity::Warning => NotificationType::Warning,
            ErrorSeverity::Info => NotificationType::Info,
        };

        let title = match ui_error.severity {
            ErrorSeverity::Critical => "🚨 Critical Error",
            ErrorSeverity::Error => "❌ Error",
            ErrorSeverity::Warning => "⚠️ Warning",
            ErrorSeverity::Info => "ℹ️ Info",
        };

        let mut content = ui_error.display_message.clone();
        if let Some(suggestion) = &ui_error.recovery_suggestion {
            content.push_str(&format!("\n💡 {}", suggestion));
        }

        let mut notification = Notification::new(title.to_string(), content, notification_type);
        notification.ttl = Some(self.auto_dismiss_duration);

        let _ = self.error_notification_sender.send(notification);
    }

    /// Check if we should show an error popup
    pub fn should_show_error_popup(&self) -> bool {
        if let Some((_, timestamp)) = &self.last_error {
            timestamp.elapsed() < Duration::from_secs(5) // Show popup for 5 seconds
        } else {
            false
        }
    }

    /// Render error popup if needed
    pub fn render_error_popup(&mut self, f: &mut Frame, area: Rect, theme: &Theme) {
        if let Some((ui_error, timestamp)) = &self.last_error {
            if timestamp.elapsed() < Duration::from_secs(5) {
                self.render_error_dialog(f, area, theme, ui_error);
            } else {
                // Clear expired error
                self.last_error = None;
            }
        }
    }

    /// Render error dialog
    fn render_error_dialog(&self, f: &mut Frame, area: Rect, theme: &Theme, ui_error: &UIError) {
        // Calculate dialog size
        let dialog_width = std::cmp::min(80, area.width.saturating_sub(4));
        let dialog_height = std::cmp::min(20, area.height.saturating_sub(4));
        
        // Center the dialog
        let popup_area = centered_rect(dialog_width, dialog_height, area);

        // Clear the background
        f.render_widget(Clear, popup_area);

        // Create the dialog content
        let mut lines = vec![
            Line::from(vec![
                Span::styled(
                    self.get_severity_icon(ui_error.severity),
                    Style::default().fg(self.get_severity_color(ui_error.severity, theme)),
                ),
                Span::styled(
                    format!(" {}", ui_error.display_message),
                    Style::default().fg(theme.colors.foreground).add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        // Add technical details if available
        if let Some(details) = &ui_error.technical_details {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Details: ", Style::default().fg(theme.colors.muted)),
                Span::styled(details, Style::default().fg(theme.colors.muted)),
            ]));
        }

        // Add recovery suggestion if available
        if let Some(suggestion) = &ui_error.recovery_suggestion {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("💡 Suggestion: ", Style::default().fg(theme.colors.info)),
                Span::styled(suggestion, Style::default().fg(theme.colors.info)),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Press ", Style::default().fg(theme.colors.muted)),
            Span::styled("ESC", Style::default().fg(theme.colors.accent).add_modifier(Modifier::BOLD)),
            Span::styled(" to dismiss or wait 5 seconds", Style::default().fg(theme.colors.muted)),
        ]));

        let text = Text::from(lines);
        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title(format!(" {} Error ", self.get_severity_text(ui_error.severity)))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.get_severity_color(ui_error.severity, theme)))
            )
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, popup_area);
    }

    /// Get severity icon
    fn get_severity_icon(&self, severity: ErrorSeverity) -> &'static str {
        match severity {
            ErrorSeverity::Critical => "🚨",
            ErrorSeverity::Error => "❌",
            ErrorSeverity::Warning => "⚠️",
            ErrorSeverity::Info => "ℹ️",
        }
    }

    /// Get severity text
    fn get_severity_text(&self, severity: ErrorSeverity) -> &'static str {
        match severity {
            ErrorSeverity::Critical => "Critical",
            ErrorSeverity::Error => "Error",
            ErrorSeverity::Warning => "Warning",
            ErrorSeverity::Info => "Info",
        }
    }

    /// Get severity color
    fn get_severity_color(&self, severity: ErrorSeverity, theme: &Theme) -> ratatui::style::Color {
        match severity {
            ErrorSeverity::Critical => theme.colors.error,
            ErrorSeverity::Error => theme.colors.error,
            ErrorSeverity::Warning => theme.colors.warning,
            ErrorSeverity::Info => theme.colors.info,
        }
    }

    /// Dismiss current error popup
    pub fn dismiss_error_popup(&mut self) {
        self.last_error = None;
    }

    /// Clear error buffer
    pub fn clear_error_buffer(&mut self) {
        self.error_display_buffer.clear();
        self.last_error = None;
    }

    /// Get recent errors for display
    pub fn get_recent_errors(&self, count: usize) -> Vec<&UIError> {
        self.error_display_buffer
            .iter()
            .rev()
            .take(count)
            .collect()
    }
}

/// Helper function to create a centered rectangle
fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((r.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Length((r.height.saturating_sub(height)) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((r.width.saturating_sub(width)) / 2),
            Constraint::Length(width),
            Constraint::Length((r.width.saturating_sub(width)) / 2),
        ])
        .split(popup_layout[1])[1]
}

impl From<ErrorSeverity> for NotificationType {
    fn from(severity: ErrorSeverity) -> Self {
        match severity {
            ErrorSeverity::Critical => NotificationType::Error,
            ErrorSeverity::Error => NotificationType::Error,
            ErrorSeverity::Warning => NotificationType::Warning,
            ErrorSeverity::Info => NotificationType::Info,
        }
    }
}

/// Utility functions for error handling in UI
pub mod utils {
    use super::*;

    /// Create a user-friendly error message for display
    pub fn create_user_friendly_message(_error: &DevKitError) -> String {
        "Something went wrong, but we'll keep trying to help you.".to_string()
    }

    /// Get recovery actions for an error
    pub fn get_recovery_actions(_error: &DevKitError) -> Vec<String> {
        vec![
            "Try the operation again".to_string(),
            "Restart the application if issues persist".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_ui_error_handler() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let mut handler = UIErrorHandler::new(tx);

        // Use an existing AgentError variant for testing
        let error = crate::agents::AgentError::ConfigurationError("Test error".to_string());

        let strategy = handler.handle_error(error).await;
        assert!(matches!(strategy, RecoveryStrategy::Ignore));
        assert_eq!(handler.error_display_buffer.len(), 1);
    }

    #[test]
    fn test_error_conversion() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let handler = UIErrorHandler::new(tx);

        // Use an existing AgentError variant for testing
        let error_json = serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::Other, "test"));
        let error = crate::agents::AgentError::SerializationError(error_json);

        let ui_error = handler.convert_to_ui_error(&error);
        assert_eq!(ui_error.severity, ErrorSeverity::Error);
        assert!(ui_error.display_message.contains("agent system"));
    }
}
