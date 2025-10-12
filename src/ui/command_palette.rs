//! Command Palette System
//!
//! This module provides a comprehensive command palette with fuzzy matching,
//! quick actions, recent commands, and intelligent suggestions similar to VS Code.

use crate::interactive::history::{CompletionSuggestion, SuggestionType};
use crate::ui::shortcuts::{ShortcutAction, ShortcutManager};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame,
};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, trace};

/// Maximum number of recent commands to remember
const MAX_RECENT_COMMANDS: usize = 50;
/// Maximum number of search results to show
const MAX_SEARCH_RESULTS: usize = 100;

/// A command that can be executed from the palette
#[derive(Debug, Clone)]
pub struct PaletteCommand {
    /// Unique command identifier
    pub id: String,
    /// Display name of the command
    pub name: String,
    /// Detailed description
    pub description: String,
    /// Category for organization
    pub category: String,
    /// Keywords for searching
    pub keywords: Vec<String>,
    /// Shortcut action (if any)
    pub action: Option<ShortcutAction>,
    /// Whether this command is available in current context
    pub available: bool,
    /// Icon or symbol to display
    pub icon: Option<String>,
    /// Usage count for ranking
    pub usage_count: u32,
    /// Last used timestamp
    pub last_used: Option<SystemTime>,
    /// Custom data for the command
    pub data: HashMap<String, String>,
}

/// Result of a command search with relevance scoring
#[derive(Debug, Clone)]
pub struct CommandSearchResult {
    /// The matching command
    pub command: PaletteCommand,
    /// Relevance score (0.0 to 1.0)
    pub score: f64,
    /// Matched parts of the command
    pub matches: Vec<CommandMatch>,
}

/// A specific match within a command
#[derive(Debug, Clone)]
pub struct CommandMatch {
    /// Field that was matched (name, description, keyword)
    pub field: CommandMatchField,
    /// Character indices of the match
    pub indices: Vec<usize>,
    /// Matched text
    pub text: String,
}

/// Fields that can be matched in a command search
#[derive(Debug, Clone, PartialEq)]
pub enum CommandMatchField {
    Name,
    Description,
    Keyword,
    Category,
}

/// Quick action that can be performed
#[derive(Debug, Clone)]
pub struct QuickAction {
    /// Action identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Icon or symbol
    pub icon: Option<String>,
    /// Associated shortcut action
    pub action: ShortcutAction,
    /// Hotkey for quick access
    pub hotkey: Option<KeyCode>,
}

/// Command palette state and behavior
pub struct CommandPalette {
    /// All available commands
    commands: Vec<PaletteCommand>,
    /// Recent commands (most recent first)
    recent_commands: VecDeque<String>,
    /// Quick actions
    quick_actions: Vec<QuickAction>,
    /// Current search query
    query: String,
    /// Search results
    search_results: Vec<CommandSearchResult>,
    /// Selected result index
    selected_index: usize,
    /// List state for UI
    list_state: ListState,
    /// Scroll state for results
    scroll_state: ScrollbarState,
    /// Whether palette is visible
    visible: bool,
    /// Input cursor position
    cursor_position: usize,
    /// Mode of the palette
    mode: PaletteMode,
    /// Command categories and their visibility
    category_filter: HashMap<String, bool>,
    /// Fuzzy search engine
    fuzzy_engine: FuzzyMatcher,
    /// Statistics
    stats: PaletteStats,
}

/// Different modes of operation for the command palette
#[derive(Debug, Clone, PartialEq)]
pub enum PaletteMode {
    /// Normal command search
    Command,
    /// Quick actions mode
    QuickAction,
    /// Recent commands mode
    Recent,
    /// Category filter mode
    Category,
    /// Help mode
    Help,
}

/// Statistics for the command palette
#[derive(Debug, Default)]
pub struct PaletteStats {
    pub total_commands: usize,
    pub search_queries: u64,
    pub commands_executed: u64,
    pub average_search_time_ms: f64,
    pub most_used_commands: Vec<(String, u32)>,
}

/// Fuzzy matching engine for command search
#[derive(Debug)]
pub struct FuzzyMatcher {
    /// Case sensitivity
    case_sensitive: bool,
    /// Minimum score threshold
    min_score: f64,
    /// Bonus multipliers for different match types
    exact_match_bonus: f64,
    prefix_match_bonus: f64,
    camel_case_bonus: f64,
    word_start_bonus: f64,
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            min_score: 0.1,
            exact_match_bonus: 2.0,
            prefix_match_bonus: 1.5,
            camel_case_bonus: 1.3,
            word_start_bonus: 1.2,
        }
    }
}

impl CommandPalette {
    /// Create a new command palette
    pub fn new() -> Self {
        let mut palette = Self {
            commands: Vec::new(),
            recent_commands: VecDeque::new(),
            quick_actions: Vec::new(),
            query: String::new(),
            search_results: Vec::new(),
            selected_index: 0,
            list_state: ListState::default(),
            scroll_state: ScrollbarState::default(),
            visible: false,
            cursor_position: 0,
            mode: PaletteMode::Command,
            category_filter: HashMap::new(),
            fuzzy_engine: FuzzyMatcher::default(),
            stats: PaletteStats::default(),
        };

        palette.register_default_commands();
        palette.register_quick_actions();
        palette
    }

    /// Show the command palette
    pub fn show(&mut self) {
        self.visible = true;
        self.query.clear();
        self.cursor_position = 0;
        self.mode = PaletteMode::Command;
        self.update_search_results();
        self.stats.search_queries += 1;
        trace!("Command palette opened");
    }

    /// Hide the command palette
    pub fn hide(&mut self) {
        self.visible = false;
        self.query.clear();
        self.search_results.clear();
        self.selected_index = 0;
        trace!("Command palette closed");
    }

    /// Toggle the command palette visibility
    pub fn toggle(&mut self) {
        if self.visible {
            self.hide();
        } else {
            self.show();
        }
    }

    /// Check if the palette is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Handle key input for the command palette
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Option<PaletteAction> {
        if !self.visible {
            return None;
        }

        match key_event.code {
            KeyCode::Esc => {
                self.hide();
                Some(PaletteAction::Close)
            }
            KeyCode::Enter => {
                if let Some(result) = self.search_results.get(self.selected_index) {
                    let command = result.command.clone();
                    self.execute_command(&command.id);
                    self.hide();
                    Some(PaletteAction::ExecuteCommand(command))
                } else {
                    None
                }
            }
            KeyCode::Up => {
                self.navigate_up();
                Some(PaletteAction::NavigateUp)
            }
            KeyCode::Down => {
                self.navigate_down();
                Some(PaletteAction::NavigateDown)
            }
            KeyCode::PageUp => {
                self.navigate_page_up();
                Some(PaletteAction::PageUp)
            }
            KeyCode::PageDown => {
                self.navigate_page_down();
                Some(PaletteAction::PageDown)
            }
            KeyCode::Tab => {
                self.switch_mode();
                Some(PaletteAction::SwitchMode)
            }
            KeyCode::Backspace => {
                if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    // Delete word
                    self.delete_word_backward();
                } else {
                    // Delete character
                    self.delete_char_backward();
                }
                self.update_search_results();
                Some(PaletteAction::UpdateQuery)
            }
            KeyCode::Delete => {
                if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    // Delete word forward
                    self.delete_word_forward();
                } else {
                    // Delete character forward
                    self.delete_char_forward();
                }
                self.update_search_results();
                Some(PaletteAction::UpdateQuery)
            }
            KeyCode::Left => {
                if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    self.move_cursor_word_left();
                } else {
                    self.move_cursor_left();
                }
                Some(PaletteAction::MoveCursor)
            }
            KeyCode::Right => {
                if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    self.move_cursor_word_right();
                } else {
                    self.move_cursor_right();
                }
                Some(PaletteAction::MoveCursor)
            }
            KeyCode::Home => {
                self.cursor_position = 0;
                Some(PaletteAction::MoveCursor)
            }
            KeyCode::End => {
                self.cursor_position = self.query.len();
                Some(PaletteAction::MoveCursor)
            }
            KeyCode::Char(c) => {
                if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    match c {
                        'a' => {
                            self.cursor_position = 0;
                            Some(PaletteAction::MoveCursor)
                        }
                        'e' => {
                            self.cursor_position = self.query.len();
                            Some(PaletteAction::MoveCursor)
                        }
                        'u' => {
                            self.query.clear();
                            self.cursor_position = 0;
                            self.update_search_results();
                            Some(PaletteAction::UpdateQuery)
                        }
                        'k' => {
                            self.query.truncate(self.cursor_position);
                            self.update_search_results();
                            Some(PaletteAction::UpdateQuery)
                        }
                        _ => None,
                    }
                } else {
                    self.insert_char(c);
                    self.update_search_results();
                    Some(PaletteAction::UpdateQuery)
                }
            }
            _ => None,
        }
    }

    /// Register a new command in the palette
    pub fn register_command(&mut self, command: PaletteCommand) {
        self.commands.push(command);
        self.stats.total_commands = self.commands.len();
        self.update_search_results();
    }

    /// Remove a command from the palette
    pub fn unregister_command(&mut self, command_id: &str) -> bool {
        let initial_len = self.commands.len();
        self.commands.retain(|cmd| cmd.id != command_id);
        let removed = self.commands.len() < initial_len;
        if removed {
            self.stats.total_commands = self.commands.len();
            self.update_search_results();
        }
        removed
    }

    /// Execute a command by ID
    pub fn execute_command(&mut self, command_id: &str) {
        if let Some(command) = self.commands.iter_mut().find(|cmd| cmd.id == command_id) {
            command.usage_count += 1;
            command.last_used = Some(SystemTime::now());
            
            // Add to recent commands
            if let Some(pos) = self.recent_commands.iter().position(|id| id == command_id) {
                self.recent_commands.remove(pos);
            }
            self.recent_commands.push_front(command_id.to_string());
            
            // Maintain size limit
            if self.recent_commands.len() > MAX_RECENT_COMMANDS {
                self.recent_commands.pop_back();
            }
            
            self.stats.commands_executed += 1;
            debug!("Executed command: {}", command_id);
        }
    }

    /// Get the current mode
    pub fn get_mode(&self) -> &PaletteMode {
        &self.mode
    }

    /// Set the palette mode
    pub fn set_mode(&mut self, mode: PaletteMode) {
        self.mode = mode;
        self.update_search_results();
    }

    /// Get current search query
    pub fn get_query(&self) -> &str {
        &self.query
    }

    /// Get current search results
    pub fn get_search_results(&self) -> &[CommandSearchResult] {
        &self.search_results
    }

    /// Get selected index
    pub fn get_selected_index(&self) -> usize {
        self.selected_index
    }

    /// Get cursor position in the input
    pub fn get_cursor_position(&self) -> usize {
        self.cursor_position
    }

    /// Get palette statistics
    pub fn get_stats(&self) -> &PaletteStats {
        &self.stats
    }

    /// Render the command palette
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Calculate popup size (80% of screen, centered)
        let popup_area = self.calculate_popup_area(area);

        // Clear the background
        f.render_widget(Clear, popup_area);

        // Create the main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Input area
                Constraint::Min(5),    // Results area
                Constraint::Length(1), // Status line
            ])
            .split(popup_area);

        // Render input box
        self.render_input_box(f, chunks[0]);

        // Render results
        self.render_results(f, chunks[1]);

        // Render status line
        self.render_status_line(f, chunks[2]);
    }

    // Private methods

    fn register_default_commands(&mut self) {
        let default_commands = vec![
            PaletteCommand {
                id: "file.new".to_string(),
                name: "New File".to_string(),
                description: "Create a new file".to_string(),
                category: "File".to_string(),
                keywords: vec!["create".to_string(), "new".to_string(), "file".to_string()],
                action: Some(ShortcutAction::Custom("new_file".to_string())),
                available: true,
                icon: Some("📄".to_string()),
                usage_count: 0,
                last_used: None,
                data: HashMap::new(),
            },
            PaletteCommand {
                id: "file.open".to_string(),
                name: "Open File".to_string(),
                description: "Open an existing file".to_string(),
                category: "File".to_string(),
                keywords: vec!["open".to_string(), "file".to_string(), "load".to_string()],
                action: Some(ShortcutAction::Custom("open_file".to_string())),
                available: true,
                icon: Some("📂".to_string()),
                usage_count: 0,
                last_used: None,
                data: HashMap::new(),
            },
            PaletteCommand {
                id: "search.global".to_string(),
                name: "Search Everywhere".to_string(),
                description: "Search across all conversations and artifacts".to_string(),
                category: "Search".to_string(),
                keywords: vec!["search".to_string(), "find".to_string(), "global".to_string()],
                action: Some(ShortcutAction::OpenSearch),
                available: true,
                icon: Some("🔍".to_string()),
                usage_count: 0,
                last_used: None,
                data: HashMap::new(),
            },
            PaletteCommand {
                id: "conversation.new".to_string(),
                name: "New Conversation".to_string(),
                description: "Start a new conversation session".to_string(),
                category: "Conversation".to_string(),
                keywords: vec!["new".to_string(), "conversation".to_string(), "chat".to_string()],
                action: Some(ShortcutAction::NewConversation),
                available: true,
                icon: Some("💬".to_string()),
                usage_count: 0,
                last_used: None,
                data: HashMap::new(),
            },
            PaletteCommand {
                id: "artifact.create".to_string(),
                name: "Create Artifact".to_string(),
                description: "Create a new code artifact".to_string(),
                category: "Artifact".to_string(),
                keywords: vec!["create".to_string(), "artifact".to_string(), "code".to_string()],
                action: Some(ShortcutAction::CreateArtifact),
                available: true,
                icon: Some("⚡".to_string()),
                usage_count: 0,
                last_used: None,
                data: HashMap::new(),
            },
            PaletteCommand {
                id: "settings.open".to_string(),
                name: "Open Settings".to_string(),
                description: "Open application settings".to_string(),
                category: "Settings".to_string(),
                keywords: vec!["settings".to_string(), "preferences".to_string(), "config".to_string()],
                action: Some(ShortcutAction::OpenSettings),
                available: true,
                icon: Some("⚙️".to_string()),
                usage_count: 0,
                last_used: None,
                data: HashMap::new(),
            },
            PaletteCommand {
                id: "help.shortcuts".to_string(),
                name: "Show Keyboard Shortcuts".to_string(),
                description: "Display all available keyboard shortcuts".to_string(),
                category: "Help".to_string(),
                keywords: vec!["help".to_string(), "shortcuts".to_string(), "keys".to_string()],
                action: Some(ShortcutAction::OpenHelp),
                available: true,
                icon: Some("⌨️".to_string()),
                usage_count: 0,
                last_used: None,
                data: HashMap::new(),
            },
            PaletteCommand {
                id: "view.fullscreen".to_string(),
                name: "Toggle Fullscreen".to_string(),
                description: "Enter or exit fullscreen mode".to_string(),
                category: "View".to_string(),
                keywords: vec!["fullscreen".to_string(), "toggle".to_string(), "view".to_string()],
                action: Some(ShortcutAction::ToggleFullscreen),
                available: true,
                icon: Some("🔲".to_string()),
                usage_count: 0,
                last_used: None,
                data: HashMap::new(),
            },
        ];

        for command in default_commands {
            self.register_command(command);
        }
    }

    fn register_quick_actions(&mut self) {
        self.quick_actions = vec![
            QuickAction {
                id: "qa.search".to_string(),
                name: "Search".to_string(),
                description: "Search conversations and artifacts".to_string(),
                icon: Some("🔍".to_string()),
                action: ShortcutAction::OpenSearch,
                hotkey: Some(KeyCode::Char('s')),
            },
            QuickAction {
                id: "qa.new_conversation".to_string(),
                name: "New Chat".to_string(),
                description: "Start a new conversation".to_string(),
                icon: Some("💬".to_string()),
                action: ShortcutAction::NewConversation,
                hotkey: Some(KeyCode::Char('n')),
            },
            QuickAction {
                id: "qa.create_artifact".to_string(),
                name: "New Artifact".to_string(),
                description: "Create a code artifact".to_string(),
                icon: Some("⚡".to_string()),
                action: ShortcutAction::CreateArtifact,
                hotkey: Some(KeyCode::Char('a')),
            },
            QuickAction {
                id: "qa.settings".to_string(),
                name: "Settings".to_string(),
                description: "Open settings".to_string(),
                icon: Some("⚙️".to_string()),
                action: ShortcutAction::OpenSettings,
                hotkey: Some(KeyCode::Char(',')),
            },
        ];
    }

    fn update_search_results(&mut self) {
        match self.mode {
            PaletteMode::Command => {
                self.search_results = self.search_commands(&self.query);
            }
            PaletteMode::Recent => {
                self.search_results = self.get_recent_command_results();
            }
            PaletteMode::QuickAction => {
                self.search_results = self.get_quick_action_results();
            }
            _ => {
                self.search_results.clear();
            }
        }

        // Update selection
        if self.search_results.is_empty() {
            self.selected_index = 0;
        } else {
            self.selected_index = self.selected_index.min(self.search_results.len() - 1);
        }

        // Update list state
        self.list_state.select(Some(self.selected_index));
    }

    fn search_commands(&self, query: &str) -> Vec<CommandSearchResult> {
        let mut results = Vec::new();

        for command in &self.commands {
            if !command.available {
                continue;
            }

            if let Some(score) = self.fuzzy_engine.score_command(query, command) {
                let matches = self.fuzzy_engine.get_matches(query, command);
                results.push(CommandSearchResult {
                    command: command.clone(),
                    score,
                    matches,
                });
            }
        }

        // Sort by score (descending) and usage count (descending)
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.command.usage_count.cmp(&a.command.usage_count))
        });

        results.truncate(MAX_SEARCH_RESULTS);
        results
    }

    fn get_recent_command_results(&self) -> Vec<CommandSearchResult> {
        self.recent_commands
            .iter()
            .filter_map(|id| {
                self.commands
                    .iter()
                    .find(|cmd| &cmd.id == id)
                    .map(|cmd| CommandSearchResult {
                        command: cmd.clone(),
                        score: 1.0,
                        matches: Vec::new(),
                    })
            })
            .collect()
    }

    fn get_quick_action_results(&self) -> Vec<CommandSearchResult> {
        self.quick_actions
            .iter()
            .map(|qa| CommandSearchResult {
                command: PaletteCommand {
                    id: qa.id.clone(),
                    name: qa.name.clone(),
                    description: qa.description.clone(),
                    category: "Quick Action".to_string(),
                    keywords: Vec::new(),
                    action: Some(qa.action.clone()),
                    available: true,
                    icon: qa.icon.clone(),
                    usage_count: 0,
                    last_used: None,
                    data: HashMap::new(),
                },
                score: 1.0,
                matches: Vec::new(),
            })
            .collect()
    }

    fn navigate_up(&mut self) {
        if !self.search_results.is_empty() && self.selected_index > 0 {
            self.selected_index -= 1;
            self.list_state.select(Some(self.selected_index));
        }
    }

    fn navigate_down(&mut self) {
        if !self.search_results.is_empty() && self.selected_index < self.search_results.len() - 1 {
            self.selected_index += 1;
            self.list_state.select(Some(self.selected_index));
        }
    }

    fn navigate_page_up(&mut self) {
        let page_size = 10;
        self.selected_index = self.selected_index.saturating_sub(page_size);
        self.list_state.select(Some(self.selected_index));
    }

    fn navigate_page_down(&mut self) {
        let page_size = 10;
        if !self.search_results.is_empty() {
            self.selected_index = (self.selected_index + page_size).min(self.search_results.len() - 1);
            self.list_state.select(Some(self.selected_index));
        }
    }

    fn switch_mode(&mut self) {
        self.mode = match self.mode {
            PaletteMode::Command => PaletteMode::Recent,
            PaletteMode::Recent => PaletteMode::QuickAction,
            PaletteMode::QuickAction => PaletteMode::Command,
            _ => PaletteMode::Command,
        };
        self.update_search_results();
    }

    fn insert_char(&mut self, c: char) {
        self.query.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    fn delete_char_backward(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.query.remove(self.cursor_position);
        }
    }

    fn delete_char_forward(&mut self) {
        if self.cursor_position < self.query.len() {
            self.query.remove(self.cursor_position);
        }
    }

    fn delete_word_backward(&mut self) {
        let mut pos = self.cursor_position;
        while pos > 0 && self.query.chars().nth(pos - 1).unwrap_or(' ').is_whitespace() {
            pos -= 1;
        }
        while pos > 0 && !self.query.chars().nth(pos - 1).unwrap_or(' ').is_whitespace() {
            pos -= 1;
        }
        self.query.drain(pos..self.cursor_position);
        self.cursor_position = pos;
    }

    fn delete_word_forward(&mut self) {
        let mut pos = self.cursor_position;
        while pos < self.query.len() && self.query.chars().nth(pos).unwrap_or(' ').is_whitespace() {
            pos += 1;
        }
        while pos < self.query.len() && !self.query.chars().nth(pos).unwrap_or(' ').is_whitespace() {
            pos += 1;
        }
        self.query.drain(self.cursor_position..pos);
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_position < self.query.len() {
            self.cursor_position += 1;
        }
    }

    fn move_cursor_word_left(&mut self) {
        let mut pos = self.cursor_position;
        while pos > 0 && self.query.chars().nth(pos - 1).unwrap_or(' ').is_whitespace() {
            pos -= 1;
        }
        while pos > 0 && !self.query.chars().nth(pos - 1).unwrap_or(' ').is_whitespace() {
            pos -= 1;
        }
        self.cursor_position = pos;
    }

    fn move_cursor_word_right(&mut self) {
        let mut pos = self.cursor_position;
        while pos < self.query.len() && self.query.chars().nth(pos).unwrap_or(' ').is_whitespace() {
            pos += 1;
        }
        while pos < self.query.len() && !self.query.chars().nth(pos).unwrap_or(' ').is_whitespace() {
            pos += 1;
        }
        self.cursor_position = pos;
    }

    fn calculate_popup_area(&self, area: Rect) -> Rect {
        let width = (area.width * 4 / 5).min(120);
        let height = (area.height * 3 / 4).min(40);
        let x = (area.width - width) / 2;
        let y = (area.height - height) / 3;
        
        Rect::new(x, y, width, height)
    }

    fn render_input_box(&self, f: &mut Frame, area: Rect) {
        let mode_indicator = match self.mode {
            PaletteMode::Command => "CMD",
            PaletteMode::Recent => "REC",
            PaletteMode::QuickAction => "QA",
            PaletteMode::Category => "CAT",
            PaletteMode::Help => "HELP",
        };

        let title = format!("Command Palette - {} | Tab to switch modes | Esc to close", mode_indicator);
        
        let input_text = Text::from(Line::from(vec![
            Span::raw(&self.query[..self.cursor_position]),
            Span::styled("│", Style::default().fg(Color::Yellow)), // Cursor
            Span::raw(&self.query[self.cursor_position..]),
        ]));

        let input_paragraph = Paragraph::new(input_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().fg(Color::Blue)),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(input_paragraph, area);
    }

    fn render_results(&mut self, f: &mut Frame, area: Rect) {
        if self.search_results.is_empty() {
            let empty_text = match self.mode {
                PaletteMode::Command => "Type to search for commands...",
                PaletteMode::Recent => "No recent commands",
                PaletteMode::QuickAction => "Quick actions",
                _ => "No results",
            };
            
            let paragraph = Paragraph::new(empty_text)
                .block(Block::default().borders(Borders::ALL))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray));
            
            f.render_widget(paragraph, area);
            return;
        }

        let items: Vec<ListItem> = self
            .search_results
            .iter()
            .enumerate()
            .map(|(i, result)| {
                let is_selected = i == self.selected_index;
                let style = if is_selected {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };

                let icon = result.command.icon.as_deref().unwrap_or("•");
                let category_color = match result.command.category.as_str() {
                    "File" => Color::Green,
                    "Search" => Color::Yellow,
                    "Conversation" => Color::Cyan,
                    "Artifact" => Color::Magenta,
                    "Settings" => Color::Blue,
                    "Help" => Color::Gray,
                    "View" => Color::LightBlue,
                    _ => Color::White,
                };

                let mut spans = vec![
                    Span::styled(format!("{} ", icon), style),
                    Span::styled(&result.command.name, style.add_modifier(Modifier::BOLD)),
                ];

                if !result.command.description.is_empty() {
                    spans.push(Span::raw(" - "));
                    spans.push(Span::styled(&result.command.description, style.fg(Color::Gray)));
                }

                spans.push(Span::raw("  "));
                spans.push(Span::styled(
                    format!("[{}]", result.command.category),
                    style.fg(category_color),
                ));

                if result.command.usage_count > 0 {
                    spans.push(Span::raw("  "));
                    spans.push(Span::styled(
                        format!("({})", result.command.usage_count),
                        style.fg(Color::DarkGray),
                    ));
                }

                ListItem::new(Line::from(spans)).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Results ({}/{})", self.search_results.len(), self.commands.len()))
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD));

        f.render_stateful_widget(list, area, &mut self.list_state);

        // Render scrollbar if needed
        if self.search_results.len() > (area.height - 2) as usize {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None);
            
            f.render_stateful_widget(
                scrollbar,
                area.inner(Margin { horizontal: 0, vertical: 1 }),
                &mut self.scroll_state.content_length(self.search_results.len()),
            );
        }
    }

    fn render_status_line(&self, f: &mut Frame, area: Rect) {
        let status_text = if let Some(selected) = self.search_results.get(self.selected_index) {
            format!(
                "Enter: Execute | ↑↓: Navigate | Tab: Switch mode | Score: {:.2}",
                selected.score
            )
        } else {
            "Type to search commands...".to_string()
        };

        let status = Paragraph::new(status_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);

        f.render_widget(status, area);
    }
}

impl FuzzyMatcher {
    fn score_command(&self, query: &str, command: &PaletteCommand) -> Option<f64> {
        if query.is_empty() {
            return Some(0.5); // Base score for empty query
        }

        let mut best_score = 0.0;

        // Score against command name
        if let Some(score) = self.score_string(query, &command.name) {
            best_score = best_score.max(score * 1.0); // Full weight for name matches
        }

        // Score against description
        if let Some(score) = self.score_string(query, &command.description) {
            best_score = best_score.max(score * 0.7); // 70% weight for description matches
        }

        // Score against keywords
        for keyword in &command.keywords {
            if let Some(score) = self.score_string(query, keyword) {
                best_score = best_score.max(score * 0.8); // 80% weight for keyword matches
            }
        }

        // Score against category
        if let Some(score) = self.score_string(query, &command.category) {
            best_score = best_score.max(score * 0.5); // 50% weight for category matches
        }

        if best_score >= self.min_score {
            Some(best_score)
        } else {
            None
        }
    }

    fn score_string(&self, query: &str, target: &str) -> Option<f64> {
        let query = if self.case_sensitive { query } else { &query.to_lowercase() };
        let target = if self.case_sensitive { target.to_string() } else { target.to_lowercase() };

        if target.contains(query) {
            let mut score = 0.6; // Base score for substring match

            // Exact match bonus
            if target == query {
                score *= self.exact_match_bonus;
            }
            // Prefix match bonus
            else if target.starts_with(query) {
                score *= self.prefix_match_bonus;
            }
            // Word start bonus
            else if target.split_whitespace().any(|word| word.starts_with(query)) {
                score *= self.word_start_bonus;
            }
            // Camel case bonus
            else if self.matches_camel_case(query, &target) {
                score *= self.camel_case_bonus;
            }

            Some(score.min(1.0))
        } else {
            None
        }
    }

    fn matches_camel_case(&self, query: &str, target: &str) -> bool {
        let query_chars: Vec<char> = query.chars().collect();
        let target_chars: Vec<char> = target.chars().collect();
        
        let mut query_idx = 0;
        
        for (i, &c) in target_chars.iter().enumerate() {
            if query_idx >= query_chars.len() {
                break;
            }
            
            if c.is_uppercase() || (i == 0) {
                if c.to_lowercase().to_string() == query_chars[query_idx].to_lowercase().to_string() {
                    query_idx += 1;
                }
            }
        }
        
        query_idx == query_chars.len()
    }

    fn get_matches(&self, query: &str, command: &PaletteCommand) -> Vec<CommandMatch> {
        let mut matches = Vec::new();

        // This is a simplified implementation
        // In practice, you'd want to track exact character positions
        
        if !query.is_empty() {
            let query_lower = query.to_lowercase();
            
            if command.name.to_lowercase().contains(&query_lower) {
                matches.push(CommandMatch {
                    field: CommandMatchField::Name,
                    indices: vec![0], // Simplified
                    text: command.name.clone(),
                });
            }
            
            if command.description.to_lowercase().contains(&query_lower) {
                matches.push(CommandMatch {
                    field: CommandMatchField::Description,
                    indices: vec![0], // Simplified
                    text: command.description.clone(),
                });
            }
        }

        matches
    }
}

/// Actions that can be triggered by the command palette
#[derive(Debug, Clone)]
pub enum PaletteAction {
    /// Close the palette
    Close,
    /// Execute a specific command
    ExecuteCommand(PaletteCommand),
    /// Navigate up in results
    NavigateUp,
    /// Navigate down in results
    NavigateDown,
    /// Page up in results
    PageUp,
    /// Page down in results
    PageDown,
    /// Switch between modes
    SwitchMode,
    /// Update search query
    UpdateQuery,
    /// Move cursor in input
    MoveCursor,
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_matching() {
        let fuzzy = FuzzyMatcher::default();
        
        // Test exact match
        assert!(fuzzy.score_string("test", "test").unwrap() > 0.9);
        
        // Test prefix match
        assert!(fuzzy.score_string("te", "test").unwrap() > 0.7);
        
        // Test substring match
        assert!(fuzzy.score_string("es", "test").unwrap() > 0.5);
        
        // Test no match
        assert!(fuzzy.score_string("xyz", "test").is_none());
    }

    #[test]
    fn test_command_registration() {
        let mut palette = CommandPalette::new();
        let initial_count = palette.commands.len();
        
        let test_command = PaletteCommand {
            id: "test.command".to_string(),
            name: "Test Command".to_string(),
            description: "A test command".to_string(),
            category: "Test".to_string(),
            keywords: vec!["test".to_string()],
            action: None,
            available: true,
            icon: None,
            usage_count: 0,
            last_used: None,
            data: HashMap::new(),
        };
        
        palette.register_command(test_command);
        assert_eq!(palette.commands.len(), initial_count + 1);
    }

    #[test]
    fn test_search_functionality() {
        let mut palette = CommandPalette::new();
        
        // Search for a command that should exist
        let results = palette.search_commands("new");
        assert!(!results.is_empty());
        
        // Search for something that shouldn't exist
        let results = palette.search_commands("nonexistent12345");
        assert!(results.is_empty());
    }
}