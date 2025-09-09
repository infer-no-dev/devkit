//! Input handling system for the UI.

use crossterm::event::{KeyEvent, KeyCode};
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use crate::ui::{keybindings::{self, KeyBindings, KeyContext, Action}, themes::Theme};

/// Input handler for managing text input and cursor state
#[derive(Debug, Clone)]
pub struct InputHandler {
    input_buffer: String,
    cursor_position: usize,
    history: Vec<String>,
    history_index: Option<usize>,
    keybindings: KeyBindings,
    current_context: KeyContext,
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
        if let Some(action) = self.keybindings.get_action(&combination, &self.current_context) {
            return Ok(InputResult::Action(action.clone()));
        }
        
        // Handle input-specific keys based on context
        match self.current_context {
            KeyContext::Input | KeyContext::Command => {
                self.handle_text_input(key_event)
            }
            _ => Ok(InputResult::None)
        }
    }
    
    /// Handle text input keys
    fn handle_text_input(&mut self, key_event: KeyEvent) -> Result<InputResult, String> {
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
                    _ => Ok(InputResult::None)
                }
            }
            KeyCode::Esc => {
                self.clear_input();
                Ok(InputResult::Action(Action::SwitchToNormalMode))
            }
            KeyCode::Tab => {
                // TODO: Implement auto-completion
                Ok(InputResult::Consumed)
            }
            _ => Ok(InputResult::None)
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
        self.current_context = context;
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
    pub fn render(
        &self,
        f: &mut Frame,
        area: Rect,
        theme: &Theme,
        current_mode: &KeyContext,
    ) {
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
