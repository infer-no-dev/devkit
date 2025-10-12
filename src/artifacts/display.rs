//! Artifact Display and Visualization Components
//!
//! This module provides comprehensive UI components for displaying and visualizing
//! code artifacts with syntax highlighting, diff views, and interactive browsing.

use crate::artifacts::manager::{EnhancedArtifact, QualityMetrics, UsageStats, VersionType};
use crate::ui::syntax::SyntaxHighlighter;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        block::Position, Block, BorderType, Borders, Clear, Gauge, List, ListItem, ListState,
        Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Tabs, Wrap,
    },
    Frame,
};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, trace};

/// Artifact display configuration
#[derive(Debug, Clone)]
pub struct ArtifactDisplayConfig {
    /// Enable syntax highlighting
    pub syntax_highlighting: bool,
    /// Show line numbers
    pub show_line_numbers: bool,
    /// Show metadata panel
    pub show_metadata: bool,
    /// Show quality metrics
    pub show_quality_metrics: bool,
    /// Tab size for indentation
    pub tab_size: usize,
    /// Maximum content preview length
    pub max_preview_length: usize,
    /// Theme for syntax highlighting
    pub theme: DisplayTheme,
}

impl Default for ArtifactDisplayConfig {
    fn default() -> Self {
        Self {
            syntax_highlighting: true,
            show_line_numbers: true,
            show_metadata: true,
            show_quality_metrics: true,
            tab_size: 4,
            max_preview_length: 10000,
            theme: DisplayTheme::Dark,
        }
    }
}

/// Display themes
#[derive(Debug, Clone, PartialEq)]
pub enum DisplayTheme {
    Dark,
    Light,
    HighContrast,
    Monokai,
    Solarized,
}

/// Artifact viewer state
#[derive(Debug)]
pub struct ArtifactViewerState {
    /// Currently selected artifact
    pub selected_artifact: Option<EnhancedArtifact>,
    /// List of artifacts in the browser
    pub artifacts: Vec<EnhancedArtifact>,
    /// List selection state
    pub list_state: ListState,
    /// Scroll state for content view
    pub content_scroll_state: ScrollbarState,
    /// Current scroll position
    pub scroll_offset: usize,
    /// Active tab index
    pub active_tab: usize,
    /// Configuration
    pub config: ArtifactDisplayConfig,
    /// Search query
    pub search_query: Option<String>,
    /// View mode
    pub view_mode: ViewMode,
    /// Comparison state
    pub comparison_state: Option<ComparisonState>,
}

/// View modes for artifact display
#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    /// Single artifact view
    Single,
    /// Side-by-side comparison
    SideBySide,
    /// Diff view
    Diff,
    /// Grid layout for multiple artifacts
    Grid,
    /// List view with previews
    List,
}

/// Comparison state for diff views
#[derive(Debug, Clone)]
pub struct ComparisonState {
    /// Left artifact for comparison
    pub left_artifact: EnhancedArtifact,
    /// Right artifact for comparison
    pub right_artifact: EnhancedArtifact,
    /// Computed diff
    pub diff: ArtifactDiff,
    /// Diff scroll state
    pub diff_scroll_state: ScrollbarState,
}

/// Diff representation between artifacts
#[derive(Debug, Clone)]
pub struct ArtifactDiff {
    /// Diff hunks
    pub hunks: Vec<DiffHunk>,
    /// Statistics
    pub stats: DiffStats,
}

/// Single diff hunk
#[derive(Debug, Clone)]
pub struct DiffHunk {
    /// Old start line
    pub old_start: usize,
    /// Old line count
    pub old_count: usize,
    /// New start line
    pub new_start: usize,
    /// New line count
    pub new_count: usize,
    /// Lines in this hunk
    pub lines: Vec<DiffLine>,
}

/// Single line in a diff
#[derive(Debug, Clone)]
pub struct DiffLine {
    /// Line type
    pub line_type: DiffLineType,
    /// Line content
    pub content: String,
    /// Old line number (if applicable)
    pub old_line: Option<usize>,
    /// New line number (if applicable)
    pub new_line: Option<usize>,
}

/// Types of diff lines
#[derive(Debug, Clone, PartialEq)]
pub enum DiffLineType {
    /// Unchanged line
    Context,
    /// Added line
    Addition,
    /// Removed line
    Deletion,
    /// Modified line
    Modified,
}

/// Diff statistics
#[derive(Debug, Clone)]
pub struct DiffStats {
    /// Number of additions
    pub additions: usize,
    /// Number of deletions
    pub deletions: usize,
    /// Number of modifications
    pub modifications: usize,
    /// Total changed lines
    pub total_changes: usize,
}

impl ArtifactViewerState {
    /// Create new viewer state
    pub fn new(config: ArtifactDisplayConfig) -> Self {
        Self {
            selected_artifact: None,
            artifacts: Vec::new(),
            list_state: ListState::default(),
            content_scroll_state: ScrollbarState::default(),
            scroll_offset: 0,
            active_tab: 0,
            config,
            search_query: None,
            view_mode: ViewMode::Single,
            comparison_state: None,
        }
    }

    /// Set artifacts to display
    pub fn set_artifacts(&mut self, artifacts: Vec<EnhancedArtifact>) {
        self.artifacts = artifacts;
        if !self.artifacts.is_empty() && self.selected_artifact.is_none() {
            self.list_state.select(Some(0));
            self.selected_artifact = Some(self.artifacts[0].clone());
        }
        self.update_scroll_state();
    }

    /// Select an artifact by index
    pub fn select_artifact(&mut self, index: usize) {
        if index < self.artifacts.len() {
            self.list_state.select(Some(index));
            self.selected_artifact = Some(self.artifacts[index].clone());
            self.scroll_offset = 0;
            self.update_scroll_state();
        }
    }

    /// Scroll content up
    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
        self.update_scroll_state();
    }

    /// Scroll content down
    pub fn scroll_down(&mut self, amount: usize) {
        if let Some(artifact) = &self.selected_artifact {
            let max_scroll = artifact.artifact.content.lines().count().saturating_sub(1);
            self.scroll_offset = std::cmp::min(self.scroll_offset + amount, max_scroll);
            self.update_scroll_state();
        }
    }

    /// Switch to next tab
    pub fn next_tab(&mut self) {
        self.active_tab = (self.active_tab + 1) % 4; // 4 tabs: Content, Metadata, Quality, History
    }

    /// Switch to previous tab
    pub fn previous_tab(&mut self) {
        if self.active_tab == 0 {
            self.active_tab = 3;
        } else {
            self.active_tab -= 1;
        }
    }

    /// Set view mode
    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
    }

    /// Start comparison between two artifacts
    pub fn start_comparison(&mut self, left: EnhancedArtifact, right: EnhancedArtifact) {
        let diff = self.compute_diff(&left, &right);
        self.comparison_state = Some(ComparisonState {
            left_artifact: left,
            right_artifact: right,
            diff,
            diff_scroll_state: ScrollbarState::default(),
        });
        self.view_mode = ViewMode::Diff;
    }

    /// Update scroll state based on content
    fn update_scroll_state(&mut self) {
        if let Some(artifact) = &self.selected_artifact {
            let total_lines = artifact.artifact.content.lines().count();
            self.content_scroll_state = self.content_scroll_state.content_length(total_lines);
        }
    }

    /// Compute diff between two artifacts
    fn compute_diff(&self, left: &EnhancedArtifact, right: &EnhancedArtifact) -> ArtifactDiff {
        let left_lines: Vec<&str> = left.artifact.content.lines().collect();
        let right_lines: Vec<&str> = right.artifact.content.lines().collect();

        // Simple diff implementation - in practice, you'd use a proper diff library
        let mut hunks = Vec::new();
        let mut stats = DiffStats {
            additions: 0,
            deletions: 0,
            modifications: 0,
            total_changes: 0,
        };

        // This is a simplified unified diff - a real implementation would use proper diff algorithms
        let max_lines = std::cmp::max(left_lines.len(), right_lines.len());
        let mut diff_lines = Vec::new();

        for i in 0..max_lines {
            let left_line = left_lines.get(i);
            let right_line = right_lines.get(i);

            match (left_line, right_line) {
                (Some(left), Some(right)) => {
                    if left == right {
                        diff_lines.push(DiffLine {
                            line_type: DiffLineType::Context,
                            content: left.to_string(),
                            old_line: Some(i + 1),
                            new_line: Some(i + 1),
                        });
                    } else {
                        diff_lines.push(DiffLine {
                            line_type: DiffLineType::Deletion,
                            content: left.to_string(),
                            old_line: Some(i + 1),
                            new_line: None,
                        });
                        diff_lines.push(DiffLine {
                            line_type: DiffLineType::Addition,
                            content: right.to_string(),
                            old_line: None,
                            new_line: Some(i + 1),
                        });
                        stats.modifications += 1;
                    }
                }
                (Some(left), None) => {
                    diff_lines.push(DiffLine {
                        line_type: DiffLineType::Deletion,
                        content: left.to_string(),
                        old_line: Some(i + 1),
                        new_line: None,
                    });
                    stats.deletions += 1;
                }
                (None, Some(right)) => {
                    diff_lines.push(DiffLine {
                        line_type: DiffLineType::Addition,
                        content: right.to_string(),
                        old_line: None,
                        new_line: Some(i + 1),
                    });
                    stats.additions += 1;
                }
                (None, None) => break,
            }
        }

        stats.total_changes = stats.additions + stats.deletions + stats.modifications;

        if !diff_lines.is_empty() {
            hunks.push(DiffHunk {
                old_start: 1,
                old_count: left_lines.len(),
                new_start: 1,
                new_count: right_lines.len(),
                lines: diff_lines,
            });
        }

        ArtifactDiff { hunks, stats }
    }
}

/// Artifact display widget
pub struct ArtifactDisplay<'a> {
    state: &'a mut ArtifactViewerState,
}

impl<'a> ArtifactDisplay<'a> {
    /// Create new artifact display
    pub fn new(state: &'a mut ArtifactViewerState) -> Self {
        Self { state }
    }

    /// Render the artifact display
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        match self.state.view_mode {
            ViewMode::Single => self.render_single_view(f, area),
            ViewMode::SideBySide => self.render_side_by_side_view(f, area),
            ViewMode::Diff => self.render_diff_view(f, area),
            ViewMode::Grid => self.render_grid_view(f, area),
            ViewMode::List => self.render_list_view(f, area),
        }
    }

    fn render_single_view(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(25), Constraint::Min(0)].as_ref())
            .split(area);

        // Render artifact list
        self.render_artifact_list(f, chunks[0]);

        if self.state.selected_artifact.is_some() {
            // Render artifact content area
            self.render_artifact_content(f, chunks[1]);
        }
    }

    fn render_side_by_side_view(&mut self, f: &mut Frame, area: Rect) {
        if let Some(comparison) = &self.state.comparison_state {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(area);

            // Render left artifact
            self.render_single_artifact(f, chunks[0], &comparison.left_artifact, "Left");

            // Render right artifact
            self.render_single_artifact(f, chunks[1], &comparison.right_artifact, "Right");
        }
    }

    fn render_diff_view(&mut self, f: &mut Frame, area: Rect) {
        if let Some(comparison) = &self.state.comparison_state {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(area);

            // Render diff header
            self.render_diff_header(f, chunks[0], comparison);

            // Render diff content
            self.render_diff_content(f, chunks[1], comparison);
        }
    }

    fn render_grid_view(&mut self, f: &mut Frame, area: Rect) {
        // Calculate grid dimensions
        let artifacts_count = self.state.artifacts.len();
        if artifacts_count == 0 {
            return;
        }

        let cols = ((artifacts_count as f64).sqrt().ceil() as usize).max(1);
        let rows = (artifacts_count + cols - 1) / cols;

        let row_constraints: Vec<Constraint> = (0..rows)
            .map(|_| Constraint::Percentage(100 / rows as u16))
            .collect();

        let row_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(area);

        for (row_idx, row_area) in row_chunks.iter().enumerate() {
            let col_constraints: Vec<Constraint> = (0..cols)
                .map(|_| Constraint::Percentage(100 / cols as u16))
                .collect();

            let col_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(col_constraints)
                .split(*row_area);

            for (col_idx, col_area) in col_chunks.iter().enumerate() {
                let artifact_idx = row_idx * cols + col_idx;
                if artifact_idx < self.state.artifacts.len() {
                    self.render_artifact_preview(f, *col_area, &self.state.artifacts[artifact_idx]);
                }
            }
        }
    }

    fn render_list_view(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .state
            .artifacts
            .iter()
            .enumerate()
            .map(|(i, artifact)| {
                let style = if Some(i) == self.state.list_state.selected() {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };

                let content = vec![Line::from(vec![
                    Span::styled(&artifact.artifact.name, style.add_modifier(Modifier::BOLD)),
                    Span::styled(
                        format!(" ({})", artifact.metadata.language.as_deref().unwrap_or("unknown")),
                        style.fg(Color::Gray),
                    ),
                ])];

                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Artifacts")
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD));

        f.render_stateful_widget(list, area, &mut self.state.list_state);
    }

    fn render_artifact_list(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .state
            .artifacts
            .iter()
            .enumerate()
            .map(|(i, artifact)| {
                let is_selected = Some(i) == self.state.list_state.selected();
                let style = if is_selected {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };

                let quality_indicator = if let Some(ref metrics) = artifact.quality_metrics {
                    match metrics.maintainability.unwrap_or(0.0) {
                        score if score >= 80.0 => "🟢",
                        score if score >= 60.0 => "🟡",
                        _ => "🔴",
                    }
                } else {
                    "⚪"
                };

                ListItem::new(vec![Line::from(vec![
                    Span::styled(quality_indicator, style),
                    Span::raw(" "),
                    Span::styled(&artifact.artifact.name, style),
                ])])
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Artifacts")
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD));

        f.render_stateful_widget(list, area, &mut self.state.list_state);
    }

    fn render_artifact_content(&mut self, f: &mut Frame, area: Rect) {
        if let Some(artifact) = &self.state.selected_artifact.clone() {
            let tabs = vec!["Content", "Metadata", "Quality", "History"];
            let titles: Vec<Line> = tabs
                .iter()
                .map(|t| Line::from(*t))
                .collect();

            let tab_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(area);

            let tabs_widget = Tabs::new(titles)
                .block(Block::default().borders(Borders::ALL).title("Artifact Details"))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow))
                .select(self.state.active_tab);

            f.render_widget(tabs_widget, tab_chunks[0]);

            match self.state.active_tab {
                0 => self.render_content_tab(f, tab_chunks[1], artifact),
                1 => self.render_metadata_tab(f, tab_chunks[1], artifact),
                2 => self.render_quality_tab(f, tab_chunks[1], artifact),
                3 => self.render_history_tab(f, tab_chunks[1], artifact),
                _ => {}
            }
        }
    }

    fn render_content_tab(&mut self, f: &mut Frame, area: Rect, artifact: &EnhancedArtifact) {
        let content_lines: Vec<&str> = artifact.artifact.content.lines().collect();
        let visible_lines: Vec<&str> = content_lines
            .iter()
            .skip(self.state.scroll_offset)
            .take(area.height as usize - 2) // Account for borders
            .copied()
            .collect();

        let content_text = if self.state.config.syntax_highlighting {
            self.apply_syntax_highlighting(&visible_lines, &artifact.metadata.language)
        } else {
            Text::from(visible_lines.join("\n"))
        };

        let content_widget = Paragraph::new(content_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(
                        "Content - {} lines",
                        content_lines.len()
                    ))
                    .border_type(BorderType::Rounded),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(content_widget, area);

        // Render scrollbar
        if content_lines.len() > area.height as usize - 2 {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None);
            f.render_stateful_widget(
                scrollbar,
                area.inner(Margin { horizontal: 0, vertical: 1 }),
                &mut self.state.content_scroll_state.position(self.state.scroll_offset),
            );
        }
    }

    fn render_metadata_tab(&self, f: &mut Frame, area: Rect, artifact: &EnhancedArtifact) {
        let metadata = &artifact.metadata;
        let created_at = format_timestamp(metadata.created_at);
        let modified_at = format_timestamp(metadata.modified_at);

        let metadata_text = Text::from(vec![
            Line::from(vec![
                Span::styled("Creator Agent: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&metadata.creator_agent.name),
            ]),
            Line::from(vec![
                Span::styled("Language: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(metadata.language.as_deref().unwrap_or("unknown")),
            ]),
            Line::from(vec![
                Span::styled("Created: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(created_at),
            ]),
            Line::from(vec![
                Span::styled("Modified: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(modified_at),
            ]),
            Line::from(vec![
                Span::styled("Size: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{} bytes", metadata.size_bytes)),
            ]),
            Line::from(vec![
                Span::styled("Lines: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{}", metadata.line_count)),
            ]),
            Line::from(vec![
                Span::styled("Version: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&artifact.version.version),
            ]),
            Line::from(vec![
                Span::styled("Version Type: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{:?}", artifact.version.version_type)),
            ]),
        ]);

        let metadata_widget = Paragraph::new(metadata_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Metadata")
                    .border_type(BorderType::Rounded),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(metadata_widget, area);
    }

    fn render_quality_tab(&self, f: &mut Frame, area: Rect, artifact: &EnhancedArtifact) {
        if let Some(ref metrics) = artifact.quality_metrics {
            self.render_quality_metrics(f, area, metrics);
        } else {
            let no_metrics_text = Text::from("No quality metrics available");
            let widget = Paragraph::new(no_metrics_text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Quality Metrics")
                        .border_type(BorderType::Rounded),
                )
                .alignment(Alignment::Center);
            f.render_widget(widget, area);
        }
    }

    fn render_history_tab(&self, f: &mut Frame, area: Rect, artifact: &EnhancedArtifact) {
        let usage = &artifact.usage_stats;
        let history_text = Text::from(vec![
            Line::from(vec![
                Span::styled("Access Count: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{}", usage.access_count)),
            ]),
            Line::from(vec![
                Span::styled("Modifications: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{}", usage.modification_count)),
            ]),
            Line::from(vec![
                Span::styled("Copies: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{}", usage.copy_count)),
            ]),
            Line::from(vec![
                Span::styled("Exports: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{}", usage.export_count)),
            ]),
            Line::from(""),
            Line::from("Accessing Agents:"),
        ]);

        let widget = Paragraph::new(history_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Usage History")
                    .border_type(BorderType::Rounded),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(widget, area);
    }

    fn render_quality_metrics(&self, f: &mut Frame, area: Rect, metrics: &QualityMetrics) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(area.inner(Margin { horizontal: 1, vertical: 1 }));

        // Maintainability
        if let Some(maintainability) = metrics.maintainability {
            let gauge = Gauge::default()
                .block(Block::default().title("Maintainability").borders(Borders::ALL))
                .gauge_style(
                    Style::default()
                        .fg(if maintainability >= 80.0 {
                            Color::Green
                        } else if maintainability >= 60.0 {
                            Color::Yellow
                        } else {
                            Color::Red
                        })
                        .add_modifier(Modifier::BOLD),
                )
                .percent((maintainability as u16).min(100));
            f.render_widget(gauge, chunks[0]);
        }

        // Security Score
        if let Some(security) = metrics.security_score {
            let gauge = Gauge::default()
                .block(Block::default().title("Security Score").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
                .percent((security as u16).min(100));
            f.render_widget(gauge, chunks[1]);
        }

        // Performance Score
        if let Some(performance) = metrics.performance_score {
            let gauge = Gauge::default()
                .block(Block::default().title("Performance").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .percent((performance as u16).min(100));
            f.render_widget(gauge, chunks[2]);
        }

        // Technical Debt
        if let Some(debt) = metrics.technical_debt {
            let gauge = Gauge::default()
                .block(Block::default().title("Technical Debt").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                .percent((debt as u16).min(100));
            f.render_widget(gauge, chunks[3]);
        }

        // Complexity
        if let Some(complexity) = metrics.complexity {
            let complexity_text = Text::from(format!("Cyclomatic Complexity: {}", complexity));
            let widget = Paragraph::new(complexity_text)
                .block(Block::default().title("Complexity").borders(Borders::ALL));
            f.render_widget(widget, chunks[4]);
        }
    }

    fn render_single_artifact(&self, f: &mut Frame, area: Rect, artifact: &EnhancedArtifact, title: &str) {
        let content_lines: Vec<&str> = artifact.artifact.content.lines().take(20).collect();
        let content_text = Text::from(content_lines.join("\n"));

        let widget = Paragraph::new(content_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("{}: {}", title, artifact.artifact.name))
                    .border_type(BorderType::Rounded),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(widget, area);
    }

    fn render_artifact_preview(&self, f: &mut Frame, area: Rect, artifact: &EnhancedArtifact) {
        let preview_lines: Vec<&str> = artifact.artifact.content.lines().take(10).collect();
        let preview_text = if preview_lines.len() > 10 {
            format!("{}...", preview_lines.join("\n"))
        } else {
            preview_lines.join("\n")
        };

        let widget = Paragraph::new(preview_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                .title(artifact.artifact.name.as_str())
                    .border_type(BorderType::Rounded),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(widget, area);
    }

    fn render_diff_header(&self, f: &mut Frame, area: Rect, comparison: &ComparisonState) {
        let stats = &comparison.diff.stats;
        let header_text = Text::from(vec![
            Line::from(vec![
                Span::styled("Comparing: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&comparison.left_artifact.artifact.name),
                Span::raw(" ↔ "),
                Span::raw(&comparison.right_artifact.artifact.name),
            ]),
            Line::from(vec![
                Span::styled(format!("+{}", stats.additions), Style::default().fg(Color::Green)),
                Span::raw(" "),
                Span::styled(format!("-{}", stats.deletions), Style::default().fg(Color::Red)),
                Span::raw(" "),
                Span::styled(format!("~{}", stats.modifications), Style::default().fg(Color::Yellow)),
            ]),
        ]);

        let widget = Paragraph::new(header_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Diff Overview")
                    .border_type(BorderType::Rounded),
            );

        f.render_widget(widget, area);
    }

    fn render_diff_content(&self, f: &mut Frame, area: Rect, comparison: &ComparisonState) {
        let mut diff_lines = Vec::new();

        for hunk in &comparison.diff.hunks {
            for line in &hunk.lines {
                let (style, prefix) = match line.line_type {
                    DiffLineType::Context => (Style::default(), " "),
                    DiffLineType::Addition => (Style::default().fg(Color::Green), "+"),
                    DiffLineType::Deletion => (Style::default().fg(Color::Red), "-"),
                    DiffLineType::Modified => (Style::default().fg(Color::Yellow), "~"),
                };

                diff_lines.push(Line::from(vec![
                    Span::styled(prefix, style.add_modifier(Modifier::BOLD)),
                    Span::styled(&line.content, style),
                ]));
            }
        }

        let diff_text = Text::from(diff_lines);
        let widget = Paragraph::new(diff_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Diff")
                    .border_type(BorderType::Rounded),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(widget, area);
    }

    fn apply_syntax_highlighting(&self, lines: &[&str], language: &Option<String>) -> Text {
        // This is a simplified syntax highlighting implementation
        // In practice, you'd use a proper syntax highlighting library like syntect
        if let Some(lang) = language {
            match lang.as_str() {
                "rust" => self.highlight_rust_code(lines),
                "python" => self.highlight_python_code(lines),
                "javascript" | "typescript" => self.highlight_js_code(lines),
                _ => Text::from(lines.join("\n")),
            }
        } else {
            Text::from(lines.join("\n"))
        }
    }

    fn highlight_rust_code(&self, lines: &[&str]) -> Text<'static> {
        let mut text_lines = Vec::new();

        for line in lines {
            let mut spans = Vec::new();
            
            // Simple keyword highlighting
            let words: Vec<&str> = line.split_whitespace().collect();
            for (i, word) in words.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::raw(" "));
                }

                let span = match *word {
                    word if word.starts_with("fn") => Span::styled(word.to_string(), Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                    word if word.starts_with("let") => Span::styled(word.to_string(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    word if word.starts_with("use") => Span::styled(word.to_string(), Style::default().fg(Color::Magenta)),
                    word if word.starts_with("pub") => Span::styled(word.to_string(), Style::default().fg(Color::Yellow)),
                    word if word.starts_with("//") => Span::styled(word.to_string(), Style::default().fg(Color::Gray)),
                    _ => Span::raw(word.to_string()),
                };
                spans.push(span);
            }

            text_lines.push(Line::from(spans));
        }

        Text::from(text_lines)
    }

    fn highlight_python_code(&self, lines: &[&str]) -> Text<'static> {
        let mut text_lines = Vec::new();

        for line in lines {
            let mut spans = Vec::new();
            
            let words: Vec<&str> = line.split_whitespace().collect();
            for (i, word) in words.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::raw(" "));
                }

                let span = match *word {
                    word if word.starts_with("def") => Span::styled(word.to_string(), Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                    word if word.starts_with("class") => Span::styled(word.to_string(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    word if word.starts_with("import") => Span::styled(word.to_string(), Style::default().fg(Color::Magenta)),
                    word if word.starts_with("#") => Span::styled(word.to_string(), Style::default().fg(Color::Gray)),
                    _ => Span::raw(word.to_string()),
                };
                spans.push(span);
            }

            text_lines.push(Line::from(spans));
        }

        Text::from(text_lines)
    }

    fn highlight_js_code(&self, lines: &[&str]) -> Text<'static> {
        let mut text_lines = Vec::new();

        for line in lines {
            let mut spans = Vec::new();
            
            let words: Vec<&str> = line.split_whitespace().collect();
            for (i, word) in words.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::raw(" "));
                }

                let span = match *word {
                    word if word.starts_with("function") => Span::styled(word.to_string(), Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                    word if word.starts_with("const") || word.starts_with("let") || word.starts_with("var") => {
                        Span::styled(word.to_string(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                    }
                    word if word.starts_with("//") => Span::styled(word.to_string(), Style::default().fg(Color::Gray)),
                    _ => Span::raw(word.to_string()),
                };
                spans.push(span);
            }

            text_lines.push(Line::from(spans));
        }

        Text::from(text_lines)
    }
}

/// Helper function to format timestamps
fn format_timestamp(timestamp: SystemTime) -> String {
    match timestamp.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let secs = duration.as_secs();
            let datetime = chrono::DateTime::from_timestamp(secs as i64, 0);
            datetime
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Invalid timestamp".to_string())
        }
        Err(_) => "Invalid timestamp".to_string(),
    }
}