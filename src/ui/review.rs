//! Multi-file review system with interactive TUI
//!
//! This module provides an interactive interface for reviewing complex multi-file
//! changesets with granular apply/reject controls, side-by-side diffs, and
//! selective hunk application.

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        block::Title, Block, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame, Terminal,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use tokio::sync::RwLock;

use crate::codegen::diff_apply::{ChangeSet, FileDiff, ChangeType};

/// Multi-file review system
#[derive(Debug)]
pub struct ReviewSystem {
    state: RwLock<ReviewState>,
    config: ReviewConfig,
}

/// Configuration for the review system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewConfig {
    /// Number of context lines to show around changes
    pub context_lines: usize,
    /// Whether to show line numbers
    pub show_line_numbers: bool,
    /// Whether to use syntax highlighting
    pub syntax_highlighting: bool,
    /// Whether to show whitespace changes
    pub show_whitespace: bool,
    /// Whether to word wrap long lines
    pub word_wrap: bool,
    /// Color scheme for diffs
    pub color_scheme: ColorScheme,
}

/// Color scheme for diff display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub added_line: Color,
    pub removed_line: Color,
    pub context_line: Color,
    pub line_number: Color,
    pub selected: Color,
    pub header: Color,
}

/// Internal state of the review system
#[derive(Debug)]
struct ReviewState {
    changeset: Option<ChangeSet>,
    files: Vec<ReviewFile>,
    current_file_index: usize,
    current_hunk_index: usize,
    scroll_offset: usize,
    view_mode: ViewMode,
    decisions: HashMap<String, FileDecision>,
    hunk_decisions: HashMap<(String, usize), HunkDecision>,
    show_help: bool,
    filter: ReviewFilter,
}

/// Individual file in review
#[derive(Debug, Clone)]
struct ReviewFile {
    pub path: String,
    pub original_content: Option<String>,
    pub new_content: String,
    pub change_type: ChangeType,
    pub hunks: Vec<DiffHunk>,
    pub decision: FileDecision,
}

/// A hunk of changes within a file
#[derive(Debug, Clone)]
struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
    pub start_line: usize,
    pub added_lines: usize,
    pub removed_lines: usize,
    pub decision: HunkDecision,
}

/// A line within a diff hunk
#[derive(Debug, Clone)]
struct DiffLine {
    pub line_type: LineType,
    pub content: String,
    pub line_number_old: Option<usize>,
    pub line_number_new: Option<usize>,
}

/// Type of diff line
#[derive(Debug, Clone, PartialEq, Eq)]
enum LineType {
    Context,
    Added,
    Removed,
    Header,
}

/// Display mode for the review interface
#[derive(Debug, Clone, PartialEq, Eq)]
enum ViewMode {
    /// Show list of files
    FileList,
    /// Show unified diff view
    UnifiedDiff,
    /// Show side-by-side diff view
    SideBySide,
    /// Show hunk selector
    HunkSelector,
}

/// Decision for a file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileDecision {
    Pending,
    Accept,
    Reject,
    Partial, // Some hunks accepted, some rejected
}

/// Decision for a hunk
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HunkDecision {
    Pending,
    Accept,
    Reject,
}

/// Filter for review display
#[derive(Debug, Clone)]
struct ReviewFilter {
    show_only_modified: bool,
    file_pattern: Option<String>,
    show_only_pending: bool,
}

/// Result of the review process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub overall_decision: OverallDecision,
    pub file_decisions: HashMap<String, FileDecision>,
    pub hunk_decisions: HashMap<String, Vec<HunkDecision>>,
    pub applied_files: Vec<String>,
    pub rejected_files: Vec<String>,
    pub partial_files: Vec<String>,
}

/// Overall review decision
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverallDecision {
    AcceptAll,
    RejectAll,
    Partial,
    Cancelled,
}

/// Errors in the review system
#[derive(Debug, thiserror::Error)]
pub enum ReviewError {
    #[error("Terminal error: {0}")]
    TerminalError(String),
    
    #[error("Diff parsing error: {0}")]
    DiffParsingError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Invalid state: {0}")]
    InvalidState(String),
}

impl ReviewSystem {
    /// Create a new review system
    pub fn new(config: ReviewConfig) -> Self {
        Self {
            state: RwLock::new(ReviewState {
                changeset: None,
                files: Vec::new(),
                current_file_index: 0,
                current_hunk_index: 0,
                scroll_offset: 0,
                view_mode: ViewMode::FileList,
                decisions: HashMap::new(),
                hunk_decisions: HashMap::new(),
                show_help: false,
                filter: ReviewFilter {
                    show_only_modified: false,
                    file_pattern: None,
                    show_only_pending: true,
                },
            }),
            config,
        }
    }
    
    /// Start interactive review of a changeset
    pub async fn review_changeset(&self, changeset: ChangeSet) -> Result<ReviewResult, ReviewError> {
        // Parse changeset into review format
        let review_files = self.parse_changeset(&changeset)?;
        
        // Set initial state
        {
            let mut state = self.state.write().await;
            state.changeset = Some(changeset.clone());
            state.files = review_files;
            state.current_file_index = 0;
            state.current_hunk_index = 0;
            state.decisions.clear();
            state.hunk_decisions.clear();
        }
        
        // Start interactive TUI
        self.run_interactive_review().await
    }
    
    /// Parse changeset into review format
    fn parse_changeset(&self, changeset: &ChangeSet) -> Result<Vec<ReviewFile>, ReviewError> {
        let mut review_files = Vec::new();
        
        for file_diff in &changeset.files {
            let hunks = self.parse_file_diff(file_diff)?;
            
            let review_file = ReviewFile {
                path: file_diff.file_path.to_string_lossy().to_string(),
                original_content: file_diff.original_content.clone(),
                new_content: file_diff.new_content.clone(),
                change_type: file_diff.change_type.clone(),
                hunks,
                decision: FileDecision::Pending,
            };
            
            review_files.push(review_file);
        }
        
        Ok(review_files)
    }
    
    /// Parse file diff into hunks
    fn parse_file_diff(&self, file_diff: &FileDiff) -> Result<Vec<DiffHunk>, ReviewError> {
        // Simple diff parsing - in production this would use a proper diff library
        let hunks = if let Some(ref original) = file_diff.original_content {
            self.generate_unified_diff(original, &file_diff.new_content)?
        } else {
            // New file
            vec![self.create_new_file_hunk(&file_diff.new_content)]
        };
        
        Ok(hunks)
    }
    
    /// Generate unified diff hunks
    fn generate_unified_diff(&self, original: &str, new: &str) -> Result<Vec<DiffHunk>, ReviewError> {
        // Simplified diff generation - would use proper diff algorithm in production
        let orig_lines: Vec<&str> = original.lines().collect();
        let new_lines: Vec<&str> = new.lines().collect();
        
        // For now, create a single hunk with all changes
        let mut diff_lines = Vec::new();
        let mut line_num_old = 1;
        let mut line_num_new = 1;
        
        // Simple line-by-line comparison
        let max_lines = orig_lines.len().max(new_lines.len());
        
        for i in 0..max_lines {
            match (orig_lines.get(i), new_lines.get(i)) {
                (Some(old_line), Some(new_line)) => {
                    if old_line == new_line {
                        // Context line
                        diff_lines.push(DiffLine {
                            line_type: LineType::Context,
                            content: old_line.to_string(),
                            line_number_old: Some(line_num_old),
                            line_number_new: Some(line_num_new),
                        });
                        line_num_old += 1;
                        line_num_new += 1;
                    } else {
                        // Changed line
                        diff_lines.push(DiffLine {
                            line_type: LineType::Removed,
                            content: old_line.to_string(),
                            line_number_old: Some(line_num_old),
                            line_number_new: None,
                        });
                        diff_lines.push(DiffLine {
                            line_type: LineType::Added,
                            content: new_line.to_string(),
                            line_number_old: None,
                            line_number_new: Some(line_num_new),
                        });
                        line_num_old += 1;
                        line_num_new += 1;
                    }
                }
                (Some(old_line), None) => {
                    // Removed line
                    diff_lines.push(DiffLine {
                        line_type: LineType::Removed,
                        content: old_line.to_string(),
                        line_number_old: Some(line_num_old),
                        line_number_new: None,
                    });
                    line_num_old += 1;
                }
                (None, Some(new_line)) => {
                    // Added line
                    diff_lines.push(DiffLine {
                        line_type: LineType::Added,
                        content: new_line.to_string(),
                        line_number_old: None,
                        line_number_new: Some(line_num_new),
                    });
                    line_num_new += 1;
                }
                (None, None) => break,
            }
        }
        
        let added_lines = diff_lines.iter().filter(|l| l.line_type == LineType::Added).count();
        let removed_lines = diff_lines.iter().filter(|l| l.line_type == LineType::Removed).count();
        
        Ok(vec![DiffHunk {
            header: format!("@@ -1,{} +1,{} @@", orig_lines.len(), new_lines.len()),
            lines: diff_lines,
            start_line: 1,
            added_lines,
            removed_lines,
            decision: HunkDecision::Pending,
        }])
    }
    
    /// Create hunk for new file
    fn create_new_file_hunk(&self, content: &str) -> DiffHunk {
        let lines: Vec<DiffLine> = content
            .lines()
            .enumerate()
            .map(|(i, line)| DiffLine {
                line_type: LineType::Added,
                content: line.to_string(),
                line_number_old: None,
                line_number_new: Some(i + 1),
            })
            .collect();
        
        DiffHunk {
            header: format!("@@ -0,0 +1,{} @@", lines.len()),
            lines,
            start_line: 1,
            added_lines: lines.len(),
            removed_lines: 0,
            decision: HunkDecision::Pending,
        }
    }
    
    /// Run the interactive review interface
    async fn run_interactive_review(&self) -> Result<ReviewResult, ReviewError> {
        // Set up terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        let result = self.run_review_loop(&mut terminal).await;
        
        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        
        result
    }
    
    /// Main review loop
    async fn run_review_loop<B: Backend>(&self, terminal: &mut Terminal<B>) -> Result<ReviewResult, ReviewError> {
        loop {
            // Draw the interface
            terminal.draw(|f| {
                let rt = tokio::runtime::Handle::current();
                rt.block_on(async {
                    if let Err(e) = self.draw_frame(f).await {
                        eprintln!("Error drawing frame: {}", e);
                    }
                });
            })?;
            
            // Handle input
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match self.handle_key_event(key.code).await {
                        KeyResult::Continue => continue,
                        KeyResult::Exit(result) => return Ok(result),
                        KeyResult::Error(e) => return Err(e),
                    }
                }
            }
        }
    }
    
    /// Draw the main frame
    async fn draw_frame<B: Backend>(&self, f: &mut Frame<B>) -> Result<(), ReviewError> {
        let state = self.state.read().await;
        
        if state.show_help {
            self.draw_help_popup(f).await;
            return Ok(());
        }
        
        match state.view_mode {
            ViewMode::FileList => self.draw_file_list(f, &state).await,
            ViewMode::UnifiedDiff => self.draw_unified_diff(f, &state).await,
            ViewMode::SideBySide => self.draw_side_by_side(f, &state).await,
            ViewMode::HunkSelector => self.draw_hunk_selector(f, &state).await,
        }
    }
    
    /// Draw file list view
    async fn draw_file_list<B: Backend>(&self, f: &mut Frame<B>, state: &ReviewState) -> Result<(), ReviewError> {
        let area = f.size();
        
        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // File list
                Constraint::Length(3), // Footer
            ])
            .split(area);
        
        // Draw header
        let header = Paragraph::new("Multi-File Review - File List (Press 'h' for help)")
            .block(Block::default().borders(Borders::ALL).title("DevKit Review"))
            .style(Style::default().fg(self.config.color_scheme.header));
        f.render_widget(header, chunks[0]);
        
        // Draw file list
        let files: Vec<ListItem> = state.files
            .iter()
            .enumerate()
            .map(|(i, file)| {
                let decision_indicator = match file.decision {
                    FileDecision::Accept => "✓",
                    FileDecision::Reject => "✗",
                    FileDecision::Partial => "◐",
                    FileDecision::Pending => "○",
                };
                
                let change_indicator = match file.change_type {
                    ChangeType::Create => "+",
                    ChangeType::Modify => "~",
                    ChangeType::Delete => "-",
                    ChangeType::Rename { .. } => "→",
                };
                
                let style = if i == state.current_file_index {
                    Style::default().fg(self.config.color_scheme.selected).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                
                ListItem::new(format!(
                    "{} {} {} ({})", 
                    decision_indicator,
                    change_indicator,
                    file.path,
                    file.hunks.len()
                )).style(style)
            })
            .collect();
        
        let files_list = List::new(files)
            .block(Block::default().borders(Borders::ALL).title("Files"))
            .highlight_style(Style::default().fg(self.config.color_scheme.selected));
        
        f.render_widget(files_list, chunks[1]);
        
        // Draw footer with key bindings
        let footer = Paragraph::new("↑/↓: Navigate | Enter: View diff | Space: Toggle accept | a: Accept all | r: Reject all | q: Finish review")
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(footer, chunks[2]);
        
        Ok(())
    }
    
    /// Draw unified diff view (stub)
    async fn draw_unified_diff<B: Backend>(&self, f: &mut Frame<B>, state: &ReviewState) -> Result<(), ReviewError> {
        let area = f.size();
        let placeholder = Paragraph::new("Unified diff view - Implementation in progress")
            .block(Block::default().borders(Borders::ALL).title("Unified Diff"));
        f.render_widget(placeholder, area);
        Ok(())
    }
    
    /// Draw side-by-side diff view (stub)
    async fn draw_side_by_side<B: Backend>(&self, f: &mut Frame<B>, state: &ReviewState) -> Result<(), ReviewError> {
        let area = f.size();
        let placeholder = Paragraph::new("Side-by-side diff view - Implementation in progress")
            .block(Block::default().borders(Borders::ALL).title("Side-by-Side Diff"));
        f.render_widget(placeholder, area);
        Ok(())
    }
    
    /// Draw hunk selector view (stub)
    async fn draw_hunk_selector<B: Backend>(&self, f: &mut Frame<B>, state: &ReviewState) -> Result<(), ReviewError> {
        let area = f.size();
        let placeholder = Paragraph::new("Hunk selector view - Implementation in progress")
            .block(Block::default().borders(Borders::ALL).title("Hunk Selector"));
        f.render_widget(placeholder, area);
        Ok(())
    }
    
    /// Draw help popup
    async fn draw_help_popup<B: Backend>(&self, f: &mut Frame<B>) {
        let area = f.size();
        let popup_area = self.centered_rect(80, 60, area);
        
        f.render_widget(Clear, popup_area);
        
        let help_text = vec![
            Line::from("DevKit Multi-File Review - Help"),
            Line::from(""),
            Line::from("Navigation:"),
            Line::from("  ↑/↓ or j/k  - Navigate files/hunks"),
            Line::from("  Enter       - View detailed diff"),
            Line::from("  Tab         - Switch view mode"),
            Line::from(""),
            Line::from("Review Actions:"),
            Line::from("  Space       - Toggle accept/reject file"),
            Line::from("  a           - Accept current file"),
            Line::from("  r           - Reject current file"),
            Line::from("  A           - Accept all files"),
            Line::from("  R           - Reject all files"),
            Line::from(""),
            Line::from("Other:"),
            Line::from("  h           - Toggle this help"),
            Line::from("  q           - Finish review"),
            Line::from("  Esc         - Cancel review"),
            Line::from(""),
            Line::from("Press any key to close help"),
        ];
        
        let help_paragraph = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title("Help"))
            .wrap(Wrap { trim: true });
        
        f.render_widget(help_paragraph, popup_area);
    }
    
    /// Handle key events
    async fn handle_key_event(&self, key: KeyCode) -> KeyResult {
        let mut state = self.state.write().await;
        
        if state.show_help {
            state.show_help = false;
            return KeyResult::Continue;
        }
        
        match key {
            KeyCode::Char('q') => {
                return KeyResult::Exit(self.generate_review_result(&state));
            }
            KeyCode::Esc => {
                return KeyResult::Exit(ReviewResult {
                    overall_decision: OverallDecision::Cancelled,
                    file_decisions: HashMap::new(),
                    hunk_decisions: HashMap::new(),
                    applied_files: Vec::new(),
                    rejected_files: Vec::new(),
                    partial_files: Vec::new(),
                });
            }
            KeyCode::Char('h') => {
                state.show_help = true;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if state.current_file_index > 0 {
                    state.current_file_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if state.current_file_index + 1 < state.files.len() {
                    state.current_file_index += 1;
                }
            }
            KeyCode::Char(' ') => {
                // Toggle file decision
                if let Some(file) = state.files.get_mut(state.current_file_index) {
                    file.decision = match file.decision {
                        FileDecision::Pending => FileDecision::Accept,
                        FileDecision::Accept => FileDecision::Reject,
                        FileDecision::Reject => FileDecision::Pending,
                        FileDecision::Partial => FileDecision::Accept,
                    };
                }
            }
            KeyCode::Char('a') => {
                // Accept current file
                if let Some(file) = state.files.get_mut(state.current_file_index) {
                    file.decision = FileDecision::Accept;
                }
            }
            KeyCode::Char('r') => {
                // Reject current file
                if let Some(file) = state.files.get_mut(state.current_file_index) {
                    file.decision = FileDecision::Reject;
                }
            }
            KeyCode::Char('A') => {
                // Accept all files
                for file in &mut state.files {
                    file.decision = FileDecision::Accept;
                }
            }
            KeyCode::Char('R') => {
                // Reject all files
                for file in &mut state.files {
                    file.decision = FileDecision::Reject;
                }
            }
            KeyCode::Enter => {
                // Switch to diff view for current file
                state.view_mode = ViewMode::UnifiedDiff;
            }
            KeyCode::Tab => {
                // Cycle through view modes
                state.view_mode = match state.view_mode {
                    ViewMode::FileList => ViewMode::UnifiedDiff,
                    ViewMode::UnifiedDiff => ViewMode::SideBySide,
                    ViewMode::SideBySide => ViewMode::HunkSelector,
                    ViewMode::HunkSelector => ViewMode::FileList,
                };
            }
            _ => {}
        }
        
        KeyResult::Continue
    }
    
    /// Generate final review result
    fn generate_review_result(&self, state: &ReviewState) -> ReviewResult {
        let mut file_decisions = HashMap::new();
        let mut applied_files = Vec::new();
        let mut rejected_files = Vec::new();
        let mut partial_files = Vec::new();
        
        for file in &state.files {
            file_decisions.insert(file.path.clone(), file.decision.clone());
            
            match file.decision {
                FileDecision::Accept => applied_files.push(file.path.clone()),
                FileDecision::Reject => rejected_files.push(file.path.clone()),
                FileDecision::Partial => partial_files.push(file.path.clone()),
                FileDecision::Pending => {} // No action
            }
        }
        
        let overall_decision = if applied_files.len() == state.files.len() {
            OverallDecision::AcceptAll
        } else if rejected_files.len() == state.files.len() {
            OverallDecision::RejectAll
        } else {
            OverallDecision::Partial
        };
        
        ReviewResult {
            overall_decision,
            file_decisions,
            hunk_decisions: HashMap::new(), // Would be populated with actual hunk decisions
            applied_files,
            rejected_files,
            partial_files,
        }
    }
    
    /// Helper to create centered rectangle
    fn centered_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);
        
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}

/// Result of key event handling
enum KeyResult {
    Continue,
    Exit(ReviewResult),
    Error(ReviewError),
}

impl Default for ReviewConfig {
    fn default() -> Self {
        Self {
            context_lines: 3,
            show_line_numbers: true,
            syntax_highlighting: false, // Would require additional dependencies
            show_whitespace: false,
            word_wrap: false,
            color_scheme: ColorScheme::default(),
        }
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            added_line: Color::Green,
            removed_line: Color::Red,
            context_line: Color::White,
            line_number: Color::Yellow,
            selected: Color::Cyan,
            header: Color::Blue,
        }
    }
}