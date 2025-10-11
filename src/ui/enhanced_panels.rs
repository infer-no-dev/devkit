//! Enhanced UI panels with improved styling, error handling, and progress integration.

use crate::agents::{AgentStatus, TaskPriority};
use crate::ui::{
    error_handler::UIErrorHandler,
    progress::ProgressManager,
    themes::Theme,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Wrap,
    },
    Frame,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tracing::trace;

/// Enhanced panel manager with better error handling and visual improvements
#[derive(Debug)]
pub struct EnhancedPanelManager {
    panels: HashMap<String, EnhancedPanel>,
    layout_state: PanelLayoutState,
    error_handler: Option<UIErrorHandler>,
    progress_manager: Option<ProgressManager>,
    notification_sender: Option<mpsc::UnboundedSender<crate::ui::notifications::Notification>>,
    scroll_states: HashMap<String, ScrollbarState>,
    animation_states: HashMap<String, AnimationState>,
    last_update: Instant,
}

/// Enhanced panel with rich content and styling
#[derive(Debug)]
pub struct EnhancedPanel {
    pub id: String,
    pub title: String,
    pub content: PanelContent,
    pub style_config: PanelStyleConfig,
    pub scroll_position: usize,
    pub max_items: Option<usize>,
    pub auto_scroll: bool,
    pub visible: bool,
    pub focused: bool,
    pub last_updated: Instant,
    pub error_count: usize,
    pub warning_count: usize,
}

/// Panel content types with rich formatting
#[derive(Debug, Clone)]
pub enum PanelContent {
    /// Agent status with progress indicators
    AgentStatus {
        agents: Vec<EnhancedAgentInfo>,
        summary: AgentSummary,
    },
    /// Command output with syntax highlighting
    Output {
        lines: Vec<OutputLine>,
        filter: Option<String>,
    },
    /// Interactive input with completions
    Input {
        current_input: String,
        history: Vec<String>,
        completions: Vec<String>,
        cursor_position: usize,
    },
    /// Help information with searchable content
    Help {
        sections: Vec<HelpSection>,
        search_term: Option<String>,
    },
    /// System status and diagnostics
    Status {
        metrics: SystemMetrics,
        alerts: Vec<StatusAlert>,
    },
    /// Logs with filtering and level indicators
    Logs {
        entries: Vec<LogEntry>,
        level_filter: LogLevel,
        search_filter: Option<String>,
    },
}

/// Enhanced agent information
#[derive(Debug, Clone)]
pub struct EnhancedAgentInfo {
    pub name: String,
    pub status: AgentStatus,
    pub current_task: Option<String>,
    pub priority: Option<TaskPriority>,
    pub progress: Option<f64>,
    pub last_activity: Instant,
    pub error_count: usize,
    pub success_count: usize,
    pub estimated_completion: Option<Instant>,
    pub resource_usage: ResourceUsage,
}

/// Agent summary statistics
#[derive(Debug, Clone)]
pub struct AgentSummary {
    pub total_agents: usize,
    pub active_agents: usize,
    pub idle_agents: usize,
    pub failed_agents: usize,
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub avg_response_time: Duration,
}

/// Styled output line
#[derive(Debug, Clone)]
pub struct OutputLine {
    pub content: String,
    pub line_type: OutputType,
    pub timestamp: Instant,
    pub metadata: Option<HashMap<String, String>>,
}

/// Output line types
#[derive(Debug, Clone, PartialEq)]
pub enum OutputType {
    Normal,
    Error,
    Warning,
    Success,
    Debug,
    Command,
    Result,
}

/// Panel styling configuration
#[derive(Debug, Clone)]
pub struct PanelStyleConfig {
    pub border_style: BorderStyle,
    pub title_alignment: Alignment,
    pub highlight_current_line: bool,
    pub show_line_numbers: bool,
    pub show_scrollbar: bool,
    pub animated_borders: bool,
    pub gradient_background: bool,
    pub transparency: f32,
}

/// Border styling options
#[derive(Debug, Clone)]
pub enum BorderStyle {
    Normal,
    Rounded,
    Thick,
    Double,
    Dashed,
    Custom(symbols::border::Set),
}

/// Panel layout state
#[derive(Debug)]
pub struct PanelLayoutState {
    pub current_layout: String,
    pub panel_sizes: HashMap<String, (u16, u16)>, // (width, height)
    pub panel_positions: HashMap<String, (u16, u16)>, // (x, y)
    pub focus_stack: Vec<String>,
    pub modal_stack: Vec<String>,
}

/// Animation state for panels
#[derive(Debug)]
pub struct AnimationState {
    pub pulse_phase: f32,
    pub blink_state: bool,
    pub last_blink: Instant,
    pub scroll_offset: f32,
}

/// Help section
#[derive(Debug, Clone)]
pub struct HelpSection {
    pub title: String,
    pub content: String,
    pub examples: Vec<String>,
    pub related_commands: Vec<String>,
}

/// System metrics
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_activity: f64,
    pub active_connections: usize,
    pub uptime: Duration,
}

/// Status alert
#[derive(Debug, Clone)]
pub struct StatusAlert {
    pub level: AlertLevel,
    pub message: String,
    pub timestamp: Instant,
    pub source: String,
}

/// Alert levels
#[derive(Debug, Clone, PartialEq)]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
    Critical,
}

/// Log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: Instant,
    pub source: String,
    pub metadata: HashMap<String, String>,
}

/// Log levels
#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warning,
    Error,
}

/// Resource usage information
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub cpu_percent: f64,
    pub memory_mb: u64,
    pub network_bytes: u64,
    pub disk_io: u64,
}

impl EnhancedPanelManager {
    /// Create a new enhanced panel manager
    pub fn new() -> Self {
        Self {
            panels: HashMap::new(),
            layout_state: PanelLayoutState {
                current_layout: "default".to_string(),
                panel_sizes: HashMap::new(),
                panel_positions: HashMap::new(),
                focus_stack: Vec::new(),
                modal_stack: Vec::new(),
            },
            error_handler: None,
            progress_manager: None,
            notification_sender: None,
            scroll_states: HashMap::new(),
            animation_states: HashMap::new(),
            last_update: Instant::now(),
        }
    }

    /// Set error handler
    pub fn set_error_handler(&mut self, error_handler: UIErrorHandler) {
        self.error_handler = Some(error_handler);
    }

    /// Set progress manager
    pub fn set_progress_manager(&mut self, progress_manager: ProgressManager) {
        self.progress_manager = Some(progress_manager);
    }

    /// Set notification sender
    pub fn set_notification_sender(&mut self, sender: mpsc::UnboundedSender<crate::ui::notifications::Notification>) {
        self.notification_sender = Some(sender);
    }

    /// Add a new panel
    pub fn add_panel(&mut self, mut panel: EnhancedPanel) {
        panel.last_updated = Instant::now();
        
        // Initialize scroll and animation states
        self.scroll_states.insert(panel.id.clone(), ScrollbarState::default());
        self.animation_states.insert(panel.id.clone(), AnimationState {
            pulse_phase: 0.0,
            blink_state: false,
            last_blink: Instant::now(),
            scroll_offset: 0.0,
        });

        self.panels.insert(panel.id.clone(), panel);
    }

    /// Update panel content with error handling
    pub async fn update_panel_content(&mut self, panel_id: &str, content: PanelContent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(panel) = self.panels.get_mut(panel_id) {
            panel.content = content;
            panel.last_updated = Instant::now();
            
            // Update error and warning counts based on content
            Self::update_panel_metrics_static(panel);
            
            trace!("Updated panel content for {}", panel_id);
            Ok(())
        } else {
            Err(format!("Panel {} not found", panel_id).into())
        }
    }

    /// Update panel metrics based on content
    fn update_panel_metrics_static(panel: &mut EnhancedPanel) {
        panel.error_count = 0;
        panel.warning_count = 0;

        match &panel.content {
            PanelContent::Output { lines, .. } => {
                for line in lines {
                    match line.line_type {
                        OutputType::Error => panel.error_count += 1,
                        OutputType::Warning => panel.warning_count += 1,
                        _ => {}
                    }
                }
            }
            PanelContent::Logs { entries, .. } => {
                for entry in entries {
                    match entry.level {
                        LogLevel::Error => panel.error_count += 1,
                        LogLevel::Warning => panel.warning_count += 1,
                        _ => {}
                    }
                }
            }
            PanelContent::Status { alerts, .. } => {
                for alert in alerts {
                    match alert.level {
                        AlertLevel::Error | AlertLevel::Critical => panel.error_count += 1,
                        AlertLevel::Warning => panel.warning_count += 1,
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    /// Render all panels
    pub async fn render(&mut self, f: &mut Frame<'_>, area: Rect, theme: &Theme) {
        // Update animations
        self.update_animations();

        // Render error popup if needed
        if let Some(ref mut error_handler) = self.error_handler {
            error_handler.render_error_popup(f, area, theme);
        }

        // Render progress indicators
        if let Some(ref progress_manager) = self.progress_manager {
            // Reserve top area for progress indicators
            let progress_height = 6; // Adjust based on active operations
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(progress_height), Constraint::Min(0)])
                .split(area);

            progress_manager.render_progress_indicators(f, layout[0], theme).await;
            
            // Use remaining area for panels
            self.render_panels(f, layout[1], theme).await;
        } else {
            self.render_panels(f, area, theme).await;
        }
    }

    /// Render panels in the specified area
    async fn render_panels(&mut self, f: &mut Frame<'_>, area: Rect, theme: &Theme) {
        let visible_panel_ids: Vec<String> = self.panels.iter()
            .filter(|(_, p)| p.visible)
            .map(|(id, _)| id.clone())
            .collect();

        if visible_panel_ids.is_empty() {
            return;
        }

        // Simple layout for now - could be made configurable
        let panel_count = visible_panel_ids.len();
        let constraints: Vec<Constraint> = (0..panel_count)
            .map(|_| Constraint::Percentage(100 / panel_count as u16))
            .collect();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        for (i, panel_id) in visible_panel_ids.iter().enumerate() {
            if i < layout.len() {
                if let Some(panel) = self.panels.get(panel_id) {
                    self.render_single_panel(f, layout[i], theme, panel);
                }
            }
        }
    }

    /// Render a single panel
    fn render_single_panel(&self, f: &mut Frame<'_>, area: Rect, theme: &Theme, panel: &EnhancedPanel) {
        // Create border with dynamic styling
        let border_style = self.get_border_style(&panel.style_config.border_style, theme, panel.focused);
        let title_style = if panel.focused {
            Style::default().fg(theme.colors.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.colors.muted)
        };

        // Add status indicators to title
        let mut title = panel.title.clone();
        if panel.error_count > 0 {
            title.push_str(&format!(" ❌{}", panel.error_count));
        }
        if panel.warning_count > 0 {
            title.push_str(&format!(" ⚠️{}", panel.warning_count));
        }

        let block = Block::default()
            .title(format!(" {} ", title))
            .title_style(title_style)
            .borders(Borders::ALL)
            .border_style(border_style);

        // Render content based on type
        match &panel.content {
            PanelContent::AgentStatus { agents, summary } => {
                self.render_agent_status(f, area, theme, block, agents, summary);
            }
            PanelContent::Output { lines, .. } => {
                self.render_output(f, area, theme, block, lines, panel);
            }
            PanelContent::Input { current_input, history, completions, cursor_position } => {
                self.render_input(f, area, theme, block, current_input, history, completions, *cursor_position);
            }
            PanelContent::Help { sections, search_term } => {
                self.render_help(f, area, theme, block, sections, search_term);
            }
            PanelContent::Status { metrics, alerts } => {
                self.render_status(f, area, theme, block, metrics, alerts);
            }
            PanelContent::Logs { entries, level_filter, search_filter } => {
                self.render_logs(f, area, theme, block, entries, level_filter, search_filter);
            }
        }

        // Note: Scrollbar rendering temporarily disabled due to borrowing constraints
    }

    /// Get border style based on configuration
    fn get_border_style(&self, style: &BorderStyle, theme: &Theme, focused: bool) -> Style {
        let color = if focused {
            theme.colors.accent
        } else {
            theme.colors.border
        };

        let mut style = Style::default().fg(color);
        
        if focused {
            style = style.add_modifier(Modifier::BOLD);
        }

        style
    }

    /// Render agent status panel
    fn render_agent_status(
        &self,
        f: &mut Frame<'_>,
        area: Rect,
        theme: &Theme,
        block: Block,
        agents: &[EnhancedAgentInfo],
        summary: &AgentSummary,
    ) {
        let inner_area = block.inner(area);
        f.render_widget(block, area);

        // Create summary line
        let summary_text = format!(
            "Agents: {}/{} active, {} idle, {} failed | Tasks: {}/{} completed | Avg: {:?}",
            summary.active_agents,
            summary.total_agents,
            summary.idle_agents,
            summary.failed_agents,
            summary.completed_tasks,
            summary.total_tasks,
            summary.avg_response_time
        );

        let summary_paragraph = Paragraph::new(summary_text)
            .style(Style::default().fg(theme.colors.muted))
            .alignment(Alignment::Center);

        // Split area for summary and agent list
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(inner_area);

        f.render_widget(summary_paragraph, layout[0]);

        // Render agent list
        let agent_items: Vec<ListItem> = agents.iter().map(|agent| {
            let status_icon = match agent.status {
                AgentStatus::Idle => "⚪",
                AgentStatus::Processing { .. } => "🟡",
                AgentStatus::Busy => "🟠",
                AgentStatus::Error { .. } => "🔴",
                AgentStatus::ShuttingDown => "⏹️",
                AgentStatus::Offline => "⚫",
            };

            let mut spans = vec![
                Span::styled(status_icon, Style::default()),
                Span::styled(format!(" {}", agent.name), Style::default().fg(theme.colors.foreground)),
            ];

            // Add task information if available
            if let Some(task) = &agent.current_task {
                spans.push(Span::styled(
                    format!(" - {}", task),
                    Style::default().fg(theme.colors.muted),
                ));
            }

            // Add progress bar if available
            if let Some(progress) = agent.progress {
                let progress_text = format!(" ({:.1}%)", progress * 100.0);
                spans.push(Span::styled(
                    progress_text,
                    Style::default().fg(theme.colors.accent),
                ));
            }

            // Add resource usage
            spans.push(Span::styled(
                format!(
                    " [CPU: {:.1}%, MEM: {}MB]",
                    agent.resource_usage.cpu_percent,
                    agent.resource_usage.memory_mb
                ),
                Style::default().fg(theme.colors.info),
            ));

            ListItem::new(Line::from(spans))
        }).collect();

        let agent_list = List::new(agent_items)
            .style(Style::default().fg(theme.colors.foreground));

        f.render_widget(agent_list, layout[1]);
    }

    /// Render output panel
    fn render_output(
        &self,
        f: &mut Frame<'_>,
        area: Rect,
        theme: &Theme,
        block: Block,
        lines: &[OutputLine],
        panel: &EnhancedPanel,
    ) {
        let inner_area = block.inner(area);
        f.render_widget(block, area);

        // Create styled lines
        let styled_lines: Vec<Line> = lines.iter().map(|line| {
            let style = match line.line_type {
                OutputType::Error => Style::default().fg(theme.colors.error),
                OutputType::Warning => Style::default().fg(theme.colors.warning),
                OutputType::Success => Style::default().fg(theme.colors.success),
                OutputType::Debug => Style::default().fg(theme.colors.muted),
                OutputType::Command => Style::default().fg(theme.colors.accent).add_modifier(Modifier::BOLD),
                OutputType::Result => Style::default().fg(theme.colors.info),
                OutputType::Normal => Style::default().fg(theme.colors.foreground),
            };

            let prefix = match line.line_type {
                OutputType::Error => "❌ ",
                OutputType::Warning => "⚠️ ",
                OutputType::Success => "✅ ",
                OutputType::Debug => "🔍 ",
                OutputType::Command => "$ ",
                OutputType::Result => "→ ",
                OutputType::Normal => "",
            };

            Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(&line.content, style),
            ])
        }).collect();

        let text = Text::from(styled_lines);
        let paragraph = Paragraph::new(text)
            .wrap(Wrap { trim: true })
            .scroll((panel.scroll_position as u16, 0));

        f.render_widget(paragraph, inner_area);
    }

    /// Render input panel
    fn render_input(
        &self,
        f: &mut Frame<'_>,
        area: Rect,
        theme: &Theme,
        block: Block,
        current_input: &str,
        _history: &[String],
        completions: &[String],
        cursor_position: usize,
    ) {
        let inner_area = block.inner(area);
        f.render_widget(block, area);

        // Split area for input and completions
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(inner_area);

        // Render input line with cursor
        let input_text = if cursor_position <= current_input.len() {
            let (before, after) = current_input.split_at(cursor_position);
            let cursor_char = after.chars().next().unwrap_or(' ');
            let after = after.get(1..).unwrap_or("");
            
            Line::from(vec![
                Span::styled("› ", Style::default().fg(theme.colors.accent)),
                Span::styled(before, Style::default().fg(theme.colors.foreground)),
                Span::styled(cursor_char.to_string(), Style::default().bg(theme.colors.accent).fg(theme.colors.background)),
                Span::styled(after, Style::default().fg(theme.colors.foreground)),
            ])
        } else {
            Line::from(vec![
                Span::styled("› ", Style::default().fg(theme.colors.accent)),
                Span::styled(current_input, Style::default().fg(theme.colors.foreground)),
                Span::styled(" ", Style::default().bg(theme.colors.accent)),
            ])
        };

        let input_paragraph = Paragraph::new(Text::from(vec![input_text]))
            .block(Block::default().borders(Borders::ALL).title(" Input "))
            .wrap(Wrap { trim: false });

        f.render_widget(input_paragraph, layout[0]);

        // Render completions if available
        if !completions.is_empty() {
            let completion_items: Vec<ListItem> = completions.iter()
                .take(10) // Limit to 10 completions
                .map(|comp| ListItem::new(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(comp, Style::default().fg(theme.colors.muted)),
                ])))
                .collect();

            let completion_list = List::new(completion_items)
                .block(Block::default().borders(Borders::ALL).title(" Completions "))
                .style(Style::default().fg(theme.colors.muted));

            f.render_widget(completion_list, layout[1]);
        }
    }

    /// Render help panel
    fn render_help(
        &self,
        f: &mut Frame<'_>,
        area: Rect,
        theme: &Theme,
        block: Block,
        sections: &[HelpSection],
        _search_term: &Option<String>,
    ) {
        let inner_area = block.inner(area);
        f.render_widget(block, area);

        let mut lines = Vec::new();
        
        for section in sections {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("▶ {}", section.title),
                    Style::default().fg(theme.colors.accent).add_modifier(Modifier::BOLD),
                ),
            ]));
            
            lines.push(Line::from(vec![
                Span::styled(section.content.clone(), Style::default().fg(theme.colors.foreground)),
            ]));

            if !section.examples.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("  Examples:", Style::default().fg(theme.colors.info)),
                ]));
                
                for example in &section.examples {
                    lines.push(Line::from(vec![
                        Span::styled("    • ", Style::default().fg(theme.colors.muted)),
                        Span::styled(example, Style::default().fg(theme.colors.success)),
                    ]));
                }
            }
            
            lines.push(Line::from("")); // Empty line between sections
        }

        let text = Text::from(lines);
        let paragraph = Paragraph::new(text).wrap(Wrap { trim: true });

        f.render_widget(paragraph, inner_area);
    }

    /// Render status panel
    fn render_status(
        &self,
        f: &mut Frame<'_>,
        area: Rect,
        theme: &Theme,
        block: Block,
        metrics: &SystemMetrics,
        alerts: &[StatusAlert],
    ) {
        let inner_area = block.inner(area);
        f.render_widget(block, area);

        // Split area for metrics and alerts
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(8), Constraint::Min(0)])
            .split(inner_area);

        // Render system metrics
        let metrics_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2); 4])
            .split(layout[0]);

        // CPU usage
        let cpu_gauge = Gauge::default()
            .block(Block::default().title(" CPU Usage ").borders(Borders::ALL))
            .gauge_style(Style::default().fg(theme.colors.accent))
            .percent((metrics.cpu_usage * 100.0) as u16)
            .label(format!("{:.1}%", metrics.cpu_usage * 100.0));
        f.render_widget(cpu_gauge, metrics_layout[0]);

        // Memory usage
        let mem_gauge = Gauge::default()
            .block(Block::default().title(" Memory Usage ").borders(Borders::ALL))
            .gauge_style(Style::default().fg(theme.colors.info))
            .percent((metrics.memory_usage * 100.0) as u16)
            .label(format!("{:.1}%", metrics.memory_usage * 100.0));
        f.render_widget(mem_gauge, metrics_layout[1]);

        // Disk usage
        let disk_gauge = Gauge::default()
            .block(Block::default().title(" Disk Usage ").borders(Borders::ALL))
            .gauge_style(Style::default().fg(theme.colors.warning))
            .percent((metrics.disk_usage * 100.0) as u16)
            .label(format!("{:.1}%", metrics.disk_usage * 100.0));
        f.render_widget(disk_gauge, metrics_layout[2]);

        // Network activity
        let network_text = format!(
            "Connections: {} | Uptime: {:?}",
            metrics.active_connections,
            metrics.uptime
        );
        let network_paragraph = Paragraph::new(network_text)
            .block(Block::default().title(" Network ").borders(Borders::ALL))
            .style(Style::default().fg(theme.colors.muted));
        f.render_widget(network_paragraph, metrics_layout[3]);

        // Render alerts
        if !alerts.is_empty() {
            let alert_items: Vec<ListItem> = alerts.iter().map(|alert| {
                let (icon, style) = match alert.level {
                    AlertLevel::Critical => ("🚨", Style::default().fg(theme.colors.error)),
                    AlertLevel::Error => ("❌", Style::default().fg(theme.colors.error)),
                    AlertLevel::Warning => ("⚠️", Style::default().fg(theme.colors.warning)),
                    AlertLevel::Info => ("ℹ️", Style::default().fg(theme.colors.info)),
                };

                ListItem::new(Line::from(vec![
                    Span::styled(icon, style),
                    Span::styled(format!(" {}", alert.message), style),
                    Span::styled(
                        format!(" [{}]", alert.source),
                        Style::default().fg(theme.colors.muted),
                    ),
                ]))
            }).collect();

            let alert_list = List::new(alert_items)
                .block(Block::default().title(" System Alerts ").borders(Borders::ALL));

            f.render_widget(alert_list, layout[1]);
        }
    }

    /// Render logs panel
    fn render_logs(
        &self,
        f: &mut Frame<'_>,
        area: Rect,
        theme: &Theme,
        block: Block,
        entries: &[LogEntry],
        _level_filter: &LogLevel,
        _search_filter: &Option<String>,
    ) {
        let inner_area = block.inner(area);
        f.render_widget(block, area);

        let log_items: Vec<ListItem> = entries.iter().map(|entry| {
            let (level_icon, level_style) = match entry.level {
                LogLevel::Error => ("❌", Style::default().fg(theme.colors.error)),
                LogLevel::Warning => ("⚠️", Style::default().fg(theme.colors.warning)),
                LogLevel::Info => ("ℹ️", Style::default().fg(theme.colors.info)),
                LogLevel::Debug => ("🔍", Style::default().fg(theme.colors.muted)),
                LogLevel::Trace => ("📝", Style::default().fg(theme.colors.muted)),
            };

            let timestamp = format!("{:02}:{:02}:{:02}",
                entry.timestamp.elapsed().as_secs() / 3600 % 24,
                entry.timestamp.elapsed().as_secs() / 60 % 60,
                entry.timestamp.elapsed().as_secs() % 60
            );

            ListItem::new(Line::from(vec![
                Span::styled(level_icon, level_style),
                Span::styled(
                    format!(" [{}] ", timestamp),
                    Style::default().fg(theme.colors.muted),
                ),
                Span::styled(
                    format!("{}: ", entry.source),
                    Style::default().fg(theme.colors.accent),
                ),
                Span::styled(&entry.message, Style::default().fg(theme.colors.foreground)),
            ]))
        }).collect();

        let log_list = List::new(log_items);
        f.render_widget(log_list, inner_area);
    }

    /// Render scrollbar for a panel
    fn render_scrollbar(&self, f: &mut Frame<'_>, area: Rect, theme: &Theme, panel_id: &str) {
        if let Some(scroll_state) = self.scroll_states.get_mut(panel_id) {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"))
                .track_symbol(Some("│"))
                .thumb_symbol("█")
                .style(Style::default().fg(theme.colors.muted))
                .thumb_style(Style::default().fg(theme.colors.accent));

            f.render_stateful_widget(scrollbar, area, scroll_state);
        }
    }

    /// Update animation states
    fn update_animations(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_update).as_secs_f32();

        for (_, animation_state) in self.animation_states.iter_mut() {
            // Update pulse phase
            animation_state.pulse_phase += delta * 2.0; // 2 Hz pulse
            if animation_state.pulse_phase > 2.0 * std::f32::consts::PI {
                animation_state.pulse_phase -= 2.0 * std::f32::consts::PI;
            }

            // Update blink state
            if now.duration_since(animation_state.last_blink) > Duration::from_millis(500) {
                animation_state.blink_state = !animation_state.blink_state;
                animation_state.last_blink = now;
            }
        }

        self.last_update = now;
    }

    /// Focus a panel
    pub fn focus_panel(&mut self, panel_id: &str) -> Result<(), String> {
        if self.panels.contains_key(panel_id) {
            // Remove from focus stack if already present
            self.layout_state.focus_stack.retain(|id| id != panel_id);
            // Add to top of focus stack
            self.layout_state.focus_stack.push(panel_id.to_string());
            
            // Update panel focus states
            for (id, panel) in &mut self.panels {
                panel.focused = id == panel_id;
            }
            
            Ok(())
        } else {
            Err(format!("Panel {} not found", panel_id))
        }
    }

    /// Get currently focused panel
    pub fn get_focused_panel(&self) -> Option<&str> {
        self.layout_state.focus_stack.last().map(|s| s.as_str())
    }

    /// Toggle panel visibility
    pub fn toggle_panel_visibility(&mut self, panel_id: &str) -> Result<bool, String> {
        if let Some(panel) = self.panels.get_mut(panel_id) {
            panel.visible = !panel.visible;
            Ok(panel.visible)
        } else {
            Err(format!("Panel {} not found", panel_id))
        }
    }

    /// Clear all panels
    pub fn clear_all_panels(&mut self) {
        self.panels.clear();
        self.scroll_states.clear();
        self.animation_states.clear();
        self.layout_state.focus_stack.clear();
    }
}

impl Default for EnhancedPanelManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PanelStyleConfig {
    fn default() -> Self {
        Self {
            border_style: BorderStyle::Normal,
            title_alignment: Alignment::Left,
            highlight_current_line: true,
            show_line_numbers: false,
            show_scrollbar: true,
            animated_borders: false,
            gradient_background: false,
            transparency: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_panel_manager_creation() {
        let manager = EnhancedPanelManager::new();
        assert_eq!(manager.panels.len(), 0);
        assert_eq!(manager.layout_state.current_layout, "default");
    }

    #[test]
    fn test_panel_focus() {
        let mut manager = EnhancedPanelManager::new();
        
        let panel = EnhancedPanel {
            id: "test_panel".to_string(),
            title: "Test Panel".to_string(),
            content: PanelContent::Output { lines: vec![], filter: None },
            style_config: PanelStyleConfig::default(),
            scroll_position: 0,
            max_items: None,
            auto_scroll: false,
            visible: true,
            focused: false,
            last_updated: Instant::now(),
            error_count: 0,
            warning_count: 0,
        };

        manager.add_panel(panel);
        assert!(manager.focus_panel("test_panel").is_ok());
        assert_eq!(manager.get_focused_panel(), Some("test_panel"));
        assert!(manager.panels["test_panel"].focused);
    }

    #[test]
    fn test_panel_visibility_toggle() {
        let mut manager = EnhancedPanelManager::new();
        
        let panel = EnhancedPanel {
            id: "test_panel".to_string(),
            title: "Test Panel".to_string(),
            content: PanelContent::Output { lines: vec![], filter: None },
            style_config: PanelStyleConfig::default(),
            scroll_position: 0,
            max_items: None,
            auto_scroll: false,
            visible: true,
            focused: false,
            last_updated: Instant::now(),
            error_count: 0,
            warning_count: 0,
        };

        manager.add_panel(panel);
        
        // Toggle visibility
        assert_eq!(manager.toggle_panel_visibility("test_panel").unwrap(), false);
        assert!(!manager.panels["test_panel"].visible);
        
        assert_eq!(manager.toggle_panel_visibility("test_panel").unwrap(), true);
        assert!(manager.panels["test_panel"].visible);
    }
}