//! Input handling system for the UI.

use crate::ui::{
    keybindings::{self, Action, KeyBindings, KeyContext},
    themes::Theme,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use std::path::PathBuf;

/// Input handler for managing text input and cursor state
#[derive(Debug, Clone)]
pub struct InputHandler {
    input_buffer: String,
    cursor_position: usize,
    history: Vec<String>,
    history_index: Option<usize>,
    keybindings: KeyBindings,
    current_context: KeyContext,
    completion_candidates: Vec<String>,
    completion_index: Option<usize>,
    tab_completion_active: bool,
}

/// Input mode for the handler
#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Insert,
    Command,
}

/// Result of input processing
#[derive(Debug, Clone)]
pub enum InputResult {
    None,
    Consumed,
    Action(Action),
    Input(String),
    Command(String),
}

impl InputHandler {
    /// Create a new input handler
    pub fn new(keybindings: KeyBindings) -> Self {
        Self {
            input_buffer: String::new(),
            cursor_position: 0,
            history: Vec::new(),
            history_index: None,
            keybindings,
            current_context: KeyContext::Normal,
            completion_candidates: Vec::new(),
            completion_index: None,
            tab_completion_active: false,
        }
    }

    /// Get current input context
    pub fn current_context(&self) -> &KeyContext {
        &self.current_context
    }

    /// Handle a key event and return the result
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<InputResult, String> {
        // First check for keybinding actions
        let combination = keybindings::KeyCombination::from_key_event(&key_event);
        if let Some(action) = self
            .keybindings
            .get_action(&combination, &self.current_context)
        {
            return Ok(InputResult::Action(action.clone()));
        }

        // Handle input-specific keys based on context
        match self.current_context {
            KeyContext::Input | KeyContext::Command => self.handle_text_input(key_event),
            _ => Ok(InputResult::None),
        }
    }

    /// Handle text input keys
    fn handle_text_input(&mut self, key_event: KeyEvent) -> Result<InputResult, String> {
        eprintln!("DEBUG: Handling text input: {:?}", key_event);
        // Reset completion on most key presses
        match key_event.code {
            KeyCode::Tab => {}, // Don't reset on tab
            _ => self.reset_completion(),
        }
        
        match key_event.code {
            KeyCode::Char(c) => {
                self.insert_char(c);
                Ok(InputResult::Consumed)
            }
            KeyCode::Backspace => {
                self.delete_char_before_cursor();
                Ok(InputResult::Consumed)
            }
            KeyCode::Delete => {
                self.delete_char_at_cursor();
                Ok(InputResult::Consumed)
            }
            KeyCode::Left => {
                self.move_cursor_left();
                Ok(InputResult::Consumed)
            }
            KeyCode::Right => {
                self.move_cursor_right();
                Ok(InputResult::Consumed)
            }
            KeyCode::Home => {
                self.move_cursor_to_start();
                Ok(InputResult::Consumed)
            }
            KeyCode::End => {
                self.move_cursor_to_end();
                Ok(InputResult::Consumed)
            }
            KeyCode::Up => {
                self.previous_history();
                Ok(InputResult::Consumed)
            }
            KeyCode::Down => {
                self.next_history();
                Ok(InputResult::Consumed)
            }
            KeyCode::Enter => {
                let input = self.get_current_input();
                if !input.is_empty() {
                    self.add_to_history(input.clone());
                }
                self.clear_input();

                match self.current_context {
                    KeyContext::Input => Ok(InputResult::Input(input)),
                    KeyContext::Command => Ok(InputResult::Command(input)),
                    _ => Ok(InputResult::None),
                }
            }
            KeyCode::Esc => {
                self.clear_input();
                Ok(InputResult::Action(Action::SwitchToNormalMode))
            }
            KeyCode::Tab => {
                self.handle_tab_completion()
            }
            _ => Ok(InputResult::None),
        }
    }

    /// Insert a character at the cursor position
    pub fn insert_char(&mut self, c: char) {
        if self.cursor_position >= self.input_buffer.len() {
            self.input_buffer.push(c);
        } else {
            self.input_buffer.insert(self.cursor_position, c);
        }
        self.cursor_position += 1;
    }

    /// Insert a string at the cursor position
    pub fn insert_str(&mut self, s: &str) {
        for c in s.chars() {
            self.insert_char(c);
        }
    }

    /// Delete character before cursor (backspace)
    pub fn delete_char_before_cursor(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input_buffer.remove(self.cursor_position);
        }
    }

    /// Delete character at cursor position
    pub fn delete_char_at_cursor(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.input_buffer.remove(self.cursor_position);
        }
    }

    /// Delete word before cursor
    pub fn delete_word_before_cursor(&mut self) {
        let original_pos = self.cursor_position;
        self.move_cursor_to_previous_word();
        let start_pos = self.cursor_position;

        if start_pos < original_pos {
            self.input_buffer.drain(start_pos..original_pos);
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.cursor_position += 1;
        }
    }

    /// Move cursor to start of line
    pub fn move_cursor_to_start(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to end of line
    pub fn move_cursor_to_end(&mut self) {
        self.cursor_position = self.input_buffer.len();
    }

    /// Move cursor to previous word
    pub fn move_cursor_to_previous_word(&mut self) {
        if self.cursor_position == 0 {
            return;
        }

        // Skip current whitespace
        while self.cursor_position > 0 {
            let pos = self.cursor_position - 1;
            if let Some(c) = self.input_buffer.chars().nth(pos) {
                if !c.is_whitespace() {
                    break;
                }
            }
            self.cursor_position -= 1;
        }

        // Skip current word
        while self.cursor_position > 0 {
            let pos = self.cursor_position - 1;
            if let Some(c) = self.input_buffer.chars().nth(pos) {
                if c.is_whitespace() {
                    break;
                }
            }
            self.cursor_position -= 1;
        }
    }

    /// Move cursor to next word
    pub fn move_cursor_to_next_word(&mut self) {
        let len = self.input_buffer.len();
        if self.cursor_position >= len {
            return;
        }

        // Skip current word
        while self.cursor_position < len {
            if let Some(c) = self.input_buffer.chars().nth(self.cursor_position) {
                if c.is_whitespace() {
                    break;
                }
            }
            self.cursor_position += 1;
        }

        // Skip whitespace
        while self.cursor_position < len {
            if let Some(c) = self.input_buffer.chars().nth(self.cursor_position) {
                if !c.is_whitespace() {
                    break;
                }
            }
            self.cursor_position += 1;
        }
    }

    /// Get current input string
    pub fn get_current_input(&self) -> String {
        self.input_buffer.clone()
    }

    /// Clear the input buffer
    pub fn clear_input(&mut self) {
        self.input_buffer.clear();
        self.cursor_position = 0;
        self.history_index = None;
    }

    /// Set the input buffer content
    pub fn set_input(&mut self, input: String) {
        self.cursor_position = input.len();
        self.input_buffer = input;
    }

    /// Add input to history
    pub fn add_to_history(&mut self, input: String) {
        if !input.trim().is_empty() {
            // Remove duplicate if it exists
            self.history.retain(|h| h != &input);

            // Add to end of history
            self.history.push(input);

            // Limit history size
            let max_history = 1000;
            if self.history.len() > max_history {
                self.history.drain(0..self.history.len() - max_history);
            }
        }
        self.history_index = None;
    }

    /// Navigate to previous history entry
    pub fn previous_history(&mut self) {
        if self.history.is_empty() {
            return;
        }

        match self.history_index {
            None => {
                self.history_index = Some(self.history.len() - 1);
                self.set_input(self.history[self.history.len() - 1].clone());
            }
            Some(index) if index > 0 => {
                self.history_index = Some(index - 1);
                self.set_input(self.history[index - 1].clone());
            }
            _ => {} // Already at the oldest entry
        }
    }

    /// Navigate to next history entry
    pub fn next_history(&mut self) {
        match self.history_index {
            Some(index) if index < self.history.len() - 1 => {
                self.history_index = Some(index + 1);
                self.set_input(self.history[index + 1].clone());
            }
            Some(_) => {
                // At newest entry, clear input
                self.history_index = None;
                self.clear_input();
            }
            None => {} // No history navigation active
        }
    }

    /// Set the current input context
    pub fn set_context(&mut self, context: KeyContext) {
        eprintln!("DEBUG: Setting input context to {:?}", context);
        self.current_context = context;
        // Reset completion when switching contexts
        self.reset_completion();
    }

    /// Handle tab completion
    fn handle_tab_completion(&mut self) -> Result<InputResult, String> {
        if !self.tab_completion_active {
            // Start new completion
            self.generate_completion_candidates();
            if !self.completion_candidates.is_empty() {
                self.tab_completion_active = true;
                self.completion_index = Some(0);
                self.apply_completion();
            }
        } else {
            // Cycle through candidates
            self.cycle_completion();
        }
        Ok(InputResult::Consumed)
    }

    /// Generate completion candidates based on current input
    fn generate_completion_candidates(&mut self) {
        self.completion_candidates.clear();
        
        let input = self.get_current_input();
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        if input.starts_with('/') {
            // Command completion
            self.generate_command_completions(&input[1..]);
        } else if parts.len() >= 2 && (parts[0] == "/cd" || parts[0] == "/ls" || parts[0] == "/load") {
            // File/directory completion
            let partial_path = parts.last().map_or("", |&p| p);
            self.generate_path_completions(partial_path);
        } else if parts.len() >= 2 && parts[0] == "/theme" {
            // Theme completion
            self.generate_theme_completions(parts.last().map_or("", |&p| p));
        } else if parts.len() >= 2 && parts[0] == "/config" {
            // Config key completion
            if parts.len() == 2 {
                self.generate_config_key_completions(parts[1]);
            }
        }
    }

    /// Generate command completions
    fn generate_command_completions(&mut self, partial: &str) {
        let commands = [
            "help", "status", "agents", "clear", "save", "load", "ls", "list", 
            "cd", "pwd", "history", "artifacts", "tasks", "config", "theme", 
            "quit", "exit"
        ];
        
        for cmd in &commands {
            if cmd.starts_with(partial) {
                self.completion_candidates.push(format!("/{}", cmd));
            }
        }
    }

    /// Generate file/directory path completions
    fn generate_path_completions(&mut self, partial_path: &str) {
        let (dir_path, file_prefix) = if partial_path.contains('/') {
            let path = std::path::Path::new(partial_path);
            if let Some(parent) = path.parent() {
                (parent.to_path_buf(), path.file_name().and_then(|n| n.to_str()).unwrap_or(""))
            } else {
                (PathBuf::from("."), partial_path)
            }
        } else {
            (PathBuf::from("."), partial_path)
        };

        if let Ok(entries) = std::fs::read_dir(&dir_path) {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                if let Some(name) = file_name.to_str() {
                    if name.starts_with(file_prefix) && !name.starts_with('.') {
                        let full_path = if dir_path == PathBuf::from(".") {
                            name.to_string()
                        } else {
                            format!("{}/{}", dir_path.display(), name)
                        };
                        
                        // Add trailing slash for directories
                        if entry.file_type().map_or(false, |ft| ft.is_dir()) {
                            self.completion_candidates.push(format!("{}/", full_path));
                        } else {
                            self.completion_candidates.push(full_path);
                        }
                    }
                }
            }
        }
        
        // Sort completions
        self.completion_candidates.sort();
    }

    /// Generate theme completions
    fn generate_theme_completions(&mut self, partial: &str) {
        let themes = ["dark", "light", "blue", "green"];
        for theme in &themes {
            if theme.starts_with(partial) {
                self.completion_candidates.push(theme.to_string());
            }
        }
    }

    /// Generate config key completions
    fn generate_config_key_completions(&mut self, partial: &str) {
        let config_keys = [
            "auto-save", "default-language", "show-confidence", 
            "verbose", "max-history"
        ];
        for key in &config_keys {
            if key.starts_with(partial) {
                self.completion_candidates.push(key.to_string());
            }
        }
    }

    /// Cycle through completion candidates
    fn cycle_completion(&mut self) {
        if let Some(index) = self.completion_index {
            let new_index = (index + 1) % self.completion_candidates.len();
            self.completion_index = Some(new_index);
            self.apply_completion();
        }
    }

    /// Apply the current completion candidate
    fn apply_completion(&mut self) {
        if let (Some(index), Some(candidate)) = (self.completion_index, self.completion_candidates.get(self.completion_index.unwrap_or(0))) {
            let input = self.get_current_input();
            let parts: Vec<&str> = input.split_whitespace().collect();
            
            if input.starts_with('/') && parts.len() == 1 {
                // Replace command
                self.input_buffer = candidate.clone();
                self.cursor_position = self.input_buffer.len();
            } else if parts.len() >= 2 {
                // Replace last part (file path, theme, config key, etc.)
                let prefix = parts[..parts.len()-1].join(" ");
                self.input_buffer = format!("{} {}", prefix, candidate);
                self.cursor_position = self.input_buffer.len();
            }
        }
    }

    /// Reset completion state
    fn reset_completion(&mut self) {
        self.tab_completion_active = false;
        self.completion_candidates.clear();
        self.completion_index = None;
    }

    /// Get current completion candidates (for UI display)
    pub fn get_completion_candidates(&self) -> &[String] {
        &self.completion_candidates
    }

    /// Check if tab completion is active
    pub fn is_completion_active(&self) -> bool {
        self.tab_completion_active
    }

    /// Get current cursor position
    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    /// Check if input buffer is empty
    pub fn is_empty(&self) -> bool {
        self.input_buffer.is_empty()
    }

    /// Get input buffer length
    pub fn len(&self) -> usize {
        self.input_buffer.len()
    }

    /// Render the input area
    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme, current_mode: &KeyContext) {
        let input_style = theme.input_style();
        let cursor_style = theme.input_cursor_style();

        // Create the input text with cursor highlighting
        let mut spans = Vec::new();

        // Add prompt based on mode
        let prompt = match current_mode {
            KeyContext::Input => "→ ",
            KeyContext::Command => ": ",
            _ => "",
        };

        if !prompt.is_empty() {
            spans.push(Span::styled(prompt, theme.user_input_style()));
        }

        // Add input text with cursor
        if self.input_buffer.is_empty() {
            // Show cursor at empty position
            spans.push(Span::styled(" ", cursor_style));
        } else {
            let chars: Vec<char> = self.input_buffer.chars().collect();

            for (i, &c) in chars.iter().enumerate() {
                let style = if i == self.cursor_position {
                    cursor_style
                } else {
                    input_style
                };

                spans.push(Span::styled(c.to_string(), style));
            }

            // Show cursor at end if needed
            if self.cursor_position >= chars.len() {
                spans.push(Span::styled(" ", cursor_style));
            }
        }

        let input_line = Line::from(spans);
        let input_paragraph = Paragraph::new(vec![input_line])
            .style(input_style)
            .wrap(Wrap { trim: false });

        // Draw the input area with a border
        let block = Block::default()
            .borders(Borders::ALL)
            .title(match current_mode {
                KeyContext::Input => "Input",
                KeyContext::Command => "Command",
                _ => "Input",
            })
            .style(theme.border_style());

        let inner_area = block.inner(area);
        f.render_widget(block, area);
        f.render_widget(input_paragraph, inner_area);
    }

    /// Get autocomplete suggestions (placeholder implementation)
    pub fn get_completions(&self) -> Vec<String> {
        // TODO: Implement intelligent autocompletion based on context
        vec![]
    }

    /// Get input history
    pub fn get_history(&self) -> &[String] {
        &self.history
    }

    /// Clear input history
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.history_index = None;
    }

    /// Called on each tick for any time-based updates
    pub fn tick(&mut self) {
        // Currently no time-based updates needed
        // Could be used for cursor blinking, auto-completion, etc.
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new(KeyBindings::default())
    }
}

/// Input validation result
#[derive(Debug, Clone)]
pub enum ValidationResult {
    Valid,
    Invalid(String),
    Warning(String),
}

/// Input validator trait
pub trait InputValidator {
    fn validate(&self, input: &str) -> ValidationResult;
}

/// Command input validator
pub struct CommandValidator;

impl InputValidator for CommandValidator {
    fn validate(&self, input: &str) -> ValidationResult {
        if input.trim().is_empty() {
            return ValidationResult::Invalid("Command cannot be empty".to_string());
        }

        // Basic command validation - could be expanded
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return ValidationResult::Invalid("Invalid command format".to_string());
        }

        ValidationResult::Valid
    }
}

/// Natural language input validator
pub struct NaturalLanguageValidator;

impl InputValidator for NaturalLanguageValidator {
    fn validate(&self, input: &str) -> ValidationResult {
        if input.trim().is_empty() {
            return ValidationResult::Invalid("Input cannot be empty".to_string());
        }

        if input.len() > 1000 {
            return ValidationResult::Warning("Input is quite long".to_string());
        }

        ValidationResult::Valid
    }
}
