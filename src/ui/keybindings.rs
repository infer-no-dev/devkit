//! Configurable key bindings for the UI.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Action that can be triggered by a key binding
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Action {
    // Navigation actions
    Quit,
    SwitchToNormalMode,
    SwitchToInputMode,
    SwitchToCommandMode,
    SwitchToAgentView,
    SwitchToSettingsView,
    SwitchToHelpView,
    
    // Input actions
    ConfirmInput,
    CancelInput,
    ClearInput,
    DeleteChar,
    DeleteWord,
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorStart,
    MoveCursorEnd,
    
    // Scrolling actions
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    ScrollToTop,
    ScrollToBottom,
    
    // Agent actions
    StopAllAgents,
    RestartAgent,
    ShowAgentDetails,
    
    // Block actions
    ClearOutput,
    FilterByType,
    FilterByAgent,
    SearchOutput,
    
    // Theme actions
    NextTheme,
    PreviousTheme,
    
    // Other actions
    ShowHelp,
    ToggleHelp,
    ShowStatus,
    ToggleTimestamps,
    ToggleLineNumbers,
    RefreshView,
    SwitchTheme,
    
    // Custom actions (for extension)
    Custom(String),
}

/// Key combination for triggering actions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyCombination {
    pub key: KeyCode,
    pub modifiers: KeyModifiers,
}

/// Context in which keybindings are active
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum KeyContext {
    Global,          // Active in all modes
    Normal,          // Normal navigation mode
    Input,           // Text input mode
    Command,         // Command entry mode
    AgentView,       // Agent status view
    Settings,        // Settings view
    Help,            // Help view
}

/// Type alias for backward compatibility
pub type Context = KeyContext;

/// Keybinding manager
#[derive(Debug, Clone)]
pub struct KeybindingManager {
    bindings: KeyBindings,
}

/// Keybinding configuration
#[derive(Debug, Clone)]
pub struct KeyBindings {
    bindings: HashMap<KeyContext, HashMap<KeyCombination, Action>>,
}

impl KeyBindings {
    /// Create new keybindings with defaults
    pub fn new() -> Self {
        let mut bindings = HashMap::new();
        
        // Global keybindings (active in all contexts)
        let mut global = HashMap::new();
        global.insert(
            KeyCombination::new(KeyCode::Char('q'), KeyModifiers::empty()),
            Action::Quit
        );
        global.insert(
            KeyCombination::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            Action::Quit
        );
        global.insert(
            KeyCombination::new(KeyCode::F(1), KeyModifiers::empty()),
            Action::ShowHelp
        );
        global.insert(
            KeyCombination::new(KeyCode::F(5), KeyModifiers::empty()),
            Action::RefreshView
        );
        bindings.insert(KeyContext::Global, global);
        
        // Normal mode keybindings
        let mut normal = HashMap::new();
        normal.insert(
            KeyCombination::new(KeyCode::Char('i'), KeyModifiers::empty()),
            Action::SwitchToInputMode
        );
        normal.insert(
            KeyCombination::new(KeyCode::Char(':'), KeyModifiers::empty()),
            Action::SwitchToCommandMode
        );
        normal.insert(
            KeyCombination::new(KeyCode::Char('a'), KeyModifiers::empty()),
            Action::SwitchToAgentView
        );
        normal.insert(
            KeyCombination::new(KeyCode::Char('s'), KeyModifiers::empty()),
            Action::SwitchToSettingsView
        );
        normal.insert(
            KeyCombination::new(KeyCode::Char('?'), KeyModifiers::empty()),
            Action::ShowHelp
        );
        normal.insert(
            KeyCombination::new(KeyCode::Char('k'), KeyModifiers::empty()),
            Action::ScrollUp
        );
        normal.insert(
            KeyCombination::new(KeyCode::Char('j'), KeyModifiers::empty()),
            Action::ScrollDown
        );
        normal.insert(
            KeyCombination::new(KeyCode::Char('u'), KeyModifiers::empty()),
            Action::PageUp
        );
        normal.insert(
            KeyCombination::new(KeyCode::Char('d'), KeyModifiers::empty()),
            Action::PageDown
        );
        normal.insert(
            KeyCombination::new(KeyCode::Char('g'), KeyModifiers::empty()),
            Action::ScrollToTop
        );
        normal.insert(
            KeyCombination::new(KeyCode::Char('G'), KeyModifiers::empty()),
            Action::ScrollToBottom
        );
        normal.insert(
            KeyCombination::new(KeyCode::Char('c'), KeyModifiers::empty()),
            Action::ClearOutput
        );
        normal.insert(
            KeyCombination::new(KeyCode::Char('/'), KeyModifiers::empty()),
            Action::SearchOutput
        );
        normal.insert(
            KeyCombination::new(KeyCode::Char('t'), KeyModifiers::empty()),
            Action::NextTheme
        );
        normal.insert(
            KeyCombination::new(KeyCode::Char('T'), KeyModifiers::empty()),
            Action::PreviousTheme
        );
        bindings.insert(KeyContext::Normal, normal);
        
        // Input mode keybindings
        let mut input = HashMap::new();
        input.insert(
            KeyCombination::new(KeyCode::Esc, KeyModifiers::empty()),
            Action::SwitchToNormalMode
        );
        input.insert(
            KeyCombination::new(KeyCode::Enter, KeyModifiers::empty()),
            Action::ConfirmInput
        );
        input.insert(
            KeyCombination::new(KeyCode::Backspace, KeyModifiers::empty()),
            Action::DeleteChar
        );
        input.insert(
            KeyCombination::new(KeyCode::Char('w'), KeyModifiers::CONTROL),
            Action::DeleteWord
        );
        input.insert(
            KeyCombination::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
            Action::ClearInput
        );
        input.insert(
            KeyCombination::new(KeyCode::Left, KeyModifiers::empty()),
            Action::MoveCursorLeft
        );
        input.insert(
            KeyCombination::new(KeyCode::Right, KeyModifiers::empty()),
            Action::MoveCursorRight
        );
        input.insert(
            KeyCombination::new(KeyCode::Home, KeyModifiers::empty()),
            Action::MoveCursorStart
        );
        input.insert(
            KeyCombination::new(KeyCode::End, KeyModifiers::empty()),
            Action::MoveCursorEnd
        );
        input.insert(
            KeyCombination::new(KeyCode::Char('a'), KeyModifiers::CONTROL),
            Action::MoveCursorStart
        );
        input.insert(
            KeyCombination::new(KeyCode::Char('e'), KeyModifiers::CONTROL),
            Action::MoveCursorEnd
        );
        bindings.insert(KeyContext::Input, input);
        
        // Command mode keybindings (similar to input mode)
        let mut command = HashMap::new();
        command.insert(
            KeyCombination::new(KeyCode::Esc, KeyModifiers::empty()),
            Action::SwitchToNormalMode
        );
        command.insert(
            KeyCombination::new(KeyCode::Enter, KeyModifiers::empty()),
            Action::ConfirmInput
        );
        command.insert(
            KeyCombination::new(KeyCode::Backspace, KeyModifiers::empty()),
            Action::DeleteChar
        );
        command.insert(
            KeyCombination::new(KeyCode::Char('w'), KeyModifiers::CONTROL),
            Action::DeleteWord
        );
        command.insert(
            KeyCombination::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
            Action::ClearInput
        );
        bindings.insert(KeyContext::Command, command);
        
        // Agent view keybindings
        let mut agent_view = HashMap::new();
        agent_view.insert(
            KeyCombination::new(KeyCode::Esc, KeyModifiers::empty()),
            Action::SwitchToNormalMode
        );
        agent_view.insert(
            KeyCombination::new(KeyCode::Char('r'), KeyModifiers::empty()),
            Action::RestartAgent
        );
        agent_view.insert(
            KeyCombination::new(KeyCode::Char('x'), KeyModifiers::empty()),
            Action::StopAllAgents
        );
        agent_view.insert(
            KeyCombination::new(KeyCode::Enter, KeyModifiers::empty()),
            Action::ShowAgentDetails
        );
        bindings.insert(KeyContext::AgentView, agent_view);
        
        // Settings view keybindings
        let mut settings = HashMap::new();
        settings.insert(
            KeyCombination::new(KeyCode::Esc, KeyModifiers::empty()),
            Action::SwitchToNormalMode
        );
        bindings.insert(KeyContext::Settings, settings);
        
        // Help view keybindings
        let mut help = HashMap::new();
        help.insert(
            KeyCombination::new(KeyCode::Esc, KeyModifiers::empty()),
            Action::SwitchToNormalMode
        );
        help.insert(
            KeyCombination::new(KeyCode::Char('q'), KeyModifiers::empty()),
            Action::SwitchToNormalMode
        );
        bindings.insert(KeyContext::Help, help);
        
        Self { bindings }
    }
    
    /// Get action for a key combination in a specific context
    pub fn get_action(&self, combination: &KeyCombination, context: &KeyContext) -> Option<&Action> {
        
        // Check context-specific bindings first
        if let Some(context_bindings) = self.bindings.get(context) {
            if let Some(action) = context_bindings.get(&combination) {
                return Some(action);
            }
        }
        
        // Check global bindings
        if let Some(global_bindings) = self.bindings.get(&KeyContext::Global) {
            global_bindings.get(&combination)
        } else {
            None
        }
    }
    
    /// Add or update a key binding
    pub fn bind_key(&mut self, context: KeyContext, combination: KeyCombination, action: Action) {
        self.bindings
            .entry(context)
            .or_insert_with(HashMap::new)
            .insert(combination, action);
    }
    
    /// Remove a key binding
    pub fn unbind_key(&mut self, context: &KeyContext, combination: &KeyCombination) -> bool {
        if let Some(context_bindings) = self.bindings.get_mut(context) {
            context_bindings.remove(combination).is_some()
        } else {
            false
        }
    }
    
    /// Get all bindings for a context
    pub fn get_bindings_for_context(&self, context: &KeyContext) -> Option<&HashMap<KeyCombination, Action>> {
        self.bindings.get(context)
    }
    
    /// Get help text for keybindings in a specific context
    pub fn get_help_text(&self, context: &KeyContext) -> Vec<String> {
        let mut help_lines = Vec::new();
        
        // Add global bindings
        if let Some(global_bindings) = self.bindings.get(&KeyContext::Global) {
            help_lines.push("Global Commands:".to_string());
            for (key, action) in global_bindings {
                help_lines.push(format!("  {} - {}", key.to_string(), action.description()));
            }
            help_lines.push(String::new()); // Empty line
        }
        
        // Add context-specific bindings
        if let Some(context_bindings) = self.bindings.get(context) {
            help_lines.push(format!("{:?} Mode Commands:", context));
            for (key, action) in context_bindings {
                help_lines.push(format!("  {} - {}", key.to_string(), action.description()));
            }
        }
        
        help_lines
    }
    
    /// Load keybindings from a configuration
    pub fn load_from_config(&mut self, config: &HashMap<String, HashMap<String, String>>) {
        for (context_name, context_bindings) in config {
            if let Ok(context) = context_name.parse::<KeyContext>() {
                for (key_string, action_string) in context_bindings {
                    if let (Ok(combination), Ok(action)) = (
                        KeyCombination::from_string(key_string),
                        action_string.parse::<Action>()
                    ) {
                        self.bind_key(context.clone(), combination, action);
                    }
                }
            }
        }
    }
}

impl KeybindingManager {
    /// Create new keybinding manager with default bindings
    pub fn new() -> Self {
        Self {
            bindings: KeyBindings::default(),
        }
    }
    
    /// Get action for a key combination in a context
    pub fn get_action(&self, key: KeyCode, modifiers: KeyModifiers, context: &KeyContext) -> Option<Action> {
        let combination = KeyCombination::new(key, modifiers);
        self.bindings.get_action(&combination, context).cloned()
    }
    
    /// Add a new key binding
    pub fn bind_key(&mut self, context: KeyContext, key: KeyCode, modifiers: KeyModifiers, action: Action) {
        let combination = KeyCombination::new(key, modifiers);
        self.bindings.bind_key(context, combination, action);
    }
    
    /// Get help text for context
    pub fn get_help_text(&self, context: &KeyContext) -> Vec<String> {
        self.bindings.get_help_text(context)
    }
}

impl Default for KeybindingManager {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyCombination {
    /// Create a new key combination
    pub fn new(key: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { key, modifiers }
    }
    
    /// Create from a KeyEvent
    pub fn from_key_event(event: &KeyEvent) -> Self {
        Self {
            key: event.code,
            modifiers: event.modifiers,
        }
    }
    
    /// Parse from string representation (e.g., "Ctrl+C", "Alt+F4")
    pub fn from_string(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('+').collect();
        if parts.is_empty() {
            return Err("Empty key combination".to_string());
        }
        
        let mut modifiers = KeyModifiers::empty();
        let key_part = parts[parts.len() - 1];
        
        for part in &parts[..parts.len() - 1] {
            match part.to_lowercase().as_str() {
                "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
                "alt" => modifiers |= KeyModifiers::ALT,
                "shift" => modifiers |= KeyModifiers::SHIFT,
                "super" | "cmd" => modifiers |= KeyModifiers::SUPER,
                _ => return Err(format!("Unknown modifier: {}", part)),
            }
        }
        
        let key = match key_part.to_lowercase().as_str() {
            "enter" | "return" => KeyCode::Enter,
            "space" => KeyCode::Char(' '),
            "backspace" => KeyCode::Backspace,
            "tab" => KeyCode::Tab,
            "esc" | "escape" => KeyCode::Esc,
            "up" => KeyCode::Up,
            "down" => KeyCode::Down,
            "left" => KeyCode::Left,
            "right" => KeyCode::Right,
            "home" => KeyCode::Home,
            "end" => KeyCode::End,
            "pageup" => KeyCode::PageUp,
            "pagedown" => KeyCode::PageDown,
            "delete" | "del" => KeyCode::Delete,
            "insert" | "ins" => KeyCode::Insert,
            s if s.starts_with('f') && s.len() <= 3 => {
                if let Ok(num) = s[1..].parse::<u8>() {
                    KeyCode::F(num)
                } else {
                    return Err(format!("Invalid function key: {}", s));
                }
            }
            s if s.len() == 1 => {
                KeyCode::Char(s.chars().next().unwrap())
            }
            _ => return Err(format!("Unknown key: {}", key_part)),
        };
        
        Ok(Self { key, modifiers })
    }
    
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        let mut parts = Vec::new();
        
        if self.modifiers.contains(KeyModifiers::CONTROL) {
            parts.push("Ctrl");
        }
        if self.modifiers.contains(KeyModifiers::ALT) {
            parts.push("Alt");
        }
        if self.modifiers.contains(KeyModifiers::SHIFT) {
            parts.push("Shift");
        }
        if self.modifiers.contains(KeyModifiers::SUPER) {
            parts.push("Super");
        }
        
        let key_str = match self.key {
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::Backspace => "Backspace".to_string(),
            KeyCode::Esc => "Esc".to_string(),
            KeyCode::Up => "Up".to_string(),
            KeyCode::Down => "Down".to_string(),
            KeyCode::Left => "Left".to_string(),
            KeyCode::Right => "Right".to_string(),
            KeyCode::Home => "Home".to_string(),
            KeyCode::End => "End".to_string(),
            KeyCode::PageUp => "PageUp".to_string(),
            KeyCode::PageDown => "PageDown".to_string(),
            KeyCode::Delete => "Delete".to_string(),
            KeyCode::Insert => "Insert".to_string(),
            KeyCode::F(n) => format!("F{}", n),
            _ => format!("{:?}", self.key),
        };
        
        parts.push(&key_str);
        parts.join("+")
    }
}

impl Action {
    /// Get description for the action
    pub fn description(&self) -> &'static str {
        match self {
            Action::Quit => "Quit application",
            Action::SwitchToNormalMode => "Normal mode",
            Action::SwitchToInputMode => "Input mode",
            Action::SwitchToCommandMode => "Command mode",
            Action::SwitchToAgentView => "Agent view",
            Action::SwitchToSettingsView => "Settings",
            Action::SwitchToHelpView => "Help",
            Action::ConfirmInput => "Confirm input",
            Action::CancelInput => "Cancel input",
            Action::ClearInput => "Clear input",
            Action::DeleteChar => "Delete character",
            Action::DeleteWord => "Delete word",
            Action::MoveCursorLeft => "Move cursor left",
            Action::MoveCursorRight => "Move cursor right",
            Action::MoveCursorStart => "Move cursor to start",
            Action::MoveCursorEnd => "Move cursor to end",
            Action::ScrollUp => "Scroll up",
            Action::ScrollDown => "Scroll down",
            Action::PageUp => "Page up",
            Action::PageDown => "Page down",
            Action::ScrollToTop => "Scroll to top",
            Action::ScrollToBottom => "Scroll to bottom",
            Action::StopAllAgents => "Stop all agents",
            Action::RestartAgent => "Restart agent",
            Action::ShowAgentDetails => "Show agent details",
            Action::ClearOutput => "Clear output",
            Action::FilterByType => "Filter by type",
            Action::FilterByAgent => "Filter by agent",
            Action::SearchOutput => "Search output",
            Action::NextTheme => "Next theme",
            Action::PreviousTheme => "Previous theme",
            Action::ShowHelp => "Show help",
            Action::ToggleHelp => "Toggle help",
            Action::ShowStatus => "Show status",
            Action::ToggleTimestamps => "Toggle timestamps",
            Action::ToggleLineNumbers => "Toggle line numbers",
            Action::RefreshView => "Refresh view",
            Action::SwitchTheme => "Switch theme",
            Action::Custom(_name) => "Custom action",
        }
    }
}

impl std::str::FromStr for KeyContext {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "global" => Ok(KeyContext::Global),
            "normal" => Ok(KeyContext::Normal),
            "input" => Ok(KeyContext::Input),
            "command" => Ok(KeyContext::Command),
            "agentview" | "agent_view" => Ok(KeyContext::AgentView),
            "settings" => Ok(KeyContext::Settings),
            "help" => Ok(KeyContext::Help),
            _ => Err(format!("Unknown key context: {}", s)),
        }
    }
}

impl std::str::FromStr for Action {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "quit" => Ok(Action::Quit),
            "normal_mode" => Ok(Action::SwitchToNormalMode),
            "input_mode" => Ok(Action::SwitchToInputMode),
            "command_mode" => Ok(Action::SwitchToCommandMode),
            "agent_view" => Ok(Action::SwitchToAgentView),
            "settings" => Ok(Action::SwitchToSettingsView),
            "help" => Ok(Action::SwitchToHelpView),
            "confirm" => Ok(Action::ConfirmInput),
            "cancel" => Ok(Action::CancelInput),
            "clear" => Ok(Action::ClearInput),
            "delete_char" => Ok(Action::DeleteChar),
            "delete_word" => Ok(Action::DeleteWord),
            "scroll_up" => Ok(Action::ScrollUp),
            "scroll_down" => Ok(Action::ScrollDown),
            "page_up" => Ok(Action::PageUp),
            "page_down" => Ok(Action::PageDown),
            "next_theme" => Ok(Action::NextTheme),
            "previous_theme" => Ok(Action::PreviousTheme),
            "refresh" => Ok(Action::RefreshView),
            _ => Ok(Action::Custom(s.to_string())),
        }
    }
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self::new()
    }
}
