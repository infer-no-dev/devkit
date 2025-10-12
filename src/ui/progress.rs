//! Enhanced progress indicator system with visual feedback for long-running operations.

use crate::ui::themes::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Gauge, LineGauge, Paragraph},
    Frame,
};
use std::{
    collections::HashMap,
    fmt,
    time::{Duration, Instant},
};
use tokio::sync::{mpsc, RwLock};
use tracing::trace;
use uuid::Uuid;

/// Progress indicator manager
#[derive(Debug)]
pub struct ProgressManager {
    operations: RwLock<HashMap<String, ProgressOperation>>,
    notification_sender: mpsc::UnboundedSender<ProgressUpdate>,
    notification_receiver: RwLock<Option<mpsc::UnboundedReceiver<ProgressUpdate>>>,
}

/// Individual progress operation
#[derive(Debug, Clone)]
pub struct ProgressOperation {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub progress: f64, // 0.0 to 1.0
    pub status: ProgressStatus,
    pub started_at: Instant,
    pub estimated_duration: Option<Duration>,
    pub steps: Vec<ProgressStep>,
    pub current_step: usize,
    pub style: ProgressStyle,
}

/// Progress operation status
#[derive(Debug, Clone, PartialEq)]
pub enum ProgressStatus {
    Running,
    Completed,
    Failed(String),
    Cancelled,
    Paused,
}

/// Individual step in a progress operation
#[derive(Debug, Clone)]
pub struct ProgressStep {
    pub name: String,
    pub description: Option<String>,
    pub progress: f64,
    pub status: ProgressStatus,
    pub started_at: Option<Instant>,
    pub completed_at: Option<Instant>,
}

/// Visual style for progress indicators
#[derive(Debug, Clone, PartialEq)]
pub enum ProgressStyle {
    /// Standard progress bar
    Bar,
    /// Compact line gauge
    Line,
    /// Spinner for indeterminate progress
    Spinner,
    /// Pulse effect for subtle operations
    Pulse,
    /// Step indicator with numbered stages
    Steps,
}

/// Progress update message
#[derive(Debug, Clone)]
pub enum ProgressUpdate {
    Started {
        id: String,
        title: String,
        description: Option<String>,
        style: ProgressStyle,
        estimated_duration: Option<Duration>,
        steps: Vec<String>,
    },
    Progress {
        id: String,
        progress: f64,
        description: Option<String>,
        current_step: Option<usize>,
    },
    StepProgress {
        id: String,
        step_index: usize,
        step_progress: f64,
        step_description: Option<String>,
    },
    Completed {
        id: String,
        message: Option<String>,
    },
    Failed {
        id: String,
        error: String,
    },
    Cancelled {
        id: String,
    },
}

/// Progress tracker for individual operations
#[derive(Debug, Clone)]
pub struct ProgressTracker {
    id: String,
    sender: mpsc::UnboundedSender<ProgressUpdate>,
}

impl ProgressManager {
    /// Create a new progress manager
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            operations: RwLock::new(HashMap::new()),
            notification_sender: tx,
            notification_receiver: RwLock::new(Some(rx)),
        }
    }

    /// Start a new progress operation
    pub async fn start_operation(
        &self,
        title: String,
        description: Option<String>,
        style: ProgressStyle,
        estimated_duration: Option<Duration>,
        steps: Vec<String>,
    ) -> ProgressTracker {
        let id = Uuid::new_v4().to_string();
        
        let operation = ProgressOperation {
            id: id.clone(),
            title: title.clone(),
            description: description.clone(),
            progress: 0.0,
            status: ProgressStatus::Running,
            started_at: Instant::now(),
            estimated_duration,
            steps: steps.iter().enumerate().map(|(i, name)| ProgressStep {
                name: name.clone(),
                description: None,
                progress: 0.0,
                status: if i == 0 { ProgressStatus::Running } else { ProgressStatus::Running },
                started_at: if i == 0 { Some(Instant::now()) } else { None },
                completed_at: None,
            }).collect(),
            current_step: 0,
            style: style.clone(),
        };

        let mut operations = self.operations.write().await;
        operations.insert(id.clone(), operation);

        // Send start notification
        let _ = self.notification_sender.send(ProgressUpdate::Started {
            id: id.clone(),
            title,
            description,
            style,
            estimated_duration,
            steps,
        });

        ProgressTracker {
            id,
            sender: self.notification_sender.clone(),
        }
    }

    /// Update progress for an operation
    pub async fn update_progress(
        &self,
        id: &str,
        progress: f64,
        description: Option<String>,
        current_step: Option<usize>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut operations = self.operations.write().await;
        if let Some(operation) = operations.get_mut(id) {
            operation.progress = progress.clamp(0.0, 1.0);
            if let Some(desc) = description {
                operation.description = Some(desc);
            }
            if let Some(step) = current_step {
                if step < operation.steps.len() {
                    // Complete previous steps
                    for i in 0..step {
                        if operation.steps[i].status == ProgressStatus::Running {
                            operation.steps[i].status = ProgressStatus::Completed;
                            operation.steps[i].completed_at = Some(Instant::now());
                            operation.steps[i].progress = 1.0;
                        }
                    }
                    // Update current step
                    if operation.current_step != step {
                        operation.current_step = step;
                        operation.steps[step].started_at = Some(Instant::now());
                    }
                }
            }
        }
        Ok(())
    }

    /// Complete an operation
    pub async fn complete_operation(
        &self,
        id: &str,
        message: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut operations = self.operations.write().await;
        if let Some(operation) = operations.get_mut(id) {
            operation.status = ProgressStatus::Completed;
            operation.progress = 1.0;
            
            // Complete all steps
            for step in &mut operation.steps {
                if step.status == ProgressStatus::Running {
                    step.status = ProgressStatus::Completed;
                    step.completed_at = Some(Instant::now());
                    step.progress = 1.0;
                }
            }
        }
        Ok(())
    }

    /// Fail an operation
    pub async fn fail_operation(
        &self,
        id: &str,
        error: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut operations = self.operations.write().await;
        if let Some(operation) = operations.get_mut(id) {
            operation.status = ProgressStatus::Failed(error);
        }
        Ok(())
    }

    /// Get active operations
    pub async fn get_active_operations(&self) -> Vec<ProgressOperation> {
        let operations = self.operations.read().await;
        operations
            .values()
            .filter(|op| matches!(op.status, ProgressStatus::Running | ProgressStatus::Paused))
            .cloned()
            .collect()
    }

    /// Get all operations (for history view)
    pub async fn get_all_operations(&self) -> Vec<ProgressOperation> {
        let operations = self.operations.read().await;
        operations.values().cloned().collect()
    }

    /// Clear completed operations
    pub async fn clear_completed(&self) {
        let mut operations = self.operations.write().await;
        operations.retain(|_, op| !matches!(op.status, ProgressStatus::Completed));
    }

    /// Render active progress indicators
    pub async fn render_progress_indicators(
        &self,
        f: &mut Frame<'_>,
        area: Rect,
        theme: &Theme,
    ) {
        let active_operations = self.get_active_operations().await;
        if active_operations.is_empty() {
            return;
        }

        // Calculate layout
        let operations_count = active_operations.len();
        let height_per_operation = if operations_count > 0 {
            std::cmp::max(3, area.height / operations_count as u16)
        } else {
            3
        };

        let constraints: Vec<Constraint> = (0..operations_count)
            .map(|_| Constraint::Length(height_per_operation))
            .collect();

        if !constraints.is_empty() {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(area);

            for (i, operation) in active_operations.iter().enumerate() {
                if i < layout.len() {
                    self.render_single_progress(f, layout[i], theme, operation).await;
                }
            }
        }
    }

    /// Render a single progress operation
    async fn render_single_progress(
        &self,
        f: &mut Frame<'_>,
        area: Rect,
        theme: &Theme,
        operation: &ProgressOperation,
    ) {
        match operation.style {
            ProgressStyle::Bar => self.render_progress_bar(f, area, theme, operation),
            ProgressStyle::Line => self.render_progress_line(f, area, theme, operation),
            ProgressStyle::Spinner => self.render_progress_spinner(f, area, theme, operation),
            ProgressStyle::Pulse => self.render_progress_pulse(f, area, theme, operation),
            ProgressStyle::Steps => self.render_progress_steps(f, area, theme, operation),
        }
    }

    /// Render standard progress bar
    fn render_progress_bar(&self, f: &mut Frame, area: Rect, theme: &Theme, operation: &ProgressOperation) {
        let progress_percent = (operation.progress * 100.0) as u16;
        
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title(format!(" {} ", operation.title))
                    .borders(Borders::ALL)
                    .border_style(self.get_status_style(&operation.status, theme))
            )
            .gauge_style(
                Style::default()
                    .fg(theme.colors.accent)
                    .bg(theme.colors.background)
            )
            .percent(progress_percent)
            .label(format!(
                "{:.1}% {}",
                operation.progress * 100.0,
                operation.description.as_deref().unwrap_or("")
            ));

        f.render_widget(gauge, area);
    }

    /// Render compact line gauge
    fn render_progress_line(&self, f: &mut Frame, area: Rect, theme: &Theme, operation: &ProgressOperation) {
        let progress_percent = (operation.progress * 100.0) as u16;
        
        let line_gauge = LineGauge::default()
            .block(
                Block::default()
                    .title(format!(" {} ", operation.title))
                    .borders(Borders::ALL)
                    .border_style(self.get_status_style(&operation.status, theme))
            )
            .filled_style(
                Style::default()
                    .fg(theme.colors.accent)
                    .add_modifier(Modifier::BOLD)
            )
            .unfilled_style(Style::default().fg(theme.colors.muted))
            .ratio(operation.progress)
            .label(format!(
                "{:.1}% {}",
                operation.progress * 100.0,
                operation.description.as_deref().unwrap_or("")
            ));

        f.render_widget(line_gauge, area);
    }

    /// Render spinner for indeterminate progress
    fn render_progress_spinner(&self, f: &mut Frame, area: Rect, theme: &Theme, operation: &ProgressOperation) {
        let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let spinner_index = (operation.started_at.elapsed().as_millis() / 80) % spinner_chars.len() as u128;
        let spinner_char = spinner_chars[spinner_index as usize];

        let mut content = vec![
            Line::from(vec![
                Span::styled(
                    spinner_char,
                    Style::default().fg(theme.colors.accent).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {}", operation.title),
                    Style::default().fg(theme.colors.foreground),
                ),
            ]),
        ];

        if let Some(description) = &operation.description {
            content.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(description, Style::default().fg(theme.colors.muted)),
            ]));
        }

        let text = Text::from(content);
        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.get_status_style(&operation.status, theme))
            );

        f.render_widget(paragraph, area);
    }

    /// Render pulse effect
    fn render_progress_pulse(&self, f: &mut Frame, area: Rect, theme: &Theme, operation: &ProgressOperation) {
        let pulse_intensity = ((operation.started_at.elapsed().as_millis() as f64 / 1000.0 * 2.0 * std::f64::consts::PI).sin() + 1.0) / 2.0;
        let pulse_color = theme.colors.accent; // Could interpolate with background for pulse effect

        let content = vec![
            Line::from(vec![
                Span::styled(
                    "●",
                    Style::default().fg(pulse_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {}", operation.title),
                    Style::default().fg(theme.colors.foreground),
                ),
            ]),
        ];

        let text = Text::from(content);
        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.get_status_style(&operation.status, theme))
            );

        f.render_widget(paragraph, area);
    }

    /// Render step-by-step progress
    fn render_progress_steps(&self, f: &mut Frame, area: Rect, theme: &Theme, operation: &ProgressOperation) {
        let mut lines = vec![
            Line::from(vec![
                Span::styled(
                    format!("{} ({}/{})", operation.title, operation.current_step + 1, operation.steps.len()),
                    Style::default().fg(theme.colors.foreground).add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        // Show current and next few steps
        let start_step = operation.current_step.saturating_sub(1);
        let end_step = std::cmp::min(operation.steps.len(), start_step + 4);

        for (i, step) in operation.steps[start_step..end_step].iter().enumerate() {
            let step_index = start_step + i;
            let (icon, style) = match (&step.status, step_index == operation.current_step) {
                (ProgressStatus::Completed, _) => ("✓", Style::default().fg(theme.colors.success)),
                (ProgressStatus::Failed(_), _) => ("✗", Style::default().fg(theme.colors.error)),
                (ProgressStatus::Running, true) => ("●", Style::default().fg(theme.colors.accent)),
                _ => ("○", Style::default().fg(theme.colors.muted)),
            };

            lines.push(Line::from(vec![
                Span::styled(format!("  {}", icon), style),
                Span::styled(
                    format!(" {}", step.name),
                    if step_index == operation.current_step {
                        Style::default().fg(theme.colors.foreground)
                    } else {
                        Style::default().fg(theme.colors.muted)
                    },
                ),
            ]));
        }

        let text = Text::from(lines);
        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.get_status_style(&operation.status, theme))
            );

        f.render_widget(paragraph, area);
    }

    /// Get style for operation status
    fn get_status_style(&self, status: &ProgressStatus, theme: &Theme) -> Style {
        match status {
            ProgressStatus::Running => Style::default().fg(theme.colors.accent),
            ProgressStatus::Completed => Style::default().fg(theme.colors.success),
            ProgressStatus::Failed(_) => Style::default().fg(theme.colors.error),
            ProgressStatus::Cancelled => Style::default().fg(theme.colors.warning),
            ProgressStatus::Paused => Style::default().fg(theme.colors.warning),
        }
    }

    /// Clean up old operations
    pub async fn cleanup_old_operations(&self, max_age: Duration) {
        let mut operations = self.operations.write().await;
        let now = Instant::now();
        
        operations.retain(|_, op| {
            match &op.status {
                ProgressStatus::Running | ProgressStatus::Paused => true,
                _ => now.duration_since(op.started_at) < max_age,
            }
        });
    }
}

impl ProgressTracker {
    /// Update progress
    pub fn update_progress(&self, progress: f64, description: Option<String>) {
        let _ = self.sender.send(ProgressUpdate::Progress {
            id: self.id.clone(),
            progress,
            description,
            current_step: None,
        });
    }

    /// Update step progress
    pub fn update_step(&self, step_index: usize, step_progress: f64, description: Option<String>) {
        let _ = self.sender.send(ProgressUpdate::StepProgress {
            id: self.id.clone(),
            step_index,
            step_progress,
            step_description: description,
        });
    }

    /// Complete the operation
    pub fn complete(&self, message: Option<String>) {
        let _ = self.sender.send(ProgressUpdate::Completed {
            id: self.id.clone(),
            message,
        });
    }

    /// Fail the operation
    pub fn fail(&self, error: String) {
        let _ = self.sender.send(ProgressUpdate::Failed {
            id: self.id.clone(),
            error,
        });
    }

    /// Cancel the operation
    pub fn cancel(&self) {
        let _ = self.sender.send(ProgressUpdate::Cancelled {
            id: self.id.clone(),
        });
    }

    /// Get operation ID
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl Default for ProgressManager {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ProgressStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProgressStatus::Running => write!(f, "Running"),
            ProgressStatus::Completed => write!(f, "Completed"),
            ProgressStatus::Failed(err) => write!(f, "Failed: {}", err),
            ProgressStatus::Cancelled => write!(f, "Cancelled"),
            ProgressStatus::Paused => write!(f, "Paused"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_progress_manager() {
        let manager = ProgressManager::new();
        
        let tracker = manager.start_operation(
            "Test Operation".to_string(),
            Some("Testing progress".to_string()),
            ProgressStyle::Bar,
            Some(Duration::from_secs(10)),
            vec!["Step 1".to_string(), "Step 2".to_string()],
        ).await;

        // Check active operations
        let active = manager.get_active_operations().await;
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].title, "Test Operation");

        // Update progress
        tracker.update_progress(0.5, Some("Half done".to_string()));
        
        // Complete operation
        tracker.complete(Some("Done!".to_string()));
        
        // Should still be in operations but not active
        let all_ops = manager.get_all_operations().await;
        assert_eq!(all_ops.len(), 1);
        
        let active = manager.get_active_operations().await;
        assert_eq!(active.len(), 0);
    }

    #[test]
    fn test_progress_status_display() {
        assert_eq!(ProgressStatus::Running.to_string(), "Running");
        assert_eq!(ProgressStatus::Completed.to_string(), "Completed");
        assert_eq!(
            ProgressStatus::Failed("Network error".to_string()).to_string(),
            "Failed: Network error"
        );
        assert_eq!(ProgressStatus::Cancelled.to_string(), "Cancelled");
        assert_eq!(ProgressStatus::Paused.to_string(), "Paused");
    }
}