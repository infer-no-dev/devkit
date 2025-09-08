//! Notification system for displaying alerts and updates.

use std::collections::VecDeque;
use std::time::{SystemTime, Duration};
use ratatui::{
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use serde::{Deserialize, Serialize};
use crate::ui::themes::Theme;

/// Types of notifications
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
    AgentUpdate,
    SystemMessage,
    UserAction,
}

/// Priority levels for notifications
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// A notification message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub priority: NotificationPriority,
    pub timestamp: SystemTime,
    pub ttl: Option<Duration>, // Time to live
    pub actions: Vec<NotificationAction>,
    pub dismissible: bool,
    pub sticky: bool, // Won't auto-dismiss
}

/// Action that can be performed on a notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    pub id: String,
    pub label: String,
    pub action_type: ActionType,
}

/// Types of actions available for notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Dismiss,
    ViewDetails,
    Retry,
    Cancel,
    Custom(String),
}

/// Notification panel for managing and displaying notifications
#[derive(Debug)]
pub struct NotificationPanel {
    notifications: VecDeque<Notification>,
    max_notifications: usize,
    auto_dismiss_timeout: Duration,
    show_timestamps: bool,
    filter_type: Option<NotificationType>,
    min_priority: NotificationPriority,
}

impl Notification {
    /// Create a new notification
    pub fn new(
        title: String,
        message: String,
        notification_type: NotificationType,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            message,
            notification_type,
            priority: NotificationPriority::Normal,
            timestamp: SystemTime::now(),
            ttl: None,
            actions: vec![NotificationAction::dismiss()],
            dismissible: true,
            sticky: false,
        }
    }
    
    /// Create an info notification
    pub fn info(title: String, message: String) -> Self {
        let mut notification = Self::new(title, message, NotificationType::Info);
        notification.ttl = Some(Duration::from_secs(10));
        notification
    }
    
    /// Create a success notification
    pub fn success(title: String, message: String) -> Self {
        let mut notification = Self::new(title, message, NotificationType::Success);
        notification.ttl = Some(Duration::from_secs(8));
        notification
    }
    
    /// Create a warning notification
    pub fn warning(title: String, message: String) -> Self {
        let mut notification = Self::new(title, message, NotificationType::Warning);
        notification.priority = NotificationPriority::High;
        notification.ttl = Some(Duration::from_secs(15));
        notification
    }
    
    /// Create an error notification
    pub fn error(title: String, message: String) -> Self {
        let mut notification = Self::new(title, message, NotificationType::Error);
        notification.priority = NotificationPriority::Critical;
        notification.sticky = true;
        notification.actions = vec![
            NotificationAction::dismiss(),
            NotificationAction::view_details(),
        ];
        notification
    }
    
    /// Create an agent update notification
    pub fn agent_update(agent_name: String, status: String) -> Self {
        let mut notification = Self::new(
            format!("Agent: {}", agent_name),
            status,
            NotificationType::AgentUpdate,
        );
        notification.ttl = Some(Duration::from_secs(5));
        notification
    }
    
    /// Create a system message notification
    pub fn system_message(message: String) -> Self {
        let mut notification = Self::new(
            "System".to_string(),
            message,
            NotificationType::SystemMessage,
        );
        notification.ttl = Some(Duration::from_secs(12));
        notification
    }
    
    /// Check if the notification has expired
    pub fn is_expired(&self) -> bool {
        if self.sticky {
            return false;
        }
        
        if let Some(ttl) = self.ttl {
            if let Ok(elapsed) = self.timestamp.elapsed() {
                return elapsed > ttl;
            }
        }
        
        false
    }
    
    /// Get age of the notification
    pub fn age(&self) -> Duration {
        self.timestamp.elapsed().unwrap_or(Duration::ZERO)
    }
    
    /// Format the notification for display
    pub fn format_for_display(&self, theme: &Theme, show_timestamp: bool) -> Line<'_> {
        let mut spans = Vec::new();
        
        // Add timestamp if requested
        if show_timestamp {
            if let Ok(duration) = self.timestamp.duration_since(std::time::UNIX_EPOCH) {
                let timestamp = duration.as_secs();
                let time_str = format!("[{}] ",
                    chrono::DateTime::from_timestamp(timestamp as i64, 0)
                        .unwrap_or_default()
                        .format("%H:%M:%S")
                );
                spans.push(Span::styled(time_str, theme.timestamp_style()));
            }
        }
        
        // Add type indicator
        let (indicator, style) = match self.notification_type {
            NotificationType::Info => ("ℹ️  ", theme.info_style()),
            NotificationType::Success => ("✅ ", theme.success_style()),
            NotificationType::Warning => ("⚠️  ", theme.warning_style()),
            NotificationType::Error => ("❌ ", theme.error_style()),
            NotificationType::AgentUpdate => ("🤖 ", theme.agent_response_style()),
            NotificationType::SystemMessage => ("⚙️  ", theme.system_style()),
            NotificationType::UserAction => ("👤 ", theme.user_input_style()),
        };
        
        spans.push(Span::styled(indicator.to_string(), style));
        
        // Add title if different from message
        if self.title != self.message && !self.title.is_empty() {
            spans.push(Span::styled(format!("{}: ", self.title), style.add_modifier(Modifier::BOLD)));
        }
        
        // Add message
        spans.push(Span::styled(self.message.clone(), style));
        
        // Add priority indicator for high/critical
        match self.priority {
            NotificationPriority::Critical => {
                spans.push(Span::styled(" [!]", theme.error_style().add_modifier(Modifier::BOLD)));
            }
            NotificationPriority::High => {
                spans.push(Span::styled(" [!]", theme.warning_style()));
            }
            _ => {}
        }
        
        Line::from(spans)
    }
    
    /// Set time-to-live for the notification
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }
    
    /// Set priority for the notification
    pub fn with_priority(mut self, priority: NotificationPriority) -> Self {
        self.priority = priority;
        self
    }
    
    /// Make notification sticky (won't auto-dismiss)
    pub fn sticky(mut self) -> Self {
        self.sticky = true;
        self.ttl = None;
        self
    }
    
    /// Add an action to the notification
    pub fn with_action(mut self, action: NotificationAction) -> Self {
        self.actions.push(action);
        self
    }
}

impl NotificationAction {
    /// Create a new notification action
    pub fn new(id: String, label: String, action_type: ActionType) -> Self {
        Self { id, label, action_type }
    }
    
    /// Create a dismiss action
    pub fn dismiss() -> Self {
        Self::new(
            "dismiss".to_string(),
            "Dismiss".to_string(),
            ActionType::Dismiss,
        )
    }
    
    /// Create a view details action
    pub fn view_details() -> Self {
        Self::new(
            "details".to_string(),
            "Details".to_string(),
            ActionType::ViewDetails,
        )
    }
    
    /// Create a retry action
    pub fn retry() -> Self {
        Self::new(
            "retry".to_string(),
            "Retry".to_string(),
            ActionType::Retry,
        )
    }
    
    /// Create a cancel action
    pub fn cancel() -> Self {
        Self::new(
            "cancel".to_string(),
            "Cancel".to_string(),
            ActionType::Cancel,
        )
    }
    
    /// Create a custom action
    pub fn custom(id: String, label: String, action: String) -> Self {
        Self::new(id, label, ActionType::Custom(action))
    }
}

impl NotificationPanel {
    /// Create a new notification panel
    pub fn new(auto_dismiss_timeout_ms: u64) -> Self {
        Self {
            notifications: VecDeque::new(),
            max_notifications: 100,
            auto_dismiss_timeout: Duration::from_millis(auto_dismiss_timeout_ms),
            show_timestamps: true,
            filter_type: None,
            min_priority: NotificationPriority::Low,
        }
    }
    
    /// Add a notification to the panel
    pub fn add_notification(&mut self, notification: Notification) {
        // Check if notification meets minimum priority
        if notification.priority < self.min_priority {
            return;
        }
        
        // Add to front of deque (most recent first)
        self.notifications.push_front(notification);
        
        // Maintain size limit
        if self.notifications.len() > self.max_notifications {
            self.notifications.truncate(self.max_notifications);
        }
    }
    
    /// Remove a notification by ID
    pub fn remove_notification(&mut self, id: &str) -> bool {
        if let Some(pos) = self.notifications.iter().position(|n| n.id == id) {
            self.notifications.remove(pos);
            true
        } else {
            false
        }
    }
    
    /// Clear all notifications
    pub fn clear_all(&mut self) {
        self.notifications.clear();
    }
    
    /// Clear only dismissible notifications
    pub fn clear_dismissible(&mut self) {
        self.notifications.retain(|n| !n.dismissible || n.sticky);
    }
    
    /// Remove expired notifications
    pub fn cleanup_expired(&mut self) {
        self.notifications.retain(|n| !n.is_expired());
    }
    
    /// Get all notifications
    pub fn get_notifications(&self) -> &VecDeque<Notification> {
        &self.notifications
    }
    
    /// Get notifications filtered by type
    pub fn get_notifications_by_type(&self, notification_type: &NotificationType) -> Vec<&Notification> {
        self.notifications
            .iter()
            .filter(|n| n.notification_type == *notification_type)
            .collect()
    }
    
    /// Get notifications filtered by priority
    pub fn get_notifications_by_priority(&self, min_priority: &NotificationPriority) -> Vec<&Notification> {
        self.notifications
            .iter()
            .filter(|n| n.priority >= *min_priority)
            .collect()
    }
    
    /// Get recent notifications (last n)
    pub fn get_recent_notifications(&self, count: usize) -> Vec<&Notification> {
        self.notifications.iter().take(count).collect()
    }
    
    /// Set notification type filter
    pub fn set_type_filter(&mut self, filter: Option<NotificationType>) {
        self.filter_type = filter;
    }
    
    /// Set minimum priority filter
    pub fn set_min_priority(&mut self, priority: NotificationPriority) {
        self.min_priority = priority;
    }
    
    /// Toggle timestamp display
    pub fn toggle_timestamps(&mut self) {
        self.show_timestamps = !self.show_timestamps;
    }
    
    /// Render the notification panel
    pub fn render(
        &self,
        f: &mut Frame,
        area: Rect,
        theme: &Theme,
    ) {
        // Get notifications to display based on filters
        let notifications_to_show: Vec<&Notification> = self.notifications
            .iter()
            .filter(|n| {
                if let Some(ref filter_type) = self.filter_type {
                    if n.notification_type != *filter_type {
                        return false;
                    }
                }
                n.priority >= self.min_priority
            })
            .take(area.height.saturating_sub(2) as usize) // Account for borders
            .collect();
        
        // Create list items
        let items: Vec<ListItem> = notifications_to_show
            .iter()
            .map(|notification| {
                ListItem::new(notification.format_for_display(theme, self.show_timestamps))
            })
            .collect();
        
        // Create the notification list
        let notification_list = List::new(items)
            .block(Block::default()
                .title("Notifications")
                .borders(Borders::ALL)
                .style(theme.border_style()));
        
        f.render_widget(notification_list, area);
    }
    
    /// Render notification count badge
    pub fn render_count_badge(
        &self,
        f: &mut Frame,
        area: Rect,
        theme: &Theme,
    ) {
        let count = self.notifications.len();
        if count == 0 {
            return;
        }
        
        let critical_count = self.notifications
            .iter()
            .filter(|n| n.priority == NotificationPriority::Critical)
            .count();
        
        let warning_count = self.notifications
            .iter()
            .filter(|n| n.priority == NotificationPriority::High)
            .count();
        
        let badge_text = if critical_count > 0 {
            format!("!{}", critical_count)
        } else if warning_count > 0 {
            format!("⚠{}", warning_count)
        } else {
            format!("{}", count)
        };
        
        let style = if critical_count > 0 {
            theme.error_style().add_modifier(Modifier::BOLD)
        } else if warning_count > 0 {
            theme.warning_style()
        } else {
            theme.info_style()
        };
        
        let badge = Paragraph::new(badge_text)
            .style(style)
            .wrap(Wrap { trim: true });
        
        f.render_widget(badge, area);
    }
    
    /// Get notification count
    pub fn count(&self) -> usize {
        self.notifications.len()
    }
    
    /// Get count by priority
    pub fn count_by_priority(&self, priority: NotificationPriority) -> usize {
        self.notifications
            .iter()
            .filter(|n| n.priority == priority)
            .count()
    }
    
    /// Check if there are any critical notifications
    pub fn has_critical(&self) -> bool {
        self.notifications
            .iter()
            .any(|n| n.priority == NotificationPriority::Critical)
    }
    
    /// Process notification action
    pub fn process_action(&mut self, notification_id: &str, action_id: &str) -> Option<ActionType> {
        if let Some(notification) = self.notifications.iter().find(|n| n.id == notification_id) {
            if let Some(action) = notification.actions.iter().find(|a| a.id == action_id) {
                let action_type = action.action_type.clone();
                
                // Handle built-in actions
                match &action_type {
                    ActionType::Dismiss => {
                        self.remove_notification(notification_id);
                    }
                    _ => {} // Other actions handled by caller
                }
                
                return Some(action_type);
            }
        }
        None
    }
}
