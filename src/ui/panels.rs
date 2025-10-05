//! Panel management system for displaying different UI views.

use crate::{
    agents::{AgentStatus, TaskPriority},
    ui::blocks::OutputBlockCollection,
    ui::notifications::NotificationPanel,
    ui::themes::Theme,
};
use ratatui::{
    layout::{Constraint, Direction, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use std::collections::HashMap;

/// Types of panels available in the UI
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PanelType {
    AgentStatus,
    Output,
    Input,
    Notifications,
    Help,
    Settings,
    Logs,
    CodePreview,
    FileExplorer,
    TaskQueue,
    SessionHistory,
}

/// Layout modes for the UI
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutMode {
    /// Single panel takes full screen
    Single(PanelType),
    /// Two panels side by side
    SideBySide(PanelType, PanelType),
    /// Two panels stacked vertically
    TopBottom(PanelType, PanelType),
    /// Three panels with main on left, two on right stacked
    ThreePane(PanelType, PanelType, PanelType),
    /// Four panels in quadrants
    Quadrant(PanelType, PanelType, PanelType, PanelType),
    /// Custom layout with explicit areas
    Custom(Vec<(PanelType, Rect)>),
}

/// Resizable pane information
#[derive(Debug, Clone)]
pub struct ResizablePane {
    pub panel_type: PanelType,
    pub size: u16, // Percentage or fixed size
    pub min_size: u16,
    pub max_size: u16,
    pub resizable: bool,
}

/// Configuration for panel layout
#[derive(Debug, Clone)]
pub struct PanelLayout {
    pub panel_type: PanelType,
    pub constraints: Vec<Constraint>,
    pub direction: Direction,
    pub visible: bool,
    pub title: String,
    pub bordered: bool,
}

/// Manager for all UI panels
#[derive(Debug)]
pub struct PanelManager {
    layouts: HashMap<PanelType, PanelLayout>,
    current_focus: Option<PanelType>,
    agent_panel: AgentStatusPanel,
    notification_panel: NotificationPanel,
    output_blocks: OutputBlockCollection,
    help_visible: bool,
    layout_mode: LayoutMode,
    resizable_panes: HashMap<PanelType, ResizablePane>,
    screen_size: Rect,
    panel_areas: HashMap<PanelType, Rect>,
}

/// Panel for displaying agent status information
#[derive(Debug)]
pub struct AgentStatusPanel {
    agents: HashMap<String, AgentDisplayInfo>,
    expanded_agent: Option<String>,
    show_inactive: bool,
    sort_by: AgentSortBy,
}

/// Information about an agent for display purposes
#[derive(Debug, Clone)]
pub struct AgentDisplayInfo {
    pub name: String,
    pub agent_type: String,
    pub status: AgentStatus,
    pub current_task: Option<String>,
    pub task_priority: Option<TaskPriority>,
    pub progress: f64, // 0.0 to 1.0
    pub last_update: std::time::SystemTime,
    pub error_count: usize,
    pub completed_tasks: usize,
}

/// Sorting options for agent display
#[derive(Debug, Clone)]
pub enum AgentSortBy {
    Name,
    Status,
    LastUpdate,
    Priority,
    Progress,
}

impl Default for PanelLayout {
    fn default() -> Self {
        Self {
            panel_type: PanelType::Output,
            constraints: vec![Constraint::Percentage(100)],
            direction: Direction::Vertical,
            visible: true,
            title: String::new(),
            bordered: true,
        }
    }
}

impl PanelLayout {
    /// Create a new panel layout
    pub fn new(panel_type: PanelType) -> Self {
        let (title, constraints, direction) = match panel_type {
            PanelType::AgentStatus => (
                "Agent Status".to_string(),
                vec![Constraint::Percentage(30)],
                Direction::Vertical,
            ),
            PanelType::Output => (
                "Output".to_string(),
                vec![Constraint::Percentage(50)],
                Direction::Vertical,
            ),
            PanelType::Input => (
                "Input".to_string(),
                vec![Constraint::Length(3)],
                Direction::Vertical,
            ),
            PanelType::Notifications => (
                "Notifications".to_string(),
                vec![Constraint::Percentage(20)],
                Direction::Vertical,
            ),
            PanelType::Help => (
                "Help".to_string(),
                vec![Constraint::Percentage(100)],
                Direction::Vertical,
            ),
            PanelType::Settings => (
                "Settings".to_string(),
                vec![Constraint::Percentage(100)],
                Direction::Vertical,
            ),
            PanelType::Logs => (
                "Logs".to_string(),
                vec![Constraint::Percentage(100)],
                Direction::Vertical,
            ),
            PanelType::CodePreview => (
                "Code Preview".to_string(),
                vec![Constraint::Percentage(50)],
                Direction::Vertical,
            ),
            PanelType::FileExplorer => (
                "File Explorer".to_string(),
                vec![Constraint::Percentage(30)],
                Direction::Vertical,
            ),
            PanelType::TaskQueue => (
                "Task Queue".to_string(),
                vec![Constraint::Percentage(25)],
                Direction::Vertical,
            ),
            PanelType::SessionHistory => (
                "Session History".to_string(),
                vec![Constraint::Percentage(40)],
                Direction::Vertical,
            ),
        };

        Self {
            panel_type,
            title,
            constraints,
            direction,
            visible: true,
            bordered: true,
        }
    }

    /// Set panel constraints
    pub fn with_constraints(mut self, constraints: Vec<Constraint>) -> Self {
        self.constraints = constraints;
        self
    }

    /// Set panel direction
    pub fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Set panel visibility
    pub fn with_visibility(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set panel title
    pub fn with_title(mut self, title: String) -> Self {
        self.title = title;
        self
    }

    /// Set whether panel has borders
    pub fn with_borders(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }
}

impl AgentDisplayInfo {
    /// Create new agent display info
    pub fn new(name: String, agent_type: String) -> Self {
        Self {
            name,
            agent_type,
            status: AgentStatus::Idle,
            current_task: None,
            task_priority: None,
            progress: 0.0,
            last_update: std::time::SystemTime::now(),
            error_count: 0,
            completed_tasks: 0,
        }
    }

    /// Update agent status
    pub fn update_status(&mut self, status: AgentStatus) {
        self.status = status;
        self.last_update = std::time::SystemTime::now();
    }

    /// Set current task
    pub fn set_current_task(&mut self, task: Option<String>, priority: Option<TaskPriority>) {
        self.current_task = task;
        self.task_priority = priority;
        self.last_update = std::time::SystemTime::now();
    }

    /// Update progress
    pub fn update_progress(&mut self, progress: f64) {
        self.progress = progress.clamp(0.0, 1.0);
        self.last_update = std::time::SystemTime::now();
    }

    /// Increment error count
    pub fn increment_errors(&mut self) {
        self.error_count += 1;
        self.last_update = std::time::SystemTime::now();
    }

    /// Increment completed tasks
    pub fn increment_completed(&mut self) {
        self.completed_tasks += 1;
        self.progress = 0.0; // Reset progress after completion
        self.current_task = None;
        self.task_priority = None;
        self.last_update = std::time::SystemTime::now();
    }

    /// Get status color
    pub fn status_style(&self, theme: &Theme) -> Style {
        match self.status {
            AgentStatus::Idle => theme.secondary_style(),
            AgentStatus::Processing { task_id: _ } => theme.info_style(),
            AgentStatus::Error { message: _ } => theme.error_style(),
            AgentStatus::Busy => theme.warning_style(),
            AgentStatus::Offline => theme.muted_style(),
            AgentStatus::ShuttingDown => theme.warning_style(),
        }
    }

    /// Get priority color
    pub fn priority_style(&self, theme: &Theme) -> Style {
        match self.task_priority {
            Some(TaskPriority::Low) => theme.secondary_style(),
            Some(TaskPriority::Normal) => theme.primary_style(),
            Some(TaskPriority::High) => theme.warning_style(),
            Some(TaskPriority::Critical) => theme.error_style(),
            None => theme.secondary_style(),
        }
    }

    /// Format agent info for display
    pub fn format_for_display(&self, theme: &Theme, expanded: bool) -> Vec<Line<'_>> {
        let mut lines = Vec::new();

        // Agent name and type
        let name_line = Line::from(vec![
            Span::styled(
                &self.name,
                theme.primary_style().add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" ({})", self.agent_type), theme.secondary_style()),
        ]);
        lines.push(name_line);

        if expanded {
            // Status with icon
            let status_icon = match self.status {
                AgentStatus::Idle => "💤",
                AgentStatus::Processing { task_id: _ } => "⚙️",
                AgentStatus::Error { message: _ } => "❌",
                AgentStatus::Busy => "⌨️",
                AgentStatus::Offline => "⭕",
                AgentStatus::ShuttingDown => "🟠",
            };

            let status_line = Line::from(vec![
                Span::styled("  Status: ", theme.label_style()),
                Span::styled(status_icon, self.status_style(theme)),
                Span::styled(format!(" {:?}", self.status), self.status_style(theme)),
            ]);
            lines.push(status_line);

            // Current task
            if let Some(ref task) = self.current_task {
                let task_line = Line::from(vec![
                    Span::styled("  Task: ", theme.label_style()),
                    Span::styled(task, theme.value_style()),
                ]);
                lines.push(task_line);

                // Priority if available
                if let Some(priority) = &self.task_priority {
                    let priority_line = Line::from(vec![
                        Span::styled("  Priority: ", theme.label_style()),
                        Span::styled(format!("{:?}", priority), self.priority_style(theme)),
                    ]);
                    lines.push(priority_line);
                }

                // Progress bar (only if processing)
                if matches!(self.status, AgentStatus::Processing { task_id: _ })
                    && self.progress > 0.0
                {
                    let progress_text = format!("  Progress: {:.1}%", self.progress * 100.0);
                    lines.push(Line::from(Span::styled(progress_text, theme.value_style())));
                }
            }

            // Statistics
            let stats_line = Line::from(vec![
                Span::styled("  Completed: ", theme.label_style()),
                Span::styled(self.completed_tasks.to_string(), theme.success_style()),
                Span::styled("  Errors: ", theme.label_style()),
                Span::styled(self.error_count.to_string(), theme.error_style()),
            ]);
            lines.push(stats_line);

            // Last update
            if let Ok(elapsed) = self.last_update.elapsed() {
                let time_str = if elapsed.as_secs() < 60 {
                    format!("{}s ago", elapsed.as_secs())
                } else if elapsed.as_secs() < 3600 {
                    format!("{}m ago", elapsed.as_secs() / 60)
                } else {
                    format!("{}h ago", elapsed.as_secs() / 3600)
                };

                let update_line = Line::from(vec![
                    Span::styled("  Updated: ", theme.label_style()),
                    Span::styled(time_str, theme.timestamp_style()),
                ]);
                lines.push(update_line);
            }
        } else {
            // Compact view - status and current task on same line
            let mut spans = vec![
                Span::styled("  ", theme.secondary_style()),
                Span::styled(format!("{:?}", self.status), self.status_style(theme)),
            ];

            if let Some(ref task) = self.current_task {
                spans.push(Span::styled(" - ", theme.secondary_style()));
                spans.push(Span::styled(
                    if task.len() > 30 {
                        format!("{}...", &task[..27])
                    } else {
                        task.clone()
                    },
                    theme.value_style(),
                ));
            }

            lines.push(Line::from(spans));
        }

        lines
    }
}

impl AgentStatusPanel {
    /// Create a new agent status panel
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            expanded_agent: None,
            show_inactive: true,
            sort_by: AgentSortBy::Name,
        }
    }

    /// Add or update an agent
    pub fn update_agent(&mut self, name: String, info: AgentDisplayInfo) {
        self.agents.insert(name, info);
    }

    /// Remove an agent
    pub fn remove_agent(&mut self, name: &str) -> bool {
        self.agents.remove(name).is_some()
    }

    /// Toggle expanded view for an agent
    pub fn toggle_expanded(&mut self, agent_name: &str) {
        if self.expanded_agent.as_ref() == Some(&agent_name.to_string()) {
            self.expanded_agent = None;
        } else {
            self.expanded_agent = Some(agent_name.to_string());
        }
    }

    /// Set sorting method
    pub fn set_sort_by(&mut self, sort_by: AgentSortBy) {
        self.sort_by = sort_by;
    }

    /// Toggle showing inactive agents
    pub fn toggle_show_inactive(&mut self) {
        self.show_inactive = !self.show_inactive;
    }

    /// Get sorted agents for display
    fn get_sorted_agents(&self) -> Vec<(&String, &AgentDisplayInfo)> {
        let mut agents: Vec<(&String, &AgentDisplayInfo)> = self.agents.iter().collect();

        // Filter inactive if needed
        if !self.show_inactive {
            agents.retain(|(_, info)| !matches!(info.status, AgentStatus::Idle));
        }

        // Sort agents
        agents.sort_by(|a, b| {
            match self.sort_by {
                AgentSortBy::Name => a.0.cmp(b.0),
                AgentSortBy::Status => {
                    a.1.status
                        .partial_cmp(&b.1.status)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
                AgentSortBy::LastUpdate => b.1.last_update.cmp(&a.1.last_update), // Most recent first
                AgentSortBy::Priority => {
                    match (&a.1.task_priority, &b.1.task_priority) {
                        (Some(ap), Some(bp)) => bp.cmp(ap), // Higher priority first
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    }
                }
                AgentSortBy::Progress => {
                    b.1.progress
                        .partial_cmp(&a.1.progress)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
            }
        });

        agents
    }

    /// Render the agent status panel
    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let agents = self.get_sorted_agents();

        // Create list items
        let mut items = Vec::new();
        for (name, info) in agents {
            let expanded = self.expanded_agent.as_ref() == Some(name);
            let lines = info.format_for_display(theme, expanded);

            for line in lines {
                items.push(ListItem::new(line));
            }

            // Add separator between agents if expanded
            if expanded {
                items.push(ListItem::new(Line::from("")));
            }
        }

        // Create the agent list
        let agent_list = List::new(items).block(
            Block::default()
                .title(format!("Agents ({})", self.agents.len()))
                .borders(Borders::ALL)
                .style(theme.border_style()),
        );

        f.render_widget(agent_list, area);
    }

    /// Get agent count by status
    pub fn count_by_status(&self, status: &AgentStatus) -> usize {
        self.agents
            .values()
            .filter(|info| info.status == *status)
            .count()
    }

    /// Get all agent names
    pub fn get_agent_names(&self) -> Vec<String> {
        self.agents.keys().cloned().collect()
    }

    /// Check if agent exists
    pub fn has_agent(&self, name: &str) -> bool {
        self.agents.contains_key(name)
    }
}

impl PanelManager {
    /// Create a new panel manager
    pub fn new() -> Self {
        let mut layouts = HashMap::new();

        // Add default panel layouts
        layouts.insert(
            PanelType::AgentStatus,
            PanelLayout::new(PanelType::AgentStatus).with_visibility(false), // Hide agent status by default
        );
        layouts.insert(PanelType::Output, PanelLayout::new(PanelType::Output).with_constraints(vec![Constraint::Min(5)]));
        layouts.insert(PanelType::Input, PanelLayout::new(PanelType::Input));
        layouts.insert(
            PanelType::Notifications,
            PanelLayout::new(PanelType::Notifications).with_visibility(false), // Hide notifications by default
        );
        layouts.insert(PanelType::Help, PanelLayout::new(PanelType::Help));
        layouts.insert(PanelType::Settings, PanelLayout::new(PanelType::Settings));
        layouts.insert(PanelType::Logs, PanelLayout::new(PanelType::Logs));

        Self {
            layouts,
            current_focus: Some(PanelType::Input),
            agent_panel: AgentStatusPanel::new(),
            notification_panel: NotificationPanel::new(5000), // 5 second auto-dismiss
            output_blocks: OutputBlockCollection::new(1000),  // Max 1000 blocks
            help_visible: false,
            layout_mode: LayoutMode::TopBottom(PanelType::Output, PanelType::Input),
            resizable_panes: HashMap::new(),
            screen_size: Rect::default(),
            panel_areas: HashMap::new(),
        }
    }

    /// Set panel layout
    pub fn set_panel_layout(&mut self, layout: PanelLayout) {
        self.layouts.insert(layout.panel_type.clone(), layout);
    }

    /// Get panel layout
    pub fn get_panel_layout(&self, panel_type: &PanelType) -> Option<&PanelLayout> {
        self.layouts.get(panel_type)
    }

    /// Set current focus
    pub fn set_focus(&mut self, panel_type: Option<PanelType>) {
        self.current_focus = panel_type;
    }

    /// Get current focus
    pub fn get_focus(&self) -> Option<&PanelType> {
        self.current_focus.as_ref()
    }

    /// Toggle panel visibility
    pub fn toggle_panel_visibility(&mut self, panel_type: &PanelType) {
        if let Some(layout) = self.layouts.get_mut(panel_type) {
            layout.visible = !layout.visible;
        }
    }

    /// Show help panel
    pub fn show_help(&mut self) {
        self.help_visible = true;
    }

    /// Hide help panel
    pub fn hide_help(&mut self) {
        self.help_visible = false;
    }

    /// Toggle help visibility
    pub fn toggle_help(&mut self) {
        self.help_visible = !self.help_visible;
    }

    /// Get agent panel
    pub fn agent_panel(&mut self) -> &mut AgentStatusPanel {
        &mut self.agent_panel
    }

    /// Get notification panel
    pub fn notification_panel(&mut self) -> &mut NotificationPanel {
        &mut self.notification_panel
    }

    /// Get output blocks
    pub fn output_blocks(&mut self) -> &mut OutputBlockCollection {
        &mut self.output_blocks
    }

    /// Calculate main layout constraints
    pub fn calculate_layout_constraints(&self) -> Vec<Constraint> {
        let mut constraints = Vec::new();

        // Add constraints for visible panels in order
        let panel_order = vec![
            PanelType::AgentStatus,
            PanelType::Notifications,
            PanelType::Output,
            PanelType::Input,
        ];

        for panel_type in panel_order {
            if let Some(layout) = self.layouts.get(&panel_type) {
                if layout.visible {
                    constraints.extend(layout.constraints.clone());
                }
            }
        }

        // Ensure we have at least one constraint
        if constraints.is_empty() {
            constraints.push(Constraint::Percentage(100));
        }

        constraints
    }

    /// Render help overlay
    pub fn render_help_overlay(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        if !self.help_visible {
            return;
        }

        // Calculate overlay area (80% of screen, centered)
        let overlay_area = Rect {
            x: area.width / 10,
            y: area.height / 10,
            width: area.width * 8 / 10,
            height: area.height * 8 / 10,
        };

        // Clear the area
        f.render_widget(Clear, overlay_area);

        // Help content
        let help_text = vec![
            "DevKit Dashboard - Keybindings Help",
            "",
            "=== QUICK START ===",
            "  ?  or  F1     - Show/hide this help",
            "  q  or  Ctrl+C - Quit application",
            "  i             - Enter input mode (type commands)",
            "  :             - Enter command mode (system commands)",
            "  Tab           - Cycle through panels",
            "",
            "=== INPUT MODES ===",
            "  Input Mode (press 'i'):",
            "    • Type natural language commands",
            "    • Examples: 'generate a function', 'debug this code'",
            "    • Press Enter to execute, Escape to cancel",
            "",
            "  Command Mode (press ':'):",
            "    • Type system commands starting with /",
            "    • /help - Show command reference",
            "    • /status - Show system status",
            "    • /clear - Clear output",
            "    • /quit - Exit dashboard",
            "",
            "=== PANEL NAVIGATION ===",
            "  Tab           - Cycle panel focus",
            "  Shift+Tab     - Cycle focus backwards",
            "",
            "=== AGENT PANEL ===",
            "  Enter         - Expand/collapse agent details",
            "  s             - Cycle sorting method",
            "  i             - Toggle show inactive agents",
            "",
            "=== NOTIFICATION PANEL ===",
            "  d             - Dismiss selected notification",
            "  c             - Clear all dismissible notifications",
            "  Del           - Delete selected notification",
            "",
            "=== OUTPUT PANEL ===",
            "  PageUp        - Scroll output up",
            "  PageDown      - Scroll output down",
            "  Home          - Go to start of output",
            "  End           - Go to end of output",
            "  Ctrl+L        - Clear output",
            "",
            "=== GETTING STARTED ===",
            "  1. Press 'i' to enter input mode",
            "  2. Type what you want in natural language",
            "  3. Press Enter to let agents process it",
            "  4. View results in the output panel",
            "",
            "Press '?' or F1 again to close this help",
        ];

        let help_lines: Vec<Line> = help_text.iter().map(|line| Line::from(*line)).collect();

        let help_paragraph = Paragraph::new(help_lines)
            .block(
                Block::default()
                    .title("Help")
                    .borders(Borders::ALL)
                    .style(theme.border_style()),
            )
            .wrap(Wrap { trim: true })
            .style(theme.primary_style());

        f.render_widget(help_paragraph, overlay_area);
    }
}

impl Default for PanelManager {
    fn default() -> Self {
        Self::new()
    }
}
